use crate::auth::{Allowlist, AuthError};
#[cfg(feature = "liquid")]
use crate::chain::address;
use crate::chain::{
    BlockHash, Network, OutPoint, Script, Transaction, TxIn, TxOut, Txid, TxidCompat,
};
use crate::config::{BITCOIND_SUBVER, Config, VERSION_STRING};
use crate::errors;
use crate::metrics::Metrics;
use crate::mwck::{ClientState, MwckHub};
use crate::new_index::{Query, SpendingInput, Utxo, compute_script_hash};
use crate::util::{
    BlockHeaderMeta, BlockId, FullHash, IsProvablyUnspendable, ScriptToAddr, ScriptToAsm,
    SegwitDetection, TransactionStatus, create_socket, electrum_merkle, extract_tx_prevouts,
    full_hash, get_innerscripts, get_tx_fee, has_prevout, is_coinbase, transaction_sigop_count,
};

#[cfg(not(feature = "liquid"))]
use bitcoin::consensus::encode;

#[cfg(feature = "liquid")]
use std::str::FromStr;

use bitcoin::blockdata::opcodes;
use bitcoin::hashes::Hash;
use futures_util::{SinkExt, StreamExt};
use hex::{self, FromHexError};
use hyper::{Body, Method, Request, Response, Server, StatusCode};
use hyper::{
    header::{self, HeaderValue},
    service::{make_service_fn, service_fn},
    upgrade::Upgraded,
};
use prometheus::{HistogramOpts, HistogramVec};
use rayon::iter::ParallelIterator;
use tokio::sync::mpsc;
use tokio::sync::oneshot;
use tokio::time::timeout;
use tokio_tungstenite::WebSocketStream;
use tokio_tungstenite::tungstenite::Message as WsMessage;
use tokio_tungstenite::tungstenite::handshake::derive_accept_key;
use tokio_tungstenite::tungstenite::protocol::Role;

use hyperlocal::UnixServerExt;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use std::{cmp, fs};
#[cfg(feature = "liquid")]
use {
    crate::elements::{AssetMeta, AssetSorting, IssuanceValue, peg::PegoutValue},
    elements::{
        AssetId,
        confidential::{Asset, Nonce, Value},
        encode,
    },
};

use serde::Serialize;
use serde_json;
use std::collections::HashMap;
use std::num::ParseIntError;
use std::os::unix::fs::FileTypeExt;
use std::sync::Arc;
use std::thread;
use url::form_urlencoded;

const ADDRESS_SEARCH_LIMIT: usize = 10;
// Limit to 300 addresses
const MULTI_ADDRESS_LIMIT: usize = 300;

#[cfg(feature = "liquid")]
const ASSETS_PER_PAGE: usize = 25;
#[cfg(feature = "liquid")]
const ASSETS_MAX_PER_PAGE: usize = 100;
#[cfg(feature = "liquid")]
const ASSETS_SEARCH_DEFAULT_LIMIT: usize = 15;
#[cfg(feature = "liquid")]
const ASSETS_SEARCH_MAX_LIMIT: usize = 100;
#[cfg(feature = "liquid")]
const ASSETS_SEARCH_MAX_QUERY_LEN: usize = 64;

const TTL_LONG: u32 = 157_784_630; // ttl for static resources (5 years)
const TTL_SHORT: u32 = 10; // ttl for volatie resources
const TTL_MEMPOOL_RECENT: u32 = 5; // ttl for GET /mempool/recent
const CONF_FINAL: usize = 10; // reorgs deeper than this are considered unlikely

const REQUEST_BODY_TIMEOUT: Duration = Duration::from_secs(30);
const MAX_BODY_SIZE: usize = 1_000_000;
const NIP98_WINDOW_SECS: u64 = 60;
const WS_ALLOWLIST_POLL: Duration = Duration::from_secs(2);
// Header-read timeout (~10s) is a Blockstream new-index valve. hyper 0.14.32 in this
// lockfile has Server::http1_header_read_timeout (landed in 0.14.20). Not wired.

// internal api prefix
const INTERNAL_PREFIX: &str = "internal";

#[derive(Serialize, Deserialize)]
struct BlockValue {
    id: String,
    height: u32,
    version: u32,
    timestamp: u32,
    tx_count: u32,
    size: u32,
    weight: u32,
    merkle_root: String,
    previousblockhash: Option<String>,
    mediantime: u32,

    #[cfg(not(feature = "liquid"))]
    nonce: u32,
    #[cfg(not(feature = "liquid"))]
    bits: u32,
    #[cfg(not(feature = "liquid"))]
    difficulty: f64,

    #[cfg(feature = "liquid")]
    #[serde(skip_serializing_if = "Option::is_none")]
    ext: Option<serde_json::Value>,
}

impl BlockValue {
    #[cfg_attr(feature = "liquid", allow(unused_variables))]
    fn new(blockhm: BlockHeaderMeta) -> Self {
        let header = blockhm.header_entry.header();

        #[cfg(not(feature = "liquid"))]
        let version = header.version.to_consensus() as u32;
        #[cfg(feature = "liquid")]
        let version = header.version;

        BlockValue {
            id: header.block_hash().to_string(),
            height: blockhm.header_entry.height() as u32,
            version,
            timestamp: header.time,
            tx_count: blockhm.meta.tx_count,
            size: blockhm.meta.size,
            weight: blockhm.meta.weight,
            merkle_root: header.merkle_root.to_string(),
            previousblockhash: if header.prev_blockhash != BlockHash::all_zeros() {
                Some(header.prev_blockhash.to_string())
            } else {
                None
            },
            mediantime: blockhm.mtp,

            #[cfg(not(feature = "liquid"))]
            bits: header.bits.to_consensus(),
            #[cfg(not(feature = "liquid"))]
            nonce: header.nonce,
            #[cfg(not(feature = "liquid"))]
            difficulty: difficulty_new(header),

            #[cfg(feature = "liquid")]
            ext: Some(json!(header.ext)),
        }
    }
}

#[cfg(feature = "liquid")]
#[derive(Serialize)]
struct AssetRegistrySearchResult {
    asset_id: AssetId,
    name: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    ticker: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    domain: Option<String>,
}

#[cfg(feature = "liquid")]
impl AssetRegistrySearchResult {
    fn new(asset_id: &AssetId, meta: &AssetMeta) -> Self {
        let domain = meta.domain().map(String::from);
        Self {
            asset_id: *asset_id,
            name: meta.name.clone(),
            ticker: meta.ticker.clone(),
            domain,
        }
    }
}

/// Calculate the difficulty of a BlockHeader
/// using Bitcoin Core code ported to Rust.
///
/// https://github.com/bitcoin/bitcoin/blob/v25.0/src/rpc/blockchain.cpp#L75-L97
#[cfg_attr(feature = "liquid", allow(dead_code))]
fn difficulty_new(bh: &bitcoin::block::Header) -> f64 {
    let mut n_shift = (bh.bits.to_consensus() >> 24) & 0xff;
    let mut d_diff = (0x0000ffff as f64) / ((bh.bits.to_consensus() & 0x00ffffff) as f64);

    while n_shift < 29 {
        d_diff *= 256.0;
        n_shift += 1;
    }
    while n_shift > 29 {
        d_diff /= 256.0;
        n_shift -= 1;
    }

    d_diff
}

#[derive(Serialize, Deserialize)]
struct TransactionValue {
    txid: Txid,
    version: u32,
    locktime: u32,
    vin: Vec<TxInValue>,
    vout: Vec<TxOutValue>,
    size: u32,
    weight: u32,
    sigops: u32,
    fee: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    status: Option<TransactionStatus>,
}

impl TransactionValue {
    fn new(
        tx: Transaction,
        blockid: Option<BlockId>,
        txos: &HashMap<OutPoint, TxOut>,
        config: &Config,
    ) -> Result<Self, errors::Error> {
        let prevouts = extract_tx_prevouts(&tx, txos)?;
        let sigops = transaction_sigop_count(&tx, &prevouts)
            .map_err(|_| errors::Error::from("Couldn't count sigops"))? as u32;

        let vins: Vec<TxInValue> = tx
            .input
            .iter()
            .enumerate()
            .map(|(index, txin)| {
                TxInValue::new(txin, prevouts.get(&(index as u32)).cloned(), config)
            })
            .collect();
        let vouts: Vec<TxOutValue> = tx
            .output
            .iter()
            .map(|txout| TxOutValue::new(txout, config))
            .collect();

        let fee = get_tx_fee(&tx, &prevouts, config.network_type);

        #[cfg(not(feature = "liquid"))]
        let size = tx.total_size() as u32;
        #[cfg(feature = "liquid")]
        let size = tx.size() as u32;

        #[cfg(not(feature = "liquid"))]
        let weight = tx.weight().to_wu() as u32;
        #[cfg(feature = "liquid")]
        let weight = tx.weight() as u32;

        #[cfg(not(feature = "liquid"))]
        let version = tx.version.0 as u32;
        #[cfg(feature = "liquid")]
        let version = tx.version;

        #[allow(clippy::unnecessary_cast)]
        Ok(TransactionValue {
            txid: tx.get_txid(),
            version,
            locktime: tx.lock_time.to_consensus_u32(),
            vin: vins,
            vout: vouts,
            size,
            weight,
            sigops,
            fee,
            status: Some(TransactionStatus::from(blockid)),
        })
    }
}

#[derive(Serialize, Deserialize, Clone)]
struct TxInValue {
    txid: Txid,
    vout: u32,
    prevout: Option<TxOutValue>,
    scriptsig: Script,
    scriptsig_asm: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    witness: Option<Vec<String>>,
    is_coinbase: bool,
    sequence: u32,

    #[serde(skip_serializing_if = "Option::is_none")]
    inner_redeemscript_asm: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    inner_witnessscript_asm: Option<String>,

    #[cfg(feature = "liquid")]
    is_pegin: bool,
    #[cfg(feature = "liquid")]
    #[serde(skip_serializing_if = "Option::is_none")]
    issuance: Option<IssuanceValue>,
}

impl TxInValue {
    fn new(txin: &TxIn, prevout: Option<&TxOut>, config: &Config) -> Self {
        let witness = &txin.witness;
        #[cfg(feature = "liquid")]
        let witness = &witness.script_witness;

        let witness = if !witness.is_empty() {
            Some(witness.iter().map(hex::encode).collect())
        } else {
            None
        };

        let is_coinbase = is_coinbase(txin);

        let innerscripts = prevout.map(|prevout| get_innerscripts(txin, prevout));

        TxInValue {
            txid: txin.previous_output.txid,
            vout: txin.previous_output.vout,
            prevout: prevout.map(|prevout| TxOutValue::new(prevout, config)),
            scriptsig_asm: txin.script_sig.to_asm(),
            witness,

            inner_redeemscript_asm: innerscripts
                .as_ref()
                .and_then(|i| i.redeem_script.as_ref())
                .map(ScriptToAsm::to_asm),
            inner_witnessscript_asm: innerscripts
                .as_ref()
                .and_then(|i| i.witness_script.as_ref())
                .map(ScriptToAsm::to_asm),

            is_coinbase,
            sequence: txin.sequence.to_consensus_u32(),
            #[cfg(feature = "liquid")]
            is_pegin: txin.is_pegin,
            #[cfg(feature = "liquid")]
            issuance: if txin.has_issuance() {
                Some(IssuanceValue::from(txin))
            } else {
                None
            },

            scriptsig: txin.script_sig.clone(),
        }
    }
}

#[derive(Serialize, Deserialize, Clone)]
struct TxOutValue {
    scriptpubkey: Script,
    scriptpubkey_asm: String,
    scriptpubkey_type: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    scriptpubkey_address: Option<String>,

    #[cfg(not(feature = "liquid"))]
    value: u64,

    #[cfg(feature = "liquid")]
    #[serde(skip_serializing_if = "Option::is_none")]
    value: Option<u64>,

    #[cfg(feature = "liquid")]
    #[serde(skip_serializing_if = "Option::is_none")]
    valuecommitment: Option<String>,

    #[cfg(feature = "liquid")]
    #[serde(skip_serializing_if = "Option::is_none")]
    asset: Option<String>,

    #[cfg(feature = "liquid")]
    #[serde(skip_serializing_if = "Option::is_none")]
    assetcommitment: Option<String>,

    #[cfg(feature = "liquid")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pegout: Option<PegoutValue>,
}

