//! Native string render after the pause icon is clicked.
//! Visible txids come from the shared pause function. This file does not open a window.

/// Renders the txids that stay visible after the pause icon click, plus a pause control.
///
/// `paused_ids` is the list captured at the click. `later_ids` is the later batch.
/// The shared function chooses which ids stay on screen.
pub fn paint_pauses_updates_when_clicking_the_pause_icon<'a, const N: usize, const M: usize>(
    paused_ids: [&'a str; N],
    later_ids: [&'a str; M],
) -> String {
    let visible =
        splora_frontend_shared::after_pause_icon_click_updates_are_paused(paused_ids, later_ids);
    let mut painted = String::from("data-cy=\"btn-pause\"");
    for id in visible {
        painted.push('\n');
        painted.push_str(id);
    }
    painted
}
