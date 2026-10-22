//! Output script type line on a fee output from `GET /tx/:txid`.
//!
//! A `vout` whose `scriptpubkey_type` is `fee` paints the exact line
//! `output script type fee`. An ordinary output and a fee prevout omit
//! that line. The scriptpubkey-type span stays `fee`. The path has no
//! query string.

use splora_web::{transaction_requests, transaction_row_markup};

const TXID: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
const PREV: &str = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
const LINE: &str = "output script type fee";

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
fn fee_output_paints_output_script_type_fee() {
    let markup = paint(&tx(&empty_input(), &fee_output()));
    assert_eq!(markup.matches(LINE).count(), 1, "{markup}");
    assert_eq!(
        markup
            .matches("<span class=\"scriptpubkey-type\">fee</span>")
            .count(),
        1,
        "{markup}"
    );

    let ordinary = paint(&tx(&empty_input(), &ordinary_output()));
    assert!(!ordinary.contains(LINE), "{ordinary}");

    let input_only = paint(&tx(&fee_input(), &ordinary_output()));
    assert!(!input_only.contains(LINE), "{input_only}");
}
