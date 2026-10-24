//! Whole line `Type ANCHOR` on a transaction output from `GET /tx/:txid`.
//!
//! GPUI `transaction_screen_paints_anchor_output_type` requires one line whose
//! entire text is `Type ANCHOR` when `vout.scriptpubkey_type` is `anchor`.
//! The scriptpubkey-type span and the uppercase Type cell stay. An empty
//! string, a missing key, and JSON null omit that line. An input whose
//! prevout type is `anchor` does not gain this line. A non-anchor output and
//! the address page omit the line. A span that contains the words is not
//! that line.

use splora_web::{address_rows_markup, transaction_requests, transaction_row_markup};

const TXID: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
const PREV: &str = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
const LINE: &str = "Type ANCHOR";

fn tx(vin: &str, vout: &str) -> String {
    format!(
        r#"{{"txid":"{TXID}","version":1,"locktime":0,"vin":[{vin}],"vout":[{vout}],"size":10,"weight":40,"sigops":0,"fee":50,"status":{{"confirmed":true}}}}"#
    )
}

fn empty_input() -> String {
    format!(
        r#"{{"txid":"{PREV}","vout":0,"prevout":null,"scriptsig":"","scriptsig_asm":"","is_coinbase":false,"sequence":1}}"#
    )
}

fn anchor_output() -> String {
    r#"{"scriptpubkey":"5120aa","scriptpubkey_asm":"OP_1","scriptpubkey_type":"anchor","value":1}"#
        .to_string()
}

fn ordinary_output() -> String {
    r#"{"scriptpubkey":"0014aa","scriptpubkey_asm":"OP_0","scriptpubkey_type":"v0_p2wpkh","scriptpubkey_address":"bc1qordinary","value":9}"#
        .to_string()
}

fn empty_type_output() -> String {
    r#"{"scriptpubkey":"5120aa","scriptpubkey_asm":"OP_1","scriptpubkey_type":"","value":1}"#
        .to_string()
}

fn missing_type_output() -> String {
    r#"{"scriptpubkey":"5120aa","scriptpubkey_asm":"OP_1","value":1}"#.to_string()
}

fn null_type_output() -> String {
    r#"{"scriptpubkey":"5120aa","scriptpubkey_asm":"OP_1","scriptpubkey_type":null,"value":1}"#
        .to_string()
}

fn anchor_input() -> String {
    format!(
        r#"{{"txid":"{PREV}","vout":0,"prevout":{{"scriptpubkey":"5120aa","scriptpubkey_asm":"OP_1","scriptpubkey_type":"anchor","value":5}},"scriptsig":"deadbeef","scriptsig_asm":"OP_CHECKSIG","is_coinbase":false,"sequence":1}}"#
    )
}

fn address_page_with_anchor_output() -> String {
    let stats = r#"{"address":"3L4YUynB4X44rJBY9CmiLMN8Wjti49JCYB","chain_stats":{"tx_count":1,"funded_txo_count":1,"spent_txo_count":0,"funded_txo_sum":1000,"spent_txo_sum":0},"mempool_stats":{"tx_count":0,"funded_txo_count":0,"spent_txo_count":0,"funded_txo_sum":0,"spent_txo_sum":0}}"#;
    let txs = r#"[{"txid":"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa","version":1,"locktime":0,"vin":[{"txid":"bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb","vout":0,"prevout":null,"scriptsig":"","scriptsig_asm":"","is_coinbase":false,"sequence":1}],"vout":[{"scriptpubkey":"5120aa","scriptpubkey_asm":"OP_1","scriptpubkey_type":"anchor","value":1}],"size":10,"weight":40,"sigops":0,"fee":1}]"#;
    address_rows_markup(stats, txs, "")
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
fn anchor_output_paints_type_anchor_as_its_own_line() {
    let markup = paint(&tx(&empty_input(), &anchor_output()));
    assert!(
        markup.lines().any(|line| line == LINE),
        "no line equals Type ANCHOR: {markup}"
    );
    assert_eq!(
        markup.lines().filter(|line| *line == LINE).count(),
        1,
        "scriptpubkey_type anchor did not paint the exact line Type ANCHOR once: {markup}"
    );
    assert!(
        markup
            .lines()
            .any(|line| line == LINE && !line.contains('<')),
        "Type ANCHOR was inside a span: {markup}"
    );
    assert!(
        markup.contains("<span class=\"scriptpubkey-type\">anchor</span>"),
        "the scriptpubkey-type span was dropped: {markup}"
    );
    assert!(
        markup.contains("<span class=\"output-type-label\">Type</span>"),
        "the Type span was dropped: {markup}"
    );
    assert!(
        markup.contains("<span class=\"output-type\">ANCHOR</span>"),
        "the uppercase anchor span was dropped: {markup}"
    );

    let other = paint(&tx(&empty_input(), &ordinary_output()));
    assert_eq!(
        other.lines().filter(|line| *line == LINE).count(),
        0,
        "a non-anchor output painted {LINE}: {other}"
    );

    let empty = paint(&tx(&empty_input(), &empty_type_output()));
    assert_eq!(
        empty.lines().filter(|line| *line == LINE).count(),
        0,
        "an empty scriptpubkey_type painted {LINE}: {empty}"
    );

    let missing = paint(&tx(&empty_input(), &missing_type_output()));
    assert_eq!(
        missing.lines().filter(|line| *line == LINE).count(),
        0,
        "a missing scriptpubkey_type painted {LINE}: {missing}"
    );

    let null_type = paint(&tx(&empty_input(), &null_type_output()));
    assert_eq!(
        null_type.lines().filter(|line| *line == LINE).count(),
        0,
        "JSON null scriptpubkey_type painted {LINE}: {null_type}"
    );

    let input_anchor = paint(&tx(&anchor_input(), &ordinary_output()));
    assert_eq!(
        input_anchor.lines().filter(|line| *line == LINE).count(),
        0,
        "Type ANCHOR was painted on the input path: {input_anchor}"
    );
    assert!(
        input_anchor.contains("<span class=\"previous-output-type\">ANCHOR</span>"),
        "the previous-output span was rewritten: {input_anchor}"
    );

    let address = address_page_with_anchor_output();
    assert_eq!(
        address.lines().filter(|line| *line == LINE).count(),
        0,
        "the address page painted {LINE}: {address}"
    );
    assert!(
        address.contains("<span class=\"scriptpubkey-type\">anchor</span>"),
        "the address page dropped the scriptpubkey-type span: {address}"
    );
    assert!(
        address.contains("<span class=\"output-type\">ANCHOR</span>"),
        "the address page dropped the uppercase anchor span: {address}"
    );
}
