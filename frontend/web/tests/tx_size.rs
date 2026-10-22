//! Transaction size on the details row from `GET /tx/:txid`.
//!
//! Angular paints `tx.size | bytes: 2`: decimal 2, from `B`, plaintext false.
//! Thresholds are powers of 1000. The cell label is Size. The left-to-right
//! mark stays off the page. Script hex and asm stay off the page.

use splora_web::transaction_row_markup;

const SCRIPT_HEX: &str = "TXSIZEPUBHEXDEAD";
const SCRIPT_SIG_HEX: &str = "TXSIZESIGHEXDEAD";
const SCRIPT_SIG_ASM: &str = "TXSIZESIGASMDEAD";
const SCRIPT_ASM: &str = "TXSIZEASMDEAD";

fn tx(size: u32, weight: u32) -> String {
    format!(
        r#"{{"txid":"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa","version":1,"locktime":0,"vin":[{{"txid":"bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb","vout":0,"prevout":null,"scriptsig":"{SCRIPT_SIG_HEX}","scriptsig_asm":"{SCRIPT_SIG_ASM}","is_coinbase":false,"sequence":1}}],"vout":[{{"scriptpubkey":"{SCRIPT_HEX}","scriptpubkey_asm":"{SCRIPT_ASM}","scriptpubkey_type":"p2pkh","value":1}}],"size":{size},"weight":{weight},"sigops":0,"fee":1}}"#
    )
}

fn size_cell(inner: &str) -> String {
    format!("<td class=\"size\">{inner}</td>")
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
    assert_absent(markup, "&lrm;");
    assert_absent(markup, "\u{200e}");
}

#[test]
fn zero_size_paints_zero_bytes() {
    let markup = transaction_row_markup(&tx(0, 4));
    assert_one(
        &markup,
        &size_cell("Size 0 <span class=\"symbol\">B</span>"),
    );
    assert_scripts_stay_off(&markup);
}

#[test]
fn ten_bytes_stays_bytes() {
    let markup = transaction_row_markup(&tx(10, 7));
    assert_one(
        &markup,
        &size_cell("Size 10 <span class=\"symbol\">B</span>"),
    );
    assert_absent(&markup, "Size 10 <span class=\"symbol\">kB</span>");
    assert_absent(&markup, "Size 10 kB");
    assert_one(&markup, "<td class=\"virtual-size\">Virtual size 1</td>");
    assert_one(
        &markup,
        "<td class=\"adjusted-vsize\">Adjusted vsize 1.75</td>",
    );
    assert_scripts_stay_off(&markup);
}

#[test]
fn nine_hundred_ninety_nine_stays_bytes() {
    let markup = transaction_row_markup(&tx(999, 4));
    assert_one(
        &markup,
        &size_cell("Size 999 <span class=\"symbol\">B</span>"),
    );
    assert_scripts_stay_off(&markup);
}

#[test]
fn one_thousand_is_one_kilobyte() {
    let markup = transaction_row_markup(&tx(1000, 4));
    assert_one(
        &markup,
        &size_cell("Size 1 <span class=\"symbol\">kB</span>"),
    );
    assert_absent(&markup, "1000 <span class=\"symbol\">B</span>");
    assert_absent(&markup, "1000 B");
    assert_absent(&markup, "<td class=\"size\">1000</td>");
    assert_scripts_stay_off(&markup);
}

#[test]
fn fifteen_hundred_is_one_point_five_kilobytes() {
    let markup = transaction_row_markup(&tx(1500, 4));
    assert_one(
        &markup,
        &size_cell("Size 1.5 <span class=\"symbol\">kB</span>"),
    );
    assert_scripts_stay_off(&markup);
}

#[test]
fn fourteen_ninety_nine_rounds_to_one_point_five_kilobytes() {
    let markup = transaction_row_markup(&tx(1499, 4));
    assert_one(
        &markup,
        &size_cell("Size 1.5 <span class=\"symbol\">kB</span>"),
    );
    assert_scripts_stay_off(&markup);
}

#[test]
fn one_point_five_megabytes() {
    let markup = transaction_row_markup(&tx(1_500_000, 4));
    assert_one(
        &markup,
        &size_cell("Size 1.5 <span class=\"symbol\">MB</span>"),
    );
    assert_scripts_stay_off(&markup);
}

#[test]
fn megabytes_round_to_two_decimals() {
    let markup = transaction_row_markup(&tx(1_234_567, 4));
    assert_one(
        &markup,
        &size_cell("Size 1.23 <span class=\"symbol\">MB</span>"),
    );
    assert_scripts_stay_off(&markup);
}

#[test]
fn two_point_five_gigabytes() {
    let markup = transaction_row_markup(&tx(2_500_000_000, 4));
    assert_one(
        &markup,
        &size_cell("Size 2.5 <span class=\"symbol\">GB</span>"),
    );
    assert_scripts_stay_off(&markup);
}

#[test]
fn one_thousand_five_bytes_matches_float_round() {
    let markup = transaction_row_markup(&tx(1005, 4));
    assert_one(
        &markup,
        &size_cell("Size 1 <span class=\"symbol\">kB</span>"),
    );
    assert_absent(&markup, "1.01");
    assert_scripts_stay_off(&markup);
}

/// Whole line `Size 1.5 kB` from no-query `GET /tx/:txid`.
///
/// Size 1500 paints that line once. The Size cell keeps the number and the
/// unit apart. Size 1000 does not paint this line. The decimal sequence span
/// and the `nSequence 0x00` span stay. `nSequence 0x00` is not its own line.
#[test]
fn size_1500_paints_size_1_5_kb_as_its_own_line() {
    const LINE: &str = "Size 1.5 kB";
    let markup = transaction_row_markup(&tx(1500, 4));
    assert!(
        markup.lines().any(|line| line == LINE),
        "{LINE} is absent: {markup}"
    );
    assert_eq!(
        markup.lines().filter(|line| *line == LINE).count(),
        1,
        "{LINE} was not one line: {markup}"
    );
    assert_one(
        &markup,
        &size_cell("Size 1.5 <span class=\"symbol\">kB</span>"),
    );
    assert!(
        !markup.lines().any(|line| line == "Size 1 kB"),
        "size 1500 painted Size 1 kB: {markup}"
    );
    assert_one(&markup, "<span class=\"sequence\">sequence 1</span>");
    assert_one(&markup, "<td class=\"virtual-size\">Virtual size 1</td>");
    assert_scripts_stay_off(&markup);

    let zero_seq = transaction_row_markup(&tx(1500, 4).replace("\"sequence\":1", "\"sequence\":0"));
    assert!(
        zero_seq.lines().any(|line| line == LINE),
        "{LINE} is absent when sequence is 0: {zero_seq}"
    );
    assert_one(&zero_seq, "<span class=\"sequence\">sequence 0</span>");
    assert_one(
        &zero_seq,
        "<span class=\"sequence\">sequence 0</span><span class=\"nsequence\">nSequence 0x00</span>",
    );
    assert!(
        zero_seq.lines().any(|line| line == "sequence 0"),
        "sequence 0 line was dropped: {zero_seq}"
    );
    assert!(
        !zero_seq.lines().any(|line| line == "nSequence 0x00"),
        "nSequence 0x00 was painted as its own line: {zero_seq}"
    );
    assert_scripts_stay_off(&zero_seq);
}
