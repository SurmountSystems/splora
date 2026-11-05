//! Cypress title: shows input labels for currency, BTC, and sats.
//!
//! Spec: ref/mempool/frontend/cypress/e2e/mainnet/calculator.spec.ts line 66.
//! The web paint must return `shows_input_labels_for_currency_btc_and_sats`.
//! The comparison is that paint call against the shared function, not a
//! copied label string.

#[test]
fn shows_input_labels_for_currency_btc_and_sats() {
    assert_eq!(
        splora_web::paint_shows_input_labels_for_currency_btc_and_sats(),
        splora_frontend_shared::shows_input_labels_for_currency_btc_and_sats()
    );
}
