//! Recent transactions heading from no-query `GET /mempool/recent`.
//!
//! Angular `recent-transactions-list.component.html` paints
//! `<h1>Recent Transactions</h1>` once on that page when it is not a widget.
//! The dashboard card title uses the same letters. The heading is not a
//! `recent:` row and not a `recent txid` row. The fee, vsize, and value
//! cells stay the raw integers.

use splora_web::{blocks_rows_markup, dashboard_rows_markup, recent_rows_markup};

const HEADING: &str = "<h1>Recent Transactions</h1>";
const LETTERS: &str = "Recent Transactions";
const TXID: &str = "abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789";
const TXID_B: &str = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";
const BLOCK_ID: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";

fn recent(txid: &str, fee: u64, vsize: u32, value: u64) -> String {
    format!(r#"[{{"txid":"{txid}","fee":{fee},"vsize":{vsize},"value":{value}}}]"#)
}

fn two_recent() -> String {
    format!(
        r#"[{{"txid":"{TXID}","fee":10,"vsize":141,"value":1}},{{"txid":"{TXID_B}","fee":20,"vsize":1000,"value":99000}}]"#
    )
}

fn block_list() -> String {
    format!(
        r#"[{{"id":"{BLOCK_ID}","height":1,"version":1,"timestamp":1,"tx_count":1,"size":1234,"weight":5678,"merkle_root":"merkle","mediantime":1}}]"#
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

fn assert_heading_once_before_the_table(markup: &str) {
    assert_one(markup, HEADING);
    // The h1 and the whole line each contain the words.
    assert_eq!(markup.matches(LETTERS).count(), 2, "{markup}");
    assert_eq!(markup.matches("<h1>").count(), 1, "{markup}");
    let heading_at = markup.find(HEADING).expect("heading");
    let table_at = markup.find("<table class=\"tx-rows\"").expect("tx table");
    assert!(
        heading_at < table_at,
        "heading was not before the tx table: {markup}"
    );
    assert!(
        !markup[..heading_at].contains("<td"),
        "heading is inside a cell: {markup}"
    );
    assert!(
        !markup[..table_at].contains("<tr class=\"tx-row\">"),
        "heading is inside a row: {markup}"
    );
}

fn assert_cells_stay(markup: &str, txid: &str, fee: u64, vsize: u32, value: u64) {
    let cell = format!("<td class=\"txid\"><a href=\"/tx/{txid}\">{txid}</a></td>");
    assert_one(markup, &cell);
    assert_one(markup, &format!("<td class=\"fee\">{fee}</td>"));
    assert_one(markup, &format!("<td class=\"vsize\">{vsize}</td>"));
    assert_one(markup, &format!("<td class=\"value\">{value}</td>"));
    assert_absent(markup, "recent txid ");
    assert_absent(markup, "recent:");
}

#[test]
fn one_recent_row_paints_the_heading_once() {
    let markup = recent_rows_markup(&recent(TXID, 10, 141, 99000));
    assert_heading_once_before_the_table(&markup);
    assert_cells_stay(&markup, TXID, 10, 141, 99000);
}

#[test]
fn dashboard_recent_list_paints_the_same_heading_once() {
    let markup = dashboard_rows_markup("[]", &recent(TXID, 10, 141, 99000));
    assert_heading_once_before_the_table(&markup);
    assert_cells_stay(&markup, TXID, 10, 141, 99000);
}

#[test]
fn two_rows_paint_the_heading_once_and_keep_the_cells() {
    let markup = recent_rows_markup(&two_recent());
    assert_heading_once_before_the_table(&markup);
    assert_cells_stay(&markup, TXID, 10, 141, 1);
    assert_cells_stay(&markup, TXID_B, 20, 1000, 99000);
}

#[test]
fn blocks_list_does_not_gain_the_heading() {
    let markup = blocks_rows_markup(&block_list());
    assert_absent(&markup, LETTERS);
    assert_absent(&markup, HEADING);
}

/// Whole line beside the Recent Transactions h1 from `GET /mempool/recent`.
///
/// A line equal to Recent Transactions passes. The h1 stays. An empty list
/// omits the line. The blocks list does not gain it.
#[test]
fn recent_page_paints_recent_transactions_as_its_own_line() {
    const LINE: &str = "Recent Transactions";
    let markup = recent_rows_markup(&recent(TXID, 10, 141, 99000));
    assert!(
        markup.lines().any(|line| line == LINE),
        "no line equals {LINE}: {markup}"
    );
    assert_eq!(
        markup.lines().filter(|line| *line == LINE).count(),
        1,
        "{LINE} was not painted once: {markup}"
    );
    assert_one(&markup, HEADING);
    assert!(
        !markup
            .lines()
            .any(|line| line.starts_with("Recent Transactions ")),
        "{LINE} was painted as a longer sentence: {markup}"
    );
    let empty = recent_rows_markup("[]");
    assert!(
        !empty.lines().any(|line| line == LINE),
        "an empty list painted {LINE}: {empty}"
    );
    let blocks = blocks_rows_markup(&block_list());
    assert!(
        !blocks.lines().any(|line| line == LINE),
        "the blocks list painted {LINE}: {blocks}"
    );
}
