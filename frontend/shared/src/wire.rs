//! Serde shapes for indexer JSON.
//!
//! Fields follow the structs this tree already serializes. String parsers in
//! [`crate::parse`] stay as they are. Extra keys are ignored. `is_pegin`,
//! `pegout`, `issuance`, and `ext` stay optional because the indexer emits
//! them only on a liquid build.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

/// One block from `GET /block/:hash` or an element of `GET /blocks`.
///
/// `BlockValue` in `src/rest.rs`. `previousblockhash` is null for the genesis
/// block and is omitted by some clients; both deserialize. `ext` is the liquid
/// header extra object. A Bitcoin body omits it. A missing key is `None`. The
/// value is the JSON object the indexer already emits. There is no elements
/// crate and no unblinding. `nonce`, `bits`, and `difficulty` are omitted on
/// a liquid build. A missing key or JSON null is `None`, not zero. Explicit
/// `0` / `0.0` stays `Some`. `stale` is omitted on an ordinary block. A
/// missing key or JSON null is not stale. JSON `true` is the stale heading.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Block {
    pub id: String,
    pub height: u32,
    pub version: u32,
    pub timestamp: u32,
    pub tx_count: u32,
    pub size: u32,
    pub weight: u32,
    pub merkle_root: String,
    #[serde(default, rename = "previousblockhash")]
    pub previous_block_hash: Option<String>,
    #[serde(rename = "mediantime")]
    pub median_time: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub nonce: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bits: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub difficulty: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ext: Option<serde_json::Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stale: Option<bool>,
}

/// JSON array of blocks. An element that does not deserialize is dropped.
///
/// A missing `height`, JSON null, or any other mismatch drops that element
/// and keeps the rest. A body that is not a JSON array returns the strict
/// `Vec<Block>` error.
pub fn blocks_keeping_valid_elements(body: &str) -> Result<Vec<Block>, serde_json::Error> {
    match serde_json::from_str::<serde_json::Value>(body)? {
        serde_json::Value::Array(elements) => Ok(elements
            .into_iter()
            .filter_map(|element| serde_json::from_value(element).ok())
            .collect()),
        _ => serde_json::from_str(body),
    }
}

/// `TransactionStatus` in `src/util/transaction.rs`.
///
/// `block_height`, `block_hash`, and `block_time` are omitted when the
/// transaction is unconfirmed.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TxStatus {
    pub confirmed: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub block_height: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub block_hash: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub block_time: Option<u64>,
}

/// `IssuanceValue` in `src/elements/mod.rs`.
///
/// Present on an input only when that input has an issuance. `assetamount`
/// and `tokenamount` are omitted when those values are confidential. A
/// missing amount is `None`, not zero. Each commitment is the hex of the
/// consensus encoding. There is no unblinding step.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Issuance {
    pub asset_id: String,
    pub is_reissuance: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub asset_blinding_nonce: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub contract_hash: Option<String>,
    pub asset_entropy: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub assetamount: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub assetamountcommitment: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tokenamount: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tokenamountcommitment: Option<String>,
}

/// One input from `TransactionValue` in `src/rest.rs`.
///
/// `is_pegin` is always present on a liquid build, including false. A Bitcoin
/// body omits it. A missing key is `None`. A present `false` stays
/// `Some(false)`. JSON `null` is `None`, not `Some(false)`. `issuance` is
/// omitted when the input has no issuance. A missing key is `None`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TxInput {
    pub txid: String,
    pub vout: u32,
    pub prevout: Option<TxOutput>,
    #[serde(rename = "scriptsig")]
    pub script_sig: String,
    #[serde(rename = "scriptsig_asm")]
    pub script_sig_asm: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub witness: Option<Vec<String>>,
    pub is_coinbase: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub is_pegin: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub issuance: Option<Issuance>,
    pub sequence: u32,
    #[serde(
        default,
        rename = "inner_redeemscript_asm",
        skip_serializing_if = "Option::is_none"
    )]
    pub inner_redeem_script_asm: Option<String>,
    #[serde(
        default,
        rename = "inner_witnessscript_asm",
        skip_serializing_if = "Option::is_none"
    )]
    pub inner_witness_script_asm: Option<String>,
}

/// `PegoutValue` in `src/elements/peg.rs`.
///
/// `scriptpubkey` is the parent-chain script as lowercase hex. There is no
/// value field. `scriptpubkey_address` is omitted when absent.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Pegout {
    pub genesis_hash: String,
    #[serde(rename = "scriptpubkey")]
    pub script_pubkey: String,
    #[serde(rename = "scriptpubkey_asm")]
    pub script_pubkey_asm: String,
    #[serde(
        default,
        rename = "scriptpubkey_address",
        skip_serializing_if = "Option::is_none"
    )]
    pub script_pubkey_address: Option<String>,
}

