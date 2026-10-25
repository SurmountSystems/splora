//! Previous output script on the transaction input from `GET /tx/:txid`.
//!
//! Angular paints `Previous output script` when `vin.prevout` is present.
//! The cell sits beside `vin.prevout.scriptpubkey_asm`. This row paints the
//! words only. JSON null, an omitted `prevout` key, and a failed parse omit
//! the words. An output is not this label. Script hex, script asm, scriptsig,
//! witness, the inner tapscript string, and leaf version stay off the page.
//! `P2TR tapscript`, `P2SH redeem script`, `Issuance`, and `Transaction fee`
//! stay on their own rows and are not copied.

use splora_web::{transaction_requests, transaction_row_markup};

const TXID: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
const PREV: &str = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
const LABEL: &str = "Previous output script";
const LABEL_SPAN: &str =
    "<span class=\"previous-output-script-label\">Previous output script</span>";
const SCRIPT_HEX: &str = "PREVOUTHEXDEAD";
const SCRIPT_ASM: &str = "PREVOUTASMDEAD";
const SCRIPT_SIG: &str = "PREVSIGHEXDEAD";
const SCRIPT_SIG_ASM: &str = "PREVSIGASMDEAD";
const WITNESS: &str = "PREVWITDEAD";
const INNER_WITNESS_ASM: &str = "TAPSCRIPTASMDEAD";
const LEAF_VERSION: &str = "LEAFVERSIONDEAD";
const REDEEM_ASM: &str = "OP_DUP OP_HASH160";
const P2TR_LABEL: &str = "P2TR tapscript";
const P2TR_SPAN: &str = "<span class=\"p2tr-tapscript-label\">P2TR tapscript</span>";
const REDEEM_LABEL: &str = "P2SH redeem script";
const ISSUANCE_LABEL: &str = "Issuance";
const FEE_LABEL: &str = "Transaction fee";

fn tx(vin: &str, vout: &str) -> String {
    format!(
        r#"{{"txid":"{TXID}","version":1,"locktime":0,"vin":[{vin}],"vout":[{vout}],"size":10,"weight":40,"sigops":0,"fee":1}}"#
    )
}

fn input(prevout: &str, extra: &str) -> String {
    format!(
        r#"{{"txid":"{PREV}","vout":0,"prevout":{prevout},"scriptsig":"{SCRIPT_SIG}","scriptsig_asm":"{SCRIPT_SIG_ASM}","witness":["{WITNESS}"],"is_coinbase":false,"sequence":1{extra}}}"#
    )
}

