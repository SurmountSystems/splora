//! Painted JPY amount with no fractional digits.
//! The digits are caller-supplied. This is not a live price feed.

use splora_frontend_shared::{FiatCode, format_fiat};

/// Renders the shared JPY format of a caller-supplied amount.
pub fn paint_displays_jpy_values_without_decimal_places(amount: &str) -> String {
    format_fiat(FiatCode::Jpy, amount)
}
