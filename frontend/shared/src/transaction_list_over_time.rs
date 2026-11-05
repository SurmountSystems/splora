//! Visible txids on the recent transaction list as later batches arrive.
//!
//! The first list is the ids from the first batch. The later list puts the
//! second batch ahead of the first, newest first. Anchor text is the txid
//! argument itself, so the leading characters of a new txid are in that
//! text. This module does not read a GET body.

/// Txids visible after the first batch, then after the later batch.
///
/// `first` and `second` are the two batches the list receives. Neither
/// list is a fixed prefix. Each id is copied from those arguments.
pub fn updates_the_transaction_list_over_time<'a>(
    first: &[&'a str; 6],
    second: &[&'a str; 6],
) -> (Vec<&'a str>, Vec<&'a str>) {
    let after_first: Vec<&'a str> = first.iter().copied().collect();
    let mut after_second = Vec::with_capacity(first.len() + second.len());
    after_second.extend(second.iter().copied());
    after_second.extend(first.iter().copied());
    (after_first, after_second)
}
