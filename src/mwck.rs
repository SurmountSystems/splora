// SPDX-License-Identifier: Unlicense
//! Mempool Wallet Connector Kit websocket JSON dialect.
//!
//! Speaks `track-address` / `track-addresses` / `track-scriptpubkeys` in and
//! emits `address-transactions` / `address-removed-transactions` /
//! `block-transactions` plus `multi-address-transactions` /
//! `multi-scriptpubkey-transactions` with `{mempool, confirmed, removed}`
//! per key.
//!
//! Hyper 0.14 WebSocket upgrade is wired from `rest.rs` later. This module is
//! the in-process JSON state machine. Do not HTTP self-poll.
//!
//! The `/api/v1/ws` upgrade must require [`crate::auth::Allowlist`]. After
//! upgrade, that pubkey owns the connection.

use serde_json::{Value, json};
use std::collections::{BTreeMap, HashMap, HashSet, VecDeque};
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc::UnboundedSender;

use crate::auth::Allowlist;
use crate::chain::{BlockHash, Network, Script};
use crate::new_index::{Query, compute_script_hash};

/// Safety bound on mempool tx JSON kept for later `{txid}`-only drops.
/// Populated on add and subscribe fill, not by scanning the whole mempool.
const MAX_MEMPOOL_TX_BODIES: usize = 65_536;

#[derive(Default)]
struct MempoolTxBodies {
    by_txid: HashMap<String, Value>,
    order: VecDeque<String>,
}

struct HubClient {
    state: ClientState,
    out: Option<UnboundedSender<Value>>,
}

/// One indexed height last emitted as confirmed. Used to replay orphaned
/// txs after Query has already moved to the new tip.
#[derive(Clone)]
struct TipBlock {
    hash: BlockHash,
    txs: Vec<Value>,
}

/// Per-key buckets that [`emit_multi`] fills for MWCK multi-* events.
#[derive(Clone, Debug, Default, PartialEq, Serialize)]
pub struct TxBuckets {
    pub mempool: Vec<Value>,
    pub confirmed: Vec<Value>,
    pub removed: Vec<Value>,
}

/// One websocket client's active MWCK subscriptions.
///
/// Each client has at most one active subscription of each type. A later
/// message of the same type replaces the previous one. An empty array on
/// `track-addresses` or `track-scriptpubkeys` unsubscribes that type.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ClientState {
    /// Canonical match key for the single-address subscription.
    pub track_address: Option<String>,
    /// Original `track-address` string (P2PK hex or address).
    pub track_address_original: Option<String>,
    /// Original address/pubkey -> canonical match key.
    pub track_addresses: BTreeMap<String, String>,
    /// Canonical lowercase scriptpubkey hex strings.
    pub track_scriptpubkeys: Vec<String>,
}

/// In-process MWCK hub. Reuses [`Query`] for subscribe snapshot fills; notify
/// paths take transaction JSON already loaded from mempool/chain.
pub struct MwckHub {
    query: Option<Arc<Query>>,
    clients: Mutex<BTreeMap<u64, HubClient>>,
    next_id: Mutex<u64>,
    /// Test-only current history keyed by original `track-addresses` /
    /// `track-scriptpubkeys` string. Production fills from [`Query`].
    #[cfg(test)]
    test_history: Option<BTreeMap<String, TxBuckets>>,
    /// Heights last emitted as confirmed, keyed by height. Reorg replay
    /// compares these hashes to the current [`Query`] (or test) chain.
    recent_confirmed: Mutex<BTreeMap<usize, TipBlock>>,
    /// Test-only stand-in for [`Query`] `hash_by_height` / `get_block_txs`.
    #[cfg(test)]
    test_blocks: Mutex<BTreeMap<usize, TipBlock>>,
    /// Full JSON for mempool txs this hub has seen, keyed by txid. Used when a
    /// later drop is `{txid}` only and Query no longer has the body.
    mempool_tx_bodies: Mutex<MempoolTxBodies>,
}

impl MwckHub {
    pub fn new(query: Arc<Query>) -> Arc<Self> {
        Arc::new(MwckHub {
            query: Some(query),
            clients: Mutex::new(BTreeMap::new()),
            next_id: Mutex::new(1),
            #[cfg(test)]
            test_history: None,
            recent_confirmed: Mutex::new(BTreeMap::new()),
            #[cfg(test)]
            test_blocks: Mutex::new(BTreeMap::new()),
            mempool_tx_bodies: Mutex::new(MempoolTxBodies::default()),
        })
    }

    /// Handshake-only hub. No chain [`Query`]. REST tests use this so a live
    /// hyper 0.14 upgrade does not boot the indexer.
    #[cfg(test)]
    pub fn handshake_fixture() -> Arc<Self> {
        Arc::new(MwckHub {
            query: None,
            clients: Mutex::new(BTreeMap::new()),
            next_id: Mutex::new(1),
            test_history: None,
            recent_confirmed: Mutex::new(BTreeMap::new()),
            test_blocks: Mutex::new(BTreeMap::new()),
            mempool_tx_bodies: Mutex::new(MempoolTxBodies::default()),
        })
    }

    /// Subscribe-fill fixture: known mempool/confirmed history without a live
    /// indexer. Keys are original track-addresses / track-scriptpubkeys strings.
    #[cfg(test)]
    pub fn with_history_fixture(history: BTreeMap<String, TxBuckets>) -> Arc<Self> {
        Arc::new(MwckHub {
            query: None,
            clients: Mutex::new(BTreeMap::new()),
            next_id: Mutex::new(1),
            test_history: Some(history),
            recent_confirmed: Mutex::new(BTreeMap::new()),
            test_blocks: Mutex::new(BTreeMap::new()),
            mempool_tx_bodies: Mutex::new(MempoolTxBodies::default()),
        })
    }

    pub fn query(&self) -> &Query {
        self.query
            .as_ref()
            .expect("MwckHub handshake fixture has no Query")
    }

    pub fn attach_client(&self, state: ClientState) -> u64 {
        self.attach(state, None)
    }

    /// Register a live websocket. `out` receives notify JSON and subscribe replies
    /// are written by the rest.rs socket loop.
    pub fn register_socket(&self, out: UnboundedSender<Value>) -> u64 {
        self.attach(ClientState::default(), Some(out))
    }

    fn attach(&self, state: ClientState, out: Option<UnboundedSender<Value>>) -> u64 {
        let mut next = self.next_id.lock().expect("mwck next_id");
        let id = *next;
        *next += 1;
        self.clients
            .lock()
            .expect("mwck clients")
            .insert(id, HubClient { state, out });
        id
    }

