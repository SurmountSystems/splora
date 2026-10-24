use crate::FAIL_CLOSED;
use crate::browser::{browser_signer_present, page_origin, signed_get};
use crate::liquid::{
    asset_cards_markup, asset_confirmed_page_request, asset_history_markup, liquid_asset_requests,
    registry_entry_markup, registry_entry_request, registry_list_markup, registry_list_request,
    registry_page_request, registry_search_markup, registry_search_request, supply_markup,
};
use crate::local_pages::{
    JPY_PER_BTC, USD_PER_BTC, calculator_fiat_text, capped_btc_text, graphs_page_text,
    hashrate_page_text, jpy_fixture_text, lightning_page_text, mining_page_text, one_btc_sats_text,
    price_chart_page_text, statistics_page_text, switched_fiat_symbols, truncated_btc_text,
    usd_fixture_text,
};
use crate::models::{
    parse_block_json, parse_blocks_json, parse_fee_estimates, parse_mempool_summary_json,
    parse_transaction_json, parse_transactions_json, parse_utxos_json,
};
use crate::paths::{docs_api_type_page, mempool_paths, static_page_requests, tx_path};
use crate::rows::{
    address_highlight_markup, address_load_error_markup, address_rows_markup,
    address_tx_of_heading, address_type_badge_markup, asset_markup, batch_outspends_markup,
    block_details_markup, block_header_aside_markup, block_header_hex_markup,
    block_load_error_markup, block_raw_beside, block_rows_markup, block_status_markup,
    block_transactions_markup, block_txid_index_markup, block_txid_links_markup,
    blocks_rows_markup, broadcast_txid_markup, chain_tip_markup, confirmation_markup,
    dashboard_rows_markup, details_open_after_click, effective_fee_rate_markup,
    features_label_markup, height_body_markup, indexed_bytes_markup, mempool_txids_markup,
    merkle_proof_markup, multi_address_links_markup, one_outspend_markup, outspends_markup,
    package_txid_markup, poison_alert_markup, recent_rows_markup, rows_with_notices,
    scripthash_markup, segwit_badge_markup, summary_rows_markup, summary_rows_with_status,
    taproot_badge_markup, test_txid_markup, tip_links_markup, transaction_hex_markup,
    transaction_row_markup, tx_raw_beside, tx_status_markup,
};
use crate::screens::{
    address_chain_request, address_fields_text, address_mempool_txs_request, address_requests,
    address_summary_page_request, address_summary_request, asset_requests, block_fields_text,
    block_header_request, block_load_plan_with_txids, block_raw_request, block_status_request,
    block_txid_index_request, block_txids_fields_text, block_txs_older_href,
    blocks_list_fields_text, blocks_list_requests, blocks_list_requests_at, blocks_older_href,
    broadcast_fields_text, broadcast_payload, broadcast_screen_request, broadcast_submission,
    dashboard_fields_text, dashboard_screen_requests, entered_addresses, indexer_failure_text,
    is_block_hash, mempool_txid_page_request, mempool_txids_request, multi_address_fields_text,
    multi_address_screen_requests, multi_address_summary_post, multi_address_txs_post,
    resolved_block_hash, test_transactions_fields_text, test_transactions_payload,
    test_transactions_screen_request, transaction_detail_requests, transaction_fields_text,
    transaction_requests, tx_merkleblock_proof_request, tx_outspend_index_request, tx_raw_request,
    tx_status_request, txs_outspends_request,
};
use crate::search::{plan_search_on, search_links_markup, search_loaded_markup};
use crate::theme::{ACCENT, BACKGROUND, TEXT};
use leptos::prelude::*;
use leptos_router::components::{A, Outlet, ParentRoute, Route, Router, Routes};
use leptos_router::hooks::{use_params_map, use_query_map};
use leptos_router::path;
use splora_api::Network;
use splora_frontend_shared::{
    FiatCode, address_path, address_txs_path, address_utxo_path, addresses_txs_path,
    block_height_path, block_path, block_txs_path, blocks_path, blocks_start_height_path,
    blocks_tip_hash_path, blocks_tip_height_path, broadcast_path, btc_fiat_jpy, btc_fiat_usd,
    btc_to_sats, calculator_field_names, cap_btc_supply, enter_btc, enter_fiat, fee_estimates_path,
    format_fiat, loaded_calculator, mempool_path, mempool_recent_path, parse_address_txs_summary,
    parse_blocks_tip_hash, parse_blocks_tip_height, route_availability, sats_fiat_usd, sats_to_btc,
    scripthash_txs_chain_page_path, scripthash_txs_mempool_path, scripthashes_txs_path,
    switch_usd_jpy_usd, test_txs_path, truncate_btc_decimals, usd_to_btc_and_sats,
};

fn shell_style() -> String {
    format!("background:{BACKGROUND};color:{TEXT};min-height:100vh;margin:0")
}

fn link_style() -> String {
    format!("color:{ACCENT}")
}

fn network_button_style(selected: bool) -> String {
    if selected {
        format!("background:{ACCENT};color:{BACKGROUND};border:1px solid {ACCENT}")
    } else {
        format!("background:{BACKGROUND};color:{TEXT};border:1px solid {ACCENT}")
    }
}

#[component]
fn Switcher() -> impl IntoView {
    let network = use_context::<RwSignal<Network>>().expect("network");
    let labels = [
        (Network::Mainnet, "mainnet"),
        (Network::Testnet3, "testnet3"),
        (Network::Testnet4, "testnet4"),
        (Network::Mutinynet, "mutinynet"),
        (Network::Liquid, "liquid"),
    ];
    view! {
        <nav>
            {labels
                .into_iter()
                .map(|(id, label)| {
                    let network = network;
                    view! {
                        <button
                            style=move || network_button_style(network.get() == id)
                            on:click=move |_| network.set(id)
                        >
                            {label}
                        </button>
                    }
                })
                .collect_view()}
            <A href="/"><span style=link_style()>"dashboard"</span></A>
            <A href="/blocks"><span style=link_style()>"blocks"</span></A>
            <A href="/mempool"><span style=link_style()>"mempool"</span></A>
            <A href="/widget/wallet"><span style=link_style()>"addresses"</span></A>
            <A href="/tx/push"><span style=link_style()>"broadcast"</span></A>
            <A href="/tx/test"><span style=link_style()>"test"</span></A>
            <A href="/terms-of-service"><span style=link_style()>"terms"</span></A>
            <A href="/privacy-policy"><span style=link_style()>"privacy"</span></A>
            <A href="/trademark-policy"><span style=link_style()>"trademark"</span></A>
            <A href="/docs"><span style=link_style()>"docs"</span></A>
            <A href="/docs/faq"><span style=link_style()>"faq"</span></A>
            <A href="/docs/api/rest"><span style=link_style()>"rest"</span></A>
            <A href="/docs/api/websocket"><span style=link_style()>"websocket"</span></A>
            {move || {
                (network.get() == Network::Liquid).then(|| {
                    view! {
                        <A href="/assets"><span style=link_style()>"assets"</span></A>
                        <A href="/assets/registry"><span style=link_style()>"registry"</span></A>
                    }
                })
            }}
        </nav>
    }
}

#[component]
fn FailClosed() -> impl IntoView {
    view! { <p>{FAIL_CLOSED}</p> }
}

#[cfg_attr(not(target_arch = "wasm32"), allow(dead_code))]
fn signed_text(result: Result<String, crate::ClientError>, method: &str, path: &str) -> String {
    match result {
        Ok(text) => text,
        Err(crate::ClientError::MissingSigner) => crate::FAIL_CLOSED.to_string(),
        Err(crate::ClientError::Indexer(sentence)) => indexer_failure_text(method, path, &sentence),
        Err(crate::ClientError::RejectedEvent) => format!("indexer error {method} {path}"),
    }
}

fn schedule_bytes(indexer_path: String, body: RwSignal<Vec<u8>>, network: RwSignal<Network>) {
    #[cfg(target_arch = "wasm32")]
    if browser_signer_present() {
        leptos::task::spawn_local(async move {
            let path = indexer_path;
            let result =
                crate::browser::signed_get_bytes(&page_origin(), network.get_untracked(), &path)
                    .await;
            let bytes = match result {
                Ok(bytes) => bytes,
                Err(err) => signed_text(Err(err), "GET", &path).into_bytes(),
            };
            body.set(bytes);
        });
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = (indexer_path, body, network);
    }
}

fn schedule(indexer_path: String, body: RwSignal<String>, network: RwSignal<Network>) {
    #[cfg(target_arch = "wasm32")]
    if browser_signer_present() {
        leptos::task::spawn_local(async move {
            let path = indexer_path;
            let result = signed_get(&page_origin(), network.get_untracked(), &path).await;
            body.set(signed_text(result, "GET", &path));
        });
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = (indexer_path, body, network);
    }
}

fn schedule_post(
    method: &'static str,
    indexer_path: String,
    payload: String,
    body: RwSignal<String>,
    network: RwSignal<Network>,
) {
    #[cfg(target_arch = "wasm32")]
    if browser_signer_present() {
        leptos::task::spawn_local(async move {
            let result = crate::browser::signed_post(
                &page_origin(),
                network.get_untracked(),
                method,
                &indexer_path,
                &payload,
            )
            .await;
            body.set(signed_text(result, method, &indexer_path));
        });
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = (method, indexer_path, payload, body, network);
    }
}

