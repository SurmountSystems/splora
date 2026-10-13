//! `parse_mempool_recent` maps `RecentJson.value` onto `RecentTx.value`.
//!
//! A missing key and JSON null become an empty string, never `"0"`.
//! Explicit `0` stays `"0"`. A present integer stays its decimal text.

use splora_frontend_shared::{RecentTx, parse_mempool_recent};

fn one(body: &str, message: &str) -> RecentTx {
    let rows = parse_mempool_recent(body).expect(message);
    assert_eq!(rows.len(), 1);
    rows.into_iter().next().expect("one row")
}

#[test]
fn missing_value_key_is_empty_string_not_zero() {
    let row = one(
        r#"[{"txid":"recent-missing-value","fee":10,"vsize":140}]"#,
        "missing value key is not a parse error",
    );
    assert_eq!(row.value, "");
    assert_ne!(row.value, "0");
}

#[test]
fn explicit_zero_stays_zero_text() {
    let row = one(
        r#"[{"txid":"recent-explicit-zero","fee":11,"vsize":141,"value":0}]"#,
        "explicit zero is not a parse error",
    );
    assert_eq!(row.value, "0");
}

#[test]
fn present_integer_stays_decimal_text() {
    let row = one(
        r#"[{"txid":"recent-present-99000","fee":12,"vsize":142,"value":99000}]"#,
        "present integer is not a parse error",
    );
    assert_eq!(row.value, "99000");
}

#[test]
fn json_null_value_is_empty_string_not_zero() {
    let row = one(
        r#"[{"txid":"recent-null-value","fee":13,"vsize":143,"value":null}]"#,
        "json null value is not a parse error",
    );
    assert_eq!(row.value, "");
    assert_ne!(row.value, "0");
}