/// One output from `TransactionValue` in `src/rest.rs`.
///
/// `pegout` is omitted when the output is not a peg-out. The object is not
/// null when absent, and it has no value field.
///
/// On a liquid build, `value` is `Option<u64>` and is omitted when the amount
/// is confidential. A missing key is `None`. It is not zero. `asset` is the
/// explicit asset id string. `valuecommitment` and `assetcommitment` are the
/// hex of the 33-byte consensus encoding when that field is blinded.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TxOutput {
    #[serde(rename = "scriptpubkey")]
    pub script_pubkey: String,
    #[serde(rename = "scriptpubkey_asm")]
    pub script_pubkey_asm: String,
    #[serde(rename = "scriptpubkey_type")]
    pub script_pubkey_type: String,
    #[serde(
        default,
        rename = "scriptpubkey_address",
        skip_serializing_if = "Option::is_none"
    )]
    pub script_pubkey_address: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub value: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub valuecommitment: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub asset: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub assetcommitment: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pegout: Option<Pegout>,
}

/// One transaction from `GET /tx/:txid`, `GET /block/:hash/txs`, or
/// `GET /address/:script/txs`.
///
/// `TransactionValue` in `src/rest.rs`. Block height is `status.block_height`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Transaction {
    pub txid: String,
    pub version: u32,
    #[serde(rename = "locktime")]
    pub lock_time: u32,
    pub vin: Vec<TxInput>,
    pub vout: Vec<TxOutput>,
    pub size: u32,
    pub weight: u32,
    pub sigops: u32,
    pub fee: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub status: Option<TxStatus>,
}

/// `ScriptStats` in `src/new_index/schema.rs`.
///
/// A liquid build omits `funded_txo_sum` and `spent_txo_sum`. A missing
/// key is `None`, not zero. Explicit `0` stays `Some(0)`. JSON `null` is
/// `None`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScriptStats {
    pub tx_count: u64,
    pub funded_txo_count: u64,
    pub spent_txo_count: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub funded_txo_sum: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub spent_txo_sum: Option<u64>,
}

/// `GET /address/:script` or `GET /scripthash/:script`.
///
/// The indexer uses the script type as the object key (`src/rest.rs`), so
/// exactly one of `address` and `scripthash` is set.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AddressStats {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub address: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scripthash: Option<String>,
    pub chain_stats: ScriptStats,
    pub mempool_stats: ScriptStats,
}

/// One output from `GET /address/:script/utxo`.
///
/// `UtxoValue` in `src/rest.rs`. Confirmation is `status.confirmed`.
///
/// On a liquid build the indexer also emits `asset`, `valuecommitment`,
/// `assetcommitment`, `nonce`, `noncecommitment`, `surjection_proof`, and
/// `range_proof`. Each is omitted when absent. A missing key is `None`.
/// `surjection_proof` and `range_proof` are the hex strings `serde_hex`
/// already emits. An empty proof is omitted, not `""`. A missing `value`
/// is `None`, not zero. A non-liquid body still sends the number. There
/// is no unblinding step.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Utxo {
    pub txid: String,
    pub vout: u32,
    pub status: TxStatus,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub value: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub valuecommitment: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub asset: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub assetcommitment: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub nonce: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub noncecommitment: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub surjection_proof: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub range_proof: Option<String>,
}

/// One row from `GET /mempool/recent`.
///
/// `TxOverview` in `src/new_index/mempool.rs`. A liquid build omits `value`.
/// A missing key is `None`, not zero. Explicit `0` stays `Some(0)`. JSON
/// `null` is `None` and is omitted on serialize.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RecentTransaction {
    pub txid: String,
    pub fee: u64,
    pub vsize: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub value: Option<u64>,
}

/// Integer text for a recent mempool amount.
///
/// `Some(n)` is that integer, including zero. `None` with a non-empty
/// commitment is the word Confidential. `None` with an empty commitment is
/// blank. This does not convert satoshis to BTC. Recent rows have no
/// commitment field, so callers with none in hand pass `""`.
pub fn recent_value_text(value: Option<u64>, commitment: &str) -> String {
    if let Some(amount) = value {
        return amount.to_string();
    }
    if commitment.is_empty() {
        String::new()
    } else {
        "Confidential".to_string()
    }
}

