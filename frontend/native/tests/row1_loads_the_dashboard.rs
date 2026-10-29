//! Row 1. Cypress it() title: loads the dashboard.
//!
//! Testnet4 spec
//! `ref/mempool/frontend/cypress/e2e/testnet4/testnet4.spec.ts`
//! the it() calls `cy.waitForSkeletonGone();` (line 17).
//! The command body in `ref/mempool/frontend/cypress/support/commands.ts`
//! line 59 is `return Cypress.$('.skeleton-loader').length === 0;`.
//! This test does not hold a block body, a transaction body, or a fiat amount.
//! This check does not fetch a price and it does not add a price, mining,
//! or Lightning route.
//! `paint_loads_the_dashboard` is not defined on `splora_native`.

const CYPRESS_IT: &str = "loads the dashboard";

/// Count of `.skeleton-loader` markers in the paint string.
/// This is the Cypress length check. It does not invent a zero.
fn skeleton_loader_len(painted: &str) -> usize {
    painted.matches("skeleton-loader").count()
}

#[test]
fn row1_loads_the_dashboard() {
    let painted: String = splora_native::paint_loads_the_dashboard();
    let skeleton_len = skeleton_loader_len(&painted);
    assert_eq!(
        skeleton_len, 0,
        "{CYPRESS_IT}: Cypress.$('.skeleton-loader').length === 0, got {skeleton_len} in {painted:?}"
    );
}
