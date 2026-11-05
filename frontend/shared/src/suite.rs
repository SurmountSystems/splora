//! Indexer-backed explorer cases both apps can call.
//!
//! Bodies are the JSON or plain text the indexer already returns. Paths stay
//! free of `?`, `#`, spaces, and `%`. This module does not fetch and it does
//! not name a widget.

use crate::block_prev_next_links::block_prev_next_links;
use crate::details_open_after_click::details_open_after_click;
use crate::mempool_blocks::{
    MempoolBacklog, first_mempool_block_from_backlog, load_first_mempool_block_after_skeleton_loads,
};
use crate::parse::{ParseError, parse_address_prefix, parse_block_height};
use crate::paths::{
    address_path, address_prefix_path, block_path, block_txs_path, block_txs_start_index_path,
};
use crate::pause_icon_updates::visible_txids_after_pause;
use crate::screen_text::{HighlightCount, highlight_counts, transaction_count_heading};
use crate::wire::{Block, MempoolSummary, RecentTransaction, Transaction};

/// Default `rest_default_chain_txs_per_page` in `src/config.rs`.
pub const BLOCK_TX_PAGE_LEN: u64 = 25;

/// One case name both apps dispatch on.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SharedCase {
    pub name: &'static str,
}

const CASES: &[SharedCase] = &[
    SharedCase {
        name: "home_projected_mempool_block",
    },
    SharedCase {
        name: "home_recent_confirmed_blocks",
    },
    SharedCase {
        name: "search_partial_legacy_address",
    },
    SharedCase {
        name: "search_narrows",
    },
    SharedCase {
        name: "search_bech32_case_insensitive",
    },
    SharedCase {
        name: "search_bech32m_case_insensitive",
    },
    SharedCase {
        name: "address_txs_newest_first",
    },
    SharedCase {
        name: "address_highlight_inputs",
    },
    SharedCase {
        name: "address_highlight_outputs",
    },
    SharedCase {
        name: "address_highlight_both",
    },
    SharedCase {
        name: "address_highlight_several_inputs",
    },
    SharedCase {
        name: "address_highlight_several_outputs",
    },
    SharedCase {
        name: "transaction_inputs_outputs",
    },
    SharedCase {
        name: "transaction_confirmed",
    },
    SharedCase {
        name: "transaction_unconfirmed",
    },
    SharedCase {
        name: "block_by_hash",
    },
    SharedCase {
        name: "block_by_height",
    },
    SharedCase {
        name: "block_height_zero",
    },
    SharedCase {
        name: "block_coinbase_only",
    },
    SharedCase {
        name: "block_next_page",
    },
    SharedCase {
        name: "block_adjacent",
    },
    SharedCase {
        name: "block_genesis_no_previous",
    },
    SharedCase {
        name: "block_tip_no_next",
    },
    SharedCase {
        name: "block_details_toggle",
    },
    SharedCase {
        name: "recent_arrival",
    },
    SharedCase {
        name: "recent_pause",
    },
    SharedCase { name: "recent_cap" },
    SharedCase {
        name: "status_unconfirmed_count",
    },
    SharedCase {
        name: "status_mempool_vsize",
    },
];

pub fn shared_cases() -> &'static [SharedCase] {
    CASES
}

/// `frontend/shared/Cargo.toml` has no features. Package `splora` names `liquid`.
pub fn liquid_rows_painted() -> bool {
    false
}

/// True when a path has no query, fragment, space, or percent-escape.
pub fn indexer_path_is_plain(path: &str) -> bool {
    !path.chars().any(|ch| matches!(ch, '?' | '#' | ' ' | '%'))
}

/// Confirmed block row that opens a block view.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConfirmedBlockLink {
    pub id: String,
    pub height: u32,
    pub open_path: String,
}

/// Home from `GET /mempool` and `GET /blocks`.
///
/// The projected link is the backlog receipt. `GET /mempool` has no `nTx`
/// and no median fee. `vsize` on that body is virtual bytes, not a byte size.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HomeView {
    pub projected_link_id: String,
    pub projected_open_path: String,
    pub projected_navigable: bool,
    pub confirmed: Vec<ConfirmedBlockLink>,
}

