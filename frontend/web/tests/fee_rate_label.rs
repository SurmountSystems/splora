//! Fee rate label on the transaction row from `GET /tx/:txid`.
//!
//! Angular paints the label Fee rate beside `tx.fee / (tx.weight / 4)`.
//! The value cell stays the existing rate. A finite rate paints the bare
//! whole line `Fee rate` and does not paint `sat/vB`. Weight under 4 has
//! virtual size 0, so the rate and the label stay off. A missing fee key
//! or JSON null fails the transaction parse and does not paint Fee rate.
//! Script hex, asm, scriptsig, and witness stay off the page.

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

fn assert_sat_per_vb_stays_off(markup: &str) {
    assert_absent(markup, "sat/vB");
}

#[test]
fn finite_rate_paints_the_label_beside_the_number() {
    let markup = transaction_row_markup(&tx(r#","fee":584"#, 572));
    let number = rate_cell(584, 572);
    assert_absent(&markup, "sat/vB");
    assert_one(&markup, LABEL);
    assert_one(&markup, &number);
    assert!(
        markup.lines().any(|line| line == "Fee rate"),
        "bare Fee rate was dropped: {markup}"
    );
    // The label cell and the bare line each contain the words.
    assert_eq!(markup.matches("Fee rate").count(), 2, "{markup}");
    assert_absent(&number, "Fee rate");
    assert_scripts_stay_off(&markup);
}

#[test]
fn fee_zero_still_paints_the_label() {
    let markup = transaction_row_markup(&tx(r#","fee":0"#, 4));
    assert_absent(&markup, "sat/vB");
    assert_one(&markup, LABEL);
    assert_one(&markup, &rate_cell(0, 4));
    assert!(
        markup.lines().any(|line| line == "Fee rate"),
        "bare Fee rate was dropped: {markup}"
    );
    // The label cell and the bare line each contain the words.
    assert_eq!(markup.matches("Fee rate").count(), 2, "{markup}");
    assert_scripts_stay_off(&markup);
}

#[test]
fn weight_zero_omits_the_label() {
    let markup = transaction_row_markup(&tx(r#","fee":5"#, 0));
    assert!(markup.contains("<tr class=\"tx-row\">"), "{markup}");
    assert_absent(&markup, "Fee rate");
    assert_absent(&markup, "fee-rate");
    assert_sat_per_vb_stays_off(&markup);
    assert_scripts_stay_off(&markup);
}

#[test]
fn weight_under_four_omits_the_label() {
    let markup = transaction_row_markup(&tx(r#","fee":5"#, 3));
    assert!(markup.contains("<tr class=\"tx-row\">"), "{markup}");
    assert_absent(&markup, "Fee rate");
    assert_absent(&markup, "fee-rate");
    assert_sat_per_vb_stays_off(&markup);
    assert_scripts_stay_off(&markup);
}

#[test]
fn missing_fee_key_omits_the_label() {
    let markup = transaction_row_markup(&tx("", 572));
    assert_absent(&markup, "Fee rate");
    assert_absent(&markup, "fee-rate");
    assert_absent(&markup, "<tr class=\"tx-row\">");
    assert_sat_per_vb_stays_off(&markup);
    assert_scripts_stay_off(&markup);
}

#[test]
fn null_fee_omits_the_label() {
    let markup = transaction_row_markup(&tx(r#","fee":null"#, 572));
    assert_absent(&markup, "Fee rate");
    assert_absent(&markup, "fee-rate");
    assert_absent(&markup, "<tr class=\"tx-row\">");
    assert_sat_per_vb_stays_off(&markup);
    assert_scripts_stay_off(&markup);
}

/// Whole line `Fee rate` from no-query `GET /tx/:txid`.
///
/// The label cell stays. This line is the label text alone.
#[test]
fn fee_rate_paints_fee_rate_as_its_own_line() {
    const LINE: &str = "Fee rate";
    let markup = transaction_row_markup(&tx(r#","fee":584"#, 572));
    assert!(
        markup.lines().any(|line| line == LINE),
        "Fee rate is absent: {markup}"
    );
    assert_eq!(
        markup.lines().filter(|line| *line == LINE).count(),
        1,
        "Fee rate was not painted once: {markup}"
    );
    assert_absent(&markup, "sat/vB");
}

/// Fee 1 and weight 7 is a finite rate. The bare line `Fee rate` stays.
/// `sat/vB` stays off.
#[test]
fn gpui_whole_line_fee_rate_0_57_sat_per_vb() {
    let markup = transaction_row_markup(&tx(r#","fee":1"#, 7));
    assert_absent(&markup, "sat/vB");
    assert!(
        markup.lines().any(|line| line == "Fee rate"),
        "bare Fee rate was dropped: {markup}"
    );
}

/// Fee 15 and weight 4 is a finite rate. The bare line `Fee rate` stays.
/// `sat/vB` stays off.
#[test]
fn gpui_whole_line_fee_rate_15_0_sat_per_vb() {
    let markup = transaction_row_markup(&tx(r#","fee":15"#, 4));
    assert_absent(&markup, "sat/vB");
    assert!(
        markup.lines().any(|line| line == "Fee rate"),
        "bare Fee rate was dropped: {markup}"
    );
}

/// Fee 1500 and weight 4 is a finite rate. The bare line `Fee rate` stays.
/// `sat/vB` stays off.
#[test]
fn gpui_whole_line_fee_rate_1_500_sat_per_vb() {
    let markup = transaction_row_markup(&tx(r#","fee":1500"#, 4));
    assert_absent(&markup, "sat/vB");
    assert!(
        markup.lines().any(|line| line == "Fee rate"),
        "bare Fee rate was dropped: {markup}"
    );
}
