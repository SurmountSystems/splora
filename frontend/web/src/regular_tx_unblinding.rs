//! Markup for a regular Bitcoin transaction's unblinding line.
//! The hex parse lives in the shared crate.

/// Paints the shared unblinding result for `tx_hex`.
///
/// `None` is markup without the words `unblinding error`.
/// `Some` is that error string itself.
pub fn regular_tx_unblinding_markup(tx_hex: &str) -> String {
    match splora_frontend_shared::unblinding_error_for_regular_tx(tx_hex) {
        None => String::new(),
        Some(error) => error,
    }
}
