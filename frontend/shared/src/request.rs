//! One indexer call: method, path, and a body only when the route posts one.
//!
//! Paths come from the public builders in [`crate::paths`]. This module does
//! not replace those builders and does not send HTTP.

use crate::paths::{
    address_path, address_txs_path, address_utxo_path, block_height_path, block_path,
    block_txids_path, block_txs_path, block_txs_start_index_path, blocks_path,
    blocks_start_height_path, blocks_tip_hash_path, blocks_tip_height_path, broadcast_body,
    broadcast_path, fee_estimates_path, mempool_path, mempool_recent_path, test_txs_body,
    test_txs_path, tx_path,
};

/// Method, path after the public prefix, and an optional body.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IndexerRequest {
    pub method: &'static str,
    pub path: String,
    pub body: Option<String>,
}

/// Which indexer route to build.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IndexerCall {
    Blocks,
    BlocksFromHeight(u64),
    BlocksTipHash,
    BlocksTipHeight,
    MempoolRecent,
    Mempool,
    FeeEstimates,
    Block { hash: String },
    BlockTxs { hash: String },
    BlockTxsFromIndex { hash: String, start_index: u64 },
    BlockTxids { hash: String },
    BlockHeight { height: u64 },
    Tx { txid: String },
    Address { script: String },
    AddressTxs { script: String },
    AddressUtxo { script: String },
    Broadcast { hex: String },
    TestTxs { hexes: Vec<String> },
}

/// Build one call from the existing path builders.
pub fn indexer_request(call: &IndexerCall) -> IndexerRequest {
    match call {
        IndexerCall::Blocks => get(blocks_path()),
        IndexerCall::BlocksFromHeight(height) => get(blocks_start_height_path(*height)),
        IndexerCall::BlocksTipHash => get(blocks_tip_hash_path()),
        IndexerCall::BlocksTipHeight => get(blocks_tip_height_path()),
        IndexerCall::MempoolRecent => get(mempool_recent_path()),
        IndexerCall::Mempool => get(mempool_path()),
        IndexerCall::FeeEstimates => get(fee_estimates_path()),
        IndexerCall::Block { hash } => get(block_path(hash)),
        IndexerCall::BlockTxs { hash } => get(block_txs_path(hash)),
        IndexerCall::BlockTxsFromIndex { hash, start_index } => {
            get(block_txs_start_index_path(hash, *start_index))
        }
        IndexerCall::BlockTxids { hash } => get(block_txids_path(hash)),
        IndexerCall::BlockHeight { height } => get(block_height_path(*height)),
        IndexerCall::Tx { txid } => get(tx_path(txid)),
        IndexerCall::Address { script } => get(address_path(script)),
        IndexerCall::AddressTxs { script } => get(address_txs_path(script)),
        IndexerCall::AddressUtxo { script } => get(address_utxo_path(script)),
        IndexerCall::Broadcast { hex } => IndexerRequest {
            method: "POST",
            path: broadcast_path().to_string(),
            body: Some(broadcast_body(hex)),
        },
        IndexerCall::TestTxs { hexes } => {
            let refs: Vec<&str> = hexes.iter().map(String::as_str).collect();
            IndexerRequest {
                method: "POST",
                path: test_txs_path().to_string(),
                body: Some(test_txs_body(&refs)),
            }
        }
    }
}

fn get(path: impl Into<String>) -> IndexerRequest {
    IndexerRequest {
        method: "GET",
        path: path.into(),
        body: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn indexer_request_uses_the_path_builders() {
        let blocks = indexer_request(&IndexerCall::Blocks);
        assert_eq!(blocks.method, "GET");
        assert_eq!(blocks.path, blocks_path());
        assert_eq!(blocks.body, None);

        let from = indexer_request(&IndexerCall::BlocksFromHeight(800_000));
        assert_eq!(from.path, blocks_start_height_path(800_000));

        let block = indexer_request(&IndexerCall::Block {
            hash: "abc".to_string(),
        });
        assert_eq!(block.path, block_path("abc"));

        let page = indexer_request(&IndexerCall::BlockTxsFromIndex {
            hash: "abc".to_string(),
            start_index: 25,
        });
        assert_eq!(page.path, block_txs_start_index_path("abc", 25));
        assert_eq!(page.method, "GET");
        assert_eq!(page.body, None);

        let broadcast = indexer_request(&IndexerCall::Broadcast {
            hex: "  deadbeef\n".to_string(),
        });
        assert_eq!(broadcast.method, "POST");
        assert_eq!(broadcast.path, broadcast_path());
        assert_eq!(broadcast.body.as_deref(), Some("deadbeef"));

        let test = indexer_request(&IndexerCall::TestTxs {
            hexes: vec!["aa".to_string(), "bb".to_string()],
        });
        assert_eq!(test.method, "POST");
        assert_eq!(test.path, test_txs_path());
        assert_eq!(test.body.as_deref(), Some("[\"aa\",\"bb\"]"));
    }
}
