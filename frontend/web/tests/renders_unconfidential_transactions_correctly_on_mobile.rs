//! Liquid testnet transaction page has no horizontal scrollbar on a phone.
//!
//! Cypress title `renders unconfidential transactions correctly on mobile`
//! at ref/mempool/frontend/cypress/e2e/liquidtestnet/liquidtestnet.spec.ts line 67.
//! Viewport `iphone-xr`. Expected scrollbar width 0.

#[test]
fn renders_unconfidential_transactions_correctly_on_mobile() {
    let painted = splora_web::paint_loads_the_dashboard_with_no_scrollbars_on_mobile("iphone-xr");
    let shared =
        splora_frontend_shared::loads_the_dashboard_with_no_scrollbars_on_mobile("iphone-xr");
    assert_eq!(painted.scroll_bar_width, 0);
    assert_eq!(painted, shared);
}
