//! Preview fee on the transaction details row from `GET /tx/:txid`.
//!
//! Angular paints `tx.fee | number` with DecimalPipe `1.0-3` and an en-US
//! grouping comma. Fee is an integer, so there is no fraction. The cell is
//! `<span class="label">Fee</span> {grouped} <span class="symbol">sats</span>`.
//! The raw fee cell stays. Virtual size and adjusted vsize stay.

use splora_web::transaction_row_markup;

const SCRIPT_HEX: &str = "TXFEEPUBHEXDEAD";
const SCRIPT_SIG_HEX: &str = "TXFEESIGHEXDEAD";
const SCRIPT_SIG_ASM: &str = "TXFEESIGASMDEAD";
const SCRIPT_ASM: &str = "TXFEEASMDEAD";

fn tx(fee: u64, weight: u32) -> String {
    format!(
        r#"{{"txid":"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa","version":1,"locktime":0,"vin":[{{"txid":"bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb","vout":0,"prevout":null,"scriptsig":"{SCRIPT_SIG_HEX}","scriptsig_asm":"{SCRIPT_SIG_ASM}","is_coinbase":false,"sequence":1}}],"vout":[{{"scriptpubkey":"{SCRIPT_HEX}","scriptpubkey_asm":"{SCRIPT_ASM}","scriptpubkey_type":"p2pkh","value":1}}],"size":10,"weight":{weight},"sigops":0,"fee":{fee}}}"#
    )
}

fn preview_cell(grouped: &str) -> String {
    format!(
        "<td class=\"preview-fee\"><span class=\"label\">Fee</span> {grouped} <span class=\"symbol\">sats</span></td>"
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

fn assert_scripts_stay_off(markup: &str) {
    assert_absent(markup, SCRIPT_HEX);
    assert_absent(markup, SCRIPT_SIG_HEX);
    assert_absent(markup, SCRIPT_SIG_ASM);
    assert_absent(markup, SCRIPT_ASM);
}

#[test]
fn fee_zero_paints_zero_without_a_comma() {
    let markup = transaction_row_markup(&tx(0, 4));
    assert_one(&markup, &preview_cell("0"));
    assert_one(&markup, "<td class=\"fee\">0</td>");
    assert_absent(&markup, ",");
    assert_scripts_stay_off(&markup);
}

#[test]
fn fee_nine_hundred_ninety_nine_paints_fee_without_a_trailing_comma() {
    let markup = transaction_row_markup(&tx(999, 4));
    assert_one(&markup, &preview_cell("999"));
    assert_one(&markup, "<td class=\"fee\">999</td>");
    assert_absent(&markup, "999,");
    assert_scripts_stay_off(&markup);
}

#[test]
fn fee_fifteen_hundred_paints_grouped_and_keeps_vsize() {
    let markup = transaction_row_markup(&tx(1500, 7));
    assert_one(&markup, &preview_cell("1,500"));
    assert_one(&markup, "<td class=\"fee\">1500</td>");
    assert_one(&markup, "<td class=\"virtual-size\">Virtual size 1</td>");
    assert_one(
        &markup,
        "<td class=\"adjusted-vsize\">Adjusted vsize 1.75</td>",
    );
    assert_scripts_stay_off(&markup);
}

#[test]
fn fee_one_million_paints_grouped() {
    let markup = transaction_row_markup(&tx(1_000_000, 4));
    assert_one(&markup, &preview_cell("1,000,000"));
    assert_one(&markup, "<td class=\"fee\">1000000</td>");
    assert_scripts_stay_off(&markup);
}
