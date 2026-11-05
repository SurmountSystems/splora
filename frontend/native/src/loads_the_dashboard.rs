//! Loaded dashboard after the skeleton wait.
//!
//! The skeleton flag is [`splora_frontend_shared::dashboard_page_initial`].
//! `dashboard_loaded_markup` is a web painter and is not in this crate.
//! An empty mempool body has no summary, size, or block-link text, so the
//! loaded section is those three empty slots plus the shared opening
//! requests. A hidden skeleton adds no `.skeleton-loader` element.

use splora_frontend_shared::{DashboardPage, dashboard_page_initial};

/// Paints the loaded dashboard. A hidden skeleton adds no `.skeleton-loader`.
pub fn paint_loads_the_dashboard() -> String {
    let page = dashboard_page_initial();
    let mut markup = loaded_dashboard_markup(&page);
    if page.skeleton_visible {
        markup.push_str("<div class=\"skeleton-loader\"></div>");
    }
    markup
}

/// Loaded section for the opening dashboard page.
///
/// The web painter wraps an empty mempool body as `<pre>`, then two `<div>`s.
/// This crate does not own that painter. The request lines are the shared
/// opening GETs on `page`.
fn loaded_dashboard_markup(page: &DashboardPage) -> String {
    let mut markup = String::from("<pre></pre><div></div><div></div>");
    for request in &page.requests {
        markup.push('\n');
        markup.push_str(request);
    }
    markup
}
