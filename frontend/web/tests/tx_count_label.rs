//! TXs label on the block row from `GET /block/:hash`.
//!
//! Angular paints the column label TXs beside `block.tx_count` on the blocks
//! list. The value cell stays the existing raw integer. Explicit 0 still
//! paints the label. A missing tx_count key or JSON null fails the block
//! parse and does not paint TXs. Size, weight, and the other skipped labels
//! stay off.

use splora_web::{block_rows_markup, blocks_rows_markup};

const ID: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
const LABEL: &str = "<td class=\"tx-count-label\">TXs</td>";

fn block_json(tx_count: u32) -> String {
    format!(
        r#"{{"id":"{ID}","height":1,"version":1,"timestamp":1,"tx_count":{tx_count},"size":1,"weight":4,"merkle_root":"merkle","mediantime":1}}"#
    )
}

fn missing_tx_count_json() -> &'static str {
    r#"{"id":"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa","height":1,"version":1,"timestamp":1,"size":1,"weight":4,"merkle_root":"merkle","mediantime":1}"#
}

fn null_tx_count_json() -> &'static str {
    r#"{"id":"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa","height":1,"version":1,"timestamp":1,"tx_count":null,"size":1,"weight":4,"merkle_root":"merkle","mediantime":1}"#
}

fn tx_count_cell(tx_count: u32) -> String {
    format!("<td class=\"tx-count\">{tx_count}</td>")
}

fn pair(tx_count: u32) -> String {
    format!("{LABEL}{}", tx_count_cell(tx_count))
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

fn assert_skipped_stay_off(markup: &str) {
    for needle in [
        "Size ",
        "Weight ",
        "Previous Block",
        "Unconfirmed",
        "Timestamp",
        "Merkle root",
        "Nonce",
        "Why is this block empty?",
        "Pool",
        "Miner",
        "Health",
        "Reward",
        "Transactions",
        "1,234",
        "\u{200e}",
    ] {
        assert_absent(markup, needle);
    }
}

#[test]
fn tx_count_paints_the_label_beside_the_integer() {
    let markup = block_rows_markup(&block_json(17), "[]");
    assert_one(&markup, &pair(17));
    assert_one(&markup, LABEL);
    assert_eq!(markup.matches("TXs").count(), 1, "{markup}");
    assert_absent(&tx_count_cell(17), "TXs");
    assert_skipped_stay_off(&markup);
}

#[test]
fn tx_count_on_the_blocks_list_paints_the_same_label() {
    let markup = blocks_rows_markup(&format!("[{}]", block_json(17)));
    assert_one(&markup, &pair(17));
    assert_eq!(markup.matches("TXs").count(), 1, "{markup}");
    assert_skipped_stay_off(&markup);
}

#[test]
fn tx_count_zero_still_paints_the_label() {
    let markup = block_rows_markup(&block_json(0), "[]");
    assert_one(&markup, &pair(0));
    assert_eq!(markup.matches("TXs").count(), 1, "{markup}");
    assert_skipped_stay_off(&markup);
}

#[test]
fn tx_count_stays_a_raw_integer() {
    let markup = block_rows_markup(&block_json(1234), "[]");
    assert_one(&markup, &pair(1234));
    assert_skipped_stay_off(&markup);
}

#[test]
fn missing_tx_count_key_omits_the_label() {
    let markup = block_rows_markup(missing_tx_count_json(), "[]");
    assert_absent(&markup, "TXs");
    assert_absent(&markup, "tx-count-label");
    assert_absent(&markup, "<tr class=\"block-row\">");
    assert_skipped_stay_off(&markup);
}

#[test]
fn null_tx_count_omits_the_label() {
    let markup = block_rows_markup(null_tx_count_json(), "[]");
    assert_absent(&markup, "TXs");
    assert_absent(&markup, "tx-count-label");
    assert_absent(&markup, "<tr class=\"block-row\">");
    assert_skipped_stay_off(&markup);
}

#[test]
fn missing_tx_count_on_the_blocks_list_omits_the_label() {
    let markup = blocks_rows_markup(&format!("[{}]", missing_tx_count_json()));
    assert_absent(&markup, "TXs");
    assert_absent(&markup, "tx-count-label");
    assert_absent(&markup, "<tr class=\"block-row\">");
    assert_skipped_stay_off(&markup);
}
