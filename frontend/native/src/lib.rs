//! Desktop explorer library. Tests do not open a window or touch the network.

pub mod api;
pub mod api_screen;
mod asset_page_path;
mod block_prev_next_links;
pub mod broadcast_route_request;
pub mod calculator_is_usable_on_desktop;
pub mod calculator_is_usable_on_mobile;
pub mod caps_the_list_when_changing_the_limit_to_10;
pub mod check_buttons_desktop;
pub mod check_buttons_mobile;
pub mod check_buttons_tablet;
pub mod copy_buttons_exist_and_are_visible;
pub mod correctly_formats_when_switching_from_jpy_back_to_usd;
pub mod correctly_formats_when_switching_from_usd_to_jpy;
pub mod desktop_tx_rbf;
pub mod details_open_after_click;
pub mod displays_jpy_fiat_values_without_decimals;
pub mod displays_jpy_price_without_decimals_after_switching_currency;
pub mod displays_jpy_values_with_yen_symbol;
pub mod displays_jpy_values_without_decimal_places;
pub mod displays_usd_fiat_values_with_dollar_symbol;
pub mod displays_usd_price_with_dollar_symbol;
pub mod docs_screen;
pub mod empty_block_screen;
pub mod formats_all_jpy_values_correctly_without_decimals;
pub mod graphs_empty_child_redirects_to_mempool;
pub mod graphs_screen;
mod invalid_blinding_hex;
pub mod keystore;
mod language_cookie;
pub mod limits_btc_to_8_decimals;
pub mod load_first_mempool_block_after_skeleton_loads;
pub mod loads_first_blockchain_block_visible_and_keypress_arrow_left;
pub mod loads_first_blockchain_block_visible_and_keypress_arrow_right;
pub mod loads_first_blockchain_blocks_visible_and_click_on_the_arrow_right;
pub mod loads_genesis_block_and_click_on_the_arrow_left;
pub mod loads_genesis_block_and_keypress_arrow_left;
pub mod loads_genesis_block_and_keypress_arrow_right;
pub mod loads_peg_in_addresses;
pub mod loads_peg_in_addresses_bitcoin_tx_url;
pub mod loads_peg_out_addresses;
pub mod loads_peg_out_addresses_bitcoin_address_url;
pub mod loads_the_blocks_page;
pub mod loads_the_dashboard;
pub mod loads_the_dashboard_with_no_scrollbars_on_mobile;
pub mod loads_the_graphs_page;
pub mod loads_the_graphs_page_mobile;
pub mod loads_the_status_screen_backend_synchronizing;
pub mod loads_the_status_screen_block;
pub mod loads_the_status_screen_blocks;
pub mod loads_the_status_screen_incoming_transactions;
pub mod loads_the_status_screen_mempool_size;
pub mod loads_the_status_screen_unconfirmed;
pub mod loads_the_status_screen_vbytes_per_second;
pub mod loads_the_status_screen_weight_units_per_second;
mod mempool_block_ordinal;
pub mod network_skeleton;
pub mod new_tx_pill;
pub mod no_pagination_block_screen;
pub mod normalizes_comma_to_dot_in_fiat_input;
mod official_nodes_status_url;
pub mod op_return_coinbase_tooltip;
pub mod op_return_tooltip;
pub mod pause_icon_updates;
pub mod pools_screen;
pub mod popup;
mod rbf_mobile_details;
mod rbf_mobile_tracker;
mod regular_tx_unblinding;
pub mod screens;
mod show_empty_unblinded_tx;
pub mod show_first_unblinded_vout;
pub mod show_invalid_error_unblinded_tx;
pub mod show_second_unblinded_vout;
mod show_unblinded_tx;
pub mod shows_blocks_pagination_with_5_pages_desktop;
pub mod shows_calculator_form_after_price_feed_loads;
pub mod shows_copy_buttons_for_each_input;
pub mod shows_input_labels_for_currency_btc_and_sats;
pub mod shows_the_difficulty_adjustment_stats;
pub mod shows_the_hashrate_graph;
pub mod shows_the_latest_adjustments;
pub mod shows_the_latest_blocks;
pub mod shows_the_mempool_blocks;
pub mod shows_the_mined_blocks;
pub mod shows_the_pools_pie_chart;
pub mod shows_the_reward_stats_for_the_last_144_blocks;
pub mod shows_warning_when_entering_21m_btc;
pub mod signer;
pub mod signet_redirect;
pub mod strips_decimals_from_satoshis_input;
pub mod supports_pagination_on_the_block_screen;
pub mod theme;
mod theme_option_label;
mod theme_preference_storage;
mod timezone_option_label;
pub mod updates_the_transaction_list_over_time;
pub mod usd_decimal;
pub mod usd_symbol;

