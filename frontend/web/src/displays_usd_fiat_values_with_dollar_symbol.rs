//! Transaction-details fiat, then transactions-list fiat, in USD.
//! This is not a live price feed.

/// Forwards the shared USD fiat cells.
pub fn paint_displays_usd_fiat_values_with_dollar_symbol() -> [String; 2] {
    splora_frontend_shared::displays_usd_fiat_values_with_dollar_symbol()
}
