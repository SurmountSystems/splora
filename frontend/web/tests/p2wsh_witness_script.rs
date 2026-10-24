//! P2WSH witness script on the transaction input from `GET /tx/:txid`.
//!
//! Angular paints the label `P2WSH witness script` when
//! `vin.inner_witnessscript_asm` is present and non-empty and the prevout
//! is not `v1_p2tr`. A `v1_p2tr` prevout keeps `P2TR tapscript` and omits
//! this label. A missing key, JSON null, and an empty string omit it.
//! An output whose type is `v0_p2wsh` does not paint it. The witness script
//! asm, script hex, script asm, scriptsig, witness stack, and leaf version
//! stay off the page.

use splora_web::{transaction_requests, transaction_row_markup};

const LABEL: &str = "P2WSH witness script";
const LABEL_SPAN: &str = "<span class=\"p2wsh-witness-script-label\">P2WSH witness script</span>";
const P2TR_LABEL: &str = "P2TR tapscript";
const TXID: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
const INNER: &str = "P2WSHWITNESSASMDEAD";
const SCRIPT_SIG: &str = "P2WSHSIGHEXDEAD";
const SCRIPT_SIG_ASM: &str = "P2WSHSIGASMDEAD";
const WITNESS: &str = "P2WSHWITDEAD";
const LEAF_VERSION: &str = "P2WSHLEAFDEAD";
const PREV_SCRIPT: &str = "P2WSHPUBHEXDEAD";
const PREV_ASM: &str = "P2WSHPUBASMDEAD";
const OUT_SCRIPT: &str = "P2WSHOUTHEXDEAD";
const OUT_ASM: &str = "P2WSHOUTASMDEAD";

enum Inner {
    Present,
    Missing,
    Null,
    Empty,
}

