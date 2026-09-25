//! Parsers for indexer JSON and plain-text bodies.
//!
//! Field names follow the serde structs the indexer emits. This module does
//! not invent a second shape.

use serde::Deserialize;

/// Why a response body did not match the indexer shape.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseError {
    pub detail: String,
}

impl ParseError {
    fn new(detail: impl Into<String>) -> Self {
        Self {
            detail: detail.into(),
        }
    }
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.detail)
    }
}

impl std::error::Error for ParseError {}

impl From<serde_json::Error> for ParseError {
    fn from(err: serde_json::Error) -> Self {
        Self {
            detail: err.to_string(),
        }
    }
}

fn yes_no(value: bool) -> String {
    if value {
        "true".to_string()
    } else {
        "false".to_string()
    }
}

/// One block from `GET /blocks` or `GET /block/:hash`.
///
/// JSON keys from `BlockValue` in `src/rest.rs`: `id`, `height`, `tx_count`,
/// `timestamp`, `size`, `weight`, `previousblockhash`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Block {
    pub id: String,
    pub height: String,
    pub tx_count: String,
    pub timestamp: String,
    pub size: String,
    pub weight: String,
    pub previous_hash: String,
}

#[derive(Deserialize)]
struct BlockJson {
    id: String,
    height: u64,
    tx_count: u64,
    timestamp: u64,
    size: u64,
    weight: u64,
    previousblockhash: Option<String>,
}

impl Block {
    fn from_json(raw: BlockJson) -> Result<Self, ParseError> {
        let previous_hash = raw
            .previousblockhash
            .ok_or_else(|| ParseError::new("missing previousblockhash"))?;
        Ok(Self {
            id: raw.id,
            height: raw.height.to_string(),
            tx_count: raw.tx_count.to_string(),
            timestamp: raw.timestamp.to_string(),
            size: raw.size.to_string(),
            weight: raw.weight.to_string(),
            previous_hash,
        })
    }
}

/// One transaction from `GET /tx/:txid`, `GET /block/:hash/txs`, or
/// `GET /address/:script/txs`.
///
/// JSON keys from `TransactionValue` in `src/rest.rs`: `txid`, `fee`, `size`,
/// `weight`, and from `TransactionStatus` in `src/util/transaction.rs`:
/// `status.confirmed` and `status.block_height`. The indexer omits
/// `block_height` when it is `None` (an unconfirmed transaction).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Tx {
    pub txid: String,
    pub confirmed: String,
    pub block_height: String,
    pub fee: String,
    pub size: String,
    pub weight: String,
}

#[derive(Deserialize)]
struct TxJson {
    txid: String,
    fee: u64,
    size: u64,
    weight: u64,
    status: StatusJson,
}

#[derive(Deserialize)]
struct StatusJson {
    confirmed: bool,
    #[serde(default)]
    block_height: Option<u64>,
}

fn block_height_text(height: Option<u64>) -> String {
    height.map(|value| value.to_string()).unwrap_or_default()
}

impl From<TxJson> for Tx {
    fn from(raw: TxJson) -> Self {
        Self {
            txid: raw.txid,
            confirmed: yes_no(raw.status.confirmed),
            block_height: block_height_text(raw.status.block_height),
            fee: raw.fee.to_string(),
            size: raw.size.to_string(),
            weight: raw.weight.to_string(),
        }
    }
}

/// `GET /address/:script`.
///
/// The object keys are `address`, `chain_stats`, and `mempool_stats`
/// (`src/rest.rs`). `chain_stats.tx_count` and `chain_stats.funded_txo_sum`
/// come from `ScriptStats` in `src/new_index/schema.rs`. Mempool tx count is
/// `mempool_stats.tx_count`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AddressStats {
    pub address: String,
    pub chain_tx_count: String,
    pub funded_sum: String,
    pub mempool_tx_count: String,
}

#[derive(Deserialize)]
struct AddressJson {
    address: String,
    chain_stats: ScriptStatsJson,
    mempool_stats: ScriptStatsJson,
}

#[derive(Deserialize)]
struct ScriptStatsJson {
    tx_count: u64,
    #[serde(default)]
    funded_txo_sum: Option<u64>,
}

