//! Cypress it() title: loads first blockchain block visible and keypress arrow right.
//!
//! Spec: ref/mempool/frontend/cypress/e2e/mainnet/mainnet.spec.ts lines 287-299.
//! `right` dispatches keydown ArrowRight. After the newest confirmed block
//! is selected, Previous Block is visible and Next Block is absent. After
//! one ArrowRight, both icons are visible. The labels come from the web paint.

#[test]
fn loads_first_blockchain_block_visible_and_keypress_arrow_right() {
    let painted = splora_web::paint_loads_first_blockchain_block_visible_and_keypress_arrow_right();
    let (after_selecting_newest, after_arrow_right) = painted;

    assert!(
        after_selecting_newest.contains("Previous Block"),
        "cy.get('[ngbtooltip=\"Previous Block\"]').should('be.visible') after the newest block click: {after_selecting_newest}"
    );
    assert!(
        !after_selecting_newest.contains("Next Block"),
        "cy.get('[ngbtooltip=\"Next Block\"]').should('not.exist') after the newest block click: {after_selecting_newest}"
    );
    assert!(
        after_arrow_right.contains("Next Block"),
        "cy.get('[ngbtooltip=\"Next Block\"]').should('be.visible') after ArrowRight: {after_arrow_right}"
    );
    assert!(
        after_arrow_right.contains("Previous Block"),
        "cy.get('[ngbtooltip=\"Previous Block\"]').should('be.visible') after ArrowRight: {after_arrow_right}"
    );
}
