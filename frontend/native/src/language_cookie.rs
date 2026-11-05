//! Native paint of the language choice cookie.
//! This module does not open a window.

/// Forwards the language into the shared language cookie.
pub fn paint_language_cookie(language: &str) -> String {
    splora_frontend_shared::language_cookie(language)
}
