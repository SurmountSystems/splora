//! Timestamp label on the transaction details row from `GET /tx/:txid`.
//!
//! Angular `transaction-details.component.html` paints the label Timestamp
//! beside `tx.status.block_time` when that time is present. The value cell
//! stays the existing raw integer. Explicit 0 still paints the label. A
//! missing `block_time` key or JSON null omits the label and does not invent
//! a zero. The block list row keeps its raw `time` cell and does not gain
//! this label. No formatted date, no Transaction hex, and no Version label.

use splora_web::{block_rows_markup, transaction_row_markup};

const SCRIPT_HEX: &str = "TIMEPUBHEXDEAD";
const SCRIPT_SIG_HEX: &str = "TIMESIGHEXDEAD";
const SCRIPT_SIG_ASM: &str = "TIMESIGASMDEAD";
const SCRIPT_ASM: &str = "TIMEASMDEAD";
const WITNESS: &str = "TIMEWITDEAD";
const BLOCK_ID: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";

const LABEL: &str = "<td class=\"timestamp-label\">Timestamp</td>";

fn status_field(status: &str) -> String {
    if status.is_empty() {
        String::new()
    } else {
        format!(r#","status":{status}"#)
    }
}

fn tx(status: &str) -> String {
    let status_field = status_field(status);
    format!(
        r#"{{"txid":"bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb","version":1,"locktime":0,"vin":[{{"txid":"cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc","vout":0,"prevout":null,"scriptsig":"{SCRIPT_SIG_HEX}","scriptsig_asm":"{SCRIPT_SIG_ASM}","witness":["{WITNESS}"],"is_coinbase":false,"sequence":1}}],"vout":[{{"scriptpubkey":"{SCRIPT_HEX}","scriptpubkey_asm":"{SCRIPT_ASM}","scriptpubkey_type":"p2pkh","value":1}}],"size":10,"weight":7,"sigops":0,"fee":1{status_field}}}"#
    )
}

fn raw_cell(block_time: u64) -> String {
    format!("<td class=\"block-time\">{block_time}</td>")
}

fn pair(block_time: u64) -> String {
    format!("{LABEL}{}", raw_cell(block_time))
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

/// The timestamp label cell contains the word once. A whole line is separate.
fn assert_label_word_once(markup: &str) {
    const OPEN: &str = "<td class=\"timestamp-label\">";
    let start = markup
        .find(OPEN)
        .unwrap_or_else(|| panic!("timestamp label missing: {markup}"));
    let rest = &markup[start + OPEN.len()..];
    let end = rest
        .find("</td>")
        .unwrap_or_else(|| panic!("timestamp label unclosed: {markup}"));
    assert_eq!(
        rest[..end].matches("Timestamp").count(),
        1,
        "the label cell does not contain the word once: {markup}"
    );
}

fn assert_timestamp_line(markup: &str) {
    assert!(
        markup.lines().any(|line| line == "Timestamp"),
        "no line equals Timestamp: {markup}"
    );
    assert_eq!(
        markup.lines().filter(|line| *line == "Timestamp").count(),
        1,
        "Timestamp was not one line: {markup}"
    );
}

fn assert_scripts_stay_off(markup: &str) {
    assert_absent(markup, SCRIPT_HEX);
    assert_absent(markup, SCRIPT_SIG_HEX);
    assert_absent(markup, SCRIPT_SIG_ASM);
    assert_absent(markup, SCRIPT_ASM);
    assert_absent(markup, WITNESS);
    assert_absent(markup, "Transaction hex");
    assert_absent(markup, "Version");
    assert_absent(markup, "&lrm;");
    assert_absent(markup, "\u{200e}");
    assert_absent(markup, ":");
    assert_absent(markup, "Miner");
    assert_absent(markup, "Pool");
    assert_absent(markup, "Health");
    assert_absent(markup, "Reward");
}

/// Confirmed block time paints the label once, immediately before the raw integer.
#[test]
fn confirmed_block_time_paints_the_label_beside_the_integer() {
    let markup = transaction_row_markup(&tx(
        r#"{"confirmed":true,"block_height":100,"block_hash":"dddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddd","block_time":1600000000}"#,
    ));
    assert_one(&markup, &pair(1_600_000_000));
    assert_one(&markup, LABEL);
    assert_one(&markup, &raw_cell(1_600_000_000));
    assert_label_word_once(&markup);
    assert_timestamp_line(&markup);
    assert_absent(&raw_cell(1_600_000_000), "Timestamp");
    assert_absent(&markup, "1,600,000,000");
    assert_one(&markup, "<td class=\"virtual-size\">Virtual size 1</td>");
    assert_one(
        &markup,
        "<td class=\"adjusted-vsize\">Adjusted vsize 1.75</td>",
    );
    assert_scripts_stay_off(&markup);
}

/// Explicit 0 is a real block time. The label stays, and the cell stays 0.
#[test]
fn block_time_zero_still_paints_the_label() {
    let markup = transaction_row_markup(&tx(r#"{"confirmed":true,"block_time":0}"#));
    assert_one(&markup, &pair(0));
    assert_one(&markup, LABEL);
    assert_label_word_once(&markup);
    assert_timestamp_line(&markup);
    assert_scripts_stay_off(&markup);
}

/// A missing block_time key leaves the transaction row up and omits the label.
#[test]
fn missing_block_time_omits_the_label() {
    let markup = transaction_row_markup(&tx(r#"{"confirmed":true,"block_height":100}"#));
    assert!(markup.contains("<tr class=\"tx-row\">"), "{markup}");
    assert_absent(&markup, "Timestamp");
    assert_absent(&markup, "timestamp-label");
    assert_absent(&markup, "block-time");
    assert_scripts_stay_off(&markup);
}

/// JSON null is not zero. The label stays off.
#[test]
fn null_block_time_omits_the_label() {
    let markup = transaction_row_markup(&tx(r#"{"confirmed":true,"block_time":null}"#));
    assert!(markup.contains("<tr class=\"tx-row\">"), "{markup}");
    assert_absent(&markup, "Timestamp");
    assert_absent(&markup, "timestamp-label");
    assert_absent(&markup, "block-time");
    assert_scripts_stay_off(&markup);
}

/// No status object is the unconfirmed shape. The label stays off.
#[test]
fn missing_status_omits_the_label() {
    let markup = transaction_row_markup(&tx(""));
    assert!(markup.contains("<tr class=\"tx-row\">"), "{markup}");
    assert_absent(&markup, "Timestamp");
    assert_absent(&markup, "block-time");
    assert_scripts_stay_off(&markup);
}

/// The blocks list already shows a raw time cell. It does not gain Timestamp.
#[test]
fn block_list_row_does_not_gain_the_label() {
    let block = format!(
        r#"{{"id":"{BLOCK_ID}","height":1,"version":1,"timestamp":1600000000,"tx_count":1,"size":1,"weight":4,"merkle_root":"merkle","mediantime":1}}"#
    );
    let markup = block_rows_markup(&block, "[]");
    assert_one(&markup, "<td class=\"time\">1600000000</td>");
    assert_absent(&markup, "Timestamp");
    assert_absent(&markup, "timestamp-label");
}
