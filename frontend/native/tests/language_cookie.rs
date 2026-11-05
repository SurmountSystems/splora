//! Native paint of the language choice cookie.
//!
//! English and German each set `lang` with the Angular expiry and `path=/`.
//! This test does not open a window and does not send an indexer request.

#[test]
fn paint_language_choice_sets_lang_cookie_and_sends_no_indexer_request() {
    assert_eq!(
        splora_native::paint_language_cookie("en"),
        "lang=en; expires=Thu, 18 Dec 2050 12:00:00 UTC; path=/"
    );
    assert_eq!(
        splora_native::paint_language_cookie("de"),
        "lang=de; expires=Thu, 18 Dec 2050 12:00:00 UTC; path=/"
    );
}