pub fn home_view(mempool_json: &str, blocks_json: &str) -> Result<HomeView, ParseError> {
    let summary: MempoolSummary = serde_json::from_str(mempool_json)?;
    let backlog = MempoolBacklog {
        count: summary.count,
        vsize: summary.vsize,
        total_fee: summary.total_fee,
        fee_histogram_len: summary.fee_histogram.len(),
    };
    let projected = first_mempool_block_from_backlog(backlog);
    let opened = load_first_mempool_block_after_skeleton_loads();
    let blocks: Vec<Block> = serde_json::from_str(blocks_json)?;
    let confirmed = blocks
        .iter()
        .map(|block| ConfirmedBlockLink {
            id: block.id.clone(),
            height: block.height,
            open_path: block_path(&block.id),
        })
        .collect();
    Ok(HomeView {
        projected_link_id: projected.link_id,
        projected_open_path: opened.route,
        projected_navigable: projected.block_link_exists && projected.skeleton_gone,
        confirmed,
    })
}

/// `GET /address-prefix/:prefix` for one typed string.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PrefixLookup {
    pub path: String,
    pub folded: bool,
}

pub fn prefix_lookup(typed: &str) -> Result<PrefixLookup, ParseError> {
    let len = typed.chars().count();
    if !(1..=64).contains(&len) {
        return Err(ParseError::new("address prefix length is out of range"));
    }
    if typed
        .chars()
        .any(|ch| matches!(ch, '/' | '?' | '#' | ' ' | '%') || ch.is_control())
    {
        return Err(ParseError::new("address prefix is not a plain path"));
    }
    let lowered = typed.to_ascii_lowercase();
    let folded = lowered.starts_with("bc1");
    let segment = if folded { lowered.as_str() } else { typed };
    let path = address_prefix_path(segment);
    if !indexer_path_is_plain(&path) {
        return Err(ParseError::new("address prefix path is not plain"));
    }
    Ok(PrefixLookup { path, folded })
}

/// Addresses from a prefix body that still match `typed`.
///
/// Order is the served array order. A folded `bc1` prefix compares in
/// lowercase. Any other prefix compares the typed characters.
pub fn prefix_hits(typed: &str, body: &str) -> Result<Vec<String>, ParseError> {
    let lookup = prefix_lookup(typed)?;
    let addresses = parse_address_prefix(body)?;
    let needle = if lookup.folded {
        typed.to_ascii_lowercase()
    } else {
        typed.to_string()
    };
    Ok(addresses
        .into_iter()
        .filter(|address| {
            if lookup.folded {
                address.to_ascii_lowercase().starts_with(&needle)
            } else {
                address.starts_with(typed)
            }
        })
        .collect())
}

pub fn address_view_path(address: &str) -> String {
    address_path(address)
}

/// Address transactions in served order, with highlight counts.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AddressTxList {
    pub txids: Vec<String>,
    pub marks: Vec<HighlightCount>,
}

pub fn address_tx_list(address: &str, txs_json: &str) -> Result<AddressTxList, ParseError> {
    let txs: Vec<Transaction> = serde_json::from_str(txs_json)?;
    let marks = highlight_counts(address, &txs);
    let txids = txs.into_iter().map(|tx| tx.txid).collect();
    Ok(AddressTxList { txids, marks })
}

/// One transaction from a block list or from the mempool.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TransactionView {
    pub txid: String,
    pub input_count: usize,
    pub output_count: usize,
    pub confirmed: bool,
    pub block_hash: Option<String>,
}

pub fn transaction_view(tx_json: &str) -> Result<TransactionView, ParseError> {
    let tx: Transaction = serde_json::from_str(tx_json)?;
    let (confirmed, block_hash) = match tx.status {
        Some(status) => (status.confirmed, status.block_hash),
        None => (false, None),
    };
    Ok(TransactionView {
        txid: tx.txid,
        input_count: tx.vin.len(),
        output_count: tx.vout.len(),
        confirmed,
        block_hash,
    })
}

/// Block opened from `GET /block/:hash` or from a height lookup.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlockOpen {
    pub hash: String,
    pub height: u32,
    pub tx_count: u32,
    pub heading: String,
    pub open_path: String,
}

pub fn block_open(block_json: &str) -> Result<BlockOpen, ParseError> {
    let block: Block = serde_json::from_str(block_json)?;
    Ok(BlockOpen {
        hash: block.id.clone(),
        height: block.height,
        tx_count: block.tx_count,
        heading: transaction_count_heading(block.tx_count),
        open_path: block_path(&block.id),
    })
}

pub fn block_open_from_height(
    height: u32,
    height_body: &str,
    block_json: &str,
) -> Result<BlockOpen, ParseError> {
    let hash = parse_block_height(height_body)?;
    let opened = block_open(block_json)?;
    if opened.hash != hash || opened.height != height {
        return Err(ParseError::new("block height did not match the block"));
    }
    Ok(opened)
}

