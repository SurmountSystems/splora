//! Leptos paint of the pools pie chart host attribute.
//!
//! The visible text is the shared function's return value. This file does
//! not hard-code that attribute, a pool name, a share, a hashrate,
//! or a block count.

/// Paints the pools pie chart host attribute. The string is the shared function.
pub fn paint_shows_the_pools_pie_chart() -> &'static str {
    splora_frontend_shared::shows_the_pools_pie_chart()
}
