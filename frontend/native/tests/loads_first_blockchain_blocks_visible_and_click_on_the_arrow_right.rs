//! Cypress title: loads first blockchain blocks visible and click on the arrow right.
//!
//! Spec: ref/mempool/frontend/cypress/e2e/mainnet/mainnet.spec.ts lines 375-388.
//! The newest confirmed block is clicked. The Next Block icon is absent and
//! the Previous Block icon is visible. The second click is that Previous
//! Block icon. Both icons are then visible, the height is 1, and the route
//! is `/block/1`. This test calls
//! `paint_loads_first_blockchain_blocks_visible_and_click_on_the_arrow_right`.
//! That paint opens one headless window with `show: false` and quits before
//! it returns. Assertions read the paint result. This test does not
//! reimplement the link rule and does not invent a block hash.

#[test]
fn loads_first_blockchain_blocks_visible_and_click_on_the_arrow_right() {
    let painted =
        splora_native::paint_loads_first_blockchain_blocks_visible_and_click_on_the_arrow_right();
    let after_selecting_newest = painted.after_selecting_newest.as_str();
    let after_previous = painted.after_previous_block_click.as_str();

    assert!(
        !after_selecting_newest.contains("Next Block"),
        "cy.get('[ngbtooltip=\"Next Block\"]').should('not.exist') after the newest block click: {after_selecting_newest}"
    );
    assert!(
        after_selecting_newest.contains("Previous Block"),
        "cy.get('[ngbtooltip=\"Previous Block\"]').should('be.visible') after the newest block click: {after_selecting_newest}"
    );
    assert!(
        after_previous.contains("Next Block"),
        "cy.get('[ngbtooltip=\"Next Block\"]').should('be.visible') after the Previous Block click: {after_previous}"
    );
    assert!(
        after_previous.contains("Previous Block"),
        "cy.get('[ngbtooltip=\"Previous Block\"]').should('be.visible') after the Previous Block click: {after_previous}"
    );
    assert_eq!(
        painted.height, 1,
        "the Previous Block click is the older-block step, so the height is 1"
    );
    assert_eq!(
        painted.route, "/block/1",
        "the Previous Block click is the older-block step, so the route is /block/1"
    );
    assert!(
        after_previous.contains("/block/1"),
        "the route laid out after the Previous Block click is /block/1: {after_previous}"
    );
    let newest_rows = painted.after_selecting_newest.lines().count();
    let previous_rows = painted.after_previous_block_click.lines().count();
    assert_eq!(
        painted.row_heights.len(),
        newest_rows + previous_rows,
        "each laid-out line needs a row height, newest {after_selecting_newest:?} previous {after_previous:?} heights {:?}",
        painted.row_heights
    );
    assert!(
        !painted.row_heights.is_empty() && painted.row_heights.iter().all(|height| *height > 0.0),
        "rows were not laid out, heights {:?}",
        painted.row_heights
    );
}
