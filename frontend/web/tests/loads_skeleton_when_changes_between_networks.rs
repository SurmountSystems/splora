//! Cypress `it('loads skeleton when changes between networks')`.
//! File: `ref/mempool/frontend/cypress/e2e/mainnet/mainnet.spec.ts` line 406.
//!
//! The it() visits `/`, calls `cy.waitForSkeletonGone()`, then
//! `cy.changeNetwork('signet')` and `cy.changeNetwork('mainnet')`.
//! `changeNetwork` in `ref/mempool/frontend/cypress/support/commands.ts`
//! clicks `a.${network}` and then calls `cy.waitForSkeletonGone()`.
//! That helper checks `Cypress.$('.skeleton-loader').length === 0`.
//!
//! `splora_frontend_shared::loads_skeleton_when_changes_between_networks`
//! takes that same sequence (`first`, then `second`) and returns the
//! loader transition. The settled field is the count the Cypress check
//! sees after both waits.
//! The paint repeats one `skeleton-loader` element for each settled count.

#[test]
fn loads_skeleton_when_changes_between_networks() {
    let shared =
        splora_frontend_shared::loads_skeleton_when_changes_between_networks("signet", "mainnet");
    let markup =
        splora_web::paint_loads_skeleton_when_changes_between_networks("signet", "mainnet");
    let loader_count = markup.matches("skeleton-loader").count();
    assert_eq!(
        loader_count, shared.settled as usize,
        "painted .skeleton-loader count must equal the shared settled return, got {loader_count} from {shared:?} in {markup:?}"
    );
    assert_eq!(
        shared.settled, 0,
        "Cypress.$('.skeleton-loader').length === 0 after changeNetwork('signet') and changeNetwork('mainnet'), got {}",
        shared.settled
    );
}
