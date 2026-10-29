//! Painted USD text for the sample amount. The digits are caller-supplied.
//! This is not a live dashboard fiat cell.

use splora_frontend_shared::{FiatCode, displays_usd_values_with_correct_currency_symbol};

/// Renders the shared USD formatting of the sample amount `1234.50`.
pub fn paint_displays_usd_values_with_correct_currency_symbol() -> String {
    displays_usd_values_with_correct_currency_symbol("1234.50", FiatCode::Usd)
}
