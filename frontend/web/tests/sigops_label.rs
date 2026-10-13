//! Sigops label on the transaction details row from `GET /tx/:txid`.
//!
//! Angular shows the label Sigops when sigops is not null, including 0.
//! The raw integer cell stays beside it. A missing key or JSON null fails
//! deserialization, so the markup does not contain Sigops. The FAQ info
//! icon and the left-to-right mark stay off the page. Script hex, asm,
//! scriptsig, and witness stay off the page.

use splora_web::transaction_row_markup;

const SCRIPT_HEX: &str = "SIGOPSPUBHEXDEAD";
const SCRIPT_SIG_HEX: &str = "SIGOPSSIGHEXDEAD";
const SCRIPT_SIG_ASM: &str = "SIGOPSSIGASMDEAD";
const SCRIPT_ASM: &str = "SIGOPSASMDEAD";
const WITNESS: &str = "SIGOPSWITDEAD";

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

const LABEL: &str = "<td class=\"sigops-label\">Sigops</td>";

fn raw_cell(sigops: u32) -> String {
    format!("<td class=\"sigops\">{sigops}</td>")
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
    assert_absent(markup, "&lrm;");
    assert_absent(markup, "\u{200e}");
    assert_absent(markup, "info-circle");
    assert_absent(markup, "info-link");
}

#[test]
fn sigops_zero_paints_the_label_and_keeps_adjusted_vsize() {
    let markup = transaction_row_markup(&tx(7, Sigops::Present(0)));
    assert_one(&markup, LABEL);
    assert_one(&markup, &raw_cell(0));
    assert_one(
        &markup,
        "<td class=\"adjusted-vsize\">Adjusted vsize 1.75</td>",
    );
    assert_one(&markup, "<td class=\"virtual-size\">Virtual size 1</td>");
    assert_scripts_stay_off(&markup);
}

#[test]
fn sigops_three_paints_the_label_and_the_raw_integer() {
    let markup = transaction_row_markup(&tx(7, Sigops::Present(3)));
    assert_one(&markup, LABEL);
    assert_one(&markup, &raw_cell(3));
    assert_scripts_stay_off(&markup);
}

#[test]
fn missing_sigops_omits_the_label() {
    let markup = transaction_row_markup(&tx(7, Sigops::Missing));
    assert_absent(&markup, "Sigops");
    assert_absent(&markup, "sigops-label");
    assert_scripts_stay_off(&markup);
}

#[test]
fn null_sigops_omits_the_label() {
    let markup = transaction_row_markup(&tx(7, Sigops::Null));
    assert_absent(&markup, "Sigops");
    assert_absent(&markup, "sigops-label");
    assert_scripts_stay_off(&markup);
}
