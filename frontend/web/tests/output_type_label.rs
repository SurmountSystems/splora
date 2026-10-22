//! Output Type on a transaction output from `GET /tx/:txid`.
//!
//! Angular paints the row name `Type` and
//! `vout.scriptpubkey_type.toUpperCase()` when that field is nonempty.
//! An empty type string omits the label for that output. A missing
//! `scriptpubkey_type` key and JSON null do not parse, so the label stays
//! off. An empty body, `not-json`, and `[]` do not parse either.
//! The lowercase `scriptpubkey-type` span stays. Previous output type is
//! the input prevout, not this label. A fee output's Type cell is the
//! pipe text Transaction fee, not FEE. A fee input still uppercases to FEE.
//! Script hex, script asm, scriptsig, and witness stay off the page.
//! This row has no icon, clipboard control, or formatted amount.

use splora_web::transaction_row_markup;

const TXID: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
const PREV: &str = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
const SCRIPT_HEX: &str = "OTYPUBHEXDEAD";
const SCRIPT_ASM: &str = "OTYASMDEAD";
const SCRIPT_SIG: &str = "OTYSIGHEXDEAD";
const SCRIPT_SIG_ASM: &str = "OTYSIGASMDEAD";
const WITNESS: &str = "OTYWITDEAD";
const LABEL: &str = "Type";

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
        "<span class=\"output-type-label\">{LABEL}</span><span class=\"output-type\">{upper}</span>"
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

fn assert_no_extra_controls(markup: &str) {
    assert_absent(markup, "clipboard");
    assert_absent(markup, "fa-icon");
    assert_absent(markup, "copy-button");
    assert_absent(markup, " BTC");
}

#[test]
fn output_type_paints_the_label_and_the_uppercase_type() {
    let body = tx(&input("null"), &output("v0_p2wpkh"));
    let markup = transaction_row_markup(&body);
    assert_eq!(markup.matches(LABEL).count(), 2, "{markup}");
    assert_one(&markup, &label_spans("V0_P2WPKH"));
    assert_one(
        &markup,
        "<span class=\"scriptpubkey-type\">v0_p2wpkh</span>",
    );
    assert_absent(&markup, "Previous output type");
    assert_absent(&markup, "previous-output-type");
    assert_no_extra_controls(&markup);
    assert_sentinels_stay_off(&markup);
}

#[test]
fn type_with_markup_characters_is_uppercased_then_escaped() {
    let body = tx(&input("null"), &output("a&b<c"));
    let markup = transaction_row_markup(&body);
    assert_one(&markup, &label_spans("A&amp;B&lt;C"));
    assert_one(
        &markup,
        "<span class=\"scriptpubkey-type\">a&amp;b&lt;c</span>",
    );
    assert_absent(&markup, "A&B<C");
    assert_absent(&markup, "a&b<c");
    assert_no_extra_controls(&markup);
    assert_sentinels_stay_off(&markup);
}

#[test]
fn two_outputs_paint_the_label_twice() {
    let vout = format!("{},{}", output("v0_p2wpkh"), output("p2sh"));
    let markup = transaction_row_markup(&tx(&input("null"), &vout));
    assert_eq!(markup.matches(LABEL).count(), 4, "{markup}");
    assert_one(&markup, &label_spans("V0_P2WPKH"));
    assert_one(&markup, &label_spans("P2SH"));
    assert_one(
        &markup,
        "<span class=\"scriptpubkey-type\">v0_p2wpkh</span>",
    );
    assert_one(&markup, "<span class=\"scriptpubkey-type\">p2sh</span>");
    assert_sentinels_stay_off(&markup);
}

#[test]
fn output_type_stays_beside_the_previous_output_label() {
    let body = tx(&input(&prevout("v0_p2wpkh")), &output("p2pkh"));
    let markup = transaction_row_markup(&body);
    assert_eq!(markup.matches(LABEL).count(), 2, "{markup}");
    assert_one(&markup, &label_spans("P2PKH"));
    assert_one(&markup, "Previous output type");
    assert_one(
        &markup,
        "<span class=\"previous-output-type\">V0_P2WPKH</span>",
    );
    assert_one(&markup, "<span class=\"scriptpubkey-type\">p2pkh</span>");
    assert_one(
        &markup,
        "<span class=\"scriptpubkey-type\">input script type v0_p2wpkh</span>",
    );
    assert_sentinels_stay_off(&markup);
}

