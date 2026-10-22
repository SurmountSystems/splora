//! Output script type `anchor` on a transaction output from `GET /tx/:txid`.
//!
//! The Angular scriptpubkey type pipe uppercases the raw type. An output
//! whose `scriptpubkey_type` is `anchor` shows `ANCHOR` once in the Type
//! cell. That cell is not the lowercase word `anchor`. The lowercase
//! `scriptpubkey-type` span stays the indexer spelling. Script hex, script
//! asm, scriptsig, and witness stay off the page.

use splora_web::{transaction_requests, transaction_row_markup};

const TXID: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
const PREV: &str = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
const SCRIPT_HEX: &str = "ANCHORPUBHEXDEAD";
const SCRIPT_ASM: &str = "ANCHORASMDEAD";
const SCRIPT_SIG: &str = "ANCHORSIGHEXDEAD";
const SCRIPT_SIG_ASM: &str = "ANCHORSIGASMDEAD";
const WITNESS: &str = "ANCHORWITDEAD";
const TYPE_SPANS: &str =
    "<span class=\"output-type-label\">Type</span><span class=\"output-type\">ANCHOR</span>";

fn tx(vin: &str, vout: &str) -> String {
    format!(
        r#"{{"txid":"{TXID}","version":1,"locktime":0,"vin":[{vin}],"vout":[{vout}],"size":10,"weight":40,"sigops":0,"fee":1}}"#
    )
}

fn input(prevout: &str) -> String {
    format!(
        r#"{{"txid":"{PREV}","vout":0,"prevout":{prevout},"scriptsig":"{SCRIPT_SIG}","scriptsig_asm":"{SCRIPT_SIG_ASM}","witness":["{WITNESS}"],"is_coinbase":false,"sequence":1}}"#
    )
}

fn output(kind: &str) -> String {
    format!(
        r#"{{"scriptpubkey":"{SCRIPT_HEX}","scriptpubkey_asm":"{SCRIPT_ASM}","scriptpubkey_type":"{kind}","value":1}}"#
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

fn assert_one(markup: &str, expected: &str) {
    assert_eq!(
        markup.matches(expected).count(),
        1,
        "expected one {expected} in {markup}"
    );
}

fn assert_absent(markup: &str, needle: &str) {
    assert!(
        !markup.contains(needle),
        "{needle} leaked into the markup: {markup}"
    );
}

fn class_span_values<'a>(markup: &'a str, class: &str) -> Vec<&'a str> {
    let open = format!("<span class=\"{class}\">");
    let close = "</span>";
    let mut rest = markup;
    let mut values = Vec::new();
    while let Some(at) = rest.find(&open) {
        let after = &rest[at + open.len()..];
        let end = after
            .find(close)
            .unwrap_or_else(|| panic!("unclosed {class} span in {markup}"));
        values.push(&after[..end]);
        rest = &after[end + close.len()..];
    }
    values
}

#[test]
fn anchor_output_shows_uppercase_type_once() {
    let markup = paint(&tx(&input("null"), &output("anchor")));
    assert_eq!(
        markup.matches("ANCHOR").count(),
        1,
        "expected ANCHOR once in {markup}"
    );
    assert_one(&markup, TYPE_SPANS);
    assert_eq!(
        class_span_values(&markup, "output-type"),
        ["ANCHOR"],
        "the Type cell must be ANCHOR, not the lowercase word anchor: {markup}"
    );
    assert!(
        !class_span_values(&markup, "output-type")
            .iter()
            .any(|value| *value == "anchor"),
        "lowercase anchor leaked as the output type: {markup}"
    );
    assert_absent(&markup, "<span class=\"output-type\">anchor</span>");
    assert_one(&markup, "<span class=\"scriptpubkey-type\">anchor</span>");
    assert_absent(&markup, SCRIPT_HEX);
    assert_absent(&markup, SCRIPT_ASM);
    assert_absent(&markup, SCRIPT_SIG);
    assert_absent(&markup, SCRIPT_SIG_ASM);
    assert_absent(&markup, WITNESS);
}