/// One output from `GET /address/:script/utxo`.
///
/// JSON keys from `UtxoValue` in `src/rest.rs`: `txid`, `vout`, `value`, and
/// `status.confirmed` from `TransactionStatus`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Utxo {
    pub txid: String,
    pub vout: String,
    pub value: String,
    pub confirmed: String,
}

#[derive(Deserialize)]
struct UtxoJson {
    txid: String,
    vout: u64,
    value: u64,
    status: StatusJson,
}

/// One row from `GET /mempool/recent`.
///
/// JSON keys from `TxOverview` in `src/new_index/mempool.rs`: `txid`, `fee`,
/// `vsize`, `value`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecentTx {
    pub txid: String,
    pub fee: String,
    pub vsize: String,
    pub value: String,
}

#[derive(Deserialize)]
struct RecentJson {
    txid: String,
    fee: u64,
    vsize: u64,
    value: u64,
}

/// One element of the `POST /txs/test` JSON array.
///
/// Keys from `MempoolAcceptResult` in `src/daemon.rs`: `txid`, `allowed`,
/// and `reject-reason`. An allowed row may omit `reject-reason`. A rejected
/// row still has to include it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TestTxResult {
    pub txid: String,
    pub allowed: String,
    pub reject_reason: String,
}

#[derive(Deserialize)]
struct TestTxJson {
    txid: String,
    allowed: bool,
    #[serde(rename = "reject-reason")]
    reject_reason: Option<String>,
}

pub fn parse_blocks(body: &str) -> Result<Vec<Block>, ParseError> {
    let raw: Vec<BlockJson> = serde_json::from_str(body)?;
    raw.into_iter().map(Block::from_json).collect()
}

pub fn parse_block(body: &str) -> Result<Block, ParseError> {
    Block::from_json(serde_json::from_str(body)?)
}

pub fn parse_block_txs(body: &str) -> Result<Vec<Tx>, ParseError> {
    let raw: Vec<TxJson> = serde_json::from_str(body)?;
    Ok(raw.into_iter().map(Tx::from).collect())
}

pub fn parse_tx(body: &str) -> Result<Tx, ParseError> {
    let raw: TxJson = serde_json::from_str(body)?;
    Ok(Tx::from(raw))
}

pub fn parse_address(body: &str) -> Result<AddressStats, ParseError> {
    let raw: AddressJson = serde_json::from_str(body)?;
    let funded_sum = raw
        .chain_stats
        .funded_txo_sum
        .ok_or_else(|| ParseError::new("missing chain_stats.funded_txo_sum"))?;
    Ok(AddressStats {
        address: raw.address,
        chain_tx_count: raw.chain_stats.tx_count.to_string(),
        funded_sum: funded_sum.to_string(),
        mempool_tx_count: raw.mempool_stats.tx_count.to_string(),
    })
}

pub fn parse_address_txs(body: &str) -> Result<Vec<Tx>, ParseError> {
    parse_block_txs(body)
}

pub fn parse_address_utxos(body: &str) -> Result<Vec<Utxo>, ParseError> {
    let raw: Vec<UtxoJson> = serde_json::from_str(body)?;
    Ok(raw
        .into_iter()
        .map(|row| Utxo {
            txid: row.txid,
            vout: row.vout.to_string(),
            value: row.value.to_string(),
            confirmed: yes_no(row.status.confirmed),
        })
        .collect())
}

pub fn parse_mempool_recent(body: &str) -> Result<Vec<RecentTx>, ParseError> {
    let raw: Vec<RecentJson> = serde_json::from_str(body)?;
    Ok(raw
        .into_iter()
        .map(|row| RecentTx {
            txid: row.txid,
            fee: row.fee.to_string(),
            vsize: row.vsize.to_string(),
            value: row.value.to_string(),
        })
        .collect())
}

/// `POST /tx` returns the txid as `text/plain`, not a JSON object
/// (`http_message` in `src/rest.rs`).
pub fn parse_broadcast(body: &str) -> Result<String, ParseError> {
    let txid = body.trim();
    if txid.is_empty() {
        return Err(ParseError::new("broadcast result was empty"));
    }
    Ok(txid.to_string())
}

