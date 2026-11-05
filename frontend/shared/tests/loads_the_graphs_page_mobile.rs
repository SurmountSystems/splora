//! Cypress it() title: loads the graphs page - mobile.
//!
//! Spec: ref/mempool/frontend/cypress/e2e/liquidtestnet/liquidtestnet.spec.ts
//! lines 60-65. Viewport iphone-6. The control `.tv-only` does not exist.
//! Local UI. cy.waitForSkeletonGone() polls the DOM and does not read a
//! response body. No indexer body.

#[test]
fn loads_the_graphs_page_mobile() {
    let result = splora_frontend_shared::loads_the_graphs_page_mobile("iphone-6");
    assert!(!result.tv_only_exists);
}
