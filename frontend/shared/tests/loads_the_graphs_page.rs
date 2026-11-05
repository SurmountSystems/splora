//! Cypress it() title: loads the graphs page.
//!
//! Spec: ref/mempool/frontend/cypress/e2e/liquid/liquid.spec.ts line 52.
//! The it() visits the dashboard, waits until `.skeleton-loader` length is
//! 0, then clicks `#btn-graphs`. The liquid master page routerLink is
//! `/graphs` through `relativeUrl`, which drops the network prefix when
//! `BASE_MODULE` is not `mempool`. This does not read an indexer body.

#[test]
fn loads_the_graphs_page() {
    let loaded = splora_frontend_shared::loads_the_graphs_page();
    assert_eq!(
        loaded.button_id, "btn-graphs",
        "cy.get('#btn-graphs') clicks that control"
    );
    assert_eq!(
        loaded.route, "/graphs",
        "liquid master page routerLink is /graphs after relativeUrl"
    );
    assert_eq!(
        loaded.skeleton_loader_count, 0,
        "Cypress.$('.skeleton-loader').length === 0 before the click"
    );
}
