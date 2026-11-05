//! Cypress it() title: loads the graphs page - mobile.
//!
//! Spec: ref/mempool/frontend/cypress/e2e/liquidtestnet/liquidtestnet.spec.ts
//! lines 60-65. Viewport iphone-6. `.tv-only` does not exist.
//! Local UI. Endpoint none. Do not assert an indexer body.
//! `paint_loads_the_graphs_page_mobile` is not defined on `splora_web`.
//! This test does not add it.

#[test]
fn loads_the_graphs_page_mobile() {
    let painted = splora_web::paint_loads_the_graphs_page_mobile("iphone-6");
    let shared = splora_frontend_shared::loads_the_graphs_page_mobile("iphone-6");
    assert!(!painted.tv_only_exists);
    assert_eq!(painted, shared);
}
