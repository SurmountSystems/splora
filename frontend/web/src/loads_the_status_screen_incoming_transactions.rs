//! Leptos paint of the status-screen incoming-transactions label.
//!
//! The visible text is the shared function's return value. This file does
//! not hard-code that label, a rate, or a fee figure.

/// Paints the incoming-transactions label. The string is the shared function.
pub fn paint_loads_the_status_screen_incoming_transactions() -> &'static str {
    splora_frontend_shared::loads_the_status_screen_incoming_transactions()
}
