//! Visible txids after the pause icon is clicked.
//!
//! The ids captured at the click stay on screen. A later id is not appended
//! unless it was already in that list.

/// Txids that stay visible once updates are paused.
///
/// `paused_ids` is the list captured at the click. A later id stays only
/// when it is already in that list. A later id that was not already visible
/// is not appended. Every paused id is kept, in the same order, so the
/// returned length is `paused_ids.len()`.
pub fn visible_txids_after_pause<'a>(
    paused_ids: &'a [&'a str],
    later_ids: &[&str],
) -> Vec<&'a str> {
    let mut visible: Vec<&'a str> = Vec::with_capacity(paused_ids.len());
    for id in paused_ids.iter().copied() {
        visible.push(id);
    }
    for later in later_ids {
        if visible.iter().any(|id| *id == *later) {
            // Already one of the paused ids, so it remains.
            continue;
        }
        // Was not visible at the click. Do not append.
    }
    visible
}

/// Txids that stay visible once updates are paused.
///
/// `paused_ids` is the list captured at the click. Each id in `later_ids`
/// stays only when it is already in that list. A later id that was not
/// already visible is not appended.
pub fn after_pause_icon_click_updates_are_paused<'a, const N: usize, const M: usize>(
    paused_ids: [&'a str; N],
    later_ids: [&'a str; M],
) -> [&'a str; N] {
    let visible = visible_txids_after_pause(&paused_ids, &later_ids);
    if visible.len() != N {
        // A vec whose length is not N cannot be copied into [&str; N].
        // The slice filter keeps every paused id, so this return is unused.
        return paused_ids;
    }
    let mut copied = paused_ids;
    for index in 0..N {
        let shown = visible[index];
        if let Some(kept) = copied.iter().copied().find(|id| *id == shown) {
            copied[index] = kept;
        }
    }
    copied
}