    pub fn set_client_state(&self, id: u64, state: ClientState) {
        if let Some(client) = self.clients.lock().expect("mwck clients").get_mut(&id) {
            client.state = state;
        }
    }

    pub fn detach_client(&self, id: u64) {
        self.clients.lock().expect("mwck clients").remove(&id);
    }

    pub fn notify_mempool(&self, added: &[Value], removed: &[Value]) {
        self.remember_mempool_bodies(added);
        let removed = self.enrich_removed(removed);
        let pending: Vec<(Option<UnboundedSender<Value>>, Value)> = {
            let clients = self.clients.lock().expect("mwck clients");
            clients
                .values()
                .filter_map(|c| {
                    c.state
                        .on_mempool(added, &removed)
                        .map(|ev| (c.out.clone(), ev))
                })
                .collect()
        };
        for (out, ev) in pending {
            if let Some(tx) = out {
                let _ = tx.send(ev);
            }
        }
    }

    pub fn notify_block(&self, confirmed: &[Value]) {
        let pending: Vec<(Option<UnboundedSender<Value>>, Value)> = {
            let clients = self.clients.lock().expect("mwck clients");
            clients
                .values()
                .filter_map(|c| c.state.on_block(confirmed).map(|ev| (c.out.clone(), ev)))
                .collect()
        };
        for (out, ev) in pending {
            if let Some(tx) = out {
                let _ = tx.send(ev);
            }
        }
    }

    /// Confirmed txs for each new height after the previous tip. One notify
    /// with every matching tx. Does not replay orphaned blocks on a reorg.
    pub fn notify_blocks(&self, blocks: &[Vec<Value>]) {
        let confirmed: Vec<Value> = blocks.iter().flatten().cloned().collect();
        if !confirmed.is_empty() {
            self.notify_block(&confirmed);
        }
    }

    /// Walk chain heights via in-process [`Query`] (or a test chain).
    /// Orphaned heights are emitted as `removed`, then the new branch as
    /// `confirmed`. A lower or equal new height is not a no-op when the tip
    /// hash changed. Does not HTTP self-poll.
    pub fn notify_new_tip(&self, prev_height: usize, new_height: usize) {
        if !self.has_block_source() {
            return;
        }

        let mut orphaned = Vec::new();
        let mut replacements = BTreeMap::new();
        {
            let mut recent = self.recent_confirmed.lock().expect("mwck recent");
            let stale: Vec<usize> = recent
                .iter()
                .filter_map(|(&h, old)| match self.block_at(h) {
                    Some(cur) if cur.hash == old.hash => None,
                    _ => Some(h),
                })
                .collect();
            for h in stale {
                if let Some(old) = recent.remove(&h) {
                    orphaned.extend(old.txs);
                }
                if let Some(cur) = self.block_at(h) {
                    replacements.insert(h, cur);
                }
            }
        }

        let mut forward = Vec::new();
        if new_height > prev_height {
            let recent = self.recent_confirmed.lock().expect("mwck recent");
            for h in prev_height.saturating_add(1)..=new_height {
                if recent.contains_key(&h) || replacements.contains_key(&h) {
                    continue;
                }
                if let Some(cur) = self.block_at(h) {
                    forward.push((h, cur));
                }
            }
        }

        if !orphaned.is_empty() {
            self.notify_mempool(&[], &orphaned);
        }

        let mut confirmed_blocks = Vec::new();
        let mut to_store = Vec::new();
        for (h, block) in replacements {
            if !block.txs.is_empty() {
                confirmed_blocks.push(block.txs.clone());
            }
            to_store.push((h, block));
        }
        for (h, block) in forward {
            if !block.txs.is_empty() {
                confirmed_blocks.push(block.txs.clone());
            }
            to_store.push((h, block));
        }
        self.notify_blocks(&confirmed_blocks);

        let mut recent = self.recent_confirmed.lock().expect("mwck recent");
        for (h, block) in to_store {
            recent.insert(h, block);
        }
        let keep_from = new_height.saturating_sub(16);
        recent.retain(|&h, _| h >= keep_from);
    }

    fn has_block_source(&self) -> bool {
        if self.query.is_some() {
            return true;
        }
        #[cfg(test)]
        {
            !self
                .test_blocks
                .lock()
                .expect("mwck test_blocks")
                .is_empty()
        }
        #[cfg(not(test))]
        {
            false
        }
    }

    fn block_at(&self, height: usize) -> Option<TipBlock> {
        #[cfg(test)]
        {
            let test = self.test_blocks.lock().expect("mwck test_blocks");
            if !test.is_empty() {
                return test.get(&height).cloned();
            }
        }
        let query = self.query.as_ref()?;
        let chain = query.chain();
        let hash = chain.hash_by_height(height)?;
        let txs = match chain.get_block_txs(&hash) {
            Some(txs) => {
                let blockid = chain.blockid_by_hash(&hash);
                let pairs = txs.into_iter().map(|tx| (tx, blockid.clone())).collect();
                crate::rest::transactions_as_json(pairs, query, query.config())
            }
            None => Vec::new(),
        };
        Some(TipBlock { hash, txs })
    }

    #[cfg(test)]
    pub fn set_test_block(&self, height: usize, hash: BlockHash, txs: Vec<Value>) {
        self.test_blocks
            .lock()
            .expect("mwck test_blocks")
            .insert(height, TipBlock { hash, txs });
    }

    #[cfg(test)]
    pub fn remember_confirmed_for_test(&self, height: usize, hash: BlockHash, txs: Vec<Value>) {
        self.recent_confirmed
            .lock()
            .expect("mwck recent")
            .insert(height, TipBlock { hash, txs });
    }

    /// Apply one client JSON message. REST must not call this unless the
    /// upgrade already proved `pubkey` is on `allow`. Hyper 0.14 upgrade
    /// stays in `rest.rs`.
    ///
    /// The first `multi-address-transactions` / `multi-scriptpubkey-transactions`
    /// after track-* is filled from in-process [`Query`] (mempool + chain), not
    /// left empty until a later notify.
    pub async fn handle_socket(
        &self,
        allow: &Allowlist,
        pubkey: &[u8; 32],
        state: &mut ClientState,
        msg: Value,
    ) -> Option<Value> {
        let reply = handle_authorized_json(allow, pubkey, state, msg)?;
        Some(self.fill_subscribe_snapshot(state, reply))
    }

