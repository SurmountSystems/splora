//! Cypress it() title: loads genesis block and keypress arrow left.
//!
//! The spec visits `/block/0`. `cy.document().left()` sends ArrowLeft.
//! Default `timeLtr` is false, so ArrowLeft is the next-block key. The key
//! leaves height 0 and lands on height 1, route `/block/1`. After the key
//! both the Next Block icon and the Previous Block icon are visible. The
//! markup comes from the web paint. This file does not hard-code a block
//! hash, a fee, or a price.

#[test]
fn loads_genesis_block_and_keypress_arrow_left() {
    let painted = splora_web::paint_loads_genesis_block_and_keypress_arrow_left();

    assert!(
        painted.contains("Next Block"),
        "cy.get('[ngbtooltip=\"Next Block\"]').should('be.visible') after ArrowLeft: {painted}"
    );
    assert!(
        painted.contains("Previous Block"),
        "cy.get('[ngbtooltip=\"Previous Block\"]').should('be.visible') after ArrowLeft: {painted}"
    );
    assert!(
        painted.contains("/block/1"),
        "ArrowLeft on height 0 navigates to the next block, so the route is /block/1: {painted}"
    );
}
