//! Genesis title on the single block page.
//!
//! Angular `block.component.html` paints the word Genesis only when the
//! block height is 0. A missing height key fails parse, so the span stays
//! off. Height 1 and every height above 0 stay off. The blocks list does
//! not paint it.

use splora_web::{block_rows_markup, blocks_rows_markup};

const ID: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
const SPAN: &str = "<span class=\"genesis\">Genesis</span>";
const WORD: &str = "Genesis";
const EMPTY_SENTENCE: &str = "Why is this block empty?";
const EMPTY_SPAN: &str = "<span class=\"empty-block\">Why is this block empty?</span>";
const BLOCK_ROW: &str = "<tr class=\"block-row\">";

fn block_json(height: u32, tx_count: u32) -> String {
    format!(
        r#"{{"id":"{ID}","height":{height},"version":1,"timestamp":1,"tx_count":{tx_count},"size":1,"weight":4,"merkle_root":"merkle","mediantime":1}}"#
    )
}

fn missing_height_json() -> &'static str {
    r#"{"id":"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa","version":1,"timestamp":1,"tx_count":1,"size":1,"weight":4,"merkle_root":"merkle","mediantime":1}"#
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

#[test]
fn height_0_and_tx_count_1_paints_genesis_once() {
    let markup = block_rows_markup(&block_json(0, 1), "[]");
    assert_one(&markup, SPAN);
    assert_absent(&markup, EMPTY_SENTENCE);
}

#[test]
fn height_0_and_tx_count_0_paints_genesis_once() {
    let markup = block_rows_markup(&block_json(0, 0), "[]");
    assert_one(&markup, SPAN);
    assert_absent(&markup, EMPTY_SENTENCE);
}

#[test]
fn height_1_omits_genesis() {
    let markup = block_rows_markup(&block_json(1, 1), "[]");
    assert_absent(&markup, WORD);
    assert!(markup.contains(BLOCK_ROW), "block row missing: {markup}");
}

#[test]
fn height_100000_and_tx_count_2_omits_genesis_and_the_empty_sentence() {
    let markup = block_rows_markup(&block_json(100_000, 2), "[]");
    assert_absent(&markup, WORD);
    assert_absent(&markup, EMPTY_SENTENCE);
    assert!(markup.contains(BLOCK_ROW), "block row missing: {markup}");
}

#[test]
fn height_100001_and_tx_count_1_paints_the_empty_span_and_omits_genesis() {
    let markup = block_rows_markup(&block_json(100_001, 1), "[]");
    assert_one(&markup, EMPTY_SPAN);
    assert_absent(&markup, WORD);
}

#[test]
fn omitted_height_key_omits_genesis_and_the_empty_sentence() {
    let markup = block_rows_markup(missing_height_json(), "[]");
    assert_absent(&markup, WORD);
    assert_absent(&markup, EMPTY_SENTENCE);
}

#[test]
fn blocks_list_height_0_omits_genesis() {
    let markup = blocks_rows_markup(&format!("[{}]", block_json(0, 1)));
    assert_absent(&markup, WORD);
    assert!(markup.contains(BLOCK_ROW), "block row missing: {markup}");
}
