//! Output script type `empty` on a transaction output from `GET /tx/:txid`.
//!
//! The Angular scriptpubkey type pipe uppercases the raw type. An output
//! whose `scriptpubkey_type` is `empty` shows `EMPTY` once in the Type
//! cell. That cell is not the lowercase word `empty`. The lowercase
//! `scriptpubkey-type` span stays the indexer spelling. A type string of
//! `""`, a missing key, and JSON null do not paint `EMPTY`. Those three
//! are not the type value `empty`. Script hex, script asm, scriptsig,
//! witness, leaf version, and the inner tapscript stay off the page.
//! There is no address-type span and no second Type row. The block
//! sentence `Why is this block empty?` is a different string.

use splora_web::{transaction_requests, transaction_row_markup};

const TXID: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
const PREV: &str = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
const SCRIPT_HEX: &str = "NILPUBHEXDEAD";
const SCRIPT_ASM: &str = "NILASMDEAD";
const SCRIPT_SIG: &str = "NILSIGHEXDEAD";
const SCRIPT_SIG_ASM: &str = "NILSIGASMDEAD";
const WITNESS: &str = "NILWITDEAD";
const LEAF_VERSION: &str = "NILLEAFDEAD";
const INNER_TAPSCRIPT: &str = "NILTAPSCRIPTDEAD";
const TYPE_SPANS: &str =
    "<span class=\"output-type-label\">Type</span><span class=\"output-type\">EMPTY</span>";
const TYPE_LABEL: &str = "<span class=\"output-type-label\">Type</span>";
const BLOCK_SENTENCE: &str = "Why is this block empty?";

fn tx(vin: &str, vout: &str) -> String {
    tx_status(vin, vout, None)
}

