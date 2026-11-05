//! Cypress it() title: shows the new transaction pill when there are new transactions and scrolls to the top when clicked.
//!
//! The it() visits `/txs`, sends six transactions, scrolls until
//! `window.scrollY` is greater than 0, sends six more, then clicks
//! `[data-cy="new-tx-pill"]`. After that click, `window.scrollY` is 0.
//! The button is in the document only while the pill is visible.
//! `scrollToTop` calls `window.scrollTo` only in a browser. A hidden
//! pill, the widget, and manual pause leave the window at the scroll
//! from before the click. This it() does not read a websocket body or
//! an indexer body.
//! `paint_shows_the_new_transaction_pill_when_there_are_new_transactions_and_scrolls_to_the_top_when_clicked`
//! is not defined on `splora_native`. This test does not open a window.

const CYPRESS_IT: &str = "shows the new transaction pill when there are new transactions and scrolls to the top when clicked";

fn paint(
    widget: bool,
    manual_paused: bool,
    scroll_y_before: i64,
    before: &[&str],
    after: &[&str],
    is_browser: bool,
) -> String {
    splora_native::paint_shows_the_new_transaction_pill_when_there_are_new_transactions_and_scrolls_to_the_top_when_clicked(
        widget,
        manual_paused,
        scroll_y_before,
        before,
        after,
        is_browser,
    )
}

fn has_line(painted: &str, line: &str) -> bool {
    painted.lines().any(|row| row == line)
}

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

    let painted = paint(false, false, scroll_y_before, &before, &after, true);
    assert!(
        has_line(&painted, "data-cy=\"transactions-list\"")
            && has_line(&painted, "data-cy=\"new-tx-pill\"")
            && has_line(&painted, "6 new transactions")
            && has_line(&painted, "settled scroll 0"),
        "{CYPRESS_IT}: cy.window().then((win) => expect(win.scrollY).to.be.eq(0)) after the new-tx-pill click; got {painted:?}"
    );
    assert_ne!(
        painted, "0",
        "{CYPRESS_IT}: painted text must not be only the string 0; got {painted:?}"
    );

    let off_browser = paint(false, false, scroll_y_before, &before, &after, false);
    assert!(
        has_line(&off_browser, "data-cy=\"transactions-list\"")
            && has_line(&off_browser, "data-cy=\"new-tx-pill\"")
            && has_line(&off_browser, "6 new transactions")
            && has_line(&off_browser, "settled scroll 640")
            && !has_line(&off_browser, "settled scroll 0"),
        "{CYPRESS_IT}: scrollToTop returns before window.scrollTo when the runtime is not a browser; got {off_browser:?}"
    );

    let no_new = paint(false, false, scroll_y_before, &before, &before, true);
    assert!(
        has_line(&no_new, "data-cy=\"transactions-list\"")
            && !has_line(&no_new, "data-cy=\"new-tx-pill\"")
            && has_line(&no_new, "settled scroll 640")
            && !has_line(&no_new, "settled scroll 0"),
        "{CYPRESS_IT}: a hidden pill is not in the document, so the click does not move scrollY; got {no_new:?}"
    );

    let widget = paint(true, false, scroll_y_before, &before, &after, true);
    assert!(
        has_line(&widget, "data-cy=\"transactions-list\"")
            && !has_line(&widget, "data-cy=\"new-tx-pill\"")
            && has_line(&widget, "settled scroll 640")
            && !has_line(&widget, "settled scroll 0"),
        "{CYPRESS_IT}: the widget does not render the pill, so the click does not move scrollY; got {widget:?}"
    );

    let manual = paint(false, true, scroll_y_before, &before, &after, true);
    assert!(
        has_line(&manual, "data-cy=\"transactions-list\"")
            && !has_line(&manual, "data-cy=\"new-tx-pill\"")
            && has_line(&manual, "settled scroll 640")
            && !has_line(&manual, "settled scroll 0"),
        "{CYPRESS_IT}: manual pause hides the pill, so the click does not move scrollY; got {manual:?}"
    );
}
