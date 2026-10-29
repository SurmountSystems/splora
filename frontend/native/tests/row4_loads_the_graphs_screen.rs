//! Row 4. Cypress it() title: loads the graphs screen.
//!
//! The it() visits `/testnet4`, waits until
//! `Cypress.$('.skeleton-loader').length === 0`, clicks `#btn-graphs`,
//! and that control routes to `/graphs`.
//! `paint_loads_the_graphs_screen` is not defined on `splora_native`.
//! This test does not open a window and does not start an event loop.

const CYPRESS_IT: &str = "loads the graphs screen";

#[test]
fn row4_loads_the_graphs_screen() {
    let painted: String = splora_native::paint_loads_the_graphs_screen(
        "/testnet4",
        "#btn-graphs",
        "/graphs",
        0,
        1000,
    );
    assert!(
        painted.contains("#btn-graphs")
            && painted.contains("/graphs")
            && !painted.contains("skeleton-loader"),
        "{CYPRESS_IT}: cy.visit('/testnet4'); cy.get('#btn-graphs').click() routes to /graphs; Cypress.$('.skeleton-loader').length === 0; cy.wait(1000); got {painted:?}"
    );
}
