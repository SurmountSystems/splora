//! Row 104. Cypress it() title: loads skeleton when changes between networks.
//!
//! Mainnet spec
//! `ref/mempool/frontend/cypress/e2e/mainnet/mainnet.spec.ts`
//! visits `/`, waits for skeleton loaders to leave, then calls
//! `cy.changeNetwork('signet')` and `cy.changeNetwork('mainnet')`.
//! `changeNetwork` in `ref/mempool/frontend/cypress/support/commands.ts`
//! clicks `a.${network}` and then calls `cy.waitForSkeletonGone()`.
//! That helper's check is `Cypress.$('.skeleton-loader').length === 0`
//! (the assertion inside `it('loads skeleton when changes between networks')`).
//! The network names are caller-supplied strings. This test does not click
//! a dropdown, and it does not hold a block body, a transaction body, or a
//! fiat amount.
//! This check does not fetch a price and it does not add a price, mining,
//! or Lightning route.
//! `paint_loads_skeleton_when_changes_between_networks` is not defined on
//! `splora_native`.

const CYPRESS_IT: &str = "loads skeleton when changes between networks";

/// Caller-supplied network `changeNetwork` selects first.
const SIGNET: &str = "signet";

/// Caller-supplied network `changeNetwork` selects second.
const MAINNET: &str = "mainnet";

/// Count of `.skeleton-loader` markers in the paint string.
/// This is the Cypress length check. It does not invent a zero.
fn skeleton_loader_len(painted: &str) -> usize {
    painted.matches("skeleton-loader").count()
}

#[test]
fn row104_loads_skeleton_when_changes_between_networks() {
    let painted: String =
        splora_native::paint_loads_skeleton_when_changes_between_networks(SIGNET, MAINNET);
    let skeleton_len = skeleton_loader_len(&painted);
    assert_eq!(
        skeleton_len, 0,
        "{CYPRESS_IT}: Cypress.$('.skeleton-loader').length === 0, got {skeleton_len} in {painted:?}"
    );
}
