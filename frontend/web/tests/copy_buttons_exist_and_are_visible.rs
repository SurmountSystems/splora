//! Cypress it() title: copy buttons exist and are visible.
//!
//! Spec: ref/mempool/frontend/cypress/e2e/mainnet/calculator.spec.ts
//! lines 184-189. Three app-clipboard elements, and each clipboard button
//! is visible. Local UI. Endpoint none. No price.
//! `paint_copy_buttons_exist_and_are_visible` is not defined on
//! `splora_web`. This test does not add it.

#[test]
fn copy_buttons_exist_and_are_visible() {
    let painted = splora_web::paint_copy_buttons_exist_and_are_visible();
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
