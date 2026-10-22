//! Block Header Hex on the single block details panel.
//!
//! Angular `block.component.html` labels the row `Block Header Hex`.
//! The next cell is a link to the header route, not the hex digits.
//! The digits are the 80-byte body from `GET /block/:hash/header`:
//! 160 characters, `0-9` and `a-f`. A missing or failed body does not
//! invent a header of zeros and does not paint an empty hex cell.
//! The label stays, because that Angular row is not hidden.

use splora_web::{block_details_markup, block_header_aside_markup, block_header_hex_markup};

const ID: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";

fn block_json() -> String {
    format!(
        r#"{{"id":"{ID}","height":2,"version":1,"timestamp":1700000000,"tx_count":1,"size":100,"weight":400,"merkle_root":"merkle-root","previousblockhash":null,"mediantime":1699990000,"nonce":9,"bits":386547056,"difficulty":1}}"#
    )
}

fn header_hex() -> String {
    "0123456789abcdef".repeat(10)
}

fn zero_header() -> String {
    "0".repeat(160)
}

fn details() -> String {
    block_details_markup(&block_json(), true)
}

fn assert_one(markup: &str, expected: &str) {
    assert_eq!(
        markup.matches(expected).count(),
        1,
        "expected one {expected} in {markup}"
    );
}

#[test]
fn eighty_byte_header_paints_the_label_and_the_hex() {
    let open = block_header_hex_markup(&details(), &header_hex());
    let row = format!(
        "<tr><td>Block Header Hex</td><td><a href=\"/block/{ID}/header\"></a></td><td class=\"block-header-hex\">{}</td></tr>",
        header_hex()
    );
    assert_one(&open, &row);
    assert_eq!(open.matches("Block Header Hex").count(), 1, "{open}");
    assert!(
        open.contains(&format!("href=\"/block/{ID}/header\"")),
        "{open}"
    );
}

#[test]
fn whitespace_around_the_header_is_trimmed() {
    let body = format!("  {}\n", header_hex());
    let open = block_header_hex_markup(&details(), &body);
    assert_one(
        &open,
        &format!("<td class=\"block-header-hex\">{}</td>", header_hex()),
    );
}

#[test]
fn uppercase_header_paints_lowercase_hex() {
    let open = block_header_hex_markup(&details(), &header_hex().to_ascii_uppercase());
    assert_one(
        &open,
        &format!("<td class=\"block-header-hex\">{}</td>", header_hex()),
    );
}

#[test]
fn indexer_zero_header_is_painted() {
    let open = block_header_hex_markup(&details(), &zero_header());
    assert_one(
        &open,
        &format!("<td class=\"block-header-hex\">{}</td>", zero_header()),
    );
}

#[test]
fn missing_header_keeps_the_label_and_omits_hex() {
    let open = block_header_hex_markup(&details(), "");
    assert_one(&open, "<td>Block Header Hex</td>");
    assert_one(&open, &format!("href=\"/block/{ID}/header\""));
    assert!(!open.contains("block-header-hex"), "{open}");
    assert!(!open.contains(&zero_header()), "{open}");
    assert_eq!(open.matches("Block Header Hex").count(), 1, "{open}");
}

#[test]
fn failed_header_does_not_paint_empty_hex_or_the_error() {
    let open = block_header_hex_markup(
        &details(),
        "Block not found\nindexer error GET /block/abc/header",
    );
    assert_one(&open, "<td>Block Header Hex</td>");
    assert!(!open.contains("block-header-hex"), "{open}");
    assert!(!open.contains("Block not found"), "{open}");
    assert!(!open.contains(&zero_header()), "{open}");
}

#[test]
fn short_hex_is_not_the_block_header() {
    let open = block_header_hex_markup(&details(), "  aa55\n");
    assert!(!open.contains("block-header-hex"), "{open}");
    assert!(!open.contains(">aa55<"), "{open}");
    assert_one(&open, "<td>Block Header Hex</td>");
}

#[test]
fn block_header_aside_shows_block_header_aabb() {
    let markup = block_header_aside_markup("aabb");
    assert!(markup.contains("block header aabb"), "{markup}");
}
