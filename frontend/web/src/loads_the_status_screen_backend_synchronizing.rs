//! Leptos paint of the status-screen backend-synchronizing sentence.
//!
//! The visible text is the shared function's return value. This file does
//! not hard-code that sentence, a percent, or a rate.

/// Paints the backend-synchronizing sentence. The string is the shared function.
pub fn paint_loads_the_status_screen_backend_synchronizing() -> &'static str {
    splora_frontend_shared::loads_the_status_screen_backend_synchronizing()
}