impl TxOutValue {
    fn new(txout: &TxOut, config: &Config) -> Self {
        #[cfg(not(feature = "liquid"))]
        let value = txout.value.to_sat();

        #[cfg(feature = "liquid")]
        let value = txout.value.explicit();
        #[cfg(feature = "liquid")]
        let valuecommitment = match txout.value {
            Value::Confidential(..) => Some(hex::encode(encode::serialize(&txout.value))),
            _ => None,
        };

        #[cfg(feature = "liquid")]
        let asset = match txout.asset {
            Asset::Explicit(value) => Some(value.to_string()),
            _ => None,
        };
        #[cfg(feature = "liquid")]
        let assetcommitment = match txout.asset {
            Asset::Confidential(..) => Some(hex::encode(encode::serialize(&txout.asset))),
            _ => None,
        };

        #[cfg(not(feature = "liquid"))]
        let is_fee = false;
        #[cfg(feature = "liquid")]
        let is_fee = txout.is_fee();

        let script = &txout.script_pubkey;
        let script_asm = script.to_asm();
        let script_addr = script.to_address_str(config.network_type);

        // TODO should the following something to put inside rust-elements lib?
        let script_type = if is_fee {
            "fee"
        } else if script.is_empty() {
            "empty"
        } else if script.is_provably_unspendable_() {
            "op_return"
        } else if script.is_p2pk() {
            "p2pk"
        } else if script.is_p2pkh() {
            "p2pkh"
        } else if script.is_p2sh() {
            "p2sh"
        } else if script.segwit_is_p2wpkh() {
            "v0_p2wpkh"
        } else if script.segwit_is_p2wsh() {
            "v0_p2wsh"
        } else if script.segwit_is_p2tr() {
            "v1_p2tr"
        } else if is_anchor(script) {
            "anchor"
        } else if is_bare_multisig(script) {
            "multisig"
        } else {
            "unknown"
        };

        #[cfg(feature = "liquid")]
        let pegout = PegoutValue::from_txout(txout, config.network_type, config.parent_network);

        TxOutValue {
            scriptpubkey: script.clone(),
            scriptpubkey_asm: script_asm,
            scriptpubkey_address: script_addr,
            scriptpubkey_type: script_type.to_string(),
            value,
            #[cfg(feature = "liquid")]
            valuecommitment,
            #[cfg(feature = "liquid")]
            asset,
            #[cfg(feature = "liquid")]
            assetcommitment,
            #[cfg(feature = "liquid")]
            pegout,
        }
    }
}
fn is_bare_multisig(script: &Script) -> bool {
    let len = script.len();
    let bytes = script.as_bytes();
    // 1-of-1 multisig is 37 bytes
    // Max is 15 pubkeys
    // Min is 1
    // First byte must be <= the second to last (4-of-2 makes no sense)
    // We won't check the pubkeys, just assume anything with the form
    //   OP_M ... OP_N OP_CHECKMULTISIG
    // is bare multisig
    len >= 37
        && bytes[len - 1] == opcodes::all::OP_CHECKMULTISIG.to_u8()
        && bytes[len - 2] >= opcodes::all::OP_PUSHNUM_1.to_u8()
        && bytes[len - 2] <= opcodes::all::OP_PUSHNUM_15.to_u8()
        && bytes[0] >= opcodes::all::OP_PUSHNUM_1.to_u8()
        && bytes[0] <= bytes[len - 2]
}

fn is_anchor(script: &Script) -> bool {
    let bytes = script.as_bytes();
    let len = bytes.len();
    len == 4
        && bytes[0] == opcodes::all::OP_PUSHNUM_1.to_u8()
        && bytes[1] == opcodes::all::OP_PUSHBYTES_2.to_u8()
        && bytes[2] == 0x4e
        && bytes[3] == 0x73
}

#[derive(Serialize)]
struct UtxoValue {
    txid: Txid,
    vout: u32,
    status: TransactionStatus,

    #[cfg(not(feature = "liquid"))]
    value: u64,

    #[cfg(feature = "liquid")]
    #[serde(skip_serializing_if = "Option::is_none")]
    value: Option<u64>,

    #[cfg(feature = "liquid")]
    #[serde(skip_serializing_if = "Option::is_none")]
    valuecommitment: Option<String>,

    #[cfg(feature = "liquid")]
    #[serde(skip_serializing_if = "Option::is_none")]
    asset: Option<String>,

    #[cfg(feature = "liquid")]
    #[serde(skip_serializing_if = "Option::is_none")]
    assetcommitment: Option<String>,

    #[cfg(feature = "liquid")]
    #[serde(skip_serializing_if = "Option::is_none")]
    nonce: Option<String>,

    #[cfg(feature = "liquid")]
    #[serde(skip_serializing_if = "Option::is_none")]
    noncecommitment: Option<String>,

    #[cfg(feature = "liquid")]
    #[serde(skip_serializing_if = "Vec::is_empty", with = "crate::util::serde_hex")]
    surjection_proof: Vec<u8>,

    #[cfg(feature = "liquid")]
    #[serde(skip_serializing_if = "Vec::is_empty", with = "crate::util::serde_hex")]
    range_proof: Vec<u8>,
}
impl From<Utxo> for UtxoValue {
    fn from(utxo: Utxo) -> Self {
        UtxoValue {
            txid: utxo.txid,
            vout: utxo.vout,
            status: TransactionStatus::from(utxo.confirmed),

            #[cfg(not(feature = "liquid"))]
            value: utxo.value,

            #[cfg(feature = "liquid")]
            value: match utxo.value {
                Value::Explicit(value) => Some(value),
                _ => None,
            },
            #[cfg(feature = "liquid")]
            valuecommitment: match utxo.value {
                Value::Confidential(..) => Some(hex::encode(encode::serialize(&utxo.value))),
                _ => None,
            },
            #[cfg(feature = "liquid")]
            asset: match utxo.asset {
                Asset::Explicit(asset) => Some(asset.to_string()),
                _ => None,
            },
            #[cfg(feature = "liquid")]
            assetcommitment: match utxo.asset {
                Asset::Confidential(..) => Some(hex::encode(encode::serialize(&utxo.asset))),
                _ => None,
            },
            #[cfg(feature = "liquid")]
            nonce: match utxo.nonce {
                Nonce::Explicit(nonce) => Some(hex::encode(nonce)),
                _ => None,
            },
            #[cfg(feature = "liquid")]
            noncecommitment: match utxo.nonce {
                Nonce::Confidential(..) => Some(hex::encode(encode::serialize(&utxo.nonce))),
                _ => None,
            },
            #[cfg(feature = "liquid")]
            surjection_proof: utxo
                .witness
                .surjection_proof
                .map_or(vec![], |p| (*p).serialize()),
            #[cfg(feature = "liquid")]
            range_proof: utxo.witness.rangeproof.map_or(vec![], |p| (*p).serialize()),
        }
    }
}

#[derive(Serialize, Default)]
struct SpendingValue {
    spent: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    txid: Option<Txid>,
    #[serde(skip_serializing_if = "Option::is_none")]
    vin: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    status: Option<TransactionStatus>,
}
impl From<SpendingInput> for SpendingValue {
    fn from(spend: SpendingInput) -> Self {
        SpendingValue {
            spent: true,
            txid: Some(spend.txid),
            vin: Some(spend.vin),
            status: Some(TransactionStatus::from(spend.confirmed)),
        }
    }
}

fn ttl_by_depth(height: Option<usize>, query: &Query) -> u32 {
    height.map_or(TTL_SHORT, |height| {
        if query.chain().best_height() - height >= CONF_FINAL {
            TTL_LONG
        } else {
            TTL_SHORT
        }
    })
}

enum TxidLocation {
    Mempool,
    Chain(u32), // contains height
    None,
}

#[inline]
fn find_txid(
    txid: &Txid,
    mempool: &crate::new_index::Mempool,
    chain: &crate::new_index::ChainQuery,
) -> TxidLocation {
    if mempool.lookup_txn(txid).is_some() {
        TxidLocation::Mempool
    } else if let Some(block) = chain.tx_confirming_block(txid) {
        TxidLocation::Chain(block.height as u32)
    } else {
        TxidLocation::None
    }
}

#[inline]
fn confirmed_after_txid<'a>(
    after_txid_location: &TxidLocation,
    after_txid: Option<&'a Txid>,
) -> Option<&'a Txid> {
    match after_txid_location {
        // A mempool cursor never exists in chain history, so always
        // start from the newest confirmed tx when crossing the boundary.
        TxidLocation::Mempool | TxidLocation::None => None,
        TxidLocation::Chain(_) => after_txid,
    }
}

/// Prepare transactions to be serialized in a JSON response
///
/// Any transactions with missing prevouts will be filtered out of the response, rather than returned with incorrect data.
fn prepare_txs(
    txs: Vec<(Transaction, Option<BlockId>)>,
    query: &Query,
    config: &Config,
) -> Vec<TransactionValue> {
    let outpoints = txs
        .iter()
        .flat_map(|(tx, _)| {
            tx.input
                .iter()
                .filter(|txin| has_prevout(txin))
                .map(|txin| txin.previous_output)
        })
        .collect();

    let prevouts = query.lookup_txos(&outpoints);

    txs.into_iter()
        .filter_map(|(tx, blockid)| TransactionValue::new(tx, blockid, &prevouts, config).ok())
        .collect()
}

/// Serialize confirmed or mempool txs for MWCK notify.
pub fn transactions_as_json(
    txs: Vec<(Transaction, Option<BlockId>)>,
    query: &Query,
    config: &Config,
) -> Vec<serde_json::Value> {
    prepare_txs(txs, query, config)
        .into_iter()
        .filter_map(|tv| serde_json::to_value(&tv).ok())
        .collect()
}

/// Outcome of the HTTP / websocket NIP-98 gate.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HttpAuthOutcome {
    PublicHealth,
    Pubkey([u8; 32]),
}

fn is_public_health_path(path: &str) -> bool {
    path == "/blocks/tip/hash" || path == "/blocks/tip/height"
}

/// Reconstruct the absolute URL NIP-98 `u` must match: proto from
/// `X-Forwarded-Proto` (first hop) else `http`, host from `Host`, path+query.
pub fn reconstruct_absolute_url(
    forwarded_proto: Option<&str>,
    host: Option<&str>,
    path_and_query: &str,
) -> String {
    let proto = forwarded_proto
        .unwrap_or("http")
        .split(',')
        .next()
        .unwrap_or("http")
        .trim();
    let proto = if proto.is_empty() { "http" } else { proto };
    let host = host.unwrap_or("localhost").trim();
    let host = if host.is_empty() { "localhost" } else { host };
    let path = if path_and_query.is_empty() {
        "/"
    } else {
        path_and_query
    };
    format!("{}://{}{}", proto, host, path)
}

fn reconstruct_from_req(req: &Request<Body>) -> String {
    let proto = req
        .headers()
        .get("x-forwarded-proto")
        .and_then(|v| v.to_str().ok());
    let host = req
        .headers()
        .get(header::HOST)
        .and_then(|v| v.to_str().ok());
    let pq = req
        .uri()
        .path_and_query()
        .map(|p| p.as_str())
        .unwrap_or_else(|| req.uri().path());
    reconstruct_absolute_url(proto, host, pq)
}

fn now_unix() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

/// HTTP and `/api/v1/ws` NIP-98 gate. `--public-health` skips auth only for
/// GET `/blocks/tip/hash` and GET `/blocks/tip/height`. An empty allowlist is
/// 401 for everything else. Localhost is not an exception.
pub fn http_ws_auth_gate(
    method: &str,
    path: &str,
    authorization: Option<&str>,
    absolute_url: &str,
    body: &[u8],
    now_unix: u64,
    public_health: bool,
    allow: &Allowlist,
) -> Result<HttpAuthOutcome, AuthError> {
    if public_health && method.eq_ignore_ascii_case("GET") && is_public_health_path(path) {
        return Ok(HttpAuthOutcome::PublicHealth);
    }
    let pk = crate::auth::verify_nip98(
        authorization,
        method,
        absolute_url,
        body,
        now_unix,
        NIP98_WINDOW_SECS,
        allow,
    )?;
    Ok(HttpAuthOutcome::Pubkey(pk))
}

fn unauthorized_response(err: &AuthError) -> Response<Body> {
    Response::builder()
        .status(StatusCode::UNAUTHORIZED)
        .header("Content-Type", "text/plain")
        .header("WWW-Authenticate", "Nostr")
        .body(Body::from(err.to_string()))
        .unwrap()
}

fn apply_common_headers(resp: &mut Response<Body>, config: &Config) {
    resp.headers_mut()
        .insert("X-Powered-By", HeaderValue::from_static(&VERSION_STRING));
    if let Some(ref origins) = config.cors {
        resp.headers_mut()
            .insert("Access-Control-Allow-Origin", origins.parse().unwrap());
    }
    if let Some(subver) = BITCOIND_SUBVER.get() {
        resp.headers_mut()
            .insert("X-Bitcoin-Version", HeaderValue::from_static(subver));
    }
}

fn is_websocket_upgrade(req: &Request<Body>) -> bool {
    let upgrade = req
        .headers()
        .get(header::UPGRADE)
        .and_then(|v| v.to_str().ok())
        .map(|s| s.eq_ignore_ascii_case("websocket"))
        .unwrap_or(false);
    let conn = req
        .headers()
        .get(header::CONNECTION)
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_ascii_lowercase().contains("upgrade"))
        .unwrap_or(false);
    upgrade && conn
}

fn auth_header_from(req: &Request<Body>) -> Option<String> {
    req.headers()
        .get(header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string())
}

/// HTTP status for GET /api/v1/ws after the NIP-98 gate. This does not run
/// the hyper upgrade. 401 when the gate fails; 101 when listed plus websocket
/// headers; 400 when authorized but the request is not a websocket upgrade.
pub fn mwck_upgrade_http_status(
    gate: Result<HttpAuthOutcome, AuthError>,
    is_websocket_upgrade: bool,
    has_sec_websocket_key: bool,
) -> StatusCode {
    match gate {
        Ok(HttpAuthOutcome::Pubkey(_)) => {
            if !is_websocket_upgrade || !has_sec_websocket_key {
                StatusCode::BAD_REQUEST
            } else {
                StatusCode::SWITCHING_PROTOCOLS
            }
        }
        Ok(HttpAuthOutcome::PublicHealth) | Err(_) => StatusCode::UNAUTHORIZED,
    }
}

