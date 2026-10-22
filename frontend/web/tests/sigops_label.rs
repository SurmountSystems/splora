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

/// Whole line `Sigops 0` from no-query `GET /tx/:txid`.
///
/// Sigops 0 paints that line once. The label cell and the raw integer cell
/// stay. Sigops 1 paints `Sigops 1` once. A value that groups with a comma
/// does not gain a whole line. Adjusted vsize and virtual size stay in their
/// cells and do not become their own lines.
#[test]
fn sigops_zero_paints_sigops_0_as_its_own_line() {
    const LINE: &str = "Sigops 0";
    let markup = transaction_row_markup(&tx(7, Sigops::Present(0)));
    assert!(
        markup.lines().any(|line| line == LINE),
        "{LINE} is absent: {markup}"
    );
    assert_eq!(
        markup.lines().filter(|line| *line == LINE).count(),
        1,
        "{LINE} was not one line: {markup}"
    );
    assert!(
        markup.lines().filter(|line| *line == "Sigops").count() == 1,
        "Sigops was not one line: {markup}"
    );
    assert_one(&markup, LABEL);
    assert_one(&markup, &raw_cell(0));
    assert_one(
        &markup,
        "<td class=\"adjusted-vsize\">Adjusted vsize 1.75</td>",
    );
    assert_one(&markup, "<td class=\"virtual-size\">Virtual size 1</td>");
    assert!(
        !markup.lines().any(|line| line == "Adjusted vsize 1.75"),
        "Adjusted vsize 1.75 was painted as its own line: {markup}"
    );
    assert!(
        !markup.lines().any(|line| line == "virtual size 1"),
        "virtual size 1 was painted as its own line: {markup}"
    );
    assert_scripts_stay_off(&markup);

    let one = transaction_row_markup(&tx(40, Sigops::Present(1)));
    assert!(
        one.lines().any(|line| line == "Sigops 1"),
        "Sigops 1 is absent: {one}"
    );
    assert_eq!(
        one.lines().filter(|line| *line == "Sigops 1").count(),
        1,
        "Sigops 1 was not one line: {one}"
    );
    assert_one(&one, LABEL);
    assert_one(&one, &raw_cell(1));
    assert_scripts_stay_off(&one);

    let grouped = transaction_row_markup(&tx(40, Sigops::Present(1500)));
    assert!(
        !grouped.lines().any(|line| line == "Sigops 1,500"),
        "sigops 1500 painted Sigops 1,500: {grouped}"
    );
    assert!(
        !grouped.lines().any(|line| line == "Sigops 1500"),
        "sigops 1500 painted Sigops 1500: {grouped}"
    );
    assert_one(&grouped, LABEL);
    assert_one(&grouped, &raw_cell(1500));
    assert_scripts_stay_off(&grouped);
}

/// Whole line `Sigops` from no-query `GET /tx/:txid`.
///
/// A present sigops, including explicit 0, paints a line whose entire text
/// is Sigops. The label cell and the raw integer cell stay. `Sigops 0`
/// stays. This calls `transaction_row_markup`. A missing key or JSON null
/// is covered by the omit tests above.
#[test]
fn sigops_present_paints_sigops_as_its_own_line() {
    const LINE: &str = "Sigops";
    let markup = transaction_row_markup(&tx(7, Sigops::Present(0)));
    assert!(
        markup.lines().any(|line| line == LINE),
        "no line equals {LINE}: {markup}"
    );
    assert_eq!(
        markup.lines().filter(|line| *line == LINE).count(),
        1,
        "{LINE} was not painted once: {markup}"
    );
    assert!(
        markup.lines().any(|line| line == "Sigops 0"),
        "Sigops 0 was dropped: {markup}"
    );
    assert_one(&markup, LABEL);
    assert_one(&markup, &raw_cell(0));
    assert_scripts_stay_off(&markup);
}

