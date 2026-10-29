//! Cypress `it('loads skeleton when changes between networks')`.
//! File: `ref/mempool/frontend/cypress/e2e/mainnet/mainnet.spec.ts`.
//!
//! Matrix row 104. The it() visits `/`, waits until the skeleton is gone,
//! then calls `cy.changeNetwork('signet')` and `cy.changeNetwork('mainnet')`.
//! `changeNetwork` clicks `a.${network}` and then calls
//! `cy.waitForSkeletonGone()`. That helper's check is
//! `Cypress.$('.skeleton-loader').length === 0`.
//! The network names `signet` and `mainnet` are caller-supplied. This does
//! not read an indexer body and it does not add a price route.
//! `paint_loads_skeleton_when_changes_between_networks` is not in this crate.
//! The call is the missing paint.

#[test]
fn loads_skeleton_when_changes_between_networks() {
    let markup =
        splora_web::paint_loads_skeleton_when_changes_between_networks("signet", "mainnet");
    let loader_count = markup.matches("skeleton-loader").count();
    assert_eq!(
        loader_count, 0,
        "loads skeleton when changes between networks: Cypress.$('.skeleton-loader').length === 0 after changeNetwork('signet') and changeNetwork('mainnet'), got {loader_count} in {markup:?}"
    );
}
