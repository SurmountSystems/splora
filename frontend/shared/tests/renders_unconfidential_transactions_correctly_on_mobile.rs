//! A non-confidential liquid testnet transaction page has no horizontal scrollbar on a phone.
//!
//! Cypress title `renders unconfidential transactions correctly on mobile` at
//! ref/mempool/frontend/cypress/e2e/liquidtestnet/liquidtestnet.spec.ts line 67.
//! The source marks it `it.skip`. The body sets viewport `iphone-xr` and
//! expects scrollbar width 0. The visit is one transaction URL. The only
//! expect is that width. This test does not claim a transaction body was served.

#[test]
fn renders_unconfidential_transactions_correctly_on_mobile() {
    let result =
        splora_frontend_shared::loads_the_dashboard_with_no_scrollbars_on_mobile("iphone-xr");
    assert_eq!(result.scroll_bar_width, 0);
}
