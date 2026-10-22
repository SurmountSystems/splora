//! `funded_txo_sum` and `spent_txo_sum` on address stats.
//!
//! A non-liquid indexer writes both keys as numbers, including `0`.
//! A liquid indexer omits both keys. A missing key is `None`, not a
//! parse error and not zero. Explicit `0` stays `Some(0)`. JSON `null`
//! is `None`.

use splora_frontend_shared::wire::AddressStats;

/// A script sum is `u64` until the wire field is `Option<u64>`.
/// The `u64` impl cannot be `None`, so a missing key or JSON `null`
/// fails in `serde_json::from_str` before `expect_none` runs.
trait ScriptSum {
    fn expect_none(self);
    fn expect_some(self, amount: u64);
}

impl ScriptSum for u64 {
    fn expect_none(self) {
        panic!("sum {self} is a number; a missing or null script sum is not None");
    }

    fn expect_some(self, amount: u64) {
        assert_eq!(self, amount);
    }
}

impl ScriptSum for Option<u64> {
    fn expect_none(self) {
        assert_eq!(self, None);
        assert_ne!(self, Some(0));
    }

    fn expect_some(self, amount: u64) {
        assert_eq!(self, Some(amount));
    }
}

fn address_json(chain_sums: &str, mempool_sums: &str) -> String {
    format!(
        r#"{{"address":"xyz","chain_stats":{{"tx_count":9,"funded_txo_count":6,"spent_txo_count":2{chain_sums}}},"mempool_stats":{{"tx_count":3,"funded_txo_count":1,"spent_txo_count":0{mempool_sums}}}}}"#
    )
}

#[test]
fn missing_sum_keys_are_none_not_zero() {
    let stats: AddressStats = serde_json::from_str(&address_json("", ""))
        .expect("missing script sum is not a parse error");
    stats.chain_stats.funded_txo_sum.expect_none();
    stats.chain_stats.spent_txo_sum.expect_none();
    stats.mempool_stats.funded_txo_sum.expect_none();
    stats.mempool_stats.spent_txo_sum.expect_none();
}

#[test]
fn explicit_zero_sums_stay_some_zero() {
    let sums = r#","funded_txo_sum":0,"spent_txo_sum":0"#;
    let stats: AddressStats = serde_json::from_str(&address_json(sums, sums)).unwrap();
    stats.chain_stats.funded_txo_sum.expect_some(0);
    stats.chain_stats.spent_txo_sum.expect_some(0);
    stats.mempool_stats.funded_txo_sum.expect_some(0);
    stats.mempool_stats.spent_txo_sum.expect_some(0);
}

#[test]
fn present_sum_integers_stay_some() {
    let stats: AddressStats = serde_json::from_str(&address_json(
        r#","funded_txo_sum":500000,"spent_txo_sum":100000"#,
        r#","funded_txo_sum":2500,"spent_txo_sum":0"#,
    ))
    .unwrap();
    stats.chain_stats.funded_txo_sum.expect_some(500_000);
    stats.chain_stats.spent_txo_sum.expect_some(100_000);
    stats.mempool_stats.funded_txo_sum.expect_some(2_500);
    stats.mempool_stats.spent_txo_sum.expect_some(0);
}

#[test]
fn json_null_sums_are_none_not_zero() {
    let sums = r#","funded_txo_sum":null,"spent_txo_sum":null"#;
    let stats: AddressStats = serde_json::from_str(&address_json(sums, sums))
        .expect("json null script sum is not a parse error");
    stats.chain_stats.funded_txo_sum.expect_none();
    stats.chain_stats.spent_txo_sum.expect_none();
    stats.mempool_stats.funded_txo_sum.expect_none();
    stats.mempool_stats.spent_txo_sum.expect_none();
}

fn sum_object<'a>(
    encoded: &'a serde_json::Value,
    side: &str,
) -> &'a serde_json::Map<String, serde_json::Value> {
    encoded[side]
        .as_object()
        .unwrap_or_else(|| panic!("{side} was not an object"))
}

#[test]
fn serialize_none_omits_both_sum_keys() {
    let stats: AddressStats = serde_json::from_str(&address_json("", "")).unwrap();
    let encoded = serde_json::to_value(&stats).unwrap();
    for side in ["chain_stats", "mempool_stats"] {
        let object = sum_object(&encoded, side);
        assert!(
            !object.contains_key("funded_txo_sum"),
            "{side} still has funded_txo_sum: {encoded}"
        );
        assert!(
            !object.contains_key("spent_txo_sum"),
            "{side} still has spent_txo_sum: {encoded}"
        );
        assert_eq!(
            object["tx_count"],
            if side == "chain_stats" { 9 } else { 3 }
        );
    }
}

#[test]
fn serialize_some_zero_keeps_the_key_and_the_digit_zero() {
    let sums = r#","funded_txo_sum":0,"spent_txo_sum":0"#;
    let stats: AddressStats = serde_json::from_str(&address_json(sums, sums)).unwrap();
    let text = serde_json::to_string(&stats).unwrap();
    let encoded: serde_json::Value = serde_json::from_str(&text).unwrap();
    for side in ["chain_stats", "mempool_stats"] {
        let object = sum_object(&encoded, side);
        assert_eq!(object["funded_txo_sum"], 0, "{side} funded was not 0");
        assert_eq!(object["spent_txo_sum"], 0, "{side} spent was not 0");
        assert_ne!(object["funded_txo_sum"], serde_json::Value::Null);
        assert_ne!(object["spent_txo_sum"], serde_json::Value::Null);
    }
    assert!(
        text.contains("\"funded_txo_sum\":0"),
        "encoded text dropped the zero digit: {text}"
    );
    assert!(
        text.contains("\"spent_txo_sum\":0"),
        "encoded text dropped the zero digit: {text}"
    );
}