fn inner_field(inner: Inner) -> String {
    match inner {
        Inner::Present => format!(r#","inner_witnessscript_asm":"{INNER}""#),
        Inner::Missing => String::new(),
        Inner::Null => r#","inner_witnessscript_asm":null"#.to_string(),
        Inner::Empty => r#","inner_witnessscript_asm":"""#.to_string(),
    }
}

fn tx(prevout: &str, out_kind: &str, inner: Inner) -> String {
    let inner_field = inner_field(inner);
    format!(
        r#"{{"txid":"{TXID}","version":1,"locktime":0,"vin":[{{"txid":"bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb","vout":0,"prevout":{prevout},"scriptsig":"{SCRIPT_SIG}","scriptsig_asm":"{SCRIPT_SIG_ASM}","witness":["{WITNESS}","{LEAF_VERSION}"],"is_coinbase":false,"sequence":1{inner_field}}}],"vout":[{{"scriptpubkey":"{OUT_SCRIPT}","scriptpubkey_asm":"{OUT_ASM}","scriptpubkey_type":"{out_kind}","value":1}}],"size":10,"weight":7,"sigops":0,"fee":1}}"#
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
    assert_absent(markup, INNER);
    assert_absent(markup, SCRIPT_SIG);
    assert_absent(markup, SCRIPT_SIG_ASM);
    assert_absent(markup, WITNESS);
    assert_absent(markup, LEAF_VERSION);
    assert_absent(markup, PREV_SCRIPT);
    assert_absent(markup, PREV_ASM);
    assert_absent(markup, OUT_SCRIPT);
    assert_absent(markup, OUT_ASM);
    assert_absent(markup, "Witness");
    assert_absent(markup, "leaf version");
}

fn assert_row_still_paints(markup: &str) {
    assert_one(markup, "<span class=\"sequence\">sequence 1</span>");
}

#[test]
fn v0_p2wsh_input_paints_the_label_once() {
    let calls = transaction_requests(TXID);
    assert!(
        calls.iter().any(|call| {
            call.method == "GET"
                && call.path == format!("/tx/{TXID}")
                && !call.path.contains('?')
                && !call.path.contains('#')
        }),
        "no-query GET /tx/:txid is missing: {calls:?}"
    );
    let markup = transaction_row_markup(&tx(&prevout("v0_p2wsh"), "p2pkh", Inner::Present));
    assert_eq!(markup.matches(LABEL).count(), 2, "{markup}");
    assert_one(&markup, LABEL_SPAN);
    assert_absent(&markup, P2TR_LABEL);
    assert_row_still_paints(&markup);
    assert_sentinels_stay_off(&markup);
}

#[test]
fn v1_p2tr_prevout_keeps_tapscript_and_omits_this_label() {
    let markup = transaction_row_markup(&tx(&prevout("v1_p2tr"), "p2pkh", Inner::Present));
    assert_row_still_paints(&markup);
    // The span and the whole line each contain the words.
    assert_eq!(markup.matches(P2TR_LABEL).count(), 2, "{markup}");
    assert_absent(&markup, LABEL);
    assert_sentinels_stay_off(&markup);
}

#[test]
fn null_prevout_with_inner_script_paints_the_label() {
    let markup = transaction_row_markup(&tx("null", "p2pkh", Inner::Present));
    assert_row_still_paints(&markup);
    assert_eq!(markup.matches(LABEL).count(), 2, "{markup}");
    assert_one(&markup, LABEL_SPAN);
    assert_absent(&markup, P2TR_LABEL);
    assert_sentinels_stay_off(&markup);
}

#[test]
fn missing_inner_script_omits_the_label() {
    let markup = transaction_row_markup(&tx(&prevout("v0_p2wsh"), "p2pkh", Inner::Missing));
    assert_row_still_paints(&markup);
    assert_absent(&markup, LABEL);
    assert_sentinels_stay_off(&markup);
}

#[test]
fn null_inner_script_omits_the_label() {
    let markup = transaction_row_markup(&tx(&prevout("v0_p2wsh"), "p2pkh", Inner::Null));
    assert_row_still_paints(&markup);
    assert_absent(&markup, LABEL);
    assert_sentinels_stay_off(&markup);
}

#[test]
fn empty_inner_script_omits_the_label() {
    let markup = transaction_row_markup(&tx(&prevout("v0_p2wsh"), "p2pkh", Inner::Empty));
    assert_row_still_paints(&markup);
    assert_absent(&markup, LABEL);
    assert_sentinels_stay_off(&markup);
}

#[test]
fn v0_p2wsh_output_does_not_paint_the_label() {
    let markup = transaction_row_markup(&tx(&prevout("p2pkh"), "v0_p2wsh", Inner::Missing));
    assert_row_still_paints(&markup);
    assert_absent(&markup, LABEL);
    assert_sentinels_stay_off(&markup);
}

/// Whole line `P2WSH witness script` when the inner witness script is set.
///
/// GPUI `transaction_screen_paints_p2wsh_witness_script` requires one line
/// equal to those words. The span stays. A v1_p2tr prevout, a missing inner
/// script, and a v0_p2wsh output omit the line. The script text stays off.
#[test]
fn v0_p2wsh_input_paints_p2wsh_witness_script_as_its_own_line() {
    const LINE: &str = "P2WSH witness script";
    let markup = transaction_row_markup(&tx(&prevout("v0_p2wsh"), "p2pkh", Inner::Present));
    assert!(
        markup.lines().any(|line| line == LINE),
        "no line equals {LINE}: {markup}"
    );
    assert_eq!(
        markup.lines().filter(|line| *line == LINE).count(),
        1,
        "{LINE} was not painted once: {markup}"
    );
    assert!(
        markup.contains(LABEL_SPAN),
        "the span was dropped: {markup}"
    );
    assert!(
        !markup
            .lines()
            .any(|line| line.starts_with("P2WSH witness script ")),
        "{LINE} was painted as a longer sentence: {markup}"
    );
    assert_sentinels_stay_off(&markup);
    let null_prevout = transaction_row_markup(&tx("null", "p2pkh", Inner::Present));
    assert_eq!(
        null_prevout.lines().filter(|line| *line == LINE).count(),
        1,
        "a null prevout did not paint {LINE} once: {null_prevout}"
    );
    assert_sentinels_stay_off(&null_prevout);
    let tapscript = transaction_row_markup(&tx(&prevout("v1_p2tr"), "p2pkh", Inner::Present));
    assert!(
        !tapscript.lines().any(|line| line == LINE),
        "a v1_p2tr prevout painted {LINE}: {tapscript}"
    );
    assert!(
        tapscript.contains(P2TR_LABEL),
        "P2TR tapscript was dropped: {tapscript}"
    );
    for omitted in [
        transaction_row_markup(&tx(&prevout("v0_p2wsh"), "p2pkh", Inner::Missing)),
        transaction_row_markup(&tx(&prevout("v0_p2wsh"), "p2pkh", Inner::Null)),
        transaction_row_markup(&tx(&prevout("v0_p2wsh"), "p2pkh", Inner::Empty)),
        transaction_row_markup(&tx(&prevout("p2pkh"), "v0_p2wsh", Inner::Missing)),
    ] {
        assert!(
            !omitted.lines().any(|line| line == LINE),
            "an omit path painted {LINE}: {omitted}"
        );
        assert_sentinels_stay_off(&omitted);
    }
}
