//! `other-wtxid` on one `tx-results` entry from `POST /txs/package`.
//!
//! `TxResult` in `src/daemon.rs` carries that key. The indexer omits it when
//! there is no other witness id. A missing key stays absent. This test does
//! not fill one in from `txid` or from the `tx-results` map key.

use splora_frontend_shared::parse_package;

/// 32-byte witness id, hex. Not the entry's `txid` and not the map key.
const OTHER_WTXID: &str = "cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc";

#[test]
fn package_tx_keeps_other_wtxid_and_omits_a_missing_key() {
    let present = r#"{
        "package_msg":"success",
        "tx-results":{
            "wtxid-1":{
                "txid":"txid-1",
                "other-wtxid":"cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc",
                "vsize":141
            }
        }
    }"#;
    let parsed = parse_package(present).expect("package with other-wtxid");
    let tx = parsed.tx_results.get("wtxid-1").expect("wtxid-1");
    assert_eq!(tx.txid, "txid-1");
    assert_eq!(tx.other_wtxid.as_deref(), Some(OTHER_WTXID));
    assert_ne!(tx.other_wtxid.as_deref(), Some(tx.txid.as_str()));
    assert_ne!(tx.other_wtxid.as_deref(), Some("wtxid-1"));

    let encoded = serde_json::to_value(tx).expect("serialize present");
    assert_eq!(encoded["other-wtxid"], OTHER_WTXID);

    let absent = r#"{
        "package_msg":"success",
        "tx-results":{
            "wtxid-2":{"txid":"txid-2","error":"txn-already-in-mempool"}
        }
    }"#;
    let missing = parse_package(absent).expect("package without other-wtxid");
    let err = missing.tx_results.get("wtxid-2").expect("wtxid-2");
    assert_eq!(err.txid, "txid-2");
    assert_eq!(err.other_wtxid, None);

    let encoded = serde_json::to_value(err).expect("serialize absent");
    assert!(
        encoded.get("other-wtxid").is_none(),
        "a missing other-wtxid was written back as {encoded}"
    );
    assert_eq!(encoded["txid"], "txid-2");
}