pub fn parse_test_txs(body: &str) -> Result<Vec<TestTxResult>, ParseError> {
    let raw: Vec<TestTxJson> = serde_json::from_str(body)?;
    raw.into_iter()
        .map(|row| {
            let reject_reason = match (row.allowed, row.reject_reason) {
                (_, Some(reason)) => reason,
                (true, None) => String::new(),
                (false, None) => return Err(ParseError::new("missing reject-reason")),
            };
            Ok(TestTxResult {
                txid: row.txid,
                allowed: yes_no(row.allowed),
                reject_reason,
            })
        })
        .collect()
}

/// `GET /blocks/tip/hash` returns the best block hash as plain text.
pub fn parse_blocks_tip_hash(body: &str) -> Result<String, ParseError> {
    let hash = body.trim();
    if hash.is_empty() {
        return Err(ParseError::new("tip hash was empty"));
    }
    Ok(hash.to_string())
}

/// `GET /blocks/tip/height` returns the best height as plain decimal text.
pub fn parse_blocks_tip_height(body: &str) -> Result<String, ParseError> {
    let text = body.trim();
    if text.is_empty() {
        return Err(ParseError::new("tip height was empty"));
    }
    let height: u64 = text
        .parse()
        .map_err(|_| ParseError::new("tip height was not an integer"))?;
    Ok(height.to_string())
}

/// One confirmation target from `GET /fee-estimates`.
///
/// The indexer returns a map of target to sat/vB (`estimate_fee_map` in
/// `src/new_index/query.rs`). `target` is the map key. `rate` is the decimal
/// text of the JSON number.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FeeEstimate {
    pub target: String,
    pub rate: String,
}

pub fn parse_fee_estimates(body: &str) -> Result<Vec<FeeEstimate>, ParseError> {
    let map: serde_json::Map<String, serde_json::Value> = serde_json::from_str(body)?;
    let mut rows = Vec::with_capacity(map.len());
    for (key, value) in map {
        let target: u64 = key
            .parse()
            .map_err(|_| ParseError::new(format!("bad fee target {key}")))?;
        let rate = json_number_text(&value, "fee rate")?;
        rows.push((
            target,
            FeeEstimate {
                target: target.to_string(),
                rate,
            },
        ));
    }
    rows.sort_by_key(|(target, _)| *target);
    Ok(rows.into_iter().map(|(_, row)| row).collect())
}

/// `GET /mempool` (`BacklogStats` in `src/new_index/mempool.rs`).
///
/// `fee_histogram` pairs keep the tuple order from `make_fee_histogram`:
/// fee rate, then vsize.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BacklogStats {
    pub count: String,
    pub vsize: String,
    pub total_fee: String,
    pub fee_histogram: Vec<(String, String)>,
}

#[derive(Deserialize)]
struct BacklogJson {
    count: u64,
    vsize: u64,
    total_fee: u64,
    fee_histogram: Vec<Vec<serde_json::Value>>,
}

fn json_number_text(value: &serde_json::Value, what: &str) -> Result<String, ParseError> {
    match value {
        serde_json::Value::Number(number) => Ok(number.to_string()),
        _ => Err(ParseError::new(format!("{what} was not a number"))),
    }
}

pub fn parse_mempool(body: &str) -> Result<BacklogStats, ParseError> {
    let raw: BacklogJson = serde_json::from_str(body)?;
    let mut fee_histogram = Vec::with_capacity(raw.fee_histogram.len());
    for bin in raw.fee_histogram {
        if bin.len() != 2 {
            return Err(ParseError::new("fee histogram entry was not a pair"));
        }
        fee_histogram.push((
            json_number_text(&bin[0], "fee rate")?,
            json_number_text(&bin[1], "vsize")?,
        ));
    }
    Ok(BacklogStats {
        count: raw.count.to_string(),
        vsize: raw.vsize.to_string(),
        total_fee: raw.total_fee.to_string(),
        fee_histogram,
    })
}

/// `GET /block/:hash/txids` returns a JSON array of txid strings.
pub fn parse_block_txids(body: &str) -> Result<Vec<String>, ParseError> {
    Ok(serde_json::from_str(body)?)
}

/// `GET /block-height/:height` returns the block hash as plain text
/// (`header.hash()` in `src/rest.rs`), not the height and not JSON.
pub fn parse_block_height(body: &str) -> Result<String, ParseError> {
    let hash = body.trim();
    if hash.is_empty() {
        return Err(ParseError::new("block height result was empty"));
    }
    Ok(hash.to_string())
}
