//! Cypress it() title: loads the dashboard.
//!
//! Spec: ref/mempool/frontend/cypress/e2e/testnet4/testnet4.spec.ts
//! The it() visits `/testnet4` and calls `cy.waitForSkeletonGone()`.
//! That command waits until `Cypress.$('.skeleton-loader').length === 0`.
//! This calls a shared function that is not defined. It does not read an indexer body.

#[test]
fn loads_the_dashboard() {
    let loader_count: u32 = splora_frontend_shared::loads_the_dashboard();
    assert_eq!(
        loader_count, 0,
        "Cypress.$('.skeleton-loader').length === 0"
    );
}
