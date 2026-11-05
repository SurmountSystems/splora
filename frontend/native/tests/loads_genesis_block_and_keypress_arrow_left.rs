//! Cypress title: loads genesis block and keypress arrow left.
//!
//! Spec visits `/block/0` and sends ArrowLeft. Default `timeLtr` is false,
//! so ArrowLeft is the next-block key. The key leaves height 0 and lands on
//! height 1, route `/block/1`. After the key both the Next Block icon and
//! the Previous Block icon are visible. This test calls
//! `paint_loads_genesis_block_and_keypress_arrow_left`. That paint opens
//! one headless window with `show: false` and quits before it returns.
//! Laid-out labels and the route come from
//! `splora_frontend_shared::loads_genesis_block_and_keypress_arrow_left`.
//! Each icon label is included only when the shared return says it is
//! visible. This return has no heading. This test does not invent a block
//! hash, a fee, a price, or a display amount.

const CYPRESS_IT: &str = "loads genesis block and keypress arrow left";

fn lines_from(page: &splora_frontend_shared::GenesisBlockArrowLeft) -> Vec<String> {
    let icons = &page.after_arrow_left;
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
fn loads_genesis_block_and_keypress_arrow_left() {
    let painted = splora_native::paint_loads_genesis_block_and_keypress_arrow_left();
    let shared = splora_frontend_shared::loads_genesis_block_and_keypress_arrow_left();
    assert_eq!(
        painted.shown, shared,
        "{CYPRESS_IT}: painted labels must match the shared call"
    );
    assert!(
        shared.after_arrow_left.next_block_icon_visible,
        "{CYPRESS_IT}: Next Block is visible after ArrowLeft on genesis"
    );
    assert!(
        shared.after_arrow_left.previous_block_icon_visible,
        "{CYPRESS_IT}: Previous Block is visible after ArrowLeft on genesis"
    );
    assert_eq!(
        shared.route, "/block/1",
        "{CYPRESS_IT}: ArrowLeft on height 0 navigates to the next block, so the route is /block/1"
    );
    assert_eq!(
        shared.height, 1,
        "{CYPRESS_IT}: ArrowLeft on height 0 navigates to height 1"
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
        lines.iter().any(|line| line.contains("Previous Block")),
        "{CYPRESS_IT}: visible previous label must be laid out, got {:?}",
        painted.visible_text
    );
    assert!(
        lines.iter().any(|line| line.contains("/block/1")),
        "{CYPRESS_IT}: route is /block/1, got {:?}",
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
