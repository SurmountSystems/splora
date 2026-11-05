//! After-click api screen. The destination and the active flag come from
//! `after_docs_button_loads_the_api_screen` in the shared crate.
//!
//! This paint forwards the clicked control and the docs route table.
//! It is not a live docs page.

use splora_frontend_shared::{ApiScreen, after_docs_button_loads_the_api_screen};

/// Destination and api-screen flag for one clicked control.
///
/// Both fields are the shared view. This paint does not match routes.
pub fn paint_loads_the_api_screen<'a>(
    clicked_control: &str,
    docs_routes: &'a [(&'a str, &'a str)],
) -> ApiScreen<'a> {
    after_docs_button_loads_the_api_screen(clicked_control, docs_routes)
}
