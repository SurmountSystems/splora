//! Recent-list rows after a later batch of txids arrives.
//! Visible ids come from `updates_the_transaction_list_over_time`.
//! This file does not open a window.

/// Paints `[data-cy="transactions-list"]` for the later batch.
///
/// Anchor text is the id that function returns, so the leading characters
/// of a new txid are in that text. This paint does not take a fixed prefix.
pub fn paint_updates_the_transaction_list_over_time(
    first: &[&str; 6],
    second: &[&str; 6],
) -> String {
    let (_after_first, after_second) =
        splora_frontend_shared::updates_the_transaction_list_over_time(first, second);
    let mut markup = String::from("<div data-cy=\"transactions-list\">");
    markup.push_str("<table><tbody>");
    for id in after_second {
        markup.push_str("<tr><td class=\"table-cell-txid\"><a>");
        markup.push_str(id);
        markup.push_str("</a></td></tr>");
    }
    markup.push_str("</tbody></table></div>");
    markup
}
