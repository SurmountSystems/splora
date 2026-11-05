//! Cypress it() title: loads skeleton when changes between networks.
//!
//! Spec: ref/mempool/frontend/cypress/e2e/mainnet/mainnet.spec.ts
//! changeNetwork clicks `a.${network}` and waits until
//! Cypress.$('.skeleton-loader').length === 0.
//! A real network change shows the loader before that settled check.
//! Two different name pairs must not collapse to one ignored-input result.

#[test]
fn loads_skeleton_when_changes_between_networks() {
    let signet_then_mainnet =
        splora_frontend_shared::loads_skeleton_when_changes_between_networks("signet", "mainnet");
    assert_eq!(
        signet_then_mainnet.settled, 0,
        "Cypress.$('.skeleton-loader').length === 0 after changeNetwork('signet') and changeNetwork('mainnet')"
    );
    assert!(
        signet_then_mainnet.peak > 0,
        "switching networks shows .skeleton-loader before waitForSkeletonGone"
    );
    assert_eq!(signet_then_mainnet.network, "mainnet");

    let testnet_then_signet =
        splora_frontend_shared::loads_skeleton_when_changes_between_networks("testnet", "signet");
    assert!(testnet_then_signet.peak > 0);
    assert_eq!(testnet_then_signet.settled, 0);
    assert_eq!(testnet_then_signet.network, "signet");
    assert_ne!(
        signet_then_mainnet, testnet_then_signet,
        "different network name pairs must not share one ignored-input record"
    );

    let stayed_on_mainnet =
        splora_frontend_shared::loads_skeleton_when_changes_between_networks("mainnet", "mainnet");
    assert_eq!(stayed_on_mainnet.peak, 0);
    assert_eq!(stayed_on_mainnet.settled, 0);
    assert_eq!(stayed_on_mainnet.network, "mainnet");
}
