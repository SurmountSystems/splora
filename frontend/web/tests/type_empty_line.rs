//! Output script type `empty` from `GET /tx/:txid`.
//!
//! A nonempty `vout.scriptpubkey_type` of `empty` paints one markup line
//! whose entire text is `Type EMPTY`, and one markup line whose entire text
//! is `output script type empty`. An empty string, a missing key, and JSON
//! null omit those lines. The Type cell stays `EMPTY`. The path has no query
//! string. A span that contains the words is not that line. The bare lines
//! `empty` and `EMPTY` stay off.

use splora_web::{transaction_requests, transaction_row_markup};

const TXID: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
const PREV: &str = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
const LINE: &str = "Type EMPTY";
const SCRIPT_LINE: &str = "output script type empty";
const TYPE_VALUE: &str = "<span class=\"output-type\">EMPTY</span>";
const TYPE_LABEL: &str = "<span class=\"output-type-label\">Type</span>";

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
fn empty_output_type_paints_type_empty_as_its_own_line() {
    let markup = paint(&tx(&output(r#""scriptpubkey_type":"empty","#)));
    assert_eq!(
        markup.lines().filter(|line| *line == LINE).count(),
        1,
        "scriptpubkey_type empty did not paint the exact line Type EMPTY: {markup}"
    );
    assert!(
        markup
            .lines()
            .any(|line| line == LINE && !line.contains('<')),
        "Type EMPTY was inside a span: {markup}"
    );
    assert_eq!(markup.matches(LINE).count(), 1, "{markup}");
    assert_eq!(markup.matches(TYPE_VALUE).count(), 1, "{markup}");
    assert_eq!(markup.matches(TYPE_LABEL).count(), 1, "{markup}");
    assert!(
        markup.lines().any(|line| line == SCRIPT_LINE),
        "no line equals output script type empty: {markup}"
    );
    assert_eq!(
        markup.lines().filter(|line| *line == SCRIPT_LINE).count(),
        1,
        "scriptpubkey_type empty did not paint the exact line output script type empty once: {markup}"
    );
    assert!(
        markup
            .lines()
            .any(|line| line == SCRIPT_LINE && !line.contains('<')),
        "output script type empty was inside a span: {markup}"
    );
    assert!(
        !markup
            .lines()
            .any(|line| line == "empty" || line == "EMPTY"),
        "a bare type token was painted: {markup}"
    );

    let blank = paint(&tx(&output(r#""scriptpubkey_type":"","#)));
    assert!(
        !blank.lines().any(|line| line == LINE),
        "a blank type painted {LINE}: {blank}"
    );
    assert!(!blank.contains(LINE), "{blank}");
    assert!(
        !blank.lines().any(|line| line == SCRIPT_LINE),
        "a blank type painted {SCRIPT_LINE}: {blank}"
    );

    let missing = paint(&tx(&output("")));
    assert!(
        !missing.lines().any(|line| line == LINE),
        "a missing type painted {LINE}: {missing}"
    );
    assert!(!missing.contains(LINE), "{missing}");
    assert!(
        !missing.lines().any(|line| line == SCRIPT_LINE),
        "a missing type painted {SCRIPT_LINE}: {missing}"
    );

    let null = paint(&tx(&output(r#""scriptpubkey_type":null,"#)));
    assert!(
        !null.lines().any(|line| line == LINE),
        "a JSON null type painted {LINE}: {null}"
    );
    assert!(!null.contains(LINE), "{null}");
    assert!(
        !null.lines().any(|line| line == SCRIPT_LINE),
        "a JSON null type painted {SCRIPT_LINE}: {null}"
    );
}
