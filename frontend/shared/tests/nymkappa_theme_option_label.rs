//! Nymkappa theme option label for the theme selector.
//!
//! The nymkappa option text is nymkappa. Softsimon stays softsimon.
//! Contrast stays BlueMatt. The default option is an empty string.

#[test]
fn nymkappa_theme_option_label_is_nymkappa() {
    assert_eq!(
        splora_frontend_shared::theme_option_label("nymkappa"),
        "nymkappa"
    );
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
