//! Painted api screen after `#btn-docs`. The shared view decides whether
//! the api screen is active and which empty-path redirect is the destination.
//! This module does not choose either. A passing paint is not a live docs page.

/// Paint for `loads the api screen`.
///
/// `clicked_control` and `docs_routes` are the inputs
/// `after_docs_button_loads_the_api_screen` already takes. The painted
/// destination is that function's redirect target. This is not a docs page body.
pub fn paint_loads_the_api_screen(clicked_control: &str, docs_routes: &[(&str, &str)]) -> String {
    let view = splora_frontend_shared::after_docs_button_loads_the_api_screen(
        clicked_control,
        docs_routes,
    );
    format!(
        "control={clicked_control}\napi_screen_active={}\ndestination={}\n",
        view.api_screen_active, view.destination
    )
}
