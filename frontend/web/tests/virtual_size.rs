//! Virtual size on the transaction details row from `GET /tx/:txid`.
//!
//! `transaction_row_markup` paints weight divided by 4, integer division.
//! A missing weight fails deserialization and omits the label. Script hex
//! and asm stay off the page.

use splora_web::transaction_row_markup;

const SCRIPT_HEX: &str = "VSIZEPUBHEXDEAD";

fn tx(weight: Option<u32>) -> String {
    let weight_field = match weight {
        Some(weight) => format!(r#","weight":{weight}"#),
        None => String::new(),
    };
    format!(
        r#"{{"txid":"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa","version":1,"locktime":0,"vin":[{{"txid":"bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb","vout":0,"prevout":null,"scriptsig":"VSIZESIGHEXDEAD","scriptsig_asm":"VSIZESIGASMDEAD","is_coinbase":false,"sequence":1}}],"vout":[{{"scriptpubkey":"{SCRIPT_HEX}","scriptpubkey_asm":"VSIZEASMDEAD","scriptpubkey_type":"p2pkh","value":1}}],"size":10{weight_field},"sigops":0,"fee":1}}"#
    )
}

fn cell(virtual_size: u32) -> String {
    format!("<td class=\"virtual-size\">Virtual size {virtual_size}</td>")
}

fn assert_one_cell(markup: &str, virtual_size: u32) {
    let expected = cell(virtual_size);
    assert_eq!(
        markup.matches(&expected).count(),
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
    assert_absent(markup, "VSIZESIGHEXDEAD");
    assert_absent(markup, "VSIZESIGASMDEAD");
    assert_absent(markup, "VSIZEASMDEAD");
}

#[test]
fn virtual_size_is_weight_divided_by_four() {
    let markup = transaction_row_markup(&tx(Some(5)));
    assert_one_cell(&markup, 1);
    assert_absent(&markup, "Virtual size 2");
    assert_scripts_stay_off(&markup);
}

#[test]
fn weight_seven_does_not_round_up() {
    let markup = transaction_row_markup(&tx(Some(7)));
    assert_one_cell(&markup, 1);
    assert_absent(&markup, "Virtual size 2");
    assert_scripts_stay_off(&markup);
}

#[test]
fn weight_zero_paints_virtual_size_zero() {
    let markup = transaction_row_markup(&tx(Some(0)));
    assert_one_cell(&markup, 0);
    assert_scripts_stay_off(&markup);
}

#[test]
fn weight_three_is_zero() {
    let markup = transaction_row_markup(&tx(Some(3)));
    assert_one_cell(&markup, 0);
    assert_absent(&markup, "Virtual size 1");
    assert_scripts_stay_off(&markup);
}

#[test]
fn missing_weight_omits_virtual_size() {
    let markup = transaction_row_markup(&tx(None));
    assert_absent(&markup, "class=\"virtual-size\"");
    assert_absent(&markup, "Virtual size 0");
    assert_scripts_stay_off(&markup);
}