pub use api::{
    ClientError, ClientRequest, ExplorerClient, ExplorerGet, ExplorerPost, UreqExplorer,
    ureq_client,
};
pub use api_screen::paint_loads_the_api_screen;
pub use asset_page_path::paint_asset_page_path;
pub use block_prev_next_links::paint_block_prev_next_links;
pub use broadcast_route_request::paint_broadcast_route_request;
pub use calculator_is_usable_on_desktop::paint_calculator_is_usable_on_desktop;
pub use calculator_is_usable_on_mobile::paint_calculator_is_usable_on_mobile;
pub use caps_the_list_when_changing_the_limit_to_10::paint_caps_the_list_when_changing_the_limit_to_10;
pub use check_buttons_desktop::paint_check_buttons_desktop;
pub use check_buttons_mobile::paint_check_buttons_mobile;
pub use check_buttons_tablet::paint_check_buttons_tablet;
pub use copy_buttons_exist_and_are_visible::paint_copy_buttons_exist_and_are_visible;
pub use correctly_formats_when_switching_from_jpy_back_to_usd::paint_correctly_formats_when_switching_from_jpy_back_to_usd;
pub use correctly_formats_when_switching_from_usd_to_jpy::paint_correctly_formats_when_switching_from_usd_to_jpy;
pub use desktop_tx_rbf::{DesktopTxRbfBounds, LaidOutRect, measure_desktop_transaction_rbf_bounds};
pub use details_open_after_click::paint_details_open_after_click;
pub use displays_jpy_fiat_values_without_decimals::paint_displays_jpy_fiat_values_without_decimals;
pub use displays_jpy_price_without_decimals_after_switching_currency::paint_displays_jpy_price_without_decimals_after_switching_currency;
pub use displays_jpy_values_with_yen_symbol::paint_displays_jpy_values_with_yen_symbol;
pub use displays_jpy_values_without_decimal_places::paint_displays_jpy_values_without_decimal_places;
pub use displays_usd_fiat_values_with_dollar_symbol::paint_displays_usd_fiat_values_with_dollar_symbol;
pub use displays_usd_price_with_dollar_symbol::paint_displays_usd_price_with_dollar_symbol;
pub use docs_screen::{DocsScreen, docs_screen_text};
pub use empty_block_screen::paint_shows_empty_blocks_properly;
pub use formats_all_jpy_values_correctly_without_decimals::paint_formats_all_jpy_values_correctly_without_decimals;
pub use graphs_empty_child_redirects_to_mempool::paint_graphs_empty_child_redirects_to_mempool;
pub use graphs_screen::{graphs_screen_lines, paint_loads_the_graphs_screen};
pub use invalid_blinding_hex::paint_invalid_blinding_data_hex_error;
pub use keystore::{KeyStoreError, SploraNamedKeyStore, launch_npub};
pub use language_cookie::paint_language_cookie;
pub use limits_btc_to_8_decimals::paint_limits_btc_to_8_decimals;
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
pub use mempool_block_ordinal::paint_mempool_block_ordinal_title;
pub use network_skeleton::paint_loads_skeleton_when_changes_between_networks;
pub use new_tx_pill::{
    paint_shows_the_new_transaction_pill_when_there_are_new_transactions,
    paint_shows_the_new_transaction_pill_when_there_are_new_transactions_and_scrolls_to_the_top_when_clicked,
};
pub use no_pagination_block_screen::paint_shows_blocks_with_no_pagination;
pub use normalizes_comma_to_dot_in_fiat_input::paint_normalizes_comma_to_dot_in_fiat_input;
pub use official_nodes_status_url::paint_official_nodes_status_url;
pub use op_return_coinbase_tooltip::paint_check_op_return_coinbase_tooltip;
pub use op_return_tooltip::paint_check_op_return_tx_tooltip;
pub use pause_icon_updates::paint_pauses_updates_when_clicking_the_pause_icon;
pub use pools_screen::pools_screen_lines;
pub use popup::{NpubPopupModel, PopupError, PopupWidgetKind};
pub use rbf_mobile_details::paint_mobile_rbf_replacement_alert;
pub use rbf_mobile_tracker::paint_mobile_rbf_tracker_alert;
pub use regular_tx_unblinding::regular_tx_unblinding_text;
pub use screens::{
    AddressScreen, BlockScreen, BlocksScreen, BroadcastScreen, Dashboard, DocPage, MempoolScreen,
    MultiAddressScreen, NetworkSwitcher, Screen, ScreenHost, StaticPage, StaticScreen,
    TestTransactionsScreen, TxScreen, known_doc_lines, network_label, notice_from_error,
    open_doc_route, painted_rows, painted_shell_text, resolve_doc_route, transaction_id_in_row,
};
pub use show_empty_unblinded_tx::show_empty_unblinded_tx_text;
pub use show_first_unblinded_vout::paint_show_first_unblinded_vout;
pub use show_invalid_error_unblinded_tx::paint_show_invalid_error_unblinded_tx;
pub use show_second_unblinded_vout::paint_show_second_unblinded_vout;
pub use show_unblinded_tx::show_unblinded_tx_text;
pub use shows_blocks_pagination_with_5_pages_desktop::paint_shows_blocks_pagination_with_5_pages_desktop;
pub use shows_calculator_form_after_price_feed_loads::paint_shows_calculator_form_after_price_feed_loads;
pub use shows_copy_buttons_for_each_input::paint_shows_copy_buttons_for_each_input;
pub use shows_input_labels_for_currency_btc_and_sats::paint_shows_input_labels_for_currency_btc_and_sats;
pub use shows_the_difficulty_adjustment_stats::paint_shows_the_difficulty_adjustment_stats;
pub use shows_the_hashrate_graph::paint_shows_the_hashrate_graph;
pub use shows_the_latest_adjustments::paint_shows_the_latest_adjustments;
pub use shows_the_latest_blocks::paint_shows_the_latest_blocks;
pub use shows_the_mempool_blocks::paint_shows_the_mempool_blocks;
pub use shows_the_mined_blocks::paint_shows_the_mined_blocks;
pub use shows_the_pools_pie_chart::paint_shows_the_pools_pie_chart;
pub use shows_the_reward_stats_for_the_last_144_blocks::paint_shows_the_reward_stats_for_the_last_144_blocks;
pub use shows_warning_when_entering_21m_btc::paint_shows_warning_when_entering_21m_btc;
pub use signer::{IndexerAuthCall, SignedAuth, SignerError, call_indexer_with_signer, sign_http};
pub use signet_redirect::paint_signet_redirect;
pub use strips_decimals_from_satoshis_input::paint_strips_decimals_from_satoshis_input;
pub use supports_pagination_on_the_block_screen::paint_supports_pagination_on_the_block_screen;
pub use theme::{ACCENT, ACCENT_RGB, BACKGROUND, BACKGROUND_RGB, TEXT, TEXT_RGB};
pub use theme_option_label::paint_theme_option_label;
pub use theme_preference_storage::paint_theme_preference_storage;
pub use timezone_option_label::paint_timezone_option_label;
pub use updates_the_transaction_list_over_time::paint_updates_the_transaction_list_over_time;
pub use usd_decimal::paint_displays_usd_values_with_proper_decimal_format;
pub use usd_symbol::paint_displays_usd_values_with_correct_currency_symbol;

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
        product.contains("splora_frontend_shared::shows_the_latest_adjustments"),
        "the host id is the shared call, not a second copy"
    );
    assert!(
        !product.contains("\"difficulty-adjustments-table\""),
        "the host id must not be stored again as a display amount"
    );
    assert!(
        !product.contains("Adjustments"),
        "the paint must not copy the heading Adjustments"
    );
    assert!(
        !has_difficulty_percent_or_numeric_difficulty(&product),
        "frontend/native/src/shows_the_latest_adjustments.rs must not contain a difficulty percent or a numeric difficulty"
    );
}

