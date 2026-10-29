//! Cypress it() title: displays USD values with correct currency symbol.
//!
//! Spec: ref/mempool/frontend/cypress/e2e/mainnet/fiat-currency-formatting.spec.ts
//! After Fiat mode and USD are selected, the dashboard fiat cell text includes `$`.
//! A sample amount and a currency code are enough. This does not read an indexer body.

use splora_frontend_shared::FiatCode;

#[test]
fn displays_usd_values_with_correct_currency_symbol() {
    let text: String = splora_frontend_shared::displays_usd_values_with_correct_currency_symbol(
        "1234.50",
        FiatCode::Usd,
    );
    let trimmed = text.trim();
    assert!(
        trimmed.contains('$'),
        "USD fiat cell text includes the dollar symbol, got {trimmed:?}"
    );
    let digits: String = trimmed.chars().filter(|c| c.is_ascii_digit()).collect();
    assert!(
        digits.contains("1234"),
        "formatted USD text keeps the sample amount digits, got {trimmed:?}"
    );
    let dollar_at = trimmed
        .find('$')
        .expect("dollar symbol present after the contains check");
    let first_digit = trimmed
        .find(|c: char| c.is_ascii_digit())
        .expect("sample amount digits present");
    assert!(
        dollar_at < first_digit,
        "the dollar symbol comes before the amount digits, got {trimmed:?}"
    );
}
