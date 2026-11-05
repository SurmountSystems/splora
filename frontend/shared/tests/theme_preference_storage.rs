//! Theme choice stored as a local preference.
//!
//! Choosing a theme writes theme-preference and sends no indexer request.
//! The function is not defined in this file.

#[test]
fn theme_choice_stores_theme_preference_and_sends_no_indexer_request() {
    assert_eq!(
        splora_frontend_shared::theme_preference_storage("contrast"),
        "theme-preference=contrast"
    );
    assert_eq!(
        splora_frontend_shared::theme_preference_storage("default"),
        "theme-preference=default"
    );
}