fn schedule_block(
    id: String,
    page_start: Option<u32>,
    initial: Vec<crate::paths::IndexerRequest>,
    network: RwSignal<Network>,
    block_body: RwSignal<String>,
    txs_body: RwSignal<String>,
    txids_body: RwSignal<String>,
    height_body: RwSignal<String>,
) {
    #[cfg(target_arch = "wasm32")]
    load_block_signed(
        id,
        page_start,
        initial,
        network,
        block_body,
        txs_body,
        txids_body,
        height_body,
    );
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = (
            id,
            page_start,
            initial,
            network,
            block_body,
            txs_body,
            txids_body,
            height_body,
        );
    }
}

#[cfg(target_arch = "wasm32")]
fn store_block_response(
    path: &str,
    text: String,
    height_text: &mut Option<String>,
    height_body: RwSignal<String>,
    block_body: RwSignal<String>,
    txs_body: RwSignal<String>,
    txids_body: RwSignal<String>,
) {
    if path.starts_with("/block-height/") {
        height_body.set(text.clone());
        *height_text = Some(text);
    } else if path.ends_with("/txids") {
        txids_body.set(text);
    } else if path.contains("/txs") {
        txs_body.set(text);
    } else {
        block_body.set(text);
    }
}

#[cfg(target_arch = "wasm32")]
fn load_block_signed(
    id: String,
    page_start: Option<u32>,
    initial: Vec<crate::IndexerRequest>,
    network: RwSignal<Network>,
    block_body: RwSignal<String>,
    txs_body: RwSignal<String>,
    txids_body: RwSignal<String>,
    height_body: RwSignal<String>,
) {
    if !browser_signer_present() {
        return;
    }
    leptos::task::spawn_local(async move {
        let mut height_text = None;
        for req in initial {
            let result = signed_get(&page_origin(), network.get_untracked(), &req.path).await;
            let text = signed_text(result, req.method, &req.path);
            store_block_response(
                &req.path,
                text,
                &mut height_text,
                height_body,
                block_body,
                txs_body,
                txids_body,
            );
        }
        if let Some(body) = height_text {
            let mut ignored_height = None;
            for req in block_load_plan_with_txids(&id, Some(&body), page_start) {
                let result = signed_get(&page_origin(), network.get_untracked(), &req.path).await;
                let text = signed_text(result, req.method, &req.path);
                store_block_response(
                    &req.path,
                    text,
                    &mut ignored_height,
                    height_body,
                    block_body,
                    txs_body,
                    txids_body,
                );
            }
        }
    });
}

fn schedule_row(path: String, rows: RwSignal<Vec<(String, String)>>, network: RwSignal<Network>) {
    #[cfg(target_arch = "wasm32")]
    if browser_signer_present() {
        leptos::task::spawn_local(async move {
            let result = signed_get(&page_origin(), network.get_untracked(), &path).await;
            let text = signed_text(result, "GET", &path);
            rows.update(|rows| {
                if let Some(slot) = rows.iter_mut().find(|(existing, _)| existing == &path) {
                    slot.1 = text;
                } else {
                    rows.push((path, text));
                }
            });
        });
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = (path, rows, network);
    }
}

fn load_entered(
    calls: Vec<crate::paths::IndexerRequest>,
    rows: RwSignal<Vec<(String, String)>>,
    network: RwSignal<Network>,
) {
    rows.set(
        calls
            .iter()
            .map(|call| (call.path.clone(), String::new()))
            .collect(),
    );
    for call in calls {
        schedule_row(call.path, rows, network);
    }
}

#[component]
fn SearchBox() -> impl IntoView {
    let network = use_context::<RwSignal<Network>>().expect("network");
    let draft = RwSignal::new(String::new());
    let query = RwSignal::new(String::new());
    let parts = RwSignal::new(Vec::new());
    view! {
        <label>
            "Search"
            <input on:input=move |event| draft.set(event_target_value(&event))/>
        </label>
        <button
            type="button"
            on:click=move |_| {
                let text = draft.get_untracked();
                query.set(text.clone());
                parts.set(Vec::new());
                let liquid = network.get_untracked() == Network::Liquid;
                if let Some(plan) = plan_search_on(&text, liquid) {
                    for req in plan.requests {
                        schedule_row(req.path, parts, network);
                    }
                }
            }
        >
            "Search"
        </button>
        <div inner_html=move || search_links_markup(&query.get())></div>
        <div inner_html=move || {
            search_loaded_markup(
                &query.get(),
                network.get() == Network::Liquid,
                &parts.get(),
            )
        }></div>
    }
}

#[component]
fn Shell() -> impl IntoView {
    view! {
        <div style=shell_style()>
            <h1>"Splora"</h1>
            <Switcher/>
            <SearchBox/>
            {(!browser_signer_present()).then(|| view! { <FailClosed/> })}
            <Outlet/>
        </div>
    }
}

fn push_notice(lines: &mut Vec<String>, notice: &str) {
    if !lines.iter().any(|line| line == notice) {
        lines.push(notice.to_string());
    }
}

pub fn looks_like_block_hash(value: &str) -> bool {
    is_block_hash(value)
}

fn last_seen_txid(body: &str) -> Option<String> {
    if let Ok(rows) = splora_frontend_shared::parse_address_txs_summary(body) {
        if let Some(row) = rows.last() {
            if !row.txid.is_empty() {
                return Some(row.txid.clone());
            }
        }
    }
    parse_transactions_json(body)
        .ok()
        .and_then(|txs| txs.last().map(|tx| tx.txid.clone()))
        .filter(|txid| !txid.is_empty())
}

fn history_markup(body: &str) -> String {
    let summary = summary_rows_markup(body);
    if !summary.is_empty() {
        return summary;
    }
    address_rows_markup("", body, "")
}

fn indexer_failure(body: &str) -> bool {
    body.lines().any(|line| {
        let line = line.trim();
        line.starts_with("indexer error ") && line.len() > "indexer error ".len()
    })
}

pub fn submission_links(path: &str, body: &str) -> String {
    if path == broadcast_path() {
        let mut html = package_txid_markup(body);
        let trimmed = body.trim();
        if !trimmed.starts_with('{')
            && !trimmed.starts_with('[')
            && !indexer_failure(trimmed)
            && !trimmed.contains(char::is_whitespace)
        {
            html.push_str(&broadcast_txid_markup(trimmed));
        }
        html
    } else if path == test_txs_path() {
        let markup = test_txid_markup(body);
        let rate = effective_fee_rate_markup(body);
        if rate.is_empty() {
            markup
        } else if let Some(rest) = markup.strip_prefix("<th class=\"allowed\">Allowed?</th>") {
            format!("<th class=\"allowed\">Allowed?</th>{rate}{rest}")
        } else {
            format!("{markup}{rate}")
        }
    } else {
        String::new()
    }
}

fn skip_closed_or_empty(body: &str, lines: &mut Vec<String>) -> bool {
    if body.trim().is_empty() {
        return true;
    }
    if let Some(line) = crate::screens::closed_notice(body) {
        push_notice(lines, line);
        return true;
    }
    false
}

/// Footer label from `footer.component.html`.
///
/// The words are Mempool size. The value is raw `vsize` from `GET /mempool`.
/// It is not kB or MB, and it is not a projected block count.
pub fn mempool_size_markup(body: &str) -> String {
    let Ok(stats) = parse_mempool_summary_json(body) else {
        return String::new();
    };
    format!(
        "<span class=\"mempoolSize\">Mempool size:</span>\nMempool size:\n<div class=\"sub-text\">{}</div>",
        stats.vsize
    )
}

pub fn mempool_summary_text(body: &str) -> String {
    let mut lines = Vec::new();
    if skip_closed_or_empty(body, &mut lines) {
        return lines.join("\n");
    }
    match parse_mempool_summary_json(body) {
        Ok(stats) => {
            // GPUI whole line. The count line stays next. The confirmation word stays off.
            lines.push("Unconfirmed:".to_string());
            lines.push(format!(
                "mempool count {count} vsize {vsize} total_fee {total_fee}",
                count = stats.count,
                vsize = stats.vsize,
                total_fee = stats.total_fee,
            ));
            for (rate, vsize) in stats.fee_histogram {
                lines.push(format!("fee_histogram {rate} {vsize}"));
            }
        }
        Err(_) => push_notice(&mut lines, "unreadable indexer response"),
    }
    lines.push("The incoming virtual-bytes-per-second rate is not in the payload.".to_string());
    lines.join("\n")
}

