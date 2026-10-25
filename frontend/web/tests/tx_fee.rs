//! Preview fee on the transaction details row from `GET /tx/:txid`.
//!
//! Angular paints `tx.fee | number` with DecimalPipe `1.0-3` and an en-US
//! grouping comma. Fee is an integer, so there is no fraction. The cell is
//! `<span class="label">Fee</span> {grouped} <span class="symbol">sats</span>`.
//! The raw fee cell stays. Virtual size stays. Adjusted vsize 1.75 stays off.

use splora_web::transaction_row_markup;

const SCRIPT_HEX: &str = "TXFEEPUBHEXDEAD";
const SCRIPT_SIG_HEX: &str = "TXFEESIGHEXDEAD";
const SCRIPT_SIG_ASM: &str = "TXFEESIGASMDEAD";
const SCRIPT_ASM: &str = "TXFEEASMDEAD";

fn tx(fee: u64, weight: u32) -> String {
    format!(
        r#"{{"txid":"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa","version":1,"locktime":0,"vin":[{{"txid":"bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb","vout":0,"prevout":null,"scriptsig":"{SCRIPT_SIG_HEX}","scriptsig_asm":"{SCRIPT_SIG_ASM}","is_coinbase":false,"sequence":1}}],"vout":[{{"scriptpubkey":"{SCRIPT_HEX}","scriptpubkey_asm":"{SCRIPT_ASM}","scriptpubkey_type":"p2pkh","value":1}}],"size":10,"weight":{weight},"sigops":0,"fee":{fee}}}"#
    )
}

fn preview_cell(grouped: &str) -> String {
    format!(
        "<td class=\"preview-fee\"><span class=\"label\">Fee</span> {grouped} <span class=\"symbol\">sats</span></td>"
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

fn assert_scripts_stay_off(markup: &str) {
    assert_absent(markup, SCRIPT_HEX);
    assert_absent(markup, SCRIPT_SIG_HEX);
    assert_absent(markup, SCRIPT_SIG_ASM);
    assert_absent(markup, SCRIPT_ASM);
}

#[test]
fn fee_zero_paints_zero_without_a_comma() {
    let markup = transaction_row_markup(&tx(0, 4));
    assert_one(&markup, &preview_cell("0"));
    assert_one(&markup, "<td class=\"fee\">0</td>");
    assert_absent(&markup, ",");
    assert_scripts_stay_off(&markup);
}

#[test]
fn fee_nine_hundred_ninety_nine_paints_fee_without_a_trailing_comma() {
    let markup = transaction_row_markup(&tx(999, 4));
    assert_one(&markup, &preview_cell("999"));
    assert_one(&markup, "<td class=\"fee\">999</td>");
    assert_absent(&markup, "999,");
    assert_scripts_stay_off(&markup);
}

#[test]
fn fee_fifteen_hundred_paints_grouped_and_keeps_vsize() {
    let markup = transaction_row_markup(&tx(1500, 7));
    assert_one(&markup, &preview_cell("1,500"));
    assert_one(&markup, "<td class=\"fee\">1500</td>");
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
fn fee_one_million_paints_grouped() {
    let markup = transaction_row_markup(&tx(1_000_000, 4));
    assert_one(&markup, &preview_cell("1,000,000"));
    assert_one(&markup, "<td class=\"fee\">1000000</td>");
    assert_scripts_stay_off(&markup);
}

/// Whole line `Fee 0 sats` from no-query `GET /tx/:txid`.
///
/// Fee 0 paints that line once. The Fee cell keeps the label and the unit
/// apart. Fee 1 paints `Fee 1 sats` once. The decimal sequence span and the
/// `nSequence 0x00` span stay. `nSequence 0x00` is not its own line.
#[test]
fn fee_zero_paints_fee_0_sats_as_its_own_line() {
    const LINE: &str = "Fee 0 sats";
    let markup = transaction_row_markup(&tx(0, 4));
    assert!(
        markup.lines().any(|line| line == LINE),
        "{LINE} is absent: {markup}"
    );
    assert_eq!(
        markup.lines().filter(|line| *line == LINE).count(),
        1,
        "{LINE} was not one line: {markup}"
    );
    assert_one(&markup, &preview_cell("0"));
    assert_one(&markup, "<td class=\"fee\">0</td>");
    assert_one(&markup, "<span class=\"sequence\">sequence 1</span>");
    assert_scripts_stay_off(&markup);

    let one = transaction_row_markup(&tx(1, 4));
    assert!(
        one.lines().any(|line| line == "Fee 1 sats"),
        "Fee 1 sats is absent: {one}"
    );
    assert_eq!(
        one.lines().filter(|line| *line == "Fee 1 sats").count(),
        1,
        "Fee 1 sats was not one line: {one}"
    );
    assert_one(&one, &preview_cell("1"));
    assert_scripts_stay_off(&one);

    let zero_seq = transaction_row_markup(&tx(0, 4).replace("\"sequence\":1", "\"sequence\":0"));
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

/// Whole line `Fee 50 sats` from no-query `GET /tx/:txid`.
///
/// GPUI `assert_one_fee_and_confirmed` requires one line whose entire text
/// equals `Fee 50 sats`. The Fee cell keeps the label and the unit apart.
/// A grouped fee does not use this sentence. Fee 50 and weight 400 keep the
/// bare line `Fee rate` and do not paint `sat/vB`.
#[test]
fn fee_fifty_paints_fee_50_sats_as_its_own_line() {
    const LINE: &str = "Fee 50 sats";
    let markup = transaction_row_markup(&tx(50, 400));
    assert!(
        markup.lines().any(|line| line == LINE),
        "no line equals {LINE}: {markup}"
    );
    assert_eq!(
        markup.lines().filter(|line| *line == LINE).count(),
        1,
        "{LINE} was not painted once: {markup}"
    );
    assert_one(&markup, &preview_cell("50"));
    assert_one(&markup, "<td class=\"fee\">50</td>");
    assert!(
        !markup.contains("sat/vB"),
        "fee 50 and weight 400 painted sat/vB: {markup}"
    );
    assert!(
        markup.lines().any(|line| line == "Fee rate"),
        "bare Fee rate was dropped: {markup}"
    );
    assert_scripts_stay_off(&markup);
}

/// Whole line `Fee 1,500 sats`. Fee 1500 and weight 40, same as GPUI `paint_tx_fee`.
#[test]
fn gpui_whole_line_fee_1_500_sats() {
    let markup = transaction_row_markup(&tx(1500, 40));
    assert!(
        markup.lines().any(|line| line == "Fee 1,500 sats"),
        "fee 1500 and weight 40 did not paint Fee 1,500 sats: {markup}"
    );
}

/// Whole line `Fee 1,000,000 sats`. Fee 1000000 and weight 40, same as GPUI `paint_tx_fee`.
#[test]
fn gpui_whole_line_fee_1_000_000_sats() {
    let markup = transaction_row_markup(&tx(1_000_000, 40));
    assert!(
        markup.lines().any(|line| line == "Fee 1,000,000 sats"),
        "fee 1000000 and weight 40 did not paint Fee 1,000,000 sats: {markup}"
    );
}
