//! Static host attribute on the difficulty-adjustment component.
//!
//! Cypress selects `[data-cy="difficulty-adjustment"]`. The attribute sits
//! on the component tag. This is not a difficulty percent, a block count,
//! a timestamp, or a price.

/// Static host attribute `difficulty-adjustment` from the mining dashboard.
pub fn shows_the_difficulty_adjustment_stats() -> &'static str {
    "difficulty-adjustment"
}
