//! Static mining-dashboard host attribute for the difficulty-adjustment element.
//!
//! Cypress only checks that the host exists. The attribute value is
//! `difficulty-adjustment`. The paint function is not defined in this file.
//! This test does not check a digit, a percent, a block count, or a price.

#[test]
fn shows_the_difficulty_adjustment_stats() {
    let host = splora_web::paint_shows_the_difficulty_adjustment_stats();
    let host = host.trim();
    assert!(
        host.contains("difficulty-adjustment"),
        "static host attribute, got {host}"
    );
}
