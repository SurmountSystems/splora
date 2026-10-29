//! Pause-icon paint for the recent transactions list.
//! Visible ids come from `visible_txids_after_pause`. This file does not open a window.

use splora_frontend_shared::visible_txids_after_pause;

/// Paints the pause control and the transaction rows that stay on screen.
///
/// Cypress reads `data-cy="transactions-list"`, a pause control whose
/// attribute contains `btn-pause`, and the text of each `.table-cell-txid a`.
pub fn paint_pauses_updates_when_clicking_the_pause_icon(
    paused_ids: &[&str],
    later_ids: &[&str],
) -> String {
    let visible = visible_txids_after_pause(paused_ids, later_ids);
    let mut markup = String::from("<div data-cy=\"transactions-list\">");
    markup.push_str("<button type=\"button\" data-cy=\"btn-pause\">Pause</button>");
    markup.push_str("<table><tbody>");
    for id in visible {
        markup.push_str("<tr><td class=\"table-cell-txid\"><a>");
        markup.push_str(id);
        markup.push_str("</a></td></tr>");
    }
    markup.push_str("</tbody></table></div>");
    markup
}
