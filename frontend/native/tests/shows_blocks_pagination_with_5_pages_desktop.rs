//! Cypress `shows blocks pagination with 5 pages (desktop)`.
//! `ref/mempool/frontend/cypress/e2e/mainnet/mainnet.spec.ts`.
//! Viewport 760 by 800, and the next it() at viewport 669 by 800.
//! The child count comes from
//! `splora_frontend_shared::shows_blocks_pagination_with_5_pages_desktop`.
//! This test does not hard-code that count. It paints through the native
//! block page and reads laid-out children. The page closes its window
//! before it returns.

fn assert_block_page_matches_shared(viewport_width: u32, tx_count: u32) {
    let expected = splora_frontend_shared::shows_blocks_pagination_with_5_pages_desktop(
        viewport_width,
        tx_count,
    );
    let painted =
        splora_native::paint_shows_blocks_pagination_with_5_pages_desktop(viewport_width, tx_count);
    assert_eq!(
        painted.child_heights.len(),
        painted.child_widths.len(),
        "each block-page pagination child needs a laid-out width and height"
    );
    assert!(
        painted.child_heights.iter().all(|height| *height > 0.0)
            && painted.child_widths.iter().all(|width| *width > 0.0),
        "block page pagination children were not laid out, heights {:?} widths {:?}",
        painted.child_heights,
        painted.child_widths
    );
    assert_eq!(
        painted.child_heights.len(),
        expected.first_pagination_children,
        "block page child count must follow shows_blocks_pagination_with_5_pages_desktop at {viewport_width}px with {tx_count} txs, heights {:?}",
        painted.child_heights
    );
}

#[test]
fn shows_blocks_pagination_with_5_pages_desktop() {
    // Wide viewport with at least 125 transactions, then width 669.
    assert_block_page_matches_shared(760, 125);
    assert_block_page_matches_shared(669, 125);
    let wide = splora_native::paint_shows_blocks_pagination_with_5_pages_desktop(760, 125);
    let narrow = splora_native::paint_shows_blocks_pagination_with_5_pages_desktop(669, 125);
    assert_ne!(
        wide.child_heights.len(),
        narrow.child_heights.len(),
        "760px and 669px must not paint the same number of pagination children"
    );

    assert_block_page_matches_shared(670, 125);
    assert_block_page_matches_shared(671, 125);
    assert_block_page_matches_shared(760, 126);
    assert_block_page_matches_shared(669, 126);
    assert_block_page_matches_shared(760, 100);
    assert_block_page_matches_shared(760, 18);
    assert_block_page_matches_shared(669, 50);
}
