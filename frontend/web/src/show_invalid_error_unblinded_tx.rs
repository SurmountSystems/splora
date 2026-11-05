//! Leptos paint of the unblinded-transaction invalid-blinding sentence.
//!
//! The visible text is the shared function's return value. This file does
//! not hard-code that sentence.

/// Paints the invalid-blinding sentence for an unblinded transaction.
/// The string is the shared function.
pub fn paint_show_invalid_error_unblinded_tx() -> &'static str {
    splora_frontend_shared::show_invalid_error_unblinded_tx()
}