#[cfg(test)]
fn has_difficulty_percent_or_numeric_difficulty(source: &str) -> bool {
    source.contains('%') || source.bytes().any(|byte| byte.is_ascii_digit())
}

#[cfg(test)]
#[test]
fn loads_the_status_screen_incoming_transactions() {
    let host = paint_loads_the_status_screen_incoming_transactions();
    assert_eq!(host, "Incoming Transactions");

    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("src/loads_the_status_screen_incoming_transactions.rs");
    let product =
        std::fs::read_to_string(&path).unwrap_or_else(|error| panic!("read {path:?}: {error}"));
    assert!(
        product.contains("splora_frontend_shared::loads_the_status_screen_incoming_transactions"),
        "the label is the shared call, not a second copy"
    );
    assert!(
        !product.contains("\"Incoming Transactions\""),
        "the label must not be stored again as a display amount"
    );
}

#[cfg(test)]
#[test]
fn loads_the_status_screen_vbytes_per_second() {
    let host = paint_loads_the_status_screen_vbytes_per_second();
    assert_eq!(host, "vB/s");

    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("src/loads_the_status_screen_vbytes_per_second.rs");
    let product =
        std::fs::read_to_string(&path).unwrap_or_else(|error| panic!("read {path:?}: {error}"));
    assert!(
        product.contains("splora_frontend_shared::loads_the_status_screen_vbytes_per_second"),
        "the unit is the shared call, not a second copy"
    );
    assert!(
        !product.contains("\"vB/s\""),
        "the unit must not be stored again as a display amount"
    );
}

