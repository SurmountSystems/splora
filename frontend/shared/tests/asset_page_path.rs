//! Cypress `should link to the asset page from the unblinded tx`.
//! `ref/mempool/frontend/cypress/e2e/liquidtestnet/liquidtestnet.spec.ts` lines 146-151.
//! The amount link is `/assets/asset/` plus the output asset id.
//! `asset_page_path` is not defined in this file.

const ASSET_ID: &str = "38fca2d939696061a8f76d4e6b5eecd54e3b4221c846f24a6b279e79952850a5";

/// A different 64-hex asset id. Its page path must not contain `ASSET_ID`.
const OTHER_ASSET_ID: &str = "0000000000000000000000000000000000000000000000000000000000000001";

#[test]
fn unblinded_tx_links_to_asset_page() {
    let path = splora_frontend_shared::asset_page_path(ASSET_ID);
    assert!(
        path.contains(
            "/assets/asset/38fca2d939696061a8f76d4e6b5eecd54e3b4221c846f24a6b279e79952850a5"
        ),
        "unblinded amount link must contain the asset page path, got {path}"
    );
    let other = splora_frontend_shared::asset_page_path(OTHER_ASSET_ID);
    assert!(
        !other.contains(ASSET_ID),
        "a different asset id must not link to this asset, got {other}"
    );
}
