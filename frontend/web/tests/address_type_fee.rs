//! Address-type caption on a fee output from `GET /tx/:txid`.
//!
//! Angular `address-type.component.html` shows the word `fee` when the
//! type is `fee`. That caption is not the scriptpubkey-type span, not
//! `Transaction fee`, and not `Type FEE`. An ordinary output does not
//! gain it. An input whose prevout type is `fee` does not gain it.
//! The load is `GET /tx/:txid` with no query string.

use splora_web::{transaction_requests, transaction_row_markup};

const TXID: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
const PREV: &str = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
const CAPTION: &str = "<span class=\"address-fee\">fee</span>";

fn tx(vin: &str, vout: &str) -> String {
    format!(
        r#"{{"txid":"{TXID}","version":1,"locktime":0,"vin":[{vin}],"vout":[{vout}],"size":10,"weight":40,"sigops":0,"fee":50,"status":{{"confirmed":true,"block_height":4}}}}"#
    )
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
        r#"{{"txid":"{PREV}","vout":0,"prevout":{{"scriptpubkey":"","scriptpubkey_asm":"","scriptpubkey_type":"fee","value":5}},"scriptsig":"deadbeef","scriptsig_asm":"OP_CHECKSIG","witness":["abcd"],"is_coinbase":false,"sequence":1}}"#
    )
}

fn empty_input() -> String {
    r#"{"txid":"prev","vout":0,"prevout":null,"scriptsig":"","scriptsig_asm":"","is_coinbase":false,"sequence":1}"#
        .to_string()
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

fn assert_one(markup: &str, expected: &str) {
    assert_eq!(
        markup.matches(expected).count(),
        1,
        "expected one {expected} in {markup}"
    );
}

#[test]
fn fee_output_paints_the_address_fee_caption() {
    let markup = paint(&tx(&empty_input(), &fee_output()));
    assert_one(&markup, CAPTION);
    assert_one(
        &markup,
        "<span class=\"output-type\">Transaction fee</span>",
    );
    assert_one(&markup, "<span class=\"scriptpubkey-type\">fee</span>");
    assert!(
        !markup.contains("<span class=\"output-type\">FEE</span>"),
        "the Type cell changed to FEE: {markup}"
    );

    let ordinary = paint(&tx(&empty_input(), &ordinary_output()));
    assert!(
        !ordinary.contains(CAPTION),
        "an ordinary output painted the fee caption: {ordinary}"
    );

    let input_only = paint(&tx(&fee_input(), &ordinary_output()));
    assert!(
        !input_only.contains(CAPTION),
        "a fee input painted the fee caption: {input_only}"
    );
    assert_one(
        &input_only,
        "<span class=\"previous-output-type\">FEE</span>",
    );
}
