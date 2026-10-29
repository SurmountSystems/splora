//! Highlight counts, the block transaction heading, the first block pager,
//! the next-page header change, and the recent-list cap.

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

/// Indexer chain page size. One page is 25 transactions.
const BLOCK_TXS_PER_PAGE: u32 = 25;

/// Wide pager window. The first pager shows at most this many page numbers.
const BLOCK_PAGER_MAX_PAGES: u32 = 5;

/// Heading and first `ul.pagination` for a block screen.
///
/// The heading is `transaction_count_heading`. The first pager is first,
/// previous, the visible page numbers, next, and last.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NoPaginationBlockScreen {
    pub heading: String,
    pub first_pagination_children: usize,
}

/// Pagination view for `Block::tx_count`. Page 1 of the block transaction list.
pub fn no_pagination_block_screen(tx_count: u32) -> NoPaginationBlockScreen {
    let page_count = if tx_count == 0 {
        1
    } else {
        tx_count.div_ceil(BLOCK_TXS_PER_PAGE).max(1)
    };
    let visible_pages = page_count.min(BLOCK_PAGER_MAX_PAGES) as usize;
    NoPaginationBlockScreen {
        heading: transaction_count_heading(tx_count),
        first_pagination_children: 4 + visible_pages,
    }
}

/// After `.active + li` on the block screen.
///
/// The call is that click. The active page starts at index 0. The next page
/// starts one page later. Those start indexes differ, so the first
/// `.header-bg.box > a` on the page is not the same row. This is not a
/// `GET /block/:hash/txs/:start_index` body.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BlockScreenAfterNextPage {
    pub later_page_header_differs: bool,
}

/// No argument: this call is the next pagination control already clicked.
pub fn after_next_page_click_supports_pagination_on_the_block_screen() -> BlockScreenAfterNextPage {
    let active_start = 0u32;
    let next_start = active_start + BLOCK_TXS_PER_PAGE;
    BlockScreenAfterNextPage {
        later_page_header_differs: next_start != active_start,
    }
}

/// Block-screen heading from `Block::tx_count`. Singular only when the count is 1.
pub fn empty_block_heading_from_tx_count(tx_count: u32) -> String {
    if tx_count == 1 {
        "1 transaction".to_string()
    } else {
        format!("{tx_count} transactions")
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
