//! Path builders and JSON parsers for the indexer HTTP API after splora-http
//! strips the public prefix.
//!
//! Satoshi conversion and fiat formatting in this crate are pure functions.
//! They do not use the network. This crate is not the explorer UI.

mod address_parts;
mod calculator;
mod fiat;
mod form;
mod liquid;
mod parse;
mod paths;
mod request;
mod rows;
mod screen_text;
mod unavailable;
pub mod wire;

pub use address_parts::{AddressParts, PoisonAlert, address_parts, poison_alerts};
pub use calculator::{
    CappedBtc, btc_to_sats, cap_btc_supply, sats_to_btc, strip_sats_decimal, truncate_btc_decimals,
};
pub use fiat::{
    FIXTURE_JPY_PER_BTC, FIXTURE_USD_PER_BTC, FiatCode, JpyBtcQuote, UsdBtcQuote, btc_fiat_jpy,
    btc_fiat_usd, decimal_comma_to_dot, format_fiat, jpy_to_btc_and_sats, sats_fiat_jpy,
    sats_fiat_usd, switch_usd_jpy_usd, usd_to_btc_and_sats,
};
pub use form::{CalculatorForm, calculator_field_names, enter_btc, enter_fiat, loaded_calculator};
pub use liquid::{
    AssetOutPoint, AssetRegistryEntry, AssetRegistrySearchHit, AssetTxIn, IssuedAssetStats,
    IssuedRegistryAsset, NativeRegistryAsset, PeggedAssetStats, RegistryAsset,
    parse_asset_registry, parse_asset_supply, parse_asset_supply_decimal, parse_asset_txs_chain,
    parse_asset_txs_mempool, parse_assets_registry, parse_assets_registry_search,
    parse_assets_registry_total,
};
pub use parse::{
    AddressStats, BacklogStats, Block, FeeEstimate, ParseError, RecentTx, ScriptHashStats,
    TestTxResult, Tx, Utxo, parse_address, parse_address_prefix, parse_address_txs,
    parse_address_txs_chain, parse_address_txs_mempool, parse_address_txs_summary,
    parse_address_utxos, parse_addresses_txs, parse_addresses_txs_summary, parse_block,
    parse_block_header, parse_block_header_bytes, parse_block_height, parse_block_raw,
    parse_block_raw_hex, parse_block_status, parse_block_txid, parse_block_txids, parse_block_txs,
    parse_blocks, parse_blocks_tip_hash, parse_blocks_tip_height, parse_broadcast,
    parse_fee_estimates, parse_mempool, parse_mempool_recent, parse_mempool_txids,
    parse_merkle_proof, parse_merkleblock_proof, parse_merkleblock_proof_bytes, parse_outspend,
    parse_outspends, parse_package, parse_scripthash, parse_scripthash_txs,
    parse_scripthash_txs_chain, parse_scripthash_txs_mempool, parse_scripthash_txs_summary,
    parse_scripthash_utxos, parse_scripthashes_txs, parse_scripthashes_txs_summary, parse_test_txs,
    parse_tx, parse_tx_hex, parse_tx_raw, parse_tx_raw_hex, parse_tx_status, parse_txs_outspends,
};
pub use paths::{
    Route, TXS_OUTSPENDS_MAX, address_path, address_prefix_path, address_txs_chain_page_path,
    address_txs_chain_path, address_txs_mempool_path, address_txs_path,
    address_txs_summary_page_path, address_txs_summary_path, address_utxo_path, addresses_txs_body,
    addresses_txs_path, addresses_txs_summary_page_path, addresses_txs_summary_path, asset_path,
    asset_registry_path, asset_supply_decimal_path, asset_supply_path, asset_txs_chain_page_path,
    asset_txs_chain_path, asset_txs_mempool_path, asset_txs_path, assets_registry_page_path,
    assets_registry_path, assets_registry_search_path, block_header_path, block_height_path,
    block_path, block_raw_path, block_status_path, block_txid_path, block_txids_path,
    block_txs_path, block_txs_start_index_path, blocks_path, blocks_start_height_path,
    blocks_tip_hash_path, blocks_tip_height_path, broadcast_body, broadcast_path,
    fee_estimates_path, indexer_routes, mempool_path, mempool_recent_path, mempool_txids_page_path,
    mempool_txids_path, package_body, package_path, path_list, scripthash_path,
    scripthash_txs_chain_page_path, scripthash_txs_chain_path, scripthash_txs_mempool_path,
    scripthash_txs_path, scripthash_txs_summary_page_path, scripthash_txs_summary_path,
    scripthash_utxo_path, scripthashes_txs_body, scripthashes_txs_path,
    scripthashes_txs_summary_page_path, scripthashes_txs_summary_path, test_txs_body,
    test_txs_path, tx_hex_path, tx_merkle_proof_path, tx_merkleblock_proof_path, tx_outspend_path,
    tx_outspends_path, tx_path, tx_raw_path, tx_status_path, txs_outspends_path,
};
pub use request::{IndexerCall, IndexerRequest, indexer_request};
pub use rows::{
    AddressRow, BlockRow, FeeEstimateRow, ScriptStatsRow, TxRow, UtxoRow, fee_estimate_rows,
    sat_per_vbyte_from_vsize, sat_per_vbyte_from_weight,
};
pub use screen_text::{
    HighlightCount, cap_recent_transactions, highlight_counts, transaction_count_heading,
};
pub use unavailable::route_availability;
pub use wire::{
    BlockStatus, Issuance, MerkleProof, Outspend, PackageResult, PackageTx, PackageTxFees, Pegout,
    TxSummary,
};

#[cfg(test)]
mod tests {
    #[test]
    fn public_calculator_field_names_are_fiat_bitcoin_and_satoshis() {
        assert_eq!(
            crate::calculator_field_names(),
            ["fiat", "bitcoin", "satoshis"]
        );
    }
}
