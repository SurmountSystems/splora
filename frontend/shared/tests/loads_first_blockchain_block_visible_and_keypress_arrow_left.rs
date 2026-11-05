//! Cypress it() title: loads first blockchain block visible and keypress arrow left.
//!
//! The spec visits `/`, waits until the skeleton is gone, and clicks
//! `bitcoin-block-offset-0-index-0`. That index is the newest confirmed
//! block. The Next Block icon is absent and the Previous Block icon is
//! visible. `cy.document().left()` sends ArrowLeft. Default `timeLtr` is
//! false, so ArrowLeft is the next-block key. The newest block hides that
//! next link, so the key opens mempool block index 0. The heading is
//! `Next Block` and the route is `/mempool-block/0`. Index 0 uses that
//! label even when the mempool block list is empty. This file does not
//! read a block hash or a fee.

#[test]
fn loads_first_blockchain_block_visible_and_keypress_arrow_left() {
    let page =
        splora_frontend_shared::loads_first_blockchain_block_visible_and_keypress_arrow_left();

    assert!(
        !page.after_selecting_newest.next_block_icon_visible,
        "cy.get('[ngbtooltip=\"Next Block\"]').should('not.exist') after the newest block click"
    );
    assert!(
        page.after_selecting_newest.previous_block_icon_visible,
        "cy.get('[ngbtooltip=\"Previous Block\"]').should('be.visible') after the newest block click"
    );
    assert_eq!(
        page.heading, "Next Block",
        "cy.get('.title-block h1').invoke('text').should('equal', 'Next Block') after ArrowLeft"
    );
    assert_eq!(
        page.route, "/mempool-block/0",
        "ArrowLeft from the newest confirmed block navigates to /mempool-block/0"
    );
}
