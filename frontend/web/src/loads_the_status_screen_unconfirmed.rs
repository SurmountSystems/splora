//! Leptos paint of the status-screen unconfirmed label.
//!
//! The visible text is the shared function's return value. This file does
//! not hard-code that label, a count, or a colon.

/// Paints the unconfirmed label. The string is the shared function.
pub fn paint_loads_the_status_screen_unconfirmed() -> &'static str {
    splora_frontend_shared::loads_the_status_screen_unconfirmed()
}
