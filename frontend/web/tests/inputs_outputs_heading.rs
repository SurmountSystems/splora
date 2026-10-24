//! Inputs & Outputs heading on the loaded transaction from `GET /tx/:txid`.
//!
//! Angular paints `<h2>Inputs & Outputs</h2>` in the subtitle block before
//! the transaction list, whenever the transaction is present. The heading
//! is once, not once per input or output. Empty `vin` and empty `vout`
//! still paint it. An empty body, invalid JSON, and `[]` do not. No
//! skeleton, no Transaction hex, no Show diagram, and no Allowed?.

use splora_web::transaction_row_markup;

const HEADING: &str = "<h2>Inputs & Outputs</h2>";
const TXID: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
const PREV: &str = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";

fn input() -> String {
    format!(
        r#"{{"txid":"{PREV}","vout":0,"prevout":null,"scriptsig":"","scriptsig_asm":"","is_coinbase":false,"sequence":1}}"#
    )
}

fn output() -> String {
    r#"{"scriptpubkey":"","scriptpubkey_asm":"","scriptpubkey_type":"p2pkh","value":1}"#.to_string()
}

fn tx(inputs: usize, outputs: usize) -> String {
    let vin = (0..inputs).map(|_| input()).collect::<Vec<_>>().join(",");
    let vout = (0..outputs).map(|_| output()).collect::<Vec<_>>().join(",");
    format!(
        r#"{{"txid":"{TXID}","version":1,"locktime":0,"vin":[{vin}],"vout":[{vout}],"size":10,"weight":40,"sigops":0,"fee":1}}"#
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

fn assert_heading_once_before_the_table(markup: &str) {
    assert_one(markup, HEADING);
    // The h2 and the whole line each contain the words.
    assert_eq!(markup.matches("Inputs & Outputs").count(), 2, "{markup}");
    assert_eq!(markup.matches("<h2>").count(), 1, "{markup}");
    let heading_at = markup.find(HEADING).expect("heading");
    let table_at = markup
        .find("<table class=\"tx-rows\"")
        .or_else(|| markup.find("<tr class=\"tx-row\">"))
        .expect("tx table or row");
    assert!(
        heading_at < table_at,
        "heading was not before the tx table: {markup}"
    );
    assert!(
        !markup[..heading_at].contains("<td"),
        "heading is inside a cell: {markup}"
    );
}

fn assert_skipped_labels_stay_off(markup: &str) {
    assert_absent(markup, "Transaction hex");
    assert_absent(markup, "skeleton-loader");
    assert_absent(markup, "Show diagram");
    assert_absent(markup, "Allowed?");
}

#[test]
fn one_input_and_one_output_paints_the_heading_once_before_the_table() {
    let markup = transaction_row_markup(&tx(1, 1));
    assert!(
        markup.contains("<table class=\"tx-rows\"") || markup.contains("<tr class=\"tx-row\">"),
        "{markup}"
    );
    assert_heading_once_before_the_table(&markup);
    assert_skipped_labels_stay_off(&markup);
}

#[test]
fn two_inputs_and_two_outputs_still_paint_the_heading_once() {
    let markup = transaction_row_markup(&tx(2, 2));
    assert_eq!(markup.matches("class=\"party\"").count(), 4, "{markup}");
    assert_heading_once_before_the_table(&markup);
    assert_skipped_labels_stay_off(&markup);
}

#[test]
fn empty_vin_and_vout_still_paint_the_heading_once() {
    let markup = transaction_row_markup(&tx(0, 0));
    assert_eq!(markup.matches("class=\"party\"").count(), 0, "{markup}");
    assert_heading_once_before_the_table(&markup);
    assert_skipped_labels_stay_off(&markup);
}

#[test]
fn empty_body_does_not_paint_the_heading() {
    let markup = transaction_row_markup("");
    assert_absent(&markup, "Inputs & Outputs");
    assert_skipped_labels_stay_off(&markup);
}

#[test]
fn invalid_json_does_not_paint_the_heading() {
    let markup = transaction_row_markup("not-json");
    assert_absent(&markup, "Inputs & Outputs");
    assert_skipped_labels_stay_off(&markup);
}

#[test]
fn empty_array_does_not_paint_the_heading() {
    let markup = transaction_row_markup("[]");
    assert_absent(&markup, "Inputs & Outputs");
    assert_skipped_labels_stay_off(&markup);
}
