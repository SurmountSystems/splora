//! Whole line `output script type v0_p2wsh` on a transaction output from `GET /tx/:txid`.
//!
//! GPUI `transaction_screen_paints_v0_p2wsh_output_type` requires one line
//! whose entire text is `output script type v0_p2wsh` when
//! `vout.scriptpubkey_type` is `v0_p2wsh`. The count assert starts at
//! frontend/native/tests/rest_of_indexer.rs:6075 and the comparison is line
//! 6078. The scriptpubkey-type span and the `Type V0_P2WSH` line stay. An
//! empty string, a missing key, and JSON null omit this line. An input whose
//! prevout type is `v0_p2wsh` does not gain this line. A non-v0_p2wsh output
//! and the address page omit the line. A span that contains the words is not
//! that line. The bare lines `V0_P2WSH` and `v0_p2wsh` stay off.

use splora_web::{address_rows_markup, transaction_requests, transaction_row_markup};

const TXID: &str = "wshone";
const PREV: &str = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
const LINE: &str = "output script type v0_p2wsh";

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

fn v0_p2wsh_output() -> String {
    r#"{"scriptpubkey":"0020aa","scriptpubkey_asm":"OP_0","scriptpubkey_type":"v0_p2wsh","value":1}"#
        .to_string()
}

fn ordinary_output() -> String {
    r#"{"scriptpubkey":"0014aa","scriptpubkey_asm":"OP_0","scriptpubkey_type":"v0_p2wpkh","scriptpubkey_address":"bc1qordinary","value":9}"#
        .to_string()
}

fn empty_type_output() -> String {
    r#"{"scriptpubkey":"0020aa","scriptpubkey_asm":"OP_0","scriptpubkey_type":"","value":1}"#
        .to_string()
}

fn missing_type_output() -> String {
    r#"{"scriptpubkey":"0020aa","scriptpubkey_asm":"OP_0","value":1}"#.to_string()
}

fn null_type_output() -> String {
    r#"{"scriptpubkey":"0020aa","scriptpubkey_asm":"OP_0","scriptpubkey_type":null,"value":1}"#
        .to_string()
}

fn v0_p2wsh_input() -> String {
    format!(
        r#"{{"txid":"{PREV}","vout":0,"prevout":{{"scriptpubkey":"0020aa","scriptpubkey_asm":"OP_0","scriptpubkey_type":"v0_p2wsh","value":5}},"scriptsig":"deadbeef","scriptsig_asm":"OP_CHECKSIG","is_coinbase":false,"sequence":1}}"#
    )
}

fn address_page_with_v0_p2wsh_output() -> String {
    let stats = r#"{"address":"3L4YUynB4X44rJBY9CmiLMN8Wjti49JCYB","chain_stats":{"tx_count":1,"funded_txo_count":1,"spent_txo_count":0,"funded_txo_sum":1000,"spent_txo_sum":0},"mempool_stats":{"tx_count":0,"funded_txo_count":0,"spent_txo_count":0,"funded_txo_sum":0,"spent_txo_sum":0}}"#;
    let txs = r#"[{"txid":"wshone","version":1,"locktime":0,"vin":[{"txid":"bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb","vout":0,"prevout":null,"scriptsig":"","scriptsig_asm":"","is_coinbase":false,"sequence":1}],"vout":[{"scriptpubkey":"0020aa","scriptpubkey_asm":"OP_0","scriptpubkey_type":"v0_p2wsh","value":1}],"size":10,"weight":400,"sigops":0,"fee":1}]"#;
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
fn v0_p2wsh_output_paints_output_script_type_v0_p2wsh_as_its_own_line() {
    let markup = paint(&tx(&empty_input(), &v0_p2wsh_output()));
    assert!(
        markup.lines().any(|line| line == LINE),
        "no line equals output script type v0_p2wsh: {markup}"
    );
    assert_eq!(
        markup.lines().filter(|line| *line == LINE).count(),
        1,
        "scriptpubkey_type v0_p2wsh did not paint the exact line output script type v0_p2wsh once: {markup}"
    );
    assert!(
        markup
            .lines()
            .any(|line| line == LINE && !line.contains('<')),
        "output script type v0_p2wsh was inside a span: {markup}"
    );
    assert!(
        !markup
            .lines()
            .any(|line| line == "V0_P2WSH" || line == "v0_p2wsh"),
        "a bare type token was painted: {markup}"
    );
    assert!(
        markup.contains("<span class=\"scriptpubkey-type\">v0_p2wsh</span>"),
        "the scriptpubkey-type span was dropped: {markup}"
    );
    assert!(
        markup.contains("<span class=\"output-type\">V0_P2WSH</span>"),
        "the uppercase v0_p2wsh span was dropped: {markup}"
    );
    assert_eq!(
        markup
            .lines()
            .filter(|line| *line == "Type V0_P2WSH")
            .count(),
        1,
        "Type V0_P2WSH was dropped: {markup}"
    );

    let other = paint(&tx(&empty_input(), &ordinary_output()));
    assert_eq!(
        other.lines().filter(|line| *line == LINE).count(),
        0,
        "a non-v0_p2wsh output painted {LINE}: {other}"
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

    let input_v0_p2wsh = paint(&tx(&v0_p2wsh_input(), &ordinary_output()));
    assert_eq!(
        input_v0_p2wsh.lines().filter(|line| *line == LINE).count(),
        0,
        "output script type v0_p2wsh was painted on the input path: {input_v0_p2wsh}"
    );

    let address = address_page_with_v0_p2wsh_output();
    assert_eq!(
        address.lines().filter(|line| *line == LINE).count(),
        0,
        "the address page painted {LINE}: {address}"
    );
    assert!(
        address.contains("<span class=\"scriptpubkey-type\">v0_p2wsh</span>"),
        "the address page dropped the scriptpubkey-type span: {address}"
    );

    let blank = transaction_row_markup("");
    assert_eq!(
        blank.lines().filter(|line| *line == LINE).count(),
        0,
        "an empty body painted {LINE}: {blank}"
    );
}
