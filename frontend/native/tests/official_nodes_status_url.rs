//! Native paint of the official nodes status URL.
//!
//! The four cases call `paint_official_nodes_status_url` and compare the
//! returned status URL. This test does not construct a window and does not
//! open a GPUI window.

#[test]
fn paint_official_nodes_status_url_is_built_in_the_browser() {
    assert_eq!(
        splora_native::paint_official_nodes_status_url(
            "mempool.space",
            "/nodes",
            true,
            "https://ignored.example/x"
        ),
        "https://mempool.space/status"
    );
    assert_eq!(
        splora_native::paint_official_nodes_status_url(
            "mempool.space",
            "/testnet4/nodes",
            true,
            "https://ignored.example/x"
        ),
        "https://mempool.space/testnet4/status"
    );
    assert_eq!(
        splora_native::paint_official_nodes_status_url(
            "mempool.space",
            "/nodes",
            false,
            "https://example.com/foo"
        ),
        "https://example.com/status"
    );
    assert_eq!(
        splora_native::paint_official_nodes_status_url(
            "mempool.space",
            "/testnet4/nodes",
            false,
            "https://example.com:8443/foo"
        ),
        "https://example.com/testnet4/status"
    );
}
