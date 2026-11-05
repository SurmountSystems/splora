//! Leptos paint of the latest blocks host attribute.
//!
//! The visible text is the shared function's return value. This file does
//! not hard-code that attribute, a percent, a block count, or a price.

/// Paints the latest blocks host attribute. The string is the shared function.
pub fn paint_shows_the_latest_blocks() -> &'static str {
    splora_frontend_shared::shows_the_latest_blocks()
}
