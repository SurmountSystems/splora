//! Cypress it() title: loads the api screen.
//!
//! Signet visits `/signet`, waits for the skeleton, then clicks `#btn-docs`.
//! That control routes to `/docs`. It is not an indexer body.

use splora_frontend_shared::after_docs_button_loads_the_api_screen;

#[test]
fn loads_the_api_screen() {
    let view = after_docs_button_loads_the_api_screen();
    assert!(
        view.api_screen_active,
        "after cy.visit('/signet'), cy.waitForSkeletonGone(), and #btn-docs is clicked, the api screen is the active view"
    );
}
