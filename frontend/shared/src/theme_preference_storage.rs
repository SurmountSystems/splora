//! Theme choice stored as a local preference.
//!
//! Angular `theme.service.ts` writes `theme-preference` to the theme
//! argument. This function does not use the network.

/// Storage pair written when a theme is chosen.
///
/// `theme-preference=` plus `theme`.
pub fn theme_preference_storage(theme: &str) -> String {
    format!("theme-preference={theme}")
}
