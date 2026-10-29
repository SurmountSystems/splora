//! Markup for the graphs screen. Whether the screen is active is decided by
//! `after_graphs_button_loads_the_graphs_screen` in the shared crate.

use splora_frontend_shared::after_graphs_button_loads_the_graphs_screen;

/// One `#btn-graphs` control, plus the graphs screen name, only when the shared
/// view says that screen is active. An inactive view stays `not-served`.
pub fn graphs_screen_markup() -> String {
    let view = after_graphs_button_loads_the_graphs_screen();
    if !view.graphs_screen_active {
        return "not-served".to_string();
    }
    format!(
        r#"<button id="{}" type="button" href="{}"></button><section>graphs screen</section>"#,
        view.button_id, view.route,
    )
}
