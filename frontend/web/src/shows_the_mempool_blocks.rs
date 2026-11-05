//! Painted mempool-block strings for the fields the caller passes.
//!
//! The five strings are `splora_frontend_shared::shows_the_mempool_blocks`.
//! This file does not hard-code a block's display text and does not fetch a URL.

/// Returns the shared mempool-block strings for the fields the caller passes.
pub fn paint_shows_the_mempool_blocks(
    median_fee: u64,
    fee_range_start: u64,
    fee_range_end: u64,
    total_fee_sats: u64,
    transaction_count: u64,
    minutes: u64,
) -> [String; 5] {
    splora_frontend_shared::shows_the_mempool_blocks(
        median_fee,
        fee_range_start,
        fee_range_end,
        total_fee_sats,
        transaction_count,
        minutes,
    )
}
