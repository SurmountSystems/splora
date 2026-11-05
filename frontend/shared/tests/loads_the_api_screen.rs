//! Cypress it() title: loads the api screen.
//!
//! Mainnet visits `/`, waits until `.skeleton-loader` length is 0, clicks
//! `#btn-docs`, and waits 1000ms. That control's routerLink is `/docs`.
//! The non-liquid docs route `path: ''` has `redirectTo: 'faq'`.

use splora_frontend_shared::after_docs_button_loads_the_api_screen;

/// Non-liquid docs child routes. `path: ''` redirects to `faq`.
/// `path: 'api'` redirects to `api/rest`, and it is listed first so a
/// lookup that ignores the path cannot return `faq` by taking the first row.
fn bitcoin_docs_routes() -> [(&'static str, &'static str); 3] {
    [("api", "api/rest"), ("", "faq"), ("**", "faq")]
}

/// Liquid docs child routes. Every `redirectTo` in that branch is `api/rest`,
/// so a docs click must not keep the non-liquid `faq` destination.
fn liquid_docs_routes() -> [(&'static str, &'static str); 3] {
    [("api", "api/rest"), ("**", "api/rest"), ("", "api/rest")]
}

#[test]
fn loads_the_api_screen() {
    let routes = bitcoin_docs_routes();
    let view = after_docs_button_loads_the_api_screen("#btn-docs", &routes);
    assert!(
        view.api_screen_active,
        "clicking #btn-docs activates the api screen"
    );
    assert_eq!(
        view.destination, "faq",
        "client destination is the docs empty-path redirectTo"
    );

    let other = after_docs_button_loads_the_api_screen("#btn-about", &routes);
    assert!(
        !other.api_screen_active,
        "a different control does not activate the api screen"
    );
    assert_ne!(
        other.destination, "faq",
        "a different control must not receive the faq redirect"
    );

    let liquid = liquid_docs_routes();
    let liquid_view = after_docs_button_loads_the_api_screen("#btn-docs", &liquid);
    assert!(liquid_view.api_screen_active);
    assert_eq!(
        liquid_view.destination, "api/rest",
        "the clicked docs button follows the empty-path redirect from the route table"
    );
    assert_ne!(liquid_view.destination, "faq");
}
