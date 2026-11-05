//! Painted mined-block strings for the fields the caller passes.
//!
//! The seven strings are `splora_frontend_shared::shows_the_mined_blocks`.
//! This file does not hard-code a block's display text and does not fetch a URL.

/// Returns the shared mined-block strings for the fields the caller passes.
pub fn paint_shows_the_mined_blocks(
    height: u64,
    median_fee: u64,
    fee_range: &[u64],
    total_fee_sats: u64,
    transaction_count: u64,
    block_timestamp: u64,
    clock_unix: u64,
    pool_name: &str,
) -> [String; 7] {
    splora_frontend_shared::shows_the_mined_blocks(
        height,
        median_fee,
        fee_range,
        total_fee_sats,
        transaction_count,
        block_timestamp,
        clock_unix,
        pool_name,
    )
}
