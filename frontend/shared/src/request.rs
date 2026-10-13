//! One indexer call: method, path, and a body only when the route posts one.
//!
//! Paths come from the public builders in [`crate::paths`]. This module does
//! not replace those builders and does not send HTTP.

use crate::paths::{
    address_path, address_prefix_path, address_txs_chain_page_path, address_txs_chain_path,
    address_txs_mempool_path, address_txs_path, address_txs_summary_page_path,
    address_txs_summary_path, address_utxo_path, addresses_txs_body, addresses_txs_path,
    addresses_txs_summary_page_path, addresses_txs_summary_path, asset_path,
    asset_supply_decimal_path, asset_supply_path, asset_txs_chain_page_path, asset_txs_chain_path,
    asset_txs_mempool_path, assets_registry_path, assets_registry_search_path, block_header_path,
    block_height_path, block_path, block_raw_path, block_status_path, block_txid_path,
    block_txids_path, block_txs_path, block_txs_start_index_path, blocks_path,
    blocks_start_height_path, blocks_tip_hash_path, blocks_tip_height_path, broadcast_body,
    broadcast_path, fee_estimates_path, mempool_path, mempool_recent_path, mempool_txids_page_path,
    mempool_txids_path, package_body, package_path, scripthash_path,
    scripthash_txs_chain_page_path, scripthash_txs_chain_path, scripthash_txs_mempool_path,
    scripthash_txs_path, scripthash_txs_summary_page_path, scripthash_txs_summary_path,
    scripthash_utxo_path, scripthashes_txs_body, scripthashes_txs_path,
    scripthashes_txs_summary_page_path, scripthashes_txs_summary_path, test_txs_body,
    test_txs_path, tx_hex_path, tx_merkle_proof_path, tx_merkleblock_proof_path, tx_outspend_path,
    tx_outspends_path, tx_path, tx_raw_path, tx_status_path,
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
    Block {
        hash: String,
    },
    BlockTxs {
        hash: String,
    },
    BlockTxsFromIndex {
        hash: String,
        start_index: u64,
    },
    BlockTxids {
        hash: String,
    },
    BlockHeight {
        height: u64,
    },
    Tx {
        txid: String,
    },
    Address {
        script: String,
    },
    AddressTxs {
        script: String,
    },
    AddressUtxo {
        script: String,
    },
    Broadcast {
        hex: String,
    },
    TestTxs {
        hexes: Vec<String>,
    },
    BlockStatus {
        hash: String,
    },
    TxOutspends {
        txid: String,
    },
    TxOutspend {
        txid: String,
        index: u64,
    },
    TxMerkleProof {
        txid: String,
    },
    TxHex {
        txid: String,
    },
    AddressTxsChain {
        script: String,
    },
    AddressTxsChainPage {
        script: String,
        last_seen_txid: String,
    },
    AddressTxsMempool {
        script: String,
    },
    AddressTxsSummary {
        script: String,
    },
    AddressTxsSummaryPage {
        script: String,
        last_seen_txid: String,
    },
    MempoolTxids,
    MempoolTxidsPage {
        last_seen_txid: String,
    },
    AddressPrefix {
        prefix: String,
    },
    ScriptHash {
        hash: String,
    },
    ScriptHashTxs {
        hash: String,
    },
    ScriptHashTxsChain {
        hash: String,
    },
    ScriptHashTxsChainPage {
        hash: String,
        last_seen_txid: String,
    },
    ScriptHashTxsMempool {
        hash: String,
    },
    ScriptHashUtxo {
        hash: String,
    },
    Package {
        hexes: Vec<String>,
    },
    Asset {
        asset: String,
    },
    ScriptHashTxsSummary {
        hash: String,
    },
    ScriptHashTxsSummaryPage {
        hash: String,
        last_seen_txid: String,
    },
    AddressesTxs {
        scripts: Vec<String>,
    },
    ScriptHashesTxs {
        hashes: Vec<String>,
    },
    AddressesTxsSummary {
        scripts: Vec<String>,
    },
    AddressesTxsSummaryPage {
        scripts: Vec<String>,
        last_seen_txid: String,
    },
    ScriptHashesTxsSummary {
        hashes: Vec<String>,
    },
    ScriptHashesTxsSummaryPage {
        hashes: Vec<String>,
        last_seen_txid: String,
    },
    TxStatus {
        txid: String,
    },
    BlockTxid {
        hash: String,
        index: u64,
    },
    BlockHeader {
        hash: String,
    },
    BlockRaw {
        hash: String,
    },
    TxRaw {
        txid: String,
    },
    TxMerkleBlockProof {
        txid: String,
    },
    AssetsRegistry,
    AssetsRegistrySearch {
        query: String,
    },
    AssetRegistry {
        asset: String,
    },
    AssetTxsChain {
        asset: String,
    },
    AssetTxsChainPage {
        asset: String,
        last_seen_txid: String,
    },
    AssetTxsMempool {
        asset: String,
    },
    AssetSupply {
        asset: String,
    },
    AssetSupplyDecimal {
        asset: String,
    },
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
        IndexerCall::BlockStatus { hash } => get(block_status_path(hash)),
        IndexerCall::TxOutspends { txid } => get(tx_outspends_path(txid)),
        IndexerCall::TxOutspend { txid, index } => get(tx_outspend_path(txid, *index)),
        IndexerCall::TxMerkleProof { txid } => get(tx_merkle_proof_path(txid)),
        IndexerCall::TxHex { txid } => get(tx_hex_path(txid)),
        IndexerCall::AddressTxsChain { script } => get(address_txs_chain_path(script)),
        IndexerCall::AddressTxsChainPage {
            script,
            last_seen_txid,
        } => get(address_txs_chain_page_path(script, last_seen_txid)),
        IndexerCall::AddressTxsMempool { script } => get(address_txs_mempool_path(script)),
        IndexerCall::AddressTxsSummary { script } => get(address_txs_summary_path(script)),
        IndexerCall::AddressTxsSummaryPage {
            script,
            last_seen_txid,
        } => get(address_txs_summary_page_path(script, last_seen_txid)),
        IndexerCall::MempoolTxids => get(mempool_txids_path()),
        IndexerCall::MempoolTxidsPage { last_seen_txid } => {
            get(mempool_txids_page_path(last_seen_txid))
        }
        IndexerCall::AddressPrefix { prefix } => get(address_prefix_path(prefix)),
        IndexerCall::ScriptHash { hash } => get(scripthash_path(hash)),
        IndexerCall::ScriptHashTxs { hash } => get(scripthash_txs_path(hash)),
        IndexerCall::ScriptHashTxsChain { hash } => get(scripthash_txs_chain_path(hash)),
        IndexerCall::ScriptHashTxsChainPage {
            hash,
            last_seen_txid,
        } => get(scripthash_txs_chain_page_path(hash, last_seen_txid)),
        IndexerCall::ScriptHashTxsMempool { hash } => get(scripthash_txs_mempool_path(hash)),
        IndexerCall::ScriptHashUtxo { hash } => get(scripthash_utxo_path(hash)),
        IndexerCall::Package { hexes } => {
            let refs: Vec<&str> = hexes.iter().map(String::as_str).collect();
            IndexerRequest {
                method: "POST",
                path: package_path().to_string(),
                body: Some(package_body(&refs)),
            }
        }
        IndexerCall::Asset { asset } => get(asset_path(asset)),
        IndexerCall::ScriptHashTxsSummary { hash } => get(scripthash_txs_summary_path(hash)),
        IndexerCall::ScriptHashTxsSummaryPage {
            hash,
            last_seen_txid,
        } => get(scripthash_txs_summary_page_path(hash, last_seen_txid)),
        IndexerCall::AddressesTxs { scripts } => post(
            addresses_txs_path(),
            addresses_txs_body(&script_refs(scripts)),
        ),
        IndexerCall::ScriptHashesTxs { hashes } => post(
            scripthashes_txs_path(),
            scripthashes_txs_body(&script_refs(hashes)),
        ),
        IndexerCall::AddressesTxsSummary { scripts } => post(
            addresses_txs_summary_path(),
            addresses_txs_body(&script_refs(scripts)),
        ),
        IndexerCall::AddressesTxsSummaryPage {
            scripts,
            last_seen_txid,
        } => post(
            addresses_txs_summary_page_path(last_seen_txid),
            addresses_txs_body(&script_refs(scripts)),
        ),
        IndexerCall::ScriptHashesTxsSummary { hashes } => post(
            scripthashes_txs_summary_path(),
            scripthashes_txs_body(&script_refs(hashes)),
        ),
        IndexerCall::ScriptHashesTxsSummaryPage {
            hashes,
            last_seen_txid,
        } => post(
            scripthashes_txs_summary_page_path(last_seen_txid),
            scripthashes_txs_body(&script_refs(hashes)),
        ),
        IndexerCall::TxStatus { txid } => get(tx_status_path(txid)),
        IndexerCall::BlockTxid { hash, index } => get(block_txid_path(hash, *index)),
        IndexerCall::BlockHeader { hash } => get(block_header_path(hash)),
        IndexerCall::BlockRaw { hash } => get(block_raw_path(hash)),
        IndexerCall::TxRaw { txid } => get(tx_raw_path(txid)),
        IndexerCall::TxMerkleBlockProof { txid } => get(tx_merkleblock_proof_path(txid)),
        IndexerCall::AssetsRegistry => get(assets_registry_path()),
        IndexerCall::AssetsRegistrySearch { query } => get(assets_registry_search_path(query)),
        IndexerCall::AssetRegistry { asset } => get(crate::paths::asset_registry_path(asset)),
        IndexerCall::AssetTxsChain { asset } => get(asset_txs_chain_path(asset)),
        IndexerCall::AssetTxsChainPage {
            asset,
            last_seen_txid,
        } => get(asset_txs_chain_page_path(asset, last_seen_txid)),
        IndexerCall::AssetTxsMempool { asset } => get(asset_txs_mempool_path(asset)),
        IndexerCall::AssetSupply { asset } => get(asset_supply_path(asset)),
        IndexerCall::AssetSupplyDecimal { asset } => get(asset_supply_decimal_path(asset)),
    }
}

fn script_refs(values: &[String]) -> Vec<&str> {
    values.iter().map(String::as_str).collect()
}

fn post(path: impl Into<String>, body: String) -> IndexerRequest {
    IndexerRequest {
        method: "POST",
        path: path.into(),
        body: Some(body),
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
