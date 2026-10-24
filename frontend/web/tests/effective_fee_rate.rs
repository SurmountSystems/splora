//! Effective fee rate header on test-transaction results from `POST /txs/test`.
//!
//! Angular paints `<th class="rate">Effective fee rate</th>` once on the
//! results table. The number is `fees.effective-feerate` on the test-tx JSON
//! the client already parses. The header stays off `test_txid_markup`, which
//! keeps the Allowed? cell, the txid anchors, and one line equal to Allowed?.
//! A row with no fees object, an
//! empty array, a blank body, and invalid JSON do not paint the header.

use splora_frontend_shared::broadcast_path;
use splora_web::{submission_links, test_transactions_request, test_txid_markup};

const TX: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
const WTX: &str = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
const OTHER: &str = "cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc";
const OTHER_WTX: &str = "dddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddd";
const HEADER: &str = "<th class=\"rate\">Effective fee rate</th>";
const ALLOWED: &str = "<th class=\"allowed\">Allowed?</th>";

fn test_path() -> String {
    test_transactions_request().path
}

fn tx_anchor(txid: &str) -> String {
    format!("<a href=\"/tx/{txid}\">{txid}</a>")
}

fn row_with_fees(txid: &str, wtxid: &str, rate: &str) -> String {
    format!(
        r#"{{"txid":"{txid}","wtxid":"{wtxid}","allowed":false,"vsize":141,"fees":{{"base":0.00001,"effective-feerate":{rate},"effective-includes":["{txid}"]}},"reject-reason":"min relay fee not met"}}"#
    )
}

fn with_fees(rate: &str) -> String {
    format!("[{}]", row_with_fees(TX, WTX, rate))
}

fn without_fees() -> String {
    format!(r#"[{{"txid":"{TX}","wtxid":"{WTX}","allowed":true}}]"#)
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
fn served_effective_feerate_paints_the_header_once() {
    let body = with_fees("1.0");
    let markup = submission_links(&test_path(), &body);
    let anchor = tx_anchor(TX);
    assert_one(&markup, HEADER);
    assert_eq!(markup.matches("Effective fee rate").count(), 2, "{markup}");
    assert_one(&markup, ALLOWED);
    assert_one(&markup, &anchor);
    assert!(
        markup.find(ALLOWED).unwrap() < markup.find(HEADER).unwrap(),
        "Allowed? stays before Effective fee rate: {markup}"
    );
    assert!(
        markup.find(HEADER).unwrap() < markup.find(&anchor).unwrap(),
        "the header belongs before the txid: {markup}"
    );
    assert_absent(&test_txid_markup(&body), "Effective fee rate");
    assert_rejection_reason_line(&markup);
    assert_absent(&markup, "sat/vB");
}

#[test]
fn zero_rate_still_paints_the_header() {
    let markup = submission_links(&test_path(), &with_fees("0.0"));
    assert_one(&markup, HEADER);
    assert_eq!(markup.matches("Effective fee rate").count(), 2, "{markup}");
    assert_absent(&test_txid_markup(&with_fees("0.0")), "Effective fee rate");
}

#[test]
fn two_rows_paint_the_header_once() {
    let body = format!(
        "[{},{}]",
        row_with_fees(TX, WTX, "1.0"),
        row_with_fees(OTHER, OTHER_WTX, "2.5")
    );
    let markup = submission_links(&test_path(), &body);
    assert_one(&markup, HEADER);
    assert_one(&markup, &tx_anchor(TX));
    assert_one(&markup, &tx_anchor(OTHER));
    assert_eq!(markup.matches("Allowed?").count(), 2, "{markup}");
    assert_rejection_reason_line(&markup);
}

fn assert_rejection_reason_line(markup: &str) {
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
fn one_row_with_fees_among_rows_without_fees_paints_once() {
    let body = format!(
        r#"[{},{{"txid":"{OTHER}","wtxid":"{OTHER_WTX}","allowed":true}}]"#,
        row_with_fees(TX, WTX, "1.0")
    );
    let markup = submission_links(&test_path(), &body);
    assert_one(&markup, HEADER);
    assert_one(&markup, &tx_anchor(TX));
    assert_one(&markup, &tx_anchor(OTHER));
}

#[test]
fn missing_fees_omits_the_header() {
    let body = without_fees();
    let markup = submission_links(&test_path(), &body);
    assert_one(&markup, ALLOWED);
    assert_one(&markup, &tx_anchor(TX));
    assert_absent(&markup, "Effective fee rate");
    assert_absent(&markup, HEADER);
    assert_eq!(markup, test_txid_markup(&body));
}

#[test]
fn empty_array_omits_the_header() {
    let markup = submission_links(&test_path(), "[]");
    assert_eq!(markup, "");
    assert_absent(&markup, "Effective fee rate");
    assert_absent(&markup, "Allowed?");
}

#[test]
fn empty_string_omits_the_header() {
    let markup = submission_links(&test_path(), "");
    assert_eq!(markup, "");
    assert_absent(&markup, "Effective fee rate");
}

#[test]
fn invalid_json_omits_the_header() {
    let markup = submission_links(&test_path(), "{not-json");
    assert_eq!(markup, "");
    assert_absent(&markup, "Effective fee rate");
}

#[test]
fn broadcast_path_omits_the_header() {
    let markup = submission_links(broadcast_path(), &with_fees("1.0"));
    assert_absent(&markup, "Effective fee rate");
}

/// Whole line beside the effective fee rate header.
///
/// A line equal to Effective fee rate passes. The header cell stays. A row
/// with no fees, an empty list, an empty body, and invalid JSON omit the
/// words. Broadcast does not paint them. This is not a sat/vB sentence.
#[test]
fn served_fee_paints_effective_fee_rate_as_its_own_line() {
    const LINE: &str = "Effective fee rate";
    let body = with_fees("1.0");
    let markup = submission_links(&test_path(), &body);
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
        markup.contains("<th class=\"rate\">Effective fee rate</th>"),
        "the header cell was dropped: {markup}"
    );
    assert!(
        !markup
            .lines()
            .any(|line| line.starts_with("Effective fee rate ")),
        "{LINE} was painted as a longer sentence: {markup}"
    );
    assert_ne!(LINE, "effective_feerate");
    assert_ne!(LINE, "effective-feerate");
    assert_absent(&markup, "sat/vB");
    assert_absent(&test_txid_markup(&body), LINE);
    let rate_at = markup.lines().position(|line| line == LINE);
    let anchor = tx_anchor(TX);
    let anchor_at = markup.lines().position(|line| line.contains(&anchor));
    assert!(
        rate_at.is_some() && anchor_at.is_some() && rate_at < anchor_at,
        "{LINE} was not before the txid: {markup}"
    );
    assert!(!submission_links(&test_path(), &without_fees()).contains(LINE));
    assert!(!submission_links(&test_path(), "[]").contains(LINE));
    assert!(!submission_links(&test_path(), "").contains(LINE));
    assert!(!submission_links(&test_path(), "{not-json").contains(LINE));
    assert!(!submission_links(broadcast_path(), &body).contains(LINE));
}
