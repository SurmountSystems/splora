//! Footer timezone option labels. The words come from the shared formatter.

/// Forwards the offset and the name to the shared timezone option label.
pub fn paint_timezone_option_label(offset: &str, name: &str) -> String {
    splora_frontend_shared::timezone_option_label(offset, name)
}
