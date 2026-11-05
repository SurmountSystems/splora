//! Row selected by the parity awk: endpoint cell `none`, web and native are
//! `partial`. Cypress it() title: loads the graphs screen.
//!
//! ```text
//! cy.visit('/testnet4');
//! cy.waitForSkeletonGone();
//! cy.get('#btn-graphs').click().then(() => {
//!     cy.wait(1000);
//! });
//! ```
//!
//! That it() is the behavior. It is not the not-served sentence.

use splora_frontend_shared::after_graphs_button_loads_the_graphs_screen;

#[test]
fn testnet4_graphs_screen_opens_from_the_graphs_nav_button() {
    let view = after_graphs_button_loads_the_graphs_screen();
    assert_eq!(
        view.button_id, "btn-graphs",
        "cy.get('#btn-graphs') after cy.visit('/testnet4') and cy.waitForSkeletonGone()"
    );
    assert_eq!(
        view.route, "/graphs",
        "clicking #btn-graphs opens the graphs screen route, not an indexer path"
    );
    assert!(
        view.graphs_screen_active,
        "after cy.visit('/testnet4'), cy.waitForSkeletonGone(), cy.get('#btn-graphs').click(), and cy.wait(1000), the graphs screen is loaded"
    );
}
