//! Fee rate label on the transaction row from `GET /tx/:txid`.
//!
//! Angular paints the label Fee rate beside `tx.fee / (tx.weight / 4)`.
//! The value cell stays the existing rate. Weight under 4 has virtual
//! size 0, so the rate and the label stay off. A missing fee key or JSON
//! null fails the transaction parse and does not paint Fee rate. Script
//! hex, asm, scriptsig, and witness stay off the page.

use splora_web::transaction_row_markup;

const SCRIPT_HEX: &str = "FEERATEPUBHEXDEAD";
const SCRIPT_SIG_HEX: &str = "FEERATESIGHEXDEAD";
const SCRIPT_SIG_ASM: &str = "FEERATESIGASMDEAD";
const SCRIPT_ASM: &str = "FEERATEASMDEAD";
const WITNESS: &str = "FEERATEWITDEAD";

const LABEL: &str = "<td class=\"fee-rate-label\">Fee rate</td>";

fn tx(fee_field: &str, weight: u32) -> String {
    format!(
        r#"{{"txid":"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa","version":1,"locktime":0,"vin":[{{"txid":"bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb","vout":0,"prevout":null,"scriptsig":"{SCRIPT_SIG_HEX}","scriptsig_asm":"{SCRIPT_SIG_ASM}","witness":["{WITNESS}"],"is_coinbase":false,"sequence":1}}],"vout":[{{"scriptpubkey":"{SCRIPT_HEX}","scriptpubkey_asm":"{SCRIPT_ASM}","scriptpubkey_type":"p2pkh","value":1}}],"size":10,"weight":{weight},"sigops":0{fee_field}}}"#
    )
}

fn rate_text(fee: u64, weight: u32) -> String {
    let virtual_size = weight / 4;
    format!("{}", fee as f64 / virtual_size as f64)
}

fn rate_cell(fee: u64, weight: u32) -> String {
    format!("<td class=\"fee-rate\">{}</td>", rate_text(fee, weight))
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
    assert_absent(markup, "Effective fee rate");
    assert_absent(markup, "Allowed?");
    assert_absent(markup, "Transaction hex");
    assert_absent(markup, "sat/vB");
}

#[test]
fn finite_rate_paints_the_label_beside_the_number() {
    let markup = transaction_row_markup(&tx(r#","fee":584"#, 572));
    let number = rate_cell(584, 572);
    assert_one(&markup, LABEL);
    assert_one(&markup, &number);
    assert_eq!(markup.matches("Fee rate").count(), 1, "{markup}");
    assert_absent(&number, "Fee rate");
    assert_scripts_stay_off(&markup);
}

#[test]
fn fee_zero_still_paints_the_label() {
    let markup = transaction_row_markup(&tx(r#","fee":0"#, 4));
    assert_one(&markup, LABEL);
    assert_one(&markup, &rate_cell(0, 4));
    assert_eq!(markup.matches("Fee rate").count(), 1, "{markup}");
    assert_scripts_stay_off(&markup);
}

#[test]
fn weight_zero_omits_the_label() {
    let markup = transaction_row_markup(&tx(r#","fee":5"#, 0));
    assert!(markup.contains("<tr class=\"tx-row\">"), "{markup}");
    assert_absent(&markup, "Fee rate");
    assert_absent(&markup, "fee-rate");
    assert_scripts_stay_off(&markup);
}

#[test]
fn weight_under_four_omits_the_label() {
    let markup = transaction_row_markup(&tx(r#","fee":5"#, 3));
    assert!(markup.contains("<tr class=\"tx-row\">"), "{markup}");
    assert_absent(&markup, "Fee rate");
    assert_absent(&markup, "fee-rate");
    assert_scripts_stay_off(&markup);
}

#[test]
fn missing_fee_key_omits_the_label() {
    let markup = transaction_row_markup(&tx("", 572));
    assert_absent(&markup, "Fee rate");
    assert_absent(&markup, "fee-rate");
    assert_absent(&markup, "<tr class=\"tx-row\">");
    assert_scripts_stay_off(&markup);
}

#[test]
fn null_fee_omits_the_label() {
    let markup = transaction_row_markup(&tx(r#","fee":null"#, 572));
    assert_absent(&markup, "Fee rate");
    assert_absent(&markup, "fee-rate");
    assert_absent(&markup, "<tr class=\"tx-row\">");
    assert_scripts_stay_off(&markup);
}
