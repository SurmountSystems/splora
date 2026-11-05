//! Tooltip paint for one OP_RETURN badge on the block transaction list.
//! The inner text comes from `check_op_return_tx_tooltip`. This file does not decode the script.

use splora_frontend_shared::check_op_return_tx_tooltip;

/// Paints `.tooltip-inner` when the shared function returns tooltip text.
///
/// The text is that return value. The span is absent when the shared
/// function returns `None`. This does not decode the asm itself.
pub fn paint_check_op_return_tx_tooltip(
    network: &str,
    script_pubkey: &str,
    script_pubkey_type: &str,
    script_pubkey_asm: &str,
) -> String {
    let Some(text) = check_op_return_tx_tooltip(
        network,
        script_pubkey,
        script_pubkey_type,
        script_pubkey_asm,
    ) else {
        return String::new();
    };
    let mut markup = String::from("<span class=\"tooltip-inner\">");
    markup.push_str(&text);
    markup.push_str("</span>");
    markup
}
