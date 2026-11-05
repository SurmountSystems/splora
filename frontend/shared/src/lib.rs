//! Path builders and JSON parsers for the indexer HTTP API after splora-http
//! strips the public prefix.
//!
//! Satoshi conversion and fiat formatting in this crate are pure functions.
//! They do not use the network. This crate is not the explorer UI.

mod address_parts;
mod api_screen;
mod block_pagination;
mod block_prev_next_links;
mod blocks_page;
mod calculator;
mod dashboard_page;
pub mod dashboard_phone_scrollbar;
mod details_open_after_click;
pub mod difficulty_adjustment;
mod fiat;
mod form;
mod graphs_empty_child_redirect;
mod graphs_page;
pub mod graphs_phone;
mod graphs_phone_buttons;
mod graphs_screen;
mod hashrate_graph;
mod invalid_blinding_hex;
mod language_cookie;
pub mod latest_adjustments;
pub mod latest_blocks;
mod liquid;
mod loads_peg_in_addresses;
mod loads_peg_in_addresses_bitcoin_tx_url;
mod loads_peg_out_addresses;
mod loads_peg_out_addresses_bitcoin_address_url;
mod loads_the_status_screen_backend_synchronizing;
mod loads_the_status_screen_block;
mod loads_the_status_screen_blocks;
mod loads_the_status_screen_incoming_transactions;
mod loads_the_status_screen_mempool_size;
mod loads_the_status_screen_unconfirmed;
mod loads_the_status_screen_vbytes_per_second;
mod loads_the_status_screen_weight_units_per_second;
mod mempool_block_ordinal;
mod mempool_blocks;
mod mined_blocks;
mod network_skeleton;
mod new_tx_pill;
mod new_tx_pill_scroll;
mod official_nodes_status_url;
mod op_return_coinbase_tooltip;
mod op_return_tooltip;
mod parse;
mod paths;
mod pause_icon_updates;
pub mod pools_pie_chart;
mod pools_screen;
mod rbf_mobile_details;
mod rbf_mobile_tracker;
mod rbf_page;
mod recent_list_limit;
mod request;
pub mod reward_stats;
mod rows;
mod screen_text;
mod show_invalid_error_unblinded_tx;
mod theme_option_label;
mod theme_preference_storage;
mod timezone_option_label;
mod transaction_list_over_time;
mod unavailable;
mod unblinded_tx;
mod unblinding;
pub mod wire;

