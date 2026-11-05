//! Splora browser explorer.
//!
//! `frontend/splora-api` is the shared prefix table and NIP-98 header encoder.
//! This crate does not fork that client.

mod api_screen;
mod auth;
mod block_prev_next_links;
mod broadcast_route_request;
mod browser;
pub mod calculator_is_usable_on_desktop;
pub mod calculator_is_usable_on_mobile;
mod caps_the_list_when_changing_the_limit_to_10;
mod check_buttons_desktop;
mod check_buttons_mobile;
mod check_buttons_tablet;
mod check_op_return_coinbase_tooltip;
mod check_op_return_tx_tooltip;
pub mod copy_buttons_exist_and_are_visible;
pub mod correctly_formats_when_switching_from_jpy_back_to_usd;
pub mod correctly_formats_when_switching_from_usd_to_jpy;
mod desktop_rbf;
pub mod displays_jpy_fiat_values_without_decimals;
pub mod displays_jpy_price_without_decimals_after_switching_currency;
mod displays_jpy_values_with_yen_symbol;
mod displays_jpy_values_without_decimal_places;
pub mod displays_usd_fiat_values_with_dollar_symbol;
pub mod displays_usd_price_with_dollar_symbol;
pub mod formats_all_jpy_values_correctly_without_decimals;
pub mod graphs_empty_child_redirects_to_mempool;
mod graphs_screen;
mod invalid_blinding_hex;
mod language_cookie;
pub mod limits_btc_to_8_decimals;
mod liquid;
mod load_first_mempool_block_after_skeleton_loads;
mod loads_first_blockchain_block_visible_and_keypress_arrow_left;
mod loads_first_blockchain_block_visible_and_keypress_arrow_right;
mod loads_first_blockchain_blocks_visible_and_click_on_the_arrow_right;
mod loads_genesis_block_and_click_on_the_arrow_left;
mod loads_genesis_block_and_keypress_arrow_left;
mod loads_genesis_block_and_keypress_arrow_right;
pub mod loads_peg_in_addresses;
pub mod loads_peg_in_addresses_bitcoin_tx_url;
pub mod loads_peg_out_addresses;
pub mod loads_peg_out_addresses_bitcoin_address_url;
mod loads_the_blocks_page;
mod loads_the_dashboard;
pub mod loads_the_dashboard_with_no_scrollbars_on_mobile;
mod loads_the_graphs_page;
pub mod loads_the_graphs_page_mobile;
pub mod loads_the_status_screen_backend_synchronizing;
pub mod loads_the_status_screen_block;
pub mod loads_the_status_screen_blocks;
pub mod loads_the_status_screen_incoming_transactions;
pub mod loads_the_status_screen_mempool_size;
pub mod loads_the_status_screen_unconfirmed;
pub mod loads_the_status_screen_vbytes_per_second;
pub mod loads_the_status_screen_weight_units_per_second;
mod local_pages;
mod mempool_block_ordinal;
mod models;
mod network_skeleton;
mod normalizes_comma_to_dot_in_fiat_input;
mod official_nodes_status_url;
mod paths;
mod pause_icon_updates;
mod pools_screen;
mod rbf_mobile_details;
mod rbf_mobile_tracker;
mod regular_tx_unblinding;
mod rows;
mod screens;
mod search;
mod show_empty_unblinded_tx;
mod show_first_unblinded_vout;
pub mod show_invalid_error_unblinded_tx;
pub mod show_second_unblinded_vout;
mod show_unblinded_tx;
mod shows_calculator_form_after_price_feed_loads;
mod shows_copy_buttons_for_each_input;
mod shows_input_labels_for_currency_btc_and_sats;
pub mod shows_the_difficulty_adjustment_stats;
pub mod shows_the_hashrate_graph;
pub mod shows_the_latest_adjustments;
pub mod shows_the_latest_blocks;
pub mod shows_the_mempool_blocks;
pub mod shows_the_mined_blocks;
mod shows_the_new_transaction_pill_when_there_are_new_transactions;
pub mod shows_the_pools_pie_chart;
pub mod shows_the_reward_stats_for_the_last_144_blocks;
pub mod shows_warning_when_entering_21m_btc;
mod signet_redirect;
pub mod strips_decimals_from_satoshis_input;
mod theme;
mod theme_option_label;
mod theme_preference_storage;
mod timezone_option_label;
mod unblinded_tx_asset_page;
pub mod updates_the_transaction_list_over_time;
mod usd_decimal;
mod usd_symbol;
mod views;

