//! Height label on the block row from `GET /block/:hash`.
//!
//! Angular paints the column label Height beside `block.height`. The value
//! cell stays the existing height link. Explicit 0 still paints the label.
//! A missing height key or JSON null fails the block parse and does not
//! paint Height. Size, weight, and the other skipped labels stay off.

use splora_web::block_rows_markup;

const ID: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
const LABEL: &str = "<td class=\"height-label\">Height</td>";

fn block_json(height: u32) -> String {
    format!(
        r#"{{"id":"{ID}","height":{height},"version":1,"timestamp":1,"tx_count":1,"size":1,"weight":4,"merkle_root":"merkle","mediantime":1}}"#
    )
}

fn missing_height_json() -> &'static str {
    r#"{"id":"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa","version":1,"timestamp":1,"tx_count":1,"size":1,"weight":4,"merkle_root":"merkle","mediantime":1}"#
}

fn null_height_json() -> &'static str {
    r#"{"id":"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa","height":null,"version":1,"timestamp":1,"tx_count":1,"size":1,"weight":4,"merkle_root":"merkle","mediantime":1}"#
}

fn height_cell(height: u32) -> String {
    format!("<td class=\"height\"><a href=\"/block/{ID}\">{height}</a></td>")
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
    ] {
        assert_absent(markup, needle);
    }
}

#[test]
fn height_one_paints_the_label_beside_the_link() {
    let markup = block_rows_markup(&block_json(1), "[]");
    assert_one(&markup, LABEL);
    assert_one(&markup, &height_cell(1));
    assert_eq!(markup.matches("Height").count(), 1, "{markup}");
    assert_absent(&height_cell(1), "Height");
    assert_skipped_stay_off(&markup);
}

#[test]
fn height_zero_still_paints_the_label() {
    let markup = block_rows_markup(&block_json(0), "[]");
    assert_one(&markup, LABEL);
    assert_one(&markup, &height_cell(0));
    assert_eq!(markup.matches("Height").count(), 1, "{markup}");
    assert_skipped_stay_off(&markup);
}

#[test]
fn missing_height_key_omits_the_label() {
    let markup = block_rows_markup(missing_height_json(), "[]");
    assert_absent(&markup, "Height");
    assert_absent(&markup, "height-label");
    assert_absent(&markup, "<tr class=\"block-row\">");
    assert_skipped_stay_off(&markup);
}

#[test]
fn null_height_omits_the_label() {
    let markup = block_rows_markup(null_height_json(), "[]");
    assert_absent(&markup, "Height");
    assert_absent(&markup, "height-label");
    assert_absent(&markup, "<tr class=\"block-row\">");
    assert_skipped_stay_off(&markup);
}
