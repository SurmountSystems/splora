//! Softsimon theme option label for the theme selector.
//!
//! The softsimon option text is softsimon. Contrast stays BlueMatt.
//! The default option is an empty string.

#[test]
fn softsimon_theme_option_label_is_softsimon() {
    assert_eq!(
        splora_frontend_shared::theme_option_label("softsimon"),
        "softsimon"
    );
    assert_eq!(
        splora_frontend_shared::theme_option_label("contrast"),
        "BlueMatt"
    );
    assert_eq!(splora_frontend_shared::theme_option_label("default"), "");
}
