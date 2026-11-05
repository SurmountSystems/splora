//! Calculator `.symbol` text after USD is selected.
//! This is not a live price feed.

/// Forwards the shared USD currency symbol. Not a hard-coded price.
pub fn paint_displays_usd_price_with_dollar_symbol() -> String {
    splora_frontend_shared::displays_usd_price_with_dollar_symbol()
}
