//! Static host attribute on the difficulty adjustments table.
//!
//! Cypress selects `[data-cy="difficulty-adjustments-table"]`. The attribute sits
//! on the component tag in the mining dashboard. That tag has no
//! static text. This is not a difficulty percent, a difficulty number,
//! a block count, a hashrate, a price, or a mining amount.

/// Static host attribute `difficulty-adjustments-table` from the mining dashboard.
pub fn shows_the_latest_adjustments() -> &'static str {
    "difficulty-adjustments-table"
}
