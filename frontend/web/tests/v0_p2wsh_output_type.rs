//! Output script type `v0_p2wsh` on a transaction output from `GET /tx/:txid`.
//!
//! The Angular scriptpubkey type pipe uppercases the raw type. An output
//! whose `scriptpubkey_type` is `v0_p2wsh` shows `V0_P2WSH` once in the Type
//! cell. That cell is not the lowercase word `v0_p2wsh`. The lowercase
//! `scriptpubkey-type` span stays the indexer spelling. An empty type, a
//! missing key, and JSON null do not paint `V0_P2WSH`. Script hex, script
//! asm, scriptsig, witness, leaf version, and the inner tapscript stay off
//! the page. There is no address-type span and no second Type row.

use splora_web::{transaction_requests, transaction_row_markup};

const TXID: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
const PREV: &str = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
const SCRIPT_HEX: &str = "V0P2WSHPUBHEXDEAD";
const SCRIPT_ASM: &str = "V0P2WSHASMDEAD";
const SCRIPT_SIG: &str = "V0P2WSHSIGHEXDEAD";
const SCRIPT_SIG_ASM: &str = "V0P2WSHSIGASMDEAD";
const WITNESS: &str = "V0P2WSHWITDEAD";
const LEAF_VERSION: &str = "V0P2WSHLEAFDEAD";
const INNER_TAPSCRIPT: &str = "V0P2WSHTAPSCRIPTDEAD";
const TYPE_SPANS: &str =
    "<span class=\"output-type-label\">Type</span><span class=\"output-type\">V0_P2WSH</span>";
const TYPE_LABEL: &str = "<span class=\"output-type-label\">Type</span>";

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
fn v0_p2wsh_output_shows_uppercase_type_once() {
    let markup = paint(&tx_status(
        &input("null"),
        &output("v0_p2wsh"),
        Some(r#"{"confirmed":true}"#),
    ));
    assert_eq!(
        markup.matches("V0_P2WSH").count(),
        1,
        "expected V0_P2WSH once in {markup}"
    );
    assert_one(&markup, TYPE_SPANS);
    assert_eq!(
        markup.matches(TYPE_LABEL).count(),
        1,
        "a second Type row leaked into {markup}"
    );
    assert_eq!(
        class_span_values(&markup, "output-type"),
        ["V0_P2WSH"],
        "the Type cell must be V0_P2WSH, not the lowercase word v0_p2wsh: {markup}"
    );
    assert!(
        !class_span_values(&markup, "output-type")
            .iter()
            .any(|value| *value == "v0_p2wsh"),
        "lowercase v0_p2wsh leaked as the output type: {markup}"
    );
    assert_absent(&markup, "<span class=\"output-type\">v0_p2wsh</span>");
    assert_one(&markup, "<span class=\"scriptpubkey-type\">v0_p2wsh</span>");
    assert_absent(&markup, "<span class=\"scriptpubkey-type\">V0_P2WSH</span>");
    assert_one(&markup, "<td class=\"confirmed\">true</td>");
    assert_absent(&markup, "<td class=\"confirmed\">TRUE</td>");
    assert_absent(&markup, "<td class=\"confirmed\">True</td>");
    assert_absent(&markup, "<td class=\"confirmed\">FALSE</td>");
    assert_absent(&markup, "<td class=\"confirmed\">False</td>");
    assert_absent(&markup, "Transaction fee");
    assert_absent(&markup, "P2PK");
    assert_absent(&markup, "OP_RETURN");
    assert_absent(&markup, "ANCHOR");
    assert_absent(&markup, "Previous output script");
    assert_absent(&markup, "Show all");
    assert_absent(&markup, "Show more");
    assert_sentinels_stay_off(&markup);
}

#[test]
fn empty_type_does_not_paint_v0_p2wsh() {
    let markup = paint(&tx(&input("null"), &output("")));
    assert_one(&markup, "<h2>Inputs & Outputs</h2>");
    assert_absent(&markup, "V0_P2WSH");
    assert_absent(&markup, "v0_p2wsh");
    assert_absent(&markup, TYPE_LABEL);
    assert_absent(&markup, "class=\"output-type\"");
    assert_sentinels_stay_off(&markup);
}

#[test]
fn missing_scriptpubkey_type_does_not_paint_v0_p2wsh() {
    let vout =
        format!(r#"{{"scriptpubkey":"{SCRIPT_HEX}","scriptpubkey_asm":"{SCRIPT_ASM}","value":1}}"#);
    let markup = paint(&tx(&input("null"), &vout));
    assert_absent(&markup, "V0_P2WSH");
    assert_absent(&markup, "v0_p2wsh");
    assert_absent(&markup, TYPE_LABEL);
    assert_absent(&markup, "<h2>Inputs & Outputs</h2>");
    assert_sentinels_stay_off(&markup);
}

#[test]
fn null_scriptpubkey_type_does_not_paint_v0_p2wsh() {
    let vout = format!(
        r#"{{"scriptpubkey":"{SCRIPT_HEX}","scriptpubkey_asm":"{SCRIPT_ASM}","scriptpubkey_type":null,"value":1}}"#
    );
    let markup = paint(&tx(&input("null"), &vout));
    assert_absent(&markup, "V0_P2WSH");
    assert_absent(&markup, "v0_p2wsh");
    assert_absent(&markup, TYPE_LABEL);
    assert_absent(&markup, "<h2>Inputs & Outputs</h2>");
    assert_sentinels_stay_off(&markup);
}

#[test]
fn known_type_cells_stay_beside_v0_p2wsh() {
    let vout = [
        output("fee"),
        output("p2pk"),
        output("op_return"),
        output("anchor"),
        output("v0_p2wsh"),
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
    assert_one(&markup, "<span class=\"output-type\">V0_P2WSH</span>");
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
    assert_sentinels_stay_off(&markup);
}