fn dashboard_tip_fees_text(tip_hash: &str, tip_height: &str, fees: &str) -> String {
    let mut lines = Vec::new();
    if !skip_closed_or_empty(tip_hash, &mut lines) {
        match parse_blocks_tip_hash(tip_hash) {
            Ok(hash) => lines.push(format!("tip hash {hash}")),
            Err(_) => push_notice(&mut lines, "unreadable indexer response"),
        }
    }
    if !skip_closed_or_empty(tip_height, &mut lines) {
        match parse_blocks_tip_height(tip_height) {
            Ok(height) => lines.push(format!("tip height {height}")),
            Err(_) => push_notice(&mut lines, "unreadable indexer response"),
        }
    }
    if !skip_closed_or_empty(fees, &mut lines) {
        match parse_fee_estimates(fees) {
            Ok(estimates) => {
                for (target, rate) in &estimates.0 {
                    lines.push(format!("fee target {target} rate {rate}"));
                }
            }
            Err(_) => push_notice(&mut lines, "unreadable indexer response"),
        }
    }
    lines.join("\n")
}

/// Fees card title from `dashboard.component.html` line 6.
/// Liquid keeps the fee estimates and does not paint this heading.
pub fn transaction_fees_heading(liquid: bool) -> String {
    if liquid {
        String::new()
    } else {
        let mut markup = String::from("<h2>Transaction Fees</h2>");
        // GPUI whole line. The h2 stays. Liquid returns above.
        markup.push('\n');
        markup.push_str("Transaction Fees");
        markup.push('\n');
        markup
    }
}

/// Dashboard column from `dashboard.component.html` beside `block.timestamp`.
///
/// Angular `th i18n="dashboard.latest-blocks.mined"` is the word Mined.
/// The value cell stays the raw timestamp. A body that does not parse, or
/// an empty list, leaves the rows unchanged. This is not a clock.
pub fn dashboard_mined_markup(rows: &str, blocks_json: &str) -> String {
    let Ok(blocks) = parse_blocks_json(blocks_json) else {
        return rows.to_string();
    };
    if blocks.is_empty() {
        return rows.to_string();
    }
    let needle = "<td class=\"time\">";
    let Some(at) = rows.find(needle) else {
        return rows.to_string();
    };
    let label = "<td class=\"table-cell-mined\">Mined</td>";
    let mut out = String::with_capacity(rows.len() + label.len());
    out.push_str(&rows[..at]);
    // GPUI whole line. The cell stays beside the raw timestamp.
    out.push('\n');
    out.push_str("Mined");
    out.push('\n');
    out.push_str(label);
    out.push_str(&rows[at..]);
    out
}

/// Dashboard heading from `dashboard.component.html` above the latest blocks.
///
/// Angular `h5 i18n="dashboard.recent-blocks"` is the words Recent Blocks.
/// The rows stay the block list from `GET /blocks`. A body that does not
/// parse, or an empty list, leaves the rows unchanged. This is not a clock.
pub fn dashboard_recent_blocks_markup(rows: &str, blocks_json: &str, paint_size: bool) -> String {
    let Ok(blocks) = parse_blocks_json(blocks_json) else {
        return rows.to_string();
    };
    if blocks.is_empty() {
        return rows.to_string();
    }
    let heading = "<h5>Recent Blocks</h5>";
    let mut out = String::with_capacity(heading.len() + rows.len());
    out.push_str(heading);
    // GPUI whole line. The heading tag stays. This line is the words alone.
    out.push_str("\nRecent Blocks\n");
    if paint_size {
        // GPUI whole line. The size value cell stays. Liquid does not get this line.
        out.push_str("\nSize\n");
    }
    out.push_str(rows);
    out
}

#[component]
fn Dashboard() -> impl IntoView {
    let network = use_context::<RwSignal<Network>>().expect("network");
    let blocks_body = RwSignal::new(String::new());
    let recent_body = RwSignal::new(String::new());
    let tip_hash_body = RwSignal::new(String::new());
    let tip_height_body = RwSignal::new(String::new());
    let fees_body = RwSignal::new(String::new());
    let mempool_body = RwSignal::new(String::new());
    let blocks = blocks_path();
    for req in dashboard_screen_requests() {
        if req.path == mempool_recent_path() {
            schedule(req.path, recent_body, network);
        } else if req.path == blocks {
            schedule(blocks.to_string(), blocks_body, network);
        } else {
            schedule(req.path, blocks_body, network);
        }
    }
    schedule(blocks_tip_hash_path().to_string(), tip_hash_body, network);
    schedule(
        blocks_tip_height_path().to_string(),
        tip_height_body,
        network,
    );
    schedule(fee_estimates_path().to_string(), fees_body, network);
    schedule(mempool_path().to_string(), mempool_body, network);
    view! {
        <h2>"Dashboard"</h2>
        <div inner_html=move || {
            let blocks = blocks_body.get();
            let recent = recent_body.get();
            rows_with_notices(
                &dashboard_fields_text(&blocks, &recent),
                &dashboard_recent_blocks_markup(
                    &dashboard_mined_markup(
                        &dashboard_rows_markup(&blocks, &recent),
                        &blocks,
                    ),
                    &blocks,
                    network.get() != Network::Liquid,
                ),
            )
        }></div>
        <div inner_html=move || {
            let hash = parse_blocks_tip_hash(&tip_hash_body.get()).unwrap_or_default();
            let height = parse_blocks_tip_height(&tip_height_body.get()).unwrap_or_default();
            if looks_like_block_hash(&hash) {
                chain_tip_markup(&hash, &height)
            } else {
                tip_links_markup(&hash, &height)
            }
        }></div>
        <div inner_html=move || transaction_fees_heading(network.get() == Network::Liquid)></div>
        <pre>{move || dashboard_tip_fees_text(&tip_hash_body.get(), &tip_height_body.get(), &fees_body.get())}</pre>
        <pre>{move || mempool_summary_text(&mempool_body.get())}</pre>
        <div inner_html=move || mempool_size_markup(&mempool_body.get())></div>
    }
}

#[component]
fn BlocksPage() -> impl IntoView {
    let network = use_context::<RwSignal<Network>>().expect("network");
    let query = use_query_map();
    let body = RwSignal::new(String::new());
    Effect::new(move |_| {
        let start_height = query.with(|map| {
            map.get("start_height")
                .and_then(|text| text.parse::<u64>().ok())
        });
        let calls = match start_height {
            Some(height) => blocks_list_requests_at(Some(height)),
            None => blocks_list_requests(),
        };
        for req in calls {
            let path = match start_height {
                Some(height) => {
                    let indexed = blocks_start_height_path(height);
                    if req.path == indexed {
                        indexed
                    } else {
                        req.path
                    }
                }
                None => {
                    let indexed = blocks_path().to_string();
                    if req.path == indexed {
                        indexed
                    } else {
                        req.path
                    }
                }
            };
            schedule(path, body, network);
        }
    });
    view! {
        <h1>"Blocks"</h1>
        <th class="txs">"Transactions"</th>
        <div inner_html=move || {
            blocks_older_href(&body.get())
                .map(|href| format!("<a href=\"{href}\">Older blocks</a>"))
                .unwrap_or_default()
        }></div>
        <div inner_html=move || {
            let blocks = body.get();
            rows_with_notices(
                &blocks_list_fields_text(&blocks),
                &blocks_rows_markup(&blocks),
            )
        }></div>
    }
}

/// One block-page `h1` from the loaded `GET /block/:hash` body.
///
/// Raw JSON `stale: true` is Stale Block, including height 0. A missing
/// `stale` key is not stale: height 0 is then Genesis, and any other parsed
/// height is Block. A body that does not parse has no height, so the title
/// word stays Block. Digits are the raw height. There is no thousands separator.
pub fn block_page_heading_markup(block_json: &str) -> String {
    let parsed = parse_block_json(block_json).ok();
    // Same read as the native block screen: the raw body, not `wire::Block`.
    let stale = serde_json::from_str::<serde_json::Value>(block_json)
        .ok()
        .and_then(|value| value.get("stale").and_then(|flag| flag.as_bool()))
        == Some(true);
    let title = if stale {
        "Stale Block"
    } else {
        match parsed.as_ref().map(|block| block.height) {
            Some(0) => "Genesis",
            _ => "Block",
        }
    };
    let mut markup = if let Some(block) = parsed {
        let id = &block.id;
        let height = block.height;
        format!("<h1>{title} <a class=\"block-link\" href=\"/block/{id}\">{height}</a></h1>")
    } else {
        format!("<h1>{title}</h1>")
    };
    // GPUI whole line. The h1 stays. Genesis and Stale Block do not use this line.
    if title == "Block" {
        markup.push_str("\nBlock\n");
    }
    markup
}

/// Block-details Size label beside the other detail rows.
///
/// Angular `td i18n="block.size"` is the word Size. The value cell is the
/// raw size integer. A body that does not parse leaves the details table
/// unchanged.
pub fn block_page_size_markup(details: &str, block_json: &str) -> String {
    let Ok(block) = parse_block_json(block_json) else {
        return details.to_string();
    };
    let row = format!("<tr><td>Size</td><td>{}</td></tr>", block.size);
    let Some(at) = details.rfind("</tbody>") else {
        return details.to_string();
    };
    let mut out = String::with_capacity(details.len() + row.len());
    out.push_str(&details[..at]);
    out.push_str(&row);
    out.push_str(&details[at..]);
    out
}

