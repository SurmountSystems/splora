//! Softsimon theme option label paint matches the shared function.
//!
//! softsimon is softsimon, contrast is BlueMatt, and any other value is empty.

#[test]
fn paint_softsimon_theme_option_label_matches_shared() {
    assert_eq!(
        splora_web::paint_theme_option_label("softsimon"),
        "softsimon"
    );
    assert_eq!(splora_web::paint_theme_option_label("contrast"), "BlueMatt");
    assert_eq!(splora_web::paint_theme_option_label("default"), "");
}
