//! Previous output type on a fee prevout from `GET /tx/:txid`.
//!
//! A prevout whose `scriptpubkey_type` is `fee` paints the exact line
//! `Previous output type FEE`. An ordinary prevout, a fee output, an empty
//! type, a missing key, and JSON null omit that line. The existing
//! previous-output-type span stays. The path has no query string.

use splora_web::{transaction_requests, transaction_row_markup};

const TXID: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
const PREV: &str = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
const LINE: &str = "Previous output type FEE";

fn tx(vin: &str, vout: &str) -> String {
    format!(
        r#"{{"txid":"{TXID}","version":1,"locktime":0,"vin":[{vin}],"vout":[{vout}],"size":10,"weight":40,"sigops":0,"fee":1,"status":{{"confirmed":true}}}}"#
    )
}

fn input(prevout: &str) -> String {
    format!(
        r#"{{"txid":"{PREV}","vout":0,"prevout":{prevout},"scriptsig":"","scriptsig_asm":"","is_coinbase":false,"sequence":1}}"#
    )
}

fn prevout(type_field: &str) -> String {
    format!(r#"{{"scriptpubkey":"","scriptpubkey_asm":"",{type_field}"value":5}}"#)
}

fn output(kind: &str) -> String {
    format!(
        r#"{{"scriptpubkey":"0014aa","scriptpubkey_asm":"OP_0","scriptpubkey_type":"{kind}","scriptpubkey_address":"bc1qordinary","value":9}}"#
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
fn fee_prevout_paints_previous_output_type_fee() {
    let markup = paint(&tx(
        &input(&prevout(r#""scriptpubkey_type":"fee","#)),
        &output("v0_p2wpkh"),
    ));
    assert_eq!(markup.matches(LINE).count(), 1, "{markup}");
    assert_eq!(
        markup
            .matches("<span class=\"previous-output-type\">FEE</span>")
            .count(),
        1,
        "{markup}"
    );

    let ordinary = paint(&tx(
        &input(&prevout(r#""scriptpubkey_type":"v0_p2wpkh","#)),
        &output("v0_p2wpkh"),
    ));
    assert!(!ordinary.contains(LINE), "{ordinary}");

    let fee_output = paint(&tx(
        &input("null"),
        r#"{"scriptpubkey":"","scriptpubkey_asm":"","scriptpubkey_type":"fee","value":0}"#,
    ));
    assert!(!fee_output.contains(LINE), "{fee_output}");

    let blank = paint(&tx(
        &input(&prevout(r#""scriptpubkey_type":"","#)),
        &output("v0_p2wpkh"),
    ));
    assert!(!blank.contains(LINE), "{blank}");

    let missing = paint(&tx(&input(&prevout("")), &output("v0_p2wpkh")));
    assert!(!missing.contains(LINE), "{missing}");

    let null = paint(&tx(
        &input(&prevout(r#""scriptpubkey_type":null,"#)),
        &output("v0_p2wpkh"),
    ));
    assert!(!null.contains(LINE), "{null}");
}
