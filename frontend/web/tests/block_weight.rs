//! Block summary Weight value from `GET /block/:hash`.
//!
//! Angular `block.component.html` puts the word Weight in its own label cell.
//! The value cell is `block.weight | wuBytes: 2`: decimal 2, from `WU`, powers of
//! 1000, plaintext false. That value is the number, a space, and
//! `<span class="symbol">{unit}</span>`. The left-to-right mark stays off the
//! page. The word Weight stays out of the class weight cell.
//! `weight_number_and_unit` picks the unit from the raw weight before rounding,
//! then divides in f64, then `(value * 100).round() / 100`.
//! Weight 1500 is `1.5` kWU. Weight 999999 is `1000` kWU because the unit is
//! chosen before rounding. A missing weight key fails the block parse and
//! does not paint class weight.

use splora_web::block_rows_markup;

const ID: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";

fn block_json(weight: u32) -> String {
    format!(
        r#"{{"id":"{ID}","height":1,"version":1,"timestamp":1,"tx_count":1,"size":1,"weight":{weight},"merkle_root":"merkle","mediantime":1}}"#
    )
}

fn missing_weight_json() -> &'static str {
    r#"{"id":"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa","height":1,"version":1,"timestamp":1,"tx_count":1,"size":1,"merkle_root":"merkle","mediantime":1}"#
}

fn weight_cell(inner: &str) -> String {
    format!("<td class=\"weight\">{inner}</td>")
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
fn weight_1500_paints_one_point_five_kwu_without_the_word_weight() {
    let markup = block_rows_markup(&block_json(1500), "[]");
    let row = block_row(&markup);
    let cell = weight_cell("1.5 <span class=\"symbol\">kWU</span>");
    assert_one(row, &cell);
    assert!(row.contains("1.5"), "1.5 missing from the block row: {row}");
    assert!(
        row.contains("<span class=\"symbol\">kWU</span>"),
        "symbol span missing from the block row: {row}"
    );
    assert_absent(row, "Weight 1.5 kWU");
    assert_absent(row, "Weight ");
    assert_absent(row, &weight_cell("1500"));
    assert_no_lrm(&markup);
}

#[test]
fn weight_zero_paints_zero_wu() {
    let markup = block_rows_markup(&block_json(0), "[]");
    let row = block_row(&markup);
    assert_one(row, &weight_cell("0 <span class=\"symbol\">WU</span>"));
    assert_absent(row, "Weight ");
    assert_no_lrm(&markup);
}

#[test]
fn weight_999_paints_unscaled_wu() {
    let markup = block_rows_markup(&block_json(999), "[]");
    let row = block_row(&markup);
    assert_one(row, &weight_cell("999 <span class=\"symbol\">WU</span>"));
    assert_absent(row, "kWU");
    assert_absent(row, "Weight ");
    assert_no_lrm(&markup);
}

#[test]
fn weight_999999_paints_1000_kwu_unit_chosen_before_rounding() {
    let markup = block_rows_markup(&block_json(999_999), "[]");
    let row = block_row(&markup);
    assert_one(row, &weight_cell("1000 <span class=\"symbol\">kWU</span>"));
    assert_absent(row, "MWU");
    assert_absent(row, &weight_cell("999999"));
    assert_absent(row, "Weight ");
    assert_no_lrm(&markup);
}

#[test]
fn missing_weight_key_does_not_paint_weight_cell() {
    let markup = block_rows_markup(missing_weight_json(), "[]");
    assert_absent(&markup, "class=\"weight\"");
    assert_absent(&markup, "<tr class=\"block-row\">");
    assert_absent(&markup, &weight_cell("0 <span class=\"symbol\">WU</span>"));
}
