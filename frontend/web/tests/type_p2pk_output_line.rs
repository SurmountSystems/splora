//! Whole line `Type P2PK` on a transaction output from `GET /tx/:txid`.
//!
//! GPUI `transaction_screen_paints_p2pk_label` requires one line whose entire
//! text is `Type P2PK` when `vout.scriptpubkey_type` is `p2pk`. The Type span
//! and the uppercase value span stay. An input whose prevout type is `p2pk`
//! does not gain this line. A fee output, an op_return output, a fee input,
//! and the address page omit the line. A span that contains the words is not
//! that line.

use splora_web::{address_rows_markup, transaction_requests, transaction_row_markup};

const TXID: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
const PREV: &str = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
const LINE: &str = "Type P2PK";
const TYPE_LABEL: &str = "<span class=\"output-type-label\">Type</span>";
const TYPE_VALUE: &str = "<span class=\"output-type\">P2PK</span>";

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

fn p2pk_output() -> String {
    r#"{"scriptpubkey":"0014aa","scriptpubkey_asm":"OP_0","scriptpubkey_type":"p2pk","value":1}"#
        .to_string()
}

fn ordinary_output() -> String {
    r#"{"scriptpubkey":"0014aa","scriptpubkey_asm":"OP_0","scriptpubkey_type":"v0_p2wpkh","scriptpubkey_address":"bc1qordinary","value":9}"#
        .to_string()
}

fn fee_output() -> String {
    r#"{"scriptpubkey":"","scriptpubkey_asm":"","scriptpubkey_type":"fee","value":0}"#.to_string()
}

fn op_return_output() -> String {
    r#"{"scriptpubkey":"6a","scriptpubkey_asm":"OP_RETURN","scriptpubkey_type":"op_return","value":0}"#
        .to_string()
}

fn p2pk_input() -> String {
    format!(
        r#"{{"txid":"{PREV}","vout":0,"prevout":{{"scriptpubkey":"0014aa","scriptpubkey_asm":"OP_0","scriptpubkey_type":"p2pk","value":5}},"scriptsig":"deadbeef","scriptsig_asm":"OP_CHECKSIG","is_coinbase":false,"sequence":1}}"#
    )
}

fn fee_input() -> String {
    format!(
        r#"{{"txid":"{PREV}","vout":0,"prevout":{{"scriptpubkey":"","scriptpubkey_asm":"","scriptpubkey_type":"fee","value":5}},"scriptsig":"deadbeef","scriptsig_asm":"OP_CHECKSIG","is_coinbase":false,"sequence":1}}"#
    )
}

fn address_page_with_p2pk_output() -> String {
    let stats = r#"{"address":"3L4YUynB4X44rJBY9CmiLMN8Wjti49JCYB","chain_stats":{"tx_count":1,"funded_txo_count":1,"spent_txo_count":0,"funded_txo_sum":1000,"spent_txo_sum":0},"mempool_stats":{"tx_count":0,"funded_txo_count":0,"spent_txo_count":0,"funded_txo_sum":0,"spent_txo_sum":0}}"#;
    let txs = r#"[{"txid":"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa","version":1,"locktime":0,"vin":[{"txid":"bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb","vout":0,"prevout":null,"scriptsig":"","scriptsig_asm":"","is_coinbase":false,"sequence":1}],"vout":[{"scriptpubkey":"0014aa","scriptpubkey_asm":"OP_0","scriptpubkey_type":"p2pk","value":1}],"size":10,"weight":40,"sigops":0,"fee":1}]"#;
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
fn p2pk_output_paints_type_p2pk_as_its_own_line() {
    let markup = paint(&tx(&empty_input(), &p2pk_output()));
    assert!(
        markup.lines().any(|line| line == LINE),
        "scriptpubkey_type p2pk did not paint the exact line Type P2PK: {markup}"
    );
    assert_eq!(
        markup.lines().filter(|line| *line == LINE).count(),
        1,
        "scriptpubkey_type p2pk did not paint the exact line Type P2PK once: {markup}"
    );
    assert!(
        markup
            .lines()
            .any(|line| line == LINE && !line.contains('<')),
        "Type P2PK was inside a span: {markup}"
    );
    assert!(
        markup.contains(TYPE_LABEL),
        "the Type span was dropped: {markup}"
    );
    assert!(
        markup.contains(TYPE_VALUE),
        "the uppercase p2pk span was dropped: {markup}"
    );
    assert!(
        markup.contains("<span class=\"scriptpubkey-type\">p2pk</span>"),
        "the scriptpubkey-type span was dropped: {markup}"
    );

    let other = paint(&tx(&empty_input(), &ordinary_output()));
    assert!(
        !other.lines().any(|line| line == LINE),
        "a non-p2pk output painted {LINE}: {other}"
    );

    let fee = paint(&tx(&empty_input(), &fee_output()));
    assert!(
        !fee.lines().any(|line| line == LINE),
        "a fee output painted {LINE}: {fee}"
    );
    assert_eq!(
        fee.lines().filter(|line| *line == "Type FEE").count(),
        1,
        "Type FEE was dropped or repeated: {fee}"
    );

    let op_return = paint(&tx(&empty_input(), &op_return_output()));
    assert!(
        !op_return.lines().any(|line| line == LINE),
        "an op_return output painted {LINE}: {op_return}"
    );
    assert!(
        op_return.contains("<span class=\"output-type\">OP_RETURN</span>"),
        "the op_return type span was dropped: {op_return}"
    );

    let input_p2pk = paint(&tx(&p2pk_input(), &ordinary_output()));
    assert!(
        !input_p2pk.lines().any(|line| line == LINE),
        "Type P2PK was painted on the input path: {input_p2pk}"
    );
    assert!(
        input_p2pk.contains("<span class=\"previous-output-type\">P2PK</span>"),
        "the previous-output span was rewritten: {input_p2pk}"
    );

    let input_fee = paint(&tx(&fee_input(), &ordinary_output()));
    assert!(
        !input_fee.lines().any(|line| line == LINE),
        "a fee input painted {LINE}: {input_fee}"
    );
    assert!(
        input_fee.contains("Previous output type FEE"),
        "the previous-output span was rewritten: {input_fee}"
    );

    let address = address_page_with_p2pk_output();
    assert!(
        !address.lines().any(|line| line == LINE),
        "the address page painted {LINE}: {address}"
    );
}
