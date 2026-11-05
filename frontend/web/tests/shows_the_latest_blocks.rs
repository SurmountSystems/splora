//! Static mining-dashboard host attribute for the latest-blocks element.
//!
//! Cypress only checks that the host exists. The attribute value is
//! `latest-blocks`. The paint function is not defined in this file.
//! This test does not check a digit, a percent, a block count, or a price.

#[test]
fn shows_the_latest_blocks() {
    let host = splora_web::paint_shows_the_latest_blocks();
    let host = host.trim();
    assert!(
        host.contains("latest-blocks"),
        "static host attribute, got {host}"
    );
}
