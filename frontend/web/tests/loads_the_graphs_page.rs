//! Cypress title: loads the graphs page.
//!
//! Spec: ref/mempool/frontend/cypress/e2e/liquid/liquid.spec.ts line 52.
//! The web paint must return `loads_the_graphs_page`.
//! The comparison is that paint call against the shared function, not a
//! copied route string.

#[test]
fn loads_the_graphs_page() {
    assert_eq!(
        splora_web::paint_loads_the_graphs_page(),
        splora_frontend_shared::loads_the_graphs_page()
    );
}
