//! Cypress `it('displays USD price with dollar symbol')`.
//! File: `ref/mempool/frontend/cypress/e2e/mainnet/fiat-currency-formatting.spec.ts`
//! lines 213-218.
//!
//! After USD is selected, the calculator `.symbol` text is read and trimmed.
//! The visible Cypress assertion is `expect(text.trim()).to.include('$')`.
//! This does not assert a price and does not assert amount digits.
//! `paint_displays_usd_price_with_dollar_symbol` is not in this crate.
//! The call is the missing paint. The paint forwards the shared return.

#[test]
fn displays_usd_price_with_dollar_symbol() {
    let text = splora_web::paint_displays_usd_price_with_dollar_symbol();
    let trimmed = text.trim();
    assert!(
        trimmed.contains('$'),
        "calculator symbol includes `$` after trim, got {trimmed:?}"
    );
}
