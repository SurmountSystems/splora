//! Cypress it() title: loads genesis block and keypress arrow right.
//!
//! The spec visits `/block/0`. `cy.document().right()` sends ArrowRight.
//! Default `timeLtr` is false, so ArrowRight is the previous-block key.
//! Height 0 hides that previous link, so the key does not navigate. After
//! the key, the Next Block icon is visible, the Previous Block icon is
//! absent, and the route stays `/block/0`. The markup comes from the web
//! paint. This file does not hard-code a block hash, a fee, or a price.

#[test]
fn loads_genesis_block_and_keypress_arrow_right() {
    let painted = splora_web::paint_loads_genesis_block_and_keypress_arrow_right();

    assert!(
        painted.contains("Next Block"),
        "cy.get('[ngbtooltip=\"Next Block\"]').should('be.visible') after ArrowRight: {painted}"
    );
    assert!(
        !painted.contains("Previous Block"),
        "cy.get('[ngbtooltip=\"Previous Block\"]').should('not.exist') after ArrowRight: {painted}"
    );
    assert!(
        painted.contains("/block/0"),
        "ArrowRight on height 0 does not navigate, so the route stays /block/0: {painted}"
    );
}
