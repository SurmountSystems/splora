//! Cypress it() title: "loads the graphs page - mobile".
//!
//! Spec: ref/mempool/frontend/cypress/e2e/liquid/liquid.spec.ts
//! lines 60-68. Viewport iphone-6. `.tv-only` does not exist.
//! Local UI. Endpoint none. Do not assert an indexer body.

#[test]
fn liquid_graphs_page_mobile() {
    let painted = splora_web::paint_loads_the_graphs_page_mobile("iphone-6");
    let shared = splora_frontend_shared::loads_the_graphs_page_mobile("iphone-6");
    assert!(!painted.tv_only_exists);
    assert_eq!(painted, shared);
}
