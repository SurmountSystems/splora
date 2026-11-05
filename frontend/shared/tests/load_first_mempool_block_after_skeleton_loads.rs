//! Cypress it() title: load first mempool block after skeleton loads.
//!
//! Spec: ref/mempool/frontend/cypress/e2e/liquid/liquid.spec.ts
//! The it() visits the dashboard, waits until the skeleton is gone, clicks
//! `#mempool-block-0 > .blockLink`, and waits until the skeleton is gone again.
//! The dashboard does not pass `getHref`, so the click follows
//! `/mempool-block/0`. This calls a shared function that is not defined yet.
//! It does not read an indexer body.

#[test]
fn load_first_mempool_block_after_skeleton_loads() {
    let loaded = splora_frontend_shared::load_first_mempool_block_after_skeleton_loads();
    assert_eq!(
        loaded.link_id, "mempool-block-0",
        "cy.get('#mempool-block-0 > .blockLink') clicks that row"
    );
    assert_eq!(
        loaded.block_link_class, "blockLink",
        "the clicked child is .blockLink"
    );
    assert_eq!(
        loaded.route, "/mempool-block/0",
        "dashboard app-mempool-blocks uses the default getHref(0)"
    );
    assert_eq!(
        loaded.skeleton_loader_count, 0,
        "Cypress.$('.skeleton-loader').length === 0 after the click"
    );
}