    fn fill_subscribe_snapshot(&self, state: &ClientState, mut reply: Value) -> Value {
        if let Some(obj) = reply
            .get_mut("multi-address-transactions")
            .and_then(|v| v.as_object_mut())
        {
            for (orig, canon) in &state.track_addresses {
                let buckets = self.snapshot_for(orig, Some(canon));
                self.remember_mempool_bodies(&buckets.mempool);
                if let Ok(v) = serde_json::to_value(&buckets) {
                    obj.insert(orig.clone(), v);
                }
            }
        }
        if let Some(obj) = reply
            .get_mut("multi-scriptpubkey-transactions")
            .and_then(|v| v.as_object_mut())
        {
            for spk in &state.track_scriptpubkeys {
                let buckets = self.snapshot_for(spk, None);
                self.remember_mempool_bodies(&buckets.mempool);
                if let Ok(v) = serde_json::to_value(&buckets) {
                    obj.insert(spk.clone(), v);
                }
            }
        }
        reply
    }

    fn snapshot_for(&self, orig: &str, canon: Option<&str>) -> TxBuckets {
        #[cfg(test)]
        if let Some(map) = &self.test_history {
            if let Some(b) = map.get(orig) {
                return b.clone();
            }
            if let Some(c) = canon {
                if let Some(b) = map.get(c) {
                    return b.clone();
                }
            }
        }
        let Some(query) = self.query.as_ref() else {
            return TxBuckets::default();
        };
        match canon {
            Some(canon) => snapshot_from_address(query, orig, canon),
            None => snapshot_from_spk(query, orig),
        }
    }

    fn remember_mempool_bodies(&self, txs: &[Value]) {
        let mut cache = self.mempool_tx_bodies.lock().expect("mwck mempool bodies");
        for tx in txs {
            remember_mempool_body(&mut cache, tx);
        }
    }

    fn enrich_removed(&self, removed: &[Value]) -> Vec<Value> {
        let mut available = BTreeMap::new();
        {
            let cache = self.mempool_tx_bodies.lock().expect("mwck mempool bodies");
            for v in removed {
                if is_full_tx_object(v) {
                    continue;
                }
                let Some(txid_s) = v.get("txid").and_then(|t| t.as_str()) else {
                    continue;
                };
                if let Some(full) = cache.by_txid.get(txid_s) {
                    available.insert(txid_s.to_string(), full.clone());
                }
            }
        }
        #[cfg(test)]
        {
            let history = self.available_tx_bodies();
            for v in removed {
                if is_full_tx_object(v) {
                    continue;
                }
                let Some(txid_s) = v.get("txid").and_then(|t| t.as_str()) else {
                    continue;
                };
                if available.contains_key(txid_s) {
                    continue;
                }
                if let Some(full) = history.get(txid_s) {
                    available.insert(txid_s.to_string(), full.clone());
                }
            }
        }
        if let Some(query) = &self.query {
            for v in removed {
                if is_full_tx_object(v) {
                    continue;
                }
                let Some(txid_s) = v.get("txid").and_then(|t| t.as_str()) else {
                    continue;
                };
                if available.contains_key(txid_s) {
                    continue;
                }
                if let Ok(txid) = txid_s.parse::<crate::chain::Txid>()
                    && let Some(tx) = query.lookup_txn(&txid)
                    && let Some(full) =
                        crate::rest::transactions_as_json(vec![(tx, None)], query, query.config())
                            .into_iter()
                            .next()
                {
                    available.insert(txid_s.to_string(), full);
                }
            }
        }
        let out = prefer_full_removed(removed, &available);
        let mut cache = self.mempool_tx_bodies.lock().expect("mwck mempool bodies");
        for v in removed {
            if let Some(txid) = v.get("txid").and_then(|t| t.as_str()) {
                cache.by_txid.remove(txid);
            }
        }
        out
    }

    #[cfg(test)]
    fn available_tx_bodies(&self) -> BTreeMap<String, Value> {
        let mut map = BTreeMap::new();
        if let Some(history) = &self.test_history {
            for buckets in history.values() {
                for tx in buckets
                    .mempool
                    .iter()
                    .chain(buckets.confirmed.iter())
                    .chain(buckets.removed.iter())
                {
                    if let Some(txid) = tx.get("txid").and_then(|t| t.as_str()) {
                        map.entry(txid.to_string()).or_insert_with(|| tx.clone());
                    }
                }
            }
        }
        map
    }
}

/// Sync helper: same gate as [`MwckHub::handle_socket`] without a `Query`.
pub fn handle_authorized_json(
    allow: &Allowlist,
    pubkey: &[u8; 32],
    state: &mut ClientState,
    msg: Value,
) -> Option<Value> {
    if !allow.contains(pubkey) {
        return None;
    }
    handle_client_json(state, msg)
}

fn snapshot_from_address(query: &Query, orig: &str, canon: &str) -> TxBuckets {
    match script_from_address_key(orig, canon, query.network()) {
        Some(script) => snapshot_from_script(query, &script),
        None => TxBuckets::default(),
    }
}

fn snapshot_from_spk(query: &Query, spk: &str) -> TxBuckets {
    match hex::decode(spk)
        .ok()
        .filter(|b| !b.is_empty())
        .map(Script::from)
    {
        Some(script) => snapshot_from_script(query, &script),
        None => TxBuckets::default(),
    }
}

fn script_from_address_key(orig: &str, canon: &str, network: Network) -> Option<Script> {
    if is_hex_len(orig, 66) || is_hex_len(orig, 130) {
        return hex::decode(canon)
            .ok()
            .filter(|b| !b.is_empty())
            .map(Script::from);
    }
    parse_address_script(orig, network)
}

fn parse_address_script(addr: &str, network: Network) -> Option<Script> {
    #[cfg(not(feature = "liquid"))]
    {
        use bitcoin::address::NetworkUnchecked;
        let unchecked: bitcoin::Address<NetworkUnchecked> = addr.parse().ok()?;
        let bnetwork = bitcoin::Network::from(network);
        let testnet_family = [
            bitcoin::Network::Testnet,
            bitcoin::Network::Regtest,
            bitcoin::Network::Signet,
            bitcoin::Network::Testnet4,
        ];
        if testnet_family.contains(&bnetwork) {
            testnet_family
                .iter()
                .find_map(|&net| unchecked.clone().require_network(net).ok())
        } else {
            unchecked.require_network(bnetwork).ok()
        }
        .map(|a| a.script_pubkey())
    }
    #[cfg(feature = "liquid")]
    {
        crate::chain::address::Address::parse_with_params(addr, network.address_params())
            .ok()
            .map(|a| a.script_pubkey())
    }
}

