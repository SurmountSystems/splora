//! Cypress it() title: loads genesis block and click on the arrow left.
//!
//! The spec visits `/block/0`. The skeleton wait is not an assertion.
//! The Next Block icon is visible and the Previous Block icon is absent.
//! The click is the Next Block icon. Default `timeLtr` is false, so that
//! click is the newer-block step, the same direction as ArrowLeft. One
//! newer step from genesis is height 1 and route `/block/1`, and both
//! icons are then visible. This click does not open `/mempool-block/0`.
//! This file does not sleep and does not read a block hash, a fee, or a
//! price.

#[test]
fn loads_genesis_block_and_click_on_the_arrow_left() {
    let page = splora_frontend_shared::loads_genesis_block_and_click_on_the_arrow_left();

    assert!(
        page.on_genesis.next_block_icon_visible,
        "cy.get('[ngbtooltip=\"Next Block\"]').should('be.visible') on /block/0"
    );
    assert!(
        !page.on_genesis.previous_block_icon_visible,
        "cy.get('[ngbtooltip=\"Previous Block\"]').should('not.exist') on /block/0"
    );
    assert!(
        page.after_next_block_click.next_block_icon_visible,
        "cy.get('[ngbtooltip=\"Next Block\"]').should('be.visible') after the Next Block click"
    );
    assert!(
        page.after_next_block_click.previous_block_icon_visible,
        "cy.get('[ngbtooltip=\"Previous Block\"]').should('be.visible') after the Next Block click"
    );
    assert_eq!(
        page.height, 1,
        "the Next Block click is the newer-block step, so the height is 1"
    );
    assert_eq!(
        page.route, "/block/1",
        "the Next Block click is the newer-block step, so the route is /block/1"
    );
    assert_ne!(
        page.route, "/mempool-block/0",
        "the Next Block click on genesis does not open /mempool-block/0"
    );
}
