//! Cypress it() title: shows calculator form after price feed loads.
//!
//! Spec: ref/mempool/frontend/cypress/e2e/mainnet/calculator.spec.ts
//! lines 209-212. The loaded calculator form does not contain
//! "Waiting for price feed..." and the bitcoin input
//! (`formControlName` `bitcoin`) is visible. No price number.
//! No MOCK_BTC_PRICE_USD. `/v1/prices` stays not-served.

#[test]
fn shows_calculator_form_after_price_feed_loads() {
    let form = splora_frontend_shared::shows_calculator_form_after_price_feed_loads();
    assert!(
        !form.text.contains("Waiting for price feed..."),
        "cy.contains('Waiting for price feed...').should('not.exist')"
    );
    assert!(
        form.bitcoin_visible,
        "input[formControlName=\"bitcoin\"] is visible"
    );
}
