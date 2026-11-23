//! Markup for an unblinded Liquid transaction.
//! Amount lines come from the shared preview. This module does not add the fee.

/// Paints `unblinded_tx_preview` for `fragment` and `fee_sats`.
///
/// An empty preview stays empty. An empty fragment is the confidential
/// record: `ng-star-inserted` rows, not `assetBox`. Fee does not move the
/// last confidential word onto the vin. Parsed amount lines are a vin table,
/// then a vout table, and each amount row has class `assetBox`. When
/// `fee_sats` is positive the shared preview appends the input after the
/// outputs, so that last line is the vin amount.
pub fn show_unblinded_tx_markup(fragment: &str, fee_sats: u64) -> String {
    let preview = splora_frontend_shared::unblinded_tx_preview(fragment, fee_sats);
    if preview.is_empty() {
        return preview;
    }
    let lines: Vec<&str> = preview.lines().filter(|line| !line.is_empty()).collect();
    if lines.is_empty() {
        return String::new();
    }
    let amounts: Vec<&str> = lines
        .iter()
        .copied()
        .filter(|line| is_parsed_amount_line(line))
        .collect();
    if amounts.is_empty() {
        return confidential_inserted_markup(&preview);
    }
    let mut markup = String::from("<table class=\"table-tx-vin\"><tbody>");
    let outputs: &[&str] = if fee_sats > 0 && amounts.len() >= 2 {
        push_asset_row(&mut markup, amounts[amounts.len() - 1]);
        &amounts[..amounts.len() - 1]
    } else {
        amounts.as_slice()
    };
    markup.push_str("</tbody></table><table class=\"table-tx-vout\"><tbody>");
    for amount in outputs {
        push_asset_row(&mut markup, amount);
    }
    markup.push_str("</tbody></table>");
    markup
}

/// True when `line` is an eight-decimal LBTC amount from the shared preview.
fn is_parsed_amount_line(line: &str) -> bool {
    line.ends_with(" LBTC")
}

/// Empty-fragment confidential record. The first word is the input. Later
/// words are outputs. Fee is not applied, so the last word is not the vin.
fn confidential_inserted_markup(preview: &str) -> String {
    let labels: Vec<&str> = preview
        .split_whitespace()
        .filter(|token| *token == "Confidential")
        .collect();
    if labels.is_empty() {
        return String::new();
    }
    let mut markup = String::from("<table class=\"table-tx-vin\"><tbody>");
    push_inserted_row(&mut markup, labels[0]);
    markup.push_str("</tbody></table><table class=\"table-tx-vout\"><tbody>");
    for label in labels.iter().skip(1) {
        push_inserted_row(&mut markup, label);
    }
    markup.push_str("</tbody></table>");
    markup
}

fn push_inserted_row(markup: &mut String, amount: &str) {
    markup.push_str("<tr class=\"ng-star-inserted\"><td class=\"amount\">");
    markup.push_str(amount);
    markup.push_str("</td></tr>");
}

fn push_asset_row(markup: &mut String, amount: &str) {
    markup.push_str("<tr class=\"assetBox\"><td class=\"amount\">");
    markup.push_str(amount);
    markup.push_str("</td></tr>");
}
