//! Cypress it() title: copy buttons exist and are visible.
//!
//! Spec: `ref/mempool/frontend/cypress/e2e/mainnet/calculator.spec.ts`
//! lines 184-189. Three `app-clipboard` elements, and each clipboard
//! button is visible. Local UI. Endpoint none. No price.
//! The web test at `frontend/web/tests/copy_buttons_exist_and_are_visible.rs`
//! calls `splora_frontend_shared::copy_buttons_exist_and_are_visible()`
//! with no arguments. This test calls that same function.
//! `paint_copy_buttons_exist_and_are_visible` is not defined on
//! `splora_native`. This test does not add it. This test does not open
//! a window.

#[test]
fn copy_buttons_exist_and_are_visible() {
    let painted = splora_native::paint_copy_buttons_exist_and_are_visible();
    let shared = splora_frontend_shared::copy_buttons_exist_and_are_visible();
    assert_eq!(painted.len(), 3, "three app-clipboard buttons");
    assert!(
        painted.iter().all(|button| button.visible),
        "each clipboard button is visible"
    );
    assert_eq!(
        painted, shared,
        "painted clipboard buttons equal the shared clipboard buttons"
    );
}
