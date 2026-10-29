//! Painted api screen after `#btn-docs`. The shared view decides whether
//! the api screen is active. This module does not choose that. A passing
//! paint of this flag is not a live docs page.

/// Paint for `loads the api screen`.
///
/// Signet visits `/signet`. `#btn-docs` routes to `/docs`. The active flag
/// is `after_docs_button_loads_the_api_screen`. This is not a docs page body.
pub fn paint_loads_the_api_screen() -> String {
    let view = splora_frontend_shared::after_docs_button_loads_the_api_screen();
    if !view.api_screen_active {
        return String::new();
    }
    format!(
        "btn-docs\n/signet\n/docs\napi_screen_active={}",
        view.api_screen_active
    )
}
