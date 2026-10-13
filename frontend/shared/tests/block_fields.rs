//! Optional `nonce`, `bits`, and `difficulty` on the shared block type.
//!
//! A liquid indexer omits all three keys. A default indexer writes `nonce`
//! and `bits` as numbers, including `0`, and writes `difficulty` as a finite
//! number, including `0.0`. JSON null is `None`, not zero. Explicit `0`
//! stays `Some(0)`.

use splora_frontend_shared::wire::Block;

/// `nonce` and `bits` are `u32` until the wire fields are `Option<u32>`.
/// The `u32` impl cannot be `None`, so a missing key or JSON `null` fails
/// in `serde_json::from_str` before `expect_none` runs.
trait BlockInt {
    fn expect_none(self);
    fn expect_some(self, value: u32);
}

impl BlockInt for u32 {
    fn expect_none(self) {
        panic!("field {self} is a number; a missing or null block field is not None");
    }

    fn expect_some(self, value: u32) {
        assert_eq!(self, value);
    }
}

impl BlockInt for Option<u32> {
    fn expect_none(self) {
        assert_eq!(self, None);
        assert_ne!(self, Some(0));
    }

    fn expect_some(self, value: u32) {
        assert_eq!(self, Some(value));
    }
}

/// `difficulty` is `f64` until the wire field is `Option<f64>`.
/// The `f64` impl cannot be `None`, so a missing key or JSON `null` fails
/// in `serde_json::from_str` before `expect_none` runs.
trait BlockDifficulty {
    fn expect_none(self);
    fn expect_some(self, value: f64);
}

impl BlockDifficulty for f64 {
    fn expect_none(self) {
        panic!("difficulty {self} is a number; a missing or null difficulty is not None");
    }

    fn expect_some(self, value: f64) {
        assert_eq!(self, value);
    }
}

impl BlockDifficulty for Option<f64> {
    fn expect_none(self) {
        assert_eq!(self, None);
        assert_ne!(self, Some(0.0));
    }

    fn expect_some(self, value: f64) {
        assert_eq!(self, Some(value));
    }
}

fn block_json(tail: &str) -> String {
    format!(
        r#"{{"id":"block-id-1","height":100,"version":536870912,"timestamp":1600000000,"tx_count":12,"size":1500,"weight":4000,"merkle_root":"merkle-root-1","mediantime":1599990000{tail}}}"#
    )
}

#[test]
fn missing_nonce_bits_and_difficulty_are_none() {
    let block: Block =
        serde_json::from_str(&block_json("")).expect("missing block field is not a parse error");
    block.nonce.expect_none();
    block.bits.expect_none();
    block.difficulty.expect_none();
}

#[test]
fn explicit_zero_stays_some_zero() {
    let block: Block = serde_json::from_str(&block_json(
        r#","nonce":0,"bits":0,"difficulty":0.0"#,
    ))
    .expect("explicit zero");
    block.nonce.expect_some(0);
    block.bits.expect_some(0);
    block.difficulty.expect_some(0.0);
}

#[test]
fn present_numbers_stay_some() {
    let block: Block = serde_json::from_str(&block_json(
        r#","nonce":7,"bits":386604799,"difficulty":1.0"#,
    ))
    .expect("present numbers");
    block.nonce.expect_some(7);
    block.bits.expect_some(386_604_799);
    block.difficulty.expect_some(1.0);
}

#[test]
fn json_null_is_none_not_zero() {
    let block: Block = serde_json::from_str(&block_json(
        r#","nonce":null,"bits":null,"difficulty":null"#,
    ))
    .expect("json null block field is not a parse error");
    block.nonce.expect_none();
    block.bits.expect_none();
    block.difficulty.expect_none();
}

#[test]
fn serialize_none_omits_nonce_bits_and_difficulty() {
    let block: Block =
        serde_json::from_str(&block_json("")).expect("missing block field is not a parse error");
    block.nonce.expect_none();
    block.bits.expect_none();
    block.difficulty.expect_none();
    let encoded = serde_json::to_value(&block).expect("serialize");
    let object = encoded.as_object().expect("object");
    for key in ["nonce", "bits", "difficulty"] {
        assert!(
            !object.contains_key(key),
            "None must omit {key}, got {encoded}"
        );
    }
}

#[test]
fn serialize_some_zero_keeps_the_key_and_the_digit_zero() {
    let block: Block = serde_json::from_str(&block_json(
        r#","nonce":0,"bits":0,"difficulty":0.0"#,
    ))
    .expect("explicit zero");
    block.nonce.expect_some(0);
    block.bits.expect_some(0);
    block.difficulty.expect_some(0.0);
    let text = serde_json::to_string(&block).expect("serialize");
    let encoded: serde_json::Value = serde_json::from_str(&text).expect("encoded json");
    assert_eq!(encoded["nonce"], 0);
    assert_eq!(encoded["bits"], 0);
    assert_eq!(encoded["difficulty"], 0.0);
    assert_ne!(encoded["nonce"], serde_json::Value::Null);
    assert_ne!(encoded["bits"], serde_json::Value::Null);
    assert_ne!(encoded["difficulty"], serde_json::Value::Null);
    assert!(
        text.contains("\"nonce\":0"),
        "encoded text dropped the zero digit: {text}"
    );
    assert!(
        text.contains("\"bits\":0"),
        "encoded text dropped the zero digit: {text}"
    );
    assert!(
        text.contains("\"difficulty\":0.0") || text.contains("\"difficulty\":0"),
        "encoded text dropped the zero digit: {text}"
    );
}
