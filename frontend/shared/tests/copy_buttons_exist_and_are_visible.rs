//! Cypress it() title: copy buttons exist and are visible.
//!
//! Spec: ref/mempool/frontend/cypress/e2e/mainnet/calculator.spec.ts
//! lines 184-189. Three calculator clipboard controls exist, and each
//! clipboard button is visible. Local UI. No price, indexer, or websocket.

#[test]
fn copy_buttons_exist_and_are_visible() {
    let buttons = splora_frontend_shared::copy_buttons_exist_and_are_visible();
    assert_eq!(buttons.len(), 3);
    assert!(
        buttons.iter().all(|button| button.visible),
        "each calculator clipboard button is visible"
    );
}
