//! Cypress it() title: calculator is usable on mobile.
//!
//! Spec: ref/mempool/frontend/cypress/e2e/mainnet/calculator.spec.ts
//! lines 200-205. At viewport iphone-6 the bitcoin, fiat, and satoshis
//! inputs are visible. Local UI. No typed text. No price, indexer, or websocket.

#[test]
fn calculator_is_usable_on_mobile() {
    let result = splora_frontend_shared::calculator_is_usable_on_mobile("iphone-6");
    assert!(result.bitcoin_visible, "bitcoin input visible at iphone-6");
    assert!(result.fiat_visible, "fiat input visible at iphone-6");
    assert!(
        result.satoshis_visible,
        "satoshis input visible at iphone-6"
    );
}
