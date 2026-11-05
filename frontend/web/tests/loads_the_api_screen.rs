//! Cypress it() title: loads the api screen.
//!
//! The click is `#btn-docs`, a routerLink to `/docs`. The shared function
//! is the authority for the destination. This test uses the same docs-button
//! input and the same route tables as the shared test. The painted
//! destination must equal that return, including the empty-path redirect.
//! A different control must not paint that destination.

use splora_frontend_shared::after_docs_button_loads_the_api_screen;

/// Non-liquid docs child routes. `path: ''` redirects to `faq`.
/// `path: 'api'` redirects to `api/rest`, and it is listed first so a
/// lookup that ignores the path cannot return `faq` by taking the first row.
fn bitcoin_docs_routes() -> [(&'static str, &'static str); 3] {
    [("api", "api/rest"), ("", "faq"), ("**", "faq")]
}

/// Liquid docs child routes. Every `redirectTo` in that branch is `api/rest`.
fn liquid_docs_routes() -> [(&'static str, &'static str); 3] {
    [("api", "api/rest"), ("**", "api/rest"), ("", "api/rest")]
}

#[test]
fn loads_the_api_screen() {
    let routes = bitcoin_docs_routes();
    let shared = after_docs_button_loads_the_api_screen("#btn-docs", &routes);
    let painted = splora_web::paint_loads_the_api_screen("#btn-docs", &routes);
    assert_eq!(
        painted.destination, shared.destination,
        "painted destination must equal the shared return"
    );
    assert_eq!(
        painted.api_screen_active, shared.api_screen_active,
        "painted api screen flag must equal the shared return"
    );
    assert!(
        shared.api_screen_active,
        "clicking #btn-docs activates the api screen"
    );
    assert_eq!(
        shared.destination, "faq",
        "client destination is the docs empty-path redirectTo"
    );
    assert_eq!(
        painted.destination, "faq",
        "painted destination is the docs empty-path redirectTo"
    );

    let other = after_docs_button_loads_the_api_screen("#btn-about", &routes);
    let other_painted = splora_web::paint_loads_the_api_screen("#btn-about", &routes);
    assert_eq!(
        other_painted.destination, other.destination,
        "a different control's painted destination must equal the shared return"
    );
    assert_eq!(other_painted.api_screen_active, other.api_screen_active);
    assert!(
        !other.api_screen_active,
        "a different control does not activate the api screen"
    );
    assert_ne!(
        other_painted.destination, "faq",
        "a different control must not paint the faq redirect"
    );
    assert_ne!(
        other.destination, "faq",
        "a different control must not receive the faq redirect"
    );

    let liquid = liquid_docs_routes();
    let liquid_shared = after_docs_button_loads_the_api_screen("#btn-docs", &liquid);
    let liquid_painted = splora_web::paint_loads_the_api_screen("#btn-docs", &liquid);
    assert_eq!(
        liquid_painted.destination, liquid_shared.destination,
        "liquid painted destination must equal the shared return"
    );
    assert_eq!(
        liquid_painted.api_screen_active,
        liquid_shared.api_screen_active
    );
    assert!(liquid_shared.api_screen_active);
    assert_eq!(
        liquid_shared.destination, "api/rest",
        "the clicked docs button follows the empty-path redirect from the route table"
    );
    assert_eq!(liquid_painted.destination, "api/rest");
    assert_ne!(
        liquid_painted.destination, "faq",
        "the liquid route table must not paint the non-liquid faq redirect"
    );
}
