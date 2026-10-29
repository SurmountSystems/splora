//! Painted USD decimal text. The amount is caller-supplied.
//! This is not a live indexer fiat cell.

use splora_frontend_shared::{FiatCode, displays_usd_values_with_proper_decimal_format};

/// Renders the shared USD decimal format of a caller-supplied amount.
pub fn paint_displays_usd_values_with_proper_decimal_format(amount: &str) -> String {
    displays_usd_values_with_proper_decimal_format(amount, FiatCode::Usd)
}
