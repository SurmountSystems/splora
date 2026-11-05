//! Cypress `should link to the asset page from the unblinded tx`.
//! The link target is `/assets/asset/` plus the given asset id.

const CYPRESS_ASSET_ID: &str = "38fca2d939696061a8f76d4e6b5eecd54e3b4221c846f24a6b279e79952850a5";

const OTHER_ASSET_ID: &str = "0000000000000000000000000000000000000000000000000000000000000001";

#[test]
fn unblinded_tx_links_to_asset_page() {
    let linked = splora_web::unblinded_tx_asset_page_path(CYPRESS_ASSET_ID);
    assert!(
        linked.contains(&format!("/assets/asset/{CYPRESS_ASSET_ID}")),
        "unblinded tx asset link must contain the Cypress asset page path, got {linked}"
    );
    let other = splora_web::unblinded_tx_asset_page_path(OTHER_ASSET_ID);
    assert!(
        other.contains(&format!("/assets/asset/{OTHER_ASSET_ID}")),
        "a different asset id must be forwarded into the asset page path, got {other}"
    );
    assert!(
        !other.contains(CYPRESS_ASSET_ID),
        "a different asset id must not contain the Cypress asset id, got {other}"
    );
}