#[cfg(test)]
#[test]
fn loads_the_status_screen_unconfirmed() {
    let host = paint_loads_the_status_screen_unconfirmed();
    assert_eq!(host, "Unconfirmed");

    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("src/loads_the_status_screen_unconfirmed.rs");
    let product =
        std::fs::read_to_string(&path).unwrap_or_else(|error| panic!("read {path:?}: {error}"));
    assert!(
        product.contains("splora_frontend_shared::loads_the_status_screen_unconfirmed"),
        "the label is the shared call, not a second copy"
    );
    assert!(
        !product.contains("\"Unconfirmed\""),
        "the label must not be stored again as a display amount"
    );
}

#[cfg(test)]
#[test]
fn loads_the_status_screen_mempool_size() {
    let host = paint_loads_the_status_screen_mempool_size();
    assert_eq!(host, "Mempool size");

    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("src/loads_the_status_screen_mempool_size.rs");
    let product =
        std::fs::read_to_string(&path).unwrap_or_else(|error| panic!("read {path:?}: {error}"));
    assert!(
        product.contains("splora_frontend_shared::loads_the_status_screen_mempool_size"),
        "the label is the shared call, not a second copy"
    );
    assert!(
        !product.contains("\"Mempool size\""),
        "the label must not be stored again as a display amount"
    );
}

