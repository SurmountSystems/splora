//! Cypress it() title: shows the new transaction pill when there are new transactions and scrolls to the top when clicked.
//!
//! The spec visits `/txs`, sends six transactions, scrolls until `scrollY`
//! is greater than 0, sends six more, then clicks `[data-cy="new-tx-pill"]`.
//! After that click, `window.scrollY` is 0. The button is rendered only
//! while the pill is visible. `scrollToTop` calls `window.scrollTo` only
//! in a browser. This it() does not read a websocket body or an indexer body.

use splora_frontend_shared::shows_the_new_transaction_pill_when_there_are_new_transactions_and_scrolls_to_the_top_when_clicked as scroll_after_pill_click;

#[test]
fn shows_the_new_transaction_pill_when_there_are_new_transactions_and_scrolls_to_the_top_when_clicked()
 {
    let before = [
        "already-0",
        "already-1",
        "already-2",
        "already-3",
        "already-4",
        "already-5",
    ];
    let after = [
        "later-0", "later-1", "later-2", "later-3", "later-4", "later-5",
    ];
    let scroll_y_before: i64 = 640;

    let scroll_y = scroll_after_pill_click(false, false, scroll_y_before, &before, &after, true);
    assert_eq!(
        scroll_y, 0,
        "cy.window().then((win) => expect(win.scrollY).to.be.eq(0)) after the new-tx-pill click"
    );

    let off_browser =
        scroll_after_pill_click(false, false, scroll_y_before, &before, &after, false);
    assert_eq!(
        off_browser, scroll_y_before,
        "scrollToTop returns before window.scrollTo when the runtime is not a browser"
    );

    let no_new = scroll_after_pill_click(false, false, scroll_y_before, &before, &before, true);
    assert_eq!(
        no_new, scroll_y_before,
        "a hidden pill is not in the document, so the click does not move scrollY"
    );

    let widget = scroll_after_pill_click(true, false, scroll_y_before, &before, &after, true);
    assert_eq!(
        widget, scroll_y_before,
        "the widget does not render the pill, so the click does not move scrollY"
    );

    let manual = scroll_after_pill_click(false, true, scroll_y_before, &before, &after, true);
    assert_eq!(
        manual, scroll_y_before,
        "manual pause hides the pill, so the click does not move scrollY"
    );
}
