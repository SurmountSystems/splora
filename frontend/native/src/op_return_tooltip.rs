//! Native paint of the OP_RETURN badge tooltip on the block transaction list.
//! The tooltip text comes from the shared function. This file does not open a window.

/// Screen lines for one output on the block transaction list.
///
/// `tbody` is always present. `tooltip-inner` and the next line are present
/// only when [`splora_frontend_shared::check_op_return_tx_tooltip`] returns
/// text. That next line is the shared string. An empty string is an empty
/// line, so the paint ends with a newline after the text. This file does
/// not open a window.
pub fn paint_check_op_return_tx_tooltip(
    network: &str,
    script_pubkey: &str,
    script_pubkey_type: &str,
    script_pubkey_asm: &str,
) -> String {
    let tooltip = splora_frontend_shared::check_op_return_tx_tooltip(
        network,
        script_pubkey,
        script_pubkey_type,
        script_pubkey_asm,
    );
    let mut painted = String::from("tbody");
    if let Some(text) = tooltip {
        painted.push('\n');
        painted.push_str("tooltip-inner");
        painted.push('\n');
        painted.push_str(&text);
        painted.push('\n');
    }
    painted
}
