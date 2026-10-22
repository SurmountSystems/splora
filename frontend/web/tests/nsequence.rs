//! nSequence hex on the transaction input from `GET /tx/:txid`.
//!
//! Angular paints the label text `nSequence` and `formatHex(vin.sequence)`:
//! lowercase hex, with one leading zero after `0x` when the digit count is
//! odd. The decimal sequence span stays immediately before it. Explicit 0
//! is `0x00`. A missing sequence key fails input parse, so the markup is
//! empty and does not contain nSequence. Script hex, script asm, scriptsig,
//! and witness stay off the page. No RBF text and no Version label.

use splora_web::transaction_row_markup;

const SCRIPT_HEX: &str = "NSEQPUBHEXDEAD";
const SCRIPT_SIG_HEX: &str = "NSEQSIGHEXDEAD";
const SCRIPT_SIG_ASM: &str = "NSEQSIGASMDEAD";
const SCRIPT_ASM: &str = "NSEQASMDEAD";
const WITNESS: &str = "NSEQWITDEAD";

fn tx(sequence: Option<u32>) -> String {
    let sequence_field = match sequence {
        Some(value) => format!(r#","sequence":{value}"#),
        None => String::new(),
    };
    format!(
        r#"{{"txid":"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa","version":1,"locktime":0,"vin":[{{"txid":"bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb","vout":0,"prevout":null,"scriptsig":"{SCRIPT_SIG_HEX}","scriptsig_asm":"{SCRIPT_SIG_ASM}","witness":["{WITNESS}"],"is_coinbase":false{sequence_field}}}],"vout":[{{"scriptpubkey":"{SCRIPT_HEX}","scriptpubkey_asm":"{SCRIPT_ASM}","scriptpubkey_type":"p2pkh","value":1}}],"size":10,"weight":7,"sigops":0,"fee":1}}"#
    )
}

fn sequence_spans(sequence: u32, hex: &str) -> String {
    format!(
        "<span class=\"sequence\">sequence {sequence}</span><span class=\"nsequence\">nSequence {hex}</span>"
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

fn assert_fixtures_stay_off(markup: &str) {
    assert_absent(markup, SCRIPT_HEX);
    assert_absent(markup, SCRIPT_SIG_HEX);
    assert_absent(markup, SCRIPT_SIG_ASM);
    assert_absent(markup, SCRIPT_ASM);
    assert_absent(markup, WITNESS);
    assert_absent(markup, "RBF");
    assert_absent(markup, "Version");
}

#[test]
fn sequence_zero_paints_nsequence_hex_and_keeps_the_decimal() {
    let markup = transaction_row_markup(&tx(Some(0)));
    assert_one(&markup, &sequence_spans(0, "0x00"));
    assert_one(&markup, "<span class=\"sequence\">sequence 0</span>");
    assert_one(&markup, "<td class=\"virtual-size\">Virtual size 1</td>");
    assert_one(
        &markup,
        "<td class=\"adjusted-vsize\">Adjusted vsize 1.75</td>",
    );
    assert_fixtures_stay_off(&markup);
}

#[test]
fn sequence_256_paints_the_odd_length_pad() {
    let markup = transaction_row_markup(&tx(Some(256)));
    assert_one(&markup, &sequence_spans(256, "0x0100"));
    assert_fixtures_stay_off(&markup);
}

#[test]
fn sequence_u32_max_paints_nsequence_hex_and_keeps_the_decimal() {
    let markup = transaction_row_markup(&tx(Some(u32::MAX)));
    assert_one(&markup, &sequence_spans(u32::MAX, "0xffffffff"));
    assert_one(
        &markup,
        "<span class=\"sequence\">sequence 4294967295</span>",
    );
    assert_fixtures_stay_off(&markup);
}

#[test]
fn missing_sequence_key_omits_nsequence() {
    let markup = transaction_row_markup(&tx(None));
    assert!(
        markup.is_empty(),
        "a missing sequence key painted markup: {markup}"
    );
    assert_absent(&markup, "nSequence");
    assert_fixtures_stay_off(&markup);
}

/// Whole line `sequence 0` from no-query `GET /tx/:txid`.
///
/// A present sequence paints the decimal as its own line, including zero.
/// The decimal span and the `nSequence` hex span stay. The hex line is not
/// this line. A missing sequence key still fails input parse.
#[test]
fn sequence_zero_paints_sequence_0_as_its_own_line() {
    const LINE: &str = "sequence 0";
    let markup = transaction_row_markup(&tx(Some(0)));
    assert!(
        markup.lines().any(|line| line == LINE),
        "{LINE} is absent: {markup}"
    );
    assert_eq!(
        markup.lines().filter(|line| *line == LINE).count(),
        1,
        "{LINE} was not one line: {markup}"
    );
    assert_one(&markup, "<span class=\"sequence\">sequence 0</span>");
    assert_one(&markup, &sequence_spans(0, "0x00"));
    assert!(
        !markup.lines().any(|line| line == "nSequence 0x00"),
        "nSequence 0x00 was painted as its own line: {markup}"
    );
    assert_fixtures_stay_off(&markup);

    let padded = transaction_row_markup(&tx(Some(256)));
    assert!(
        padded.lines().any(|line| line == "sequence 256"),
        "sequence 256 is absent: {padded}"
    );
    assert_eq!(
        padded
            .lines()
            .filter(|line| *line == "sequence 256")
            .count(),
        1,
        "sequence 256 was not one line: {padded}"
    );
    assert_one(&padded, &sequence_spans(256, "0x0100"));

    let max = transaction_row_markup(&tx(Some(u32::MAX)));
    assert!(
        max.lines().any(|line| line == "sequence 4294967295"),
        "sequence 4294967295 is absent: {max}"
    );
    assert_eq!(
        max.lines()
            .filter(|line| *line == "sequence 4294967295")
            .count(),
        1,
        "sequence 4294967295 was not one line: {max}"
    );
    assert_one(&max, "<span class=\"sequence\">sequence 4294967295</span>");
}
