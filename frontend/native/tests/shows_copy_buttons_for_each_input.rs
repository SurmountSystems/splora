//! Cypress title: shows copy buttons for each input.
//!
//! Spec: ref/mempool/frontend/cypress/e2e/mainnet/calculator.spec.ts line 52.
//! This test calls `paint_shows_copy_buttons_for_each_input` and reads the
//! laid-out lines from that one headless window. It does not open a window
//! of its own. Labels come from
//! `splora_frontend_shared::shows_copy_buttons_for_each_input`. The clipboard
//! count is that return value's length, not a literal.

const CYPRESS_IT: &str = "shows copy buttons for each input";

#[test]
fn shows_copy_buttons_for_each_input() {
    let painted = splora_native::paint_shows_copy_buttons_for_each_input();
    let shared = splora_frontend_shared::shows_copy_buttons_for_each_input();
    assert_eq!(
        painted.shown, shared,
        "{CYPRESS_IT}: painted clipboard buttons must match the shared call"
    );
    assert_eq!(
        shared.buttons.len(),
        shared.inputs.len(),
        "{CYPRESS_IT}: one app-clipboard for each calculator input"
    );
    let mut expected = Vec::with_capacity(shared.buttons.len());
    for (input, button) in shared.inputs.iter().zip(shared.buttons.iter()) {
        assert_eq!(
            button.field, *input,
            "{CYPRESS_IT}: each app-clipboard copies that input"
        );
        assert!(
            button.visible,
            "{CYPRESS_IT}: each app-clipboard is visible"
        );
        expected.push(format!("{input} {} {}", button.field, button.visible));
    }
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
        shared.buttons.len(),
        "{CYPRESS_IT}: laid-out row count comes from the shared clipboard buttons"
    );
    assert_eq!(
        lines.as_slice(),
        expected_refs.as_slice(),
        "{CYPRESS_IT}: laid-out text must match the shared clipboard buttons, got {:?}",
        painted.visible_text
    );
}
