//! Newest-first caps for the recent transaction list.
//!
//! Batches are taken from the end of the batch list, so a later batch
//! comes first. Index 0 inside a batch stays ahead of later indexes in
//! that batch. The default list shows 50 ids. The limit-10 control shows
//! the first 10 of that same order. Visible ids come only from `batches`.
//! This module does not read a GET body.

/// Rows shown before a limit control is clicked.
const DEFAULT_LIST_CAP: usize = 50;
/// Rows shown after the limit-10 control.
const LIMIT_10_CAP: usize = 10;

/// Newest-first ids at the default list cap, then at the limit-10 control.
///
/// The first list is the ordered ids sliced to 50. The second list is that
/// same ordered id list sliced to 10. Neither list is a length by itself.
pub fn caps_the_list_when_changing_the_limit_to_10<'a>(
    batches: &[[&'a str; 6]; 9],
) -> (Vec<&'a str>, Vec<&'a str>) {
    let mut newest_first = Vec::with_capacity(batches.len() * 6);
    for batch in batches.iter().rev() {
        for txid in batch {
            newest_first.push(*txid);
        }
    }
    let rows_at_limit_50 = newest_first
        .iter()
        .copied()
        .take(DEFAULT_LIST_CAP)
        .collect();
    let rows_at_limit_10 = newest_first.iter().copied().take(LIMIT_10_CAP).collect();
    (rows_at_limit_50, rows_at_limit_10)
}
