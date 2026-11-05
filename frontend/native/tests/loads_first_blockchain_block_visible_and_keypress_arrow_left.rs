//! Cypress title: loads first blockchain block visible and keypress arrow left.
//!
//! Spec: ref/mempool/frontend/cypress/e2e/mainnet/mainnet.spec.ts lines 301-312.
//! `cy.document().left()` sends ArrowLeft (commands.ts). Default `timeLtr`
//! is false, so ArrowLeft is the next-block key. This test calls
//! `paint_loads_first_blockchain_block_visible_and_keypress_arrow_left`.
//! That paint opens one headless window with `show: false` and quits before
//! it returns. Laid-out labels, the heading, and the route come from
//! `splora_frontend_shared::loads_first_blockchain_block_visible_and_keypress_arrow_left`.
//! This test does not invent a block hash, a fee, a price, or a socket body.

const CYPRESS_IT: &str = "loads first blockchain block visible and keypress arrow left";

fn icon_line(phase: &str, icon: &str, visible: bool) -> String {
    format!("{phase} {icon} {visible}")
}

fn lines_from(page: &splora_frontend_shared::FirstBlockchainBlockArrowLeft) -> Vec<String> {
    let newest = &page.after_selecting_newest;
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
        format!("after ArrowLeft heading {}", page.heading),
        format!("after ArrowLeft route {}", page.route),
    ]
}

#[test]
fn loads_first_blockchain_block_visible_and_keypress_arrow_left() {
    let painted =
        splora_native::paint_loads_first_blockchain_block_visible_and_keypress_arrow_left();
    let shared =
        splora_frontend_shared::loads_first_blockchain_block_visible_and_keypress_arrow_left();
    assert_eq!(
        painted.shown, shared,
        "{CYPRESS_IT}: painted labels must match the shared call"
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
        "{CYPRESS_IT}: laid-out row count comes from the shared labels, heading, and route"
    );
    assert_eq!(
        lines.as_slice(),
        expected_refs.as_slice(),
        "{CYPRESS_IT}: laid-out text must match the shared labels, heading, and route, got {:?}",
        painted.visible_text
    );
}
