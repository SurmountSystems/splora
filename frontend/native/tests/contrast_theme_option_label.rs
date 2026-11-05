//! Native paint of the contrast theme option label.
//!
//! `paint_theme_option_label` is not defined on `splora_native`.
//! This test does not add it. No window.

#[test]
fn paint_theme_option_label_matches_shared() {
    assert_eq!(
        splora_native::paint_theme_option_label("contrast"),
        "BlueMatt"
    );
    assert_eq!(splora_native::paint_theme_option_label("default"), "");
}