fn snapshot_from_script(query: &Query, script: &Script) -> TxBuckets {
    let scripthash = compute_script_hash(script);
    let limit = query.config().rest_default_max_mempool_txs;

    let mempool_pairs: Vec<_> = query
        .mempool()
        .history(&scripthash[..], None, limit)
        .into_iter()
        .map(|tx| (tx, None))
        .collect();
    let mempool = crate::rest::transactions_as_json(mempool_pairs, query, query.config());

    let remaining = limit.saturating_sub(mempool.len());
    let confirmed_pairs: Vec<_> = query
        .chain()
        .history_txids(&scripthash[..], remaining)
        .into_iter()
        .filter_map(|(txid, blockid)| query.lookup_txn(&txid).map(|tx| (tx, Some(blockid))))
        .collect();
    let confirmed = crate::rest::transactions_as_json(confirmed_pairs, query, query.config());

    TxBuckets {
        mempool,
        confirmed,
        removed: vec![],
    }
}

/// Canonical MWCK match key: lowercase bech32, P2PK hex as a script.
///
/// `02|03` + 32 bytes -> `21{pk}ac`. `04` + 64 bytes -> `41{pk}ac`.
pub fn canonical_track_key(raw: &str) -> Option<String> {
    let s = raw.trim();
    if s.is_empty() {
        return None;
    }
    if is_hex_len(s, 66) {
        let prefix = &s[..2];
        if prefix.eq_ignore_ascii_case("02") || prefix.eq_ignore_ascii_case("03") {
            return Some(format!("21{}ac", s.to_ascii_lowercase()));
        }
    }
    if is_hex_len(s, 130) && s[..2].eq_ignore_ascii_case("04") {
        return Some(format!("41{}ac", s.to_ascii_lowercase()));
    }
    if is_bech32_upper(s) {
        return Some(s.to_ascii_lowercase());
    }
    if is_bech32_lower(s) || is_base58_addr(s) {
        return Some(s.to_string());
    }
    None
}

/// Build a `multi-address-transactions` or `multi-scriptpubkey-transactions`
/// object with `{mempool, confirmed, removed}` per key.
pub fn emit_multi(event_key: &str, per_key: &BTreeMap<String, TxBuckets>) -> Value {
    json!({ event_key: per_key })
}

/// Prefer a full transaction JSON in `removed`. `{txid}` only when no body
/// is available.
pub fn prefer_full_removed(removed: &[Value], available: &BTreeMap<String, Value>) -> Vec<Value> {
    removed
        .iter()
        .map(|v| {
            if is_full_tx_object(v) {
                return v.clone();
            }
            let Some(txid) = v.get("txid").and_then(|t| t.as_str()) else {
                return v.clone();
            };
            match available.get(txid) {
                Some(full) if is_full_tx_object(full) => full.clone(),
                _ => json!({ "txid": txid }),
            }
        })
        .collect()
}

fn is_full_tx_object(v: &Value) -> bool {
    v.get("vin").is_some() || v.get("vout").is_some()
}

fn remember_mempool_body(cache: &mut MempoolTxBodies, tx: &Value) {
    if !is_full_tx_object(tx) {
        return;
    }
    let Some(txid) = tx.get("txid").and_then(|t| t.as_str()) else {
        return;
    };
    if cache.by_txid.contains_key(txid) {
        cache.by_txid.insert(txid.to_string(), tx.clone());
        return;
    }
    while cache.by_txid.len() >= MAX_MEMPOOL_TX_BODIES {
        let Some(old) = cache.order.pop_front() else {
            break;
        };
        cache.by_txid.remove(&old);
    }
    cache.order.push_back(txid.to_string());
    cache.by_txid.insert(txid.to_string(), tx.clone());
}

/// Pure JSON state machine. No network. Tests call this directly.
pub fn handle_client_json(state: &mut ClientState, msg: Value) -> Option<Value> {
    if !msg.is_object() {
        return None;
    }
    let mut out = serde_json::Map::new();

    if let Some(v) = msg.get("track-address") {
        apply_track_address(state, v);
    }

    if let Some(v) = msg.get("track-addresses")
        && let Some(map) = apply_track_addresses(state, v)
    {
        let mut buckets = BTreeMap::new();
        for orig in map.keys() {
            buckets.insert(orig.clone(), TxBuckets::default());
        }
        if let Value::Object(obj) = emit_multi("multi-address-transactions", &buckets) {
            out.extend(obj);
        }
    }

    if let Some(v) = msg.get("track-scriptpubkeys")
        && let Some(spks) = apply_track_scriptpubkeys(state, v)
    {
        let mut buckets = BTreeMap::new();
        for spk in &spks {
            buckets.insert(spk.clone(), TxBuckets::default());
        }
        if let Value::Object(obj) = emit_multi("multi-scriptpubkey-transactions", &buckets) {
            out.extend(obj);
        }
    }

    if out.is_empty() {
        None
    } else {
        Some(Value::Object(out))
    }
}

impl ClientState {
    /// Mempool delta for this client's subscriptions.
    pub fn on_mempool(&self, added: &[Value], removed: &[Value]) -> Option<Value> {
        let mut out = serde_json::Map::new();

        if let Some(canon) = self.track_address.as_ref() {
            let added_hit: Vec<Value> = added
                .iter()
                .filter(|tx| tx_matches(tx, canon))
                .cloned()
                .collect();
            let removed_hit: Vec<Value> = removed
                .iter()
                .filter(|tx| tx_matches(tx, canon))
                .cloned()
                .collect();
            if !added_hit.is_empty() {
                out.insert("address-transactions".into(), Value::Array(added_hit));
            }
            if !removed_hit.is_empty() {
                out.insert(
                    "address-removed-transactions".into(),
                    Value::Array(removed_hit),
                );
            }
        }

        if !self.track_addresses.is_empty() {
            let mut per_key = BTreeMap::new();
            for (orig, canon) in &self.track_addresses {
                let b = TxBuckets {
                    mempool: added
                        .iter()
                        .filter(|tx| tx_matches(tx, canon))
                        .cloned()
                        .collect(),
                    removed: removed
                        .iter()
                        .filter(|tx| tx_matches(tx, canon))
                        .cloned()
                        .collect(),
                    ..Default::default()
                };
                if !b.mempool.is_empty() || !b.removed.is_empty() {
                    per_key.insert(orig.clone(), b);
                }
            }
            if !per_key.is_empty()
                && let Value::Object(obj) = emit_multi("multi-address-transactions", &per_key)
            {
                out.extend(obj);
            }
        }

        if !self.track_scriptpubkeys.is_empty() {
            let mut per_key = BTreeMap::new();
            for spk in &self.track_scriptpubkeys {
                let b = TxBuckets {
                    mempool: added
                        .iter()
                        .filter(|tx| tx_matches(tx, spk))
                        .cloned()
                        .collect(),
                    removed: removed
                        .iter()
                        .filter(|tx| tx_matches(tx, spk))
                        .cloned()
                        .collect(),
                    ..Default::default()
                };
                if !b.mempool.is_empty() || !b.removed.is_empty() {
                    per_key.insert(spk.clone(), b);
                }
            }
            if !per_key.is_empty()
                && let Value::Object(obj) = emit_multi("multi-scriptpubkey-transactions", &per_key)
            {
                out.extend(obj);
            }
        }

        if out.is_empty() {
            None
        } else {
            Some(Value::Object(out))
        }
    }

