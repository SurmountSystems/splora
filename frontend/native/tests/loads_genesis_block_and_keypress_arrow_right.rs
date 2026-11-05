//! Cypress title: loads genesis block and keypress arrow right.
//!
//! Spec: ref/mempool/frontend/cypress/e2e/mainnet/mainnet.spec.ts lines 348-359.
//! `cy.document().right()` sends ArrowRight. Default `timeLtr` is false, so
//! ArrowRight is the previous-block key. Height 0 hides that previous link,
//! so the key does not navigate. This test calls
//! `paint_loads_genesis_block_and_keypress_arrow_right`. That paint opens
//! one headless window with `show: false` and quits before it returns.
//! Laid-out labels and the route come from
//! `splora_frontend_shared::loads_genesis_block_and_keypress_arrow_right`.
//! Each icon label is included only when the shared return says it is
//! visible. This return has no heading. This test does not invent a block
//! hash, a fee, a price, or a display amount.

const CYPRESS_IT: &str = "loads genesis block and keypress arrow right";

fn lines_from(page: &splora_frontend_shared::GenesisBlockArrowRight) -> Vec<String> {
    let icons = &page.after_arrow_right;
    let mut lines = Vec::new();
    if icons.next_block_icon_visible {
        lines.push("Next Block".to_string());
    }
    if icons.previous_block_icon_visible {
        lines.push("Previous Block".to_string());
    }
    lines.push(format!("route {}", page.route));
    lines
}

#[test]
fn loads_genesis_block_and_keypress_arrow_right() {
    let painted = splora_native::paint_loads_genesis_block_and_keypress_arrow_right();
    let shared = splora_frontend_shared::loads_genesis_block_and_keypress_arrow_right();
    assert_eq!(
        painted.shown, shared,
        "{CYPRESS_IT}: painted labels must match the shared call"
    );
    assert!(
        shared.after_arrow_right.next_block_icon_visible,
        "{CYPRESS_IT}: Next Block is visible after ArrowRight on genesis"
    );
    assert!(
        !shared.after_arrow_right.previous_block_icon_visible,
        "{CYPRESS_IT}: Previous Block is absent after ArrowRight on genesis"
    );
    assert_eq!(
        shared.route, "/block/0",
        "{CYPRESS_IT}: ArrowRight on height 0 does not navigate, so the route stays /block/0"
    );
    assert_eq!(
        shared.height, 0,
        "{CYPRESS_IT}: ArrowRight on height 0 does not navigate, so the height stays 0"
    );
    let expected = lines_from(&shared);
    let lines: Vec<&str> = painted.visible_text.lines().collect();
    let expected_refs: Vec<&str> = expected.iter().map(String::as_str).collect();
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
    assert!(
        lines.iter().any(|line| line.contains("Next Block")),
        "{CYPRESS_IT}: visible next label must be laid out, got {:?}",
        painted.visible_text
    );
    assert!(
        lines.iter().all(|line| !line.contains("Previous Block")),
        "{CYPRESS_IT}: previous label must be absent, got {:?}",
        painted.visible_text
    );
    assert!(
        lines.iter().any(|line| line.contains("/block/0")),
        "{CYPRESS_IT}: route stays /block/0, got {:?}",
        painted.visible_text
    );
    assert_eq!(
        lines.len(),
        expected.len(),
        "{CYPRESS_IT}: laid-out row count comes from the shared visible labels and route"
    );
    assert_eq!(
        lines.as_slice(),
        expected_refs.as_slice(),
        "{CYPRESS_IT}: laid-out text must match the shared visible labels and route, got {:?}",
        painted.visible_text
    );
}
