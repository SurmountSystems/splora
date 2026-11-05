//! Cypress title: loads genesis block and click on the arrow left.
//!
//! Spec: ref/mempool/frontend/cypress/e2e/mainnet/mainnet.spec.ts lines 390-402.
//! The visit is `/block/0`. The Next Block icon is visible and the Previous
//! Block icon is absent. The click is that Next Block icon. Default
//! `timeLtr` is false, so the click is the newer-block step, the same
//! direction as ArrowLeft. Both icons are then visible, the height is 1,
//! and the route is `/block/1`. This test calls
//! `paint_loads_genesis_block_and_click_on_the_arrow_left`. That paint
//! opens one headless window with `show: false` and quits before it
//! returns. Assertions read the paint result. This test does not call a
//! keypress paint, does not open `/mempool-block/0`, and does not invent
//! a block hash, a fee, or a price.

#[test]
fn loads_genesis_block_and_click_on_the_arrow_left() {
    let painted = splora_native::paint_loads_genesis_block_and_click_on_the_arrow_left();
    let on_genesis = painted.on_genesis.as_str();
    let after_next = painted.after_next_block_click.as_str();

    assert!(
        on_genesis.contains("Next Block"),
        "cy.get('[ngbtooltip=\"Next Block\"]').should('be.visible') on /block/0: {on_genesis}"
    );
    assert!(
        !on_genesis.contains("Previous Block"),
        "cy.get('[ngbtooltip=\"Previous Block\"]').should('not.exist') on /block/0: {on_genesis}"
    );
    assert!(
        after_next.contains("Next Block"),
        "cy.get('[ngbtooltip=\"Next Block\"]').should('be.visible') after the Next Block click: {after_next}"
    );
    assert!(
        after_next.contains("Previous Block"),
        "cy.get('[ngbtooltip=\"Previous Block\"]').should('be.visible') after the Next Block click: {after_next}"
    );
    assert_eq!(
        painted.height, 1,
        "the Next Block click is the newer-block step, so the height is 1"
    );
    assert_eq!(
        painted.route, "/block/1",
        "the Next Block click is the newer-block step, so the route is /block/1"
    );
    assert!(
        after_next.contains("/block/1"),
        "the route laid out after the Next Block click is /block/1: {after_next}"
    );
    assert_ne!(
        painted.route, "/mempool-block/0",
        "the Next Block click on genesis does not open /mempool-block/0"
    );
    assert!(
        !after_next.contains("/mempool-block/0"),
        "the laid-out route after the Next Block click is not /mempool-block/0: {after_next}"
    );
    let genesis_rows = painted.on_genesis.lines().count();
    let after_rows = painted.after_next_block_click.lines().count();
    assert_eq!(
        painted.row_heights.len(),
        genesis_rows + after_rows,
        "each laid-out line needs a row height, genesis {on_genesis:?} after {after_next:?} heights {:?}",
        painted.row_heights
    );
    assert!(
        !painted.row_heights.is_empty() && painted.row_heights.iter().all(|height| *height > 0.0),
        "row heights come from laid-out bounds and must be positive, heights {:?}",
        painted.row_heights
    );
}
