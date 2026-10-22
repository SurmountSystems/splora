//! Output script type `empty` from `GET /tx/:txid`.
//!
//! A nonempty `vout.scriptpubkey_type` of `empty` paints the exact line
//! `Type EMPTY`. An empty string, a missing key, and JSON null omit that
//! line. The Type cell stays `EMPTY`. The path has no query string.

use splora_web::{transaction_requests, transaction_row_markup};

const TXID: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
const PREV: &str = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
const LINE: &str = "Type EMPTY";

fn tx(vout: &str) -> String {
    format!(
        r#"{{"txid":"{TXID}","version":1,"locktime":0,"vin":[{{"txid":"{PREV}","vout":0,"prevout":null,"scriptsig":"","scriptsig_asm":"","is_coinbase":false,"sequence":1}}],"vout":[{vout}],"size":10,"weight":40,"sigops":0,"fee":1,"status":{{"confirmed":true}}}}"#
    )
}

fn output(type_field: &str) -> String {
    format!(r#"{{"scriptpubkey":"51","scriptpubkey_asm":"OP_TRUE",{type_field}"value":1}}"#)
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
fn empty_output_type_paints_type_empty() {
    let markup = paint(&tx(&output(r#""scriptpubkey_type":"empty","#)));
    assert_eq!(markup.matches(LINE).count(), 1, "{markup}");
    assert_eq!(
        markup
            .matches("<span class=\"output-type\">EMPTY</span>")
            .count(),
        1,
        "{markup}"
    );

    let blank = paint(&tx(&output(r#""scriptpubkey_type":"","#)));
    assert!(!blank.contains(LINE), "{blank}");

    let missing = paint(&tx(&output("")));
    assert!(!missing.contains(LINE), "{missing}");

    let null = paint(&tx(&output(r#""scriptpubkey_type":null,"#)));
    assert!(!null.contains(LINE), "{null}");
}
