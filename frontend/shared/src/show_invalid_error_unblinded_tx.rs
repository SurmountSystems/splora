//! Cypress sentence when matched Liquid commitments are short of the stored set.
//!
//! Angular throws `Invalid blinding data.` Cypress `contains` sees the
//! `Error:` prefix. This is not the invalid-hex sentence.

/// Static sentence for an unblinded transaction with incomplete blinding data.
pub fn show_invalid_error_unblinded_tx() -> &'static str {
    "Error: Invalid blinding data."
}
