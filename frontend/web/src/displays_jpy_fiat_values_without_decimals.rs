//! Transaction-details fiat, then transactions-list fiat, in JPY.
//! This is not a live price feed.

/// Forwards the shared JPY fiat cells.
pub fn paint_displays_jpy_fiat_values_without_decimals() -> [String; 2] {
    splora_frontend_shared::displays_jpy_fiat_values_without_decimals()
}
