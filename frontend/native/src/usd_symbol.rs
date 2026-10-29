//! Caller-supplied USD amount painted with `$`. Not a live dashboard fiat cell.

use splora_frontend_shared::{FiatCode, displays_usd_values_with_correct_currency_symbol};

/// Format `amount` to two decimal places and return the shared USD text.
pub fn paint_displays_usd_values_with_correct_currency_symbol(amount: f64) -> String {
    let text = format!("{amount:.2}");
    displays_usd_values_with_correct_currency_symbol(&text, FiatCode::Usd)
}
