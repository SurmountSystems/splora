//! Tooltip paint for the coinbase input scriptsig on the block transaction list.
//! The inner text comes from `check_op_return_coinbase_tooltip`. This file does not decode the script.

use splora_frontend_shared::check_op_return_coinbase_tooltip;

/// Paints `.tooltip-inner` when the shared function returns tooltip text.
///
/// The text is that return value. The span is absent when the shared
/// function returns an empty string. This does not decode the scriptsig.
pub fn paint_check_op_return_coinbase_tooltip(scriptsig: &str) -> String {
    let text = check_op_return_coinbase_tooltip(scriptsig);
    if text.is_empty() {
        return String::new();
    }
    let mut markup = String::from("<span class=\"tooltip-inner\">");
    markup.push_str(&text);
    markup.push_str("</span>");
    markup
}
