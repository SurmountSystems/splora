//! Cypress it() title: check buttons - mobile.
//!
//! Spec: ref/mempool/frontend/cypress/e2e/mainnet/mainnet.spec.ts
//! lines 455-462. Viewport `iphone-6`. The second child of `.small-buttons`,
//! `#dropdownFees`, and `.btn-group` are visible. Endpoint none.
//! No indexer body. The viewport string is passed through.

#[test]
fn check_buttons_mobile() {
    let painted = splora_web::paint_check_buttons_mobile("iphone-6");
    let shared = splora_frontend_shared::check_buttons_mobile("iphone-6");
    assert_eq!(painted, shared);
    assert!(painted.small_buttons_second_child_visible);
    assert!(painted.dropdown_fees_visible);
    assert!(painted.btn_group_visible);
}