/// Whole line `Version 0` from no-query `GET /tx/:txid`.
///
/// Version 0 paints that line once. The raw version cell stays the integer.
/// A value that groups with a comma does not gain a whole line. Version 1
/// does not gain a whole line: other transaction-row tests still require
/// the word Version to stay absent when version is 1.
#[test]
fn version_zero_paints_version_0_as_its_own_line() {
    const LINE: &str = "Version 0";
    let body = tx(7, Sigops::Present(0)).replace("\"version\":1", "\"version\":0");
    let markup = transaction_row_markup(&body);
    assert!(
        markup.lines().any(|line| line == LINE),
        "{LINE} is absent: {markup}"
    );
    assert_eq!(
        markup.lines().filter(|line| *line == LINE).count(),
        1,
        "{LINE} was not one line: {markup}"
    );
    assert_one(&markup, "<td class=\"version\">0</td>");
    assert_scripts_stay_off(&markup);

    let one = transaction_row_markup(&tx(40, Sigops::Present(1)));
    assert!(
        !one.lines().any(|line| line == "Version 1"),
        "version 1 painted Version 1: {one}"
    );
    assert_one(&one, "<td class=\"version\">1</td>");
    assert_scripts_stay_off(&one);

    let grouped_body = tx(40, Sigops::Present(1)).replace("\"version\":1", "\"version\":1500");
    let grouped = transaction_row_markup(&grouped_body);
    assert!(
        !grouped.lines().any(|line| line == "Version 1,500"),
        "version 1500 painted Version 1,500: {grouped}"
    );
    assert!(
        !grouped.lines().any(|line| line == "Version 1500"),
        "version 1500 painted Version 1500: {grouped}"
    );
    assert_one(&grouped, "<td class=\"version\">1500</td>");
    assert_scripts_stay_off(&grouped);
}

/// Whole line `Version 2` from no-query `GET /tx/:txid`.
///
/// Version 2 paints that line once, and only when the version text is `2`.
/// The raw version cell stays the integer. Version 0, Version 1, and a
/// comma-grouped version do not gain this line. A version 1 row still has
/// no word Version.
#[test]
fn version_two_paints_version_2_as_its_own_line() {
    const LINE: &str = "Version 2";
    let body = tx(7, Sigops::Present(0)).replace("\"version\":1", "\"version\":2");
    let markup = transaction_row_markup(&body);
    assert!(
        markup.lines().any(|line| line == LINE),
        "{LINE} is absent: {markup}"
    );
    assert_eq!(
        markup.lines().filter(|line| *line == LINE).count(),
        1,
        "{LINE} was not one line: {markup}"
    );
    assert!(
        !markup.lines().any(|line| line == "Version 0"),
        "version 2 painted Version 0: {markup}"
    );
    assert!(
        !markup.lines().any(|line| line == "Version 1"),
        "version 2 painted Version 1: {markup}"
    );
    assert_one(&markup, "<td class=\"version\">2</td>");
    assert_scripts_stay_off(&markup);

    let one = transaction_row_markup(&tx(40, Sigops::Present(1)));
    assert!(
        !one.lines().any(|line| line == "Version 1"),
        "version 1 painted Version 1: {one}"
    );
    assert!(
        !one.contains("Version"),
        "version 1 contains the word Version: {one}"
    );
    assert_one(&one, "<td class=\"version\">1</td>");
    assert_scripts_stay_off(&one);

    let zero_body = tx(7, Sigops::Present(0)).replace("\"version\":1", "\"version\":0");
    let zero = transaction_row_markup(&zero_body);
    assert!(
        zero.lines().any(|line| line == "Version 0"),
        "version 0 dropped Version 0: {zero}"
    );
    assert!(
        !zero.lines().any(|line| line == "Version 2"),
        "version 0 painted Version 2: {zero}"
    );

    let grouped_body = tx(40, Sigops::Present(1)).replace("\"version\":1", "\"version\":1500");
    let grouped = transaction_row_markup(&grouped_body);
    assert!(
        !grouped.lines().any(|line| line == "Version 1,500"),
        "version 1500 painted Version 1,500: {grouped}"
    );
    assert!(
        !grouped.lines().any(|line| line == "Version 1500"),
        "version 1500 painted Version 1500: {grouped}"
    );
    assert!(
        !grouped.contains("Version"),
        "version 1500 contains the word Version: {grouped}"
    );
    assert_one(&grouped, "<td class=\"version\">1500</td>");
    assert_scripts_stay_off(&grouped);
}
