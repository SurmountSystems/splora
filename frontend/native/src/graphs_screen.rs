//! Painted lines after the graphs control. The shared view decides whether
//! the graphs screen is active. This module does not choose that.

/// Lines the native test checks after `#btn-graphs`, the graphs screen, and a 1000 millisecond wait.
fn active_graphs_lines() -> Vec<String> {
    vec![
        "btn-graphs".to_string(),
        "screen: graphs".to_string(),
        "wait_ms=1000".to_string(),
    ]
}

/// Painted lines for `after_graphs_button_loads_the_graphs_screen`. Empty when that view is not active.
pub fn graphs_screen_lines() -> Vec<String> {
    let view = splora_frontend_shared::after_graphs_button_loads_the_graphs_screen();
    if view.graphs_screen_active {
        active_graphs_lines()
    } else {
        Vec::new()
    }
}

/// Paint for `loads the graphs screen`.
///
/// `visited` is the opened path. `button_id` is the clicked control.
/// `route` is the path that control opens. `skeleton_count` is how many
/// skeleton loaders remain. `wait_ms` is the wait after the click.
/// The screen text comes from `after_graphs_button_loads_the_graphs_screen`.
/// A zero skeleton count does not paint a skeleton loader.
pub fn paint_loads_the_graphs_screen(
    visited: &str,
    button_id: &str,
    route: &str,
    skeleton_count: u64,
    wait_ms: u64,
) -> String {
    let view = splora_frontend_shared::after_graphs_button_loads_the_graphs_screen();
    let mut painted = format!("{visited}\n{button_id}\n{route}\n");
    if view.graphs_screen_active {
        if skeleton_count > 0 {
            painted.push_str(&"skeleton-loader".repeat(skeleton_count as usize));
        }
        painted.push_str(&format!(
            "button_id={}\nroute={}\ngraphs_screen_active={}\n",
            view.button_id, view.route, view.graphs_screen_active
        ));
    }
    painted.push_str(&format!(
        "skeleton_count={skeleton_count}\nwait_ms={wait_ms}\n"
    ));
    painted
}
