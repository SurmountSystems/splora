//! Whole line `Type OP_RETURN` on a transaction output from `GET /tx/:txid`.
//!
//! GPUI `transaction_screen_paints_op_return_label` requires one line whose
//! entire text is `Type OP_RETURN` when `vout.scriptpubkey_type` is
//! `op_return`. The scriptpubkey-type span, the OP_RETURN span, and the word
//! data stay. An input whose prevout type is `op_return` does not gain this
//! line. A p2pk output and the address page omit the line. A span that
//! contains the words is not that line.

use splora_web::{address_rows_markup, transaction_requests, transaction_row_markup};

const TXID: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
const PREV: &str = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
const LINE: &str = "Type OP_RETURN";

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

fn op_return_output() -> String {
    r#"{"scriptpubkey":"6a","scriptpubkey_asm":"OP_RETURN","scriptpubkey_type":"op_return","value":0}"#
        .to_string()
}

fn p2pk_output() -> String {
    r#"{"scriptpubkey":"0014aa","scriptpubkey_asm":"OP_0","scriptpubkey_type":"p2pk","value":1}"#
        .to_string()
}

fn ordinary_output() -> String {
    r#"{"scriptpubkey":"0014aa","scriptpubkey_asm":"OP_0","scriptpubkey_type":"v0_p2wpkh","scriptpubkey_address":"bc1qordinary","value":9}"#
        .to_string()
}

fn op_return_input() -> String {
    format!(
        r#"{{"txid":"{PREV}","vout":0,"prevout":{{"scriptpubkey":"6a","scriptpubkey_asm":"OP_RETURN","scriptpubkey_type":"op_return","value":0}},"scriptsig":"deadbeef","scriptsig_asm":"OP_CHECKSIG","is_coinbase":false,"sequence":1}}"#
    )
}

fn address_page_with_op_return_output() -> String {
    let stats = r#"{"address":"3L4YUynB4X44rJBY9CmiLMN8Wjti49JCYB","chain_stats":{"tx_count":1,"funded_txo_count":1,"spent_txo_count":0,"funded_txo_sum":1000,"spent_txo_sum":0},"mempool_stats":{"tx_count":0,"funded_txo_count":0,"spent_txo_count":0,"funded_txo_sum":0,"spent_txo_sum":0}}"#;
    let txs = r#"[{"txid":"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa","version":1,"locktime":0,"vin":[{"txid":"bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb","vout":0,"prevout":null,"scriptsig":"","scriptsig_asm":"","is_coinbase":false,"sequence":1}],"vout":[{"scriptpubkey":"6a","scriptpubkey_asm":"OP_RETURN","scriptpubkey_type":"op_return","value":0}],"size":10,"weight":40,"sigops":0,"fee":1}]"#;
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
fn op_return_output_paints_type_op_return_as_its_own_line() {
    let markup = paint(&tx(&empty_input(), &op_return_output()));
    assert!(
        markup.lines().any(|line| line == LINE),
        "no line equals Type OP_RETURN: {markup}"
    );
    assert_eq!(
        markup.lines().filter(|line| *line == LINE).count(),
        1,
        "scriptpubkey_type op_return did not paint the exact line Type OP_RETURN once: {markup}"
    );
    assert!(
        markup
            .lines()
            .any(|line| line == LINE && !line.contains('<')),
        "Type OP_RETURN was inside a span: {markup}"
    );
    assert!(
        markup.contains("<span class=\"scriptpubkey-type\">op_return</span>"),
        "the scriptpubkey-type span was dropped: {markup}"
    );
    assert!(
        markup.contains("<span class=\"op-return\">OP_RETURN</span>"),
        "the OP_RETURN span was dropped: {markup}"
    );
    assert!(
        markup.contains("<span class=\"op-return-data-label\">data</span>"),
        "the data span was dropped: {markup}"
    );
    assert_eq!(
        markup.lines().filter(|line| *line == "data").count(),
        1,
        "the word data was dropped or repeated: {markup}"
    );
    assert!(
        markup.contains("<span class=\"output-type\">OP_RETURN</span>"),
        "the output-type span was dropped: {markup}"
    );

    let input_op_return = paint(&tx(&op_return_input(), &ordinary_output()));
    assert_eq!(
        input_op_return.lines().filter(|line| *line == LINE).count(),
        0,
        "Type OP_RETURN was painted on the input path: {input_op_return}"
    );

    let p2pk = paint(&tx(&empty_input(), &p2pk_output()));
    assert_eq!(
        p2pk.lines().filter(|line| *line == LINE).count(),
        0,
        "a p2pk output painted {LINE}: {p2pk}"
    );
    assert_eq!(
        p2pk.lines().filter(|line| *line == "Type P2PK").count(),
        1,
        "Type P2PK was dropped or repeated: {p2pk}"
    );

    let address = address_page_with_op_return_output();
    assert_eq!(
        address.lines().filter(|line| *line == LINE).count(),
        0,
        "the address page painted {LINE}: {address}"
    );
    assert!(
        address.contains("<span class=\"scriptpubkey-type\">op_return</span>"),
        "the address page dropped the scriptpubkey-type span: {address}"
    );
    assert!(
        address.contains("<span class=\"op-return\">OP_RETURN</span>"),
        "the address page dropped the OP_RETURN span: {address}"
    );
    assert_eq!(
        address.lines().filter(|line| *line == "data").count(),
        1,
        "the address page dropped the word data: {address}"
    );
}
