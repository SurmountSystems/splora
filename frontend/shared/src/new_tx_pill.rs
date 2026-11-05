//! New-transaction pill on the recent transactions list.
//!
//! A non-widget list auto-pauses when `scrollTop` is greater than 0.
//! `bufferedCount` is how many deduped ids were inserted after that pause
//! froze the live length. The live map keeps at most 1000 ids. The pill
//! is hidden for the widget and while the manual pause control is on.

/// Live map cap. Older ids drop once the map is past this size.
const LIVE_CAP: usize = 1000;

/// Pill visibility, its label, and the buffered count behind that label.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NewTxPill {
    pub visible: bool,
    pub label: String,
    pub buffered_count: usize,
}

/// Pill after the ids present at the scroll sample, then the ids that follow.
///
/// `widget` skips the scroll pause. `manual_paused` pauses the list and
/// hides the pill. `scroll_top` greater than 0 auto-pauses a non-widget
/// list. `txids_before` is the list frozen at that pause. `txids_after`
/// is the later batch. Ids already in the live map do not raise the count.
pub fn shows_the_new_transaction_pill_when_there_are_new_transactions(
    widget: bool,
    manual_paused: bool,
    scroll_top: i64,
    txids_before: &[&str],
    txids_after: &[&str],
) -> NewTxPill {
    let mut live = Vec::new();
    insert_batch(&mut live, txids_before);
    let paused = manual_paused || (!widget && scroll_top > 0);
    let mut frozen = None;
    step_count(&mut frozen, live.len(), paused);
    insert_batch(&mut live, txids_after);
    let buffered_count = step_count(&mut frozen, live.len(), paused);
    let visible = !widget && buffered_count > 0 && !manual_paused;
    NewTxPill {
        visible,
        buffered_count,
        label: pill_label(buffered_count),
    }
}

fn step_count(frozen: &mut Option<usize>, live_len: usize, paused: bool) -> usize {
    if !paused {
        *frozen = None;
        return 0;
    }
    let frozen_len = frozen.unwrap_or(live_len);
    *frozen = Some(frozen_len);
    live_len.saturating_sub(frozen_len)
}

fn pill_label(count: usize) -> String {
    match count {
        0 => String::new(),
        1 => "1 new transaction".to_string(),
        n => format!("{n} new transactions"),
    }
}

fn insert_batch<'a>(live: &mut Vec<&'a str>, batch: &[&'a str]) {
    for id in batch.iter().rev() {
        if live.iter().any(|seen| seen == id) {
            continue;
        }
        live.push(*id);
        while live.len() > LIVE_CAP {
            live.remove(0);
        }
    }
}
