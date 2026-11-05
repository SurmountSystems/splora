//! Static host attribute for the latest-blocks element.
//!
//! Cypress only checks that the host exists. The attribute value is
//! `latest-blocks`. The function is not defined in this file.
//! This test does not invent a height, a hash, a fee, a pool name,
//! a transaction count, or a timestamp.

#[test]
fn shows_the_latest_blocks() {
    let host = splora_frontend_shared::shows_the_latest_blocks();
    assert_eq!(host, "latest-blocks");
}
