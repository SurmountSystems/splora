//! Serde shapes for indexer JSON.
//!
//! Fields follow the structs this tree already serializes. String parsers in
//! [`crate::parse`] stay as they are. Liquid-only keys are not modeled; extra
//! keys are ignored.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

/// One block from `GET /block/:hash` or an element of `GET /blocks`.
///
/// `BlockValue` in `src/rest.rs`. `previousblockhash` is null for the genesis
/// block and is omitted by some clients; both deserialize.
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
    pub nonce: u32,
    pub bits: u32,
    pub difficulty: f64,
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

/// One input from `TransactionValue` in `src/rest.rs` (non-liquid).
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

/// One output from `TransactionValue` in `src/rest.rs` (non-liquid).
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
    pub value: u64,
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

/// `ScriptStats` in `src/new_index/schema.rs` (non-liquid).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScriptStats {
    pub tx_count: u64,
    pub funded_txo_count: u64,
    pub spent_txo_count: u64,
    pub funded_txo_sum: u64,
    pub spent_txo_sum: u64,
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
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Utxo {
    pub txid: String,
    pub vout: u32,
    pub status: TxStatus,
    pub value: u64,
}

/// One row from `GET /mempool/recent`.
///
/// `TxOverview` in `src/new_index/mempool.rs` (non-liquid).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RecentTransaction {
    pub txid: String,
    pub fee: u64,
    pub vsize: u32,
    pub value: u64,
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
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TestTxResult {
    pub txid: String,
    pub wtxid: String,
    #[serde(default)]
    pub allowed: Option<bool>,
    #[serde(default)]
    pub vsize: Option<u32>,
    #[serde(default)]
    pub fees: Option<TestTxFees>,
    #[serde(default, rename = "reject-reason")]
    pub reject_reason: Option<String>,
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
        assert_eq!(block.nonce, 7);
        assert_eq!(block.bits, 386_604_799);
        assert_eq!(block.difficulty, 1.0);
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
        assert_eq!(input.prevout.as_ref().unwrap().value, 610_677);
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
        assert_eq!(stats.chain_stats.funded_txo_sum, 500_000);
        assert_eq!(stats.chain_stats.spent_txo_sum, 100_000);
        assert_eq!(stats.mempool_stats.tx_count, 3);
        assert_eq!(stats.mempool_stats.funded_txo_sum, 2500);
    }

    #[test]
    fn utxo_json_fixture() {
        let rows: Vec<Utxo> = assert_round_trip(&fixture("address_utxos.json"));
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].txid, "utxo-tx-1");
        assert_eq!(rows[0].vout, 2);
        assert_eq!(rows[0].value, 42_000);
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
        assert_eq!(rows[0].value, 99_000);
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
        assert_eq!(rows[0].wtxid, "test-wtxid-1");
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
    }
}