/// Block-details Weight label beside the other detail rows.
///
/// Angular `td i18n="block.weight"` is the word Weight. The value cell is
/// the raw weight integer. A body that does not parse leaves the details
/// table unchanged. Size is inserted first so the two labels stay in the
/// Angular order Size, then Weight.
pub fn block_page_weight_markup(details: &str, block_json: &str) -> String {
    let details = block_page_size_markup(details, block_json);
    let Ok(block) = parse_block_json(block_json) else {
        return details;
    };
    let row = format!("<tr><td>Weight</td><td>{}</td></tr>", block.weight);
    let Some(at) = details.rfind("</tbody>") else {
        return details;
    };
    let mut out = String::with_capacity(details.len() + row.len());
    out.push_str(&details[..at]);
    out.push_str(&row);
    // GPUI whole line. Closed details keep the cell and omit this line.
    if !details.contains("id=\"details\" hidden") {
        out.push_str("\nWeight\n");
    }
    out.push_str(&details[at..]);
    out
}

#[component]
fn BlockPage() -> impl IntoView {
    let params = use_params_map();
    let query = use_query_map();
    let network = use_context::<RwSignal<Network>>().expect("network");
    let block_body = RwSignal::new(String::new());
    let txs_body = RwSignal::new(String::new());
    let txids_body = RwSignal::new(String::new());
    let height_body = RwSignal::new(String::new());
    let status_body = RwSignal::new(String::new());
    let header_body = RwSignal::new(String::new());
    let raw_bytes = RwSignal::new(Vec::new());
    let index_body = RwSignal::new(String::new());
    let tx_index = RwSignal::new(0u64);
    let batch_spends = RwSignal::new(String::new());
    let details_open = RwSignal::new(false);
    Effect::new(move |_| {
        let id = params.with(|p| p.get("hash").unwrap_or_default());
        let page_start = query.with(|map| {
            map.get("start_index")
                .and_then(|text| text.parse::<u32>().ok())
        });
        let mut initial = block_load_plan_with_txids(&id, None, page_start);
        let trimmed = id.trim().to_string();
        if !trimmed.is_empty() && trimmed.bytes().all(|byte| byte.is_ascii_digit()) {
            if let Ok(height) = trimmed.parse::<u64>() {
                let height_path = block_height_path(height);
                for req in &mut initial {
                    if req.path == height_path {
                        req.path = height_path.clone();
                    }
                }
            }
        } else if !trimmed.is_empty() {
            let block = block_path(&trimmed);
            let txs = block_txs_path(&trimmed);
            for req in &mut initial {
                if req.path == block {
                    req.path = block.clone();
                } else if req.path == txs {
                    req.path = txs.clone();
                }
            }
        }
        schedule_block(
            id,
            page_start,
            initial,
            network,
            block_body,
            txs_body,
            txids_body,
            height_body,
        );
        if looks_like_block_hash(&trimmed) {
            let status = block_status_request(&trimmed);
            schedule(status.path, status_body, network);
        }
    });
    Effect::new(move |_| {
        let resolved = height_body.get();
        let hash = resolved.trim();
        if looks_like_block_hash(hash) {
            let status = block_status_request(hash);
            schedule(status.path, status_body, network);
        }
    });
    Effect::new(move |_| {
        let id = params.with(|p| p.get("hash").unwrap_or_default());
        let Some(hash) = resolved_block_hash(&id, &height_body.get(), &block_body.get()) else {
            return;
        };
        let header = block_header_request(&hash);
        schedule(header.path, header_body, network);
        let raw = block_raw_request(&hash);
        schedule_bytes(raw.path, raw_bytes, network);
        let index = block_txid_index_request(&hash, tx_index.get());
        schedule(index.path, index_body, network);
        if let Ok(txs) = parse_transactions_json(&txs_body.get()) {
            let ids: Vec<String> = txs.into_iter().take(50).map(|tx| tx.txid).collect();
            if let Some(req) = txs_outspends_request(&ids) {
                schedule(req.path, batch_spends, network);
            }
        }
    });
    view! {
        <div inner_html=move || block_page_heading_markup(&block_body.get())></div>
        <div inner_html=move || {
            block_header_hex_markup(
                &block_page_weight_markup(
                    &block_details_markup(&block_body.get(), details_open.get()),
                    &block_body.get(),
                ),
                &header_body.get(),
            )
        }></div>
        <button
            type="button"
            class="btn btn-outline-info"
            on:click=move |_| {
                details_open.update(|open| *open = details_open_after_click(*open));
            }
        >
            "Details"
        </button>
        <div inner_html=move || {
            let start = query
                .with(|map| {
                    map.get("start_index")
                        .and_then(|text| text.parse::<u32>().ok())
                })
                .unwrap_or(0);
            let page = start / 25 + 1;
            block_transactions_markup(&block_body.get(), &txs_body.get(), page)
        }></div>
        <div inner_html=move || block_status_markup(&status_body.get())></div>
        <div inner_html=move || {
            let id = params.with(|p| p.get("hash").unwrap_or_default());
            let start = query
                .with(|map| {
                    map.get("start_index")
                        .and_then(|text| text.parse::<u32>().ok())
                })
                .unwrap_or(0);
            resolved_block_hash(&id, &height_body.get(), &block_body.get())
                .and_then(|hash| block_txs_older_href(&hash, start))
                .map(|href| format!("<a href=\"{href}\">Older transactions</a>"))
                .unwrap_or_default()
        }></div>
        <div inner_html=move || {
            let block = block_body.get();
            let txs = txs_body.get();
            let height = height_body.get();
            let mut rows = rows_with_notices(
                &block_fields_text(&block, &txs),
                &block_rows_markup(&block, &txs),
            );
            let from_block = block_load_error_markup(&block);
            let err = if from_block.is_empty() {
                block_load_error_markup(&height)
            } else {
                from_block
            };
            if !err.is_empty() {
                let rest = rows.replacen(&err, "", 1);
                rows = format!("{err}{rest}");
            }
            block_raw_beside(&rows, &raw_bytes.get())
        }></div>
        <div inner_html=move || {
            let txids = txids_body.get();
            rows_with_notices(
                &block_txids_fields_text(&txids),
                &block_txid_links_markup(&txids),
            )
        }></div>
        <div inner_html=move || block_header_aside_markup(&header_body.get())></div>
        <label>
            "Transaction index"
            <input
                type="number"
                on:input=move |event| {
                    tx_index.set(event_target_value(&event).parse().unwrap_or(0));
                }
            />
        </label>
        <div inner_html=move || block_txid_index_markup(&index_body.get())></div>
        <div inner_html=move || batch_outspends_markup(&batch_spends.get())></div>
        <div inner_html=move || height_body_markup(&height_body.get())></div>
    }
}

#[component]
fn TransactionPage() -> impl IntoView {
    let params = use_params_map();
    let network = use_context::<RwSignal<Network>>().expect("network");
    let body = RwSignal::new(String::new());
    let spends_body = RwSignal::new(String::new());
    let proof_body = RwSignal::new(String::new());
    let hex_body = RwSignal::new(String::new());
    let raw_bytes = RwSignal::new(Vec::new());
    let merkleblock_body = RwSignal::new(String::new());
    let status_body = RwSignal::new(String::new());
    let tip_height_body = RwSignal::new(String::new());
    let spend_rows = RwSignal::new(Vec::new());
    let txid = params.with(|p| p.get("txid").unwrap_or_default());
    for req in transaction_requests(&txid) {
        let indexed = tx_path(txid.trim());
        let path = if req.path == indexed {
            indexed
        } else {
            req.path
        };
        schedule(path, body, network);
    }
    for req in transaction_detail_requests(&txid) {
        if req.path.ends_with("/hex") {
            schedule(req.path, hex_body, network);
        } else if req.path.ends_with("/merkle-proof") {
            schedule(req.path, proof_body, network);
        } else {
            schedule(req.path, spends_body, network);
        }
    }
    let raw = tx_raw_request(&txid);
    schedule_bytes(raw.path, raw_bytes, network);
    let proof = tx_merkleblock_proof_request(&txid);
    schedule(proof.path, merkleblock_body, network);
    let status = tx_status_request(&txid);
    schedule(status.path, status_body, network);
    schedule(
        blocks_tip_height_path().to_string(),
        tip_height_body,
        network,
    );
    Effect::new(move |_| {
        let txid = params.with(|p| p.get("txid").unwrap_or_default());
        let txid = txid.trim().to_string();
        if txid.is_empty() {
            return;
        }
        if let Ok(tx) = parse_transaction_json(&body.get()) {
            for (index, _) in tx.vout.iter().enumerate().take(50) {
                let req = tx_outspend_index_request(&tx.txid, index as u64);
                schedule_row(req.path, spend_rows, network);
            }
        } else {
            let req = tx_outspend_index_request(&txid, 0);
            schedule_row(req.path, spend_rows, network);
        }
    });
    view! {
        <h1>"Transaction"</h1>
        <div inner_html=move || {
            let name = match network.get() {
                Network::Mainnet => "mainnet",
                Network::Testnet3 => "testnet",
                Network::Testnet4 => "testnet4",
                Network::Mutinynet => "signet",
                Network::Liquid => "liquid",
            };
            poison_alert_markup(&body.get(), name)
        }></div>
        <div inner_html=move || {
            let tx = body.get();
            let name = match network.get() {
                Network::Mainnet => "mainnet",
                Network::Testnet3 => "testnet",
                Network::Testnet4 => "testnet4",
                Network::Mutinynet => "signet",
                Network::Liquid => "liquid",
            };
            let mut rows = rows_with_notices(
                &transaction_fields_text(&tx),
                &transaction_row_markup(&tx),
            );
            let mainnet_features = features_label_markup(&tx, "mainnet");
            if !mainnet_features.is_empty() {
                rows = rows.replacen(&mainnet_features, "", 1);
            }
            rows.push_str(&features_label_markup(&tx, name));
            let mainnet_badge = segwit_badge_markup(&tx, "mainnet");
            if !mainnet_badge.is_empty() {
                rows = rows.replacen(&mainnet_badge, "", 1);
            }
            rows.push_str(&segwit_badge_markup(&tx, name));
            let mainnet_taproot = taproot_badge_markup(&tx, "mainnet");
            if !mainnet_taproot.is_empty() {
                rows = rows.replacen(&mainnet_taproot, "", 1);
            }
            rows.push_str(&taproot_badge_markup(&tx, name));
            tx_raw_beside(&rows, &raw_bytes.get())
        }></div>
        <div inner_html=move || confirmation_markup(&body.get(), &tip_height_body.get())></div>
        <div inner_html=move || outspends_markup(&spends_body.get())></div>
        <div inner_html=move || merkle_proof_markup(&proof_body.get())></div>
        <div inner_html=move || transaction_hex_markup(&hex_body.get())></div>
        <div inner_html=move || indexed_bytes_markup(&merkleblock_body.get(), "merkleblock-proof")></div>
        <div inner_html=move || tx_status_markup(&status_body.get())></div>
        <div inner_html=move || {
            spend_rows
                .get()
                .iter()
                .map(|(_, spend)| one_outspend_markup(spend))
                .collect::<String>()
        }></div>
    }
}

