//! Calculator `.symbol` text and fiat field after JPY is selected.
//! This is not a live price feed.

/// Forwards the shared JPY calculator symbol and rounded fiat field.
pub fn paint_displays_jpy_price_without_decimals_after_switching_currency(
    bitcoin: &str,
    fiat_per_btc: &str,
) -> (String, String) {
    splora_frontend_shared::displays_jpy_price_without_decimals_after_switching_currency(
        bitcoin,
        fiat_per_btc,
    )
}
