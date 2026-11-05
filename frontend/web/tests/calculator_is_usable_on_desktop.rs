//! Cypress it() title: calculator is usable on desktop.
//!
//! Spec: ref/mempool/frontend/cypress/e2e/mainnet/calculator.spec.ts
//! lines 193-198. Viewport macbook-16. The bitcoin input is visible.
//! Typed text `1`. The field value includes `1`. Local UI. Endpoint none.
//! `paint_calculator_is_usable_on_desktop` is not defined on `splora_web`.
//! This test does not add it.

#[test]
fn calculator_is_usable_on_desktop() {
    let painted = splora_web::paint_calculator_is_usable_on_desktop("macbook-16", "1");
    let shared = splora_frontend_shared::calculator_is_usable_on_desktop("macbook-16", "1");
    assert!(
        painted.bitcoin_visible,
        "the bitcoin input is visible on macbook-16"
    );
    assert!(painted.value.contains("1"), "the bitcoin value includes 1");
    assert_eq!(
        painted, shared,
        "painted desktop calculator equals the shared desktop calculator"
    );
}
