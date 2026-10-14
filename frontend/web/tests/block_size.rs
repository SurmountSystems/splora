//! Block summary Size value from `GET /block/:hash`.
//!
//! Angular `block.component.html` puts the word Size in its own label cell.
//! The value cell is `block.size | bytes: 2`: decimal 2, from `B`, powers of
//! 1000, plaintext false. That value is the number, a space, and
//! `<span class="symbol">{unit}</span>`. The left-to-right mark stays off the
//! page. The word Size stays out of the class size cell.
//! `size_number_and_unit` divides in f64, then `(value * 100).round() / 100`.
//! Size 1005 is `1` kB, not `1.01`. Size 1234 is `1.23` kB.
//! A missing size key fails the block parse and does not paint class size.

use splora_web::block_rows_markup;

const ID: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";

fn block_json(size: u32) -> String {
    format!(
        r#"{{"id":"{ID}","height":1,"version":1,"timestamp":1,"tx_count":1,"size":{size},"weight":4,"merkle_root":"merkle","mediantime":1}}"#
    )
}

fn missing_size_json() -> &'static str {
    r#"{"id":"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa","height":1,"version":1,"timestamp":1,"tx_count":1,"weight":4,"merkle_root":"merkle","mediantime":1}"#
}

fn size_cell(inner: &str) -> String {
    format!("<td class=\"size\">{inner}</td>")
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

fn assert_no_lrm(markup: &str) {
    assert_absent(markup, "&lrm;");
    assert_absent(markup, "\u{200e}");
}

fn block_row(markup: &str) -> &str {
    let start = markup.find("<tr class=\"block-row\">").expect("block row");
    let rest = &markup[start..];
    let end = rest.find("</tr>").expect("block row end") + "</tr>".len();
    &rest[..end]
}

#[test]
fn size_1005_paints_one_kilobyte_not_one_point_zero_one() {
    let markup = block_rows_markup(&block_json(1005), "[]");
    let row = block_row(&markup);
    let cell = size_cell("1 <span class=\"symbol\">kB</span>");
    assert_one(row, &cell);
    assert_absent(row, "1.01");
    assert_absent(row, &size_cell("1005"));
    assert_absent(row, "Size ");
    assert_no_lrm(&markup);
}

#[test]
fn size_999_paints_unscaled_bytes() {
    let markup = block_rows_markup(&block_json(999), "[]");
    let row = block_row(&markup);
    assert_one(row, &size_cell("999 <span class=\"symbol\">B</span>"));
    assert_absent(row, "kB");
    assert_absent(row, "Size ");
    assert_no_lrm(&markup);
}

#[test]
fn size_zero_paints_zero_bytes() {
    let markup = block_rows_markup(&block_json(0), "[]");
    let row = block_row(&markup);
    assert_one(row, &size_cell("0 <span class=\"symbol\">B</span>"));
    assert_absent(row, "Size ");
    assert_no_lrm(&markup);
}

#[test]
fn size_1234_paints_one_point_two_three_kilobytes() {
    let markup = block_rows_markup(&block_json(1234), "[]");
    let row = block_row(&markup);
    assert_one(row, &size_cell("1.23 <span class=\"symbol\">kB</span>"));
    assert_absent(row, &size_cell("1234"));
    assert_absent(row, "Size ");
    assert_no_lrm(&markup);
}

#[test]
fn missing_size_key_does_not_paint_size_cell() {
    let markup = block_rows_markup(missing_size_json(), "[]");
    assert_absent(&markup, "class=\"size\"");
    assert_absent(&markup, "<tr class=\"block-row\">");
    assert_absent(&markup, &size_cell("0 <span class=\"symbol\">B</span>"));
}
