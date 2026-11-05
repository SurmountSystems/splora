//! Cypress it() title: loads first blockchain block visible and keypress arrow left.
//!
//! Spec: ref/mempool/frontend/cypress/e2e/mainnet/mainnet.spec.ts lines 301-312.
//! `left` sends ArrowLeft. Default `timeLtr` is false, so ArrowLeft is the
//! next-block key. After the newest confirmed block is selected, the Next
//! Block icon is absent and the Previous Block icon is visible. After
//! ArrowLeft, `.title-block h1` is `Next Block` and the route is
//! `/mempool-block/0`. The markup comes from the web paint.

#[test]
fn loads_first_blockchain_block_visible_and_keypress_arrow_left() {
    let painted = splora_web::paint_loads_first_blockchain_block_visible_and_keypress_arrow_left();
    let (after_selecting_newest, after_arrow_left) = painted;

    assert!(
        after_selecting_newest.contains("Previous Block"),
        "cy.get('[ngbtooltip=\"Previous Block\"]').should('be.visible') after the newest block click: {after_selecting_newest}"
    );
    assert!(
        !after_selecting_newest.contains("Next Block"),
        "cy.get('[ngbtooltip=\"Next Block\"]').should('not.exist') after the newest block click: {after_selecting_newest}"
    );
    assert!(
        after_arrow_left.contains("<div class=\"title-block\"><h1>Next Block</h1></div>"),
        "cy.get('.title-block h1').invoke('text').should('equal', 'Next Block') after ArrowLeft: {after_arrow_left}"
    );
    assert!(
        after_arrow_left.contains("/mempool-block/0"),
        "ArrowLeft from the newest confirmed block navigates to /mempool-block/0: {after_arrow_left}"
    );
}
