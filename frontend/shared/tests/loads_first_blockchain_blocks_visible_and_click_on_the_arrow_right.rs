//! Cypress it() title: loads first blockchain blocks visible and click on the arrow right.
//!
//! The spec visits `/`, waits until the skeleton is gone (that wait is not
//! an assertion), and clicks `bitcoin-block-offset-0-index-0`. That index
//! is the newest confirmed block. The Next Block icon is absent and the
//! Previous Block icon is visible. The second click is the Previous Block
//! icon. Default `timeLtr` is false, so that click is the older-block
//! step, the same direction as ArrowRight. One older step from the local
//! tip is height 1 and route `/block/1`, and both icons are then visible.
//! This file does not sleep and does not read a block hash, a fee, or a
//! price.

#[test]
fn loads_first_blockchain_blocks_visible_and_click_on_the_arrow_right() {
    let page =
        splora_frontend_shared::loads_first_blockchain_blocks_visible_and_click_on_the_arrow_right(
        );

    assert!(
        !page.after_selecting_newest.next_block_icon_visible,
        "cy.get('[ngbtooltip=\"Next Block\"]').should('not.exist') after the newest block click"
    );
    assert!(
        page.after_selecting_newest.previous_block_icon_visible,
        "cy.get('[ngbtooltip=\"Previous Block\"]').should('be.visible') after the newest block click"
    );
    assert!(
        page.after_previous_block_click.next_block_icon_visible,
        "cy.get('[ngbtooltip=\"Next Block\"]').should('be.visible') after the Previous Block click"
    );
    assert!(
        page.after_previous_block_click.previous_block_icon_visible,
        "cy.get('[ngbtooltip=\"Previous Block\"]').should('be.visible') after the Previous Block click"
    );
    assert_eq!(
        page.height, 1,
        "the Previous Block click is the older-block step, so the height is 1"
    );
    assert_eq!(
        page.route, "/block/1",
        "the Previous Block click is the older-block step, so the route is /block/1"
    );
}
