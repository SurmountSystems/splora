//! Loaded dashboard after the skeleton wait.
//!
//! The skeleton flag is `dashboard_page_initial`. The section is
//! `dashboard_loaded_markup`. A hidden skeleton adds no loader element.

/// Paints the loaded dashboard. A hidden skeleton adds no `.skeleton-loader`.
pub fn paint_loads_the_dashboard() -> String {
    let page = splora_frontend_shared::dashboard_page_initial();
    let mut markup = crate::dashboard_loaded_markup("");
    if page.skeleton_visible {
        markup.push_str("<div class=\"skeleton-loader\"></div>");
    }
    markup
}
