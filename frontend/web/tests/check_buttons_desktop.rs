//! Cypress it() title: check buttons - desktop.
//!
//! Spec: ref/mempool/frontend/cypress/e2e/mainnet/mainnet.spec.ts
//! lines 473-480. Viewport `macbook-16`. The second child of `.small-buttons`,
//! `#dropdownFees`, and `.btn-group` are visible. Endpoint none.
//! No indexer body. The viewport string is passed through.

#[test]
fn check_buttons_desktop() {
    let painted = splora_web::paint_check_buttons_desktop("macbook-16");
    let shared = splora_frontend_shared::check_buttons_desktop("macbook-16");
    assert_eq!(painted, shared);
    assert!(painted.small_buttons_second_child_visible);
    assert!(painted.dropdown_fees_visible);
    assert!(painted.btn_group_visible);
}
