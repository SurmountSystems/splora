//! Nymkappa theme option label paint matches the shared function.
//!
//! nymkappa is nymkappa, softsimon is softsimon, contrast is BlueMatt,
//! and default is an empty string.

#[test]
fn paint_nymkappa_theme_option_label_matches_shared() {
    assert_eq!(splora_web::paint_theme_option_label("nymkappa"), "nymkappa");
    assert_eq!(
        splora_web::paint_theme_option_label("softsimon"),
        "softsimon"
    );
    assert_eq!(splora_web::paint_theme_option_label("contrast"), "BlueMatt");
    assert_eq!(splora_web::paint_theme_option_label("default"), "");
}
