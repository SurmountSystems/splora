//! Cypress it() title: displays USD values with proper decimal format.
//!
//! Spec: ref/mempool/frontend/cypress/e2e/mainnet/fiat-currency-formatting.spec.ts
//! Trimmed fiat cell text matches `/^\$[\d,]+(\.\d{2})?$/`.
//! The caller-supplied amount is the string `1234.5` (one fractional digit).
//! This does not read an indexer body and does not use a price route.

use splora_frontend_shared::FiatCode;

#[test]
fn displays_usd_values_with_proper_decimal_format() {
    let amount = "1234.5";
    let text: String = splora_frontend_shared::displays_usd_values_with_proper_decimal_format(
        amount,
        FiatCode::Usd,
    );
    let trimmed = text.trim();
    assert!(
        trimmed == "$1234.50" || trimmed == "$1,234.50",
        "USD decimal format of caller-supplied {amount} is $1234.50 or $1,234.50, got {trimmed:?}"
    );
}
