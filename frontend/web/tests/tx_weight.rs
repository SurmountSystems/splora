//! Transaction weight units on the details row from `GET /tx/:txid`.
//!
//! Angular `tx.weight | wuBytes: 2` picks the unit from the raw weight, scale
//! 1000, then rounds half away from zero on the hundredths. The painted line
//! is plaintext `Weight {number} {unit}`. The raw weight cell stays. A missing
//! or null weight fails the transaction parse and omits the row. Script hex
//! and asm stay off the page.

use splora_web::transaction_row_markup;

const SCRIPT_HEX: &str = "TXWEIGHTPUBHEXDEAD";
const SCRIPT_SIG_HEX: &str = "TXWEIGHTSIGHEXDEAD";
const SCRIPT_SIG_ASM: &str = "TXWEIGHTSIGASMDEAD";
const SCRIPT_ASM: &str = "TXWEIGHTASMDEAD";

enum Weight {
    Present(u32),
    Missing,
    Null,
}

fn tx(weight: Weight) -> String {
    let weight_field = match weight {
        Weight::Present(weight) => format!(r#","weight":{weight}"#),
        Weight::Missing => String::new(),
        Weight::Null => r#","weight":null"#.to_string(),
    };
    format!(
        r#"{{"txid":"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa","version":1,"locktime":0,"vin":[{{"txid":"bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb","vout":0,"prevout":null,"scriptsig":"{SCRIPT_SIG_HEX}","scriptsig_asm":"{SCRIPT_SIG_ASM}","is_coinbase":false,"sequence":1}}],"vout":[{{"scriptpubkey":"{SCRIPT_HEX}","scriptpubkey_asm":"{SCRIPT_ASM}","scriptpubkey_type":"p2pkh","value":1}}],"size":10{weight_field},"sigops":0,"fee":1}}"#
    )
}

fn units_cell(text: &str) -> String {
    format!("<td class=\"weight-units\">{text}</td>")
}

fn raw_cell(weight: u32) -> String {
    format!("<td class=\"weight\">{weight}</td>")
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

fn assert_weight_row_omitted(markup: &str) {
    assert_absent(markup, "Weight");
    assert_absent(markup, "Weight 0 WU");
    assert_absent(markup, "class=\"weight-units\"");
    assert_absent(markup, "class=\"weight\"");
    assert_absent(markup, "Virtual size");
    assert_absent(markup, "class=\"virtual-size\"");
    assert_scripts_stay_off(markup);
}

#[test]
fn weight_fifteen_hundred_paints_one_point_five_kwu() {
    let markup = transaction_row_markup(&tx(Weight::Present(1500)));
    assert_one(&markup, &units_cell("Weight 1.5 kWU"));
    assert_absent(&markup, "Weight 1 kWU");
    assert_one(&markup, &raw_cell(1500));
    assert_scripts_stay_off(&markup);
}

#[test]
fn explicit_zero_weight_paints_zero_wu() {
    let markup = transaction_row_markup(&tx(Weight::Present(0)));
    assert_one(&markup, &units_cell("Weight 0 WU"));
    assert_one(&markup, &raw_cell(0));
    assert_one(&markup, "<td class=\"virtual-size\">Virtual size 0</td>");
    assert_scripts_stay_off(&markup);
}

#[test]
fn weight_seven_keeps_virtual_size_and_adjusted_vsize() {
    let markup = transaction_row_markup(&tx(Weight::Present(7)));
    assert_one(&markup, &units_cell("Weight 7 WU"));
    assert_one(&markup, "<td class=\"virtual-size\">Virtual size 1</td>");
    assert_one(
        &markup,
        "<td class=\"adjusted-vsize\">Adjusted vsize 1.75</td>",
    );
    assert_scripts_stay_off(&markup);
}

#[test]
fn weight_one_thousand_is_one_kwu() {
    let markup = transaction_row_markup(&tx(Weight::Present(1000)));
    assert_one(&markup, &units_cell("Weight 1 kWU"));
    assert_one(&markup, &raw_cell(1000));
    assert_scripts_stay_off(&markup);
}

#[test]
fn weight_one_thousand_twenty_four_is_one_point_zero_two_kwu() {
    let markup = transaction_row_markup(&tx(Weight::Present(1024)));
    assert_one(&markup, &units_cell("Weight 1.02 kWU"));
    assert_absent(&markup, "Weight 1024 WU");
    assert_one(&markup, &raw_cell(1024));
    assert_scripts_stay_off(&markup);
}

#[test]
fn weight_just_under_one_million_stays_kwu() {
    let markup = transaction_row_markup(&tx(Weight::Present(999_999)));
    assert_one(&markup, &units_cell("Weight 1000 kWU"));
    assert_absent(&markup, "Weight 1 MWU");
    assert_one(&markup, &raw_cell(999_999));
    assert_scripts_stay_off(&markup);
}

#[test]
fn weight_one_million_is_one_mwu() {
    let markup = transaction_row_markup(&tx(Weight::Present(1_000_000)));
    assert_one(&markup, &units_cell("Weight 1 MWU"));
    assert_absent(&markup, "Weight 1000 kWU");
    assert_one(&markup, &raw_cell(1_000_000));
    assert_scripts_stay_off(&markup);
}

#[test]
fn weight_two_point_five_gwu() {
    let markup = transaction_row_markup(&tx(Weight::Present(2_500_000_000)));
    assert_one(&markup, &units_cell("Weight 2.5 GWU"));
    assert_one(&markup, &raw_cell(2_500_000_000));
    assert_scripts_stay_off(&markup);
}

#[test]
fn missing_weight_omits_weight_row() {
    let markup = transaction_row_markup(&tx(Weight::Missing));
    assert_weight_row_omitted(&markup);
}

#[test]
fn null_weight_omits_weight_row() {
    let markup = transaction_row_markup(&tx(Weight::Null));
    assert_weight_row_omitted(&markup);
}