fn opened_scripthash(script: &str) -> bool {
    let script = script.trim();
    script.len() == 64 && script.bytes().all(|byte| byte.is_ascii_hexdigit())
}

fn status_pairs(rows: &[(String, String)]) -> Vec<(String, String)> {
    rows.iter()
        .filter_map(|(path, body)| {
            let txid = path.strip_prefix("/tx/")?.strip_suffix("/status")?;
            Some((txid.to_string(), body.clone()))
        })
        .collect()
}

/// Address heading from `address.component.html` above the unspent rows.
///
/// Angular `h2 i18n="address.unspent-outputs"` is the words Unspent Outputs.
/// It is shown when the utxo list has more than two rows. Fewer rows, or a
/// body that does not parse, leaves the address markup unchanged.
pub fn address_unspent_outputs_markup(rows: &str, utxo_json: &str) -> String {
    let Ok(utxos) = parse_utxos_json(utxo_json) else {
        return rows.to_string();
    };
    if utxos.len() <= 2 {
        return rows.to_string();
    }
    let needle = "<table class=\"utxo-rows\">";
    let Some(at) = rows.find(needle) else {
        return rows.to_string();
    };
    let heading = "<h2>Unspent Outputs</h2>";
    let mut out = String::with_capacity(rows.len() + heading.len());
    out.push_str(&rows[..at]);
    out.push_str(heading);
    // GPUI whole line. The h2 stays. Two or fewer utxos return above.
    out.push('\n');
    out.push_str("Unspent Outputs");
    out.push('\n');
    out.push_str(&rows[at..]);
    out
}

/// Address heading from `address.component.html` above the transaction rows.
///
/// Angular `h2 i18n="address.balance-history"` is the words Balance History.
/// It is shown when the transaction list has more than two rows. Fewer rows,
/// or a body that does not parse, leaves the address markup unchanged.
pub fn address_balance_history_markup(rows: &str, txs_json: &str) -> String {
    let Ok(txs) = parse_transactions_json(txs_json) else {
        return rows.to_string();
    };
    if txs.len() <= 2 {
        return rows.to_string();
    }
    let needle = "<table class=\"tx-rows\">";
    let Some(at) = rows.find(needle) else {
        return rows.to_string();
    };
    let heading = "<h2>Balance History</h2>";
    let mut out = String::with_capacity(rows.len() + heading.len());
    out.push_str(&rows[..at]);
    out.push_str(heading);
    // GPUI whole line. The h2 stays. Two or fewer transactions return above.
    out.push('\n');
    out.push_str("Balance History");
    out.push('\n');
    out.push_str(&rows[at..]);
    out
}

#[component]
fn AddressPage() -> impl IntoView {
    let params = use_params_map();
    let network = use_context::<RwSignal<Network>>().expect("network");
    let stats_body = RwSignal::new(String::new());
    let txs_body = RwSignal::new(String::new());
    let utxo_body = RwSignal::new(String::new());
    let chain_body = RwSignal::new(String::new());
    let mempool_body = RwSignal::new(String::new());
    let summary_body = RwSignal::new(String::new());
    let status_rows = RwSignal::new(Vec::new());
    Effect::new(move |_| {
        let addr = params.with(|p| p.get("addr").unwrap_or_default());
        let script = addr.trim().to_string();
        for req in address_requests(&script) {
            if req.path == address_utxo_path(&script) || req.path.ends_with("/utxo") {
                schedule(req.path, utxo_body, network);
            } else if req.path.ends_with("/txs/mempool") {
                schedule(req.path, mempool_body, network);
            } else if req.path == address_txs_path(&script) || req.path.ends_with("/txs") {
                schedule(req.path, txs_body, network);
            } else if req.path == address_path(&script) {
                schedule(req.path, stats_body, network);
            } else {
                schedule(req.path, stats_body, network);
            }
        }
        let summary = address_summary_request(&script);
        schedule(summary.path, summary_body, network);
    });
    Effect::new(move |_| {
        let Ok(rows) = parse_address_txs_summary(&summary_body.get()) else {
            return;
        };
        for row in rows.into_iter().take(50) {
            let req = tx_status_request(&row.txid);
            schedule_row(req.path, status_rows, network);
        }
    });
    view! {
        <h1>"Address"</h1>
        <div inner_html=move || {
            let stats = stats_body.get();
            let txs = txs_body.get();
            let utxos = utxo_body.get();
            let addr = params.with(|p| p.get("addr").unwrap_or_default());
            let script_stats = scripthash_markup(&stats);
            let mut rows = if script_stats.is_empty() {
                address_rows_markup(&stats, &txs, &utxos)
            } else {
                let mut out = script_stats;
                out.push_str(&address_tx_of_heading(&stats, &txs));
                out.push_str(&address_rows_markup("", &txs, &utxos));
                out
            };
            let badge = address_type_badge_markup(&addr, &stats);
            if !badge.is_empty() && !rows.contains(badge.as_str()) {
                rows.push_str(&badge);
            }
            let mut painted = rows_with_notices(&address_fields_text(&stats, &txs, &utxos), &rows);
            let err = address_load_error_markup(&stats);
            if !err.is_empty() {
                let rest = painted.replacen(&err, "", 1);
                painted = format!("{err}{rest}");
            }
            let with_outputs = address_unspent_outputs_markup(&painted, &utxos);
            address_balance_history_markup(&with_outputs, &txs)
        }></div>
        <div inner_html=move || {
            let addr = params.with(|p| p.get("addr").unwrap_or_default());
            address_highlight_markup(&addr, &txs_body.get())
        }></div>
        <div inner_html=move || {
            summary_rows_with_status(&summary_body.get(), &status_pairs(&status_rows.get()))
        }></div>
        <button on:click=move |_| {
            let script = params.with(|p| p.get("addr").unwrap_or_default());
            let script = script.trim().to_string();
            let Some(txid) = last_seen_txid(&summary_body.get_untracked()) else {
                return;
            };
            let page = address_summary_page_request(&script, &txid);
            schedule(page.path, summary_body, network);
        }>"Older summary"</button>
        <button on:click=move |_| {
            let script = params.with(|p| p.get("addr").unwrap_or_default());
            let script = script.trim().to_string();
            let seen = chain_body.get_untracked();
            let source = if seen.trim().is_empty() {
                txs_body.get_untracked()
            } else {
                seen
            };
            let Some(txid) = last_seen_txid(&source) else {
                return;
            };
            if opened_scripthash(&script) {
                let hash = script.to_ascii_lowercase();
                schedule(
                    scripthash_txs_chain_page_path(&hash, &txid),
                    chain_body,
                    network,
                );
            } else {
                let req = address_chain_request(&script, &txid);
                schedule(req.path, chain_body, network);
            }
        }>"Older confirmed transactions"</button>
        <button on:click=move |_| {
            let script = params.with(|p| p.get("addr").unwrap_or_default());
            let script = script.trim().to_string();
            if opened_scripthash(&script) {
                let hash = script.to_ascii_lowercase();
                schedule(scripthash_txs_mempool_path(&hash), mempool_body, network);
            } else {
                let req = address_mempool_txs_request(&script);
                schedule(req.path, mempool_body, network);
            }
        }>"Mempool transactions"</button>
        <div inner_html=move || history_markup(&chain_body.get())></div>
        <div inner_html=move || history_markup(&mempool_body.get())></div>
    }
}

