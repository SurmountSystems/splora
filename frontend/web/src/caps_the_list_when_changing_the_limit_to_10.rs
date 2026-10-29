//! Web render of the recent list at the default cap and after limit-10.
//! Visible ids come from the shared cap function. This file does not open a window.

/// Markup for the default list cap, then the rows after the limit-10 control.
///
/// Both sections list the ids the shared function returns for `batches`.
/// The first returned vec is the before-limit section. The second is the
/// after-limit section. This paint does not hard-code those ids or their lengths.
pub fn paint_caps_the_list_when_changing_the_limit_to_10(batches: &[[&str; 6]; 9]) -> String {
    let (before, after) =
        splora_frontend_shared::caps_the_list_when_changing_the_limit_to_10(batches);
    let mut markup = String::from("<div data-cy=\"transactions-list\">");
    markup.push_str("<button type=\"button\" data-cy=\"limit-10\"></button>");
    push_txid_section(&mut markup, "before-limit-control", &before);
    push_txid_section(&mut markup, "after-limit-control", &after);
    markup.push_str("</div>");
    markup
}

fn push_txid_section(markup: &mut String, marker: &str, ids: &[&str]) {
    markup.push_str("<section data-cy=\"");
    markup.push_str(marker);
    markup.push_str("\">");
    for id in ids {
        markup.push_str("<tr><td class=\"table-cell-txid\"><a>");
        markup.push_str(id);
        markup.push_str("</a></td></tr>");
    }
    markup.push_str("</section>");
}
