//! Native paint of the theme preference storage.
//! This module does not open a window.

/// Forwards the theme into the shared theme preference storage.
pub fn paint_theme_preference_storage(theme: &str) -> String {
    splora_frontend_shared::theme_preference_storage(theme)
}