pub use address_parts::{AddressParts, PoisonAlert, address_parts, poison_alerts};
pub use api_screen::{ApiScreen, after_docs_button_loads_the_api_screen};
pub use block_pagination::{BlockPagination, shows_blocks_pagination_with_5_pages_desktop};
pub use block_prev_next_links::{
    BlockNavIcons, FirstBlockchainBlockArrowLeft, FirstBlockchainBlockArrowRight,
    FirstBlockchainBlocksClickArrowRight, GenesisBlockArrowLeft, GenesisBlockArrowRight,
    GenesisBlockClickArrowLeft, block_prev_next_links,
    loads_first_blockchain_block_visible_and_keypress_arrow_left,
    loads_first_blockchain_block_visible_and_keypress_arrow_right,
    loads_first_blockchain_blocks_visible_and_click_on_the_arrow_right,
    loads_genesis_block_and_click_on_the_arrow_left, loads_genesis_block_and_keypress_arrow_left,
    loads_genesis_block_and_keypress_arrow_right,
};
pub use blocks_page::{LoadedBlocksPage, loads_the_blocks_page};
pub use calculator::{
    CappedBtc, DesktopCalculator, MaxSupplyEntry, MobileCalculator, btc_to_sats,
    calculator_is_usable_on_desktop, calculator_is_usable_on_mobile, cap_btc_supply, sats_to_btc,
    shows_warning_when_entering_21m_btc, strip_sats_decimal, strips_decimals_from_satoshis_input,
    truncate_btc_decimals,
};
pub use dashboard_page::{DashboardPage, dashboard_page_initial};
pub use dashboard_phone_scrollbar::{
    DashboardPhoneScrollbar, loads_the_dashboard_with_no_scrollbars_on_mobile,
};
#[cfg(not(test))]
pub use details_open_after_click::details_open_after_click;
pub use difficulty_adjustment::shows_the_difficulty_adjustment_stats;
pub use fiat::{
    FIXTURE_JPY_PER_BTC, FIXTURE_USD_PER_BTC, FiatCode, JpyBtcQuote, UsdBtcQuote, btc_fiat_jpy,
    btc_fiat_usd, correctly_formats_when_switching_from_jpy_back_to_usd,
    correctly_formats_when_switching_from_usd_to_jpy, decimal_comma_to_dot,
    displays_jpy_fiat_values_without_decimals,
    displays_jpy_price_without_decimals_after_switching_currency,
    displays_usd_fiat_values_with_dollar_symbol, displays_usd_price_with_dollar_symbol,
    displays_usd_values_with_correct_currency_symbol,
    displays_usd_values_with_proper_decimal_format, format_fiat,
    formats_all_jpy_values_correctly_without_decimals, jpy_to_btc_and_sats, sats_fiat_jpy,
    sats_fiat_usd, switch_usd_jpy_usd, usd_to_btc_and_sats,
};
pub use form::{
    CalculatorForm, CopyButton, CopyButtonsForEachInput, InputLabelsForCurrencyBtcAndSats,
    calculator_field_names, copy_buttons_exist_and_are_visible, enter_btc, enter_fiat,
    loaded_calculator, shows_calculator_form_after_price_feed_loads,
    shows_copy_buttons_for_each_input, shows_input_labels_for_currency_btc_and_sats,
};
#[cfg(not(test))]
pub use graphs_empty_child_redirect::graphs_empty_child_redirects_to_mempool;
pub use graphs_page::{LoadedGraphsPage, loads_the_graphs_page};
pub use graphs_phone::{GraphsPhone, loads_the_graphs_page_mobile};
pub use graphs_phone_buttons::check_buttons_desktop;
pub use graphs_phone_buttons::check_buttons_tablet;
pub use graphs_phone_buttons::{GraphsPhoneButtons, check_buttons_mobile};
pub use graphs_screen::{GraphsScreen, after_graphs_button_loads_the_graphs_screen};
pub use hashrate_graph::shows_the_hashrate_graph;
pub use invalid_blinding_hex::invalid_blinding_data_hex_error;
pub use language_cookie::language_cookie;
#[cfg(not(test))]
pub use latest_adjustments::shows_the_latest_adjustments;
pub use latest_blocks::shows_the_latest_blocks;
pub use liquid::{
    AssetOutPoint, AssetRegistryEntry, AssetRegistrySearchHit, AssetTxIn, IssuedAssetStats,
    IssuedRegistryAsset, NativeRegistryAsset, PeggedAssetStats, RegistryAsset,
    parse_asset_registry, parse_asset_supply, parse_asset_supply_decimal, parse_asset_txs_chain,
    parse_asset_txs_mempool, parse_assets_registry, parse_assets_registry_search,
    parse_assets_registry_total,
};
#[cfg(not(test))]
pub use loads_peg_in_addresses::loads_peg_in_addresses;
#[cfg(not(test))]
pub use loads_peg_in_addresses_bitcoin_tx_url::loads_peg_in_addresses_bitcoin_tx_url;
#[cfg(not(test))]
pub use loads_peg_out_addresses::loads_peg_out_addresses;
#[cfg(not(test))]
pub use loads_peg_out_addresses_bitcoin_address_url::loads_peg_out_addresses_bitcoin_address_url;
#[cfg(not(test))]
pub use loads_the_status_screen_backend_synchronizing::loads_the_status_screen_backend_synchronizing;
#[cfg(not(test))]
pub use loads_the_status_screen_block::loads_the_status_screen_block;
#[cfg(not(test))]
pub use loads_the_status_screen_blocks::loads_the_status_screen_blocks;
#[cfg(not(test))]
pub use loads_the_status_screen_incoming_transactions::loads_the_status_screen_incoming_transactions;
#[cfg(not(test))]
pub use loads_the_status_screen_mempool_size::loads_the_status_screen_mempool_size;
#[cfg(not(test))]
pub use loads_the_status_screen_unconfirmed::loads_the_status_screen_unconfirmed;
#[cfg(not(test))]
pub use loads_the_status_screen_vbytes_per_second::loads_the_status_screen_vbytes_per_second;
#[cfg(not(test))]
pub use loads_the_status_screen_weight_units_per_second::loads_the_status_screen_weight_units_per_second;
pub use mempool_block_ordinal::mempool_block_ordinal_title;
pub use mempool_blocks::{
    FirstMempoolBlock, LoadedFirstMempoolBlock, MempoolBacklog, first_mempool_block_after_skeleton,
    first_mempool_block_from_backlog, load_first_mempool_block_after_skeleton_loads,
    shows_the_mempool_blocks,
};
pub use mined_blocks::shows_the_mined_blocks;
pub use network_skeleton::{
    loads_skeleton_when_changes_between_networks, skeleton_loader_count_after_network_changes,
};
pub use new_tx_pill::{NewTxPill, shows_the_new_transaction_pill_when_there_are_new_transactions};
pub use new_tx_pill_scroll::shows_the_new_transaction_pill_when_there_are_new_transactions_and_scrolls_to_the_top_when_clicked;
pub use official_nodes_status_url::official_nodes_status_url;
pub use op_return_coinbase_tooltip::check_op_return_coinbase_tooltip;
pub use op_return_tooltip::check_op_return_tx_tooltip;
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
    addresses_txs_path, addresses_txs_summary_page_path, addresses_txs_summary_path,
    asset_page_path, asset_path, asset_registry_path, asset_supply_decimal_path, asset_supply_path,
    asset_txs_chain_page_path, asset_txs_chain_path, asset_txs_mempool_path, asset_txs_path,
    assets_registry_page_path, assets_registry_path, assets_registry_search_path,
    block_header_path, block_height_path, block_path, block_raw_path, block_status_path,
    block_txid_path, block_txids_path, block_txs_path, block_txs_start_index_path, blocks_path,
    blocks_start_height_path, blocks_tip_hash_path, blocks_tip_height_path, broadcast_body,
    broadcast_path, fee_estimates_path, indexer_routes, mempool_path, mempool_recent_path,
    mempool_txids_page_path, mempool_txids_path, package_body, package_path, path_list,
    scripthash_path, scripthash_txs_chain_page_path, scripthash_txs_chain_path,
    scripthash_txs_mempool_path, scripthash_txs_path, scripthash_txs_summary_page_path,
    scripthash_txs_summary_path, scripthash_utxo_path, scripthashes_txs_body,
    scripthashes_txs_path, scripthashes_txs_summary_page_path, scripthashes_txs_summary_path,
    test_txs_body, test_txs_path, tx_hex_path, tx_merkle_proof_path, tx_merkleblock_proof_path,
    tx_outspend_path, tx_outspends_path, tx_path, tx_raw_path, tx_status_path, txs_outspends_path,
};
pub use pause_icon_updates::{
    after_pause_icon_click_updates_are_paused, visible_txids_after_pause,
};
pub use pools_pie_chart::shows_the_pools_pie_chart;
pub use pools_screen::{PoolsScreen, pools_screen_after_pools_control};
pub use rbf_mobile_details::mobile_rbf_replacement_alert;
pub use rbf_mobile_tracker::mobile_rbf_tracker_alert;
pub use rbf_page::{RbfPageInitial, RbfSocketTrees, rbf_page_initial, rbf_socket_trees};
pub use recent_list_limit::caps_the_list_when_changing_the_limit_to_10;
pub use request::{IndexerCall, IndexerRequest, broadcast_route_request, indexer_request};
pub use reward_stats::shows_the_reward_stats_for_the_last_144_blocks;
pub use rows::{
    AddressRow, BlockRow, FeeEstimateRow, ScriptStatsRow, TxRow, UtxoRow, fee_estimate_rows,
    sat_per_vbyte_from_vsize, sat_per_vbyte_from_weight,
};
pub use screen_text::{
    BlockScreenAfterNextPage, HighlightCount, NoPaginationBlockScreen,
    after_next_page_click_supports_pagination_on_the_block_screen, cap_recent_transactions,
    empty_block_heading_from_tx_count, highlight_counts, no_pagination_block_screen,
    transaction_count_heading,
};
#[cfg(not(test))]
pub use show_invalid_error_unblinded_tx::show_invalid_error_unblinded_tx;
pub use theme_option_label::theme_option_label;
pub use theme_preference_storage::theme_preference_storage;
pub use timezone_option_label::timezone_option_label;
pub use transaction_list_over_time::updates_the_transaction_list_over_time;
pub use unavailable::route_availability;
pub use unblinded_tx::{
    FirstUnblindedVout, SecondUnblindedVout, show_first_unblinded_vout, show_second_unblinded_vout,
    unblinded_tx_preview,
};
pub use unblinding::unblinding_error_for_regular_tx;
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

