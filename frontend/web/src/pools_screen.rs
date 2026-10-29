//! Markup for the pools screen. Whether the screen is active is decided by
//! `pools_screen_after_pools_control` in the shared crate.

use splora_frontend_shared::pools_screen_after_pools_control;

/// One `#btn-pools` control, plus the pools screen name, only when the shared
/// view says that screen is active. An inactive view stays `not-served`.
pub fn pools_screen_markup() -> String {
    let view = pools_screen_after_pools_control();
    if !view.pools_screen_active {
        return "not-served".to_string();
    }
    r#"<button id="btn-pools" type="button"></button><section>pools screen</section>"#.to_string()
}