fn post_screen(
    method: &'static str,
    path: String,
    heading: &'static str,
    button_label: &'static str,
    prepare: fn(&str) -> String,
    render: fn(&str) -> String,
) -> impl IntoView {
    let network = use_context::<RwSignal<Network>>().expect("network");
    let response = RwSignal::new(String::new());
    let raw = RwSignal::new(String::new());
    let path_label = path.clone();
    let submit_path = path.clone();
    let link_path = path;
    view! {
        {(!heading.is_empty()).then(|| view! { <h2>{heading}</h2> })}
        <p>{method}" "{path_label}</p>
        <textarea on:input=move |event| raw.set(event_target_value(&event))></textarea>
        <button on:click=move |_| {
            let raw_text = raw.get_untracked();
            if submit_path == broadcast_path() {
                let submission = broadcast_submission(&raw_text);
                schedule_post(
                    submission.method,
                    submission.path.to_string(),
                    submission.body,
                    response,
                    network,
                );
            } else {
                let payload = prepare(&raw_text);
                schedule_post(method, submit_path.clone(), payload, response, network);
            }
        }>{button_label}</button>
        <pre>{move || render(&response.get())}</pre>
        <div inner_html=move || submission_links(&link_path, &response.get())></div>
    }
}

fn static_screen(route: &str, title: &str, copy: &str) -> impl IntoView + use<> {
    let _fetches = static_page_requests(route);
    let title = title.to_string();
    let copy = copy.to_string();
    view! {
        <h2>{title}</h2>
        <p>{copy}</p>
    }
}

const FAQ_COPY: &str = "Splora reads public chain data from the indexer. The dashboard shows the chain tip, the block list, fee estimates, and the mempool summary. Open a block, a transaction, or an address to read that one object. Broadcast sends the raw transaction you paste. Test transactions send the raw transactions you paste. Splora cannot reverse a broadcast. This FAQ is compiled into the page. It does not call the indexer.";

const REST_COPY: &str = "The explorer uses these indexer paths. GET /blocks/tip/hash, GET /blocks/tip/height, GET /blocks, GET /fee-estimates, and GET /mempool feed the dashboard. GET /block/:hash reads a block. GET /tx/:txid reads a transaction. GET /address/:script and GET /address/:script/txs read an address. GET /mempool/recent reads recent mempool transactions. POST /tx broadcasts one raw transaction. POST /txs/test submits test transactions. The browser prefixes the path with /api, /testnet/api, /testnet4/api, /mutinynet/api, or /liquid/api. This page is the reference. It does not call the indexer.";

const WEBSOCKET_COPY: &str = "The indexer serves a websocket at /api/v1/ws. This page does not open that socket. It does not subscribe to new blocks. The text is compiled into the page.";

const ELECTRS_COPY: &str = "Block, transaction, address, and mempool reads use the REST paths listed on the REST page. Broadcast is POST /tx. Test transactions are POST /txs/test. This page does not open an Electrum TCP connection, and it does not call the indexer.";

fn api_section_title(kind: &str) -> String {
    match kind {
        "rest" => "REST API".to_string(),
        "websocket" => "WebSocket API".to_string(),
        "electrs" => "Electrum-style HTTP API".to_string(),
        _ => "API docs".to_string(),
    }
}

fn api_section_copy(kind: &str) -> String {
    match kind {
        "rest" => REST_COPY.to_string(),
        "websocket" => WEBSOCKET_COPY.to_string(),
        "electrs" => ELECTRS_COPY.to_string(),
        _ => {
            if kind.is_empty() {
                "This API page is static text. It does not call the indexer, and it is not a proxy for blocks, transactions, or addresses.".to_string()
            } else {
                format!(
                    "There is no compiled API section named {kind}. This page is static text. It does not call the indexer, and it is not a proxy for blocks, transactions, or addresses."
                )
            }
        }
    }
}

#[component]
fn MultiAddressPage() -> impl IntoView {
    let query = use_query_map();
    let network = use_context::<RwSignal<Network>>().expect("network");
    let rows = RwSignal::new(Vec::new());
    let summary_body = RwSignal::new(String::new());
    let script_summary = RwSignal::new(String::new());
    let txs_body = RwSignal::new(String::new());
    let script_txs = RwSignal::new(String::new());
    let draft = RwSignal::new(String::new());
    let values = query.with(|map| map.get_all("addresses").unwrap_or_default());
    let scripts = values
        .iter()
        .flat_map(|value| entered_addresses(value))
        .collect::<Vec<_>>();
    let load = move |values: Vec<String>| {
        let calls = multi_address_screen_requests(&values)
            .into_iter()
            .map(|mut call| {
                if let Some(path) = values
                    .iter()
                    .map(|script| address_path(script))
                    .find(|path| path == &call.path)
                {
                    call.path = path;
                }
                call
            })
            .collect();
        load_entered(calls, rows, network);
        summary_body.set(String::new());
        script_summary.set(String::new());
        txs_body.set(String::new());
        script_txs.set(String::new());
        for post in multi_address_txs_post(&values) {
            if post.path == addresses_txs_path() {
                schedule_post(
                    post.method,
                    post.path.to_string(),
                    post.body,
                    txs_body,
                    network,
                );
            } else if post.path == scripthashes_txs_path() {
                schedule_post(
                    post.method,
                    post.path.to_string(),
                    post.body,
                    script_txs,
                    network,
                );
            }
        }
        for post in multi_address_summary_post(&values) {
            let target = if post.path.contains("scripthash") {
                script_summary
            } else {
                summary_body
            };
            schedule_post(
                post.method,
                post.path.to_string(),
                post.body,
                target,
                network,
            );
        }
    };
    load(scripts);
    view! {
        <h2>"Addresses"</h2>
        <textarea on:input=move |event| draft.set(event_target_value(&event))></textarea>
        <button on:click=move |_| {
            load(entered_addresses(&draft.get_untracked()));
        }>"Load addresses"</button>
        <pre>{move || multi_address_fields_text(&rows.get())}</pre>
        <div inner_html=move || multi_address_links_markup(&rows.get())></div>
        <div inner_html=move || summary_rows_markup(&summary_body.get())></div>
        <div inner_html=move || summary_rows_markup(&script_summary.get())></div>
        <div inner_html=move || address_rows_markup("", &txs_body.get(), "")></div>
        <div inner_html=move || address_rows_markup("", &script_txs.get(), "")></div>
    }
}

/// Broadcast form title from `push-transaction.component.html`.
///
/// The h1 stays. GPUI also requires one line whose entire text is
/// Broadcast Transaction. A failed POST body does not use this function.
pub fn broadcast_transaction_heading() -> String {
    let mut markup = String::from("<h1>Broadcast Transaction</h1>");
    // GPUI whole line. The h1 stays.
    markup.push('\n');
    markup.push_str("Broadcast Transaction");
    markup.push('\n');
    markup
}

#[component]
fn BroadcastPage() -> impl IntoView {
    let call = broadcast_screen_request();
    let path = if call.path == broadcast_path() {
        call.path
    } else {
        broadcast_path().to_string()
    };
    let heading = broadcast_transaction_heading();
    view! {
        <div inner_html=heading></div>
        <label>"Maximum burn amount (sats)"</label>
        {post_screen(
            call.method,
            path,
            "",
            "Broadcast",
            broadcast_payload,
            broadcast_fields_text,
        )}
    }
}

#[component]
fn TestTransactionsPage() -> impl IntoView {
    let call = test_transactions_screen_request();
    let path = if call.path == test_txs_path() {
        call.path
    } else {
        test_txs_path().to_string()
    };
    post_screen(
        call.method,
        path,
        "Test transactions",
        "Submit test transaction",
        test_transactions_payload,
        test_transactions_fields_text,
    )
}

#[component]
fn TermsPage() -> impl IntoView {
    static_screen(
        "/terms-of-service",
        "Terms of service",
        "Splora shows public chain data from the indexer. You use this site at your own risk. This page is not financial, legal, or tax advice. A broadcast sends a transaction to the network, and Splora cannot pull it back.",
    )
}

#[component]
fn PrivacyPage() -> impl IntoView {
    static_screen(
        "/privacy-policy",
        "Privacy policy",
        "Splora does not create an account. When a NIP-07 signer is present, the browser signs indexer requests. Splora does not ask for a secret key. Text you type into a form stays in the page until you leave it. This page does not call the indexer.",
    )
}

#[component]
fn TrademarkPage() -> impl IntoView {
    static_screen(
        "/trademark-policy",
        "Trademark policy",
        "Splora is a Surmount Systems block explorer. Bitcoin, Liquid, and other project names belong to their owners. Do not use those names to claim that Splora is endorsed by them.",
    )
}

#[component]
fn DocsPage() -> impl IntoView {
    static_screen(
        "/docs",
        "Docs",
        "The block page reads GET /block/:hash. The block list reads GET /blocks. An address reads GET /address/:script. The multi-address page reads the addresses query key and sends one GET /address/:script for each address. Broadcast sends POST /tx and no other broadcast path. Test transactions send POST /txs/test. Terms, privacy, trademark, and this page are static. They do not fetch the indexer for the page body.",
    )
}

#[component]
fn DocsLayout() -> impl IntoView {
    view! { <Outlet/> }
}

