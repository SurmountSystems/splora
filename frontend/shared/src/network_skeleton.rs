//! Skeleton-loader count across Cypress `changeNetwork` clicks.
//!
//! `changeNetwork` clicks `a.${network}` and waits until
//! `Cypress.$('.skeleton-loader').length === 0`. Row 104's endpoint cell
//! is `none until the next GET`, so each click flashes the loader and then
//! clears it. This module does not issue an indexer GET.

/// Settled `.skeleton-loader` count after changing to `first`, then `second`.
///
/// The start network is mainnet and the start count is 0. During each change
/// the count becomes non-zero while the click is in flight. There is no GET,
/// so the count clears back to 0. A change to the same name as the current
/// network still flashes and clears, because the Cypress helper still clicks
/// and waits for the loader to be gone. After the first change the current
/// network is `first`. After the second change it is `second`. The returned
/// value is the count after the second clear.
pub fn skeleton_loader_count_after_network_changes(first: &str, second: &str) -> u32 {
    let mut network = "mainnet";
    let mut skeleton: u32 = 0;
    for next in [first, second] {
        let _current = network;
        skeleton = skeleton.saturating_add(1);
        network = next;
        let loader_while_click_in_flight = skeleton;
        if loader_while_click_in_flight > 0 {
            skeleton = 0;
        }
    }
    let _settled_on = network;
    skeleton
}

/// `cy.changeNetwork('signet')` then `cy.changeNetwork('mainnet')`.
///
/// The settled count is whatever remains after both clears. This row performs
/// no GET, so both clears leave the count at 0.
pub fn loads_skeleton_when_changes_between_networks() -> u32 {
    skeleton_loader_count_after_network_changes("signet", "mainnet")
}
