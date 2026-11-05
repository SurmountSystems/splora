//! Native paint of the coinbase scriptsig tooltip on the block transaction list.
//! The tooltip text comes from the shared function. This file does not open a window.

/// Screen lines for the coinbase input on the block transaction list.
///
/// `tbody` is always present. `tooltip-inner` is always present. The next
/// line is the shared string from
/// [`splora_frontend_shared::check_op_return_coinbase_tooltip`]. An empty
/// string is an empty line, so the paint ends with a newline after the text.
/// This file does not open a window.
pub fn paint_check_op_return_coinbase_tooltip(scriptsig: &str) -> String {
    let tooltip = splora_frontend_shared::check_op_return_coinbase_tooltip(scriptsig);
    let mut painted = String::from("tbody");
    painted.push('\n');
    painted.push_str("tooltip-inner");
    painted.push('\n');
    painted.push_str(&tooltip);
    painted.push('\n');
    painted
}