#[component]
fn FaqPage() -> impl IntoView {
    static_screen("/docs/faq", "FAQ", FAQ_COPY)
}

#[component]
fn RestApiPage() -> impl IntoView {
    static_screen("/docs/api/rest", "REST API", REST_COPY)
}

#[component]
fn ApiTypePage() -> impl IntoView {
    let params = use_params_map();
    let kind = params.with(|map| map.get("type").unwrap_or_default());
    let _pattern = static_page_requests("/docs/api/:type");
    let route = docs_api_type_page(&kind).unwrap_or_else(|| "/docs/api/:type".to_string());
    let title = api_section_title(&kind);
    let copy = api_section_copy(&kind);
    static_screen(&route, &title, &copy)
}

#[component]
fn MempoolPage() -> impl IntoView {
    let network = use_context::<RwSignal<Network>>().expect("network");
    let body = RwSignal::new(String::new());
    let recent = RwSignal::new(String::new());
    let txids_body = RwSignal::new(String::new());
    let paths = mempool_paths();
    schedule(paths[0].to_string(), body, network);
    schedule(paths[1].to_string(), recent, network);
    let txids = mempool_txids_request();
    schedule(txids.path, txids_body, network);
    view! {
        <h2>"Mempool"</h2>
        <ul>
            {mempool_paths()
                .into_iter()
                .map(|path| view! { <li>{path}</li> })
                .collect_view()}
        </ul>
        <pre>{move || mempool_summary_text(&body.get())}</pre>
        <div inner_html=move || mempool_size_markup(&body.get())></div>
        <div inner_html=move || recent_rows_markup(&recent.get())></div>
        <button on:click=move |_| {
            let Ok(ids) = splora_frontend_shared::parse_mempool_txids(&txids_body.get_untracked())
            else {
                return;
            };
            let Some(last) = ids.last() else {
                return;
            };
            let page = mempool_txid_page_request(last);
            schedule(page.path, txids_body, network);
        }>"Older mempool transactions"</button>
        <div inner_html=move || mempool_txids_markup(&txids_body.get())></div>
    }
}

#[component]
fn CalculatorPage() -> impl IntoView {
    let sats_from_shared = btc_to_sats("1")
        .map(|sats| sats.to_string())
        .unwrap_or_default();
    let truncated_from_shared = truncate_btc_decimals("1.123456789");
    let capped_from_shared = cap_btc_supply("25000000").btc;
    let sats = one_btc_sats_text();
    let truncated = truncated_btc_text("1.123456789");
    let capped = capped_btc_text("25000000");
    let third_btc_usd = btc_fiat_usd("0.33");
    let one_sat_usd = btc_fiat_usd("0.00000001");
    let one_btc_jpy = btc_fiat_jpy("1");
    let ten_thousand_sats_usd = sats_fiat_usd(10_000);
    let fiat_lines = calculator_fiat_text();
    let prices = route_availability("/v1/prices");
    let fiat_code = RwSignal::new(FiatCode::Usd);
    let one_btc_usd = btc_fiat_usd("1");
    let fiat_in = RwSignal::new(String::new());
    let btc_in = RwSignal::new(String::new());
    let sats_in = RwSignal::new(String::new());
    let [fiat_name, bitcoin_name, satoshis_name] = calculator_field_names();
    view! {
        <h2>"Calculator"</h2>
        <p>"₿"</p>
        <p>{sats_from_shared}</p>
        <p>{truncated_from_shared}</p>
        <p>{capped_from_shared}</p>
        <p>{sats}</p>
        <p>{truncated}</p>
        <p>{capped}</p>
        <p>{third_btc_usd}</p>
        <p>{one_sat_usd}</p>
        <p>{one_btc_jpy}</p>
        <p>{ten_thousand_sats_usd}</p>
        <p>{fiat_lines}</p>
        <p>{prices}</p>
        <p>"A live price feed is not served."</p>
        <select on:change=move |event| {
            let code = if event_target_value(&event) == "JPY" {
                FiatCode::Jpy
            } else {
                FiatCode::Usd
            };
            fiat_code.set(code);
            let next = enter_btc(&loaded_calculator(code), &btc_in.get());
            fiat_in.set(next.fiat);
            btc_in.set(next.btc);
            sats_in.set(next.sats);
        }>
            <option value="USD">"USD"</option>
            <option value="JPY">"JPY"</option>
        </select>
        <p>{move || loaded_calculator(fiat_code.get()).symbol}</p>
        <p>{one_btc_usd}</p>
        <label>
            "fiat"
            <input
                name=fiat_name
                prop:value=move || fiat_in.get()
                on:input=move |event| {
                    let amount = event_target_value(&event);
                    let form = loaded_calculator(fiat_code.get());
                    if let Some(next) = enter_fiat(&form, &amount) {
                        fiat_in.set(next.fiat);
                        btc_in.set(next.btc);
                        sats_in.set(next.sats);
                    } else {
                        fiat_in.set(amount);
                        btc_in.set(String::new());
                        sats_in.set(String::new());
                    }
                }
            />
            <button type="button" class="copy-control">"Copy"</button>
        </label>
        <p>{move || {
            usd_to_btc_and_sats(&fiat_in.get())
                .map(|quote| quote.sats.to_string())
                .unwrap_or_default()
        }}</p>
        <label>
            "BTC"
            <input
                name=bitcoin_name
                prop:value=move || btc_in.get()
                on:input=move |event| {
                    let amount = event_target_value(&event);
                    let next = enter_btc(&loaded_calculator(fiat_code.get()), &amount);
                    fiat_in.set(next.fiat);
                    btc_in.set(next.btc);
                    sats_in.set(next.sats);
                }
            />
            <button type="button" class="copy-control">"Copy"</button>
        </label>
        <p>{move || btc_fiat_usd(&btc_in.get())}</p>
        <p>{move || btc_fiat_jpy(&btc_in.get())}</p>
        <label>
            "sats"
            <input
                name=satoshis_name
                prop:value=move || sats_in.get()
                on:input=move |event| {
                    let amount = event_target_value(&event);
                    let parsed = amount.parse::<u64>().unwrap_or(0);
                    let next = enter_btc(&loaded_calculator(fiat_code.get()), &sats_to_btc(parsed));
                    fiat_in.set(next.fiat);
                    btc_in.set(next.btc);
                    sats_in.set(next.sats);
                }
            />
            <button type="button" class="copy-control">"Copy"</button>
        </label>
        <p>{move || {
            let parsed = sats_in.get().parse::<u64>().unwrap_or(0);
            sats_fiat_usd(parsed)
        }}</p>
    }
}

#[component]
fn FiatFormatPage() -> impl IntoView {
    let usd = format_fiat(FiatCode::Usd, USD_PER_BTC);
    let jpy = format_fiat(FiatCode::Jpy, JPY_PER_BTC);
    let switched = switch_usd_jpy_usd("1500");
    let usd_again = switched[0].clone();
    let jpy_switched = switched[1].clone();
    let usd_restored = switched[2].clone();
    let usd_fixture = usd_fixture_text();
    let jpy_fixture = jpy_fixture_text();
    let symbols = switched_fiat_symbols();
    let symbol_line = format!("{} {} {}", symbols[0], symbols[1], symbols[2]);
    view! {
        <h1>"Fiat"</h1>
        <p>{usd}</p>
        <p>{jpy}</p>
        <p>{usd_again}</p>
        <p>{jpy_switched}</p>
        <p>{usd_restored}</p>
        <p>{usd_fixture}</p>
        <p>{jpy_fixture}</p>
        <p>{symbol_line}</p>
    }
}

#[component]
fn StatisticsPage() -> impl IntoView {
    let availability = route_availability("/api/v1/statistics");
    let page = statistics_page_text();
    view! {
        <h1>"Statistics"</h1>
        <p>{availability}</p>
        <p>{page}</p>
    }
}

#[component]
fn MiningPage() -> impl IntoView {
    let availability = route_availability("/v1/mining");
    let page = mining_page_text();
    view! {
        <h1>"Mining"</h1>
        <p>{availability}</p>
        <p>{page}</p>
    }
}

#[component]
fn LightningPage() -> impl IntoView {
    let availability = route_availability("/lightning");
    let page = lightning_page_text();
    view! {
        <h1>"Lightning"</h1>
        <p>{availability}</p>
        <p>{page}</p>
    }
}

#[component]
fn GraphsPage() -> impl IntoView {
    let availability = route_availability("/api/v1/statistics");
    let page = graphs_page_text();
    view! {
        <h1>"Graphs"</h1>
        <p>{availability}</p>
        <p>{page}</p>
    }
}

#[component]
fn HashratePage() -> impl IntoView {
    let availability = route_availability("/v1/mining");
    let page = hashrate_page_text();
    view! {
        <h1>"Hashrate"</h1>
        <p>{availability}</p>
        <p>{page}</p>
    }
}

#[component]
fn PriceChartPage() -> impl IntoView {
    let availability = route_availability("/api/v1/historical-price");
    let page = price_chart_page_text();
    view! {
        <h1>"Price chart"</h1>
        <p>{availability}</p>
        <p>{page}</p>
    }
}

