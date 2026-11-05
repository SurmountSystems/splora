//! Liquid testnet dashboard has no horizontal scrollbar on a phone.
//!
//! Cypress title `loads the dashboard with no scrollbars on mobile` at
//! ref/mempool/frontend/cypress/e2e/liquidtestnet/liquidtestnet.spec.ts
//! lines 31-40. The source marks it `it.skip`. The body still sets
//! viewport `iphone-xr` and expects scrollbar width 0.
//! `cy.waitForSkeletonGone` polls the skeleton element. It does not
//! read an indexer body. Endpoint none.

#[test]
fn loads_the_dashboard_with_no_scrollbars_on_mobile() {
    let result =
        splora_frontend_shared::loads_the_dashboard_with_no_scrollbars_on_mobile("iphone-xr");
    assert_eq!(result.scroll_bar_width, 0);
}
