//! Cypress `it('loads the dashboard')`.
//! File: `ref/mempool/frontend/cypress/e2e/testnet4/testnet4.spec.ts`, line 17.
//!
//! The it() calls `cy.waitForSkeletonGone()`. The command body in
//! `ref/mempool/frontend/cypress/support/commands.ts` line 59 is
//! `return Cypress.$('.skeleton-loader').length === 0;`.
//! `paint_loads_the_dashboard` is not in this crate. The call is the missing paint.

#[test]
fn loads_the_dashboard() {
    let markup = splora_web::paint_loads_the_dashboard();
    let loader_count = markup.matches("skeleton-loader").count();
    assert_eq!(
        loader_count, 0,
        "loads the dashboard: Cypress.$('.skeleton-loader').length === 0 after cy.waitForSkeletonGone(), got {loader_count} in {markup:?}"
    );
}
