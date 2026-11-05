//! Language choice cookie for the footer selector.
//!
//! Choosing a language writes this cookie and sends no indexer request.
//! The function is not defined in this file.

#[test]
fn language_choice_sets_lang_cookie_and_sends_no_indexer_request() {
    assert_eq!(
        splora_frontend_shared::language_cookie("en"),
        "lang=en; expires=Thu, 18 Dec 2050 12:00:00 UTC; path=/"
    );
    assert_eq!(
        splora_frontend_shared::language_cookie("de"),
        "lang=de; expires=Thu, 18 Dec 2050 12:00:00 UTC; path=/"
    );
}
