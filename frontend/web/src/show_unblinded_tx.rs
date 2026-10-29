//! Markup for an unblinded Liquid transaction.
//! Amount lines come from the shared preview. This module does not add the fee.

/// Paints `unblinded_tx_preview` for `fragment` and `fee_sats`.
///
/// An empty preview stays empty. A non-empty preview is a vin table, then a
/// vout table, and each amount row has class `assetBox`. When `fee_sats` is
/// positive the shared preview appends the input after the outputs, so that
/// last line is the vin amount.
pub fn show_unblinded_tx_markup(fragment: &str, fee_sats: u64) -> String {
    let preview = splora_frontend_shared::unblinded_tx_preview(fragment, fee_sats);
    if preview.is_empty() {
        return preview;
    }
    let lines: Vec<&str> = preview.lines().filter(|line| !line.is_empty()).collect();
    if lines.is_empty() {
        return String::new();
    }
    let mut markup = String::from("<table class=\"table-tx-vin\"><tbody>");
    let outputs: &[&str] = if fee_sats > 0 && lines.len() >= 2 {
        push_asset_row(&mut markup, lines[lines.len() - 1]);
        &lines[..lines.len() - 1]
    } else {
        lines.as_slice()
    };
    markup.push_str("</tbody></table><table class=\"table-tx-vout\"><tbody>");
    for amount in outputs {
        push_asset_row(&mut markup, amount);
    }
    markup.push_str("</tbody></table>");
    markup
}

fn push_asset_row(markup: &mut String, amount: &str) {
    markup.push_str("<tr class=\"assetBox\"><td class=\"amount\">");
    markup.push_str(amount);
    markup.push_str("</td></tr>");
}
