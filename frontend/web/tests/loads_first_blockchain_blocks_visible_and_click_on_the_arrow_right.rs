//! Cypress it() title: loads first blockchain blocks visible and click on the arrow right.
//!
//! Spec: ref/mempool/frontend/cypress/e2e/mainnet/mainnet.spec.ts lines 375-388.
//! The newest confirmed block is clicked. The Next Block icon is absent and
//! the Previous Block icon is visible. The second click is that Previous
//! Block icon. Both icons are then visible, and the route is `/block/1`.
//! The markup comes from the web paint. This file does not invent a block hash.

#[test]
fn loads_first_blockchain_blocks_visible_and_click_on_the_arrow_right() {
    let painted: (String, String) =
        splora_web::paint_loads_first_blockchain_blocks_visible_and_click_on_the_arrow_right();
    let (after_selecting_newest, after_previous_block_click) = painted;

    assert!(
        after_selecting_newest.contains("Previous Block"),
        "cy.get('[ngbtooltip=\"Previous Block\"]').should('be.visible') after the newest block click: {after_selecting_newest}"
    );
    assert!(
        !after_selecting_newest.contains("Next Block"),
        "cy.get('[ngbtooltip=\"Next Block\"]').should('not.exist') after the newest block click: {after_selecting_newest}"
    );
    assert!(
        after_previous_block_click.contains("Next Block"),
        "cy.get('[ngbtooltip=\"Next Block\"]').should('be.visible') after the Previous Block click: {after_previous_block_click}"
    );
    assert!(
        after_previous_block_click.contains("Previous Block"),
        "cy.get('[ngbtooltip=\"Previous Block\"]').should('be.visible') after the Previous Block click: {after_previous_block_click}"
    );
    assert!(
        after_previous_block_click.contains("/block/1"),
        "the Previous Block click is the older-block step, so the route is /block/1: {after_previous_block_click}"
    );
}
