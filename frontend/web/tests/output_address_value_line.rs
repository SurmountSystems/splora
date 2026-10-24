//! Whole line `output address  value 1` on a transaction output from `GET /tx/:txid`.
//!
//! GPUI `transaction_screen_paints_anchor_output_type` in
//! `frontend/native/tests/rest_of_indexer.rs` requires one line whose entire
//! text is `output address  value 1` when the output has no address and the
//! amount is 1. Two spaces sit between `address` and `value`. The value span
//! stays. An output that has an address omits that exact line. An output
//! whose amount is not 1 omits it. A single-space sentence is not this line.

use splora_web::{transaction_requests, transaction_row_markup};

const TXID: &str = "atypeone";
/// Two spaces between address and value. Do not collapse them.
const LINE: &str = "output address  value 1";
const ONE_SPACE: &str = "output address value 1";
const ANC_HEX: &str = "ancpubhexdead";
const ANC_ASM: &str = "ancasmdead";
const ANC_SIG: &str = "ancsighexdead";
const ANC_SIG_ASM: &str = "ancsigasmdead";
const ANC_WIT: &str = "ancwitdead";
const ANC_INNER: &str = "ancinnerdead";
const ANC_LEAF: &str = "ancleafdead";

fn anchor_vin() -> String {
    format!(
        r#"{{"txid":"prev","vout":0,"prevout":{{"scriptpubkey":"{ANC_HEX}","scriptpubkey_asm":"{ANC_ASM}","scriptpubkey_type":"v0_p2wpkh","value":5}},"scriptsig":"{ANC_SIG}","scriptsig_asm":"{ANC_SIG_ASM}","witness":["{ANC_WIT}"],"is_coinbase":false,"sequence":1,"inner_witnessscript_asm":"{ANC_INNER}","leaf_version":"{ANC_LEAF}"}}"#
    )
}

fn anchor_vout(type_field: &str, value: u64) -> String {
    format!(
        r#"{{"scriptpubkey":"{ANC_HEX}","scriptpubkey_asm":"{ANC_ASM}",{type_field}"value":{value}}}"#
    )
}

fn addressed_vout() -> String {
    r#"{"scriptpubkey":"0014aa","scriptpubkey_asm":"OP_0","scriptpubkey_type":"v0_p2wpkh","scriptpubkey_address":"bc1qordinary","value":1}"#
        .to_string()
}

fn anchor_tx(txid: &str, vout: &str) -> String {
    format!(
        r#"{{"txid":"{txid}","version":1,"locktime":0,"vin":[{}],"vout":[{vout}],"size":10,"weight":400,"sigops":0,"fee":50,"status":{{"confirmed":true,"block_height":4}}}}"#,
        anchor_vin()
    )
}

fn paint(body: &str) -> String {
    let requests = transaction_requests(TXID);
    assert_eq!(requests.len(), 1, "{requests:?}");
    assert_eq!(requests[0].method, "GET");
    assert_eq!(requests[0].path, format!("/tx/{TXID}"));
    assert!(!requests[0].path.contains('?'));
    assert!(!requests[0].path.contains('#'));
    transaction_row_markup(body)
}

#[test]
fn anchor_output_paints_output_address_value_as_its_own_line() {
    assert_ne!(LINE, ONE_SPACE);
    assert!(
        LINE.contains("address  value"),
        "address and value must be separated by two spaces"
    );
    let markup = paint(&anchor_tx(
        TXID,
        &anchor_vout(r#""scriptpubkey_type":"anchor","#, 1),
    ));
    assert!(
        markup.lines().any(|line| line == LINE),
        "no line equals output address  value 1: {markup}"
    );
    assert_eq!(
        markup.lines().filter(|line| *line == LINE).count(),
        1,
        "no line equals output address  value 1: {markup}"
    );
    assert!(
        markup
            .lines()
            .any(|line| line == LINE && !line.contains('<')),
        "the sentence was inside a tag: {markup}"
    );
    assert!(
        markup.contains("<span class=\"value\">1</span>"),
        "the value span was dropped: {markup}"
    );
    assert!(
        markup.contains("<span class=\"scriptpubkey-type\">anchor</span>"),
        "the scriptpubkey-type span was dropped: {markup}"
    );

    let addressed = paint(&anchor_tx("atypeaddr", &addressed_vout()));
    assert_eq!(
        addressed.lines().filter(|line| *line == LINE).count(),
        0,
        "an addressed output painted {LINE}: {addressed}"
    );
    assert!(
        addressed.contains("bc1qordinary"),
        "the address span was dropped: {addressed}"
    );

    let other_value = paint(&anchor_tx(
        "atypeval2",
        &anchor_vout(r#""scriptpubkey_type":"anchor","#, 2),
    ));
    assert_eq!(
        other_value.lines().filter(|line| *line == LINE).count(),
        0,
        "a value other than 1 painted {LINE}: {other_value}"
    );
}