#[cfg(test)]
#[test]
fn shows_the_latest_adjustments() {
    let host = latest_adjustments::shows_the_latest_adjustments();
    assert_eq!(host, "difficulty-adjustments-table");

    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src/latest_adjustments.rs");
    let product =
        std::fs::read_to_string(&path).unwrap_or_else(|error| panic!("read {path:?}: {error}"));
    let has_difficulty_percent_or_numeric_difficulty =
        |source: &str| source.contains('%') || source.bytes().any(|byte| byte.is_ascii_digit());
    assert!(
        !has_difficulty_percent_or_numeric_difficulty(&product),
        "latest_adjustments.rs must not contain a difficulty percent or a numeric difficulty"
    );
}

#[cfg(test)]
#[test]
fn loads_the_status_screen_incoming_transactions() {
    use loads_the_status_screen_incoming_transactions::loads_the_status_screen_incoming_transactions;
    assert_eq!(
        loads_the_status_screen_incoming_transactions(),
        "Incoming Transactions"
    );
}

#[cfg(test)]
#[test]
fn loads_the_status_screen_vbytes_per_second() {
    use loads_the_status_screen_vbytes_per_second::loads_the_status_screen_vbytes_per_second;
    assert_eq!(loads_the_status_screen_vbytes_per_second(), "vB/s");
}

