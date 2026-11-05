//! Cypress title: loads the blocks page.
//!
//! Spec: ref/mempool/frontend/cypress/e2e/liquid/liquid.spec.ts line 38.
//! The it() visits the dashboard, clicks `#btn-blocks`, and waits until
//! `.skeleton-loader` length is 0. This test calls
//! `paint_loads_the_blocks_page` and reads the laid-out lines from that
//! one headless window. It does not open a window of its own. The visible
//! lines come from `splora_frontend_shared::loads_the_blocks_page`.

const CYPRESS_IT: &str = "loads the blocks page";

#[test]
fn loads_the_blocks_page() {
    let painted = splora_native::paint_loads_the_blocks_page();
    let shared = splora_frontend_shared::loads_the_blocks_page();
    let count = shared.skeleton_loader_count.to_string();
    let expected = [shared.button_id, shared.route.as_str(), count.as_str()];
    let lines: Vec<&str> = painted.visible_text.lines().collect();
    assert_eq!(
        painted.row_heights.len(),
        lines.len(),
        "{CYPRESS_IT}: each laid-out line needs a row height, lines {lines:?} heights {:?}",
        painted.row_heights
    );
    assert!(
        !painted.row_heights.is_empty() && painted.row_heights.iter().all(|height| *height > 0.0),
        "{CYPRESS_IT}: rows were not laid out, heights {:?}",
        painted.row_heights
    );
    assert_eq!(
        lines.as_slice(),
        expected.as_slice(),
        "{CYPRESS_IT}: laid-out text must match the shared blocks page, got {:?}",
        painted.visible_text
    );
}
