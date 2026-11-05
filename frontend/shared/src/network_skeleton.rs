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

/// Peak and settled `.skeleton-loader` counts after two `changeNetwork` clicks.
///
/// The page starts on mainnet, where `waitForSkeletonGone` has already seen
/// a count of 0. `network` is the name in effect after the second click.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NetworkSkeletonChange {
    /// Highest `.skeleton-loader` count while a click was in flight.
    pub peak: u32,
    /// Count after the websocket response clears `isLoadingWebSocket$`.
    pub settled: u32,
    /// Network after both clicks. The start network is mainnet.
    pub network: String,
}

/// `cy.changeNetwork(first)` then `cy.changeNetwork(second)`, from mainnet.
///
/// Dashboard `mempoolTable` renders `.skeleton-loader` from the `#loading`
/// template while `(isLoadingWebSocket$ | async) === false && mempoolInfoData.value`
/// is false. `WebsocketService.startSubscription` sets that flag true when
/// `networkChanged$` actually differs from the current network and calls
/// `reconnectWebsocket`. The flag goes false on the websocket response, which
/// is the settled `Cypress.$('.skeleton-loader').length === 0` check.
/// The same name returns early in `networkChanged$` and does not show the loader.
pub fn loads_skeleton_when_changes_between_networks(
    first: &str,
    second: &str,
) -> NetworkSkeletonChange {
    let mut network = "mainnet".to_string();
    let mut skeleton: u32 = 0;
    let mut peak: u32 = 0;
    for next in [first, second] {
        if next == network {
            continue;
        }
        skeleton = 1;
        if skeleton > peak {
            peak = skeleton;
        }
        network = next.to_string();
        skeleton = 0;
    }
    NetworkSkeletonChange {
        peak,
        settled: skeleton,
        network,
    }
}
