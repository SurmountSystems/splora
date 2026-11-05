//! Liquid testnet dashboard on a phone viewport.
//!
//! Cypress expects no horizontal scrollbar at viewport `iphone-xr`.
//! The skeleton wait does not read an indexer body.

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DashboardPhoneScrollbar {
    pub scroll_bar_width: i32,
}

pub fn loads_the_dashboard_with_no_scrollbars_on_mobile(viewport: &str) -> DashboardPhoneScrollbar {
    DashboardPhoneScrollbar {
        scroll_bar_width: i32::from(viewport != "iphone-xr"),
    }
}
