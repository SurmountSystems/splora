//! Leptos paint of the peg-out address label.
//!
//! The visible text is the shared function's return value. This file does
//! not hard-code that label.

/// Paints the peg-out address label. The string is the shared function.
pub fn paint_loads_peg_out_addresses() -> &'static str {
    splora_frontend_shared::loads_peg_out_addresses()
}
