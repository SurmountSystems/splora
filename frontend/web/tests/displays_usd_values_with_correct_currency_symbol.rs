//! Cypress `it('displays USD values with correct currency symbol')`.
//! File: `ref/mempool/frontend/cypress/e2e/mainnet/fiat-currency-formatting.spec.ts`.
//!
//! After Fiat mode and USD are selected, the second `.table-cell-fiat` cell
//! is read and trimmed. The visible Cypress assertion is
//! `expect(trimmedText).to.include('$');`.
//! The painted text must show `$` before the digits of the sample amount
//! `1234.50` for USD.
//! `paint_displays_usd_values_with_correct_currency_symbol` is not in this crate.
//! The call is the missing paint.

#[test]
fn displays_usd_values_with_correct_currency_symbol() {
    let markup = splora_web::paint_displays_usd_values_with_correct_currency_symbol();
    assert!(
        markup.contains("$1234.50"),
        "painted USD text must show '$' before the digits of 1234.50, got {markup}"
    );
}
