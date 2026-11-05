//! Cypress it() title: loads genesis block and keypress arrow right.
//!
//! The spec visits `/block/0`. `cy.document().right()` sends ArrowRight.
//! Default `timeLtr` is false, so ArrowRight is the previous-block key.
//! Height 0 hides that previous link, so the key does not navigate. The
//! page stays on height 0 and route `/block/0`. The Next Block icon is
//! visible and the Previous Block icon is absent. This file does not
//! sleep, scroll, or read a block hash, a fee, or a price.

#[test]
fn loads_genesis_block_and_keypress_arrow_right() {
    let page = splora_frontend_shared::loads_genesis_block_and_keypress_arrow_right();

    assert_eq!(
        page.height, 0,
        "ArrowRight on height 0 does not navigate, so the height stays 0"
    );
    assert_eq!(
        page.route, "/block/0",
        "ArrowRight on height 0 does not navigate, so the route stays /block/0"
    );
    assert!(
        page.after_arrow_right.next_block_icon_visible,
        "cy.get('[ngbtooltip=\"Next Block\"]').should('be.visible') after ArrowRight"
    );
    assert!(
        !page.after_arrow_right.previous_block_icon_visible,
        "cy.get('[ngbtooltip=\"Previous Block\"]').should('not.exist') after ArrowRight"
    );
}
