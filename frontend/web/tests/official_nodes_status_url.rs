//! Official nodes status URL painted in the browser.
//!
//! The status URL text comes from `splora_web::paint_official_nodes_status_url`.
//! The paint function is not defined in this file.

#[test]
fn paint_official_nodes_status_url_is_built_in_the_browser() {
    assert_eq!(
        splora_web::paint_official_nodes_status_url(
            "mempool.space",
            "/nodes",
            true,
            "https://ignored.example/x"
        ),
        "https://mempool.space/status"
    );
    assert_eq!(
        splora_web::paint_official_nodes_status_url(
            "mempool.space",
            "/testnet4/nodes",
            true,
            "https://ignored.example/x"
        ),
        "https://mempool.space/testnet4/status"
    );
    assert_eq!(
        splora_web::paint_official_nodes_status_url(
            "mempool.space",
            "/nodes",
            false,
            "https://example.com/foo"
        ),
        "https://example.com/status"
    );
    assert_eq!(
        splora_web::paint_official_nodes_status_url(
            "mempool.space",
            "/testnet4/nodes",
            false,
            "https://example.com:8443/foo"
        ),
        "https://example.com/testnet4/status"
    );
}
