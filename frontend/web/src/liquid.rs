//! Liquid registry, confirmed and mempool asset transactions, and supply.
//!
//! These routes exist only when the indexer is built with the liquid feature.
//! Mainnet does not request them and does not show them. An absent route stays
//! the indexer sentence or `not-served`, as a notice, not as asset fields.

use serde_json::Value;
use splora_frontend_shared::{
    AssetRegistryEntry, RegistryAsset, asset_registry_path, asset_supply_decimal_path,
    asset_supply_path, asset_txs_chain_page_path, asset_txs_chain_path, asset_txs_mempool_path,
    assets_registry_page_path, assets_registry_path, assets_registry_search_path,
    parse_asset_registry, parse_asset_supply, parse_asset_supply_decimal, parse_asset_txs_chain,
    parse_assets_registry, parse_assets_registry_search,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LiquidRequest {
    pub method: &'static str,
    pub path: String,
}

fn get(path: String) -> LiquidRequest {
    LiquidRequest {
        method: "GET",
        path,
    }
}

fn ready_asset(liquid: bool, asset: &str) -> Option<&str> {
    let asset = asset.trim();
    if !liquid || asset.is_empty() {
        None
    } else {
        Some(asset)
    }
}

pub fn registry_list_request(liquid: bool) -> Option<LiquidRequest> {
    if !liquid {
        return None;
    }
    Some(get(assets_registry_path().to_string()))
}

pub fn registry_page_request(
    liquid: bool,
    start_index: Option<&str>,
    limit: Option<&str>,
    sort_field: Option<&str>,
    sort_dir: Option<&str>,
) -> Option<LiquidRequest> {
    if !liquid {
        return None;
    }
    Some(get(assets_registry_page_path(
        start_index,
        limit,
        sort_field,
        sort_dir,
    )))
}

pub fn registry_entry_request(liquid: bool, asset: &str) -> Option<LiquidRequest> {
    let asset = ready_asset(liquid, asset)?;
    Some(get(asset_registry_path(asset)))
}

pub fn registry_search_request(liquid: bool, query: &str) -> Option<LiquidRequest> {
    let query = query.trim();
    if !liquid || query.is_empty() {
        return None;
    }
    Some(get(assets_registry_search_path(query)))
}

pub fn asset_confirmed_txs_request(liquid: bool, asset: &str) -> Option<LiquidRequest> {
    let asset = ready_asset(liquid, asset)?;
    Some(get(asset_txs_chain_path(asset)))
}

pub fn asset_confirmed_page_request(
    liquid: bool,
    asset: &str,
    last_seen_txid: &str,
) -> Option<LiquidRequest> {
    let asset = ready_asset(liquid, asset)?;
    let last_seen_txid = last_seen_txid.trim();
    if last_seen_txid.is_empty() {
        return None;
    }
    Some(get(asset_txs_chain_page_path(asset, last_seen_txid)))
}

pub fn asset_mempool_txs_request(liquid: bool, asset: &str) -> Option<LiquidRequest> {
    let asset = ready_asset(liquid, asset)?;
    Some(get(asset_txs_mempool_path(asset)))
}

pub fn asset_supply_request(liquid: bool, asset: &str) -> Option<LiquidRequest> {
    let asset = ready_asset(liquid, asset)?;
    Some(get(asset_supply_path(asset)))
}

pub fn asset_supply_decimal_request(liquid: bool, asset: &str) -> Option<LiquidRequest> {
    let asset = ready_asset(liquid, asset)?;
    Some(get(asset_supply_decimal_path(asset)))
}

/// Registry entry, confirmed txs, mempool txs, integer supply, and decimal supply.
/// Empty on mainnet and for a blank asset id.
pub fn liquid_asset_requests(liquid: bool, asset: &str) -> Vec<LiquidRequest> {
    [
        registry_entry_request(liquid, asset),
        asset_confirmed_txs_request(liquid, asset),
        asset_mempool_txs_request(liquid, asset),
        asset_supply_request(liquid, asset),
        asset_supply_decimal_request(liquid, asset),
    ]
    .into_iter()
    .flatten()
    .collect()
}

fn html_escape(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for ch in text.chars() {
        match ch {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&#39;"),
            _ => out.push(ch),
        }
    }
    out
}

fn notice(text: &str) -> String {
    format!("<p class=\"notice\">{}</p>", html_escape(text))
}

/// JSON stays data. A sentence, `not-served`, or an indexer error is a notice.
fn absent_notice(body: &str) -> Option<String> {
    let trimmed = body.trim();
    if trimmed.is_empty() {
        return Some(String::new());
    }
    if trimmed.starts_with('{') || trimmed.starts_with('[') {
        return None;
    }
    if trimmed == "not-served"
        || trimmed.contains("endpoint does not exist")
        || trimmed
            .lines()
            .any(|line| line.starts_with("indexer error "))
        || trimmed.contains(' ')
    {
        let html = trimmed
            .lines()
            .map(str::trim)
            .filter(|line| !line.is_empty())
            .map(notice)
            .collect();
        return Some(html);
    }
    None
}

fn asset_anchor(asset_id: &str) -> String {
    let escaped = html_escape(asset_id);
    format!("<a href=\"/asset/{escaped}\">{escaped}</a>")
}

fn txid_anchor(txid: &str) -> String {
    let escaped = html_escape(txid);
    format!("<a href=\"/tx/{escaped}\">{escaped}</a>")
}

fn push_text(out: &mut String, text: &str) {
    if text.is_empty() {
        return;
    }
    if !out.is_empty() {
        out.push(' ');
    }
    out.push_str(&html_escape(text));
}

fn nested_string(value: &Value, key: &str) -> Option<String> {
    match value {
        Value::Object(map) => {
            if let Some(Value::String(text)) = map.get(key) {
                if !text.is_empty() {
                    return Some(text.clone());
                }
            }
            map.values().find_map(|child| nested_string(child, key))
        }
        Value::Array(items) => items.iter().find_map(|item| nested_string(item, key)),
        _ => None,
    }
}

fn paint_registry_asset(asset: &RegistryAsset) -> String {
    match asset {
        RegistryAsset::Issued(row) => {
            let mut out = asset_anchor(&row.asset_id);
            if let Some(name) = row.name.as_deref().filter(|name| !name.is_empty()) {
                // GPUI whole line. One copy, so the name substring count stays one.
                out.push('\n');
                out.push_str(&html_escape(name));
                out.push('\n');
            }
            if let Some(ticker) = row.ticker.as_deref().filter(|ticker| !ticker.is_empty()) {
                // GPUI whole line. One copy, so the ticker substring count stays one.
                out.push('\n');
                out.push_str(&html_escape(ticker));
                out.push('\n');
            }
            if let Some(precision) = row.precision {
                push_text(&mut out, &precision.to_string());
            }
            if let Some(domain) = nested_string(&row.entity, "domain") {
                push_text(&mut out, &domain);
            }
            out
        }
        RegistryAsset::Native(row) => asset_anchor(&row.asset_id),
    }
}

fn paint_registry_entry(entry: &AssetRegistryEntry) -> String {
    let mut out = String::new();
    push_text(&mut out, &entry.name);
    if let Some(ticker) = entry.ticker.as_deref() {
        push_text(&mut out, ticker);
    }
    push_text(&mut out, &entry.precision.to_string());
    if let Some(domain) = nested_string(&entry.entity, "domain") {
        push_text(&mut out, &domain);
    }
    out
}

fn exact_empty_array(body: &str) -> bool {
    let trimmed = body.trim();
    let Some(inner) = trimmed
        .strip_prefix('[')
        .and_then(|rest| rest.strip_suffix(']'))
    else {
        return false;
    };
    inner.trim().is_empty()
}

pub fn registry_list_markup(body: &str, liquid: bool) -> String {
    if !liquid {
        return String::new();
    }
    if let Some(notice) = absent_notice(body) {
        return notice;
    }
    if exact_empty_array(body) {
        return "<p class=\"empty\">There are no assets.</p>".to_string();
    }
    let Ok(rows) = parse_assets_registry(body.trim()) else {
        return String::new();
    };
    let painted: String = rows.iter().map(paint_registry_asset).collect();
    if painted.is_empty() {
        return String::new();
    }
    // Column headers once for the assets list, not once per row.
    // Loaded thead: Name, then Ticker, then Issuer domain, then Asset ID.
    // Not the loading thead. Not the Name-line parenthetical.
    // GPUI whole line. The h1 in AssetsPage stays. Empty and mainnet return above.
    // GPUI whole line. The Name th stays. Empty, invalid, and mainnet return above.
    // GPUI whole line. The Ticker th stays. Empty, invalid, and mainnet return above.
    // GPUI whole line. The Issuer domain th stays. Empty, invalid, and mainnet return above.
    // GPUI whole line. The Asset ID th stays. Empty, invalid, and mainnet return above.
    format!(
        "\nAssets\n<th>Name</th>\nName\n<th>Ticker</th>\nTicker\n<th>Issuer domain</th>\nIssuer domain\n<th>Asset ID</th>\nAsset ID\n{painted}"
    )
}

pub fn registry_entry_markup(body: &str, liquid: bool) -> String {
    if !liquid {
        return String::new();
    }
    if let Some(notice) = absent_notice(body) {
        return notice;
    }
    let Ok(entry) = parse_asset_registry(body.trim()) else {
        return String::new();
    };
    paint_registry_entry(&entry)
}

pub fn registry_search_markup(body: &str, liquid: bool) -> String {
    if !liquid {
        return String::new();
    }
    if let Some(notice) = absent_notice(body) {
        return notice;
    }
    if exact_empty_array(body) {
        return "<p class=\"empty\">There are no assets.</p>".to_string();
    }
    let Ok(hits) = parse_assets_registry_search(body.trim()) else {
        return String::new();
    };
    hits.iter()
        .map(|hit| {
            let mut out = asset_anchor(&hit.asset_id);
            push_text(&mut out, &hit.name);
            if let Some(ticker) = hit.ticker.as_deref() {
                push_text(&mut out, ticker);
            }
            if let Some(domain) = hit.domain.as_deref() {
                push_text(&mut out, domain);
            }
            out
        })
        .collect()
}

/// One suggestion window for Cypress `assets / allows searching assets`.
///
/// The request is `registry_search_request`, which calls
/// `assets_registry_search_path` and keeps `?q=`. This string is the window
/// only. Each button's `href` is `/assets/asset/{id}` so the first hit opens
/// that asset page. An empty body, a blank query, or mainnet paints none.
/// Names and tickers only. No price and no amount.
pub fn asset_search_markup(query: &str, body: &str, liquid: bool) -> Option<String> {
    if !liquid || query.trim().is_empty() {
        return None;
    }
    let Ok(hits) = parse_assets_registry_search(body.trim()) else {
        return None;
    };
    if hits.is_empty() {
        return None;
    }
    // GPUI whole line. Empty hits, a blank query, and mainnet return above.
    let mut out =
        String::from("<ngb-typeahead-window data-label=\"suggestion window\">\nLiquid Asset\n");
    for hit in &hits {
        // First button is the Cypress click. `itemSelected` opens `/assets/asset/{id}`.
        out.push_str("<button type=\"button\" href=\"/assets/asset/");
        out.push_str(&html_escape(&hit.asset_id));
        out.push_str("\">");
        out.push_str(&html_escape(&hit.name));
        if let Some(ticker) = hit.ticker.as_deref().filter(|ticker| !ticker.is_empty()) {
            out.push_str(" (");
            out.push_str(&html_escape(ticker));
            out.push(')');
        }
        out.push_str("</button>");
    }
    out.push_str("</ngb-typeahead-window>");
    Some(out)
}

/// Featured cards for the assets screen.
///
/// The request is `registry_list_request`, which calls `assets_registry_path`
/// (`GET /assets/registry`). This string is the cards only. Each named issued
/// row is one `.card` inside `.featuredBox`. A pegged row is skipped. An empty
/// name is skipped. The card text is the name and the ticker, and the link is
/// `/assets/asset/{id}`. No price and no amount. Mainnet paints none. An empty
/// list paints none. Do not pad the list to five.
pub fn asset_cards_markup(body: &str, liquid: bool) -> Option<String> {
    if !liquid {
        return None;
    }
    let Ok(rows) = parse_assets_registry(body.trim()) else {
        return None;
    };
    let mut cards = String::new();
    for asset in rows {
        let RegistryAsset::Issued(row) = asset else {
            continue;
        };
        let Some(name) = row.name.filter(|name| !name.is_empty()) else {
            continue;
        };
        cards.push_str("<div class=\"card\"><a href=\"/assets/asset/");
        cards.push_str(&html_escape(&row.asset_id));
        cards.push_str("\">");
        cards.push_str(&html_escape(&name));
        if let Some(ticker) = row.ticker.filter(|ticker| !ticker.is_empty()) {
            cards.push(' ');
            cards.push_str(&html_escape(&ticker));
        }
        cards.push_str("</a></div>");
    }
    if cards.is_empty() {
        return None;
    }
    Some(format!("<div class=\"featuredBox\">{cards}</div>"))
}

pub fn asset_history_markup(body: &str, liquid: bool) -> String {
    if !liquid {
        return String::new();
    }
    if let Some(notice) = absent_notice(body) {
        return notice;
    }
    if exact_empty_array(body) {
        return "<p class=\"empty\">There are no transactions.</p>".to_string();
    }
    let Ok(rows) = parse_asset_txs_chain(body.trim()) else {
        return String::new();
    };
    if rows.is_empty() {
        return "<p class=\"empty\">There are no transactions.</p>".to_string();
    }
    rows.iter()
        .filter(|row| !row.txid.is_empty())
        .map(|row| txid_anchor(&row.txid))
        .collect()
}

pub fn supply_markup(body: &str, liquid: bool) -> String {
    if !liquid {
        return String::new();
    }
    let trimmed = body.trim();
    if trimmed.is_empty() {
        return String::new();
    }
    if let Ok(text) = parse_asset_supply(trimmed).or_else(|_| parse_asset_supply_decimal(trimmed)) {
        return format!("<p class=\"supply\">{}</p>", html_escape(&text));
    }
    if trimmed.starts_with('{') || trimmed.starts_with('[') {
        return notice(trimmed);
    }
    trimmed
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(notice)
        .collect()
}
