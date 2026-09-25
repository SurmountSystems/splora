use crate::FAIL_CLOSED;
use crate::browser::{browser_signer_present, page_origin, signed_get};
use crate::models::recent_txids;
use crate::paths::{docs_api_type_page, mempool_paths, static_page_requests, tx_path};
use crate::screens::{
    address_fields_text, address_requests, block_fields_text, block_load_plan_with_txids,
    block_txids_fields_text, blocks_list_fields_text, blocks_list_requests,
    blocks_list_requests_at, broadcast_fields_text, broadcast_payload, broadcast_screen_request,
    dashboard_fields_text, dashboard_screen_requests, entered_addresses, multi_address_fields_text,
    multi_address_screen_requests, test_transactions_fields_text, test_transactions_payload,
    test_transactions_screen_request, transaction_fields_text, transaction_requests,
};
use crate::theme::{ACCENT, BACKGROUND, TEXT};
use leptos::prelude::*;
use leptos_router::components::{A, Outlet, ParentRoute, Route, Router, Routes};
use leptos_router::hooks::{use_params_map, use_query_map};
use leptos_router::path;
use splora_api::Network;
use splora_frontend_shared::{
    address_path, address_txs_path, address_utxo_path, block_height_path, block_path,
    block_txs_path, blocks_path, blocks_start_height_path, blocks_tip_hash_path,
    blocks_tip_height_path, broadcast_path, fee_estimates_path, mempool_path, mempool_recent_path,
    parse_blocks_tip_hash, parse_blocks_tip_height, parse_fee_estimates, parse_mempool,
    test_txs_path,
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
        </nav>
    }
}

#[component]
fn FailClosed() -> impl IntoView {
    view! { <p>{FAIL_CLOSED}</p> }
}

