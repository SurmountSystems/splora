//! Adjusted vsize on the transaction details row from `GET /tx/:txid`.
//!
//! When sigops is present, adjusted vsize is `max(weight / 4, sigops * 5)`
//! with floating division, then two decimal places. The cell and the whole
//! line are painted only when that text ends in `.00` and the whole part is
//! not `1`. A missing or null sigops omits the row. Explicit zero still
//! paints `0.00`. Script hex, asm, scriptsig, and witness stay off the page.
//! Virtual size stays integer `weight / 4`.

use splora_web::transaction_row_markup;

const SCRIPT_HEX: &str = "ADJVSIZEPUBHEXDEAD";
const SCRIPT_SIG_HEX: &str = "ADJVSIZESIGHEXDEAD";
const SCRIPT_SIG_ASM: &str = "ADJVSIZESIGASMDEAD";
const SCRIPT_ASM: &str = "ADJVSIZEASMDEAD";
const WITNESS: &str = "ADJVSIZEWITDEAD";

enum Sigops {
    Present(u32),
    Missing,
    Null,
}

fn tx(weight: u32, sigops: Sigops) -> String {
    let sigops_field = match sigops {
        Sigops::Present(sigops) => format!(r#","sigops":{sigops}"#),
        Sigops::Missing => String::new(),
        Sigops::Null => r#","sigops":null"#.to_string(),
    };
    format!(
        r#"{{"txid":"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa","version":1,"locktime":0,"vin":[{{"txid":"bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb","vout":0,"prevout":null,"scriptsig":"{SCRIPT_SIG_HEX}","scriptsig_asm":"{SCRIPT_SIG_ASM}","witness":["{WITNESS}"],"is_coinbase":false,"sequence":1}}],"vout":[{{"scriptpubkey":"{SCRIPT_HEX}","scriptpubkey_asm":"{SCRIPT_ASM}","scriptpubkey_type":"p2pkh","value":1}}],"size":10,"weight":{weight}{sigops_field},"fee":1}}"#
    )
}

fn adjusted_cell(text: &str) -> String {
    format!("<td class=\"adjusted-vsize\">Adjusted vsize {text}</td>")
}

fn virtual_cell(virtual_size: u32) -> String {
    format!("<td class=\"virtual-size\">Virtual size {virtual_size}</td>")
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
    assert_absent(markup, WITNESS);
}

#[test]
fn weight_seven_sigops_zero_omits_adjusted_vsize_1_75() {
    let markup = transaction_row_markup(&tx(7, Sigops::Present(0)));
    assert_absent(&markup, &adjusted_cell("1.75"));
    assert!(
        !markup.lines().any(|line| line == "Adjusted vsize 1.75"),
        "Adjusted vsize 1.75 was painted: {markup}"
    );
    assert_one(&markup, &virtual_cell(1));
    assert_absent(&markup, &adjusted_cell("1"));
    assert_scripts_stay_off(&markup);
}

#[test]
fn weight_over_four_larger_paints_one_hundred() {
    let markup = transaction_row_markup(&tx(400, Sigops::Present(1)));
    assert_one(&markup, &adjusted_cell("100.00"));
    assert_one(&markup, &virtual_cell(100));
    assert_absent(&markup, "Adjusted vsize 5");
    assert_scripts_stay_off(&markup);
}

#[test]
fn sigops_times_five_larger_paints_fifteen() {
    let markup = transaction_row_markup(&tx(8, Sigops::Present(3)));
    assert_one(&markup, &adjusted_cell("15.00"));
    assert!(
        markup.lines().any(|line| line == "Adjusted vsize 15.00"),
        "Adjusted vsize 15.00 is absent: {markup}"
    );
    assert_one(&markup, &virtual_cell(2));
    assert_absent(&markup, "Adjusted vsize 2");
    assert_scripts_stay_off(&markup);
}

#[test]
fn explicit_zero_sigops_paints_adjusted_vsize_zero() {
    let markup = transaction_row_markup(&tx(0, Sigops::Present(0)));
    assert_one(&markup, &adjusted_cell("0.00"));
    assert_one(&markup, &virtual_cell(0));
    assert_scripts_stay_off(&markup);
}

#[test]
fn missing_sigops_omits_adjusted_vsize() {
    let markup = transaction_row_markup(&tx(20, Sigops::Missing));
    assert_absent(&markup, "Adjusted vsize");
    assert_absent(&markup, "Adjusted vsize 5");
    assert_absent(&markup, "Adjusted vsize 0");
    assert_scripts_stay_off(&markup);
}

#[test]
fn null_sigops_omits_adjusted_vsize() {
    let markup = transaction_row_markup(&tx(20, Sigops::Null));
    assert_absent(&markup, "Adjusted vsize");
    assert_absent(&markup, "Adjusted vsize 5");
    assert_absent(&markup, "Adjusted vsize 0");
    assert_scripts_stay_off(&markup);
}

/// Whole line `Adjusted vsize 10.00` from weight 20 and sigops 2.
///
/// `max(20 / 4, 2 * 5)` is 10, then two decimal places. The adjusted-vsize
/// cell is that same `10.00` text. The weight side `Adjusted vsize 5.00` is
/// not its own line.
#[test]
fn weight_twenty_sigops_two_paints_adjusted_vsize_10_00_line() {
    const LINE: &str = "Adjusted vsize 10.00";
    let markup = transaction_row_markup(&tx(20, Sigops::Present(2)));
    assert!(
        markup.lines().any(|line| line == LINE),
        "{LINE} is absent: {markup}"
    );
    assert_eq!(
        markup.lines().filter(|line| *line == LINE).count(),
        1,
        "{LINE} was not one line: {markup}"
    );
    assert_one(&markup, &adjusted_cell("10.00"));
    assert_one(&markup, &virtual_cell(5));
    assert!(
        !markup.lines().any(|line| line == "Adjusted vsize 5.00"),
        "weight 20 painted the weight side: {markup}"
    );
    assert_scripts_stay_off(&markup);
}

/// Whole line `Adjusted vsize 15.00` from weight 8 and sigops 3.
///
/// `max(8 / 4, 3 * 5)` is 15, then two decimal places. The adjusted-vsize
/// cell is that same `15.00` text.
#[test]
fn weight_eight_sigops_three_paints_adjusted_vsize_15_00_line() {
    const LINE: &str = "Adjusted vsize 15.00";
    let markup = transaction_row_markup(&tx(8, Sigops::Present(3)));
    assert!(
        markup.lines().any(|line| line == LINE),
        "{LINE} is absent: {markup}"
    );
    assert_eq!(
        markup.lines().filter(|line| *line == LINE).count(),
        1,
        "{LINE} was not one line: {markup}"
    );
    assert_one(&markup, &adjusted_cell("15.00"));
    assert_one(&markup, &virtual_cell(2));
    assert_absent(&markup, "Adjusted vsize 2");
    assert_scripts_stay_off(&markup);
}