#[cfg(test)]
#[test]
fn loads_the_status_screen_blocks() {
    let host = paint_loads_the_status_screen_blocks();
    assert_eq!(host, "blocks");

    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("src/loads_the_status_screen_blocks.rs");
    let product =
        std::fs::read_to_string(&path).unwrap_or_else(|error| panic!("read {path:?}: {error}"));
    assert!(
        product.contains("splora_frontend_shared::loads_the_status_screen_blocks"),
        "the label is the shared call, not a second copy"
    );
    assert!(
        !product.contains("\"blocks\""),
        "the label must not be stored again as a display amount"
    );
}

#[cfg(test)]
#[test]
fn loads_the_status_screen_block() {
    let host = paint_loads_the_status_screen_block();
    assert_eq!(host, "block");

    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("src/loads_the_status_screen_block.rs");
    let product =
        std::fs::read_to_string(&path).unwrap_or_else(|error| panic!("read {path:?}: {error}"));
    assert!(
        product.contains("splora_frontend_shared::loads_the_status_screen_block()"),
        "the label is the shared call, not a second copy"
    );
    assert!(
        !product.contains("\"block\""),
        "the label must not be stored again as a display amount"
    );
}

#[cfg(test)]
#[test]
fn loads_the_status_screen_backend_synchronizing() {
    let host = paint_loads_the_status_screen_backend_synchronizing();
    assert_eq!(host, "Backend is synchronizing");

    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("src/loads_the_status_screen_backend_synchronizing.rs");
    let product =
        std::fs::read_to_string(&path).unwrap_or_else(|error| panic!("read {path:?}: {error}"));
    assert!(
        product.contains("splora_frontend_shared::loads_the_status_screen_backend_synchronizing()"),
        "the label is the shared call, not a second copy"
    );
    assert!(
        !product.contains("\"Backend is synchronizing\""),
        "the label must not be stored again as a display amount"
    );
}

#[cfg(test)]
#[test]
fn loads_the_status_screen_weight_units_per_second() {
    let host = paint_loads_the_status_screen_weight_units_per_second();
    assert_eq!(host, "WU/s");

    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("src/loads_the_status_screen_weight_units_per_second.rs");
    let product =
        std::fs::read_to_string(&path).unwrap_or_else(|error| panic!("read {path:?}: {error}"));
    assert!(
        product
            .contains("splora_frontend_shared::loads_the_status_screen_weight_units_per_second()"),
        "the unit is the shared call, not a second copy"
    );
    assert!(
        !product.contains("\"WU/s\""),
        "the unit must not be stored again as a display amount"
    );
}

#[cfg(test)]
#[test]
fn show_invalid_error_unblinded_tx() {
    let host = paint_show_invalid_error_unblinded_tx();
    assert_eq!(host, "Error: Invalid blinding data.");

    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("src/show_invalid_error_unblinded_tx.rs");
    let product =
        std::fs::read_to_string(&path).unwrap_or_else(|error| panic!("read {path:?}: {error}"));
    assert!(
        product.contains("splora_frontend_shared::show_invalid_error_unblinded_tx()"),
        "the sentence is the shared call, not a second copy"
    );
    assert!(
        !product.contains("Error: Invalid blinding data."),
        "the sentence must not be stored again in the paint file"
    );
}

#[cfg(test)]
#[test]
fn loads_peg_in_addresses() {
    let host = paint_loads_peg_in_addresses();
    assert_eq!(host, "Peg-in");

    let path =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src/loads_peg_in_addresses.rs");
    let product =
        std::fs::read_to_string(&path).unwrap_or_else(|error| panic!("read {path:?}: {error}"));
    assert!(
        product.contains("splora_frontend_shared::loads_peg_in_addresses()"),
        "the label is the shared call, not a second copy"
    );
    assert!(
        !product.contains("Peg-in"),
        "the label must not be stored again in the paint file"
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
fn graphs_empty_child_redirects_to_mempool() {
    assert_eq!(paint_graphs_empty_child_redirects_to_mempool(), "mempool");
}

#[test]
fn details_open_after_click() {
    assert_eq!(paint_details_open_after_click(false), true);
    assert_eq!(paint_details_open_after_click(true), false);
}