#[cfg(test)]
#[test]
fn loads_the_status_screen_unconfirmed() {
    use loads_the_status_screen_unconfirmed::loads_the_status_screen_unconfirmed;
    assert_eq!(loads_the_status_screen_unconfirmed(), "Unconfirmed");
}

#[cfg(test)]
#[test]
fn loads_the_status_screen_mempool_size() {
    use loads_the_status_screen_mempool_size::loads_the_status_screen_mempool_size;
    assert_eq!(loads_the_status_screen_mempool_size(), "Mempool size");
}

#[cfg(test)]
#[test]
fn loads_the_status_screen_blocks() {
    use loads_the_status_screen_blocks::loads_the_status_screen_blocks;
    assert_eq!(loads_the_status_screen_blocks(), "blocks");
}

#[cfg(test)]
#[test]
fn loads_the_status_screen_block() {
    use loads_the_status_screen_block::loads_the_status_screen_block;
    assert_eq!(loads_the_status_screen_block(), "block");
}

#[cfg(test)]
#[test]
fn loads_the_status_screen_backend_synchronizing() {
    use loads_the_status_screen_backend_synchronizing::loads_the_status_screen_backend_synchronizing;
    assert_eq!(
        loads_the_status_screen_backend_synchronizing(),
        "Backend is synchronizing"
    );
}

#[cfg(test)]
#[test]
fn loads_the_status_screen_weight_units_per_second() {
    use loads_the_status_screen_weight_units_per_second::loads_the_status_screen_weight_units_per_second;
    assert_eq!(loads_the_status_screen_weight_units_per_second(), "WU/s");
}

#[cfg(test)]
#[test]
fn show_invalid_error_unblinded_tx() {
    use show_invalid_error_unblinded_tx::show_invalid_error_unblinded_tx;
    assert_eq!(
        show_invalid_error_unblinded_tx(),
        "Error: Invalid blinding data."
    );
}

#[cfg(test)]
#[test]
fn loads_peg_in_addresses() {
    use loads_peg_in_addresses::loads_peg_in_addresses;
    assert_eq!(loads_peg_in_addresses(), "Peg-in");
}

#[cfg(test)]
#[test]
fn loads_peg_in_addresses_bitcoin_tx_url() {
    assert_eq!(
        loads_peg_in_addresses_bitcoin_tx_url::loads_peg_in_addresses_bitcoin_tx_url(
            "f148c0d854db4174ea420655235f910543f0ec3680566dcfdf84fb0a1697b592",
            0
        ),
        "https://mempool.space/tx/f148c0d854db4174ea420655235f910543f0ec3680566dcfdf84fb0a1697b592#vout=0"
    );
}

#[cfg(test)]
#[test]
fn loads_peg_out_addresses() {
    use loads_peg_out_addresses::loads_peg_out_addresses;
    assert_eq!(loads_peg_out_addresses(), "Liquid Peg Out");
}

#[cfg(test)]
#[test]
fn loads_peg_out_addresses_bitcoin_address_url() {
    use loads_peg_out_addresses_bitcoin_address_url::loads_peg_out_addresses_bitcoin_address_url;
    assert_eq!(
        loads_peg_out_addresses_bitcoin_address_url("1BxoGcMg14oaH3CwHD2hF4gU9VcfgX5yoR"),
        "https://mempool.space/address/1BxoGcMg14oaH3CwHD2hF4gU9VcfgX5yoR"
    );
}

#[cfg(test)]
#[test]
fn graphs_empty_child_redirects_to_mempool() {
    use graphs_empty_child_redirect::graphs_empty_child_redirects_to_mempool;
    assert_eq!(graphs_empty_child_redirects_to_mempool(), "mempool");
}

#[cfg(test)]
#[test]
fn details_open_after_click() {
    assert_eq!(
        details_open_after_click::details_open_after_click(false),
        true
    );
    assert_eq!(
        details_open_after_click::details_open_after_click(true),
        false
    );
}