    /// Confirmed-block delta for this client's subscriptions.
    pub fn on_block(&self, confirmed: &[Value]) -> Option<Value> {
        let mut out = serde_json::Map::new();

        if let Some(canon) = self.track_address.as_ref() {
            let hit: Vec<Value> = confirmed
                .iter()
                .filter(|tx| tx_matches(tx, canon))
                .cloned()
                .collect();
            if !hit.is_empty() {
                out.insert("block-transactions".into(), Value::Array(hit));
            }
        }

        if !self.track_addresses.is_empty() {
            let mut per_key = BTreeMap::new();
            for (orig, canon) in &self.track_addresses {
                let b = TxBuckets {
                    confirmed: confirmed
                        .iter()
                        .filter(|tx| tx_matches(tx, canon))
                        .cloned()
                        .collect(),
                    ..Default::default()
                };
                if !b.confirmed.is_empty() {
                    per_key.insert(orig.clone(), b);
                }
            }
            if !per_key.is_empty()
                && let Value::Object(obj) = emit_multi("multi-address-transactions", &per_key)
            {
                out.extend(obj);
            }
        }

        if !self.track_scriptpubkeys.is_empty() {
            let mut per_key = BTreeMap::new();
            for spk in &self.track_scriptpubkeys {
                let b = TxBuckets {
                    confirmed: confirmed
                        .iter()
                        .filter(|tx| tx_matches(tx, spk))
                        .cloned()
                        .collect(),
                    ..Default::default()
                };
                if !b.confirmed.is_empty() {
                    per_key.insert(spk.clone(), b);
                }
            }
            if !per_key.is_empty()
                && let Value::Object(obj) = emit_multi("multi-scriptpubkey-transactions", &per_key)
            {
                out.extend(obj);
            }
        }

        if out.is_empty() {
            None
        } else {
            Some(Value::Object(out))
        }
    }
}

fn apply_track_address(state: &mut ClientState, v: &Value) {
    let s = match v.as_str() {
        Some(s) => s,
        None => {
            state.track_address = None;
            state.track_address_original = None;
            return;
        }
    };
    match canonical_track_key(s) {
        Some(canon) => {
            state.track_address_original = Some(s.to_string());
            state.track_address = Some(canon);
        }
        None => {
            state.track_address = None;
            state.track_address_original = None;
        }
    }
}

fn apply_track_addresses(state: &mut ClientState, v: &Value) -> Option<BTreeMap<String, String>> {
    let arr = match v.as_array() {
        Some(arr) => arr,
        None => {
            state.track_addresses.clear();
            return None;
        }
    };
    if arr.is_empty() {
        state.track_addresses.clear();
        return None;
    }
    let mut map = BTreeMap::new();
    for item in arr {
        if let Some(s) = item.as_str()
            && let Some(canon) = canonical_track_key(s)
        {
            map.insert(s.to_string(), canon);
        }
    }
    if map.is_empty() {
        state.track_addresses.clear();
        None
    } else {
        state.track_addresses = map.clone();
        Some(map)
    }
}

fn apply_track_scriptpubkeys(state: &mut ClientState, v: &Value) -> Option<Vec<String>> {
    let arr = match v.as_array() {
        Some(arr) => arr,
        None => {
            state.track_scriptpubkeys.clear();
            return None;
        }
    };
    if arr.is_empty() {
        state.track_scriptpubkeys.clear();
        return None;
    }
    let mut spks = Vec::new();
    let mut seen = HashSet::new();
    for item in arr {
        if let Some(s) = item.as_str()
            && is_hex(s)
        {
            let lower = s.to_ascii_lowercase();
            if seen.insert(lower.clone()) {
                spks.push(lower);
            }
        }
    }
    if spks.is_empty() {
        state.track_scriptpubkeys.clear();
        None
    } else {
        state.track_scriptpubkeys = spks.clone();
        Some(spks)
    }
}

fn tx_matches(tx: &Value, key: &str) -> bool {
    let (addresses, spks) = tx_touch_set(tx);
    addresses.contains(key) || spks.contains(key)
}

fn tx_touch_set(tx: &Value) -> (HashSet<String>, HashSet<String>) {
    let mut addresses = HashSet::new();
    let mut spks = HashSet::new();
    for side in &["vin", "vout"] {
        let Some(arr) = tx.get(*side).and_then(|v| v.as_array()) else {
            continue;
        };
        for el in arr {
            let prev = if *side == "vin" {
                el.get("prevout").unwrap_or(el)
            } else {
                el
            };
            if let Some(a) = prev.get("scriptpubkey_address").and_then(|v| v.as_str()) {
                addresses.insert(a.to_string());
                if let Some(canon) = canonical_track_key(a) {
                    addresses.insert(canon);
                }
            }
            if let Some(s) = prev.get("scriptpubkey").and_then(|v| v.as_str()) {
                spks.insert(s.to_ascii_lowercase());
            }
        }
    }
    (addresses, spks)
}

fn is_hex(s: &str) -> bool {
    !s.is_empty() && s.bytes().all(|b| b.is_ascii_hexdigit())
}

fn is_hex_len(s: &str, n: usize) -> bool {
    s.len() == n && s.bytes().all(|b| b.is_ascii_hexdigit())
}

fn is_base58_char(b: u8) -> bool {
    matches!(b,
        b'1'..=b'9' | b'A'..=b'H' | b'J'..=b'N' | b'P'..=b'Z' | b'a'..=b'k' | b'm'..=b'z'
    )
}

