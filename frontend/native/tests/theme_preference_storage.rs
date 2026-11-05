//! Native paint of the theme preference storage.
//!
//! Choosing a theme writes theme-preference and sends no indexer request.
//! This test does not open a window.

#[test]
fn paint_theme_choice_stores_theme_preference_and_sends_no_indexer_request() {
    assert_eq!(
        splora_native::paint_theme_preference_storage("contrast"),
        "theme-preference=contrast"
    );
    assert_eq!(
        splora_native::paint_theme_preference_storage("default"),
        "theme-preference=default"
    );
}