async fn handle_mwck_upgrade(
    req: Request<Body>,
    allow: Arc<Allowlist>,
    hub: Arc<MwckHub>,
    public_health: bool,
) -> Response<Body> {
    let authorization = auth_header_from(&req);
    let absolute = reconstruct_from_req(&req);
    let method = req.method().as_str().to_string();
    let path = req.uri().path().to_string();
    let gate = http_ws_auth_gate(
        &method,
        &path,
        authorization.as_deref(),
        &absolute,
        b"",
        now_unix(),
        public_health,
        &allow,
    );
    let ws = is_websocket_upgrade(&req);
    let has_key = req.headers().get("sec-websocket-key").is_some();
    match mwck_upgrade_http_status(gate.clone(), ws, has_key) {
        StatusCode::UNAUTHORIZED => {
            return unauthorized_response(match &gate {
                Err(e) => e,
                Ok(_) => &AuthError::NotAllowlisted,
            });
        }
        StatusCode::BAD_REQUEST => {
            let msg = if !ws {
                "expected WebSocket upgrade on GET /api/v1/ws"
            } else {
                "missing Sec-WebSocket-Key"
            };
            return Response::builder()
                .status(StatusCode::BAD_REQUEST)
                .header("Content-Type", "text/plain")
                .body(Body::from(msg))
                .unwrap();
        }
        _ => {}
    }
    let pubkey = match gate {
        Ok(HttpAuthOutcome::Pubkey(pk)) => pk,
        _ => return unauthorized_response(&AuthError::NotAllowlisted),
    };

    let key = match req.headers().get("sec-websocket-key") {
        Some(k) => k.clone(),
        None => {
            return Response::builder()
                .status(StatusCode::BAD_REQUEST)
                .header("Content-Type", "text/plain")
                .body(Body::from("missing Sec-WebSocket-Key"))
                .unwrap();
        }
    };
    let accept = derive_accept_key(key.as_bytes());
    let on_upgrade = hyper::upgrade::on(req);
    tokio::spawn(async move {
        match on_upgrade.await {
            Ok(upgraded) => {
                let ws = WebSocketStream::from_raw_socket(upgraded, Role::Server, None).await;
                run_mwck_socket(ws, allow, hub, pubkey).await;
            }
            Err(e) => warn!("websocket upgrade failed: {}", e),
        }
    });

    Response::builder()
        .status(StatusCode::SWITCHING_PROTOCOLS)
        .header(header::UPGRADE, "websocket")
        .header(header::CONNECTION, "Upgrade")
        .header("Sec-WebSocket-Accept", accept)
        .body(Body::empty())
        .unwrap()
}

async fn run_mwck_socket(
    ws: WebSocketStream<Upgraded>,
    allow: Arc<Allowlist>,
    hub: Arc<MwckHub>,
    pubkey: [u8; 32],
) {
    let (mut sink, mut stream) = ws.split();
    let (tx, mut rx) = mpsc::unbounded_channel();
    let id = hub.register_socket(tx);
    let mut state = ClientState::default();
    let mut tick = tokio::time::interval(WS_ALLOWLIST_POLL);

    loop {
        if !allow.contains(&pubkey) {
            break;
        }
        tokio::select! {
            _ = tick.tick() => {
                if !allow.contains(&pubkey) {
                    break;
                }
            }
            maybe_out = rx.recv() => {
                match maybe_out {
                    Some(v) => {
                        if sink.send(WsMessage::Text(v.to_string())).await.is_err() {
                            break;
                        }
                    }
                    None => break,
                }
            }
            incoming = stream.next() => {
                match incoming {
                    Some(Ok(WsMessage::Text(t))) => {
                        if !allow.contains(&pubkey) {
                            break;
                        }
                        let msg: serde_json::Value = match serde_json::from_str(&t) {
                            Ok(v) => v,
                            Err(_) => continue,
                        };
                        let reply = hub.handle_socket(&allow, &pubkey, &mut state, msg).await;
                        hub.set_client_state(id, state.clone());
                        if let Some(out) = reply {
                            if sink.send(WsMessage::Text(out.to_string())).await.is_err() {
                                break;
                            }
                        }
                    }
                    Some(Ok(WsMessage::Ping(p))) => {
                        if sink.send(WsMessage::Pong(p)).await.is_err() {
                            break;
                        }
                    }
                    Some(Ok(WsMessage::Close(_))) | None | Some(Err(_)) => break,
                    Some(Ok(WsMessage::Binary(_)))
                    | Some(Ok(WsMessage::Pong(_)))
                    | Some(Ok(WsMessage::Frame(_))) => {}
                }
            }
        }
    }
    hub.detach_client(id);
    let _ = sink.send(WsMessage::Close(None)).await;
}

#[tokio::main]
async fn run_server(
    config: Arc<Config>,
    query: Arc<Query>,
    rx: oneshot::Receiver<()>,
    metric: HistogramVec,
    allow: Arc<Allowlist>,
    hub: Arc<MwckHub>,
) {
    let addr = &config.http_addr;
    let socket_file = &config.http_socket_file;

    let config = Arc::clone(&config);
    let query = Arc::clone(&query);

    let make_service_fn_inn = || {
        let query = Arc::clone(&query);
        let config = Arc::clone(&config);
        let allow = Arc::clone(&allow);
        let hub = Arc::clone(&hub);
        let metric = metric.clone();

        async move {
            Ok::<_, hyper::Error>(service_fn(move |req: Request<Body>| {
                let query = Arc::clone(&query);
                let config = Arc::clone(&config);
                let allow = Arc::clone(&allow);
                let hub = Arc::clone(&hub);
                let timer = metric.with_label_values(&["all_methods"]).start_timer();

                async move {
                    if req.uri().path() == "/api/v1/ws" {
                        let mut resp =
                            handle_mwck_upgrade(req, allow, hub, config.public_health).await;
                        apply_common_headers(&mut resp, &config);
                        timer.observe_duration();
                        return Ok::<_, hyper::Error>(resp);
                    }

                    let authorization = auth_header_from(&req);
                    let absolute = reconstruct_from_req(&req);
                    let method = req.method().clone();
                    let uri = req.uri().clone();
                    let body_result =
                        timeout(REQUEST_BODY_TIMEOUT, hyper::body::to_bytes(req.into_body())).await;

                    let body = match body_result {
                        Ok(Ok(bytes)) if bytes.len() > MAX_BODY_SIZE => {
                            return Ok(Response::builder()
                                .status(StatusCode::PAYLOAD_TOO_LARGE)
                                .header("Content-Type", "text/plain")
                                .body(Body::from("Request body too large"))
                                .unwrap());
                        }
                        Ok(Ok(bytes)) => bytes,
                        Ok(Err(e)) => return Err(e),
                        Err(_) => {
                            return Ok(Response::builder()
                                .status(StatusCode::REQUEST_TIMEOUT)
                                .header("Content-Type", "text/plain")
                                .body(Body::from("Request timeout"))
                                .unwrap());
                        }
                    };

                    if let Err(e) = http_ws_auth_gate(
                        method.as_str(),
                        uri.path(),
                        authorization.as_deref(),
                        &absolute,
                        &body,
                        now_unix(),
                        config.public_health,
                        &allow,
                    ) {
                        let mut resp = unauthorized_response(&e);
                        apply_common_headers(&mut resp, &config);
                        timer.observe_duration();
                        return Ok(resp);
                    }

                    let mut resp = tokio::task::block_in_place(|| {
                        handle_request(method, uri, body, &query, &config)
                    })
                    .unwrap_or_else(|err| {
                        warn!("{:?}", err);
                        Response::builder()
                            .status(err.0)
                            .header("Content-Type", "text/plain")
                            .body(Body::from(err.1))
                            .unwrap()
                    });
                    apply_common_headers(&mut resp, &config);
                    timer.observe_duration();
                    Ok::<_, hyper::Error>(resp)
                }
            }))
        }
    };

    let server = match socket_file {
        None => {
            info!("REST server running on {}", addr);

            let socket = create_socket(addr);
            socket.listen(511).expect("setting backlog failed");

            Server::from_tcp(socket.into())
                .expect("Server::from_tcp failed")
                .serve(make_service_fn(move |_| make_service_fn_inn()))
                .with_graceful_shutdown(async {
                    rx.await.ok();
                })
                .await
        }
        Some(path) => {
            if let Ok(meta) = fs::metadata(path) {
                // Cleanup socket file left by previous execution
                if meta.file_type().is_socket() {
                    fs::remove_file(path).ok();
                }
            }

            info!("REST server running on unix socket {}", path.display());

            Server::bind_unix(path)
                .expect("Server::bind_unix failed")
                .serve(make_service_fn(move |_| make_service_fn_inn()))
                .with_graceful_shutdown(async {
                    rx.await.ok();
                })
                .await
        }
    };

    if let Err(e) = server {
        eprintln!("server error: {}", e);
    }
}

pub fn start(
    config: Arc<Config>,
    query: Arc<Query>,
    metrics: &Metrics,
    allow: Arc<Allowlist>,
    hub: Arc<MwckHub>,
) -> Handle {
    let (tx, rx) = oneshot::channel::<()>();
    let response_timer = metrics.histogram_vec(
        HistogramOpts::new("electrs_rest_api", "Electrs REST API response timings"),
        &["method"],
    );

    Handle {
        tx,
        thread: crate::util::spawn_thread("rest-server", move || {
            run_server(config, query, rx, response_timer, allow, hub);
        }),
    }
}

pub struct Handle {
    tx: oneshot::Sender<()>,
    thread: thread::JoinHandle<()>,
}

impl Handle {
    pub fn stop(self) {
        self.tx.send(()).expect("failed to send shutdown signal");
        self.thread.join().expect("REST server failed");
    }
}

