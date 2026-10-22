//! Parsers for indexer JSON and plain-text bodies.
//!
//! Field names follow the serde structs the indexer emits. This module does
//! not invent a second shape.

use serde::{Deserialize, Serialize};

/// Why a response body did not match the indexer shape.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseError {
    pub detail: String,
}

impl ParseError {
    pub(crate) fn new(detail: impl Into<String>) -> Self {
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
/// come from `ScriptStats` in `src/new_index/schema.rs`. A missing funded
/// sum is `None`, not an error and not `"0"`. Explicit `0` is `Some("0")`.
/// Mempool tx count is `mempool_stats.tx_count`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AddressStats {
    pub address: String,
    pub chain_tx_count: String,
    pub funded_sum: Option<String>,
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
/// `status.confirmed` from `TransactionStatus`. A missing `value` is an empty
/// string, not `"0"`.
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
    #[serde(default)]
    value: Option<u64>,
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

#[derive(Deserialize, Serialize)]
struct RecentJson {
    txid: String,
    fee: u64,
    vsize: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    value: Option<u64>,
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
    Ok(AddressStats {
        address: raw.address,
        chain_tx_count: raw.chain_stats.tx_count.to_string(),
        funded_sum: raw.chain_stats.funded_txo_sum.map(|sum| sum.to_string()),
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
            value: row
                .value
                .map(|amount| amount.to_string())
                .unwrap_or_default(),
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
            value: match row.value {
                Some(amount) => amount.to_string(),
                None => String::new(),
            },
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

/// `GET /block/:hash/status` (`BlockStatus` in `src/util/block.rs`).
pub fn parse_block_status(body: &str) -> Result<crate::wire::BlockStatus, ParseError> {
    Ok(serde_json::from_str(body)?)
}

/// `GET /tx/:txid/outspends`.
pub fn parse_outspends(body: &str) -> Result<Vec<crate::wire::Outspend>, ParseError> {
    Ok(serde_json::from_str(body)?)
}

/// `GET /tx/:txid/outspend/:index`.
pub fn parse_outspend(body: &str) -> Result<crate::wire::Outspend, ParseError> {
    Ok(serde_json::from_str(body)?)
}

/// `GET /tx/:txid/merkle-proof`.
pub fn parse_merkle_proof(body: &str) -> Result<crate::wire::MerkleProof, ParseError> {
    Ok(serde_json::from_str(body)?)
}

/// `GET /tx/:txid/hex` returns the raw transaction as plain hex, not JSON.
pub fn parse_tx_hex(body: &str) -> Result<String, ParseError> {
    let hex = body.trim();
    if hex.is_empty() {
        return Err(ParseError::new("tx hex was empty"));
    }
    Ok(hex.to_string())
}

/// `GET /address/:script/txs/chain` and the `:last_seen_txid` page.
///
/// The body is the same transaction array as [`parse_address_txs`].
pub fn parse_address_txs_chain(body: &str) -> Result<Vec<Tx>, ParseError> {
    parse_address_txs(body)
}

/// `GET /address/:script/txs/mempool`.
///
/// The body is the same transaction array as [`parse_address_txs`].
pub fn parse_address_txs_mempool(body: &str) -> Result<Vec<Tx>, ParseError> {
    parse_address_txs(body)
}

/// `GET /address/:script/txs/summary` and the summary page.
pub fn parse_address_txs_summary(body: &str) -> Result<Vec<crate::wire::TxSummary>, ParseError> {
    Ok(serde_json::from_str(body)?)
}

/// `GET /mempool/txids` and `GET /mempool/txids/page/:last_seen_txid`.
pub fn parse_mempool_txids(body: &str) -> Result<Vec<String>, ParseError> {
    Ok(serde_json::from_str(body)?)
}

/// `GET /address-prefix/:prefix` returns a JSON array of address strings.
pub fn parse_address_prefix(body: &str) -> Result<Vec<String>, ParseError> {
    Ok(serde_json::from_str(body)?)
}

/// `GET /scripthash/:hash`.
///
/// Same stats object as [`AddressStats`], with `scripthash` instead of
/// `address`. `src/rest.rs` uses the script type as the object key.
/// A missing funded sum is `None`, not an error and not `"0"`.
/// Explicit `0` is `Some("0")`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScriptHashStats {
    pub scripthash: String,
    pub chain_tx_count: String,
    pub funded_sum: Option<String>,
    pub mempool_tx_count: String,
}

#[derive(Deserialize)]
struct ScriptHashJson {
    scripthash: String,
    chain_stats: ScriptStatsJson,
    mempool_stats: ScriptStatsJson,
}

pub fn parse_scripthash(body: &str) -> Result<ScriptHashStats, ParseError> {
    let raw: ScriptHashJson = serde_json::from_str(body)?;
    Ok(ScriptHashStats {
        scripthash: raw.scripthash,
        chain_tx_count: raw.chain_stats.tx_count.to_string(),
        funded_sum: raw.chain_stats.funded_txo_sum.map(|sum| sum.to_string()),
        mempool_tx_count: raw.mempool_stats.tx_count.to_string(),
    })
}

/// `GET /scripthash/:hash/txs`. Same body as [`parse_address_txs`].
pub fn parse_scripthash_txs(body: &str) -> Result<Vec<Tx>, ParseError> {
    parse_address_txs(body)
}

/// `GET /scripthash/:hash/txs/chain` and the `:last_seen_txid` page.
pub fn parse_scripthash_txs_chain(body: &str) -> Result<Vec<Tx>, ParseError> {
    parse_address_txs(body)
}

/// `GET /scripthash/:hash/txs/mempool`.
pub fn parse_scripthash_txs_mempool(body: &str) -> Result<Vec<Tx>, ParseError> {
    parse_address_txs(body)
}

/// `GET /scripthash/:hash/utxo`. Same body as [`parse_address_utxos`].
pub fn parse_scripthash_utxos(body: &str) -> Result<Vec<Utxo>, ParseError> {
    parse_address_utxos(body)
}

/// `POST /txs/package` (`SubmitPackageResult` in `src/daemon.rs`).
pub fn parse_package(body: &str) -> Result<crate::wire::PackageResult, ParseError> {
    Ok(serde_json::from_str(body)?)
}

/// `GET /scripthash/:hash/txs/summary` and the summary page.
///
/// Same rows as [`parse_address_txs_summary`]. The row type stays [`crate::wire::TxSummary`].
pub fn parse_scripthash_txs_summary(body: &str) -> Result<Vec<crate::wire::TxSummary>, ParseError> {
    parse_address_txs_summary(body)
}

/// `POST /addresses/txs`. Same transaction array as [`parse_address_txs`].
pub fn parse_addresses_txs(body: &str) -> Result<Vec<Tx>, ParseError> {
    parse_address_txs(body)
}

/// `POST /scripthashes/txs`. Same transaction array as [`parse_address_txs`].
pub fn parse_scripthashes_txs(body: &str) -> Result<Vec<Tx>, ParseError> {
    parse_address_txs(body)
}

/// `POST /addresses/txs/summary` and the `:last_seen_txid` page.
pub fn parse_addresses_txs_summary(body: &str) -> Result<Vec<crate::wire::TxSummary>, ParseError> {
    parse_address_txs_summary(body)
}

/// `POST /scripthashes/txs/summary` and the `:last_seen_txid` page.
pub fn parse_scripthashes_txs_summary(
    body: &str,
) -> Result<Vec<crate::wire::TxSummary>, ParseError> {
    parse_address_txs_summary(body)
}

/// `GET /txs/outspends?txids=`.
///
/// One spend array per transaction id, in the requested order (`src/rest.rs`).
/// The indexer accepts at most 50 ids.
pub fn parse_txs_outspends(body: &str) -> Result<Vec<Vec<crate::wire::Outspend>>, ParseError> {
    Ok(serde_json::from_str(body)?)
}

/// `GET /tx/:txid/status` (`TransactionStatus` in `src/util/transaction.rs`).
pub fn parse_tx_status(body: &str) -> Result<crate::wire::TxStatus, ParseError> {
    Ok(serde_json::from_str(body)?)
}

/// `GET /block/:hash/txid/:index` returns one txid as plain text.
pub fn parse_block_txid(body: &str) -> Result<String, ParseError> {
    let txid = body.trim();
    if txid.is_empty() {
        return Err(ParseError::new("block txid was empty"));
    }
    Ok(txid.to_string())
}

fn hex_nibble(byte: u8) -> Result<u8, ParseError> {
    match byte {
        b'0'..=b'9' => Ok(byte - b'0'),
        b'a'..=b'f' => Ok(byte - b'a' + 10),
        b'A'..=b'F' => Ok(byte - b'A' + 10),
        _ => Err(ParseError::new("hex was not hexadecimal")),
    }
}

fn decode_hex(text: &str) -> Result<Vec<u8>, ParseError> {
    if text.len() % 2 != 0 {
        return Err(ParseError::new("hex length was odd"));
    }
    let bytes = text.as_bytes();
    let mut out = Vec::with_capacity(text.len() / 2);
    let mut index = 0;
    while index < bytes.len() {
        let high = hex_nibble(bytes[index])?;
        let low = hex_nibble(bytes[index + 1])?;
        out.push((high << 4) | low);
        index += 2;
    }
    Ok(out)
}

fn hex_text(body: &str, empty_detail: &str) -> Result<String, ParseError> {
    let text = body.trim();
    if text.is_empty() {
        return Err(ParseError::new(empty_detail));
    }
    decode_hex(text)?;
    Ok(text.to_string())
}

/// `GET /block/:hash/header`. The indexer sends hex text (`hex::encode` in `src/rest.rs`).
pub fn parse_block_header(body: &str) -> Result<String, ParseError> {
    hex_text(body, "block header was empty")
}

/// Header hex decoded to bytes.
pub fn parse_block_header_bytes(body: &str) -> Result<Vec<u8>, ParseError> {
    decode_hex(&parse_block_header(body)?)
}

/// `GET /block/:hash/raw`. The indexer sends `application/octet-stream`.
pub fn parse_block_raw(body: &[u8]) -> Result<Vec<u8>, ParseError> {
    if body.is_empty() {
        return Err(ParseError::new("block raw was empty"));
    }
    Ok(body.to_vec())
}

/// Hex text for the same raw block, when the caller already encoded it.
pub fn parse_block_raw_hex(body: &str) -> Result<String, ParseError> {
    hex_text(body, "block raw hex was empty")
}

/// `GET /tx/:txid/raw`. The indexer sends `application/octet-stream`.
pub fn parse_tx_raw(body: &[u8]) -> Result<Vec<u8>, ParseError> {
    if body.is_empty() {
        return Err(ParseError::new("tx raw was empty"));
    }
    Ok(body.to_vec())
}

/// Hex text for the same raw transaction.
pub fn parse_tx_raw_hex(body: &str) -> Result<String, ParseError> {
    hex_text(body, "tx raw hex was empty")
}

/// `GET /tx/:txid/merkleblock-proof`. Hex text. Omitted when the liquid feature is on.
pub fn parse_merkleblock_proof(body: &str) -> Result<String, ParseError> {
    hex_text(body, "merkleblock proof was empty")
}

/// Merkleblock proof hex decoded to bytes.
pub fn parse_merkleblock_proof_bytes(body: &str) -> Result<Vec<u8>, ParseError> {
    decode_hex(&parse_merkleblock_proof(body)?)
}

#[cfg(test)]
mod tests {
    use super::{RecentJson, parse_address_utxos};

    #[test]
    fn address_utxo_omitted_value_displays_empty_not_zero() {
        let omitted = parse_address_utxos(
            r#"[{"txid":"blind","vout":1,"status":{"confirmed":false},"valuecommitment":"aa"}]"#,
        )
        .expect("omitted value");
        assert_eq!(omitted[0].value, "");
        assert_ne!(omitted[0].value, "0");

        let zero =
            parse_address_utxos(r#"[{"txid":"z","vout":1,"value":0,"status":{"confirmed":true}}]"#)
                .expect("explicit zero");
        assert_eq!(zero[0].value, "0");

        let present = parse_address_utxos(
            r#"[{"txid":"n","vout":2,"value":42000,"status":{"confirmed":true}}]"#,
        )
        .expect("present value");
        assert_eq!(present[0].value, "42000");

        let null_value = parse_address_utxos(
            r#"[{"txid":"n","vout":1,"value":null,"status":{"confirmed":false}}]"#,
        )
        .expect("json null");
        assert_eq!(null_value[0].value, "");
    }

    #[test]
    fn recent_json_null() {
        let cleared: RecentJson =
            serde_json::from_str(r#"{"txid":"cleared-row","fee":13,"vsize":143,"value":null}"#)
                .expect("json null value is not a parse error");
        assert_eq!(cleared.value, None);
        let encoded = serde_json::to_string(&cleared).expect("serialize null");
        assert!(
            !encoded.contains("value"),
            "null value must be omitted, got {encoded}"
        );
        assert!(
            !encoded.contains("null"),
            "null value must not be written, got {encoded}"
        );

        let zero: RecentJson =
            serde_json::from_str(r#"{"txid":"explicit-zero","fee":11,"vsize":141,"value":0}"#)
                .expect("explicit zero");
        assert_eq!(zero.value, Some(0));
        let zero_encoded = serde_json::to_string(&zero).expect("serialize zero");
        assert!(
            zero_encoded.contains("\"value\":0") || zero_encoded.contains("\"value\": 0"),
            "explicit zero must stay on the wire, got {zero_encoded}"
        );

        let missing: RecentJson =
            serde_json::from_str(r#"{"txid":"omitted-row","fee":10,"vsize":140}"#)
                .expect("missing value key is not a parse error");
        assert_eq!(missing.value, None);
    }
}
