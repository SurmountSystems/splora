//! Splora browser explorer.
//!
//! `frontend/splora-api` is the shared prefix table and NIP-98 header encoder.
//! This crate does not fork that client.

mod api_screen;
mod auth;
mod browser;
mod caps_the_list_when_changing_the_limit_to_10;
mod graphs_screen;
mod liquid;
mod local_pages;
mod models;
mod network_skeleton;
mod paths;
mod pause_icon_updates;
mod pools_screen;
mod regular_tx_unblinding;
mod rows;
mod screens;
mod search;
mod show_unblinded_tx;
mod theme;
mod usd_decimal;
mod usd_symbol;
mod views;

pub use api_screen::paint_loads_the_api_screen;
pub use auth::{
    ClientError, FAIL_CLOSED, MissingSigner, Nip98Signer, UnsignedNip98, browser_fetch_url,
    fetch_indexer, nip98_u_url, post_indexer, signer_from_presence, text_after_signed_get,
    unsigned_get,
};
pub use browser::browser_signer_present;
pub use caps_the_list_when_changing_the_limit_to_10::paint_caps_the_list_when_changing_the_limit_to_10;
pub use graphs_screen::graphs_screen_markup;
pub use liquid::{
    asset_cards_markup, asset_confirmed_page_request, asset_confirmed_txs_request,
    asset_history_markup, asset_mempool_txs_request, asset_search_markup,
    asset_supply_decimal_request, asset_supply_request, liquid_asset_requests,
    registry_entry_markup, registry_entry_request, registry_list_markup, registry_list_request,
    registry_page_request, registry_search_markup, registry_search_request, supply_markup,
};
pub use local_pages::{
    calculator_fiat_text, capped_btc_text, docs_view_text, graphs_page_text, hashrate_page_text,
    jpy_fixture_text, lightning_page_text, mining_page_text, one_btc_sats_text,
    price_chart_page_text, statistics_page_text, switched_fiat_symbols, truncated_btc_text,
    usd_fixture_text,
};
pub use models::{
    parse_address_stats_json, parse_block_json, parse_blocks_json, parse_blocks_tip_hash,
    parse_blocks_tip_height, parse_broadcast_result_json, parse_fee_estimates,
    parse_mempool_summary_json, parse_recent_transactions_json, parse_test_tx_results_json,
    parse_transaction_json, parse_transactions_json, parse_utxos_json, recent_txids,
};
pub use network_skeleton::paint_loads_skeleton_when_changes_between_networks;
pub use paths::{
    IndexerRequest, address_path, address_txs_path, block_path, block_txids_path, block_txs_path,
    broadcast_request, dashboard_paths, docs_api_type_page, mempool_paths, multi_address_requests,
    multi_address_requests_from_values, multi_address_screen_route, screen_routes,
    static_page_requests, static_page_routes, switcher_labels, test_transactions_request, tx_path,
    v1_client_requests,
};
pub use pause_icon_updates::paint_pauses_updates_when_clicking_the_pause_icon;
pub use pools_screen::pools_screen_markup;
pub use regular_tx_unblinding::regular_tx_unblinding_markup;
pub use rows::{
    address_highlight_markup, address_rows_markup, asset_markup, batch_outspends_markup,
    block_details_markup, block_header_aside_markup, block_header_hex_markup, block_raw_beside,
    block_rows_markup, block_status_markup, block_transactions_markup, block_tx_page_path,
    block_txid_index_markup, block_txid_links_markup, blocks_rows_markup, broadcast_txid_markup,
    chain_tip_markup, confirmation_markup, dashboard_rows_markup, details_open_after_click,
    height_body_markup, indexed_bytes_markup, mempool_txids_markup, merkle_proof_markup,
    multi_address_links_markup, one_outspend_markup, outspends_markup, package_txid_markup,
    poison_alert_markup, recent_rows_markup, scripthash_markup, summary_rows_markup,
    summary_rows_with_status, test_txid_markup, tip_links_markup, transaction_hex_markup,
    transaction_row_markup, tx_raw_beside, tx_status_markup,
};
pub use screens::{
    BatchPost, address_chain_request, address_fields_text, address_mempool_txs_request,
    address_requests, address_summary_page_request, address_summary_request, asset_requests,
    block_fields_text, block_header_request, block_load_plan, block_raw_request,
    block_status_request, block_txid_index_request, block_txs_older_href, blocks_list_fields_text,
    blocks_list_requests, blocks_older_href, broadcast_fields_text, broadcast_payload,
    broadcast_screen_request, broadcast_submission, dashboard_fields_text,
    dashboard_screen_requests, entered_addresses, indexer_failure_text, mempool_txid_page_request,
    mempool_txids_request, multi_address_fields_text, multi_address_screen_requests,
    multi_address_summary_post, multi_address_txs_post, resolved_block_hash,
    test_transactions_fields_text, test_transactions_payload, test_transactions_screen_request,
    transaction_detail_requests, transaction_fields_text, transaction_requests,
    tx_merkleblock_proof_request, tx_outspend_index_request, tx_raw_request, tx_status_request,
    txs_outspends_request,
};
pub use search::{
    SearchPlan, address_prefix_markup, plan_search, plan_search_on, search_links_markup,
    search_loaded_markup, search_result_markup,
};
pub use show_unblinded_tx::show_unblinded_tx_markup;
pub use splora_frontend_shared::{assets_registry_path, assets_registry_search_path};
pub use theme::{ACCENT, BACKGROUND, TEXT};
pub use usd_decimal::paint_displays_usd_values_with_proper_decimal_format;
pub use usd_symbol::paint_displays_usd_values_with_correct_currency_symbol;
pub use views::{
    App, address_balance_history_markup, address_unspent_outputs_markup, asset_page_markup,
    block_page_heading_markup, block_page_weight_markup, broadcast_transaction_heading,
    dashboard_loaded_markup, dashboard_mined_markup, dashboard_recent_blocks_markup,
    dashboard_tip_fees_text, first_mempool_block_link_markup, looks_like_block_hash,
    mempool_size_markup, mempool_summary_text, paint_supports_pagination_on_the_block_screen,
    rbf_page_socket_markup, shows_blocks_with_no_pagination_markup,
    shows_empty_blocks_properly_markup, submission_links, transaction_fees_heading,
};

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen::prelude::wasm_bindgen(start)]
pub fn start() {
    leptos::mount::mount_to_body(App);
}
