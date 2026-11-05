//! Native paint of the softsimon theme option label.
//!
//! Calls the existing `paint_theme_option_label`. This test does not add a
//! paint function. No window.

#[test]
fn paint_softsimon_theme_option_label_matches_shared() {
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