/// `GET /mempool`.
///
/// `BacklogStats` in `src/new_index/mempool.rs`. Each `fee_histogram` pair is
/// fee rate in sat/vB, then virtual size, in that order.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MempoolSummary {
    pub count: u32,
    pub vsize: u32,
    pub total_fee: u64,
    pub fee_histogram: Vec<(f32, u32)>,
}

/// `GET /fee-estimates`.
///
/// `estimate_fee_map` in `src/new_index/query.rs` is a JSON object whose keys
/// are confirmation targets and whose values are sat/vB. This is that object,
/// not a list wrapped in another key.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct FeeEstimates(pub BTreeMap<u16, f64>);

/// Txid from `POST /tx`.
///
/// The handler returns `text/plain` (`http_message` in `src/rest.rs`), not
/// this object. The struct exists so a JSON fixture can round-trip the txid.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BroadcastResult {
    pub txid: String,
}

/// `fees` on `MempoolAcceptResult` in `src/daemon.rs`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TestTxFees {
    pub base: f64,
    #[serde(rename = "effective-feerate")]
    pub effective_feerate: f64,
    #[serde(rename = "effective-includes")]
    pub effective_includes: Vec<String>,
}

/// One element of the `POST /txs/test` JSON array.
///
/// `MempoolAcceptResult` in `src/daemon.rs`. `reject-reason` is optional:
/// an allowed row often omits it, and a missing key still deserializes.
/// `wtxid` is optional the same way. A missing key is `None`, not a copy
/// of `txid`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TestTxResult {
    pub txid: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub wtxid: Option<String>,
    #[serde(default)]
    pub allowed: Option<bool>,
    #[serde(default)]
    pub vsize: Option<u32>,
    #[serde(default)]
    pub fees: Option<TestTxFees>,
    #[serde(default, rename = "reject-reason")]
    pub reject_reason: Option<String>,
}

/// `GET /block/:hash/status`.
///
/// `BlockStatus` in `src/util/block.rs`. `height` and `next_best` are null
/// when the block is not in the best chain.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BlockStatus {
    pub in_best_chain: bool,
    pub height: Option<u64>,
    pub next_best: Option<String>,
}

/// `GET /tx/:txid/merkle-proof`.
///
/// The handler in `src/rest.rs` emits `block_height`, `merkle`, and `pos`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MerkleProof {
    pub block_height: u64,
    pub merkle: Vec<String>,
    pub pos: u64,
}

/// One output from `GET /tx/:txid/outspend/:index` or `GET /tx/:txid/outspends`.
///
/// `SpendingValue` in `src/rest.rs`. An unspent output is `spent: false` and
/// omits `txid`, `vin`, and `status`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Outspend {
    pub spent: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub txid: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub vin: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub status: Option<TxStatus>,
}

/// One row from `GET /address/:script/txs/summary`.
///
/// `TxHistorySummary` in `src/new_index/schema.rs`. `value` is signed.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TxSummary {
    pub txid: String,
    pub height: u64,
    pub value: i64,
    pub time: u64,
    pub tx_position: u64,
}

/// `fees` on one `tx-results` entry from `POST /txs/package`.
///
/// `MempoolFeesSubmitPackage` in `src/daemon.rs`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PackageTxFees {
    pub base: f64,
    #[serde(
        default,
        rename = "effective-feerate",
        skip_serializing_if = "Option::is_none"
    )]
    pub effective_feerate: Option<f64>,
    #[serde(
        default,
        rename = "effective-includes",
        skip_serializing_if = "Option::is_none"
    )]
    pub effective_includes: Option<Vec<String>>,
}

/// One value in `tx-results` from `POST /txs/package`.
///
/// `TxResult` in `src/daemon.rs`. `other-wtxid` is omitted when there is no
/// other witness id. A missing key is `None`. It is not copied from `txid`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PackageTx {
    pub txid: String,
    #[serde(
        default,
        rename = "other-wtxid",
        skip_serializing_if = "Option::is_none"
    )]
    pub other_wtxid: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub vsize: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fees: Option<PackageTxFees>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

