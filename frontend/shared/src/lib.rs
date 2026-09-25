//! Path builders and JSON parsers for the indexer HTTP API after splora-http
//! strips the public prefix.
//!
//! This crate is not the explorer UI.

mod parse;
mod paths;

pub use parse::{
    AddressStats, BacklogStats, Block, FeeEstimate, ParseError, RecentTx, TestTxResult, Tx, Utxo,
    parse_address, parse_address_txs, parse_address_utxos, parse_block, parse_block_height,
    parse_block_txids, parse_block_txs, parse_blocks, parse_blocks_tip_hash,
    parse_blocks_tip_height, parse_broadcast, parse_fee_estimates, parse_mempool,
    parse_mempool_recent, parse_test_txs, parse_tx,
};
pub use paths::{
    Route, address_path, address_txs_path, address_utxo_path, block_height_path, block_path,
    block_txids_path, block_txs_path, block_txs_start_index_path, blocks_path,
    blocks_start_height_path, blocks_tip_hash_path, blocks_tip_height_path, broadcast_body,
    broadcast_path, fee_estimates_path, indexer_routes, mempool_path, mempool_recent_path,
    path_list, test_txs_body, test_txs_path, tx_path,
};
