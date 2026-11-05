//! Settled window scroll after the new-transaction pill is clicked.
//!
//! The button is in the document only while the pill is visible.
//! `scrollToTop` returns before `window.scrollTo` when the runtime is not
//! a browser. In a browser that call uses top 0, and the settled `scrollY`
//! is 0. This is not a websocket body and not an indexer body.

use crate::shows_the_new_transaction_pill_when_there_are_new_transactions;

/// Settled window `scrollY` after the pill click.
///
/// `scroll_y_before` is the window position when the button is clicked.
/// `txids_before` is the list frozen once scroll left the top.
/// `txids_after` is the later batch. `is_browser` is the Angular
/// `stateService.isBrowser` guard. A hidden pill is not in the document,
/// so the click does not run and `scroll_y_before` stays. A visible pill
/// off a browser hits that guard and also stays. A visible pill in a
/// browser settles at 0.
pub fn shows_the_new_transaction_pill_when_there_are_new_transactions_and_scrolls_to_the_top_when_clicked(
    widget: bool,
    manual_paused: bool,
    scroll_y_before: i64,
    txids_before: &[&str],
    txids_after: &[&str],
    is_browser: bool,
) -> i64 {
    let pill = shows_the_new_transaction_pill_when_there_are_new_transactions(
        widget,
        manual_paused,
        scroll_y_before,
        txids_before,
        txids_after,
    );
    if pill.visible && is_browser {
        0
    } else {
        scroll_y_before
    }
}
