//! Cypress it() title: displays USD price with dollar symbol.
//!
//! Spec: ref/mempool/frontend/cypress/e2e/mainnet/fiat-currency-formatting.spec.ts
//! lines 213-218, inside describe('Calculator Page'). After USD is selected,
//! the trimmed `.symbol` text includes `$`. This does not assert digits, a
//! BTC price, or a price-feed sentence. It does not fetch a URL.

#[test]
fn displays_usd_price_with_dollar_symbol() {
    let text: String = splora_frontend_shared::displays_usd_price_with_dollar_symbol();
    let trimmed = text.trim();
    assert!(
        trimmed.contains('$'),
        "calculator `.symbol` text includes `$`, got {trimmed:?}"
    );
}
