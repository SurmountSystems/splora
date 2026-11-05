//! Cypress `it('shows the new transaction pill when there are new transactions')`.
//! File: `ref/mempool/frontend/cypress/e2e/mainnet/recent-transactions.spec.ts`,
//! lines 120-135.
//!
//! The it() visits `/txs`, sends six transactions, scrolls until `scrollY`
//! is greater than 0, then sends six more. `[data-cy="new-tx-pill"]` must
//! be visible. The button is absent for the widget, while the manual pause
//! control is on, when `scrollTop` is 0, and when the later ids are not new.
//! One new id reads `1 new transaction`. The count is the deduped ids that
//! arrived after the pause froze the list. This does not read an indexer body.
//! `paint_shows_the_new_transaction_pill_when_there_are_new_transactions` is
//! not in this crate. The call is the missing paint.

const BEFORE: &[&str] = &[
    "already-0",
    "already-1",
    "already-2",
    "already-3",
    "already-4",
    "already-5",
];

const AFTER: &[&str] = &[
    "later-0", "later-1", "later-2", "later-3", "later-4", "later-5",
];

#[test]
fn shows_the_new_transaction_pill_when_there_are_new_transactions() {
    let shown = paint(false, false, 1, BEFORE, AFTER);
    assert_eq!(
        pill_label(&shown).as_deref(),
        Some("6 new transactions"),
        "cy.get('[data-cy=\"new-tx-pill\"]').should('be.visible') with label 6 new transactions, markup was {shown}"
    );
    assert_eq!(
        buffered_count(&shown),
        Some(6),
        "buffered count after six new ids below the fold, markup was {shown}"
    );

    let at_top = paint(false, false, 0, BEFORE, AFTER);
    assert!(
        pill_label(&at_top).is_none(),
        "scrollTop 0 must hide the pill, markup was {at_top}"
    );
    assert_eq!(
        buffered_count(&at_top),
        Some(0),
        "scrollTop 0 leaves the buffered count at 0, markup was {at_top}"
    );

    let same = paint(false, false, 1, BEFORE, BEFORE);
    assert!(
        pill_label(&same).is_none(),
        "the same ids arriving again must hide the pill, markup was {same}"
    );
    assert_eq!(
        buffered_count(&same),
        Some(0),
        "duplicate ids leave the buffered count at 0, markup was {same}"
    );

    let one = paint(false, false, 1, BEFORE, &["later-only"]);
    assert_eq!(
        pill_label(&one).as_deref(),
        Some("1 new transaction"),
        "one new id shows the singular pill, markup was {one}"
    );
    assert_eq!(
        buffered_count(&one),
        Some(1),
        "one new id buffers one transaction, markup was {one}"
    );

    let manual = paint(false, true, 1, BEFORE, AFTER);
    assert!(
        pill_label(&manual).is_none(),
        "manual pause hides the pill, markup was {manual}"
    );
    assert_eq!(
        buffered_count(&manual),
        Some(6),
        "manual pause still buffers the six new ids, markup was {manual}"
    );

    let widget = paint(true, false, 1, BEFORE, AFTER);
    assert!(
        pill_label(&widget).is_none(),
        "the widget hides the pill, markup was {widget}"
    );
    assert_eq!(
        buffered_count(&widget),
        Some(0),
        "the widget does not auto-pause from scroll, markup was {widget}"
    );
}

fn paint(
    widget: bool,
    manual_paused: bool,
    scroll_top: i64,
    txids_before: &[&str],
    txids_after: &[&str],
) -> String {
    splora_web::paint_shows_the_new_transaction_pill_when_there_are_new_transactions(
        widget,
        manual_paused,
        scroll_top,
        txids_before,
        txids_after,
    )
}

/// Label inside `[data-cy="new-tx-pill"]`. Absent when that button is not painted.
fn pill_label(markup: &str) -> Option<&str> {
    const OPEN: &str = "data-cy=\"new-tx-pill\">";
    let start = markup.find(OPEN)? + OPEN.len();
    let rest = &markup[start..];
    let end = rest.find("</button>")?;
    Some(rest[..end].trim())
}

/// Integer painted in `[data-cy="buffered-count"]`.
fn buffered_count(markup: &str) -> Option<usize> {
    const OPEN: &str = "data-cy=\"buffered-count\">";
    let start = markup.find(OPEN)? + OPEN.len();
    let rest = &markup[start..];
    let end = rest.find("</p>")?;
    rest[..end].trim().parse().ok()
}
