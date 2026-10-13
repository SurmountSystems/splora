//! Highlight counts, the block transaction heading, and the recent-list cap.

use crate::wire::{RecentTransaction, Transaction, TxOutput};

/// Matching inputs and outputs on one transaction.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HighlightCount {
    pub inputs: usize,
    pub outputs: usize,
}

/// One count per transaction, same order as `txs`.
///
/// Desktop `.highlight` (not the mobile-only table) matches when the script
/// type is not `p2pk` and `scriptpubkey_address` equals `address`, or when
/// `scriptpubkey.slice(2, -2)` equals `address`. An empty address matches
/// nothing. An input matches only through its prevout.
pub fn highlight_counts(address: &str, txs: &[Transaction]) -> Vec<HighlightCount> {
    txs.iter()
        .map(|tx| HighlightCount {
            inputs: tx
                .vin
                .iter()
                .filter(|input| {
                    input
                        .prevout
                        .as_ref()
                        .is_some_and(|prevout| output_matches(address, prevout))
                })
                .count(),
            outputs: tx
                .vout
                .iter()
                .filter(|output| output_matches(address, output))
                .count(),
        })
        .collect()
}

/// Singular only when `count == 1`. Zero is "0 transactions".
pub fn transaction_count_heading(count: u32) -> String {
    if count == 1 {
        "1 transaction".to_string()
    } else {
        format!("{count} transactions")
    }
}

/// The first 10 rows, or the whole slice when it is shorter. Does not copy.
pub fn cap_recent_transactions(rows: &[RecentTransaction]) -> &[RecentTransaction] {
    &rows[..10.min(rows.len())]
}

fn output_matches(address: &str, output: &TxOutput) -> bool {
    if address.is_empty() {
        return false;
    }
    let by_address = output.script_pubkey_type != "p2pk"
        && output.script_pubkey_address.as_deref() == Some(address);
    by_address || script_middle(&output.script_pubkey) == address
}

fn script_middle(script: &str) -> &str {
    let len = script.len();
    if len < 5 { "" } else { &script[2..len - 2] }
}
