//! Web paint of the loaded calculator form.
//! The view is `splora_frontend_shared::shows_calculator_form_after_price_feed_loads`.

/// Loaded calculator form once the price-feed wait is gone.
pub struct CalculatorFormAfterPriceFeedPaint {
    /// Heading and field controls from the shared form.
    pub text: String,
    /// Whether the bitcoin input is visible.
    pub bitcoin_visible: bool,
}

/// Forwards the shared loaded calculator form.
pub fn paint_shows_calculator_form_after_price_feed_loads() -> CalculatorFormAfterPriceFeedPaint {
    let loaded = splora_frontend_shared::shows_calculator_form_after_price_feed_loads();
    CalculatorFormAfterPriceFeedPaint {
        text: loaded.text,
        bitcoin_visible: loaded.bitcoin_visible,
    }
}
