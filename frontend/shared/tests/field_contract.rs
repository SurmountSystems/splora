//! Locks the indexer fields the shared wire structs already expose.
//!
//! Block hash is `Block.id` (`BlockValue.id` in `src/rest.rs`). Confirming
//! block hash is `TxStatus.block_hash`. The explorer is not finished.

use std::path::Path;

use splora_frontend_shared::wire::{
    AddressStats, Block, FeeEstimates, MempoolSummary, Transaction, Utxo,
};

fn fixture(name: &str) -> String {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(name);
    std::fs::read_to_string(path).unwrap()
}

fn without_key(body: &str, key: &str) -> String {
    let mut value: serde_json::Value = serde_json::from_str(body).unwrap();
    let removed = value.as_object_mut().expect("object").remove(key).is_some();
    assert!(removed, "fixture had no {key}");
    value.to_string()
}

fn assert_missing_field(err: &serde_json::Error, key: &str) {
    let message = err.to_string();
    assert!(
        message.contains("missing field") && message.contains(key),
        "{key} error was {message}"
    );
}

#[test]
fn wire_structs_expose_block_tx_address_utxo_mempool_and_fee_fields() {
    let block: Block = serde_json::from_str(&fixture("block.json")).unwrap();
    let hash = block.id.as_str();
    assert_eq!(block.height, 100);
    assert_eq!(hash, "block-id-1");
    assert_eq!(block.tx_count, 12);
    assert_eq!(block.timestamp, 1_600_000_000);

    let encoded = serde_json::to_value(&block).unwrap();
    assert_eq!(encoded["height"], 100);
    assert_eq!(encoded["id"], "block-id-1");
    assert_eq!(encoded["tx_count"], 12);
    assert_eq!(encoded["timestamp"], 1_600_000_000);

    let tx: Transaction = serde_json::from_str(&fixture("tx.json")).unwrap();
    assert_eq!(tx.txid, "tx-1");
    assert_eq!(tx.fee, 4500);
    let status = tx.status.expect("status");
    assert_eq!(status.block_hash.as_deref(), Some("block-id-1"));
    assert_eq!(status.block_height, Some(100));

    let stats: AddressStats = serde_json::from_str(&fixture("address.json")).unwrap();
    assert_eq!(stats.address.as_deref(), Some("xyz"));
    assert_eq!(stats.chain_stats.funded_txo_sum, 500_000);

    let utxos: Vec<Utxo> = serde_json::from_str(&fixture("address_utxos.json")).unwrap();
    assert_eq!(utxos.len(), 1);
    assert_eq!(utxos[0].value, 42_000);

    let mempool: MempoolSummary = serde_json::from_str(
        r#"{"count":2,"vsize":300,"total_fee":1500,"fee_histogram":[[10.5,200],[1,100]]}"#,
    )
    .unwrap();
    assert_eq!(mempool.count, 2);

    let estimates: FeeEstimates = serde_json::from_str(r#"{"6":5,"1":12.5,"144":1}"#).unwrap();
    assert_eq!(estimates.0.get(&1).copied(), Some(12.5));
    assert_eq!(estimates.0.get(&6).copied(), Some(5.0));
}

#[test]
fn wire_structs_reject_json_that_omits_a_required_field() {
    let block = fixture("block.json");
    for key in ["height", "id", "tx_count", "timestamp"] {
        let err = serde_json::from_str::<Block>(&without_key(&block, key)).unwrap_err();
        assert_missing_field(&err, key);
    }

    let tx = fixture("tx.json");
    for key in ["txid", "fee"] {
        let err = serde_json::from_str::<Transaction>(&without_key(&tx, key)).unwrap_err();
        assert_missing_field(&err, key);
    }

    let mut address: serde_json::Value = serde_json::from_str(&fixture("address.json")).unwrap();
    assert!(
        address["chain_stats"]
            .as_object_mut()
            .unwrap()
            .remove("funded_txo_sum")
            .is_some()
    );
    let err = serde_json::from_str::<AddressStats>(&address.to_string()).unwrap_err();
    assert_missing_field(&err, "funded_txo_sum");

    let mut utxos: serde_json::Value =
        serde_json::from_str(&fixture("address_utxos.json")).unwrap();
    assert!(utxos[0].as_object_mut().unwrap().remove("value").is_some());
    let err = serde_json::from_str::<Vec<Utxo>>(&utxos.to_string()).unwrap_err();
    assert_missing_field(&err, "value");

    let mempool = r#"{"count":2,"vsize":300,"total_fee":1500,"fee_histogram":[]}"#;
    let err = serde_json::from_str::<MempoolSummary>(&without_key(mempool, "count")).unwrap_err();
    assert_missing_field(&err, "count");
}
