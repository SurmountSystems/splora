//! Painted lines after the pools control. The shared view decides whether
//! the pools screen is active. This module does not choose that.

/// Lines the native test checks after `#btn-pools`, the pools screen, and a 1000 millisecond wait.
fn active_pools_lines() -> Vec<String> {
    vec![
        "btn-pools".to_string(),
        "screen: pools".to_string(),
        "wait_ms=1000".to_string(),
    ]
}

/// Painted lines for `pools_screen_after_pools_control`. Empty when that view is not active.
pub fn pools_screen_lines() -> Vec<String> {
    let view = splora_frontend_shared::pools_screen_after_pools_control();
    if view.pools_screen_active {
        active_pools_lines()
    } else {
        Vec::new()
    }
}