fn handle_request(
    method: Method,
    uri: hyper::Uri,
    body: hyper::body::Bytes,
    query: &Query,
    config: &Config,
) -> Result<Response<Body>, HttpError> {
    // TODO it looks hyper does not have routing and query parsing :(
    let path: Vec<&str> = uri.path().split('/').skip(1).collect();
    let query_params = match uri.query() {
        Some(value) => form_urlencoded::parse(value.as_bytes())
            .into_owned()
            .collect::<HashMap<String, String>>(),
        None => HashMap::new(),
    };

    info!("handle {:?} {:?}", method, uri);
    match (
        &method,
        path.first(),
        path.get(1),
        path.get(2),
        path.get(3),
        path.get(4),
    ) {
        (&Method::GET, Some(&"blocks"), Some(&"tip"), Some(&"hash"), None, None) => http_message(
            StatusCode::OK,
            query.chain().best_hash().to_string(),
            TTL_SHORT,
        ),

        (&Method::GET, Some(&"blocks"), Some(&"tip"), Some(&"height"), None, None) => http_message(
            StatusCode::OK,
            query.chain().best_height().to_string(),
            TTL_SHORT,
        ),

        (&Method::GET, Some(&"block-template"), None, None, None, None) => {
            handle_block_template(query, config)
        }

        // Authenticated Electrum JSON-RPC over HTTP. Production Electrum is
        // this path plus `--rpc-socket-file`. Do not treat raw TCP Electrum
        // as the production listener.
        (&Method::POST, Some(&"electrum"), None, None, None, None) => {
            let body_str = std::str::from_utf8(&body)
                .map_err(|_| HttpError::from("Electrum JSON-RPC body must be UTF-8".to_string()))?;
            let value = crate::electrum::handle_http_jsonrpc_body(query, body_str);
            json_response_no_store(value)
        }

        (&Method::GET, Some(&"blocks"), start_height, None, None, None) => {
            let start_height = start_height.and_then(|height| height.parse::<usize>().ok());
            blocks(query, config, start_height)
        }
        (&Method::GET, Some(&"block-height"), Some(height), None, None, None) => {
            let height = height.parse::<usize>()?;
            let header = query
                .chain()
                .header_by_height(height)
                .ok_or_else(|| HttpError::not_found("Block not found".to_string()))?;
            let ttl = ttl_by_depth(Some(height), query);
            http_message(StatusCode::OK, header.hash().to_string(), ttl)
        }
        (&Method::GET, Some(&"block"), Some(hash), None, None, None) => {
            let hash = hash.parse::<BlockHash>()?;
            let blockhm = query
                .chain()
                .get_block_with_meta(&hash)
                .ok_or_else(|| HttpError::not_found("Block not found".to_string()))?;
            let block_value = BlockValue::new(blockhm);
            json_response(block_value, TTL_LONG)
        }
        (&Method::GET, Some(&"block"), Some(hash), Some(&"status"), None, None) => {
            let hash = hash.parse::<BlockHash>()?;
            let status = query.chain().get_block_status(&hash);
            let ttl = ttl_by_depth(status.height, query);
            json_response(status, ttl)
        }
        (&Method::GET, Some(&"block"), Some(hash), Some(&"txids"), None, None) => {
            let hash = hash.parse::<BlockHash>()?;
            let txids = query
                .chain()
                .get_block_txids(&hash)
                .ok_or_else(|| HttpError::not_found("Block not found".to_string()))?;
            json_response(txids, TTL_LONG)
        }
        (&Method::GET, Some(&INTERNAL_PREFIX), Some(&"block"), Some(hash), Some(&"txs"), None) => {
            let hash = hash.parse::<BlockHash>()?;
            let block_id = query.chain().blockid_by_hash(&hash);
            let txs = query
                .chain()
                .get_block_txs(&hash)
                .ok_or_else(|| HttpError::not_found("Block not found".to_string()))?
                .into_iter()
                .map(|tx| (tx, block_id.clone()))
                .collect();

            let ttl = ttl_by_depth(block_id.map(|b| b.height), query);
            json_response(prepare_txs(txs, query, config), ttl)
        }
        (&Method::GET, Some(&"block"), Some(hash), Some(&"header"), None, None) => {
            let hash = hash.parse::<BlockHash>()?;
            let header = query
                .chain()
                .get_block_header(&hash)
                .ok_or_else(|| HttpError::not_found("Block not found".to_string()))?;

            let header_hex = hex::encode(encode::serialize(&header));
            http_message(StatusCode::OK, header_hex, TTL_LONG)
        }
        (&Method::GET, Some(&"block"), Some(hash), Some(&"raw"), None, None) => {
            let hash = hash.parse::<BlockHash>()?;
            let raw = query
                .chain()
                .get_block_raw(&hash)
                .ok_or_else(|| HttpError::not_found("Block not found".to_string()))?;

            Ok(Response::builder()
                .status(StatusCode::OK)
                .header("Content-Type", "application/octet-stream")
                .header("Cache-Control", format!("public, max-age={:}", TTL_LONG))
                .body(Body::from(raw))
                .unwrap())
        }
        (&Method::GET, Some(&"block"), Some(hash), Some(&"txid"), Some(index), None) => {
            let hash = hash.parse::<BlockHash>()?;
            let index: usize = index.parse()?;
            let txids = query
                .chain()
                .get_block_txids(&hash)
                .ok_or_else(|| HttpError::not_found("Block not found".to_string()))?;
            if index >= txids.len() {
                bail!(HttpError::not_found("tx index out of range".to_string()));
            }
            http_message(StatusCode::OK, txids[index].to_string(), TTL_LONG)
        }
        (&Method::GET, Some(&"block"), Some(hash), Some(&"txs"), start_index, None) => {
            let hash = hash.parse::<BlockHash>()?;
            let txids = query
                .chain()
                .get_block_txids(&hash)
                .ok_or_else(|| HttpError::not_found("Block not found".to_string()))?;

            let start_index = start_index.map_or(0u32, |el| el.parse().unwrap_or(0)) as usize;
            if start_index >= txids.len() {
                bail!(HttpError::not_found("start index out of range".to_string()));
            } else if start_index % config.rest_default_chain_txs_per_page != 0 {
                bail!(HttpError::from(format!(
                    "start index must be a multipication of {}",
                    config.rest_default_chain_txs_per_page
                )));
            }

            // blockid_by_hash() only returns the BlockId for non-orphaned blocks,
            // or None for orphaned
            let confirmed_blockid = query.chain().blockid_by_hash(&hash);

            let txs = txids
                .iter()
                .skip(start_index)
                .take(config.rest_default_chain_txs_per_page)
                .map(|txid| {
                    query
                        .lookup_txn(txid)
                        .map(|tx| (tx, confirmed_blockid.clone()))
                        .ok_or_else(|| "missing tx".to_string())
                })
                .collect::<Result<Vec<(Transaction, Option<BlockId>)>, _>>()?;

            // XXX orphraned blocks alway get TTL_SHORT
            let ttl = ttl_by_depth(confirmed_blockid.map(|b| b.height), query);

            json_response(prepare_txs(txs, query, config), ttl)
        }
        (&Method::GET, Some(script_type @ &"address"), Some(script_str), None, None, None)
        | (&Method::GET, Some(script_type @ &"scripthash"), Some(script_str), None, None, None) => {
            let script_hash = to_scripthash(script_type, script_str, config.network_type)?;
            let stats = query.stats(&script_hash[..]);
            json_response(
                json!({
                    *script_type: script_str,
                    "chain_stats": stats.0,
                    "mempool_stats": stats.1,
                }),
                TTL_SHORT,
            )
        }
        (
            &Method::GET,
            Some(script_type @ &"address"),
            Some(script_str),
            Some(&"txs"),
            None,
            None,
        )
        | (
            &Method::GET,
            Some(script_type @ &"scripthash"),
            Some(script_str),
            Some(&"txs"),
            None,
            None,
        ) => {
            let script_hash = to_scripthash(script_type, script_str, config.network_type)?;
            let max_txs = query_params
                .get("max_txs")
                .and_then(|s| s.parse::<usize>().ok())
                .unwrap_or(config.rest_default_max_mempool_txs);
            let after_txid = query_params
                .get("after_txid")
                .and_then(|s| s.parse::<Txid>().ok());

            let mut txs = vec![];

            let after_txid_location = if let Some(txid) = &after_txid {
                find_txid(txid, &query.mempool(), query.chain())
            } else {
                TxidLocation::Mempool
            };

            let confirmed_block_height = match after_txid_location {
                TxidLocation::Mempool => {
                    txs.extend(
                        query
                            .mempool()
                            .history(&script_hash[..], after_txid.as_ref(), max_txs)
                            .into_iter()
                            .map(|tx| (tx, None)),
                    );
                    None
                }
                TxidLocation::None => {
                    return Err(HttpError(
                        StatusCode::UNPROCESSABLE_ENTITY,
                        String::from("after_txid not found"),
                    ));
                }
                TxidLocation::Chain(height) => Some(height),
            };

            if txs.len() < max_txs {
                let after_txid_ref =
                    confirmed_after_txid(&after_txid_location, after_txid.as_ref());
                let mut confirmed_txs = query
                    .chain()
                    .history(
                        &script_hash[..],
                        after_txid_ref,
                        confirmed_block_height,
                        max_txs - txs.len(),
                    )
                    .map(|res| {
                        res.map(|(tx, blockid, tx_position)| (tx, Some(blockid), tx_position))
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                confirmed_txs.sort_unstable_by(
                    |(_, blockid1, tx_position1), (_, blockid2, tx_position2)| {
                        blockid2
                            .as_ref()
                            .map(|b| b.height)
                            .cmp(&blockid1.as_ref().map(|b| b.height))
                            .then_with(|| tx_position2.cmp(tx_position1))
                    },
                );
                txs.extend(
                    confirmed_txs
                        .into_iter()
                        .map(|(tx, blockid, _)| (tx, blockid)),
                );
            }

            json_response(prepare_txs(txs, query, config), TTL_SHORT)
        }

        (&Method::POST, Some(script_types @ &"addresses"), Some(&"txs"), None, None, None)
        | (&Method::POST, Some(script_types @ &"scripthashes"), Some(&"txs"), None, None, None) => {
            let script_type = match *script_types {
                "addresses" => "address",
                "scripthashes" => "scripthash",
                _ => "",
            };

            if multi_address_too_long(&body) {
                return Err(HttpError(
                    StatusCode::UNPROCESSABLE_ENTITY,
                    String::from("body too long"),
                ));
            }

            let script_hashes: Vec<String> =
                serde_json::from_slice(&body).map_err(|err| HttpError::from(err.to_string()))?;

            if script_hashes.len() > MULTI_ADDRESS_LIMIT {
                return Err(HttpError(
                    StatusCode::UNPROCESSABLE_ENTITY,
                    String::from("body too long"),
                ));
            }

            let script_hashes: Vec<[u8; 32]> = script_hashes
                .iter()
                .filter_map(|script_str| {
                    to_scripthash(script_type, script_str, config.network_type).ok()
                })
                .collect();

            let max_txs = query_params
                .get("max_txs")
                .and_then(|s| s.parse::<usize>().ok())
                .unwrap_or(config.rest_default_max_mempool_txs);
            let after_txid = query_params
                .get("after_txid")
                .and_then(|s| s.parse::<Txid>().ok());

            let mut txs = vec![];

            let after_txid_location = if let Some(txid) = &after_txid {
                find_txid(txid, &query.mempool(), query.chain())
            } else {
                TxidLocation::Mempool
            };

            let confirmed_block_height = match after_txid_location {
                TxidLocation::Mempool => {
                    txs.extend(
                        query
                            .mempool()
                            .history_group(&script_hashes, after_txid.as_ref(), max_txs)
                            .into_iter()
                            .map(|tx| (tx, None)),
                    );
                    None
                }
                TxidLocation::None => {
                    return Err(HttpError(
                        StatusCode::UNPROCESSABLE_ENTITY,
                        String::from("after_txid not found"),
                    ));
                }
                TxidLocation::Chain(height) => Some(height),
            };

            if txs.len() < max_txs {
                let after_txid_ref =
                    confirmed_after_txid(&after_txid_location, after_txid.as_ref());
                let mut confirmed_txs = query
                    .chain()
                    .history_group(
                        &script_hashes,
                        after_txid_ref,
                        confirmed_block_height,
                        max_txs - txs.len(),
                    )
                    .map(|res| {
                        res.map(|(tx, blockid, tx_position)| (tx, Some(blockid), tx_position))
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                confirmed_txs.sort_unstable_by(
                    |(_, blockid1, tx_position1), (_, blockid2, tx_position2)| {
                        blockid2
                            .as_ref()
                            .map(|b| b.height)
                            .cmp(&blockid1.as_ref().map(|b| b.height))
                            .then_with(|| tx_position2.cmp(tx_position1))
                    },
                );
                txs.extend(
                    confirmed_txs
                        .into_iter()
                        .map(|(tx, blockid, _)| (tx, blockid)),
                );
            }

            json_response(prepare_txs(txs, query, config), TTL_SHORT)
        }

        (
            &Method::GET,
            Some(script_type @ &"address"),
            Some(script_str),
            Some(&"txs"),
            Some(&"chain"),
            last_seen_txid,
        )
        | (
            &Method::GET,
            Some(script_type @ &"scripthash"),
            Some(script_str),
            Some(&"txs"),
            Some(&"chain"),
            last_seen_txid,
        ) => {
            let script_hash = to_scripthash(script_type, script_str, config.network_type)?;
            let last_seen_txid = last_seen_txid.and_then(|txid| txid.parse::<Txid>().ok());
            let max_txs = query_params
                .get("max_txs")
                .and_then(|s| s.parse::<usize>().ok())
                .unwrap_or(config.rest_default_chain_txs_per_page);

            let mut txs = query
                .chain()
                .history(&script_hash[..], last_seen_txid.as_ref(), None, max_txs)
                .map(|res| res.map(|(tx, blockid, tx_position)| (tx, Some(blockid), tx_position)))
                .collect::<Result<Vec<_>, _>>()?;
            txs.sort_unstable_by(|(_, blockid1, tx_position1), (_, blockid2, tx_position2)| {
                blockid2
                    .as_ref()
                    .map(|b| b.height)
                    .cmp(&blockid1.as_ref().map(|b| b.height))
                    .then_with(|| tx_position2.cmp(tx_position1))
            });
            json_response(
                prepare_txs(
                    txs.into_iter()
                        .map(|(tx, blockid, _)| (tx, blockid))
                        .collect(),
                    query,
                    config,
                ),
                TTL_SHORT,
            )
        }
        (
            &Method::GET,
            Some(script_type @ &"address"),
            Some(script_str),
            Some(&"txs"),
            Some(&"summary"),
            last_seen_txid,
        )
        | (
            &Method::GET,
            Some(script_type @ &"scripthash"),
            Some(script_str),
            Some(&"txs"),
            Some(&"summary"),
            last_seen_txid,
        ) => {
            let script_hash = to_scripthash(script_type, script_str, config.network_type)?;
            let last_seen_txid = last_seen_txid.and_then(|txid| txid.parse::<Txid>().ok());
            let max_txs = cmp::min(
                config.rest_default_max_address_summary_txs,
                query_params
                    .get("max_txs")
                    .and_then(|s| s.parse::<usize>().ok())
                    .unwrap_or(config.rest_default_max_address_summary_txs),
            );

            let last_seen_txid_location = if let Some(txid) = &last_seen_txid {
                find_txid(txid, &query.mempool(), query.chain())
            } else {
                TxidLocation::Mempool
            };

            let confirmed_block_height = match last_seen_txid_location {
                TxidLocation::Mempool => None,
                TxidLocation::None => {
                    return Err(HttpError(
                        StatusCode::UNPROCESSABLE_ENTITY,
                        String::from("after_txid not found"),
                    ));
                }
                TxidLocation::Chain(height) => Some(height),
            };

            let summary = query.chain().summary(
                &script_hash[..],
                last_seen_txid.as_ref(),
                confirmed_block_height,
                max_txs,
            );

            json_response(summary, TTL_SHORT)
        }
        (
            &Method::POST,
            Some(script_types @ &"addresses"),
            Some(&"txs"),
            Some(&"summary"),
            last_seen_txid,
            None,
        )
        | (
            &Method::POST,
            Some(script_types @ &"scripthashes"),
            Some(&"txs"),
            Some(&"summary"),
            last_seen_txid,
            None,
        ) => {
            let script_type = match *script_types {
                "addresses" => "address",
                "scripthashes" => "scripthash",
                _ => "",
            };

            if multi_address_too_long(&body) {
                return Err(HttpError(
                    StatusCode::UNPROCESSABLE_ENTITY,
                    String::from("body too long"),
                ));
            }

            let script_hashes: Vec<String> =
                serde_json::from_slice(&body).map_err(|err| HttpError::from(err.to_string()))?;

            if script_hashes.len() > MULTI_ADDRESS_LIMIT {
                return Err(HttpError(
                    StatusCode::UNPROCESSABLE_ENTITY,
                    String::from("body too long"),
                ));
            }

            let script_hashes: Vec<[u8; 32]> = script_hashes
                .iter()
                .filter_map(|script_str| {
                    to_scripthash(script_type, script_str, config.network_type).ok()
                })
                .collect();

            let last_seen_txid = last_seen_txid.and_then(|txid| txid.parse::<Txid>().ok());
            let max_txs = cmp::min(
                config.rest_default_max_address_summary_txs,
                query_params
                    .get("max_txs")
                    .and_then(|s| s.parse::<usize>().ok())
                    .unwrap_or(config.rest_default_max_address_summary_txs),
            );

            let last_seen_txid_location = if let Some(txid) = &last_seen_txid {
                find_txid(txid, &query.mempool(), query.chain())
            } else {
                TxidLocation::Mempool
            };

            let confirmed_block_height = match last_seen_txid_location {
                TxidLocation::Mempool => None,
                TxidLocation::None => {
                    return Err(HttpError(
                        StatusCode::UNPROCESSABLE_ENTITY,
                        String::from("after_txid not found"),
                    ));
                }
                TxidLocation::Chain(height) => Some(height),
            };

            let summary = query.chain().summary_group(
                &script_hashes,
                last_seen_txid.as_ref(),
                confirmed_block_height,
                max_txs,
            );

            json_response(summary, TTL_SHORT)
        }
        (
            &Method::GET,
            Some(script_type @ &"address"),
            Some(script_str),
            Some(&"txs"),
            Some(&"mempool"),
            None,
        )
        | (
            &Method::GET,
            Some(script_type @ &"scripthash"),
            Some(script_str),
            Some(&"txs"),
            Some(&"mempool"),
            None,
        ) => {
            let script_hash = to_scripthash(script_type, script_str, config.network_type)?;
            let max_txs = query_params
                .get("max_txs")
                .and_then(|s| s.parse::<usize>().ok())
                .unwrap_or(config.rest_default_max_mempool_txs);

            let txs = query
                .mempool()
                .history(&script_hash[..], None, max_txs)
                .into_iter()
                .map(|tx| (tx, None))
                .collect();

            json_response(prepare_txs(txs, query, config), TTL_SHORT)
        }

        (
            &Method::GET,
            Some(script_type @ &"address"),
            Some(script_str),
            Some(&"utxo"),
            None,
            None,
        )
        | (
            &Method::GET,
            Some(script_type @ &"scripthash"),
            Some(script_str),
            Some(&"utxo"),
            None,
            None,
        ) => {
            let script_hash = to_scripthash(script_type, script_str, config.network_type)?;
            let utxos: Vec<UtxoValue> = query
                .utxo(&script_hash[..])?
                .into_iter()
                .map(UtxoValue::from)
                .collect();
            // XXX paging?
            json_response(utxos, TTL_SHORT)
        }
        (&Method::GET, Some(&"address-prefix"), Some(prefix), None, None, None) => {
            if !config.address_search {
                return Err(HttpError::from("address search disabled".to_string()));
            }
            let results = query.chain().address_search(prefix, ADDRESS_SEARCH_LIMIT);
            json_response(results, TTL_SHORT)
        }
        (&Method::GET, Some(&"tx"), Some(hash), None, None, None) => {
            let hash = hash.parse::<Txid>()?;
            let tx = query
                .lookup_txn(&hash)
                .ok_or_else(|| HttpError::not_found("Transaction not found".to_string()))?;
            let blockid = query.chain().tx_confirming_block(&hash);
            let ttl = ttl_by_depth(blockid.as_ref().map(|b| b.height), query);

            let mut tx = prepare_txs(vec![(tx, blockid)], query, config);

            if tx.is_empty() {
                http_message(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Transaction missing prevouts",
                    0,
                )
            } else {
                json_response(tx.remove(0), ttl)
            }
        }
        (&Method::POST, Some(&INTERNAL_PREFIX), Some(&"txs"), None, None, None) => {
            let txid_strings: Vec<String> =
                serde_json::from_slice(&body).map_err(|err| HttpError::from(err.to_string()))?;

            match txid_strings
                .into_iter()
                .map(|txid| txid.parse::<Txid>())
                .collect::<Result<Vec<Txid>, _>>()
            {
                Ok(txids) => {
                    let txs: Vec<(Transaction, Option<BlockId>)> = txids
                        .iter()
                        .filter_map(|txid| {
                            query
                                .lookup_txn(txid)
                                .map(|tx| (tx, query.chain().tx_confirming_block(txid)))
                        })
                        .collect();
                    json_response(prepare_txs(txs, query, config), 0)
                }
                Err(err) => http_message(StatusCode::BAD_REQUEST, err.to_string(), 0),
            }
        }
        (&Method::GET, Some(&"tx"), Some(hash), Some(out_type @ &"hex"), None, None)
        | (&Method::GET, Some(&"tx"), Some(hash), Some(out_type @ &"raw"), None, None) => {
            let hash = hash.parse::<Txid>()?;
            let rawtx = query
                .lookup_raw_txn(&hash)
                .ok_or_else(|| HttpError::not_found("Transaction not found".to_string()))?;

            let (content_type, body) = match *out_type {
                "raw" => ("application/octet-stream", Body::from(rawtx)),
                "hex" => ("text/plain", Body::from(hex::encode(rawtx))),
                _ => unreachable!(),
            };
            let ttl = ttl_by_depth(query.get_tx_status(&hash).block_height, query);

            Ok(Response::builder()
                .status(StatusCode::OK)
                .header("Content-Type", content_type)
                .header("Cache-Control", format!("public, max-age={:}", ttl))
                .body(body)
                .unwrap())
        }
        (&Method::GET, Some(&"tx"), Some(hash), Some(&"status"), None, None) => {
            let hash = hash.parse::<Txid>()?;
            let status = query.get_tx_status(&hash);
            let ttl = ttl_by_depth(status.block_height, query);
            json_response(status, ttl)
        }

        (&Method::GET, Some(&"tx"), Some(hash), Some(&"merkle-proof"), None, None) => {
            let hash = hash.parse::<Txid>()?;
            let blockid = query.chain().tx_confirming_block(&hash).ok_or_else(|| {
                HttpError::not_found("Transaction not found or is unconfirmed".to_string())
            })?;
            let (merkle, pos) =
                electrum_merkle::get_tx_merkle_proof(query.chain(), &hash, &blockid.hash)?;
            let merkle: Vec<String> = merkle.into_iter().map(|txid| txid.to_string()).collect();
            let ttl = ttl_by_depth(Some(blockid.height), query);
            json_response(
                json!({ "block_height": blockid.height, "merkle": merkle, "pos": pos }),
                ttl,
            )
        }
        #[cfg(not(feature = "liquid"))]
        (&Method::GET, Some(&"tx"), Some(hash), Some(&"merkleblock-proof"), None, None) => {
            let hash = hash.parse::<Txid>()?;

            let merkleblock = query.chain().get_merkleblock_proof(&hash).ok_or_else(|| {
                HttpError::not_found("Transaction not found or is unconfirmed".to_string())
            })?;

            let height = query
                .chain()
                .height_by_hash(&merkleblock.header.block_hash());

            http_message(
                StatusCode::OK,
                hex::encode(encode::serialize(&merkleblock)),
                ttl_by_depth(height, query),
            )
        }
        (&Method::GET, Some(&"tx"), Some(hash), Some(&"outspend"), Some(index), None) => {
            let hash = hash.parse::<Txid>()?;
            let outpoint = OutPoint {
                txid: hash,
                vout: index.parse::<u32>()?,
            };
            let spend = query
                .lookup_spend(&outpoint)
                .map_or_else(SpendingValue::default, SpendingValue::from);
            let ttl = ttl_by_depth(
                spend.status.as_ref().and_then(|status| status.block_height),
                query,
            );
            json_response(spend, ttl)
        }
        (&Method::GET, Some(&"tx"), Some(hash), Some(&"outspends"), None, None) => {
            let hash = hash.parse::<Txid>()?;
            let tx = query
                .lookup_txn(&hash)
                .ok_or_else(|| HttpError::not_found("Transaction not found".to_string()))?;
            let spends: Vec<SpendingValue> = query
                .lookup_tx_spends(tx)
                .into_iter()
                .map(|spend| spend.map_or_else(SpendingValue::default, SpendingValue::from))
                .collect();
            // @TODO long ttl if all outputs are either spent long ago or unspendable
            json_response(spends, TTL_SHORT)
        }
        (&Method::GET, Some(&"broadcast"), None, None, None, None)
        | (&Method::POST, Some(&"tx"), None, None, None, None) => {
            // accept both POST and GET for backward compatibility.
            // GET will eventually be removed in favor of POST.
            let txhex = match method {
                Method::POST => String::from_utf8(body.to_vec())?,
                Method::GET => query_params
                    .get("tx")
                    .cloned()
                    .ok_or_else(|| HttpError::from("Missing tx".to_string()))?,
                _ => return http_message(StatusCode::METHOD_NOT_ALLOWED, "Invalid method", 0),
            };
            let txid = query.broadcast_raw(&txhex)?;
            http_message(StatusCode::OK, txid.to_string(), 0)
        }
        (&Method::POST, Some(&"txs"), Some(&"test"), None, None, None) => {
            let txhexes: Vec<String> =
                serde_json::from_str(String::from_utf8(body.to_vec())?.as_str())?;

            if txhexes.len() > 25 {
                Result::Err(HttpError::from(
                    "Exceeded maximum of 25 transactions".to_string(),
                ))?
            }

            let maxfeerate = query_params
                .get("maxfeerate")
                .map(|s| {
                    s.parse::<f64>()
                        .map_err(|_| HttpError::from("Invalid maxfeerate".to_string()))
                })
                .transpose()?;

            // pre-checks
            txhexes.iter().enumerate().try_for_each(|(index, txhex)| {
                // each transaction must be of reasonable size (more than 60 bytes, within 400kWU standardness limit)
                if !(120..800_000).contains(&txhex.len()) {
                    Result::Err(HttpError::from(format!(
                        "Invalid transaction size for item {}",
                        index
                    )))
                } else {
                    // must be a valid hex string
                    hex::decode(txhex)
                        .map_err(|_| {
                            HttpError::from(format!("Invalid transaction hex for item {}", index))
                        })
                        .map(|_| ())
                }
            })?;

            let result = query.test_mempool_accept(txhexes, maxfeerate)?;

            json_response(result, TTL_SHORT)
        }
        (&Method::POST, Some(&"txs"), Some(&"package"), None, None, None) => {
            let txhexes: Vec<String> =
                serde_json::from_str(String::from_utf8(body.to_vec())?.as_str())?;

            if txhexes.len() > 25 {
                Result::Err(HttpError::from(
                    "Exceeded maximum of 25 transactions".to_string(),
                ))?
            }

            let maxfeerate = query_params
                .get("maxfeerate")
                .map(|s| {
                    s.parse::<f64>()
                        .map_err(|_| HttpError::from("Invalid maxfeerate".to_string()))
                })
                .transpose()?;

            let maxburnamount = query_params
                .get("maxburnamount")
                .map(|s| {
                    s.parse::<f64>()
                        .map_err(|_| HttpError::from("Invalid maxburnamount".to_string()))
                })
                .transpose()?;

            // pre-checks
            txhexes.iter().enumerate().try_for_each(|(index, txhex)| {
                // each transaction must be of reasonable size (more than 60 bytes, within 400kWU standardness limit)
                if !(120..800_000).contains(&txhex.len()) {
                    Result::Err(HttpError::from(format!(
                        "Invalid transaction size for item {}",
                        index
                    )))
                } else {
                    // must be a valid hex string
                    hex::decode(txhex)
                        .map_err(|_| {
                            HttpError::from(format!("Invalid transaction hex for item {}", index))
                        })
                        .map(|_| ())
                }
            })?;

            let result = query.submit_package(txhexes, maxfeerate, maxburnamount)?;

            json_response(result, TTL_SHORT)
        }
        (&Method::GET, Some(&"txs"), Some(&"outspends"), None, None, None) => {
            let txid_strings: Vec<&str> = query_params
                .get("txids")
                .ok_or(HttpError::from("No txids specified".to_string()))?
                .as_str()
                .split(',')
                .collect();

            if txid_strings.len() > 50 {
                return http_message(StatusCode::BAD_REQUEST, "Too many txids requested", 0);
            }

            let spends: Vec<Vec<SpendingValue>> = txid_strings
                .into_iter()
                .map(|txid_str| {
                    txid_str
                        .parse::<Txid>()
                        .ok()
                        .and_then(|txid| query.lookup_txn(&txid))
                        .map_or_else(Vec::new, |tx| {
                            query
                                .lookup_tx_spends(tx)
                                .into_iter()
                                .map(|spend| {
                                    spend.map_or_else(SpendingValue::default, SpendingValue::from)
                                })
                                .collect()
                        })
                })
                .collect();

            json_response(spends, TTL_SHORT)
        }
        (
            &Method::POST,
            Some(&INTERNAL_PREFIX),
            Some(&"txs"),
            Some(&"outspends"),
            Some(&"by-txid"),
            None,
        ) => {
            let txid_strings: Vec<String> =
                serde_json::from_slice(&body).map_err(|err| HttpError::from(err.to_string()))?;

            let spends: Vec<Vec<SpendingValue>> = txid_strings
                .into_iter()
                .map(|txid_str| {
                    txid_str
                        .parse::<Txid>()
                        .ok()
                        .and_then(|txid| query.lookup_txn(&txid))
                        .map_or_else(Vec::new, |tx| {
                            query
                                .lookup_tx_spends(tx)
                                .into_iter()
                                .map(|spend| {
                                    spend.map_or_else(SpendingValue::default, SpendingValue::from)
                                })
                                .collect()
                        })
                })
                .collect();

            json_response(spends, TTL_SHORT)
        }
        (
            &Method::POST,
            Some(&INTERNAL_PREFIX),
            Some(&"txs"),
            Some(&"outspends"),
            Some(&"by-outpoint"),
            None,
        ) => {
            let outpoint_strings: Vec<String> =
                serde_json::from_slice(&body).map_err(|err| HttpError::from(err.to_string()))?;

            let spends: Vec<SpendingValue> = outpoint_strings
                .into_iter()
                .map(|outpoint_str| {
                    let mut parts = outpoint_str.split(':');
                    let hash_part = parts.next();
                    let index_part = parts.next();

                    if let (Some(hash), Some(index)) = (hash_part, index_part) {
                        if let (Ok(txid), Ok(vout)) = (hash.parse::<Txid>(), index.parse::<u32>()) {
                            let outpoint = OutPoint { txid, vout };
                            return query
                                .lookup_spend(&outpoint)
                                .map_or_else(SpendingValue::default, SpendingValue::from);
                        }
                    }
                    SpendingValue::default()
                })
                .collect();

            json_response(spends, TTL_SHORT)
        }

        (&Method::GET, Some(&"mempool"), None, None, None, None) => {
            json_response(query.mempool().backlog_stats(), TTL_SHORT)
        }
        (&Method::GET, Some(&"mempool"), Some(&"txids"), None, None, None) => {
            json_response(query.mempool().txids(), TTL_SHORT)
        }
        (&Method::GET, Some(&"mempool"), Some(&"txids"), Some(&"page"), last_seen_txid, None) => {
            let last_seen_txid = last_seen_txid.and_then(|txid| txid.parse::<Txid>().ok());
            let max_txs = query_params
                .get("max_txs")
                .and_then(|s| s.parse::<usize>().ok())
                .unwrap_or(config.rest_max_mempool_txid_page_size);
            json_response(
                query.mempool().txids_page(max_txs, last_seen_txid),
                TTL_SHORT,
            )
        }
        (
            &Method::GET,
            Some(&INTERNAL_PREFIX),
            Some(&"mempool"),
            Some(&"txs"),
            Some(&"all"),
            None,
        ) => {
            let txs = query
                .mempool()
                .txs()
                .into_iter()
                .map(|tx| (tx, None))
                .collect();

            json_response(prepare_txs(txs, query, config), TTL_SHORT)
        }
        (&Method::POST, Some(&INTERNAL_PREFIX), Some(&"mempool"), Some(&"txs"), None, None) => {
            let txid_strings: Vec<String> =
                serde_json::from_slice(&body).map_err(|err| HttpError::from(err.to_string()))?;

            match txid_strings
                .into_iter()
                .map(|txid| txid.parse::<Txid>())
                .collect::<Result<Vec<Txid>, _>>()
            {
                Ok(txids) => {
                    let txs: Vec<(Transaction, Option<BlockId>)> = {
                        let mempool = query.mempool();
                        txids
                            .iter()
                            .filter_map(|txid| mempool.lookup_txn(txid).map(|tx| (tx, None)))
                            .collect()
                    };

                    json_response(prepare_txs(txs, query, config), 0)
                }
                Err(err) => http_message(StatusCode::BAD_REQUEST, err.to_string(), 0),
            }
        }
        (
            &Method::GET,
            Some(&INTERNAL_PREFIX),
            Some(&"mempool"),
            Some(&"txs"),
            last_seen_txid,
            None,
        ) => {
            let last_seen_txid = last_seen_txid.and_then(|txid| txid.parse::<Txid>().ok());
            let max_txs = query_params
                .get("max_txs")
                .and_then(|s| s.parse::<usize>().ok())
                .unwrap_or(config.rest_max_mempool_page_size);
            let txs = query
                .mempool()
                .txs_page(max_txs, last_seen_txid)
                .into_iter()
                .map(|tx| (tx, None))
                .collect();

            json_response(prepare_txs(txs, query, config), TTL_SHORT)
        }
        (&Method::GET, Some(&"mempool"), Some(&"recent"), None, None, None) => {
            let mempool = query.mempool();
            let recent = mempool.recent_txs_overview();
            json_response(recent, TTL_MEMPOOL_RECENT)
        }

        (&Method::GET, Some(&"fee-estimates"), None, None, None, None) => {
            json_response(query.estimate_fee_map(), TTL_SHORT)
        }

        #[cfg(feature = "liquid")]
        (&Method::GET, Some(&"assets"), Some(&"registry"), Some(&"search"), None, None) => {
            let search = query_params.get("q").map(|q| q.trim()).unwrap_or("");
            let assets = if search.is_empty() {
                vec![]
            } else if search.chars().count() > ASSETS_SEARCH_MAX_QUERY_LEN {
                return Err(HttpError(
                    StatusCode::BAD_REQUEST,
                    "search query too long".to_string(),
                ));
            } else {
                let limit = query_params
                    .get("limit")
                    .and_then(|n| n.parse::<usize>().ok())
                    .unwrap_or(ASSETS_SEARCH_DEFAULT_LIMIT)
                    .min(ASSETS_SEARCH_MAX_LIMIT);

                query
                    .search_registry_assets(search, limit, AssetRegistrySearchResult::new)
                    .map_err(|e| {
                        HttpError(StatusCode::SERVICE_UNAVAILABLE, e.description().to_string())
                    })?
            };

            Ok(Response::builder()
                // Disable caching because we don't currently support caching with query string params
                .header("Cache-Control", "no-store")
                .header("Content-Type", "application/json")
                .body(Body::from(serde_json::to_string(&assets)?))
                .unwrap())
        }

        #[cfg(feature = "liquid")]
        (&Method::GET, Some(&"assets"), Some(&"registry"), None, None, None) => {
            let start_index: usize = query_params
                .get("start_index")
                .and_then(|n| n.parse().ok())
                .unwrap_or(0);

            let limit: usize = query_params
                .get("limit")
                .and_then(|n| n.parse().ok())
                .map(|n: usize| n.min(ASSETS_MAX_PER_PAGE))
                .unwrap_or(ASSETS_PER_PAGE);

            let sorting = AssetSorting::from_query_params(&query_params)?;

            let (total_num, assets) = query.list_registry_assets(start_index, limit, sorting)?;

            Ok(Response::builder()
                // Disable caching because we don't currently support caching with query string params
                .header("Cache-Control", "no-store")
                .header("Content-Type", "application/json")
                .header("X-Total-Results", total_num.to_string())
                .body(Body::from(serde_json::to_string(&assets)?))
                .unwrap())
        }

        #[cfg(feature = "liquid")]
        (&Method::GET, Some(&"assets"), Some(&"registry"), Some(asset_str), None, None) => {
            let asset_id = AssetId::from_str(asset_str)?;
            let registry_entry = query
                .lookup_registry_asset(&asset_id)
                .map_err(|e| {
                    HttpError(StatusCode::SERVICE_UNAVAILABLE, e.description().to_string())
                })?
                .ok_or_else(|| {
                    HttpError::not_found("Asset id not found in registry".to_string())
                })?;

            json_response(registry_entry, TTL_SHORT)
        }

        #[cfg(feature = "liquid")]
        (&Method::GET, Some(&"asset"), Some(asset_str), None, None, None) => {
            let asset_id = AssetId::from_str(asset_str)?;
            let asset_entry = query
                .lookup_asset(&asset_id)?
                .ok_or_else(|| HttpError::not_found("Asset id not found".to_string()))?;

            json_response(asset_entry, TTL_SHORT)
        }

        #[cfg(feature = "liquid")]
        (&Method::GET, Some(&"asset"), Some(asset_str), Some(&"txs"), None, None) => {
            let asset_id = AssetId::from_str(asset_str)?;

            let mut txs = vec![];

            txs.extend(
                query
                    .mempool()
                    .asset_history(&asset_id, config.rest_default_max_mempool_txs)
                    .into_iter()
                    .map(|tx| (tx, None)),
            );

            let mut confirmed_txs = query
                .chain()
                .asset_history(&asset_id, None, config.rest_default_chain_txs_per_page)
                .map(|res| res.map(|(tx, blockid, tx_position)| (tx, Some(blockid), tx_position)))
                .collect::<Result<Vec<_>, _>>()?;
            confirmed_txs.sort_unstable_by(
                |(_, blockid1, tx_position1), (_, blockid2, tx_position2)| {
                    blockid2
                        .as_ref()
                        .map(|b| b.height)
                        .cmp(&blockid1.as_ref().map(|b| b.height))
                        .then_with(|| tx_position2.cmp(tx_position1))
                },
            );
            txs.extend(
                confirmed_txs
                    .into_iter()
                    .map(|(tx, blockid, _)| (tx, blockid)),
            );

            json_response(prepare_txs(txs, query, config), TTL_SHORT)
        }

        #[cfg(feature = "liquid")]
        (
            &Method::GET,
            Some(&"asset"),
            Some(asset_str),
            Some(&"txs"),
            Some(&"chain"),
            last_seen_txid,
        ) => {
            let asset_id = AssetId::from_str(asset_str)?;
            let last_seen_txid = last_seen_txid.and_then(|txid| txid.parse::<Txid>().ok());

            let mut txs = query
                .chain()
                .asset_history(
                    &asset_id,
                    last_seen_txid.as_ref(),
                    config.rest_default_chain_txs_per_page,
                )
                .map(|res| res.map(|(tx, blockid, tx_position)| (tx, Some(blockid), tx_position)))
                .collect::<Result<Vec<_>, _>>()?;

            txs.sort_unstable_by(|(_, blockid1, tx_position1), (_, blockid2, tx_position2)| {
                blockid2
                    .as_ref()
                    .map(|b| b.height)
                    .cmp(&blockid1.as_ref().map(|b| b.height))
                    .then_with(|| tx_position2.cmp(tx_position1))
            });

            json_response(
                prepare_txs(
                    txs.into_iter()
                        .map(|(tx, blockid, _)| (tx, blockid))
                        .collect(),
                    query,
                    config,
                ),
                TTL_SHORT,
            )
        }

        #[cfg(feature = "liquid")]
        (&Method::GET, Some(&"asset"), Some(asset_str), Some(&"txs"), Some(&"mempool"), None) => {
            let asset_id = AssetId::from_str(asset_str)?;

            let txs = query
                .mempool()
                .asset_history(&asset_id, config.rest_default_max_mempool_txs)
                .into_iter()
                .map(|tx| (tx, None))
                .collect();

            json_response(prepare_txs(txs, query, config), TTL_SHORT)
        }

        #[cfg(feature = "liquid")]
        (&Method::GET, Some(&"asset"), Some(asset_str), Some(&"supply"), param, None) => {
            let asset_id = AssetId::from_str(asset_str)?;
            let asset_entry = query
                .lookup_asset(&asset_id)?
                .ok_or_else(|| HttpError::not_found("Asset id not found".to_string()))?;

            let supply = asset_entry
                .supply()
                .ok_or_else(|| HttpError::from("Asset supply is blinded".to_string()))?;
            let precision = asset_entry.precision();

            if param == Some(&"decimal") && precision > 0 {
                let supply_dec = supply as f64 / 10u32.pow(precision.into()) as f64;
                http_message(StatusCode::OK, supply_dec.to_string(), TTL_SHORT)
            } else {
                http_message(StatusCode::OK, supply.to_string(), TTL_SHORT)
            }
        }

        _ => Err(HttpError::not_found(format!(
            "endpoint does not exist {:?}",
            uri.path()
        ))),
    }
}

fn http_message<T>(status: StatusCode, message: T, ttl: u32) -> Result<Response<Body>, HttpError>
where
    T: Into<Body>,
{
    Ok(Response::builder()
        .status(status)
        .header("Content-Type", "text/plain")
        .header("Cache-Control", format!("public, max-age={:}", ttl))
        .body(message.into())
        .unwrap())
}

fn json_response<T: Serialize>(value: T, ttl: u32) -> Result<Response<Body>, HttpError> {
    let value = serde_json::to_string(&value)?;
    Ok(Response::builder()
        .header("Content-Type", "application/json")
        .header("Cache-Control", format!("public, max-age={:}", ttl))
        .body(Body::from(value))
        .unwrap())
}

fn json_response_no_store<T: Serialize>(value: T) -> Result<Response<Body>, HttpError> {
    let value = serde_json::to_string(&value)?;
    Ok(Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "application/json")
        .header("Cache-Control", "no-store")
        .body(Body::from(value))
        .unwrap())
}

/// When mining REST is off, GET /block-template is forbidden.
fn mining_rest_disabled_error(enable_mining_rest: bool) -> Option<HttpError> {
    if enable_mining_rest {
        None
    } else {
        Some(HttpError::forbidden(
            "mining REST endpoints are disabled".to_string(),
        ))
    }
}

fn handle_block_template(query: &Query, config: &Config) -> Result<Response<Body>, HttpError> {
    if let Some(err) = mining_rest_disabled_error(config.enable_mining_rest) {
        return Err(err);
    }
    #[cfg(not(feature = "liquid"))]
    {
        let value = query.getblocktemplate()?;
        json_response_no_store(value)
    }
    #[cfg(feature = "liquid")]
    {
        let hex = query.getnewblockhex()?;
        json_response_no_store(hex)
    }
}

// fn json_maybe_error_response<T: Serialize>(
//     value: Result<T, errors::Error>,
//     ttl: u32,
// ) -> Result<Response<Body>, HttpError> {
//     let response = Response::builder()
//         .header("Content-Type", "application/json")
//         .header("Cache-Control", format!("public, max-age={:}", ttl));
//     Ok(match value {
//         Ok(v) => response
//             .body(Body::from(serde_json::to_string(&v)?))
//             .expect("Valid http response"),
//         Err(e) => response
//             .status(500)
//             .body(Body::from(serde_json::to_string(
//                 &json!({ "error": e.to_string() }),
//             )?))
//             .expect("Valid http response"),
//     })
// }

fn blocks(
    query: &Query,
    config: &Config,
    start_height: Option<usize>,
) -> Result<Response<Body>, HttpError> {
    let mut values = Vec::new();
    let mut current_hash = match start_height {
        Some(height) => *query
            .chain()
            .header_by_height(height)
            .ok_or_else(|| HttpError::not_found("Block not found".to_string()))?
            .hash(),
        None => query.chain().best_hash(),
    };

    let zero = [0u8; 32];
    for _ in 0..config.rest_default_block_limit {
        let blockhm = query
            .chain()
            .get_block_with_meta(&current_hash)
            .ok_or_else(|| HttpError::not_found("Block not found".to_string()))?;
        current_hash = blockhm.header_entry.header().prev_blockhash;

        #[allow(unused_mut)]
        let mut value = BlockValue::new(blockhm);

        #[cfg(feature = "liquid")]
        {
            // exclude ExtData in block list view
            value.ext = None;
        }
        values.push(value);

        if current_hash[..] == zero[..] {
            break;
        }
    }
    json_response(values, TTL_SHORT)
}

fn to_scripthash(
    script_type: &str,
    script_str: &str,
    network: Network,
) -> Result<FullHash, HttpError> {
    match script_type {
        "address" => address_to_scripthash(script_str, network),
        "scripthash" => parse_scripthash(script_str),
        _ => bail!("Invalid script type".to_string()),
    }
}

fn address_to_scripthash(addr: &str, network: Network) -> Result<FullHash, HttpError> {
    #[cfg(not(feature = "liquid"))]
    let addr = {
        use bitcoin::address::NetworkUnchecked;
        let unchecked: bitcoin::Address<NetworkUnchecked> = addr.parse()?;
        let bnetwork = bitcoin::Network::from(network);
        // Testnet, Regtest and Signet all share the same version bytes,
        // so we need to allow require_network to succeed for all testnet-family networks
        let testnet_family = [
            bitcoin::Network::Testnet,
            bitcoin::Network::Regtest,
            bitcoin::Network::Signet,
            bitcoin::Network::Testnet4,
        ];
        if testnet_family.contains(&bnetwork) {
            // Try each testnet-family network
            testnet_family
                .iter()
                .find_map(|&net| unchecked.clone().require_network(net).ok())
                .ok_or_else(|| HttpError::from("Address on invalid network".to_string()))?
        } else {
            unchecked
                .require_network(bnetwork)
                .map_err(|_| HttpError::from("Address on invalid network".to_string()))?
        }
    };
    #[cfg(feature = "liquid")]
    let addr = {
        let addr = address::Address::parse_with_params(addr, network.address_params())?;
        if addr.params != network.address_params() {
            return Err(HttpError::from("Address on invalid network".to_string()));
        }
        addr
    };

    Ok(compute_script_hash(&addr.script_pubkey()))
}

fn parse_scripthash(scripthash: &str) -> Result<FullHash, HttpError> {
    let bytes = hex::decode(scripthash)?;
    if bytes.len() != 32 {
        Err(HttpError::from("Invalid scripthash".to_string()))
    } else {
        Ok(full_hash(&bytes))
    }
}

#[inline]
fn multi_address_too_long(body: &hyper::body::Bytes) -> bool {
    // ("",) (3) (quotes and comma between each entry)
    // (\n    ) (5) (allows for pretty printed JSON with 4 space indent)
    // The opening [] and whatnot don't need to be accounted for, we give more than enough leeway
    // p2tr and p2wsh are 55 length, scripthashes are 64.
    body.len() > (8 + 64) * MULTI_ADDRESS_LIMIT
}

#[derive(Debug)]
struct HttpError(StatusCode, String);

impl HttpError {
    fn not_found(msg: String) -> Self {
        HttpError(StatusCode::NOT_FOUND, msg)
    }

    fn forbidden(msg: String) -> Self {
        HttpError(StatusCode::FORBIDDEN, msg)
    }
}

impl From<String> for HttpError {
    fn from(msg: String) -> Self {
        HttpError(StatusCode::BAD_REQUEST, msg)
    }
}
impl From<ParseIntError> for HttpError {
    fn from(_e: ParseIntError) -> Self {
        //HttpError::from(e.description().to_string())
        HttpError::from("Invalid number".to_string())
    }
}
impl From<FromHexError> for HttpError {
    fn from(_e: FromHexError) -> Self {
        //HttpError::from(e.description().to_string())
        HttpError::from("Invalid hex string".to_string())
    }
}
impl From<bitcoin::hashes::hex::HexToArrayError> for HttpError {
    fn from(_e: bitcoin::hashes::hex::HexToArrayError) -> Self {
        HttpError::from("Invalid hex hash".to_string())
    }
}
impl From<bitcoin::address::ParseError> for HttpError {
    fn from(_e: bitcoin::address::ParseError) -> Self {
        //HttpError::from(e.description().to_string())
        HttpError::from("Invalid Bitcoin address".to_string())
    }
}
impl From<errors::Error> for HttpError {
    fn from(e: errors::Error) -> Self {
        warn!("errors::Error: {:?}", e);
        // Downstream daemon failures on REST proxy paths are not client 400s.
        match e.kind() {
            errors::ErrorKind::DaemonBusy(_) => {
                return HttpError(StatusCode::SERVICE_UNAVAILABLE, e.to_string());
            }
            errors::ErrorKind::DaemonUnavailable(_) => {
                return HttpError(StatusCode::GATEWAY_TIMEOUT, e.to_string());
            }
            errors::ErrorKind::Connection(msg) => {
                return HttpError(StatusCode::GATEWAY_TIMEOUT, msg.clone());
            }
            _ => {}
        }
        match e.description().to_string().as_ref() {
            "getblock RPC error: {\"code\":-5,\"message\":\"Block not found\"}" => {
                HttpError::not_found("Block not found".to_string())
            }
            _ => HttpError::from(e.to_string()),
        }
    }
}
impl From<serde_json::Error> for HttpError {
    fn from(e: serde_json::Error) -> Self {
        HttpError::from(e.to_string())
    }
}
impl From<encode::Error> for HttpError {
    fn from(e: encode::Error) -> Self {
        HttpError::from(e.to_string())
    }
}
impl From<std::string::FromUtf8Error> for HttpError {
    fn from(e: std::string::FromUtf8Error) -> Self {
        HttpError::from(e.to_string())
    }
}
#[cfg(feature = "liquid")]
impl From<address::AddressError> for HttpError {
    fn from(e: address::AddressError) -> Self {
        HttpError::from(e.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::{TxidLocation, confirmed_after_txid, mining_rest_disabled_error};
    use crate::chain::Txid;
    use crate::rest::HttpError;
    use hyper::StatusCode;
    use nostr::prelude::*;
    use serde_json::Value;
    use std::collections::HashMap;

    #[test]
    fn mining_rest_disabled_is_forbidden() {
        let err = mining_rest_disabled_error(false).expect("forbidden when mining REST is off");
        assert_eq!(err.0, StatusCode::FORBIDDEN);
        assert!(mining_rest_disabled_error(true).is_none());
    }

    /// Named contract: REST paths that proxy to bitcoind (broadcast, package,
    /// block-template, testmempoolaccept) map daemon occupancy to HTTP 503 and
    /// a missing or timed-out daemon to HTTP 504. Electrum JSON-RPC stays JSON
    /// on the wire, not these HTTP statuses.
    #[test]
    fn daemon_proxy_failures_map_to_503_and_504() {
        use crate::errors::{Error, ErrorKind};

        let busy = HttpError::from(Error::from(ErrorKind::DaemonBusy(
            "all client RPC slots are in use".to_string(),
        )));
        assert_eq!(busy.0, StatusCode::SERVICE_UNAVAILABLE);

        let unavailable = HttpError::from(Error::from(ErrorKind::DaemonUnavailable(
            "sendrawtransaction failed".to_string(),
        )));
        assert_eq!(unavailable.0, StatusCode::GATEWAY_TIMEOUT);

        let rejected = HttpError::from("min relay fee not met".to_string());
        assert_eq!(rejected.0, StatusCode::BAD_REQUEST);
    }

    fn now() -> u64 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs()
    }

    fn npub_of(keys: &Keys) -> String {
        keys.public_key().to_bech32().unwrap()
    }

    fn write_allowlist(dir: &tempfile::TempDir, npubs: &[&str]) -> std::path::PathBuf {
        let path = dir.path().join("allowlist");
        let mut body = String::new();
        for n in npubs {
            body.push_str(n);
            body.push('\n');
        }
        std::fs::write(&path, body).unwrap();
        path
    }

    fn sign_header(
        keys: &nostr::Keys,
        method: nostr::nips::nip98::HttpMethod,
        url: &str,
        created_at: u64,
    ) -> String {
        use nostr::event::EventBuilder;
        use nostr::nips::nip98::HttpData;
        use nostr::prelude::*;
        let parsed = nostr::Url::parse(url).expect("url");
        let data = HttpData::new(parsed, method);
        let event = EventBuilder::http_auth(data)
            .custom_created_at(Timestamp::from(created_at))
            .sign_with_keys(keys)
            .expect("sign");
        let json = serde_json::to_vec(&event).expect("json");
        format!("Nostr {}", base64::encode(&json))
    }

    /// Named contract: HTTP and WS gate use the live allowlist. Localhost is
    /// not an exception. Empty allowlist is 401 except public-health tip routes.
    #[test]
    fn http_ws_gate_uses_live_allowlist() {
        use super::{HttpAuthOutcome, http_ws_auth_gate};
        use crate::auth::Allowlist;
        use nostr::nips::nip98::HttpMethod;

        let keys = Keys::generate();
        let dir = tempfile::tempdir().unwrap();
        let path = write_allowlist(&dir, &[&npub_of(&keys)]);
        let allow = Allowlist::load(&path).unwrap();
        let url = "http://127.0.0.1:3000/api/v1/ws";
        let t = now();
        let header = sign_header(&keys, HttpMethod::GET, url, t);
        let out = http_ws_auth_gate(
            "GET",
            "/api/v1/ws",
            Some(&header),
            url,
            b"",
            t,
            false,
            &allow,
        )
        .expect("listed npub on WS upgrade");
        match out {
            HttpAuthOutcome::Pubkey(_) => {}
            other => panic!("expected pubkey, got {:?}", other),
        }

        std::fs::write(&path, "").unwrap();
        allow.reload().unwrap();
        let header2 = sign_header(&keys, HttpMethod::GET, url, now());
        let err = http_ws_auth_gate(
            "GET",
            "/api/v1/ws",
            Some(&header2),
            url,
            b"",
            now(),
            false,
            &allow,
        )
        .unwrap_err();
        assert_eq!(err, crate::auth::AuthError::NotAllowlisted);
    }

    #[test]
    fn http_ws_gate_empty_allowlist_is_401_even_localhost() {
        use super::http_ws_auth_gate;
        use crate::auth::Allowlist;
        use nostr::nips::nip98::HttpMethod;

        let keys = Keys::generate();
        let dir = tempfile::tempdir().unwrap();
        let path = write_allowlist(&dir, &[]);
        let allow = Allowlist::load(&path).unwrap();
        let url = "http://127.0.0.1/blocks/tip/hash";
        let header = sign_header(&keys, HttpMethod::GET, url, now());
        let err = http_ws_auth_gate(
            "GET",
            "/blocks/tip/hash",
            Some(&header),
            url,
            b"",
            now(),
            false,
            &allow,
        )
        .unwrap_err();
        assert_eq!(err, crate::auth::AuthError::NotAllowlisted);

        let err = http_ws_auth_gate(
            "GET",
            "/tx/abc",
            None,
            "http://127.0.0.1/tx/abc",
            b"",
            now(),
            false,
            &allow,
        )
        .unwrap_err();
        assert_eq!(err, crate::auth::AuthError::MissingHeader);
    }

    #[test]
    fn http_ws_gate_public_health_allows_tip_without_auth() {
        use super::{HttpAuthOutcome, http_ws_auth_gate};
        use crate::auth::Allowlist;

        let dir = tempfile::tempdir().unwrap();
        let path = write_allowlist(&dir, &[]);
        let allow = Allowlist::load(&path).unwrap();
        let out = http_ws_auth_gate(
            "GET",
            "/blocks/tip/hash",
            None,
            "http://example/blocks/tip/hash",
            b"",
            now(),
            true,
            &allow,
        )
        .unwrap();
        assert_eq!(out, HttpAuthOutcome::PublicHealth);
        let out = http_ws_auth_gate(
            "GET",
            "/blocks/tip/height",
            None,
            "http://example/blocks/tip/height",
            b"",
            now(),
            true,
            &allow,
        )
        .unwrap();
        assert_eq!(out, HttpAuthOutcome::PublicHealth);
        let err = http_ws_auth_gate(
            "GET",
            "/block-template",
            None,
            "http://example/block-template",
            b"",
            now(),
            true,
            &allow,
        )
        .unwrap_err();
        assert_eq!(err, crate::auth::AuthError::MissingHeader);
    }

    #[test]
    fn reconstruct_absolute_url_uses_host_and_forwarded_proto() {
        use super::reconstruct_absolute_url;
        assert_eq!(
            reconstruct_absolute_url(Some("https"), Some("splora.example"), "/electrum"),
            "https://splora.example/electrum"
        );
        assert_eq!(
            reconstruct_absolute_url(Some("https, http"), Some("splora.example"), "/api/v1/ws"),
            "https://splora.example/api/v1/ws"
        );
        assert_eq!(
            reconstruct_absolute_url(None, Some("127.0.0.1:3000"), "/blocks/tip/hash"),
            "http://127.0.0.1:3000/blocks/tip/hash"
        );
    }

    /// Named contract: empty allowlist is 401 on `/api/v1/ws`; listed npub plus
    /// websocket headers is 101. This tests the upgrade decision, not a live
    /// hyper SWITCHING_PROTOCOLS handshake.
    #[test]
    fn mwck_upgrade_requires_live_allowlist() {
        use super::{HttpAuthOutcome, http_ws_auth_gate, mwck_upgrade_http_status};
        use crate::auth::Allowlist;
        use nostr::nips::nip98::HttpMethod;

        let keys = Keys::generate();
        let dir = tempfile::tempdir().unwrap();
        let empty = write_allowlist(&dir, &[]);
        let allow_empty = Allowlist::load(&empty).unwrap();
        let url = "http://127.0.0.1:3000/api/v1/ws";
        let t = now();
        let header = sign_header(&keys, HttpMethod::GET, url, t);
        let gate = http_ws_auth_gate(
            "GET",
            "/api/v1/ws",
            Some(&header),
            url,
            b"",
            t,
            false,
            &allow_empty,
        );
        assert_eq!(
            mwck_upgrade_http_status(gate, true, true),
            StatusCode::UNAUTHORIZED
        );

        let path = write_allowlist(&dir, &[&npub_of(&keys)]);
        let allow = Allowlist::load(&path).unwrap();
        let header = sign_header(&keys, HttpMethod::GET, url, t);
        let gate = http_ws_auth_gate(
            "GET",
            "/api/v1/ws",
            Some(&header),
            url,
            b"",
            t,
            false,
            &allow,
        );
        match &gate {
            Ok(HttpAuthOutcome::Pubkey(_)) => {}
            other => panic!("expected listed pubkey, got {:?}", other),
        }
        assert_eq!(
            mwck_upgrade_http_status(gate.clone(), true, true),
            StatusCode::SWITCHING_PROTOCOLS
        );
        assert_eq!(
            mwck_upgrade_http_status(gate.clone(), false, true),
            StatusCode::BAD_REQUEST
        );
        assert_eq!(
            mwck_upgrade_http_status(gate, true, false),
            StatusCode::BAD_REQUEST
        );

        std::fs::write(&path, "").unwrap();
        allow.reload().unwrap();
        let header2 = sign_header(&keys, HttpMethod::GET, url, now());
        let gate2 = http_ws_auth_gate(
            "GET",
            "/api/v1/ws",
            Some(&header2),
            url,
            b"",
            now(),
            false,
            &allow,
        );
        assert_eq!(
            mwck_upgrade_http_status(gate2, true, true),
            StatusCode::UNAUTHORIZED
        );
    }

    /// Named contract: a live hyper 0.14 server (not the gate helper) returns
    /// 401 on empty allowlist GET /api/v1/ws, 101 for a listed npub, and closes
    /// the socket after an allowlist reload drops that npub. This fixture is
    /// not a full indexer.
    #[tokio::test]
    async fn live_hyper_ws_101_handshake_allowlist() {
        use super::{Server, handle_mwck_upgrade};
        use crate::auth::Allowlist;
        use crate::mwck::MwckHub;
        use futures_util::{SinkExt, StreamExt};
        use hyper::service::{make_service_fn, service_fn};
        use hyper::{Body, Request};
        use nostr::nips::nip98::HttpMethod;
        use std::sync::Arc;
        use std::time::Duration;
        use tokio::net::TcpStream;
        use tokio_tungstenite::client_async;
        use tokio_tungstenite::tungstenite::Error as WsError;
        use tokio_tungstenite::tungstenite::Message;
        use tokio_tungstenite::tungstenite::client::IntoClientRequest;

        let keys = Keys::generate();
        let dir = tempfile::tempdir().unwrap();
        let path = write_allowlist(&dir, &[]);
        // Allowlist::load already returns Arc<Allowlist>.
        let allow = Allowlist::load(&path).unwrap();
        let allow_server = Arc::clone(&allow);

        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        listener.set_nonblocking(true).unwrap();
        let addr = listener.local_addr().unwrap();
        let hub = MwckHub::handshake_fixture();
        let make_svc = make_service_fn(move |_| {
            let allow = Arc::clone(&allow_server);
            let hub = Arc::clone(&hub);
            async move {
                Ok::<_, hyper::Error>(service_fn(move |req: Request<Body>| {
                    let allow = Arc::clone(&allow);
                    let hub = Arc::clone(&hub);
                    async move {
                        Ok::<_, hyper::Error>(handle_mwck_upgrade(req, allow, hub, false).await)
                    }
                }))
            }
        });
        let server = tokio::spawn(async move {
            Server::from_tcp(listener)
                .expect("from_tcp")
                .serve(make_svc)
                .await
                .expect("serve");
        });
        for _ in 0..50 {
            if TcpStream::connect(addr).await.is_ok() {
                break;
            }
            tokio::time::sleep(Duration::from_millis(10)).await;
        }

        let http_url = format!("http://{}/api/v1/ws", addr);
        let ws_url = format!("ws://{}/api/v1/ws", addr);

        async fn handshake(
            addr: std::net::SocketAddr,
            ws_url: &str,
            auth: &str,
        ) -> Result<
            (
                tokio_tungstenite::WebSocketStream<TcpStream>,
                tokio_tungstenite::tungstenite::http::Response<Option<Vec<u8>>>,
            ),
            WsError,
        > {
            let mut req = ws_url.into_client_request().expect("ws request");
            req.headers_mut().insert(
                tokio_tungstenite::tungstenite::http::header::AUTHORIZATION,
                auth.parse().expect("auth header"),
            );
            let stream = TcpStream::connect(addr).await.expect("connect");
            client_async(req, stream).await
        }

        let header = sign_header(&keys, HttpMethod::GET, &http_url, now());
        match handshake(addr, &ws_url, &header).await {
            Err(WsError::Http(resp)) => {
                // tungstenite http 1 vs hyper http 0.2 StatusCode. Compare 401.
                assert_eq!(resp.status().as_u16(), StatusCode::UNAUTHORIZED.as_u16());
            }
            other => panic!("empty allowlist must be HTTP 401, got {:?}", other),
        }

        std::fs::write(&path, format!("{}\n", npub_of(&keys))).unwrap();
        allow.reload().unwrap();
        let header = sign_header(&keys, HttpMethod::GET, &http_url, now());
        let (mut ws, resp) = handshake(addr, &ws_url, &header)
            .await
            .expect("listed npub yields a live 101 handshake");
        assert_eq!(
            resp.status().as_u16(),
            StatusCode::SWITCHING_PROTOCOLS.as_u16()
        );

        std::fs::write(&path, "").unwrap();
        allow.reload().unwrap();
        let _ = ws.send(Message::Text("{}".into())).await;
        let closed = tokio::time::timeout(Duration::from_secs(5), async {
            loop {
                match ws.next().await {
                    Some(Ok(Message::Close(_))) | Some(Err(_)) | None => return true,
                    Some(Ok(_)) => {}
                }
            }
        })
        .await
        .expect("socket must close after allowlist reload drop");
        assert!(closed);

        server.abort();
    }

    #[test]
    fn test_parse_query_param() {
        let mut query_params = HashMap::new();

        query_params.insert("limit", "10");
        let limit = query_params
            .get("limit")
            .map_or(10u32, |el| el.parse().unwrap_or(10u32))
            .min(30u32);
        assert_eq!(10, limit);

        query_params.insert("limit", "100");
        let limit = query_params
            .get("limit")
            .map_or(10u32, |el| el.parse().unwrap_or(10u32))
            .min(30u32);
        assert_eq!(30, limit);

        query_params.insert("limit", "5");
        let limit = query_params
            .get("limit")
            .map_or(10u32, |el| el.parse().unwrap_or(10u32))
            .min(30u32);
        assert_eq!(5, limit);

        query_params.insert("limit", "aaa");
        let limit = query_params
            .get("limit")
            .map_or(10u32, |el| el.parse().unwrap_or(10u32))
            .min(30u32);
        assert_eq!(10, limit);

        query_params.remove("limit");
        let limit = query_params
            .get("limit")
            .map_or(10u32, |el| el.parse().unwrap_or(10u32))
            .min(30u32);
        assert_eq!(10, limit);
    }

    #[test]
    fn test_parse_value_param() {
        let v: Value = json!({ "confirmations": 10 });

        let confirmations = v
            .get("confirmations")
            .and_then(|el| el.as_u64())
            .ok_or(HttpError::from(
                "confirmations absent or not a u64".to_string(),
            ))
            .unwrap();

        assert_eq!(10, confirmations);

        let err = v
            .get("notexist")
            .and_then(|el| el.as_u64())
            .ok_or(HttpError::from("notexist absent or not a u64".to_string()));

        assert!(err.is_err());
    }

    #[test]
    fn test_confirmed_after_txid_uses_chain_cursor_only() {
        let txid: Txid = "0000000000000000000000000000000000000000000000000000000000000001"
            .parse()
            .unwrap();

        assert_eq!(
            confirmed_after_txid(&TxidLocation::Mempool, Some(&txid)),
            None
        );
        assert_eq!(confirmed_after_txid(&TxidLocation::None, Some(&txid)), None);
        assert_eq!(
            confirmed_after_txid(&TxidLocation::Chain(123), Some(&txid)),
            Some(&txid)
        );
    }

    #[test]
    fn test_confirmed_after_txid_allows_mempool_chain_boundary_progress() {
        let txid: Txid = "0000000000000000000000000000000000000000000000000000000000000002"
            .parse()
            .unwrap();

        // If a mempool cursor returns no newer mempool txs, confirmed history
        // must start from the newest confirmed tx instead of seeking this txid.
        assert_eq!(
            confirmed_after_txid(&TxidLocation::Mempool, Some(&txid)),
            None
        );
    }

    #[test]
    fn test_difficulty_new() {
        use super::difficulty_new;

        let vectors = [
            (
                // bits in header
                0x17053894,
                // expected output (Rust)
                53911173001054.586,
                // Block hash where found (for getblockheader)
                "0000000000000000000050b050758dd2ccb0ba96ad5e95db84efd2f6c05e4e90",
                // difficulty returned by Bitcoin Core v25
                "53911173001054.59",
            ),
            (
                0x1a0c2a12,
                1379192.2882280778,
                "0000000000000bc7636ffbc1cf90cf4a2674de7fcadbc6c9b63d31f07cb3c2c2",
                "1379192.288228078",
            ),
            (
                0x19262222,
                112628548.66634709,
                "000000000000000996b1f06771a81bcf7b15c5f859b6f8329016f01b0442ca72",
                "112628548.6663471",
            ),
            (
                0x1d00c428,
                1.3050621315915245,
                "0000000034014d731a3e1ad6078662ce19b08179dcc7ec0f5f717d4b58060736",
                "1.305062131591525",
            ),
            (
                0,
                f64::INFINITY,
                "[No Blockhash]",
                "[No Core difficulty, just checking edge cases]",
            ),
            (
                0x00000001,
                4.523059468369196e74,
                "[No Blockhash]",
                "[No Core difficulty, just checking edge cases]",
            ),
            (
                0x1d00ffff,
                1.0,
                "[No Blockhash]",
                "[No Core difficulty, just checking MAX_TARGET]",
            ),
            (
                0x1c7fff80,
                2.0,
                "[No Blockhash]",
                "[No Core difficulty, just checking MAX_TARGET >> 1]",
            ),
            (
                0x1b00ffff,
                65536.0,
                "[No Blockhash]",
                "[No Core difficulty, just checking MAX_TARGET >> 16]",
            ),
            (
                0x1a7fff80,
                131072.0,
                "[No Blockhash]",
                "[No Core difficulty, just checking MAX_TARGET >> 17]",
            ),
            (
                0x1d01fffe,
                0.5,
                "[No Blockhash]",
                "[No Core difficulty, just checking MAX_TARGET << 1]",
            ),
            (
                0x1f000080,
                0.007812380790710449,
                "[No Blockhash]",
                "[No Core difficulty, just checking 2**255]",
            ),
            (
                0x1e00ffff,
                0.00390625, // 2.0**-8
                "[No Blockhash]",
                "[No Core difficulty, just checking MAX_TARGET << 8]",
            ),
            (
                0x1e00ff00,
                0.0039215087890625,
                "[No Blockhash]",
                "[No Core difficulty, just checking MAX_TARGET << 8 - two `f` chars]",
            ),
            (
                0x1f0000ff,
                0.0039215087890625,
                "[No Blockhash]",
                "[No Core difficulty, just checking MAX_TARGET << 8]",
            ),
        ];

        let to_bh = |b| bitcoin::block::Header {
            version: bitcoin::block::Version::ONE,
            prev_blockhash: "0000000000000000000000000000000000000000000000000000000000000000"
                .parse()
                .unwrap(),
            merkle_root: "0000000000000000000000000000000000000000000000000000000000000000"
                .parse()
                .unwrap(),
            time: 0,
            bits: bitcoin::CompactTarget::from_consensus(b),
            nonce: 0,
        };

        for (bits, expected, hash, core_difficulty) in vectors {
            let result = difficulty_new(&to_bh(bits));
            assert_eq!(
                result, expected,
                "Block {} difficulty is {} but Core difficulty is {}",
                hash, result, core_difficulty,
            );
        }
    }
}
