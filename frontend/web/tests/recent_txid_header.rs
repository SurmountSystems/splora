//! Recent transactions column header from `GET /mempool/recent`.
//!
//! Angular `recent-transactions-list.component.html` paints
//! `<th class="table-cell-txid">TXID</th>` once on that page, not on the
//! blocks list. The route field is `txid` with no query string. The txid
//! cell stays that raw id. It does not add a fiat amount. An empty list, a
//! blank body, and invalid JSON do not paint the header.

use splora_web::{blocks_rows_markup, recent_rows_markup};

const HEADER: &str = "<th class=\"table-cell-txid\">TXID</th>";
const AMOUNT_HEADER: &str = "<th class=\"table-cell-satoshis\">Amount</th>";
const VSIZE_HEADER: &str = "<th class=\"table-cell-vsize\">vSize</th>";
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

fn txid_cell(txid: &str) -> String {
    format!("<td class=\"txid\"><a href=\"/tx/{txid}\">{txid}</a></td>")
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
fn one_recent_row_paints_the_txid_header_once() {
    let markup = recent_rows_markup(&recent(TXID, 10, 141, 99000));
    let cell = txid_cell(TXID);
    assert_one(&markup, HEADER);
    assert_eq!(markup.matches("TXID").count(), 1, "{markup}");
    assert_one(&markup, &cell);
    assert!(
        markup.find(HEADER).unwrap() < markup.find(AMOUNT_HEADER).unwrap(),
        "TXID belongs before Amount: {markup}"
    );
    assert!(
        markup.find(AMOUNT_HEADER).unwrap() < markup.find(VSIZE_HEADER).unwrap(),
        "Amount stays before vSize: {markup}"
    );
    assert!(
        markup.find(VSIZE_HEADER).unwrap() < markup.find("<tr class=\"tx-row\">").unwrap(),
        "the headers belong before the row: {markup}"
    );
    assert_one(&markup, AMOUNT_HEADER);
    assert_one(&markup, VSIZE_HEADER);
    assert_absent(&markup, "$");
    assert_absent(&markup, "BTC");
    assert_absent(&markup, "99,000");
}

#[test]
fn two_rows_paint_the_header_once_and_keep_raw_ids() {
    let markup = recent_rows_markup(&two_recent());
    assert_one(&markup, HEADER);
    assert_one(&markup, &txid_cell(TXID));
    assert_one(&markup, &txid_cell(TXID_B));
    assert_eq!(markup.matches("TXID").count(), 1, "{markup}");
    assert_absent(&markup, "$");
    assert_absent(&markup, "BTC");
}

#[test]
fn zero_amounts_still_print_the_header_and_the_raw_id() {
    let markup = recent_rows_markup(&recent(TXID, 0, 0, 0));
    assert_one(&markup, HEADER);
    assert_one(&markup, &txid_cell(TXID));
    assert_one(&markup, "<td class=\"value\">0</td>");
    assert_one(&markup, "<td class=\"vsize\">0</td>");
}

#[test]
fn empty_recent_omits_the_header() {
    assert_absent(&recent_rows_markup("[]"), "TXID");
    assert_absent(&recent_rows_markup(""), "TXID");
    assert_absent(&recent_rows_markup("not-json"), "TXID");
    assert_absent(&recent_rows_markup("[{}]"), "TXID");
    assert_absent(&recent_rows_markup("[]"), HEADER);
}

#[test]
fn blocks_list_row_does_not_gain_txid_amount_vsize_size_or_weight() {
    let markup = blocks_rows_markup(&block_list());
    let row = block_row(&markup);
    assert_absent(row, "TXID");
    assert_absent(row, "Amount");
    assert_absent(row, "vSize");
    assert_absent(row, "Size");
    assert_absent(row, "Weight");
    assert_absent(row, HEADER);
}
