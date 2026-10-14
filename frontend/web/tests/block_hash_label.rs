//! Hash label on the block row.
//!
//! Angular paints the label Hash and `shortenString` of length 13.
//! `half` is the number 6.5. `substring` truncates each index toward zero,
//! so a 64-character id is the first 6 characters, `...`, then the last 7.
//! An id of length 13 or less is the whole id. The hash cell stays the full
//! id link. A missing id fails the block parse and does not paint Hash.

use splora_web::block_rows_markup;

const LONG_ID: &str = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";
const SHORT_ID: &str = "0123456789abc";

const HASH_LABEL: &str = "<td class=\"hash-label\">Hash</td>";

fn block_json(id: &str) -> String {
    format!(
        r#"{{"id":"{id}","height":1,"version":1,"timestamp":1,"tx_count":1,"size":1,"weight":4,"merkle_root":"merkle","mediantime":1}}"#
    )
}

fn missing_id_json() -> &'static str {
    r#"{"height":1,"version":1,"timestamp":1,"tx_count":1,"size":1,"weight":4,"merkle_root":"merkle","mediantime":1}"#
}

fn short_cell(text: &str) -> String {
    format!("<td class=\"short-hash\">{text}</td>")
}

fn hash_anchor(id: &str) -> String {
    format!("<a href=\"/block/{id}\">{id}</a>")
}

fn assert_one(markup: &str, expected: &str) {
    assert_eq!(
        markup.matches(expected).count(),
        1,
        "expected one {expected} in {markup}"
    );
}

#[test]
fn sixty_four_character_id_paints_hash_label_and_js_shorten() {
    assert_eq!(LONG_ID.len(), 64);
    let markup = block_rows_markup(&block_json(LONG_ID), "[]");
    let shortened = short_cell("012345...9abcdef");
    let anchor = hash_anchor(LONG_ID);
    assert_one(&markup, HASH_LABEL);
    assert_one(&markup, &shortened);
    assert!(
        !markup.contains("012345...abcdef"),
        "integer 6-and-6 split leaked into the markup: {markup}"
    );
    assert_one(&markup, &anchor);
    assert_one(&markup, &format!("<td class=\"hash\">{anchor}</td>"));
}

#[test]
fn thirteen_character_id_is_the_whole_id() {
    assert_eq!(SHORT_ID.len(), 13);
    let markup = block_rows_markup(&block_json(SHORT_ID), "[]");
    let cell = short_cell(SHORT_ID);
    assert_one(&markup, &cell);
    assert!(
        !cell.contains("..."),
        "short-hash cell shortened a 13-character id: {cell}"
    );
    assert!(
        !markup.contains("..."),
        "ellipsis leaked into the markup: {markup}"
    );
}

#[test]
fn missing_id_does_not_paint_hash() {
    let markup = block_rows_markup(missing_id_json(), "[]");
    assert!(
        !markup.contains("Hash"),
        "missing id painted Hash: {markup}"
    );
}
