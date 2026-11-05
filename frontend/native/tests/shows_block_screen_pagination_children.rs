//! Block screen first-pager children.
//!
//! Cypress visits at viewport 760 by 800, then 669 by 800, with at least
//! 125 transactions. The laid-out child count is
//! `splora_frontend_shared::shows_blocks_pagination_with_5_pages_desktop`.
//! This test constructs `BlockScreen` and counts the children that screen
//! laid out. It does not call `paint_shows_blocks_pagination_with_5_pages_desktop`
//! itself, and it does not hard-code the child count.

use splora_native::BlockScreen;

fn assert_block_screen_matches_shared(
    screen: &BlockScreen,
    viewport_width: u32,
    tx_count: u32,
) -> usize {
    let expected = splora_frontend_shared::shows_blocks_pagination_with_5_pages_desktop(
        viewport_width,
        tx_count,
    );
    let (widths, heights) = screen.laid_out_pagination_children(viewport_width, tx_count);
    assert_eq!(
        widths.len(),
        heights.len(),
        "each block-screen pagination child needs a laid-out width and height"
    );
    assert!(
        heights.iter().all(|height| *height > 0.0) && widths.iter().all(|width| *width > 0.0),
        "block screen pagination children were not laid out, heights {heights:?} widths {widths:?}"
    );
    assert_eq!(
        heights.len(),
        expected.first_pagination_children,
        "block screen child count must follow shows_blocks_pagination_with_5_pages_desktop at {viewport_width}px with {tx_count} txs, heights {heights:?}"
    );
    heights.len()
}

#[test]
fn shows_block_screen_lays_out_one_child_per_first_pagination_children() {
    let screen = BlockScreen::new();
    let wide = assert_block_screen_matches_shared(&screen, 760, 125);
    let narrow = assert_block_screen_matches_shared(&screen, 669, 125);
    assert_ne!(
        wide, narrow,
        "760px and 669px must not lay out the same number of block-screen pagination children"
    );
}
