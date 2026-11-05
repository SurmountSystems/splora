//! Native paint of the new-transaction pill on the recent list.
//! The label comes from the shared pill function. This file does not open a window.

use splora_frontend_shared::shows_the_new_transaction_pill_when_there_are_new_transactions;

/// Screen text after the ids on screen, a scroll sample, and a later batch.
///
/// The pill line is present only when the shared function says the button
/// is visible. The label line is that function's label.
pub fn paint_shows_the_new_transaction_pill_when_there_are_new_transactions(
    widget: bool,
    manual_paused: bool,
    scroll_top: i64,
    txids_before: &[&str],
    txids_after: &[&str],
) -> String {
    let pill = shows_the_new_transaction_pill_when_there_are_new_transactions(
        widget,
        manual_paused,
        scroll_top,
        txids_before,
        txids_after,
    );
    let mut painted = String::from("data-cy=\"transactions-list\"");
    if pill.visible {
        painted.push('\n');
        painted.push_str("data-cy=\"new-tx-pill\"");
        painted.push('\n');
        painted.push_str(&pill.label);
    }
    painted
}

/// Screen text after a click on the new-transaction pill.
///
/// Pill lines come from the shared pill function. The settled scroll line
/// is the `scrollY` returned by the shared click function. A hidden pill
/// leaves that value at `scroll_y_before`. This file does not open a window.
pub fn paint_shows_the_new_transaction_pill_when_there_are_new_transactions_and_scrolls_to_the_top_when_clicked(
    widget: bool,
    manual_paused: bool,
    scroll_y_before: i64,
    txids_before: &[&str],
    txids_after: &[&str],
    is_browser: bool,
) -> String {
    let scroll_y = splora_frontend_shared::shows_the_new_transaction_pill_when_there_are_new_transactions_and_scrolls_to_the_top_when_clicked(
        widget,
        manual_paused,
        scroll_y_before,
        txids_before,
        txids_after,
        is_browser,
    );
    let pill = shows_the_new_transaction_pill_when_there_are_new_transactions(
        widget,
        manual_paused,
        scroll_y_before,
        txids_before,
        txids_after,
    );
    let mut painted = String::from("data-cy=\"transactions-list\"");
    if pill.visible {
        painted.push('\n');
        painted.push_str("data-cy=\"new-tx-pill\"");
        painted.push('\n');
        painted.push_str(&pill.label);
    }
    painted.push('\n');
    painted.push_str("settled scroll ");
    painted.push_str(&scroll_y.to_string());
    painted
}
