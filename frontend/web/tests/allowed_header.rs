//! Allowed? header on test-transaction txid markup from `POST /txs/test`.
//!
//! Angular paints `<th class="allowed">Allowed?</th>` once on the results
//! table, not once per row. The cell under that header is an emoji, which
//! stays off this markup. A parsed `allowed: true` row paints the header
//! once before the txid anchor. A row with `allowed: false` and no
//! `reject-reason` fails the public parse of the whole array. The fallback
//! still emits those txid anchors, and the header still belongs once in
//! front of them. An empty array, an empty string, and invalid JSON emit
//! no txid and do not paint Allowed?.

use splora_web::test_txid_markup;

const TX: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
const SPEND: &str = "eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee";
const HEADER: &str = "<th class=\"allowed\">Allowed?</th>";

fn tx_anchor(txid: &str) -> String {
    format!("<a href=\"/tx/{txid}\">{txid}</a>")
}

fn allowed_true_body() -> String {
    format!(r#"[{{"txid":"{TX}","allowed":true}}]"#)
}

/// Same body as `broadcast_and_test_txids_open_the_transaction` in walk.rs.
fn fallback_body() -> String {
    format!(r#"[{{"txid":"{TX}","allowed":true}},{{"txid":"{SPEND}","allowed":false}}]"#)
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

fn assert_other_labels_stay_off(markup: &str) {
    for needle in [
        "Transaction hex",
        "Effective fee rate",
        "Rejection reason",
        "TXID",
        "Raw hex",
        "Test Transactions",
        "✅",
        "❌",
    ] {
        assert_absent(markup, needle);
    }
}

#[test]
fn parsed_allowed_true_paints_the_header_once() {
    let markup = test_txid_markup(&allowed_true_body());
    let anchor = tx_anchor(TX);
    assert_one(&markup, HEADER);
    assert_one(&markup, &anchor);
    assert_eq!(markup.matches("Allowed?").count(), 1, "{markup}");
    assert_eq!(markup, format!("{HEADER}{anchor}"));
    assert_absent(&anchor, "Allowed?");
    assert_other_labels_stay_off(&markup);
}

#[test]
fn fallback_two_txids_paint_the_header_once() {
    let markup = test_txid_markup(&fallback_body());
    let first = tx_anchor(TX);
    let second = tx_anchor(SPEND);
    assert_one(&markup, HEADER);
    assert_one(&markup, &first);
    assert_one(&markup, &second);
    assert_eq!(markup.matches("Allowed?").count(), 1, "{markup}");
    assert_eq!(markup, format!("{HEADER}{first}{second}"));
    assert_other_labels_stay_off(&markup);
}

#[test]
fn empty_array_omits_the_header() {
    let markup = test_txid_markup("[]");
    assert_eq!(markup, "");
    assert_absent(&markup, "Allowed?");
    assert_absent(&markup, "<a href=\"/tx/");
}

#[test]
fn empty_string_omits_the_header() {
    let markup = test_txid_markup("");
    assert_eq!(markup, "");
    assert_absent(&markup, "Allowed?");
    assert_absent(&markup, "<a href=\"/tx/");
}

#[test]
fn invalid_json_omits_the_header() {
    let markup = test_txid_markup("{not-json");
    assert_eq!(markup, "");
    assert_absent(&markup, "Allowed?");
    assert_absent(&markup, "<a href=\"/tx/");
}
