//! Language choice cookie for the footer selector.
//!
//! The expiry text is the Angular literal in `language.service.ts`.
//! The language argument is the cookie value. This function does not
//! use the network.

/// Cookie written when a language is chosen.
///
/// `lang=` plus `language`, then `; expires=Thu, 18 Dec 2050 12:00:00 UTC; path=/`.
pub fn language_cookie(language: &str) -> String {
    format!("lang={language}; expires=Thu, 18 Dec 2050 12:00:00 UTC; path=/")
}
