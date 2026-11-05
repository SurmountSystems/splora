//! Native paint of one footer timezone option label.
//! This module does not open a window.

/// Forwards the offset and the name into the shared timezone option label.
pub fn paint_timezone_option_label(offset: &str, name: &str) -> String {
    splora_frontend_shared::timezone_option_label(offset, name)
}
