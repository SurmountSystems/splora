//! Cypress it() title: calculator is usable on mobile.
//!
//! Spec: ref/mempool/frontend/cypress/e2e/mainnet/calculator.spec.ts
//! lines 200-205. Viewport iphone-6. The bitcoin, fiat, and satoshis
//! inputs are visible. Local UI. Endpoint none.
//! `paint_calculator_is_usable_on_mobile` is not defined on `splora_web`.
//! This test does not add it.

#[test]
fn calculator_is_usable_on_mobile() {
    let painted = splora_web::paint_calculator_is_usable_on_mobile("iphone-6");
    let shared = splora_frontend_shared::calculator_is_usable_on_mobile("iphone-6");
    assert!(painted.bitcoin_visible);
    assert!(painted.fiat_visible);
    assert!(painted.satoshis_visible);
    assert_eq!(painted, shared);
}
