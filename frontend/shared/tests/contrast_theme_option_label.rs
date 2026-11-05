//! Contrast theme option label for the theme selector.
//!
//! The contrast option text is BlueMatt. The default option is not BlueMatt.
//! The function is not defined in this file.

#[test]
fn contrast_theme_option_label_is_bluematt() {
    assert_eq!(
        splora_frontend_shared::theme_option_label("contrast"),
        "BlueMatt"
    );
    assert_ne!(
        splora_frontend_shared::theme_option_label("default"),
        "BlueMatt"
    );
}
