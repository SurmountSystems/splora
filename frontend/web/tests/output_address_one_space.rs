//! Whole lines with one space between an output address and the word value.
//!
//! GPUI `transaction_screen_paints_transaction_fee_label` and
//! `transaction_screen_paints_op_return_label` in
//! `frontend/native/tests/rest_of_indexer.rs` require these exact lines when
//! the output has an address and a published value. One space sits between
//! the address and `value`. The two-space blank-address line is a different
//! sentence and already passes. The address anchor and the value span stay.

use splora_web::{transaction_requests, transaction_row_markup};

const OPRET_LINE: &str = "output address bc1qopret value 3";
const ORDINARY_LINE: &str = "output address bc1qordinary value 9";
const FEE_LINE: &str = "output address bc1qfeeaddr value 3";

fn vin() -> String {
    r#"{"txid":"prev","vout":0,"prevout":{"scriptpubkey":"0014aa","scriptpubkey_asm":"OP_0","scriptpubkey_type":"v0_p2wpkh","scriptpubkey_address":"bc1qin","value":8},"scriptsig":"deadbeef","scriptsig_asm":"OP_CHECKSIG","witness":["abcdwitnesshex"],"is_coinbase":false,"sequence":1}"#.to_string()
}

fn tx(txid: &str, vout: &str) -> String {
    let vin = vin();
    format!(
        r#"{{"txid":"{txid}","version":1,"locktime":0,"vin":[{vin}],"vout":[{vout}],"size":10,"weight":400,"sigops":0,"fee":50,"status":{{"confirmed":true,"block_height":4,"block_hash":"tiphash1","block_time":1600000000}}}}"#
    )
}

fn paint(txid: &str, body: &str) -> String {
    let requests = transaction_requests(txid);
    assert_eq!(requests.len(), 1, "{requests:?}");
    assert_eq!(requests[0].method, "GET");
    assert_eq!(requests[0].path, format!("/tx/{txid}"));
    assert!(!requests[0].path.contains('?'));
    assert!(!requests[0].path.contains('#'));
    transaction_row_markup(body)
}

fn assert_one_whole_line(markup: &str, line: &str) {
    assert!(
        markup.lines().any(|row| row == line),
        "no line equals {line}: {markup}"
    );
    assert_eq!(
        markup.lines().filter(|row| *row == line).count(),
        1,
        "the line was dropped or repeated: {line}: {markup}"
    );
    assert!(
        markup.lines().any(|row| row == line && !row.contains('<')),
        "the sentence was inside a tag: {line}: {markup}"
    );
}

#[test]
fn output_address_one_space_bc1qopret_value_3() {
    let vout = r#"{"scriptpubkey":"6a0468656c6c6f","scriptpubkey_asm":"OP_RETURN 68656c6c6f","scriptpubkey_type":"op_return","scriptpubkey_address":"bc1qopret","value":3}"#;
    let markup = paint("opretaddr", &tx("opretaddr", vout));
    assert_one_whole_line(&markup, OPRET_LINE);
    assert!(
        markup.contains("<a class=\"address\" href=\"/address/bc1qopret\">bc1qopret</a>"),
        "the address anchor was dropped: {markup}"
    );
    assert!(
        markup.contains("<span class=\"value\">3</span>"),
        "the value span was dropped: {markup}"
    );
    assert!(
        !markup.lines().any(|row| row == "output address  value 3"),
        "the addressed op_return output painted the two-space blank line: {markup}"
    );
}

#[test]
fn output_address_one_space_bc1qordinary_value_9() {
    let vout = r#"{"scriptpubkey":"0014aa","scriptpubkey_asm":"OP_0","scriptpubkey_type":"v0_p2wpkh","scriptpubkey_address":"bc1qordinary","value":9}"#;
    let markup = paint("feelabel", &tx("feelabel", vout));
    assert_one_whole_line(&markup, ORDINARY_LINE);
    assert!(
        markup.contains("<a class=\"address\" href=\"/address/bc1qordinary\">bc1qordinary</a>"),
        "the address anchor was dropped: {markup}"
    );
    assert!(
        markup.contains("<span class=\"value\">9</span>"),
        "the value span was dropped: {markup}"
    );
    assert!(
        !markup.lines().any(|row| row == "output address  value 9"),
        "the addressed ordinary output painted the two-space blank line: {markup}"
    );
}

#[test]
fn output_address_one_space_bc1qfeeaddr_value_3() {
    let vout = r#"{"scriptpubkey":"0014bb","scriptpubkey_asm":"OP_0","scriptpubkey_type":"fee","scriptpubkey_address":"bc1qfeeaddr","value":3}"#;
    let markup = paint("feeaddr", &tx("feeaddr", vout));
    assert_one_whole_line(&markup, FEE_LINE);
    assert!(
        markup.contains("<a class=\"address\" href=\"/address/bc1qfeeaddr\">bc1qfeeaddr</a>"),
        "the address anchor was dropped: {markup}"
    );
    assert!(
        markup.contains("<span class=\"value\">3</span>"),
        "the value span was dropped: {markup}"
    );
    assert!(
        !markup.lines().any(|row| row == "output address  value 3"),
        "the addressed fee output painted the two-space blank line: {markup}"
    );
}
