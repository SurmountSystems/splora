//! Cypress it() title: displays USD values with correct currency symbol.
//!
//! Mainnet spec
//! `ref/mempool/frontend/cypress/e2e/mainnet/fiat-currency-formatting.spec.ts`
//! reads `.table-cell-fiat`, trims the text, and expects that text to
//! include `$` (line 37: `expect(trimmedText).to.include('$')`).
//! The sample amount is `1234.50` USD. Once the paint exists, the painted
//! text must show `$` before those digits (`$1234.50` or `$1,234.50`).
//! This check does not fetch a price and it does not add a price route.
//! `paint_displays_usd_values_with_correct_currency_symbol` is not defined
//! on `splora_native`.

const CYPRESS_IT: &str = "displays USD values with correct currency symbol";

/// Sample USD amount. `$` must be painted before these digits.
const SAMPLE_USD_AMOUNT: f64 = 1234.50;

#[test]
fn displays_usd_values_with_correct_currency_symbol() {
    let painted: String =
        splora_native::paint_displays_usd_values_with_correct_currency_symbol(SAMPLE_USD_AMOUNT);
    let trimmed = painted.trim();
    let dollar_before_digits = trimmed.contains("$1234.50") || trimmed.contains("$1,234.50");
    assert!(
        dollar_before_digits,
        "{CYPRESS_IT}: painted text must show '$' before the digits of 1234.50, got {trimmed}"
    );
}
