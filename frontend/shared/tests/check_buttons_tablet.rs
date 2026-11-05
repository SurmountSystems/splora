//! Cypress it() title: check buttons - tablet.
//!
//! Spec: ref/mempool/frontend/cypress/e2e/mainnet/mainnet.spec.ts
//! lines 464-471. Viewport ipad-2. After the skeleton wait, these are
//! visible: `.small-buttons > :nth-child(2)`, `#dropdownFees`, and
//! `.btn-group`. The body does not read an indexer. Endpoint cell is `none`.

#[test]
fn check_buttons_tablet() {
    let result = splora_frontend_shared::check_buttons_tablet("ipad-2");
    assert!(result.small_buttons_second_child_visible);
    assert!(result.dropdown_fees_visible);
    assert!(result.btn_group_visible);
}
