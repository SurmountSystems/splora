//! `value` on `GET /mempool/recent`.
//!
//! `TxOverview` in `src/new_index/mempool.rs` emits `value` only when the
//! indexer is not built with `liquid`. A missing key is `None`, not zero.
//! Explicit `0` stays `Some(0)`. JSON `null` is `None` and is omitted on
//! serialize. Nothing writes null. The painter prints `Some` as the integer
//! text. It does not convert satoshis to BTC.

use splora_frontend_shared::wire::{RecentTransaction, recent_value_text};

fn one_row(body: &str) -> RecentTransaction {
    let rows: Vec<RecentTransaction> =
        serde_json::from_str(body).expect("a recent row is not a parse error");
    assert_eq!(rows.len(), 1);
    rows.into_iter().next().expect("one row")
}

#[test]
fn missing_value_key_is_none_not_zero() {
    let tx = one_row(r#"[{"txid":"omitted","fee":1,"vsize":2}]"#);
    assert_eq!(tx.value, None);
    assert_ne!(tx.value, Some(0));
}

#[test]
fn explicit_zero_stays_some_zero() {
    let tx = one_row(r#"[{"txid":"explicit-zero","fee":1,"vsize":2,"value":0}]"#);
    assert_eq!(tx.value, Some(0));
}

#[test]
fn present_integer_stays_some() {
    let tx = one_row(r#"[{"txid":"present","fee":1,"vsize":2,"value":99000}]"#);
    assert_eq!(tx.value, Some(99_000));
}

#[test]
fn json_null_is_none_and_serialize_omits_the_key() {
    let tx = one_row(r#"[{"txid":"cleared","fee":1,"vsize":2,"value":null}]"#);
    assert_eq!(tx.value, None);
    let encoded = serde_json::to_value(&tx).expect("serialize");
    assert!(encoded.get("value").is_none());
    assert_ne!(encoded.get("value"), Some(&serde_json::Value::Null));
    let text = serde_json::to_string(&tx).expect("serialize text");
    assert!(!text.contains("null"), "{text}");
    assert!(!text.contains("\"value\""), "{text}");
    assert!(!text.contains('0'), "{text}");
}

#[test]
fn painter_none_without_commitment_is_blank() {
    let text = recent_value_text(None, "");
    assert_eq!(text, "");
    assert_ne!(text, "0");
    assert_ne!(text, "None");
    assert_ne!(text, "0.00100000 LBTC");
}

#[test]
fn painter_none_with_commitment_is_confidential() {
    assert_eq!(recent_value_text(None, "aa"), "Confidential");
}

#[test]
fn painter_some_zero_is_the_integer_zero() {
    assert_eq!(recent_value_text(Some(0), ""), "0");
    assert_eq!(recent_value_text(Some(0), "aa"), "0");
}

#[test]
fn painter_some_positive_is_the_integer_text() {
    assert_eq!(recent_value_text(Some(99_000), ""), "99000");
    assert_eq!(recent_value_text(Some(99_000), "aa"), "99000");
}
