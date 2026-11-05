//! Language choice cookie. The text comes from the shared cookie.

/// Forwards the language to the shared language cookie.
pub fn paint_language_cookie(language: &str) -> String {
    splora_frontend_shared::language_cookie(language)
}
