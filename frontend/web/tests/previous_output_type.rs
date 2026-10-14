//! Previous output type on a transaction input from `GET /tx/:txid`.
//!
//! Angular paints the row name `Previous output type` and
//! `vin.prevout.scriptpubkey_type.toUpperCase()` when that prevout is
//! present. An empty type string omits the label. JSON null for `prevout`
//! omits it. A missing `prevout` key is none and still paints the transaction,
//! without this label. A missing `scriptpubkey_type` key, JSON null for that
//! key, an empty body, `not-json`, and `[]` do not parse, so the label stays
//! off. The existing lowercase `input script type` span stays. An output type
//! is not this label.
//! Script hex, script asm, scriptsig, and witness stay off the page.

use splora_web::transaction_row_markup;

const TXID: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
const PREV: &str = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
const SCRIPT_HEX: &str = "POTPUBHEXDEAD";
const SCRIPT_ASM: &str = "POTASMDEAD";
const SCRIPT_SIG: &str = "POTSIGHEXDEAD";
const SCRIPT_SIG_ASM: &str = "POTSIGASMDEAD";
const WITNESS: &str = "POTWITDEAD";
const LABEL: &str = "Previous output type";

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

fn label_spans(upper: &str) -> String {
    format!(
        "<span class=\"previous-output-type-label\">{LABEL}</span><span class=\"previous-output-type\">{upper}</span>"
    )
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
fn prevout_type_paints_the_label_and_the_uppercase_type() {
    let body = tx(&input(&prevout("v0_p2wpkh")), &output("p2pkh"));
    let markup = transaction_row_markup(&body);
    assert_one(&markup, LABEL);
    assert_one(&markup, &label_spans("V0_P2WPKH"));
    assert_one(
        &markup,
        "<span class=\"scriptpubkey-type\">input script type v0_p2wpkh</span>",
    );
    assert_one(&markup, "<span class=\"scriptpubkey-type\">p2pkh</span>");
    assert_eq!(
        class_span_values(&markup, "previous-output-type"),
        ["V0_P2WPKH"],
        "previous-output-type span must be V0_P2WPKH, not P2PKH: {markup}"
    );
    assert!(
        class_span_values(&markup, "output-type")
            .iter()
            .any(|value| *value == "P2PKH"),
        "the output-type span may contain P2PKH: {markup}"
    );
    assert_sentinels_stay_off(&markup);
}

#[test]
fn type_with_markup_characters_is_uppercased_then_escaped() {
    let body = tx(&input(&prevout("a&b<c")), &output("op_return"));
    let markup = transaction_row_markup(&body);
    assert_one(&markup, &label_spans("A&amp;B&lt;C"));
    assert_absent(&markup, "A&B<C");
    assert_absent(&markup, "a&b<c");
    assert_sentinels_stay_off(&markup);
}

#[test]
fn two_prevouts_paint_the_label_twice() {
    let vin = format!(
        "{},{}",
        input(&prevout("v0_p2wpkh")),
        input(&prevout("p2sh"))
    );
    let markup = transaction_row_markup(&tx(&vin, &output("p2pkh")));
    assert_eq!(markup.matches(LABEL).count(), 2, "{markup}");
    assert_one(&markup, &label_spans("V0_P2WPKH"));
    assert_one(&markup, &label_spans("P2SH"));
    assert_sentinels_stay_off(&markup);
}

#[test]
fn empty_type_omits_the_label() {
    let body = tx(&input(&prevout("")), &output(""));
    let markup = transaction_row_markup(&body);
    assert_absent(&markup, LABEL);
    assert_absent(&markup, "previous-output-type");
    assert_absent(&markup, "scriptpubkey-type");
    assert_sentinels_stay_off(&markup);
}

#[test]
fn output_type_alone_is_not_the_previous_output_label() {
    let body = tx(&input("null"), &output("p2pkh"));
    let markup = transaction_row_markup(&body);
    assert_absent(&markup, LABEL);
    assert_absent(&markup, "previous-output-type");
    assert!(
        class_span_values(&markup, "previous-output-type")
            .iter()
            .all(|value| !value.contains("P2PKH")),
        "P2PKH leaked into the previous-output-type span: {markup}"
    );
    assert!(
        class_span_values(&markup, "output-type")
            .iter()
            .any(|value| *value == "P2PKH"),
        "the output-type span may contain P2PKH: {markup}"
    );
    assert_one(&markup, "<span class=\"scriptpubkey-type\">p2pkh</span>");
    assert_sentinels_stay_off(&markup);
}

#[test]
fn null_prevout_omits_the_label() {
    let body = tx(&input("null"), &output("p2pkh"));
    let markup = transaction_row_markup(&body);
    assert_absent(&markup, LABEL);
    assert_sentinels_stay_off(&markup);
}

#[test]
fn missing_prevout_key_omits_the_label() {
    let vin = format!(
        r#"{{"txid":"{PREV}","vout":0,"scriptsig":"{SCRIPT_SIG}","scriptsig_asm":"{SCRIPT_SIG_ASM}","is_coinbase":false,"sequence":1}}"#
    );
    let markup = transaction_row_markup(&tx(&vin, &output("p2pkh")));
    assert_absent(&markup, LABEL);
    assert_absent(&markup, "previous-output-type");
    assert_one(&markup, "<h2>Inputs & Outputs</h2>");
    assert_one(&markup, "<span class=\"unknown-input\">UNKNOWN</span>");
    assert_sentinels_stay_off(&markup);
}

#[test]
fn null_scriptpubkey_type_omits_the_label() {
    let prevout = format!(
        r#"{{"scriptpubkey":"{SCRIPT_HEX}","scriptpubkey_asm":"{SCRIPT_ASM}","scriptpubkey_type":null}}"#
    );
    let markup = transaction_row_markup(&tx(&input(&prevout), &output("p2pkh")));
    assert_absent(&markup, LABEL);
    assert_absent(&markup, "<h2>Inputs & Outputs</h2>");
    assert_sentinels_stay_off(&markup);
}

#[test]
fn missing_scriptpubkey_type_key_omits_the_label() {
    let prevout = format!(r#"{{"scriptpubkey":"{SCRIPT_HEX}","scriptpubkey_asm":"{SCRIPT_ASM}"}}"#);
    let markup = transaction_row_markup(&tx(&input(&prevout), &output("p2pkh")));
    assert_absent(&markup, LABEL);
    assert_absent(&markup, "<h2>Inputs & Outputs</h2>");
    assert_sentinels_stay_off(&markup);
}

#[test]
fn empty_body_not_json_and_array_omit_the_label() {
    for body in ["", "not-json", "[]"] {
        let markup = transaction_row_markup(body);
        assert_absent(&markup, LABEL);
        assert_absent(&markup, "<h2>Inputs & Outputs</h2>");
    }
}
