//! Theme choice stored as a local preference.
//!
//! The stored text comes from the shared function. This file only forwards.

/// Forwards the theme argument to shared theme preference storage.
pub fn paint_theme_preference_storage(theme: &str) -> String {
    splora_frontend_shared::theme_preference_storage(theme)
}
