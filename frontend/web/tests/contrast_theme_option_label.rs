//! Contrast theme option label paint matches the shared function.
//!
//! The contrast option text is BlueMatt. Any other value is an empty string.
//! The paint function is not defined in the web crate yet.

#[test]
fn paint_theme_option_label_matches_shared() {
    assert_eq!(splora_web::paint_theme_option_label("contrast"), "BlueMatt");
    assert_eq!(splora_web::paint_theme_option_label("default"), "");
}
