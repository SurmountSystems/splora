//! Cypress `it('loads the api screen')`.
//! File: `ref/mempool/frontend/cypress/e2e/signet/signet.spec.ts`, lines 63-69.
//!
//! Visit `/signet`, wait until the skeleton is gone, then click `#btn-docs`.
//! That control routes to `/docs`. It is not an indexer body.
//! `paint_loads_the_api_screen` is not in this crate. The call is the missing paint.

#[test]
fn loads_the_api_screen() {
    let markup = splora_web::paint_loads_the_api_screen();
    assert!(
        activating_the_docs_control_shows_the_api_screen(&markup),
        "after cy.visit('/signet'), cy.waitForSkeletonGone(), and #btn-docs is clicked, the api screen is the active view: {markup}"
    );
}

/// `#btn-docs` is the control Cypress clicks. The api screen is shown only
/// when that control is painted once and the rest of the paint names the
/// api screen. The control id alone is not the screen. `not-served` is not
/// the screen.
fn activating_the_docs_control_shows_the_api_screen(markup: &str) -> bool {
    let control = r#"id="btn-docs""#;
    if markup.matches(control).count() != 1 {
        return false;
    }
    let rest = markup.replace(control, "");
    if rest.trim().is_empty() || rest.contains("not-served") {
        return false;
    }
    rest.to_ascii_lowercase().contains("api")
}
