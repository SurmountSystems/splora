//! Whole line `Type FEE` on a fee output from `GET /tx/:txid`.
//!
//! GPUI `transaction_screen_paints_transaction_fee_label` requires one line
//! whose entire text is `Type FEE` when `vout.scriptpubkey_type` is `fee`.
//! The Transaction fee cell stays. A different script type omits the line.
//! A fee prevout stays `Previous output type FEE` and does not gain this line.
//! A span that contains the words is not that line.

use splora_web::{transaction_requests, transaction_row_markup};

const TXID: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
const PREV: &str = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
const LINE: &str = "Type FEE";
const FEE_CELL: &str = "<span class=\"output-type\">Transaction fee</span>";

fn tx(vin: &str, vout: &str) -> String {
    format!(
        r#"{{"txid":"{TXID}","version":1,"locktime":0,"vin":[{vin}],"vout":[{vout}],"size":10,"weight":40,"sigops":0,"fee":50,"status":{{"confirmed":true}}}}"#
    )
}

fn empty_input() -> String {
    r#"{"txid":"prev","vout":0,"prevout":null,"scriptsig":"","scriptsig_asm":"","is_coinbase":false,"sequence":1}"#
        .to_string()
}

fn fee_output() -> String {
    r#"{"scriptpubkey":"","scriptpubkey_asm":"","scriptpubkey_type":"fee","value":0}"#.to_string()
}

fn ordinary_output() -> String {
    r#"{"scriptpubkey":"0014aa","scriptpubkey_asm":"OP_0","scriptpubkey_type":"v0_p2wpkh","scriptpubkey_address":"bc1qordinary","value":9}"#
        .to_string()
}

fn fee_input() -> String {
    format!(
        r#"{{"txid":"{PREV}","vout":0,"prevout":{{"scriptpubkey":"","scriptpubkey_asm":"","scriptpubkey_type":"fee","value":5}},"scriptsig":"deadbeef","scriptsig_asm":"OP_CHECKSIG","is_coinbase":false,"sequence":1}}"#
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
fn fee_output_paints_type_fee_as_its_own_line() {
    let markup = paint(&tx(&empty_input(), &fee_output()));
    assert_eq!(
        markup.lines().filter(|line| *line == LINE).count(),
        1,
        "scriptpubkey_type fee did not paint the exact line Type FEE: {markup}"
    );
    assert!(
        markup
            .lines()
            .any(|line| line == LINE && !line.contains('<')),
        "Type FEE was inside a span: {markup}"
    );
    assert_eq!(markup.matches(FEE_CELL).count(), 1, "{markup}");

    let ordinary = paint(&tx(&empty_input(), &ordinary_output()));
    assert!(
        !ordinary.lines().any(|line| line == LINE),
        "a non-fee output painted {LINE}: {ordinary}"
    );

    let input_only = paint(&tx(&fee_input(), &ordinary_output()));
    assert!(
        !input_only.lines().any(|line| line == LINE),
        "a fee input painted {LINE}: {input_only}"
    );
    assert!(
        input_only.contains("Previous output type FEE"),
        "the previous-output span was rewritten: {input_only}"
    );
}
