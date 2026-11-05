//! Liquid testnet dashboard has no horizontal scrollbar on a phone.
//!
//! Cypress title `loads the dashboard with no scrollbars on mobile` at
//! ref/mempool/frontend/cypress/e2e/liquidtestnet/liquidtestnet.spec.ts
//! lines 31-40. Viewport `iphone-xr`. Expected scrollbar width 0.
//! This test does not open a window.

#[test]
fn loads_the_dashboard_with_no_scrollbars_on_mobile() {
    let painted =
        splora_native::paint_loads_the_dashboard_with_no_scrollbars_on_mobile("iphone-xr");
    let shared =
        splora_frontend_shared::loads_the_dashboard_with_no_scrollbars_on_mobile("iphone-xr");
    assert_eq!(painted.scroll_bar_width, 0);
    assert_eq!(painted, shared);
}
