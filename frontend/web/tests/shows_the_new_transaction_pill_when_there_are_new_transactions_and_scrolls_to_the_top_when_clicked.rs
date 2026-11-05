//! Cypress `it('shows the new transaction pill when there are new transactions and scrolls to the top when clicked')`.
//! File: `ref/mempool/frontend/cypress/e2e/mainnet/recent-transactions.spec.ts`,
//! lines 137-156.
//!
//! The it() visits `/txs`, sends six transactions, scrolls until `scrollY`
//! is greater than 0, sends six more, then clicks `[data-cy="new-tx-pill"]`.
//! After that click, `window.scrollY` is 0. The button is rendered only
//! while the pill is visible. `scrollToTop` calls `window.scrollTo` only
//! in a browser. A hidden pill, the widget, and manual pause leave the
//! window where it was. This does not read an indexer body.
//! `paint_shows_the_new_transaction_pill_when_there_are_new_transactions_and_scrolls_to_the_top_when_clicked`
//! is not in this crate. The call is the missing paint.

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
fn shows_the_new_transaction_pill_when_there_are_new_transactions_and_scrolls_to_the_top_when_clicked()
 {
    let scroll_y_before: i64 = 640;
    let scroll_y = splora_web::paint_shows_the_new_transaction_pill_when_there_are_new_transactions_and_scrolls_to_the_top_when_clicked(
        false,
        false,
        scroll_y_before,
        BEFORE,
        AFTER,
        true,
    );
    assert_eq!(
        scroll_y, 0,
        "cy.window().then((win) => expect(win.scrollY).to.be.eq(0)) after the new-tx-pill click"
    );

    let off_browser = splora_web::paint_shows_the_new_transaction_pill_when_there_are_new_transactions_and_scrolls_to_the_top_when_clicked(
        false,
        false,
        scroll_y_before,
        BEFORE,
        AFTER,
        false,
    );
    assert_eq!(
        off_browser, 640,
        "the same paint keeps 640 when is_browser is false, got {off_browser}"
    );

    let no_new = splora_web::paint_shows_the_new_transaction_pill_when_there_are_new_transactions_and_scrolls_to_the_top_when_clicked(
        false,
        false,
        scroll_y_before,
        BEFORE,
        BEFORE,
        true,
    );
    assert_eq!(
        no_new, 640,
        "the same paint keeps 640 when the later ids are not new, got {no_new}"
    );

    let widget = splora_web::paint_shows_the_new_transaction_pill_when_there_are_new_transactions_and_scrolls_to_the_top_when_clicked(
        true,
        false,
        scroll_y_before,
        BEFORE,
        AFTER,
        true,
    );
    assert_eq!(
        widget, 640,
        "the same paint keeps 640 when the list is a widget, got {widget}"
    );

    let manual = splora_web::paint_shows_the_new_transaction_pill_when_there_are_new_transactions_and_scrolls_to_the_top_when_clicked(
        false,
        true,
        scroll_y_before,
        BEFORE,
        AFTER,
        true,
    );
    assert_eq!(
        manual, 640,
        "the same paint keeps 640 when manual pause is on, got {manual}"
    );
}
