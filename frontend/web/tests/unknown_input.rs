//! UNKNOWN on a transaction input that has no previous output.
//!
//! Angular paints UNKNOWN only in the address-cell default, when `vin.prevout`
//! is missing, the input is not lazy, and `vin.issuance` is absent. Coinbase
//! and peg-in take earlier branches. A present prevout, including one with no
//! address, does not paint UNKNOWN. A missing `prevout` key is not JSON null.
//! The row markup stays empty. Script hex,
//! scriptsig, scriptsig asm, witness, scriptpubkey, and scriptpubkey asm stay
//! off the page.

use splora_web::transaction_row_markup;

const SCRIPT_SIG: &str = "UNINSIGHEXDEAD";
const SCRIPT_SIG_ASM: &str = "UNINSIGASMDEAD";
const WITNESS: &str = "UNINWITDEAD";
const OUT_SCRIPT: &str = "UNINOUTHEXDEAD";
const OUT_ASM: &str = "UNINOUTASMDEAD";
const PREV_SCRIPT: &str = "UNINPREVHEXDEAD";
const PREV_ASM: &str = "UNINPREVASMDEAD";
const ASSET_ID: &str = "ce091c998b83c78bb71a632313ba3760f1763d9cfcffae02258ffa9865a37bd2";
const ASSET_ENTROPY: &str = "eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee";

const UNKNOWN_SPAN: &str = "<span class=\"unknown-input\">UNKNOWN</span>";

fn tx(vin: &str) -> String {
    format!(
        r#"{{"txid":"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa","version":1,"locktime":0,"vin":[{vin}],"vout":[{{"scriptpubkey":"{OUT_SCRIPT}","scriptpubkey_asm":"{OUT_ASM}","scriptpubkey_type":"p2pkh","value":1}}],"size":10,"weight":7,"sigops":0,"fee":1}}"#
    )
}

fn input(prevout_field: &str, is_coinbase: bool, extra: &str) -> String {
    format!(
        r#"{{"txid":"bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb","vout":0,{prevout_field}"scriptsig":"{SCRIPT_SIG}","scriptsig_asm":"{SCRIPT_SIG_ASM}","witness":["{WITNESS}"],"is_coinbase":{is_coinbase},"sequence":1{extra}}}"#
    )
}

fn null_prevout(is_coinbase: bool, extra: &str) -> String {
    input(r#""prevout":null,"#, is_coinbase, extra)
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
    assert_absent(markup, OUT_SCRIPT);
    assert_absent(markup, OUT_ASM);
    assert_absent(markup, PREV_SCRIPT);
    assert_absent(markup, PREV_ASM);
}

#[test]
fn null_prevout_paints_unknown_once() {
    let markup = transaction_row_markup(&tx(&null_prevout(false, "")));
    assert_one(&markup, UNKNOWN_SPAN);
    assert_eq!(
        markup.matches("UNKNOWN").count(),
        1,
        "UNKNOWN count in {markup}"
    );
    assert_one(&markup, "<span class=\"sequence\">sequence 1</span>");
    assert_sentinels_stay_off(&markup);
}

#[test]
fn present_prevout_without_an_address_does_not_paint_unknown() {
    let prevout = format!(
        r#""prevout":{{"scriptpubkey":"{PREV_SCRIPT}","scriptpubkey_asm":"{PREV_ASM}","scriptpubkey_type":"v0_p2wpkh"}},"#
    );
    let markup = transaction_row_markup(&tx(&input(&prevout, false, "")));
    assert_absent(&markup, "UNKNOWN");
    assert_one(
        &markup,
        "<span class=\"scriptpubkey-type\">input script type v0_p2wpkh</span>",
    );
    assert_sentinels_stay_off(&markup);
}

#[test]
fn issuance_without_prevout_does_not_paint_unknown() {
    let extra = format!(
        r#","issuance":{{"asset_id":"{ASSET_ID}","is_reissuance":false,"asset_entropy":"{ASSET_ENTROPY}"}}"#
    );
    let markup = transaction_row_markup(&tx(&null_prevout(false, &extra)));
    assert_absent(&markup, "UNKNOWN");
    assert_one(&markup, &format!("issuance asset {ASSET_ID}"));
    assert_sentinels_stay_off(&markup);
}

#[test]
fn coinbase_without_prevout_does_not_paint_unknown() {
    let markup = transaction_row_markup(&tx(&null_prevout(true, "")));
    assert_absent(&markup, "UNKNOWN");
    assert_one(&markup, "<span class=\"coinbase\">coinbase</span>");
    assert_sentinels_stay_off(&markup);
}

#[test]
fn peg_in_without_prevout_does_not_paint_unknown() {
    let markup = transaction_row_markup(&tx(&null_prevout(false, r#","is_pegin":true"#)));
    assert_absent(&markup, "UNKNOWN");
    assert_one(&markup, "<span class=\"sequence\">sequence 1</span>");
    assert_sentinels_stay_off(&markup);
}

#[test]
fn omitted_prevout_key_paints_unknown_once() {
    let markup = transaction_row_markup(&tx(&input("", false, "")));
    assert_one(&markup, UNKNOWN_SPAN);
    assert_eq!(
        markup.matches("UNKNOWN").count(),
        1,
        "UNKNOWN count in {markup}"
    );
    assert_one(&markup, "<span class=\"sequence\">sequence 1</span>");
    assert_sentinels_stay_off(&markup);
}
