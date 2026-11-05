//! Static host attribute for the difficulty-adjustment element.
//!
//! Cypress only checks that the host exists. The attribute value is
//! `difficulty-adjustment`. The function is not defined in this file.
//! This test does not check a percent, a block count, a timestamp, or a price.

#[test]
fn shows_the_difficulty_adjustment_stats() {
    let host = splora_frontend_shared::shows_the_difficulty_adjustment_stats();
    let host = host.trim();
    assert!(
        host.contains("difficulty-adjustment"),
        "static host attribute, got {host}"
    );
}
