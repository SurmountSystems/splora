//! Static host attribute on the pools pie chart component.
//!
//! Cypress selects `[data-cy="pool-distribution"]`. The attribute sits
//! on the component tag. This is not a pool name, a share, a hashrate,
//! or a block count.

/// Static host attribute `pool-distribution` from the mining dashboard.
pub fn shows_the_pools_pie_chart() -> &'static str {
    "pool-distribution"
}
