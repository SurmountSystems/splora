//! `is_pegin` on a transaction input.
//!
//! A liquid indexer always writes the bool, including false. A Bitcoin
//! body omits the key. A missing key is `None`, not false. Explicit
//! `false` stays `Some(false)`. JSON `null` is `None`.

use splora_frontend_shared::wire::TxInput;

/// `is_pegin` is `bool` until the wire field is `Option<bool>`.
/// The `bool` impl cannot be `None`. A missing key parses as `false`
/// because of `#[serde(default)]`, then `expect_none` panics. JSON
/// `null` fails in `serde_json::from_str` before `expect_none` runs.
trait PegIn {
    fn expect_none(self);
    fn expect_some(self, value: bool);
}

impl PegIn for bool {
    fn expect_none(self) {
        panic!("is_pegin {self} is a bool; a missing or null is_pegin is not None");
    }

    fn expect_some(self, value: bool) {
        assert_eq!(self, value);
    }
}

impl PegIn for Option<bool> {
    fn expect_none(self) {
        assert_eq!(self, None);
        assert_ne!(self, Some(false));
    }

    fn expect_some(self, value: bool) {
        assert_eq!(self, Some(value));
    }
}

fn input_json(is_pegin: &str) -> String {
    format!(
        r#"{{"txid":"prev-tx","vout":0,"prevout":null,"scriptsig":"","scriptsig_asm":"","is_coinbase":false,"sequence":4294967295{is_pegin}}}"#
    )
}

#[test]
fn missing_is_pegin_is_none_not_false() {
    let input: TxInput =
        serde_json::from_str(&input_json("")).expect("missing is_pegin is not a parse error");
    input.is_pegin.expect_none();
}

#[test]
fn explicit_false_is_some_false() {
    let input: TxInput =
        serde_json::from_str(&input_json(r#","is_pegin":false"#)).expect("explicit false");
    input.is_pegin.expect_some(false);
}

#[test]
fn explicit_true_is_some_true() {
    let input: TxInput =
        serde_json::from_str(&input_json(r#","is_pegin":true"#)).expect("explicit true");
    input.is_pegin.expect_some(true);
}

#[test]
fn json_null_is_pegin_is_none_not_false() {
    let input: TxInput = serde_json::from_str(&input_json(r#","is_pegin":null"#))
        .expect("json null is_pegin is not a parse error");
    input.is_pegin.expect_none();
}

fn encoded_object(input: &TxInput) -> serde_json::Map<String, serde_json::Value> {
    let encoded = serde_json::to_value(input).expect("serialize input");
    encoded
        .as_object()
        .expect("input encodes as an object")
        .clone()
}

#[test]
fn serialize_none_omits_is_pegin() {
    let input: TxInput =
        serde_json::from_str(&input_json("")).expect("missing is_pegin is not a parse error");
    let object = encoded_object(&input);
    assert!(
        !object.contains_key("is_pegin"),
        "none still has is_pegin: {object:?}"
    );
}

#[test]
fn serialize_some_false_keeps_the_key_and_false() {
    let input: TxInput =
        serde_json::from_str(&input_json(r#","is_pegin":false"#)).expect("explicit false");
    let object = encoded_object(&input);
    assert!(
        object.contains_key("is_pegin"),
        "some false omitted is_pegin: {object:?}"
    );
    assert_eq!(object["is_pegin"], serde_json::Value::Bool(false));
    assert_ne!(object["is_pegin"], serde_json::Value::Null);
}
