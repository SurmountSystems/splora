//! Empty-block sentence on the single block page.
//!
//! Angular `block.component.html` paints "Why is this block empty?" when
//! height is greater than 100000 and `tx_count` is at most 1. Height 100000
//! stays off. `tx_count` 2 stays off. `tx_count` 0 and `tx_count` 1 paint it.
//! A missing height key or a missing `tx_count` key fails parse, so the
//! sentence stays off. The template also requires `network === ''`.
//! `block_rows_markup` has no network argument.

use splora_web::block_rows_markup;

const ID: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
const SPAN: &str = "<span class=\"empty-block\">Why is this block empty?</span>";
const SENTENCE: &str = "Why is this block empty?";
const BLOCK_ROW: &str = "<tr class=\"block-row\">";

fn block_json(height: u32, tx_count: u32) -> String {
    format!(
        r#"{{"id":"{ID}","height":{height},"version":1,"timestamp":1,"tx_count":{tx_count},"size":1,"weight":4,"merkle_root":"merkle","mediantime":1}}"#
    )
}

fn missing_height_json() -> &'static str {
    r#"{"id":"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa","version":1,"timestamp":1,"tx_count":1,"size":1,"weight":4,"merkle_root":"merkle","mediantime":1}"#
}

fn missing_tx_count_json() -> &'static str {
    r#"{"id":"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa","height":100001,"version":1,"timestamp":1,"size":1,"weight":4,"merkle_root":"merkle","mediantime":1}"#
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
fn height_100001_and_tx_count_1_paints_the_sentence_once() {
    let markup = block_rows_markup(&block_json(100_001, 1), "[]");
    assert_one(&markup, SPAN);
    assert_one(&markup, SENTENCE);
}

#[test]
fn height_100001_and_tx_count_0_paints_the_span_once() {
    let markup = block_rows_markup(&block_json(100_001, 0), "[]");
    assert_one(&markup, SPAN);
}

#[test]
fn height_100000_and_tx_count_1_omits_the_sentence() {
    let markup = block_rows_markup(&block_json(100_000, 1), "[]");
    assert_absent(&markup, SENTENCE);
    assert!(markup.contains(BLOCK_ROW), "block row missing: {markup}");
}

#[test]
fn height_100001_and_tx_count_2_omits_the_sentence() {
    let markup = block_rows_markup(&block_json(100_001, 2), "[]");
    assert_absent(&markup, SENTENCE);
    assert!(markup.contains(BLOCK_ROW), "block row missing: {markup}");
}

#[test]
fn omitted_height_key_omits_the_sentence() {
    let markup = block_rows_markup(missing_height_json(), "[]");
    assert_absent(&markup, SENTENCE);
}

#[test]
fn omitted_tx_count_key_omits_the_sentence() {
    let markup = block_rows_markup(missing_tx_count_json(), "[]");
    assert_absent(&markup, SENTENCE);
}
