//! Cypress title: loads first blockchain block visible and keypress arrow right.
//!
//! Spec: ref/mempool/frontend/cypress/e2e/mainnet/mainnet.spec.ts line 287.
//! `cy.document().right()` sends ArrowRight (commands.ts). This test calls
//! `paint_loads_first_blockchain_block_visible_and_keypress_arrow_right`.
//! That paint opens one headless window with `show: false` and quits before
//! it returns. Laid-out labels come from
//! `splora_frontend_shared::loads_first_blockchain_block_visible_and_keypress_arrow_right`.
//! This test does not invent a block hash, a fee, a price, or a socket body.

const CYPRESS_IT: &str = "loads first blockchain block visible and keypress arrow right";

fn icon_line(phase: &str, icon: &str, visible: bool) -> String {
    format!("{phase} {icon} {visible}")
}

fn lines_from(icons: &splora_frontend_shared::FirstBlockchainBlockArrowRight) -> Vec<String> {
    let newest = &icons.after_selecting_newest;
    let older = &icons.after_arrow_right;
    vec![
        icon_line(
            "newest confirmed",
            "Next Block",
            newest.next_block_icon_visible,
        ),
        icon_line(
            "newest confirmed",
            "Previous Block",
            newest.previous_block_icon_visible,
        ),
        icon_line(
            "after ArrowRight",
            "Next Block",
            older.next_block_icon_visible,
        ),
        icon_line(
            "after ArrowRight",
            "Previous Block",
            older.previous_block_icon_visible,
        ),
    ]
}

#[test]
fn loads_first_blockchain_block_visible_and_keypress_arrow_right() {
    let painted =
        splora_native::paint_loads_first_blockchain_block_visible_and_keypress_arrow_right();
    let shared =
        splora_frontend_shared::loads_first_blockchain_block_visible_and_keypress_arrow_right();
    assert_eq!(
        painted.shown, shared,
        "{CYPRESS_IT}: painted icons must match the shared call"
    );
    assert!(
        shared.after_selecting_newest.previous_block_icon_visible,
        "{CYPRESS_IT}: Previous Block is visible after the newest confirmed block is selected"
    );
    assert!(
        !shared.after_selecting_newest.next_block_icon_visible,
        "{CYPRESS_IT}: Next Block is absent after the newest confirmed block is selected"
    );
    assert!(
        shared.after_arrow_right.next_block_icon_visible
            && shared.after_arrow_right.previous_block_icon_visible,
        "{CYPRESS_IT}: Next Block and Previous Block are both visible after one ArrowRight"
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
    assert_eq!(
        lines.len(),
        expected.len(),
        "{CYPRESS_IT}: laid-out row count comes from the shared icon pair"
    );
    assert_eq!(
        lines.as_slice(),
        expected_refs.as_slice(),
        "{CYPRESS_IT}: laid-out text must match the shared icon labels, got {:?}",
        painted.visible_text
    );
}
