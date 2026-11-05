//! Native paint of one theme option label.
//! This module does not open a window.

/// Forwards the value into the shared theme option label.
pub fn paint_theme_option_label(value: &str) -> String {
    splora_frontend_shared::theme_option_label(value)
}
