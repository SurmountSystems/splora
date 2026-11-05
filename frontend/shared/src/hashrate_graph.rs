//! Static host attribute on the hashrate graph component.
//!
//! Cypress selects `[data-cy="hashrate-graph"]`. The attribute sits
//! on the component tag in the mining dashboard. That tag has no
//! static text. This is not a hashrate, a difficulty percent, or a
//! block count.

/// Static host attribute `hashrate-graph` from the mining dashboard.
pub fn shows_the_hashrate_graph() -> &'static str {
    "hashrate-graph"
}