/// `/assets/asset/{id}` is the Cypress asset page. `/asset/{id}` is the short route.
/// Any other path does not select it. The string is `asset_markup` for that id.
pub fn asset_page_markup(path: &str, asset_json: &str, txs_json: &str) -> Option<String> {
    let id = asset_id_from_page_path(path)?;
    let value: serde_json::Value = serde_json::from_str(asset_json).ok()?;
    let asset_id = value.get("asset_id").and_then(|item| item.as_str())?;
    if asset_id != id {
        return None;
    }
    Some(asset_markup(asset_json, txs_json, true))
}

fn asset_id_from_page_path(path: &str) -> Option<&str> {
    let path = path.split(['?', '#']).next().unwrap_or(path);
    let path = path.trim_end_matches('/');
    let rest = if let Some(rest) = path.strip_prefix("/assets/asset/") {
        rest
    } else if let Some(rest) = path.strip_prefix("/asset/") {
        rest
    } else {
        return None;
    };
    if rest.is_empty() || rest.contains('/') {
        None
    } else {
        Some(rest)
    }
}

#[component]
fn AssetPage() -> impl IntoView {
    let params = use_params_map();
    let network = use_context::<RwSignal<Network>>().expect("network");
    let asset_body = RwSignal::new(String::new());
    let txs_body = RwSignal::new(String::new());
    let chain_body = RwSignal::new(String::new());
    let mempool_body = RwSignal::new(String::new());
    let supply_body = RwSignal::new(String::new());
    let decimal_body = RwSignal::new(String::new());
    let entry_body = RwSignal::new(String::new());
    Effect::new(move |_| {
        let liquid = network.get() == Network::Liquid;
        let id = params.with(|p| p.get("asset").unwrap_or_default());
        asset_body.set(String::new());
        txs_body.set(String::new());
        chain_body.set(String::new());
        mempool_body.set(String::new());
        supply_body.set(String::new());
        decimal_body.set(String::new());
        entry_body.set(String::new());
        for req in asset_requests(liquid, &id) {
            if req.path.ends_with("/txs") {
                schedule(req.path, txs_body, network);
            } else {
                schedule(req.path, asset_body, network);
            }
        }
        for req in liquid_asset_requests(liquid, &id) {
            if req.path.ends_with("/txs/chain") {
                schedule(req.path, chain_body, network);
            } else if req.path.ends_with("/txs/mempool") {
                schedule(req.path, mempool_body, network);
            } else if req.path.ends_with("/supply/decimal") {
                schedule(req.path, decimal_body, network);
            } else if req.path.ends_with("/supply") {
                schedule(req.path, supply_body, network);
            } else if req.path.contains("/assets/registry/") {
                schedule(req.path, entry_body, network);
            }
        }
    });
    view! {
        <div inner_html=move || {
            asset_markup(
                &asset_body.get(),
                &txs_body.get(),
                network.get() == Network::Liquid,
            )
        }></div>
        <div inner_html=move || registry_entry_markup(&entry_body.get(), network.get() == Network::Liquid)></div>
        <div inner_html=move || asset_history_markup(&chain_body.get(), network.get() == Network::Liquid)></div>
        <div inner_html=move || asset_history_markup(&mempool_body.get(), network.get() == Network::Liquid)></div>
        <div inner_html=move || supply_markup(&supply_body.get(), network.get() == Network::Liquid)></div>
        <div inner_html=move || supply_markup(&decimal_body.get(), network.get() == Network::Liquid)></div>
        <button on:click=move |_| {
            let liquid = network.get_untracked() == Network::Liquid;
            let id = params.with(|p| p.get("asset").unwrap_or_default());
            let Some(last) = last_seen_txid(&chain_body.get_untracked()) else {
                return;
            };
            if let Some(req) = asset_confirmed_page_request(liquid, &id, &last) {
                schedule(req.path, chain_body, network);
            }
        }>"Older asset transactions"</button>
    }
}

#[component]
fn AssetsPage() -> impl IntoView {
    let network = use_context::<RwSignal<Network>>().expect("network");
    let list_body = RwSignal::new(String::new());
    let later_body = RwSignal::new(String::new());
    Effect::new(move |_| {
        let liquid = network.get() == Network::Liquid;
        list_body.set(String::new());
        if let Some(req) = registry_list_request(liquid) {
            schedule(req.path, list_body, network);
        }
    });
    view! {
        <h1>"Assets"</h1>
        <div class="container-xl" inner_html=move || {
            asset_cards_markup(&list_body.get(), network.get() == Network::Liquid)
                .unwrap_or_default()
        }></div>
        <button on:click=move |_| {
            let liquid = network.get_untracked() == Network::Liquid;
            if let Some(req) = registry_page_request(
                liquid,
                Some("25"),
                Some("25"),
                Some("name"),
                Some("asc"),
            ) {
                schedule(req.path, later_body, network);
            }
        }>"Next registry page"</button>
        <div inner_html=move || {
            registry_list_markup(&later_body.get(), network.get() == Network::Liquid)
        }></div>
    }
}

#[component]
fn RegistryPage() -> impl IntoView {
    let network = use_context::<RwSignal<Network>>().expect("network");
    let list_body = RwSignal::new(String::new());
    let search_body = RwSignal::new(String::new());
    let draft = RwSignal::new(String::new());
    Effect::new(move |_| {
        let liquid = network.get() == Network::Liquid;
        if let Some(req) = registry_list_request(liquid) {
            schedule(req.path, list_body, network);
        }
    });
    view! {
        <h2>"Registry"</h2>
        <div inner_html=move || registry_list_markup(&list_body.get(), network.get() == Network::Liquid)></div>
        <input on:input=move |event| draft.set(event_target_value(&event))/>
        <button on:click=move |_| {
            let liquid = network.get_untracked() == Network::Liquid;
            if let Some(req) = registry_search_request(liquid, &draft.get_untracked()) {
                schedule(req.path, search_body, network);
            }
        }>"Search registry"</button>
        <div inner_html=move || registry_search_markup(&search_body.get(), network.get() == Network::Liquid)></div>
    }
}

#[component]
fn RegistryEntryPage() -> impl IntoView {
    let params = use_params_map();
    let network = use_context::<RwSignal<Network>>().expect("network");
    let body = RwSignal::new(String::new());
    Effect::new(move |_| {
        let liquid = network.get() == Network::Liquid;
        let asset = params.with(|p| p.get("asset").unwrap_or_default());
        if let Some(req) = registry_entry_request(liquid, &asset) {
            schedule(req.path, body, network);
        }
    });
    view! {
        <h2>"Registry entry"</h2>
        <div inner_html=move || registry_entry_markup(&body.get(), network.get() == Network::Liquid)></div>
    }
}

#[component]
pub fn App() -> impl IntoView {
    let network = RwSignal::new(Network::Mainnet);
    provide_context(network);
    view! {
        <Router>
            <Routes fallback=|| "Not found.">
                <ParentRoute path=path!("") view=Shell>
                    <Route path=path!("") view=Dashboard/>
                    <Route path=path!("blocks") view=BlocksPage/>
                    <Route path=path!("block/:hash") view=BlockPage/>
                    <Route path=path!("block-height/:hash") view=BlockPage/>
                    <Route path=path!("asset/:asset") view=AssetPage/>
                    <Route path=path!("assets") view=AssetsPage/>
                    <Route path=path!("assets/asset/:asset") view=AssetPage/>
                    <Route path=path!("assets/registry") view=RegistryPage/>
                    <Route path=path!("assets/registry/:asset") view=RegistryEntryPage/>
                    <Route path=path!("scripthash/:addr") view=AddressPage/>
                    <Route path=path!("widget/wallet") view=MultiAddressPage/>
                    <Route path=path!("tx/push") view=BroadcastPage/>
                    <Route path=path!("pushtx") view=BroadcastPage/>
                    <Route path=path!("tx/test") view=TestTransactionsPage/>
                    <Route path=path!("tx/:txid") view=TransactionPage/>
                    <Route path=path!("address/:addr") view=AddressPage/>
                    <Route path=path!("mempool") view=MempoolPage/>
                    <Route path=path!("calculator") view=CalculatorPage/>
                    <Route path=path!("fiat") view=FiatFormatPage/>
                    <Route path=path!("statistics") view=StatisticsPage/>
                    <Route path=path!("mining") view=MiningPage/>
                    <Route path=path!("lightning") view=LightningPage/>
                    <Route path=path!("graphs") view=GraphsPage/>
                    <Route path=path!("hashrate") view=HashratePage/>
                    <Route path=path!("price-chart") view=PriceChartPage/>
                    <Route path=path!("terms-of-service") view=TermsPage/>
                    <Route path=path!("privacy-policy") view=PrivacyPage/>
                    <Route path=path!("trademark-policy") view=TrademarkPage/>
                    <ParentRoute path=path!("docs") view=DocsLayout>
                        <Route path=path!("") view=DocsPage/>
                        <Route path=path!("faq") view=FaqPage/>
                        <Route path=path!("api/rest") view=RestApiPage/>
                        <Route path=path!("api/:type") view=ApiTypePage/>
                    </ParentRoute>
                </ParentRoute>
            </Routes>
        </Router>
    }
}
