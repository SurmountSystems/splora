//! Locktime label on the transaction details row from `GET /tx/:txid`.
//!
//! Angular shows the label Locktime whenever the transaction is shown,
//! including locktime 0. There is no omit when the key is missing, and a
//! missing key is not this test. The value cell stays the raw integer,
//! with no thousands separator and no left-to-right mark. Script hex, asm,
//! scriptsig, and witness stay off the page. No Version label.

use splora_web::transaction_row_markup;

const SCRIPT_HEX: &str = "LOCKTIMEPUBHEXDEAD";
const SCRIPT_SIG_HEX: &str = "LOCKTIMESIGHEXDEAD";
const SCRIPT_SIG_ASM: &str = "LOCKTIMESIGASMDEAD";
const SCRIPT_ASM: &str = "LOCKTIMEASMDEAD";
const WITNESS: &str = "LOCKTIMEWITDEAD";

fn tx(locktime: u32) -> String {
    format!(
        r#"{{"txid":"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa","version":1,"locktime":{locktime},"vin":[{{"txid":"bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb","vout":0,"prevout":null,"scriptsig":"{SCRIPT_SIG_HEX}","scriptsig_asm":"{SCRIPT_SIG_ASM}","witness":["{WITNESS}"],"is_coinbase":false,"sequence":1}}],"vout":[{{"scriptpubkey":"{SCRIPT_HEX}","scriptpubkey_asm":"{SCRIPT_ASM}","scriptpubkey_type":"p2pkh","value":1}}],"size":10,"weight":7,"sigops":0,"fee":1}}"#
    )
}

const LABEL: &str = "<td class=\"locktime-label\">Locktime</td>";

fn raw_cell(locktime: u32) -> String {
    format!("<td class=\"locktime\">{locktime}</td>")
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
    assert_absent(markup, "version-label");
}

#[test]
fn locktime_zero_paints_the_label_and_zero() {
    let markup = transaction_row_markup(&tx(0));
    assert_one(&markup, LABEL);
    assert_one(&markup, &raw_cell(0));
    assert_one(&markup, "<td class=\"virtual-size\">Virtual size 1</td>");
    assert_absent(
        &markup,
        "<td class=\"adjusted-vsize\">Adjusted vsize 1.75</td>",
    );
    assert!(
        !markup.lines().any(|line| line == "Adjusted vsize 1.75"),
        "Adjusted vsize 1.75 was painted: {markup}"
    );
    assert_scripts_stay_off(&markup);
}

#[test]
fn locktime_five_hundred_million_paints_the_label_and_the_raw_integer() {
    let markup = transaction_row_markup(&tx(500_000_000));
    assert_one(&markup, LABEL);
    assert_one(&markup, &raw_cell(500_000_000));
    assert_absent(&markup, "500,000,000");
    assert_scripts_stay_off(&markup);
}
