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

/// `paint_show_more("showin18", 18, 1)` in
/// `transaction_screen_paints_show_more_inputs`.
/// The one output is `show_more_output(0)`: empty script, empty type, no
/// address, amount 0. That amount is the output index.
fn show_more_input(index: usize) -> String {
    format!(
        r#"{{"txid":"prev","vout":{index},"prevout":null,"scriptsig":"","scriptsig_asm":"","is_coinbase":false,"sequence":1}}"#
    )
}

fn show_more_output(index: usize) -> String {
    format!(r#"{{"scriptpubkey":"","scriptpubkey_asm":"","scriptpubkey_type":"","value":{index}}}"#)
}

fn show_more_tx(txid: &str, inputs: usize, outputs: usize) -> String {
    let vin = (0..inputs)
        .map(show_more_input)
        .collect::<Vec<_>>()
        .join(",");
    let vout = (0..outputs)
        .map(show_more_output)
        .collect::<Vec<_>>()
        .join(",");
    format!(
        r#"{{"txid":"{txid}","version":1,"locktime":0,"vin":[{vin}],"vout":[{vout}],"size":10,"weight":40,"sigops":0,"fee":1,"status":{{"confirmed":false}}}}"#
    )
}

/// `transaction_screen_paints_transaction_fee_label` and
/// `transaction_screen_paints_p2pk_label` share this input.
/// Prevout address `bc1qin`, amount 8, type `v0_p2wpkh`.
fn fee_or_p2pk_vin() -> String {
    r#"{"txid":"prev","vout":0,"prevout":{"scriptpubkey":"0014aa","scriptpubkey_asm":"OP_0","scriptpubkey_type":"v0_p2wpkh","scriptpubkey_address":"bc1qin","value":8},"scriptsig":"deadbeef","scriptsig_asm":"OP_CHECKSIG","witness":["abcdwitnesshex"],"is_coinbase":false,"sequence":1}"#.to_string()
}

fn confirmed_tx(txid: &str, vout: &str) -> String {
    let vin = fee_or_p2pk_vin();
    format!(
        r#"{{"txid":"{txid}","version":1,"locktime":0,"vin":[{vin}],"vout":[{vout}],"size":10,"weight":400,"sigops":0,"fee":50,"status":{{"confirmed":true,"block_height":4,"block_hash":"tiphash1","block_time":1600000000}}}}"#
    )
}

fn paint_txid(txid: &str, body: &str) -> String {
    let requests = transaction_requests(txid);
    assert_eq!(requests.len(), 1, "{requests:?}");
    assert_eq!(requests[0].method, "GET");
    assert_eq!(requests[0].path, format!("/tx/{txid}"));
    assert!(!requests[0].path.contains('?'));
    assert!(!requests[0].path.contains('#'));
    transaction_row_markup(body)
}

#[test]
fn output_address_value_0() {
    let markup = paint_txid("showin18", &show_more_tx("showin18", 18, 1));
    assert!(
        markup.lines().any(|line| line == "output address  value 0"),
        "no line equals output address  value 0: {markup}"
    );
}

#[test]
fn output_address_value_2() {
    let vout = r#"{"scriptpubkey":"","scriptpubkey_asm":"","scriptpubkey_type":"fee","scriptpubkey_address":"","value":2}"#;
    let markup = paint_txid("feeempty", &confirmed_tx("feeempty", vout));
    assert!(
        markup.lines().any(|line| line == "output address  value 2"),
        "no line equals output address  value 2: {markup}"
    );
}

#[test]
fn output_address_value_4() {
    let vout = r#"{"scriptpubkey":"0014aa","scriptpubkey_asm":"OP_0","scriptpubkey_type":"p2pk","scriptpubkey_address":null,"value":4}"#;
    let markup = paint_txid("p2pknulladdr", &confirmed_tx("p2pknulladdr", vout));
    assert!(
        markup.lines().any(|line| line == "output address  value 4"),
        "no line equals output address  value 4: {markup}"
    );
}

/// Blank output address and value 11 from `GET /tx/:txid`.
/// Two spaces sit between `address` and `value`. Not value 0, 2, or 4.
#[test]
fn output_address_value_11() {
    let markup = paint_txid(
        "blankaddr11",
        &confirmed_tx("blankaddr11", &show_more_output(11)),
    );
    assert!(
        markup
            .lines()
            .any(|line| line == "output address  value 11"),
        "no line equals output address  value 11: {markup}"
    );
}
