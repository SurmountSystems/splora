//! Status footer label from no-query `GET /mempool`.
//!
//! Angular `footer.component.html` paints `Unconfirmed:` beside the mempool
//! count. GPUI `status_footer_paints_unconfirmed_label` requires one line
//! equal to `Unconfirmed:` and the next line is the existing mempool count.
//! The confirmation word `Unconfirmed` stays off this summary. Incoming
//! Transactions, `vB/s`, and `WU/s` stay off. A zero count still paints the
//! label. An empty body, invalid JSON, a body with no count, and a closed
//! indexer notice omit it.

use splora_web::{FAIL_CLOSED, mempool_summary_text};

const LINE: &str = "Unconfirmed:";

fn assert_before_count(markup: &str, count_line: &str) {
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
    let lines: Vec<&str> = markup.lines().collect();
    let label_at = lines.iter().position(|line| *line == LINE);
    let count_at = lines.iter().position(|line| *line == count_line);
    assert!(
        label_at.is_some() && count_at == label_at.map(|index| index + 1),
        "{LINE} was not the line before {count_line}: {markup}"
    );
    assert!(
        !markup.lines().any(|line| line == "Unconfirmed"),
        "the confirmation word was painted on the mempool summary: {markup}"
    );
    assert!(
        !markup.contains("Incoming Transactions"),
        "Incoming Transactions was painted: {markup}"
    );
    assert!(!markup.contains("vB/s"), "vB/s was painted: {markup}");
    assert!(!markup.contains("WU/s"), "WU/s was painted: {markup}");
}

#[test]
fn status_footer_paints_unconfirmed_as_its_own_line() {
    let counted = r#"{"count":4,"vsize":500,"total_fee":90,"fee_histogram":[[2,100]]}"#;
    assert_before_count(
        &mempool_summary_text(counted),
        "mempool count 4 vsize 500 total_fee 90",
    );

    let zero = r#"{"count":0,"vsize":0,"total_fee":0,"fee_histogram":[]}"#;
    assert_before_count(
        &mempool_summary_text(zero),
        "mempool count 0 vsize 0 total_fee 0",
    );

    for body in [
        "",
        "not-json",
        "{}",
        r#"{"vsize":1,"total_fee":0,"fee_histogram":[]}"#,
        FAIL_CLOSED,
        "indexer error down",
    ] {
        let omitted = mempool_summary_text(body);
        assert!(
            !omitted.lines().any(|line| line == LINE),
            "{body} painted {LINE}: {omitted}"
        );
    }
}
