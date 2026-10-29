//! Caller-supplied USD amount with two cent digits. Not a live indexer fiat cell.

use splora_frontend_shared::{FiatCode, displays_usd_values_with_proper_decimal_format};

/// Pass `amount` through to the shared USD decimal formatter.
pub fn paint_displays_usd_values_with_proper_decimal_format(amount: &str) -> String {
    displays_usd_values_with_proper_decimal_format(amount, FiatCode::Usd)
}
