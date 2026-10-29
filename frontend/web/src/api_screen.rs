//! Markup for the api screen. Whether the screen is active is decided by
//! `after_docs_button_loads_the_api_screen` in the shared crate.
//!
//! This paint is the after-click flag only. It is not a live docs page.

use splora_frontend_shared::after_docs_button_loads_the_api_screen;

/// One `#btn-docs` control, plus the api screen name, only when the shared
/// view says that screen is active. An inactive view stays `not-served`.
pub fn paint_loads_the_api_screen() -> String {
    let view = after_docs_button_loads_the_api_screen();
    if !view.api_screen_active {
        return "not-served".to_string();
    }
    r#"<button id="btn-docs" type="button"></button><section>api screen</section>"#.to_string()
}
