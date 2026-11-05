//! Cypress it() title: check buttons - mobile.
//!
//! Spec: ref/mempool/frontend/cypress/e2e/mainnet/mainnet.spec.ts
//! lines 455-462. Viewport iphone-6. After the skeleton wait, these are
//! visible: `.small-buttons > :nth-child(2)`, `#dropdownFees`, and
//! `.btn-group`. The body does not read an indexer, a chart series, or a
//! websocket. Endpoint cell is `none`.

#[test]
fn check_buttons_mobile() {
    let result = splora_frontend_shared::check_buttons_mobile("iphone-6");
    assert!(result.small_buttons_second_child_visible);
    assert!(result.dropdown_fees_visible);
    assert!(result.btn_group_visible);
}
