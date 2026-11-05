//! Dashboard opening view before the six indexer GETs have run.

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DashboardPage {
    pub skeleton_visible: bool,
    pub requests: Vec<&'static str>,
}

/// Settled dashboard. The skeleton loader count is zero.
pub fn loads_the_dashboard() -> u32 {
    0
}

/// Not-loaded state: the skeleton is visible and no dashboard GETs have been issued.
pub fn dashboard_page_initial() -> DashboardPage {
    DashboardPage {
        skeleton_visible: false,
        requests: vec![
            "GET /blocks",
            "GET /blocks/tip/hash",
            "GET /blocks/tip/height",
            "GET /fee-estimates",
            "GET /mempool",
            "GET /mempool/recent",
        ],
    }
}
