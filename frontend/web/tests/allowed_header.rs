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
        "TXID",
        "Raw hex",
        "Test Transactions",
        "✅",
        "❌",
    ] {
        assert_absent(markup, needle);
    }
    // One whole line. The header cell does not contain these words.
    const LINE: &str = "Rejection reason";
    assert_eq!(
        markup.lines().filter(|line| *line == LINE).count(),
        1,
        "{markup}"
    );
    let rest = markup.replacen(&format!("\n{LINE}\n"), "\n", 1);
    assert!(
        !rest.contains(LINE),
        "Rejection reason leaked beside its line: {rest}"
    );
}

#[test]
fn parsed_allowed_true_paints_the_header_once() {
    let markup = test_txid_markup(&allowed_true_body());
    let anchor = tx_anchor(TX);
    assert_one(&markup, HEADER);
    assert_one(&markup, &anchor);
    assert_eq!(markup.matches("Allowed?").count(), 2, "{markup}");
    assert_eq!(
        markup,
        format!("{HEADER}{anchor}\nAllowed?\nRejection reason\n")
    );
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
    assert_eq!(markup.matches("Allowed?").count(), 2, "{markup}");
    assert_eq!(
        markup,
        format!("{HEADER}{first}{second}\nAllowed?\nRejection reason\n")
    );
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

/// Whole line beside the test-transaction header.
///
/// A line equal to Allowed? passes. The header cell stays. An empty list,
/// an empty body, and invalid JSON omit the words. This is not an ordinary
/// transaction screen.
#[test]
fn parsed_allowed_true_paints_allowed_as_its_own_line() {
    const LINE: &str = "Allowed?";
    let markup = test_txid_markup(&allowed_true_body());
    assert!(
        markup.lines().any(|line| line == LINE),
        "no line equals {LINE}: {markup}"
    );
    assert_eq!(
        markup.lines().filter(|line| *line == LINE).count(),
        1,
        "{LINE} was not painted once: {markup}"
    );
    assert!(
        markup.contains(HEADER),
        "the header cell was dropped: {markup}"
    );
    assert_eq!(test_txid_markup("[]"), "");
    assert!(!test_txid_markup("").contains(LINE));
    assert!(!test_txid_markup("{not-json").contains(LINE));
}