pub use api_screen::paint_loads_the_api_screen;
pub use auth::{
    ClientError, FAIL_CLOSED, MissingSigner, Nip98Signer, UnsignedNip98, browser_fetch_url,
    fetch_indexer, nip98_u_url, post_indexer, signer_from_presence, text_after_signed_get,
    unsigned_get,
};
pub use block_prev_next_links::paint_block_prev_next_links;
pub use broadcast_route_request::paint_broadcast_route_request;
pub use browser::browser_signer_present;
pub use calculator_is_usable_on_desktop::paint_calculator_is_usable_on_desktop;
pub use calculator_is_usable_on_mobile::paint_calculator_is_usable_on_mobile;
pub use caps_the_list_when_changing_the_limit_to_10::paint_caps_the_list_when_changing_the_limit_to_10;
pub use check_buttons_desktop::paint_check_buttons_desktop;
pub use check_buttons_mobile::paint_check_buttons_mobile;
pub use check_buttons_tablet::paint_check_buttons_tablet;
pub use check_op_return_coinbase_tooltip::paint_check_op_return_coinbase_tooltip;
pub use check_op_return_tx_tooltip::paint_check_op_return_tx_tooltip;
pub use copy_buttons_exist_and_are_visible::paint_copy_buttons_exist_and_are_visible;
pub use correctly_formats_when_switching_from_jpy_back_to_usd::paint_correctly_formats_when_switching_from_jpy_back_to_usd;
pub use correctly_formats_when_switching_from_usd_to_jpy::paint_correctly_formats_when_switching_from_usd_to_jpy;
pub use desktop_rbf::{
    DesktopRbfMeasurement, DesktopRbfRect, desktop_rbf_transaction_markup,
    measure_desktop_rbf_transaction_paint,
};
pub use displays_jpy_fiat_values_without_decimals::paint_displays_jpy_fiat_values_without_decimals;
pub use displays_jpy_price_without_decimals_after_switching_currency::paint_displays_jpy_price_without_decimals_after_switching_currency;
pub use displays_jpy_values_with_yen_symbol::paint_displays_jpy_values_with_yen_symbol;
pub use displays_jpy_values_without_decimal_places::paint_displays_jpy_values_without_decimal_places;
pub use displays_usd_fiat_values_with_dollar_symbol::paint_displays_usd_fiat_values_with_dollar_symbol;
pub use displays_usd_price_with_dollar_symbol::paint_displays_usd_price_with_dollar_symbol;
pub use formats_all_jpy_values_correctly_without_decimals::paint_formats_all_jpy_values_correctly_without_decimals;
pub use graphs_empty_child_redirects_to_mempool::paint_graphs_empty_child_redirects_to_mempool;
pub use graphs_screen::graphs_screen_markup;
pub use invalid_blinding_hex::paint_invalid_blinding_data_hex_error;
pub use language_cookie::paint_language_cookie;
pub use limits_btc_to_8_decimals::paint_limits_btc_to_8_decimals;
pub use liquid::{
    asset_cards_markup, asset_confirmed_page_request, asset_confirmed_txs_request,
    asset_history_markup, asset_mempool_txs_request, asset_search_markup,
    asset_supply_decimal_request, asset_supply_request, liquid_asset_requests,
    registry_entry_markup, registry_entry_request, registry_list_markup, registry_list_request,
    registry_page_request, registry_search_markup, registry_search_request, supply_markup,
};
pub use load_first_mempool_block_after_skeleton_loads::paint_load_first_mempool_block_after_skeleton_loads;
pub use loads_first_blockchain_block_visible_and_keypress_arrow_left::paint_loads_first_blockchain_block_visible_and_keypress_arrow_left;
pub use loads_first_blockchain_block_visible_and_keypress_arrow_right::paint_loads_first_blockchain_block_visible_and_keypress_arrow_right;
pub use loads_first_blockchain_blocks_visible_and_click_on_the_arrow_right::paint_loads_first_blockchain_blocks_visible_and_click_on_the_arrow_right;
pub use loads_genesis_block_and_click_on_the_arrow_left::paint_loads_genesis_block_and_click_on_the_arrow_left;
pub use loads_genesis_block_and_keypress_arrow_left::paint_loads_genesis_block_and_keypress_arrow_left;
pub use loads_genesis_block_and_keypress_arrow_right::paint_loads_genesis_block_and_keypress_arrow_right;
pub use loads_peg_in_addresses::paint_loads_peg_in_addresses;
pub use loads_peg_in_addresses_bitcoin_tx_url::paint_loads_peg_in_addresses_bitcoin_tx_url;
pub use loads_peg_out_addresses::paint_loads_peg_out_addresses;
pub use loads_peg_out_addresses_bitcoin_address_url::paint_loads_peg_out_addresses_bitcoin_address_url;
pub use loads_the_blocks_page::paint_loads_the_blocks_page;
pub use loads_the_dashboard::paint_loads_the_dashboard;
pub use loads_the_dashboard_with_no_scrollbars_on_mobile::paint_loads_the_dashboard_with_no_scrollbars_on_mobile;
pub use loads_the_graphs_page::paint_loads_the_graphs_page;
pub use loads_the_graphs_page_mobile::paint_loads_the_graphs_page_mobile;
pub use loads_the_status_screen_backend_synchronizing::paint_loads_the_status_screen_backend_synchronizing;
pub use loads_the_status_screen_block::paint_loads_the_status_screen_block;
pub use loads_the_status_screen_blocks::paint_loads_the_status_screen_blocks;
pub use loads_the_status_screen_incoming_transactions::paint_loads_the_status_screen_incoming_transactions;
pub use loads_the_status_screen_mempool_size::paint_loads_the_status_screen_mempool_size;
pub use loads_the_status_screen_unconfirmed::paint_loads_the_status_screen_unconfirmed;
pub use loads_the_status_screen_vbytes_per_second::paint_loads_the_status_screen_vbytes_per_second;
pub use loads_the_status_screen_weight_units_per_second::paint_loads_the_status_screen_weight_units_per_second;
pub use local_pages::{
    calculator_fiat_text, capped_btc_text, docs_view_text, graphs_page_text, hashrate_page_text,
    jpy_fixture_text, lightning_page_text, mining_page_text, one_btc_sats_text,
    price_chart_page_text, statistics_page_text, switched_fiat_symbols, truncated_btc_text,
    usd_fixture_text,
};
pub use mempool_block_ordinal::paint_mempool_block_ordinal_title;
pub use models::{
    parse_address_stats_json, parse_block_json, parse_blocks_json, parse_blocks_tip_hash,
    parse_blocks_tip_height, parse_broadcast_result_json, parse_fee_estimates,
    parse_mempool_summary_json, parse_recent_transactions_json, parse_test_tx_results_json,
    parse_transaction_json, parse_transactions_json, parse_utxos_json, recent_txids,
};
pub use network_skeleton::paint_loads_skeleton_when_changes_between_networks;
pub use normalizes_comma_to_dot_in_fiat_input::paint_normalizes_comma_to_dot_in_fiat_input;
pub use official_nodes_status_url::paint_official_nodes_status_url;
pub use paths::{
    IndexerRequest, address_path, address_txs_path, block_path, block_txids_path, block_txs_path,
    broadcast_request, dashboard_paths, docs_api_type_page, mempool_paths, multi_address_requests,
    multi_address_requests_from_values, multi_address_screen_route, screen_routes,
    static_page_requests, static_page_routes, switcher_labels, test_transactions_request, tx_path,
    v1_client_requests,
};
pub use pause_icon_updates::paint_pauses_updates_when_clicking_the_pause_icon;
pub use pools_screen::pools_screen_markup;
pub use rbf_mobile_details::paint_mobile_rbf_replacement_alert;
pub use rbf_mobile_tracker::paint_mobile_rbf_tracker_alert;
pub use regular_tx_unblinding::regular_tx_unblinding_markup;
#[cfg(not(test))]
pub use rows::details_open_after_click;
pub use rows::{
    address_highlight_markup, address_rows_markup, asset_markup, batch_outspends_markup,
    block_details_markup, block_header_aside_markup, block_header_hex_markup, block_raw_beside,
    block_rows_markup, block_status_markup, block_transactions_markup, block_tx_page_path,
    block_txid_index_markup, block_txid_links_markup, blocks_rows_markup, broadcast_txid_markup,
    chain_tip_markup, confirmation_markup, dashboard_rows_markup, height_body_markup,
    indexed_bytes_markup, mempool_txids_markup, merkle_proof_markup, multi_address_links_markup,
    one_outspend_markup, outspends_markup, package_txid_markup, poison_alert_markup,
    recent_rows_markup, scripthash_markup, summary_rows_markup, summary_rows_with_status,
    test_txid_markup, tip_links_markup, transaction_hex_markup, transaction_row_markup,
    tx_raw_beside, tx_status_markup,
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
pub use show_empty_unblinded_tx::show_empty_unblinded_tx_markup;
pub use show_first_unblinded_vout::paint_show_first_unblinded_vout;
pub use show_invalid_error_unblinded_tx::paint_show_invalid_error_unblinded_tx;
pub use show_second_unblinded_vout::paint_show_second_unblinded_vout;
pub use show_unblinded_tx::show_unblinded_tx_markup;
pub use shows_calculator_form_after_price_feed_loads::paint_shows_calculator_form_after_price_feed_loads;
pub use shows_copy_buttons_for_each_input::paint_shows_copy_buttons_for_each_input;
pub use shows_input_labels_for_currency_btc_and_sats::paint_shows_input_labels_for_currency_btc_and_sats;
pub use shows_the_difficulty_adjustment_stats::paint_shows_the_difficulty_adjustment_stats;
pub use shows_the_hashrate_graph::paint_shows_the_hashrate_graph;
pub use shows_the_latest_adjustments::paint_shows_the_latest_adjustments;
pub use shows_the_latest_blocks::paint_shows_the_latest_blocks;
pub use shows_the_mempool_blocks::paint_shows_the_mempool_blocks;
pub use shows_the_mined_blocks::paint_shows_the_mined_blocks;
pub use shows_the_new_transaction_pill_when_there_are_new_transactions::paint_shows_the_new_transaction_pill_when_there_are_new_transactions;
pub use shows_the_new_transaction_pill_when_there_are_new_transactions::paint_shows_the_new_transaction_pill_when_there_are_new_transactions_and_scrolls_to_the_top_when_clicked;
pub use shows_the_pools_pie_chart::paint_shows_the_pools_pie_chart;
pub use shows_the_reward_stats_for_the_last_144_blocks::paint_shows_the_reward_stats_for_the_last_144_blocks;
pub use shows_warning_when_entering_21m_btc::paint_shows_warning_when_entering_21m_btc;
pub use signet_redirect::paint_signet_redirect;
pub use splora_frontend_shared::{assets_registry_path, assets_registry_search_path};
pub use strips_decimals_from_satoshis_input::paint_strips_decimals_from_satoshis_input;
pub use theme::{ACCENT, BACKGROUND, TEXT};
pub use theme_option_label::paint_theme_option_label;
pub use theme_preference_storage::paint_theme_preference_storage;
pub use timezone_option_label::paint_timezone_option_label;
pub use unblinded_tx_asset_page::unblinded_tx_asset_page_path;
pub use usd_decimal::paint_displays_usd_values_with_proper_decimal_format;
pub use usd_symbol::paint_displays_usd_values_with_correct_currency_symbol;
pub use views::{
    App, address_balance_history_markup, address_unspent_outputs_markup, asset_page_markup,
    block_page_heading_markup, block_page_transactions_markup, block_page_weight_markup,
    broadcast_transaction_heading, dashboard_loaded_markup, dashboard_mined_markup,
    dashboard_recent_blocks_markup, dashboard_tip_fees_text, first_mempool_block_link_markup,
    looks_like_block_hash, mempool_size_markup, mempool_summary_text,
    paint_supports_pagination_on_the_block_screen, rbf_page_socket_markup,
    shows_blocks_with_no_pagination_markup, shows_empty_blocks_properly_markup, submission_links,
    transaction_fees_heading,
};

/// Block transaction list for one viewport width.
///
/// The first `ul.pagination` child count is
/// `splora_frontend_shared::shows_blocks_pagination_with_5_pages_desktop`.
/// This paint does not hard-code that count.
pub fn block_transactions_markup_for_viewport(
    block_json: &str,
    txs: &str,
    page: u32,
    viewport_width: u32,
) -> String {
    let tx_count = tx_count_field(block_json);
    let pager = splora_frontend_shared::shows_blocks_pagination_with_5_pages_desktop(
        viewport_width,
        tx_count,
    );
    let mut items = String::new();
    for _ in 0..pager.first_pagination_children {
        items.push_str("<li></li>");
    }
    format!(
        "<div class=\"block-transactions\" data-page=\"{page}\">{txs}<div class=\"pagination-container\"><ul class=\"pagination\">{items}</ul></div></div>"
    )
}

fn tx_count_field(block_json: &str) -> u32 {
    const KEY: &str = "\"tx_count\":";
    let Some(at) = block_json.find(KEY) else {
        return 0;
    };
    let rest = block_json[at + KEY.len()..].trim_start();
    let digits = rest
        .chars()
        .take_while(|c| c.is_ascii_digit())
        .collect::<String>();
    digits.parse().unwrap_or(0)
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen::prelude::wasm_bindgen(start)]
pub fn start() {
    leptos::mount::mount_to_body(App);
}

#[cfg(test)]
#[test]
fn shows_the_latest_adjustments() {
    let host = paint_shows_the_latest_adjustments();
    assert_eq!(host, "difficulty-adjustments-table");

    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("src/shows_the_latest_adjustments.rs");
    let product =
        std::fs::read_to_string(&path).unwrap_or_else(|error| panic!("read {path:?}: {error}"));
    assert!(
        !has_difficulty_percent_or_numeric_difficulty(&product),
        "frontend/web/src/shows_the_latest_adjustments.rs must not contain a difficulty percent or a numeric difficulty"
    );
}

#[cfg(test)]
fn has_difficulty_percent_or_numeric_difficulty(source: &str) -> bool {
    source.contains('%') || source.bytes().any(|byte| byte.is_ascii_digit())
}

#[test]
fn loads_the_status_screen_incoming_transactions() {
    assert_eq!(
        paint_loads_the_status_screen_incoming_transactions(),
        "Incoming Transactions"
    );
}

#[test]
fn loads_the_status_screen_vbytes_per_second() {
    assert_eq!(paint_loads_the_status_screen_vbytes_per_second(), "vB/s");
}

#[test]
fn loads_the_status_screen_unconfirmed() {
    assert_eq!(paint_loads_the_status_screen_unconfirmed(), "Unconfirmed");
}

#[test]
fn loads_the_status_screen_mempool_size() {
    assert_eq!(paint_loads_the_status_screen_mempool_size(), "Mempool size");
}

#[test]
fn loads_the_status_screen_block() {
    assert_eq!(paint_loads_the_status_screen_block(), "block");
}

#[test]
fn loads_the_status_screen_blocks() {
    assert_eq!(paint_loads_the_status_screen_blocks(), "blocks");
}

#[test]
fn loads_the_status_screen_backend_synchronizing() {
    assert_eq!(
        paint_loads_the_status_screen_backend_synchronizing(),
        "Backend is synchronizing"
    );
}

#[test]
fn loads_the_status_screen_weight_units_per_second() {
    assert_eq!(
        paint_loads_the_status_screen_weight_units_per_second(),
        "WU/s"
    );
}

#[test]
fn show_invalid_error_unblinded_tx() {
    assert_eq!(
        paint_show_invalid_error_unblinded_tx(),
        "Error: Invalid blinding data."
    );
}

#[test]
fn loads_peg_in_addresses() {
    assert_eq!(paint_loads_peg_in_addresses(), "Peg-in");
}

#[test]
fn loads_peg_in_addresses_bitcoin_tx_url() {
    assert_eq!(
        paint_loads_peg_in_addresses_bitcoin_tx_url(
            "f148c0d854db4174ea420655235f910543f0ec3680566dcfdf84fb0a1697b592",
            0
        ),
        "https://mempool.space/tx/f148c0d854db4174ea420655235f910543f0ec3680566dcfdf84fb0a1697b592#vout=0"
    );
}

#[test]
fn loads_peg_out_addresses() {
    assert_eq!(paint_loads_peg_out_addresses(), "Liquid Peg Out");
}

#[test]
fn loads_peg_out_addresses_bitcoin_address_url() {
    assert_eq!(
        paint_loads_peg_out_addresses_bitcoin_address_url("1BxoGcMg14oaH3CwHD2hF4gU9VcfgX5yoR"),
        "https://mempool.space/address/1BxoGcMg14oaH3CwHD2hF4gU9VcfgX5yoR"
    );
}

#[test]
fn graphs_empty_child_redirects_to_mempool() {
    assert_eq!(paint_graphs_empty_child_redirects_to_mempool(), "mempool");
}

#[test]
fn details_open_after_click() {
    assert_eq!(rows::details_open_after_click(false), true);
    assert_eq!(rows::details_open_after_click(true), false);
}
