//! Leptos paint of the status-screen block label.
//!
//! The visible text is the shared function's return value. This file does
//! not hard-code that label, a count, or a size.

/// Paints the block label. The string is the shared function.
pub fn paint_loads_the_status_screen_block() -> &'static str {
    splora_frontend_shared::loads_the_status_screen_block()
}
