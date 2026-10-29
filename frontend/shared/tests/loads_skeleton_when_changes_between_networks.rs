//! Cypress it() title: loads skeleton when changes between networks.
//!
//! Spec: ref/mempool/frontend/cypress/e2e/mainnet/mainnet.spec.ts
//! Matrix row 104. Endpoint cell: none until the next GET.
//! The it() calls cy.changeNetwork('signet') and cy.changeNetwork('mainnet').
//! changeNetwork then waits until Cypress.$('.skeleton-loader').length === 0.
//! This calls a shared function that is not defined. It does not read an indexer body.

#[test]
fn loads_skeleton_when_changes_between_networks() {
    let loader_count: u32 = splora_frontend_shared::loads_skeleton_when_changes_between_networks();
    assert_eq!(
        loader_count, 0,
        "Cypress.$('.skeleton-loader').length === 0 after changeNetwork('signet') and changeNetwork('mainnet')"
    );
}
