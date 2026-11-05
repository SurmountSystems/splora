//! Cypress it() title: loads genesis block and click on the arrow left.
//!
//! Spec: ref/mempool/frontend/cypress/e2e/mainnet/mainnet.spec.ts lines 390-402.
//! The spec visits `/block/0`. The Next Block icon is visible and the
//! Previous Block icon is absent. The click is that Next Block icon, the
//! newer step. Both icons are then visible, and the route is `/block/1`.
//! The markup comes from the web paint. This file does not call a keypress
//! function and does not use `/mempool-block/0`.

#[test]
fn loads_genesis_block_and_click_on_the_arrow_left() {
    let painted: (String, String) =
        splora_web::paint_loads_genesis_block_and_click_on_the_arrow_left();
    let (at_genesis, after_next_block_click) = painted;

    assert!(
        at_genesis.contains("Next Block"),
        "cy.get('[ngbtooltip=\"Next Block\"]').should('be.visible') on /block/0: {at_genesis}"
    );
    assert!(
        !at_genesis.contains("Previous Block"),
        "cy.get('[ngbtooltip=\"Previous Block\"]').should('not.exist') on /block/0: {at_genesis}"
    );
    assert!(
        after_next_block_click.contains("Next Block"),
        "cy.get('[ngbtooltip=\"Next Block\"]').should('be.visible') after the Next Block click: {after_next_block_click}"
    );
    assert!(
        after_next_block_click.contains("Previous Block"),
        "cy.get('[ngbtooltip=\"Previous Block\"]').should('be.visible') after the Next Block click: {after_next_block_click}"
    );
    assert!(
        after_next_block_click.contains("/block/1"),
        "the Next Block click is the newer step, so the route is /block/1: {after_next_block_click}"
    );
    assert!(
        !at_genesis.contains("/mempool-block/0")
            && !after_next_block_click.contains("/mempool-block/0"),
        "this click does not open /mempool-block/0: {at_genesis} {after_next_block_click}"
    );
}
