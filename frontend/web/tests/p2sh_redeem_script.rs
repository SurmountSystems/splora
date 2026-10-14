//! P2SH redeem script on the transaction input from `GET /tx/:txid`.
//!
//! Angular paints the label `P2SH redeem script` and the asm string when
//! `vin.inner_redeemscript_asm` is present and non-empty. A missing key,
//! JSON null, and an empty string omit the label. The asm is HTML-escaped.
//! Script hex, scriptsig, scriptsig asm, witness, scriptpubkey, scriptpubkey
//! asm, and inner witness script asm stay off the page.

use splora_web::transaction_row_markup;

const REDEEM_ASM: &str = "OP_DUP OP_HASH160";
const SCRIPT_SIG: &str = "P2SHSIGHEXDEAD";
const SCRIPT_SIG_ASM: &str = "P2SHSIGASMDEAD";
const WITNESS: &str = "P2SHWITDEAD";
const SCRIPT_PUBKEY: &str = "P2SHPUBHEXDEAD";
const SCRIPT_PUBKEY_ASM: &str = "P2SHPUBASMDEAD";
const INNER_WITNESS_ASM: &str = "P2SHWITASMDEAD";

enum Redeem {
    Present(&'static str),
    Missing,
    Null,
    Empty,
}

fn tx(redeem: Redeem) -> String {
    let redeem_field = match redeem {
        Redeem::Present(asm) => format!(r#","inner_redeemscript_asm":"{asm}""#),
        Redeem::Missing => String::new(),
        Redeem::Null => r#","inner_redeemscript_asm":null"#.to_string(),
        Redeem::Empty => r#","inner_redeemscript_asm":"""#.to_string(),
    };
    format!(
        r#"{{"txid":"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa","version":1,"locktime":0,"vin":[{{"txid":"bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb","vout":0,"prevout":null,"scriptsig":"{SCRIPT_SIG}","scriptsig_asm":"{SCRIPT_SIG_ASM}","witness":["{WITNESS}"],"is_coinbase":false,"sequence":1,"inner_witnessscript_asm":"{INNER_WITNESS_ASM}"{redeem_field}}}],"vout":[{{"scriptpubkey":"{SCRIPT_PUBKEY}","scriptpubkey_asm":"{SCRIPT_PUBKEY_ASM}","scriptpubkey_type":"p2pkh","value":1}}],"size":10,"weight":7,"sigops":0,"fee":1}}"#
    )
}

fn redeem_spans(asm: &str) -> String {
    format!(
        "<span class=\"p2sh-redeem-label\">P2SH redeem script</span><span class=\"p2sh-redeem-script\">{asm}</span>"
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
    assert_absent(markup, SCRIPT_SIG);
    assert_absent(markup, SCRIPT_SIG_ASM);
    assert_absent(markup, WITNESS);
    assert_absent(markup, SCRIPT_PUBKEY);
    assert_absent(markup, SCRIPT_PUBKEY_ASM);
    assert_absent(markup, INNER_WITNESS_ASM);
}

fn assert_row_still_paints(markup: &str) {
    assert_one(markup, "<span class=\"sequence\">sequence 1</span>");
}

#[test]
fn present_asm_paints_the_label_and_the_asm_once() {
    let markup = transaction_row_markup(&tx(Redeem::Present(REDEEM_ASM)));
    assert_one(&markup, "P2SH redeem script");
    assert_one(&markup, REDEEM_ASM);
    assert_one(&markup, &redeem_spans(REDEEM_ASM));
    assert_sentinels_stay_off(&markup);
}

#[test]
fn angle_bracket_asm_is_escaped_and_stays_visible_as_text() {
    let asm = "OP_DUP <OP_HASH160>";
    let markup = transaction_row_markup(&tx(Redeem::Present(asm)));
    assert_one(&markup, "P2SH redeem script");
    assert_one(&markup, &redeem_spans("OP_DUP &lt;OP_HASH160&gt;"));
    assert_absent(&markup, "<OP_HASH160>");
    assert_sentinels_stay_off(&markup);
}

#[test]
fn missing_key_omits_the_label() {
    let markup = transaction_row_markup(&tx(Redeem::Missing));
    assert_row_still_paints(&markup);
    assert_absent(&markup, "P2SH redeem script");
    assert_absent(&markup, REDEEM_ASM);
    assert_sentinels_stay_off(&markup);
}

#[test]
fn json_null_omits_the_label() {
    let markup = transaction_row_markup(&tx(Redeem::Null));
    assert_row_still_paints(&markup);
    assert_absent(&markup, "P2SH redeem script");
    assert_sentinels_stay_off(&markup);
}

#[test]
fn empty_string_omits_the_label() {
    let markup = transaction_row_markup(&tx(Redeem::Empty));
    assert_row_still_paints(&markup);
    assert_absent(&markup, "P2SH redeem script");
    assert_sentinels_stay_off(&markup);
}
