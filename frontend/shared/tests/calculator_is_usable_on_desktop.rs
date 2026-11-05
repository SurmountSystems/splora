//! Cypress it() title: calculator is usable on desktop.
//!
//! Spec: ref/mempool/frontend/cypress/e2e/mainnet/calculator.spec.ts
//! lines 193-198. At viewport macbook-16 the bitcoin input is visible,
//! and after typing 1 the field value includes the character 1.
//! Local UI. No price, indexer, or websocket.

#[test]
fn calculator_is_usable_on_desktop() {
    let result = splora_frontend_shared::calculator_is_usable_on_desktop("macbook-16", "1");
    assert!(
        result.bitcoin_visible,
        "bitcoin input visible at macbook-16"
    );
    assert!(
        result.value.contains('1') || result.value.contains("1"),
        "typed value includes 1"
    );
}
