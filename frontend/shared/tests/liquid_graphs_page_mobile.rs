//! Cypress it() title: loads the graphs page - mobile.
//!
//! Spec: ref/mempool/frontend/cypress/e2e/liquid/liquid.spec.ts
//! lines 60-68. The click on `#btn-graphs` is how Cypress reaches the graphs
//! page. Viewport name iphone-6. `.tv-only` does not exist. Local UI.
//! Endpoint none. No indexer body.

#[test]
fn liquid_graphs_page_mobile() {
    let result = splora_frontend_shared::loads_the_graphs_page_mobile("iphone-6");
    assert!(!result.tv_only_exists);
}
