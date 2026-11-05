//! Cypress it() title: expands and collapses the block details.
//!
//! The spec visits `/block/0` with no `showDetails=true` query, so
//! `#details` starts hidden. It then clicks `.btn.btn-outline-info`.
//! That button is the details control when the audit button is absent.
//! `cy.get('#details').should('be.visible')` follows the first click.
//! The same click again requires `cy.get('#details').should('not.be.visible')`.
//! This it() does not read a block JSON body.

use splora_frontend_shared::expands_and_collapses_the_block_details;

#[test]
fn expands_and_collapses_the_block_details() {
    let collapsed = expands_and_collapses_the_block_details(true);
    assert!(
        !collapsed.details_visible,
        "cy.get('#details').should('not.be.visible') after the second .btn.btn-outline-info click"
    );
    assert!(!collapsed.show_details);
    assert_eq!(collapsed.query_show_details, "false");
    assert_eq!(collapsed.fragment, "block");

    let expanded = expands_and_collapses_the_block_details(false);
    assert!(
        expanded.details_visible,
        "cy.get('#details').should('be.visible') after the first .btn.btn-outline-info click"
    );
    assert!(expanded.show_details);
    assert_eq!(expanded.query_show_details, "true");
    assert_eq!(expanded.fragment, "details");

    let closed_again = expands_and_collapses_the_block_details(expanded.show_details);
    assert!(
        !closed_again.details_visible,
        "cy.get('#details').should('not.be.visible') after the second .btn.btn-outline-info click"
    );
    assert!(!closed_again.show_details);
    assert_eq!(closed_again.query_show_details, "false");
    assert_eq!(closed_again.fragment, "block");
}