fn is_base58_addr(s: &str) -> bool {
    let n = s.len();
    if n != 80 && !(26..=35).contains(&n) {
        return false;
    }
    s.bytes().all(is_base58_char)
}

fn is_bech32_hrp_lower(b: u8) -> bool {
    b.is_ascii_lowercase()
}

fn is_bech32_hrp_upper(b: u8) -> bool {
    b.is_ascii_uppercase()
}

fn is_bech32_data_lower(b: u8) -> bool {
    matches!(b, b'a'..=b'h' | b'j' | b'k' | b'm' | b'n' | b'p'..=b'z' | b'0' | b'2'..=b'9')
}

fn is_bech32_data_upper(b: u8) -> bool {
    matches!(b, b'A'..=b'H' | b'J' | b'K' | b'M' | b'N' | b'P'..=b'Z' | b'0' | b'2'..=b'9')
}

fn is_bech32_lower(s: &str) -> bool {
    bech32_shape(s, is_bech32_hrp_lower, is_bech32_data_lower)
}

fn is_bech32_upper(s: &str) -> bool {
    bech32_shape(s, is_bech32_hrp_upper, is_bech32_data_upper)
}

fn bech32_shape(s: &str, hrp: fn(u8) -> bool, data: fn(u8) -> bool) -> bool {
    let bytes = s.as_bytes();
    let sep = match bytes.iter().position(|&b| b == b'1') {
        Some(i) => i,
        None => return false,
    };
    if !(2..=5).contains(&sep) {
        return false;
    }
    let rest = bytes.len() - sep - 1;
    if !(8..=100).contains(&rest) {
        return false;
    }
    bytes[..sep].iter().all(|&b| hrp(b)) && bytes[sep + 1..].iter().all(|&b| data(b))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn genesis_addr() -> &'static str {
        "1A1zP1eP5QGefi2DMPTfTL5SLmv7DivfNa"
    }

    fn sample_spk() -> &'static str {
        "0014751e76e8199196d454941c45d1b3a323f1433bd6"
    }

    /// Named contract: `track-addresses` in yields `multi-address-transactions`
    /// with mempool / confirmed / removed keys that emit_multi fills.
    #[test]
    fn handle_client_json_track_addresses_emits_multi_buckets() {
        let mut state = ClientState::default();
        let msg = json!({
            "track-addresses": [genesis_addr(), "bc1qw508d6qejxtdg4y5r3zarvary0c5xw7kv8f3t4"]
        });
        let out = handle_client_json(&mut state, msg).expect("subscribe reply");
        let multi = out
            .get("multi-address-transactions")
            .expect("multi-address-transactions")
            .as_object()
            .expect("object");
        let first = multi.get(genesis_addr()).expect("genesis key");
        assert!(first.get("mempool").unwrap().is_array());
        assert!(first.get("confirmed").unwrap().is_array());
        assert!(first.get("removed").unwrap().is_array());
        let second = multi
            .get("bc1qw508d6qejxtdg4y5r3zarvary0c5xw7kv8f3t4")
            .expect("bech32 key");
        assert!(second.get("mempool").unwrap().is_array());
        assert!(second.get("confirmed").unwrap().is_array());
        assert!(second.get("removed").unwrap().is_array());
        assert_eq!(state.track_addresses.len(), 2);
    }

    #[test]
    fn handle_client_json_track_addresses_empty_unsubscribes() {
        let mut state = ClientState::default();
        let _ = handle_client_json(&mut state, json!({ "track-addresses": [genesis_addr()] }));
        assert!(!state.track_addresses.is_empty());
        let out = handle_client_json(&mut state, json!({ "track-addresses": [] }));
        assert!(out.is_none());
        assert!(state.track_addresses.is_empty());
    }

    #[test]
    fn handle_client_json_track_addresses_replaces_prior() {
        let mut state = ClientState::default();
        let _ = handle_client_json(&mut state, json!({ "track-addresses": [genesis_addr()] }));
        let out = handle_client_json(
            &mut state,
            json!({ "track-addresses": ["bc1qw508d6qejxtdg4y5r3zarvary0c5xw7kv8f3t4"] }),
        )
        .unwrap();
        let multi = out["multi-address-transactions"].as_object().unwrap();
        assert!(!multi.contains_key(genesis_addr()));
        assert!(multi.contains_key("bc1qw508d6qejxtdg4y5r3zarvary0c5xw7kv8f3t4"));
        assert_eq!(state.track_addresses.len(), 1);
    }

    #[test]
    fn handle_client_json_track_scriptpubkeys_emits_multi_buckets() {
        let mut state = ClientState::default();
        let msg = json!({ "track-scriptpubkeys": [sample_spk()] });
        let out = handle_client_json(&mut state, msg).expect("spk subscribe");
        let multi = out
            .get("multi-scriptpubkey-transactions")
            .expect("multi-scriptpubkey-transactions")
            .as_object()
            .unwrap();
        let buckets = multi.get(sample_spk()).expect("spk key");
        assert!(buckets.get("mempool").unwrap().is_array());
        assert!(buckets.get("confirmed").unwrap().is_array());
        assert!(buckets.get("removed").unwrap().is_array());
    }

    #[test]
    fn p2pk_compressed_pubkey_canonicalizes_to_p2pk_script() {
        let pk = "02".to_string() + &"ab".repeat(32);
        let canon = canonical_track_key(&pk).expect("compressed p2pk");
        assert_eq!(canon, format!("21{}ac", pk));
        let mut state = ClientState::default();
        let out = handle_client_json(&mut state, json!({ "track-addresses": [pk] })).unwrap();
        assert!(
            out["multi-address-transactions"]
                .as_object()
                .unwrap()
                .contains_key(&pk)
        );
        assert_eq!(state.track_addresses.get(&pk).unwrap(), &canon);
    }

    #[test]
    fn p2pk_uncompressed_pubkey_canonicalizes_to_p2pk_script() {
        let pk = "04".to_string() + &"cd".repeat(64);
        let canon = canonical_track_key(&pk).expect("uncompressed p2pk");
        assert_eq!(canon, format!("41{}ac", pk));
    }

    #[test]
    fn track_address_is_independent_of_track_addresses() {
        let mut state = ClientState::default();
        let _ = handle_client_json(
            &mut state,
            json!({
                "track-address": genesis_addr(),
                "track-addresses": ["bc1qw508d6qejxtdg4y5r3zarvary0c5xw7kv8f3t4"],
            }),
        );
        assert_eq!(
            state.track_address_original.as_deref(),
            Some(genesis_addr())
        );
        assert_eq!(state.track_addresses.len(), 1);
    }

    #[test]
    fn emit_multi_and_on_mempool_fill_matching_keys() {
        let mut state = ClientState::default();
        let _ = handle_client_json(&mut state, json!({ "track-addresses": [genesis_addr()] }));
        let tx = json!({
            "txid": "aa".repeat(32),
            "vin": [],
            "vout": [{
                "scriptpubkey": "76a91462e907b15cbf27d5425399ebf6f0fb50ebb88f1888ac",
                "scriptpubkey_address": genesis_addr(),
            }]
        });
        let ev = state.on_mempool(&[tx.clone()], &[]).expect("mempool event");
        let buckets = &ev["multi-address-transactions"][genesis_addr()];
        assert_eq!(buckets["mempool"].as_array().unwrap().len(), 1);
        assert!(buckets["confirmed"].as_array().unwrap().is_empty());
        assert!(buckets["removed"].as_array().unwrap().is_empty());

        let ev = state.on_block(&[tx]).expect("block event");
        let buckets = &ev["multi-address-transactions"][genesis_addr()];
        assert!(buckets["mempool"].as_array().unwrap().is_empty());
        assert_eq!(buckets["confirmed"].as_array().unwrap().len(), 1);
        assert!(buckets["removed"].as_array().unwrap().is_empty());
    }

    #[test]
    fn handle_socket_requires_allowlist() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("allowlist");
        std::fs::write(&path, "").unwrap();
        let allow = Allowlist::load(&path).unwrap();
        let pk = [0u8; 32];
        let mut state = ClientState::default();
        let msg = json!({ "track-addresses": [genesis_addr()] });
        assert!(handle_authorized_json(&allow, &pk, &mut state, msg.clone()).is_none());
        assert!(state.track_addresses.is_empty());
        assert!(handle_client_json(&mut state, msg).is_some());
    }

    fn listed_allow() -> (tempfile::TempDir, Arc<Allowlist>, [u8; 32]) {
        use nostr::nips::nip19::ToBech32;
        let keys = nostr::key::Keys::generate();
        let pk = *keys.public_key().as_bytes();
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("allowlist");
        std::fs::write(
            &path,
            format!("{}\n", keys.public_key().to_bech32().unwrap()),
        )
        .unwrap();
        let allow = Allowlist::load(&path).unwrap();
        (dir, allow, pk)
    }

    fn sample_tx_for(addr: &str) -> Value {
        json!({
            "txid": "aa".repeat(32),
            "vin": [],
            "vout": [{
                "scriptpubkey": "76a91462e907b15cbf27d5425399ebf6f0fb50ebb88f1888ac",
                "scriptpubkey_address": addr,
            }]
        })
    }

    /// Named contract: `track-addresses` against a fixture with known history
    /// yields non-empty mempool and/or confirmed. Empty arrays until a later
    /// notify is not the only path.
    #[tokio::test]
    async fn handle_socket_track_addresses_fills_known_history() {
        let (_dir, allow, pk) = listed_allow();
        let mut history = BTreeMap::new();
        history.insert(
            genesis_addr().to_string(),
            TxBuckets {
                mempool: vec![sample_tx_for(genesis_addr())],
                confirmed: vec![],
                removed: vec![],
            },
        );
        let hub = MwckHub::with_history_fixture(history);
        let mut state = ClientState::default();
        let out = hub
            .handle_socket(
                &allow,
                &pk,
                &mut state,
                json!({ "track-addresses": [genesis_addr()] }),
            )
            .await
            .expect("subscribe reply");
        let buckets = &out["multi-address-transactions"][genesis_addr()];
        let mempool = buckets["mempool"].as_array().expect("mempool");
        let confirmed = buckets["confirmed"].as_array().expect("confirmed");
        assert!(
            !mempool.is_empty() || !confirmed.is_empty(),
            "subscribe must include current history, not only empty arrays until notify"
        );
        assert_eq!(mempool.len(), 1);
        assert!(confirmed.is_empty());
    }

    #[tokio::test]
    async fn handle_socket_track_scriptpubkeys_fills_known_history() {
        let (_dir, allow, pk) = listed_allow();
        let mut history = BTreeMap::new();
        history.insert(
            sample_spk().to_string(),
            TxBuckets {
                mempool: vec![],
                confirmed: vec![json!({
                    "txid": "bb".repeat(32),
                    "vin": [],
                    "vout": [{ "scriptpubkey": sample_spk() }],
                })],
                removed: vec![],
            },
        );
        let hub = MwckHub::with_history_fixture(history);
        let mut state = ClientState::default();
        let out = hub
            .handle_socket(
                &allow,
                &pk,
                &mut state,
                json!({ "track-scriptpubkeys": [sample_spk()] }),
            )
            .await
            .expect("spk subscribe");
        let buckets = &out["multi-scriptpubkey-transactions"][sample_spk()];
        let confirmed = buckets["confirmed"].as_array().expect("confirmed");
        assert!(
            !confirmed.is_empty(),
            "spk subscribe must include current confirmed history"
        );
        assert_eq!(confirmed.len(), 1);
    }

    /// Named contract: dropped mempool txs keep a full object in `removed`
    /// when that body is still available. `{txid}` only when it is not.
    #[test]
    fn prefer_full_removed_uses_available_body_not_txid_stub() {
        let txid = "aa".repeat(32);
        let full = sample_tx_for(genesis_addr());
        let mut available = BTreeMap::new();
        available.insert(txid.clone(), full.clone());
        let stubs = vec![json!({ "txid": txid })];
        let out = prefer_full_removed(&stubs, &available);
        assert_eq!(out.len(), 1);
        assert!(
            out[0].get("vout").is_some(),
            "removed must keep vin/vout when the tx is still available"
        );
        assert_eq!(out[0]["txid"], full["txid"]);
        let missing = prefer_full_removed(&[json!({ "txid": "bb".repeat(32) })], &available);
        assert_eq!(missing[0], json!({ "txid": "bb".repeat(32) }));
        assert!(missing[0].get("vout").is_none());
    }

    /// Named contract: a `{txid}` stub is upgraded from fixture history so
    /// `track-addresses` can match vin/vout, not only an unmatched stub.
    #[test]
    fn notify_mempool_removed_emits_full_object_when_available() {
        let (out_tx, mut out_rx) = tokio::sync::mpsc::unbounded_channel();
        let txid = "aa".repeat(32);
        let full = sample_tx_for(genesis_addr());
        let mut history = BTreeMap::new();
        history.insert(
            genesis_addr().to_string(),
            TxBuckets {
                mempool: vec![full.clone()],
                confirmed: vec![],
                removed: vec![],
            },
        );
        let hub = MwckHub::with_history_fixture(history);
        let mut state = ClientState::default();
        let _ = handle_client_json(&mut state, json!({ "track-addresses": [genesis_addr()] }));
        let id = hub.register_socket(out_tx);
        hub.set_client_state(id, state);
        hub.notify_mempool(&[], &[json!({ "txid": txid })]);
        let ev = out_rx.try_recv().expect("removed notify");
        let removed = ev["multi-address-transactions"][genesis_addr()]["removed"]
            .as_array()
            .expect("removed array");
        assert_eq!(removed.len(), 1);
        assert!(
            removed[0].get("vout").is_some(),
            "dropped tx still in fixture history must be a full object, not only txid"
        );
    }

    /// Named contract: a mempool drop notified as `{txid}` only still emits a
    /// full object when the hub saw that tx on add, even if Query/chain no
    /// longer have it. Handshake fixture has no Query.
    #[test]
    fn notify_mempool_removed_emits_full_object_from_add_when_query_empty() {
        let (out_tx, mut out_rx) = tokio::sync::mpsc::unbounded_channel();
        let hub = MwckHub::handshake_fixture();
        let mut state = ClientState::default();
        let _ = handle_client_json(&mut state, json!({ "track-addresses": [genesis_addr()] }));
        let id = hub.register_socket(out_tx);
        hub.set_client_state(id, state);
        let full = sample_tx_for(genesis_addr());
        let txid = full["txid"].as_str().expect("txid").to_string();
        hub.notify_mempool(&[full.clone()], &[]);
        let added_ev = out_rx.try_recv().expect("added notify");
        assert_eq!(
            added_ev["multi-address-transactions"][genesis_addr()]["mempool"]
                .as_array()
                .expect("mempool array")
                .len(),
            1
        );
        hub.notify_mempool(&[], &[json!({ "txid": txid })]);
        let ev = out_rx.try_recv().expect("removed notify");
        let removed = ev["multi-address-transactions"][genesis_addr()]["removed"]
            .as_array()
            .expect("removed array");
        assert_eq!(removed.len(), 1);
        assert!(
            removed[0].get("vout").is_some(),
            "dropped tx the hub saw on add must be a full object when Query is empty"
        );
        assert_eq!(removed[0]["txid"], full["txid"]);
    }

    /// Named contract: when the tip moves more than one height, confirmed
    /// txs from every new height are emitted (not only the last height).
    #[test]
    fn notify_block_emits_confirmed_from_two_new_heights() {
        let (out_tx, mut out_rx) = tokio::sync::mpsc::unbounded_channel();
        let hub = MwckHub::handshake_fixture();
        let mut state = ClientState::default();
        let _ = handle_client_json(&mut state, json!({ "track-addresses": [genesis_addr()] }));
        let id = hub.register_socket(out_tx);
        hub.set_client_state(id, state);
        let h1 = json!({
            "txid": "aa".repeat(32),
            "status": { "block_height": 10 },
            "vin": [],
            "vout": [{
                "scriptpubkey": "76a91462e907b15cbf27d5425399ebf6f0fb50ebb88f1888ac",
                "scriptpubkey_address": genesis_addr(),
            }]
        });
        let h2 = json!({
            "txid": "cc".repeat(32),
            "status": { "block_height": 11 },
            "vin": [],
            "vout": [{
                "scriptpubkey": "76a91462e907b15cbf27d5425399ebf6f0fb50ebb88f1888ac",
                "scriptpubkey_address": genesis_addr(),
            }]
        });
        hub.notify_blocks(&[vec![h1], vec![h2]]);
        let ev = out_rx.try_recv().expect("block notify");
        let confirmed = ev["multi-address-transactions"][genesis_addr()]["confirmed"]
            .as_array()
            .expect("confirmed");
        assert_eq!(
            confirmed.len(),
            2,
            "tip jump of two heights must include confirmed txs from both heights"
        );
    }

    fn test_block_hash(n: u8) -> BlockHash {
        use crate::chain::hashes::Hash;
        let mut bytes = [0u8; 32];
        bytes[0] = n;
        BlockHash::from_raw_hash(crate::chain::hashes::sha256d::Hash::from_byte_array(bytes))
    }

    fn tracked_tx(txid_byte: &str, height: u64) -> Value {
        json!({
            "txid": txid_byte.repeat(32),
            "status": { "block_height": height },
            "vin": [],
            "vout": [{
                "scriptpubkey": "76a91462e907b15cbf27d5425399ebf6f0fb50ebb88f1888ac",
                "scriptpubkey_address": genesis_addr(),
            }]
        })
    }

    /// Named contract: when the tip moves to a different hash at the same
    /// height, clients see removed for the old-tip txs then confirmed for
    /// the new branch. A lower or equal new height is not a no-op.
    #[test]
    fn notify_new_tip_reorg_emits_removed_then_confirmed() {
        let (out_tx, mut out_rx) = tokio::sync::mpsc::unbounded_channel();
        let hub = MwckHub::handshake_fixture();
        let mut state = ClientState::default();
        let _ = handle_client_json(&mut state, json!({ "track-addresses": [genesis_addr()] }));
        let id = hub.register_socket(out_tx);
        hub.set_client_state(id, state);

        let old_tx = tracked_tx("aa", 10);
        let new_tx = tracked_tx("dd", 10);
        let old_hash = test_block_hash(1);
        let new_hash = test_block_hash(2);
        hub.remember_confirmed_for_test(10, old_hash, vec![old_tx.clone()]);
        hub.set_test_block(10, new_hash, vec![new_tx.clone()]);
        hub.notify_new_tip(10, 10);

        let removed_ev = out_rx
            .try_recv()
            .expect("reorg must emit removed for old-tip txs");
        let removed = removed_ev["multi-address-transactions"][genesis_addr()]["removed"]
            .as_array()
            .expect("removed");
        assert_eq!(removed.len(), 1);
        assert_eq!(removed[0]["txid"], old_tx["txid"]);

        let confirmed_ev = out_rx
            .try_recv()
            .expect("reorg must then emit confirmed for the new branch");
        let confirmed = confirmed_ev["multi-address-transactions"][genesis_addr()]["confirmed"]
            .as_array()
            .expect("confirmed");
        assert_eq!(confirmed.len(), 1);
        assert_eq!(confirmed[0]["txid"], new_tx["txid"]);
    }
}
