//! Native paint of the nymkappa theme option label.
//!
//! Calls the existing `paint_theme_option_label`. This test does not add a
//! paint function. This module does not open a window.

#[test]
fn paint_nymkappa_theme_option_label_matches_shared() {
    assert_eq!(
        splora_native::paint_theme_option_label("nymkappa"),
        "nymkappa"
    );
    assert_eq!(
        splora_native::paint_theme_option_label("softsimon"),
        "softsimon"
    );
    assert_eq!(
        splora_native::paint_theme_option_label("contrast"),
        "BlueMatt"
    );
    assert_eq!(splora_native::paint_theme_option_label("default"), "");
}
