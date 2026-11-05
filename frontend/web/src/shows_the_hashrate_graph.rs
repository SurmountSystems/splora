//! Leptos paint of the hashrate graph host attribute.
//!
//! The visible text is the shared function's return value. This file does
//! not hard-code that attribute or a hashrate figure.

/// Paints the hashrate graph host attribute. The string is the shared function.
pub fn paint_shows_the_hashrate_graph() -> &'static str {
    splora_frontend_shared::shows_the_hashrate_graph()
}