fn prevout(kind: &str, asm: &str) -> String {
    format!(
        r#"{{"scriptpubkey":"{SCRIPT_HEX}","scriptpubkey_asm":"{asm}","scriptpubkey_type":"{kind}"}}"#
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
    assert_absent(markup, INNER_WITNESS_ASM);
    assert_absent(markup, LEAF_VERSION);
}

fn assert_row_still_paints(markup: &str) {
    assert_one(markup, "<span class=\"sequence\">sequence 1</span>");
    assert_one(markup, "<h2>Inputs & Outputs</h2>");
}

#[test]
fn prevout_paints_the_words_once() {
    let body = tx(
        &input(&prevout("v0_p2wpkh", SCRIPT_ASM), ""),
        &output("p2pkh"),
    );
    let markup = paint(&body);
    assert_eq!(markup.matches(LABEL).count(), 2, "{markup}");
    assert_one(&markup, LABEL_SPAN);
    // The label span and the whole line each contain this token.
    assert_eq!(
        markup.matches("Previous output type").count(),
        2,
        "{markup}"
    );
    assert_sentinels_stay_off(&markup);
}

#[test]
fn empty_asm_still_paints_the_words_once() {
    let body = tx(&input(&prevout("v0_p2wpkh", ""), ""), &output("p2pkh"));
    let markup = paint(&body);
    assert_row_still_paints(&markup);
    assert_eq!(markup.matches(LABEL).count(), 2, "{markup}");
    assert_one(&markup, LABEL_SPAN);
    assert_absent(&markup, SCRIPT_HEX);
    assert_absent(&markup, SCRIPT_SIG);
    assert_absent(&markup, WITNESS);
}

#[test]
fn empty_type_still_paints_the_words_once() {
    let body = tx(&input(&prevout("", SCRIPT_ASM), ""), &output("p2pkh"));
    let markup = paint(&body);
    assert_row_still_paints(&markup);
    assert_eq!(markup.matches(LABEL).count(), 2, "{markup}");
    assert_one(&markup, LABEL_SPAN);
    assert_absent(&markup, "Previous output type");
    assert_sentinels_stay_off(&markup);
}

#[test]
fn two_prevouts_paint_the_words_twice() {
    let vin = format!(
        "{},{}",
        input(&prevout("v0_p2wpkh", SCRIPT_ASM), ""),
        input(&prevout("p2sh", SCRIPT_ASM), "")
    );
    let markup = paint(&tx(&vin, &output("p2pkh")));
    assert_eq!(markup.matches(LABEL).count(), 4, "{markup}");
    assert_eq!(markup.matches(LABEL_SPAN).count(), 2, "{markup}");
    assert_sentinels_stay_off(&markup);
}

#[test]
fn one_prevout_and_one_null_paint_the_words_once() {
    let vin = format!(
        "{},{}",
        input(&prevout("v0_p2wpkh", SCRIPT_ASM), ""),
        input("null", "")
    );
    let markup = paint(&tx(&vin, &output("p2pkh")));
    assert_eq!(markup.matches(LABEL).count(), 2, "{markup}");
    assert_one(&markup, LABEL_SPAN);
    assert_sentinels_stay_off(&markup);
}

#[test]
fn null_prevout_omits_the_words() {
    let markup = paint(&tx(&input("null", ""), &output("p2pkh")));
    assert_row_still_paints(&markup);
    assert_absent(&markup, LABEL);
    assert_absent(&markup, "previous-output-script");
    assert_sentinels_stay_off(&markup);
}

#[test]
fn missing_prevout_key_omits_the_words() {
    let vin = format!(
        r#"{{"txid":"{PREV}","vout":0,"scriptsig":"{SCRIPT_SIG}","scriptsig_asm":"{SCRIPT_SIG_ASM}","witness":["{WITNESS}"],"is_coinbase":false,"sequence":1}}"#
    );
    let markup = paint(&tx(&vin, &output("p2pkh")));
    assert_row_still_paints(&markup);
    assert_one(&markup, "<span class=\"unknown-input\">UNKNOWN</span>");
    assert_absent(&markup, LABEL);
    assert_sentinels_stay_off(&markup);
}

#[test]
fn output_alone_is_not_the_label() {
    let markup = paint(&tx(&input("null", ""), &output("v1_p2tr")));
    assert_row_still_paints(&markup);
    assert_absent(&markup, LABEL);
    assert_absent(&markup, P2TR_LABEL);
    assert_sentinels_stay_off(&markup);
}

#[test]
fn v1_p2tr_prevout_keeps_the_tapscript_label_once() {
    let extra = format!(r#","inner_witnessscript_asm":"{INNER_WITNESS_ASM}""#);
    let vin = format!(
        r#"{{"txid":"{PREV}","vout":0,"prevout":{},"scriptsig":"{SCRIPT_SIG}","scriptsig_asm":"{SCRIPT_SIG_ASM}","witness":["{WITNESS}","{LEAF_VERSION}"],"is_coinbase":false,"sequence":1{extra}}}"#,
        prevout("v1_p2tr", SCRIPT_ASM)
    );
    let markup = paint(&tx(&vin, &output("p2pkh")));
    assert_eq!(markup.matches(LABEL).count(), 2, "{markup}");
    assert_one(&markup, LABEL_SPAN);
    // The span and the whole line each contain the words.
    assert_eq!(markup.matches(P2TR_LABEL).count(), 2, "{markup}");
    assert_one(&markup, P2TR_SPAN);
    assert_absent(&markup, REDEEM_LABEL);
    assert_sentinels_stay_off(&markup);
}

#[test]
fn p2sh_redeem_with_prevout_keeps_the_redeem_label_once() {
    let extra = format!(r#","inner_redeemscript_asm":"{REDEEM_ASM}""#);
    let markup = paint(&tx(
        &input(&prevout("p2sh", SCRIPT_ASM), &extra),
        &output("p2pkh"),
    ));
    assert_eq!(markup.matches(LABEL).count(), 2, "{markup}");
    assert_one(&markup, LABEL_SPAN);
    // The span and the whole line each contain the label and the asm.
    assert_eq!(markup.matches(REDEEM_LABEL).count(), 2, "{markup}");
    assert_eq!(markup.matches(REDEEM_ASM).count(), 2, "{markup}");
    assert_one(
        &markup,
        "<span class=\"p2sh-redeem-label\">P2SH redeem script</span>",
    );
    assert_absent(&markup, P2TR_LABEL);
    assert_sentinels_stay_off(&markup);
}

#[test]
fn issuance_without_prevout_keeps_issuance_and_omits_the_words() {
    let vin = format!(
        r#"{{"txid":"{PREV}","vout":0,"prevout":null,"issuance":{{"asset_id":"ce091c998b83c78bb71a632313ba3760f1763d9cfcffae02258ffa9865a37bd2","is_reissuance":false,"asset_entropy":"eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee"}},"scriptsig":"{SCRIPT_SIG}","scriptsig_asm":"{SCRIPT_SIG_ASM}","witness":["{WITNESS}"],"is_coinbase":false,"sequence":1}}"#
    );
    let markup = paint(&tx(&vin, &output("op_return")));
    assert_row_still_paints(&markup);
    assert_one(&markup, ISSUANCE_LABEL);
    assert_one(&markup, "<span class=\"issuance-label\">Issuance</span>");
    assert_absent(&markup, LABEL);
    assert_sentinels_stay_off(&markup);
}

#[test]
fn fee_output_with_prevout_keeps_transaction_fee_once() {
    let markup = paint(&tx(
        &input(&prevout("v0_p2wpkh", SCRIPT_ASM), ""),
        &output("fee"),
    ));
    assert_eq!(markup.matches(LABEL).count(), 2, "{markup}");
    assert_one(&markup, LABEL_SPAN);
    assert_one(&markup, FEE_LABEL);
    assert_one(
        &markup,
        "<span class=\"output-type\">Transaction fee</span>",
    );
    assert_sentinels_stay_off(&markup);
}

#[test]
fn fee_output_without_prevout_keeps_transaction_fee_and_omits_the_words() {
    let markup = paint(&tx(&input("null", ""), &output("fee")));
    assert_row_still_paints(&markup);
    assert_one(&markup, FEE_LABEL);
    assert_absent(&markup, LABEL);
    assert_sentinels_stay_off(&markup);
}

#[test]
fn null_scriptpubkey_type_omits_the_words() {
    let prevout = format!(
        r#"{{"scriptpubkey":"{SCRIPT_HEX}","scriptpubkey_asm":"{SCRIPT_ASM}","scriptpubkey_type":null}}"#
    );
    let markup = paint(&tx(&input(&prevout, ""), &output("p2pkh")));
    assert_absent(&markup, LABEL);
    assert_absent(&markup, "<h2>Inputs & Outputs</h2>");
    assert_sentinels_stay_off(&markup);
}

#[test]
fn missing_scriptpubkey_type_key_omits_the_words() {
    let prevout = format!(r#"{{"scriptpubkey":"{SCRIPT_HEX}","scriptpubkey_asm":"{SCRIPT_ASM}"}}"#);
    let markup = paint(&tx(&input(&prevout, ""), &output("p2pkh")));
    assert_absent(&markup, LABEL);
    assert_absent(&markup, "<h2>Inputs & Outputs</h2>");
    assert_sentinels_stay_off(&markup);
}

#[test]
fn empty_body_not_json_and_array_omit_the_words() {
    for body in ["", "not-json", "[]"] {
        let markup = transaction_row_markup(body);
        assert_absent(&markup, LABEL);
        assert_absent(&markup, "<h2>Inputs & Outputs</h2>");
    }
}
