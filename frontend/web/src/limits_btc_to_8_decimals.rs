//! Painted BTC amount text. The typed field is caller-supplied.
//! This is not a live indexer price.

use splora_frontend_shared::truncate_btc_decimals;

/// Returns the shared truncator's text for a BTC amount field.
pub fn paint_limits_btc_to_8_decimals(input: &str) -> String {
    truncate_btc_decimals(input)
}