fn tx_status(vin: &str, vout: &str, status: Option<&str>) -> String {
    let status_field = match status {
        Some(status) => format!(r#","status":{status}"#),
        None => String::new(),
    };
    format!(
        r#"{{"txid":"{TXID}","version":1,"locktime":0,"vin":[{vin}],"vout":[{vout}],"size":10,"weight":40,"sigops":0,"fee":1{status_field}}}"#
    )
}

fn input(prevout: &str) -> String {
    format!(
        r#"{{"txid":"{PREV}","vout":0,"prevout":{prevout},"scriptsig":"{SCRIPT_SIG}","scriptsig_asm":"{SCRIPT_SIG_ASM}","witness":["{WITNESS}","{LEAF_VERSION}"],"is_coinbase":false,"sequence":1,"inner_witnessscript_asm":"{INNER_TAPSCRIPT}"}}"#
    )
}

fn prevout(kind: &str) -> String {
    format!(
        r#"{{"scriptpubkey":"{SCRIPT_HEX}","scriptpubkey_asm":"{SCRIPT_ASM}","scriptpubkey_type":"{kind}"}}"#
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

fn assert_sentinels_stay_off(markup: &str) {
    assert_absent(markup, SCRIPT_HEX);
    assert_absent(markup, SCRIPT_ASM);
    assert_absent(markup, SCRIPT_SIG);
    assert_absent(markup, SCRIPT_SIG_ASM);
    assert_absent(markup, WITNESS);
    assert_absent(markup, LEAF_VERSION);
    assert_absent(markup, INNER_TAPSCRIPT);
    assert_absent(markup, "address-type");
    assert_absent(markup, BLOCK_SENTENCE);
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
fn empty_output_shows_uppercase_type_once() {
    let markup = paint(&tx_status(
        &input("null"),
        &output("empty"),
        Some(r#"{"confirmed":true}"#),
    ));
    assert_eq!(
        markup.matches("EMPTY").count(),
        2,
        "expected the Type cell EMPTY and the line Type EMPTY in {markup}"
    );
    assert_eq!(markup.matches("Type EMPTY").count(), 1, "{markup}");
    assert_one(&markup, TYPE_SPANS);
    assert_eq!(
        markup.matches(TYPE_LABEL).count(),
        1,
        "a second Type row leaked into {markup}"
    );
    assert_eq!(
        class_span_values(&markup, "output-type"),
        ["EMPTY"],
        "the Type cell must be EMPTY, not the lowercase word empty: {markup}"
    );
    assert!(
        !class_span_values(&markup, "output-type")
            .iter()
            .any(|value| *value == "empty"),
        "lowercase empty leaked as the output type: {markup}"
    );
    assert_absent(&markup, "<span class=\"output-type\">empty</span>");
    assert_one(&markup, "<span class=\"scriptpubkey-type\">empty</span>");
    assert_absent(&markup, "<span class=\"scriptpubkey-type\">EMPTY</span>");
    assert_eq!(
        markup.matches("empty").count(),
        2,
        "lowercase empty must stay once in the scriptpubkey-type span and once on the line output script type empty: {markup}"
    );
    assert!(
        markup
            .lines()
            .any(|line| line == "output script type empty"),
        "output script type empty was dropped: {markup}"
    );
    assert!(
        !markup
            .lines()
            .any(|line| line == "empty" || line == "EMPTY"),
        "a bare type token was painted: {markup}"
    );
    assert_one(&markup, "<td class=\"confirmed\">true</td>");
    assert_absent(&markup, "<td class=\"confirmed\">TRUE</td>");
    assert_absent(&markup, "<td class=\"confirmed\">True</td>");
    assert_absent(&markup, "<td class=\"confirmed\">FALSE</td>");
    assert_absent(&markup, "<td class=\"confirmed\">False</td>");
    assert_absent(&markup, "Transaction fee");
    assert_absent(&markup, "P2PK");
    assert_absent(&markup, "OP_RETURN");
    assert_absent(&markup, "ANCHOR");
    assert_absent(&markup, "MULTISIG");
    assert_absent(&markup, "Previous output script");
    assert_absent(&markup, "Show all");
    assert_absent(&markup, "Show more");
    assert_sentinels_stay_off(&markup);
}

#[test]
fn blank_type_does_not_paint_empty() {
    let markup = paint(&tx(&input("null"), &output("")));
    assert_one(&markup, "<h2>Inputs & Outputs</h2>");
    assert_absent(&markup, "EMPTY");
    assert_absent(&markup, "empty");
    assert_absent(&markup, TYPE_LABEL);
    assert_absent(&markup, "class=\"output-type\"");
    assert_sentinels_stay_off(&markup);
}

#[test]
fn missing_scriptpubkey_type_does_not_paint_empty() {
    let vout =
        format!(r#"{{"scriptpubkey":"{SCRIPT_HEX}","scriptpubkey_asm":"{SCRIPT_ASM}","value":1}}"#);
    let markup = paint(&tx(&input("null"), &vout));
    assert_absent(&markup, "EMPTY");
    assert_absent(&markup, "empty");
    assert_absent(&markup, TYPE_LABEL);
    assert_absent(&markup, "<h2>Inputs & Outputs</h2>");
    assert_sentinels_stay_off(&markup);
}

#[test]
fn null_scriptpubkey_type_does_not_paint_empty() {
    let vout = format!(
        r#"{{"scriptpubkey":"{SCRIPT_HEX}","scriptpubkey_asm":"{SCRIPT_ASM}","scriptpubkey_type":null,"value":1}}"#
    );
    let markup = paint(&tx(&input("null"), &vout));
    assert_absent(&markup, "EMPTY");
    assert_absent(&markup, "empty");
    assert_absent(&markup, TYPE_LABEL);
    assert_absent(&markup, "<h2>Inputs & Outputs</h2>");
    assert_sentinels_stay_off(&markup);
}

#[test]
fn known_type_cells_stay_beside_empty() {
    let vout = [
        output("fee"),
        output("p2pk"),
        output("op_return"),
        output("anchor"),
        output("empty"),
    ]
    .join(",");
    let markup = paint(&tx(&input(&prevout("v0_p2wpkh")), &vout));
    assert_one(
        &markup,
        "<span class=\"output-type\">Transaction fee</span>",
    );
    assert_absent(&markup, "<span class=\"output-type\">FEE</span>");
    assert_one(&markup, "<span class=\"output-type\">P2PK</span>");
    assert_one(&markup, "<span class=\"output-type\">OP_RETURN</span>");
    assert_one(&markup, "<span class=\"op-return\">OP_RETURN</span>");
    assert_one(&markup, "<span class=\"output-type\">ANCHOR</span>");
    assert_one(&markup, "<span class=\"output-type\">EMPTY</span>");
    assert_one(&markup, "<span class=\"scriptpubkey-type\">empty</span>");
    assert_absent(&markup, "<span class=\"scriptpubkey-type\">EMPTY</span>");
    assert_absent(&markup, "<span class=\"output-type\">empty</span>");
    assert_eq!(
        markup.matches(TYPE_LABEL).count(),
        5,
        "Type row count changed: {markup}"
    );
    assert_one(
        &markup,
        "<span class=\"previous-output-script-label\">Previous output script</span>",
    );
    assert_one(&markup, "<td class=\"confirmed\">false</td>");
    assert_absent(&markup, "<td class=\"confirmed\">FALSE</td>");
    assert_absent(&markup, "<td class=\"confirmed\">False</td>");
    assert_absent(&markup, "Show all");
    assert_absent(&markup, "Show more");
    assert_absent(&markup, BLOCK_SENTENCE);
    assert_sentinels_stay_off(&markup);
}