#[test]
fn empty_type_omits_the_label_for_that_output() {
    let vout = format!("{},{}", output(""), output("p2pkh"));
    let markup = transaction_row_markup(&tx(&input("null"), &vout));
    assert_eq!(markup.matches(LABEL).count(), 2, "{markup}");
    assert_one(&markup, &label_spans("P2PKH"));
    assert_one(&markup, "<h2>Inputs & Outputs</h2>");
    assert_one(&markup, "<span class=\"scriptpubkey-type\">p2pkh</span>");
    assert_sentinels_stay_off(&markup);
}

#[test]
fn only_empty_type_omits_the_label() {
    let body = tx(&input("null"), &output(""));
    let markup = transaction_row_markup(&body);
    assert_absent(&markup, LABEL);
    assert_absent(&markup, "class=\"output-type\"");
    assert_absent(&markup, "scriptpubkey-type");
    assert_one(&markup, "<h2>Inputs & Outputs</h2>");
    assert_sentinels_stay_off(&markup);
}

#[test]
fn fee_output_type_is_transaction_fee_and_fee_input_stays_uppercase() {
    let body = tx(&input(&prevout("fee")), &output("fee"));
    let markup = transaction_row_markup(&body);
    assert_one(&markup, &label_spans("Transaction fee"));
    assert_absent(&markup, "<span class=\"output-type\">FEE</span>");
    assert_one(&markup, "<span class=\"previous-output-type\">FEE</span>");
    assert_one(&markup, "<span class=\"scriptpubkey-type\">fee</span>");
    assert_one(
        &markup,
        "<span class=\"scriptpubkey-type\">input script type fee</span>",
    );
    assert_one(&markup, "<span class=\"label\">Fee</span>");
    assert_eq!(markup.matches("Fee rate").count(), 2, "{markup}");
    assert_eq!(markup.matches("<td class=\"fee\">").count(), 1, "{markup}");
    assert_absent(&markup, "<span class=\"output-type\">OP_RETURN</span>");
    assert_no_extra_controls(&markup);
    assert_sentinels_stay_off(&markup);
}

#[test]
fn op_return_keeps_its_span_and_paints_type() {
    let body = tx(&input("null"), &output("op_return"));
    let markup = transaction_row_markup(&body);
    assert_one(
        &markup,
        "<span class=\"scriptpubkey-type\">op_return</span>",
    );
    assert_one(&markup, "<span class=\"op-return\">OP_RETURN</span>");
    assert_one(&markup, &label_spans("OP_RETURN"));
    assert_no_extra_controls(&markup);
    assert_sentinels_stay_off(&markup);
}

#[test]
fn null_scriptpubkey_type_omits_the_label() {
    let vout = format!(
        r#"{{"scriptpubkey":"{SCRIPT_HEX}","scriptpubkey_asm":"{SCRIPT_ASM}","scriptpubkey_type":null,"value":1}}"#
    );
    let markup = transaction_row_markup(&tx(&input("null"), &vout));
    assert_absent(&markup, LABEL);
    assert_absent(&markup, "<h2>Inputs & Outputs</h2>");
    assert_sentinels_stay_off(&markup);
}

#[test]
fn missing_scriptpubkey_type_key_omits_the_label() {
    let vout =
        format!(r#"{{"scriptpubkey":"{SCRIPT_HEX}","scriptpubkey_asm":"{SCRIPT_ASM}","value":1}}"#);
    let markup = transaction_row_markup(&tx(&input("null"), &vout));
    assert_absent(&markup, LABEL);
    assert_absent(&markup, "<h2>Inputs & Outputs</h2>");
    assert_sentinels_stay_off(&markup);
}

#[test]
fn empty_body_not_json_and_array_omit_the_label() {
    for body in ["", "not-json", "[]"] {
        let markup = transaction_row_markup(body);
        assert_absent(&markup, LABEL);
        assert_absent(&markup, "class=\"output-type\"");
        assert_absent(&markup, "<h2>Inputs & Outputs</h2>");
    }
}
