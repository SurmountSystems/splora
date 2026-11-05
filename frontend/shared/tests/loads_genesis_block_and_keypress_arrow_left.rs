//! Cypress it() title: loads genesis block and keypress arrow left.
//!
//! The spec visits `/block/0`. `cy.document().left()` sends ArrowLeft.
//! Default `timeLtr` is false, so ArrowLeft is the next-block key. Height 0
//! is not the latest confirmed height, so the next link is shown and the
//! key navigates to height 1 and route `/block/1`. Both Next Block and
//! Previous Block icons are visible. This file does not sleep, scroll, or
//! read a block hash, a fee, or a price.

#[test]
fn loads_genesis_block_and_keypress_arrow_left() {
    let page = splora_frontend_shared::loads_genesis_block_and_keypress_arrow_left();

    assert_eq!(
        page.height, 1,
        "ArrowLeft on height 0 navigates to the next block, so the height is 1"
    );
    assert_eq!(
        page.route, "/block/1",
        "ArrowLeft on height 0 navigates to the next block, so the route is /block/1"
    );
    assert!(
        page.after_arrow_left.next_block_icon_visible,
        "cy.get('[ngbtooltip=\"Next Block\"]').should('be.visible') after ArrowLeft"
    );
    assert!(
        page.after_arrow_left.previous_block_icon_visible,
        "cy.get('[ngbtooltip=\"Previous Block\"]').should('be.visible') after ArrowLeft"
    );
}