/// `POST /txs/package`.
///
/// `SubmitPackageResult` in `src/daemon.rs`. A missing `replaced-transactions`
/// key is an empty list.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PackageResult {
    pub package_msg: String,
    #[serde(rename = "tx-results")]
    pub tx_results: BTreeMap<String, PackageTx>,
    #[serde(default, rename = "replaced-transactions")]
    pub replaced_transactions: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture(name: &str) -> String {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures")
            .join(name);
        std::fs::read_to_string(path).unwrap()
    }

    fn assert_round_trip<T>(body: &str) -> T
    where
        T: for<'de> Deserialize<'de> + Serialize + PartialEq + std::fmt::Debug,
    {
        let parsed: T = serde_json::from_str(body).expect("deserialize");
        let encoded = serde_json::to_string(&parsed).expect("serialize");
        let again: T = serde_json::from_str(&encoded).expect("round trip");
        assert_eq!(parsed, again);
        parsed
    }

    #[test]
    fn block_json_fixture() {
        let block: Block = assert_round_trip(&fixture("block.json"));
        assert_eq!(block.id, "block-id-1");
        assert_eq!(block.height, 100);
        assert_eq!(block.tx_count, 12);
        assert_eq!(block.timestamp, 1_600_000_000);
        assert_eq!(block.size, 1500);
        assert_eq!(block.weight, 4000);
        assert_eq!(block.merkle_root, "merkle-root-1");
        assert_eq!(block.previous_block_hash.as_deref(), Some("prev-block-1"));
        assert_eq!(block.median_time, 1_599_990_000);
        assert_eq!(block.nonce, Some(7));
        assert_eq!(block.bits, Some(386_604_799));
        assert_eq!(block.difficulty, Some(1.0));
        assert_eq!(block.version, 536_870_912);

        let genesis = r#"{"id":"g","height":0,"version":1,"timestamp":0,"tx_count":1,"size":1,"weight":4,"merkle_root":"m","mediantime":0,"nonce":0,"bits":0,"difficulty":1}"#;
        let parsed: Block = serde_json::from_str(genesis).expect("absent previousblockhash");
        assert_eq!(parsed.previous_block_hash, None);
    }

    #[test]
    fn transaction_json_fixture() {
        let tx: Transaction = assert_round_trip(&fixture("tx.json"));
        assert_eq!(tx.txid, "tx-1");
        assert_eq!(tx.fee, 4500);
        assert_eq!(tx.size, 180);
        assert_eq!(tx.weight, 720);
        let status = tx.status.expect("status");
        assert!(status.confirmed);
        assert_eq!(status.block_height, Some(100));
        assert_eq!(status.block_hash.as_deref(), Some("block-id-1"));
        assert_eq!(status.block_time, Some(1_600_000_000));

        let unconfirmed: Vec<Transaction> =
            serde_json::from_str(&fixture("address_txs.json")).unwrap();
        assert_eq!(unconfirmed[0].status.as_ref().unwrap().block_height, None);
        assert!(!unconfirmed[0].status.as_ref().unwrap().confirmed);

        let with_inputs = r#"{
            "txid":"with-inputs",
            "version":2,
            "locktime":0,
            "vin":[{
                "txid":"prev",
                "vout":0,
                "prevout":{
                    "scriptpubkey":"0014aa",
                    "scriptpubkey_asm":"OP_0 OP_PUSHBYTES_20 aa",
                    "scriptpubkey_type":"v0_p2wpkh",
                    "scriptpubkey_address":"bc1qexample",
                    "value":610677
                },
                "scriptsig":"",
                "scriptsig_asm":"",
                "witness":["3043","0236"],
                "is_coinbase":false,
                "sequence":4294967295
            }],
            "vout":[{
                "scriptpubkey":"76a914bb88ac",
                "scriptpubkey_asm":"OP_DUP",
                "scriptpubkey_type":"p2pkh",
                "value":344697
            }],
            "size":224,
            "weight":572,
            "sigops":1,
            "fee":584,
            "status":{"confirmed":true,"block_height":800000}
        }"#;
        let detailed: Transaction = assert_round_trip(with_inputs);
        assert_eq!(
            detailed.status.as_ref().unwrap().block_height,
            Some(800_000)
        );
        let input = &detailed.vin[0];
        assert_eq!(input.script_sig, "");
        assert_eq!(input.witness.as_ref().unwrap().len(), 2);
        assert_eq!(input.prevout.as_ref().unwrap().value, Some(610_677));
        assert_eq!(
            input.prevout.as_ref().unwrap().script_pubkey_type,
            "v0_p2wpkh"
        );
        assert_eq!(detailed.vout[0].script_pubkey, "76a914bb88ac");
        assert_eq!(detailed.vout[0].script_pubkey_address, None);
    }

    #[test]
    fn address_stats_json_fixture() {
        let stats: AddressStats = assert_round_trip(&fixture("address.json"));
        assert_eq!(stats.address.as_deref(), Some("xyz"));
        assert_eq!(stats.scripthash, None);
        assert_eq!(stats.chain_stats.tx_count, 9);
        assert_eq!(stats.chain_stats.funded_txo_count, 6);
        assert_eq!(stats.chain_stats.spent_txo_count, 2);
        assert_eq!(stats.chain_stats.funded_txo_sum, Some(500_000));
        assert_eq!(stats.chain_stats.spent_txo_sum, Some(100_000));
        assert_eq!(stats.mempool_stats.tx_count, 3);
        assert_eq!(stats.mempool_stats.funded_txo_sum, Some(2500));
    }

    #[test]
    fn utxo_json_fixture() {
        let rows: Vec<Utxo> = assert_round_trip(&fixture("address_utxos.json"));
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].txid, "utxo-tx-1");
        assert_eq!(rows[0].vout, 2);
        assert_eq!(rows[0].value, Some(42_000));
        assert!(rows[0].status.confirmed);
        assert_eq!(rows[0].status.block_height, Some(100));

        let unconfirmed: Vec<Utxo> = serde_json::from_str(
            r#"[{"txid":"u","vout":0,"value":1,"status":{"confirmed":false}}]"#,
        )
        .unwrap();
        assert!(!unconfirmed[0].status.confirmed);
    }

    #[test]
    fn recent_transaction_json_fixture() {
        let rows: Vec<RecentTransaction> = assert_round_trip(&fixture("mempool_recent.json"));
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].txid, "recent-tx-1");
        assert_eq!(rows[0].fee, 800);
        assert_eq!(rows[0].vsize, 141);
        assert_eq!(rows[0].value, Some(99_000));
    }

    #[test]
    fn mempool_summary_json_fixture() {
        let body =
            r#"{"count":2,"vsize":300,"total_fee":1500,"fee_histogram":[[10.5,200],[1,100]]}"#;
        let stats: MempoolSummary = assert_round_trip(body);
        assert_eq!(stats.count, 2);
        assert_eq!(stats.vsize, 300);
        assert_eq!(stats.total_fee, 1500);
        assert_eq!(stats.fee_histogram, vec![(10.5, 200), (1.0, 100)]);
    }

    #[test]
    fn fee_estimates_json_fixture() {
        let body = r#"{"6":5,"1":12.5,"144":1}"#;
        let estimates: FeeEstimates = assert_round_trip(body);
        assert_eq!(estimates.0.get(&1).copied(), Some(12.5));
        assert_eq!(estimates.0.get(&6).copied(), Some(5.0));
        assert_eq!(estimates.0.get(&144).copied(), Some(1.0));
    }

    #[test]
    fn broadcast_result_json_fixture() {
        let result: BroadcastResult = assert_round_trip(r#"{"txid":"broadcast-txid-1"}"#);
        assert_eq!(result.txid, "broadcast-txid-1");
    }

    #[test]
    fn test_tx_result_json_fixture() {
        let rows: Vec<TestTxResult> = assert_round_trip(&fixture("test_tx.json"));
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].txid, "test-tx-1");
        assert_eq!(rows[0].wtxid.as_deref(), Some("test-wtxid-1"));
        assert_eq!(rows[0].allowed, Some(false));
        assert_eq!(rows[0].vsize, Some(141));
        assert_eq!(
            rows[0].reject_reason.as_deref(),
            Some("min relay fee not met")
        );
        let fees = rows[0].fees.as_ref().expect("fees");
        assert_eq!(fees.effective_feerate, 1.0);
        assert_eq!(fees.effective_includes, vec!["test-tx-1".to_string()]);

        let absent: TestTxResult =
            assert_round_trip(r#"{"txid":"ok-tx","wtxid":"ok-wtxid","allowed":true,"vsize":100}"#);
        assert_eq!(absent.allowed, Some(true));
        assert_eq!(absent.reject_reason, None);
        assert_eq!(absent.fees, None);

        let rejected_without_reason: TestTxResult =
            serde_json::from_str(r#"{"txid":"bad-tx","wtxid":"bad-wtxid","allowed":false}"#)
                .expect("absent reject-reason still deserializes");
        assert_eq!(rejected_without_reason.reject_reason, None);
        assert_eq!(rejected_without_reason.wtxid.as_deref(), Some("bad-wtxid"));

        let txid_only: TestTxResult =
            serde_json::from_str(r#"{"txid":"only-txid"}"#).expect("missing wtxid");
        assert_eq!(txid_only.txid, "only-txid");
        assert_eq!(txid_only.wtxid, None);
    }
}
