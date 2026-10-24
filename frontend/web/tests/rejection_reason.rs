//! Rejection reason header on test-transaction results from `POST /txs/test`.
//!
//! Angular `test-transactions.component.html` paints
//! `<th class="reason">Rejection reason</th>` when the result list is not empty.
//! GPUI `screen_paints_rejection_reason` requires one line equal to
//! `Rejection reason`. The data token `reject-reason` is not that line.
//! An empty list, an empty body, and invalid JSON omit the words. Broadcast
//! does not paint them.

use splora_frontend_shared::broadcast_path;
use splora_web::{submission_links, test_transactions_request, test_txid_markup};

const TX: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
const LINE: &str = "Rejection reason";

fn rejected_body() -> String {
    format!(r#"[{{"txid":"{TX}","wtxid":"{TX}","allowed":false,"reject-reason":"dust"}}]"#)
}

fn allowed_body() -> String {
    format!(r#"[{{"txid":"{TX}","allowed":true}}]"#)
}

fn assert_one_line(markup: &str) {
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
        markup
            .lines()
            .filter(|line| line.contains(LINE))
            .all(|line| line == LINE),
        "{LINE} was painted inside a longer sentence: {markup}"
    );
    assert_eq!(markup.matches(LINE).count(), 1, "{markup}");
    assert_ne!(LINE, "reject-reason");
    let rest = markup.replacen(&format!("\n{LINE}\n"), "\n", 1);
    assert!(
        !rest.contains(LINE),
        "{LINE} leaked beside its own line: {rest}"
    );
}

/// Whole line beside the test-transaction result table.
///
/// A line equal to Rejection reason passes. Allowed? stays. The header is
/// present whenever a txid parses, including a row that has no reject-reason.
#[test]
fn parsed_results_paint_rejection_reason_as_its_own_line() {
    let markup = test_txid_markup(&rejected_body());
    assert_one_line(&markup);
    assert!(
        markup.contains("<th class=\"allowed\">Allowed?</th>"),
        "Allowed? header was dropped: {markup}"
    );
    assert!(
        markup.lines().any(|line| line == "Allowed?"),
        "Allowed? line was dropped: {markup}"
    );
    assert!(
        markup.contains(&format!("<a href=\"/tx/{TX}\">{TX}</a>")),
        "the txid anchor was dropped: {markup}"
    );
    assert_one_line(&test_txid_markup(&allowed_body()));
    let composed = submission_links(&test_transactions_request().path, &rejected_body());
    assert_one_line(&composed);
    assert!(
        !submission_links(broadcast_path(), &rejected_body()).contains(LINE),
        "broadcast painted {LINE}"
    );
    assert_eq!(test_txid_markup("[]"), "");
    assert!(!test_txid_markup("").contains(LINE));
    assert!(!test_txid_markup("{not-json").contains(LINE));
    assert!(!submission_links(&test_transactions_request().path, "[]").contains(LINE));
}
