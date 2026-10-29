//! Cypress it() title: loads the api screen.
//!
//! Signet spec `ref/mempool/frontend/cypress/e2e/signet/signet.spec.ts`
//! visits `/signet`, waits for the skeleton, then clicks `#btn-docs`.
//! That control routes to `/docs`. It is not an indexer body.
//! `paint_loads_the_api_screen` is not defined on `splora_native`.

const CYPRESS_IT: &str = "loads the api screen";

#[test]
fn loads_the_api_screen() {
    let painted: String = splora_native::paint_loads_the_api_screen();
    assert!(
        painted.contains("btn-docs"),
        "{CYPRESS_IT}: cy.get('#btn-docs').click() missing from {painted}"
    );
    assert!(
        painted.contains("/docs"),
        "{CYPRESS_IT}: #btn-docs routes to /docs, missing from {painted}"
    );
    assert!(
        painted.contains("/signet"),
        "{CYPRESS_IT}: cy.visit('/signet') missing from {painted}"
    );
    assert!(
        painted.contains("api_screen_active=true"),
        "{CYPRESS_IT}: the api screen is not the active view in {painted}"
    );
}
