//! Whole line `output script type multisig` on a transaction output from `GET /tx/:txid`.
//!
//! GPUI `transaction_screen_paints_multisig_output_type` requires one line
//! whose entire text is `output script type multisig` when
//! `vout.scriptpubkey_type` is `multisig`. The count assert starts at
//! frontend/native/tests/rest_of_indexer.rs:6322 and the comparison is line
//! 6325. The scriptpubkey-type span and the `Type MULTISIG` line stay. An
//! empty string, a missing key, and JSON null omit this line. An input whose
//! prevout type is `multisig` does not gain this line. A non-multisig output
//! and the address page omit the line. A span that contains the words is not
//! that line. The bare lines `MULTISIG` and `multisig` stay off.

use splora_web::{address_rows_markup, transaction_requests, transaction_row_markup};

const TXID: &str = "msigone";
const PREV: &str = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
const LINE: &str = "output script type multisig";

fn tx(vin: &str, vout: &str) -> String {
    format!(
        r#"{{"txid":"{TXID}","version":1,"locktime":0,"vin":[{vin}],"vout":[{vout}],"size":10,"weight":400,"sigops":0,"fee":50,"status":{{"confirmed":true,"block_height":4}}}}"#
    )
}

fn empty_input() -> String {
    format!(
        r#"{{"txid":"{PREV}","vout":0,"prevout":null,"scriptsig":"","scriptsig_asm":"","is_coinbase":false,"sequence":1}}"#
    )
}

fn multisig_output() -> String {
    r#"{"scriptpubkey":"51","scriptpubkey_asm":"OP_1","scriptpubkey_type":"multisig","value":1}"#
        .to_string()
}

fn ordinary_output() -> String {
    r#"{"scriptpubkey":"0014aa","scriptpubkey_asm":"OP_0","scriptpubkey_type":"v0_p2wpkh","scriptpubkey_address":"bc1qordinary","value":9}"#
        .to_string()
}

fn empty_type_output() -> String {
    r#"{"scriptpubkey":"51","scriptpubkey_asm":"OP_1","scriptpubkey_type":"","value":1}"#
        .to_string()
}

fn missing_type_output() -> String {
    r#"{"scriptpubkey":"51","scriptpubkey_asm":"OP_1","value":1}"#.to_string()
}

fn null_type_output() -> String {
    r#"{"scriptpubkey":"51","scriptpubkey_asm":"OP_1","scriptpubkey_type":null,"value":1}"#
        .to_string()
}

fn multisig_input() -> String {
    format!(
        r#"{{"txid":"{PREV}","vout":0,"prevout":{{"scriptpubkey":"51","scriptpubkey_asm":"OP_1","scriptpubkey_type":"multisig","value":5}},"scriptsig":"deadbeef","scriptsig_asm":"OP_CHECKSIG","is_coinbase":false,"sequence":1}}"#
    )
}

fn address_page_with_multisig_output() -> String {
    let stats = r#"{"address":"3L4YUynB4X44rJBY9CmiLMN8Wjti49JCYB","chain_stats":{"tx_count":1,"funded_txo_count":1,"spent_txo_count":0,"funded_txo_sum":1000,"spent_txo_sum":0},"mempool_stats":{"tx_count":0,"funded_txo_count":0,"spent_txo_count":0,"funded_txo_sum":0,"spent_txo_sum":0}}"#;
    let txs = r#"[{"txid":"msigone","version":1,"locktime":0,"vin":[{"txid":"bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb","vout":0,"prevout":null,"scriptsig":"","scriptsig_asm":"","is_coinbase":false,"sequence":1}],"vout":[{"scriptpubkey":"51","scriptpubkey_asm":"OP_1","scriptpubkey_type":"multisig","value":1}],"size":10,"weight":400,"sigops":0,"fee":1}]"#;
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
fn multisig_output_paints_output_script_type_multisig_as_its_own_line() {
    let markup = paint(&tx(&empty_input(), &multisig_output()));
    assert!(
        markup.lines().any(|line| line == LINE),
        "no line equals output script type multisig: {markup}"
    );
    assert_eq!(
        markup.lines().filter(|line| *line == LINE).count(),
        1,
        "no line equals output script type multisig: {markup}"
    );
    assert!(
        markup
            .lines()
            .any(|line| line == LINE && !line.contains('<')),
        "output script type multisig was inside a span: {markup}"
    );
    assert!(
        !markup
            .lines()
            .any(|line| line == "MULTISIG" || line == "multisig"),
        "a bare type token was painted: {markup}"
    );
    assert!(
        markup.contains("<span class=\"scriptpubkey-type\">multisig</span>"),
        "the scriptpubkey-type span was dropped: {markup}"
    );
    assert!(
        markup.contains("<span class=\"output-type\">MULTISIG</span>"),
        "the uppercase multisig span was dropped: {markup}"
    );
    assert_eq!(
        markup
            .lines()
            .filter(|line| *line == "Type MULTISIG")
            .count(),
        1,
        "Type MULTISIG was dropped: {markup}"
    );

    let other = paint(&tx(&empty_input(), &ordinary_output()));
    assert_eq!(
        other.lines().filter(|line| *line == LINE).count(),
        0,
        "a non-multisig output painted {LINE}: {other}"
    );

    let empty = paint(&tx(&empty_input(), &empty_type_output()));
    assert_eq!(
        empty.lines().filter(|line| *line == LINE).count(),
        0,
        "an empty scriptpubkey_type painted {LINE}: {empty}"
    );
    assert!(
        !empty.contains("output script type"),
        "an empty type painted a script type line: {empty}"
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

    let input_multisig = paint(&tx(&multisig_input(), &ordinary_output()));
    assert_eq!(
        input_multisig.lines().filter(|line| *line == LINE).count(),
        0,
        "output script type multisig was painted on the input path: {input_multisig}"
    );

    let address = address_page_with_multisig_output();
    assert_eq!(
        address.lines().filter(|line| *line == LINE).count(),
        0,
        "the address page painted {LINE}: {address}"
    );
    assert!(
        address.contains("<span class=\"scriptpubkey-type\">multisig</span>"),
        "the address page dropped the scriptpubkey-type span: {address}"
    );

    let blank = transaction_row_markup("");
    assert_eq!(
        blank.lines().filter(|line| *line == LINE).count(),
        0,
        "an empty body painted {LINE}: {blank}"
    );
}
