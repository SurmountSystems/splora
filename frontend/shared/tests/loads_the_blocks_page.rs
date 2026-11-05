//! Cypress it() title: loads the blocks page.
//!
//! Spec: ref/mempool/frontend/cypress/e2e/liquid/liquid.spec.ts line 38.
//! The it() visits the dashboard, clicks `#btn-blocks`, and waits until
//! `.skeleton-loader` length is 0. The liquid master page routerLink is
//! `/blocks` through `relativeUrl`, which drops the network prefix when
//! `BASE_MODULE` is not `mempool`. This does not read an indexer body.

#[test]
fn loads_the_blocks_page() {
    let loaded = splora_frontend_shared::loads_the_blocks_page();
    assert_eq!(
        loaded.button_id, "btn-blocks",
        "cy.get('#btn-blocks') clicks that control"
    );
    assert_eq!(
        loaded.route, "/blocks",
        "liquid master page routerLink is /blocks after relativeUrl"
    );
    assert_eq!(
        loaded.skeleton_loader_count, 0,
        "Cypress.$('.skeleton-loader').length === 0 after the click"
    );
}
