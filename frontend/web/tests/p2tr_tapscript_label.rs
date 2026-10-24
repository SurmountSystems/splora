//! P2TR tapscript on the transaction input from `GET /tx/:txid`.
//!
//! Angular paints the label `P2TR tapscript` when
//! `vin.prevout.scriptpubkey_type` is `v1_p2tr`. Any other prevout type,
//! a missing prevout, and a P2SH redeem input omit it. The output type is
//! not this label. The inner tapscript, script asm, script hex, scriptsig,
//! witness, and leaf version stay off the page. `P2SH redeem script` stays
//! on a redeem input and is not added to a tapscript input.

use splora_web::transaction_row_markup;

const LABEL: &str = "P2TR tapscript";
const LABEL_SPAN: &str = "<span class=\"p2tr-tapscript-label\">P2TR tapscript</span>";
const REDEEM_LABEL: &str = "P2SH redeem script";
const INNER_WITNESS_ASM: &str = "TAPSCRIPTASMDEAD";
const SCRIPT_SIG: &str = "TAPSCRIPTSIGHEXDEAD";
const SCRIPT_SIG_ASM: &str = "TAPSCRIPTSIGASMDEAD";
const WITNESS: &str = "TAPSCRIPTWITDEAD";
const LEAF_VERSION: &str = "LEAFVERSIONDEAD";
const PREV_SCRIPT: &str = "TAPSCRIPTPUBHEXDEAD";
const PREV_ASM: &str = "TAPSCRIPTPUBASMDEAD";
const OUT_SCRIPT: &str = "TAPSCRIPTOUTHEXDEAD";
const OUT_ASM: &str = "TAPSCRIPTOUTASMDEAD";
const REDEEM_ASM: &str = "OP_DUP OP_HASH160";

fn tx(prevout: &str, out_kind: &str, redeem_field: &str) -> String {
    format!(
        r#"{{"txid":"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa","version":1,"locktime":0,"vin":[{{"txid":"bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb","vout":0,"prevout":{prevout},"scriptsig":"{SCRIPT_SIG}","scriptsig_asm":"{SCRIPT_SIG_ASM}","witness":["{WITNESS}","{LEAF_VERSION}"],"is_coinbase":false,"sequence":1,"inner_witnessscript_asm":"{INNER_WITNESS_ASM}"{redeem_field}}}],"vout":[{{"scriptpubkey":"{OUT_SCRIPT}","scriptpubkey_asm":"{OUT_ASM}","scriptpubkey_type":"{out_kind}","value":1}}],"size":10,"weight":7,"sigops":0,"fee":1}}"#
    )
}

fn prevout(kind: &str) -> String {
    format!(
        r#"{{"scriptpubkey":"{PREV_SCRIPT}","scriptpubkey_asm":"{PREV_ASM}","scriptpubkey_type":"{kind}"}}"#
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
    assert_absent(markup, INNER_WITNESS_ASM);
    assert_absent(markup, SCRIPT_SIG);
    assert_absent(markup, SCRIPT_SIG_ASM);
    assert_absent(markup, WITNESS);
    assert_absent(markup, LEAF_VERSION);
    assert_absent(markup, PREV_SCRIPT);
    assert_absent(markup, PREV_ASM);
    assert_absent(markup, OUT_SCRIPT);
    assert_absent(markup, OUT_ASM);
}

fn assert_row_still_paints(markup: &str) {
    assert_one(markup, "<span class=\"sequence\">sequence 1</span>");
}

#[test]
fn v1_p2tr_input_paints_the_words_once() {
    let markup = transaction_row_markup(&tx(&prevout("v1_p2tr"), "v1_p2tr", ""));
    // The span and the whole line each contain the words.
    assert_eq!(markup.matches(LABEL).count(), 2, "{markup}");
    assert_one(&markup, LABEL_SPAN);
    assert_absent(&markup, REDEEM_LABEL);
    assert_sentinels_stay_off(&markup);
}

#[test]
fn other_prevout_type_omits_the_words() {
    let markup = transaction_row_markup(&tx(&prevout("v0_p2wsh"), "p2pkh", ""));
    assert_row_still_paints(&markup);
    assert_absent(&markup, LABEL);
    assert_sentinels_stay_off(&markup);
}

#[test]
fn p2sh_prevout_keeps_the_redeem_label_and_omits_the_words() {
    let redeem = format!(r#","inner_redeemscript_asm":"{REDEEM_ASM}""#);
    let markup = transaction_row_markup(&tx(&prevout("p2sh"), "p2pkh", &redeem));
    assert_row_still_paints(&markup);
    // The span and the whole line each contain the label and the asm.
    assert_eq!(markup.matches(REDEEM_LABEL).count(), 2, "{markup}");
    assert_eq!(markup.matches(REDEEM_ASM).count(), 2, "{markup}");
    assert_absent(&markup, LABEL);
    assert_sentinels_stay_off(&markup);
}

#[test]
fn null_prevout_redeem_input_omits_the_words() {
    let redeem = format!(r#","inner_redeemscript_asm":"{REDEEM_ASM}""#);
    let markup = transaction_row_markup(&tx("null", "p2pkh", &redeem));
    assert_row_still_paints(&markup);
    // The span and the whole line each contain the label.
    assert_eq!(markup.matches(REDEEM_LABEL).count(), 2, "{markup}");
    assert_absent(&markup, LABEL);
    assert_sentinels_stay_off(&markup);
}

#[test]
fn v1_p2tr_output_does_not_paint_the_words() {
    let markup = transaction_row_markup(&tx(&prevout("p2pkh"), "v1_p2tr", ""));
    assert_row_still_paints(&markup);
    assert_absent(&markup, LABEL);
    assert_sentinels_stay_off(&markup);
}
