//! Paint for the new-transaction pill on the recent transactions list.
//! Visibility, the label, and the buffered count come from
//! `shows_the_new_transaction_pill_when_there_are_new_transactions`.

use splora_frontend_shared::shows_the_new_transaction_pill_when_there_are_new_transactions;

/// Recent-list markup after the ids on screen, then the ids that follow.
///
/// The pill button is painted only when the shared view says it is visible.
/// The label is that view's label. The count is that view's buffered count.
/// This does not decide the count itself.
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
    let mut markup = String::from("<div data-cy=\"transactions-list\">");
    markup.push_str("<p data-cy=\"buffered-count\">");
    markup.push_str(&pill.buffered_count.to_string());
    markup.push_str("</p>");
    if pill.visible {
        markup.push_str("<button type=\"button\" class=\"new-tx-pill\" data-cy=\"new-tx-pill\">");
        markup.push_str(&pill.label);
        markup.push_str("</button>");
    }
    markup.push_str("</div>");
    markup
}

/// Settled window `scrollY` after the new-transaction pill is clicked.
///
/// The settled value is whatever
/// `shows_the_new_transaction_pill_when_there_are_new_transactions_and_scrolls_to_the_top_when_clicked`
/// returns for these arguments. This paint does not decide 0 itself.
pub fn paint_shows_the_new_transaction_pill_when_there_are_new_transactions_and_scrolls_to_the_top_when_clicked(
    widget: bool,
    manual_paused: bool,
    scroll_y_before: i64,
    txids_before: &[&str],
    txids_after: &[&str],
    is_browser: bool,
) -> i64 {
    splora_frontend_shared::shows_the_new_transaction_pill_when_there_are_new_transactions_and_scrolls_to_the_top_when_clicked(
        widget,
        manual_paused,
        scroll_y_before,
        txids_before,
        txids_after,
        is_browser,
    )
}