/// One page of `GET /block/:hash/txs` or `GET /block/:hash/txs/:start_index`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlockTxPage {
    pub txids: Vec<String>,
    pub confirmed: Vec<bool>,
    pub input_counts: Vec<usize>,
    pub output_counts: Vec<usize>,
}

pub fn block_tx_page(txs_json: &str) -> Result<BlockTxPage, ParseError> {
    let txs: Vec<Transaction> = serde_json::from_str(txs_json)?;
    let mut page = BlockTxPage {
        txids: Vec::with_capacity(txs.len()),
        confirmed: Vec::with_capacity(txs.len()),
        input_counts: Vec::with_capacity(txs.len()),
        output_counts: Vec::with_capacity(txs.len()),
    };
    for tx in txs {
        page.txids.push(tx.txid);
        page.confirmed.push(
            tx.status
                .as_ref()
                .map(|status| status.confirmed)
                .unwrap_or(false),
        );
        page.input_counts.push(tx.vin.len());
        page.output_counts.push(tx.vout.len());
    }
    Ok(page)
}

pub fn block_pages_differ(page_json: &str, next_json: &str) -> Result<bool, ParseError> {
    let page = block_tx_page(page_json)?;
    let next = block_tx_page(next_json)?;
    Ok(page.txids != next.txids)
}

pub fn block_tx_page_path(hash: &str, start_index: u64) -> String {
    if start_index == 0 {
        block_txs_path(hash)
    } else {
        block_txs_start_index_path(hash, start_index)
    }
}

pub fn next_block_tx_path(hash: &str) -> String {
    block_tx_page_path(hash, BLOCK_TX_PAGE_LEN)
}

/// Previous and next heights. Genesis has no previous. The tip has no next.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AdjacentBlocks {
    pub previous_height: Option<u32>,
    pub next_height: Option<u32>,
    pub previous_shown: bool,
    pub next_shown: bool,
}

pub fn adjacent_blocks(height: u32, tip_height: u32) -> AdjacentBlocks {
    let (previous_shown, next_shown) = block_prev_next_links(Some(height), tip_height);
    AdjacentBlocks {
        previous_height: height.checked_sub(1).filter(|_| previous_shown),
        next_height: height.checked_add(1).filter(|_| next_shown),
        previous_shown,
        next_shown,
    }
}

pub fn block_details_open(open: bool) -> bool {
    details_open_after_click(open)
}

pub fn recent_unconfirmed(body: &str) -> Result<Vec<String>, ParseError> {
    let rows: Vec<RecentTransaction> = serde_json::from_str(body)?;
    Ok(rows.into_iter().map(|row| row.txid).collect())
}

/// Newest first. An id already visible stays where it was.
pub fn recent_with_arrival(visible: &[String], arrived: &str) -> Vec<String> {
    if visible.iter().any(|id| id == arrived) {
        return visible.to_vec();
    }
    let mut next = Vec::with_capacity(visible.len() + 1);
    next.push(arrived.to_string());
    next.extend(visible.iter().cloned());
    next
}

/// Ids captured at pause. A later id that was not visible is left out.
pub fn recent_while_paused(visible: &[String], later: &[String]) -> Vec<String> {
    let visible_ref: Vec<&str> = visible.iter().map(String::as_str).collect();
    let later_ref: Vec<&str> = later.iter().map(String::as_str).collect();
    visible_txids_after_pause(&visible_ref, &later_ref)
        .into_iter()
        .map(str::to_string)
        .collect()
}

pub fn recent_capped(txids: &[String], cap: usize) -> Vec<String> {
    txids.iter().take(cap).cloned().collect()
}

/// `count`, `vsize`, and `total_fee` from `GET /mempool`.
///
/// `vsize` is virtual bytes. The body has no byte size, no incoming rate,
/// and no projected block `nTx` or median fee.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MempoolStatus {
    pub unconfirmed_count: u32,
    pub vsize: u32,
    pub total_fee_sats: u64,
}

pub fn status_from_mempool(body: &str) -> Result<MempoolStatus, ParseError> {
    let summary: MempoolSummary = serde_json::from_str(body)?;
    Ok(MempoolStatus {
        unconfirmed_count: summary.count,
        vsize: summary.vsize,
        total_fee_sats: summary.total_fee,
    })
}
