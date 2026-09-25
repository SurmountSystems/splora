//! Splora browser explorer.
//!
//! `frontend/splora-api` is the shared prefix table and NIP-98 header encoder.
//! This crate does not fork that client.

mod auth;
mod browser;
mod models;
mod paths;
mod screens;
mod theme;
mod views;

pub use auth::{
    ClientError, FAIL_CLOSED, MissingSigner, Nip98Signer, UnsignedNip98, browser_fetch_url,
    fetch_indexer, nip98_u_url, post_indexer, signer_from_presence, text_after_signed_get,
    unsigned_get,
};
pub use browser::browser_signer_present;
pub use models::{
    FeeEstimate, parse_blocks_tip_hash, parse_blocks_tip_height, parse_fee_estimates, recent_txids,
};
pub use paths::{
    IndexerRequest, address_path, address_txs_path, block_path, block_txids_path, block_txs_path,
    broadcast_request, dashboard_paths, docs_api_type_page, mempool_paths, multi_address_requests,
    multi_address_requests_from_values, multi_address_screen_route, screen_routes,
    static_page_requests, static_page_routes, switcher_labels, test_transactions_request, tx_path,
    v1_client_requests,
};
pub use screens::{
    address_fields_text, address_requests, block_fields_text, block_load_plan,
    blocks_list_fields_text, blocks_list_requests, broadcast_fields_text, broadcast_payload,
    broadcast_screen_request, dashboard_fields_text, dashboard_screen_requests, entered_addresses,
    multi_address_fields_text, multi_address_screen_requests, test_transactions_fields_text,
    test_transactions_payload, test_transactions_screen_request, transaction_fields_text,
    transaction_requests,
};
pub use theme::{ACCENT, BACKGROUND, TEXT};
pub use views::App;

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen::prelude::wasm_bindgen(start)]
pub fn start() {
    leptos::mount::mount_to_body(App);
}