fn schedule(indexer_path: String, body: RwSignal<String>, network: RwSignal<Network>) {
    #[cfg(target_arch = "wasm32")]
    if browser_signer_present() {
        leptos::task::spawn_local(async move {
            let path = indexer_path;
            let result = signed_get(&page_origin(), network.get_untracked(), &path).await;
            body.set(crate::text_after_signed_get(result));
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
            body.set(crate::text_after_signed_get(result));
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
) {
    #[cfg(target_arch = "wasm32")]
    load_block_signed(
        id, page_start, initial, network, block_body, txs_body, txids_body,
    );
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = (
            id, page_start, initial, network, block_body, txs_body, txids_body,
        );
    }
}

#[cfg(target_arch = "wasm32")]
fn store_block_response(
    path: &str,
    text: String,
    height_body: &mut Option<String>,
    block_body: RwSignal<String>,
    txs_body: RwSignal<String>,
    txids_body: RwSignal<String>,
) {
    if path.starts_with("/block-height/") {
        *height_body = Some(text);
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
) {
    if !browser_signer_present() {
        return;
    }
    leptos::task::spawn_local(async move {
        let mut height_body = None;
        for req in initial {
            let result = signed_get(&page_origin(), network.get_untracked(), &req.path).await;
            let text = crate::text_after_signed_get(result);
            store_block_response(
                &req.path,
                text,
                &mut height_body,
                block_body,
                txs_body,
                txids_body,
            );
        }
        if let Some(body) = height_body {
            let mut ignored_height = None;
            for req in block_load_plan_with_txids(&id, Some(&body), page_start) {
                let result = signed_get(&page_origin(), network.get_untracked(), &req.path).await;
                let text = crate::text_after_signed_get(result);
                store_block_response(
                    &req.path,
                    text,
                    &mut ignored_height,
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
            let text = crate::text_after_signed_get(result);
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
fn Shell() -> impl IntoView {
    view! {
        <div style=shell_style()>
            <h1>"Splora"</h1>
            <Switcher/>
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

fn skip_closed_or_empty(body: &str, lines: &mut Vec<String>) -> bool {
    if body.trim().is_empty() {
        return true;
    }
    if body == FAIL_CLOSED {
        push_notice(lines, FAIL_CLOSED);
        return true;
    }
    false
}

fn mempool_summary_text(body: &str) -> String {
    let mut lines = Vec::new();
    if skip_closed_or_empty(body, &mut lines) {
        return lines.join("\n");
    }
    match parse_mempool(body) {
        Ok(stats) => {
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
            Ok(rows) => {
                for row in rows {
                    lines.push(format!(
                        "fee target {target} rate {rate}",
                        target = row.target,
                        rate = row.rate
                    ));
                }
            }
            Err(_) => push_notice(&mut lines, "unreadable indexer response"),
        }
    }
    lines.join("\n")
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
        <pre>{move || dashboard_fields_text(&blocks_body.get(), &recent_body.get())}</pre>
        <pre>{move || dashboard_tip_fees_text(&tip_hash_body.get(), &tip_height_body.get(), &fees_body.get())}</pre>
        <pre>{move || mempool_summary_text(&mempool_body.get())}</pre>
    }
}

#[component]
fn BlocksPage() -> impl IntoView {
    let network = use_context::<RwSignal<Network>>().expect("network");
    let query = use_query_map();
    let body = RwSignal::new(String::new());
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
    view! {
        <h2>"Blocks"</h2>
        <pre>{move || blocks_list_fields_text(&body.get())}</pre>
    }
}

#[component]
fn BlockPage() -> impl IntoView {
    let params = use_params_map();
    let query = use_query_map();
    let network = use_context::<RwSignal<Network>>().expect("network");
    let block_body = RwSignal::new(String::new());
    let txs_body = RwSignal::new(String::new());
    let txids_body = RwSignal::new(String::new());
    let id = params.with(|p| p.get("hash").unwrap_or_default());
    let page_start = query.with(|map| {
        map.get("start_index")
            .and_then(|text| text.parse::<u32>().ok())
    });
    let mut initial = block_load_plan_with_txids(&id, None, page_start);
    let trimmed = id.trim();
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
        let block = block_path(trimmed);
        let txs = block_txs_path(trimmed);
        for req in &mut initial {
            if req.path == block {
                req.path = block.clone();
            } else if req.path == txs {
                req.path = txs.clone();
            }
        }
    }
    schedule_block(
        id, page_start, initial, network, block_body, txs_body, txids_body,
    );
    view! {
        <h2>"Block"</h2>
        <pre>{move || block_fields_text(&block_body.get(), &txs_body.get())}</pre>
        <pre>{move || block_txids_fields_text(&txids_body.get())}</pre>
    }
}

#[component]
fn TransactionPage() -> impl IntoView {
    let params = use_params_map();
    let network = use_context::<RwSignal<Network>>().expect("network");
    let body = RwSignal::new(String::new());
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
    view! {
        <h2>"Transaction"</h2>
        <pre>{move || transaction_fields_text(&body.get())}</pre>
    }
}

#[component]
fn AddressPage() -> impl IntoView {
    let params = use_params_map();
    let network = use_context::<RwSignal<Network>>().expect("network");
    let stats_body = RwSignal::new(String::new());
    let txs_body = RwSignal::new(String::new());
    let utxo_body = RwSignal::new(String::new());
    let addr = params.with(|p| p.get("addr").unwrap_or_default());
    let script = addr.trim();
    for req in address_requests(script) {
        if req.path == address_utxo_path(script) {
            schedule(req.path, utxo_body, network);
        } else if req.path == address_txs_path(script) {
            schedule(req.path, txs_body, network);
        } else if req.path == address_path(script) {
            schedule(req.path, stats_body, network);
        } else {
            schedule(req.path, stats_body, network);
        }
    }
    view! {
        <h2>"Address"</h2>
        <pre>{move || address_fields_text(&stats_body.get(), &txs_body.get(), &utxo_body.get())}</pre>
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
    view! {
        <h2>{heading}</h2>
        <p>{method}" "{path_label}</p>
        <textarea on:input=move |event| raw.set(event_target_value(&event))></textarea>
        <button on:click=move |_| {
            let payload = prepare(&raw.get_untracked());
            schedule_post(method, path.clone(), payload, response, network);
        }>{button_label}</button>
        <pre>{move || render(&response.get())}</pre>
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
    };
    load(scripts);
    view! {
        <h2>"Addresses"</h2>
        <textarea on:input=move |event| draft.set(event_target_value(&event))></textarea>
        <button on:click=move |_| {
            load(entered_addresses(&draft.get_untracked()));
        }>"Load addresses"</button>
        <pre>{move || multi_address_fields_text(&rows.get())}</pre>
    }
}

#[component]
fn BroadcastPage() -> impl IntoView {
    let call = broadcast_screen_request();
    let path = if call.path == broadcast_path() {
        call.path
    } else {
        broadcast_path().to_string()
    };
    post_screen(
        call.method,
        path,
        "Broadcast",
        "Broadcast",
        broadcast_payload,
        broadcast_fields_text,
    )
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
    let paths = mempool_paths();
    schedule(paths[0].to_string(), body, network);
    schedule(paths[1].to_string(), recent, network);
    view! {
        <h2>"Mempool"</h2>
        <ul>
            {mempool_paths()
                .into_iter()
                .map(|path| view! { <li>{path}</li> })
                .collect_view()}
        </ul>
        <pre>{move || mempool_summary_text(&body.get())}</pre>
        {move || {
            recent_txids(&recent.get())
                .into_iter()
                .map(|txid| {
                    let href = tx_path(&txid);
                    view! {
                        <A href=href>
                            <span style=link_style()>{txid}</span>
                        </A>
                    }
                })
                .collect_view()
        }}
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
                    <Route path=path!("widget/wallet") view=MultiAddressPage/>
                    <Route path=path!("tx/push") view=BroadcastPage/>
                    <Route path=path!("pushtx") view=BroadcastPage/>
                    <Route path=path!("tx/test") view=TestTransactionsPage/>
                    <Route path=path!("tx/:txid") view=TransactionPage/>
                    <Route path=path!("address/:addr") view=AddressPage/>
                    <Route path=path!("mempool") view=MempoolPage/>
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
