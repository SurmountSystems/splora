//! Cypress it() title: loads first blockchain block visible and keypress arrow right.
//!
//! The spec visits `/`, waits until the skeleton is gone, and clicks
//! `bitcoin-block-offset-0-index-0`. That index is the newest confirmed
//! block. The Next Block icon is absent and the Previous Block icon is
//! visible. `cy.document().right()` sends ArrowRight. With the default
//! right-to-left time direction, that key steps one block older, and both
//! icons are then visible. This file does not read a block hash or a fee.

#[test]
fn loads_first_blockchain_block_visible_and_keypress_arrow_right() {
    let icons =
        splora_frontend_shared::loads_first_blockchain_block_visible_and_keypress_arrow_right();

    assert!(
        !icons.after_selecting_newest.next_block_icon_visible,
        "cy.get('[ngbtooltip=\"Next Block\"]').should('not.exist') after the newest block click"
    );
    assert!(
        icons.after_selecting_newest.previous_block_icon_visible,
        "cy.get('[ngbtooltip=\"Previous Block\"]').should('be.visible') after the newest block click"
    );
    assert!(
        icons.after_arrow_right.next_block_icon_visible,
        "cy.get('[ngbtooltip=\"Next Block\"]').should('be.visible') after ArrowRight"
    );
    assert!(
        icons.after_arrow_right.previous_block_icon_visible,
        "cy.get('[ngbtooltip=\"Previous Block\"]').should('be.visible') after ArrowRight"
    );
}
