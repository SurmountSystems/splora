//! Static host attribute on the latest-blocks component.
//!
//! Cypress selects `[data-cy="latest-blocks"]`. The attribute sits
//! on the component tag. This is not a height, a hash, a fee, a pool
//! name, a transaction count, or a timestamp.

/// Static host attribute `latest-blocks` from the mining dashboard.
pub fn shows_the_latest_blocks() -> &'static str {
    "latest-blocks"
}
