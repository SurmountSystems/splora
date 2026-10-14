//! Recent transactions column header from `GET /mempool/recent`.
//!
//! Angular `recent-transactions-list.component.html` paints
//! `<th class="table-cell-satoshis">Amount</th>` once on that page, not on the
//! blocks list. The overview field is the integer `value`. The value cell stays
//! that integer. It does not add a fiat amount, a currency symbol, or a
//! thousands separator. An empty list, a blank body, and invalid JSON do not
//! paint the header.

use splora_web::{blocks_rows_markup, recent_rows_markup};

const HEADER: &str = "<th class=\"table-cell-satoshis\">Amount</th>";
const VSIZE_HEADER: &str = "<th class=\"table-cell-vsize\">vSize</th>";
const BLOCK_ID: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";

fn recent(value: u64) -> String {
    format!(r#"[{{"txid":"recent-amount-header","fee":10,"vsize":141,"value":{value}}}]"#)
}

fn two_recent() -> String {
    r#"[{"txid":"recent-one","fee":10,"vsize":141,"value":1},{"txid":"recent-two","fee":20,"vsize":1000,"value":99000}]"#
        .to_string()
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

fn block_row(markup: &str) -> &str {
    let start = markup.find("<tr class=\"block-row\">").expect("block row");
    let rest = &markup[start..];
    let end = rest.find("</tr>").expect("block row end") + "</tr>".len();
    &rest[..end]
}

#[test]
fn one_recent_row_paints_the_amount_header_once() {
    let markup = recent_rows_markup(&recent(99000));
    assert_one(&markup, HEADER);
    assert_eq!(markup.matches("Amount").count(), 1, "{markup}");
    assert_one(&markup, "<td class=\"value\">99000</td>");
    assert!(
        markup.find(HEADER).unwrap() < markup.find("<tr class=\"tx-row\">").unwrap(),
        "the header belongs before the row: {markup}"
    );
    assert_one(&markup, VSIZE_HEADER);
    assert_absent(&markup, "$");
    assert_absent(&markup, "BTC");
    assert_absent(&markup, "99,000");
}

#[test]
fn two_rows_paint_the_header_once_and_keep_raw_integers() {
    let markup = recent_rows_markup(&two_recent());
    assert_one(&markup, HEADER);
    assert_one(&markup, "<td class=\"value\">1</td>");
    assert_one(&markup, "<td class=\"value\">99000</td>");
    assert_absent(&markup, "99,000");
    assert_absent(&markup, "$");
    assert_absent(&markup, "BTC");
}

#[test]
fn value_zero_still_prints_the_header_and_zero() {
    let markup = recent_rows_markup(&recent(0));
    assert_one(&markup, HEADER);
    assert_one(&markup, "<td class=\"value\">0</td>");
}

#[test]
fn empty_recent_omits_the_header() {
    assert_absent(&recent_rows_markup("[]"), "Amount");
    assert_absent(&recent_rows_markup(""), "Amount");
    assert_absent(&recent_rows_markup("not-json"), "Amount");
    assert_absent(&recent_rows_markup("[{}]"), "Amount");
}

#[test]
fn blocks_list_row_does_not_gain_amount_vsize_size_or_weight() {
    let markup = blocks_rows_markup(&block_list());
    let row = block_row(&markup);
    assert_absent(row, "Amount");
    assert_absent(row, "vSize");
    assert_absent(row, "Size");
    assert_absent(row, "Weight");
    assert_absent(row, HEADER);
}
