//! Search control. Turns one query into page links and indexer GETs.

use serde_json::Value;
use splora_frontend_shared::{
    address_path, address_prefix_path, asset_path, asset_txs_path, block_height_path, block_path,
    parse_address_prefix, parse_address_txs, scripthash_path, scripthash_txs_mempool_path,
    scripthash_txs_path, scripthash_utxo_path, tx_path,
};

use crate::paths::IndexerRequest;
use crate::rows::{address_rows_markup, asset_markup, scripthash_markup, tip_links_markup};

/// Lowercase bech32 human-readable parts plus the separator `1`.
const ADDRESS_STARTS: &[&str] = &["bc1", "tb1", "bcrt1", "lq1", "ex1", "ert1"];

/// A mainnet taproot address is `bc1p` plus 58 characters (62 in total).
const BECH32M_ADDRESS_LEN: usize = 62;

/// Bitcoin base58 alphabet. Excludes 0, O, I, and l.
const BASE58: &str = "123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SearchPlan {
    /// Page hrefs that already exist: /tx/{id}, /address/{addr}, /block/{hash}, /block-height/{height}.
    pub hrefs: Vec<String>,
    /// Indexer GETs. A 64-hex id that is not all digits requests the scripthash stats, transactions, utxos, and mempool transactions. On liquid it also requests the asset and its transactions. An address prefix requests GET /address-prefix/{prefix}. A height requests GET /block-height/{height}. An address requests GET /address/{address}.
    pub requests: Vec<IndexerRequest>,
}

/// Page hrefs and indexer GETs for a trimmed search query on mainnet.
///
/// `None` means the query is empty or not a height, id, address, or prefix.
pub fn plan_search(raw: &str) -> Option<SearchPlan> {
    plan_search_on(raw, false)
}

/// Same classification as [`plan_search`]. `liquid` adds the asset lookup for a 64-hex id.
pub fn plan_search_on(raw: &str, liquid: bool) -> Option<SearchPlan> {
    let query = raw.trim();
    if query.is_empty() {
        return None;
    }
    // Digits win over hex, so a 64-digit id is a height, not a transaction or an asset.
    if is_ascii_digits(query) {
        return Some(height_plan(query));
    }
    if is_hex_64(query) {
        return Some(hex_plan(query, liquid));
    }
    // "bc1q" starts with bc1, so it is an address, not an address prefix.
    // A short bc1p string is a prefix. A 62-character bech32m address is not.
    if is_address(query) {
        return Some(address_plan(query));
    }
    if is_prefix(query) {
        return Some(prefix_plan(query));
    }
    None
}

/// HTML for address-prefix JSON (a JSON array of address strings) or the indexer sentence.
pub fn address_prefix_markup(body: &str) -> String {
    let trimmed = body.trim();
    if trimmed.is_empty() {
        return String::new();
    }
    if let Some(html) = indexer_lines_markup(trimmed) {
        return html;
    }
    if trimmed == "address search disabled" {
        return notice(trimmed);
    }
    if let Some(addresses) = address_strings(trimmed) {
        if addresses.is_empty() {
            return paragraph("empty", "There are no addresses.");
        }
        return addresses
            .into_iter()
            .filter(|address| !address.is_empty())
            .map(|address| address_anchor(&address))
            .collect();
    }
    if single_non_json_line(trimmed) {
        return notice(trimmed);
    }
    String::new()
}

/// HTML for one search response. An indexer failure keeps that sentence.
pub fn search_result_markup(query: &str, body: &str) -> String {
    let Some(plan) = plan_search(query) else {
        return String::new();
    };
    if let Some(html) = indexer_lines_markup(body) {
        return html;
    }
    let mut html = String::new();
    let mut painted_script = false;
    let mut painted_address = false;
    let mut painted_prefix = false;
    let mut painted_tip = false;
    for req in &plan.requests {
        if req.path.contains("/address-prefix/") {
            if !painted_prefix {
                html.push_str(&address_prefix_markup(body));
                painted_prefix = true;
            }
        } else if req.path.contains("/scripthash/")
            && !req.path.contains("/txs")
            && !req.path.contains("/utxo")
        {
            if !painted_script {
                html.push_str(&scripthash_markup(body));
                painted_script = true;
            }
        } else if req.path.starts_with("/address/") {
            if !painted_address {
                html.push_str(&address_rows_markup(body, "", ""));
                painted_address = true;
            }
        } else if !req.path.contains("/asset/") && !painted_tip {
            html.push_str(&tip_links_markup(body, ""));
            painted_tip = true;
        }
    }
    html
}

/// HTML for the bodies a search actually loaded. Mainnet skips asset parts.
pub fn search_loaded_markup(query: &str, liquid: bool, parts: &[(String, String)]) -> String {
    let Some(plan) = plan_search_on(query, liquid) else {
        return String::new();
    };
    let mut html = String::new();
    for (path, body) in parts {
        if !plan.requests.iter().any(|req| req.path == *path) {
            continue;
        }
        if let Some(notice) = indexer_lines_markup(body) {
            html.push_str(&notice);
            continue;
        }
        if path.contains("/asset/") && path.ends_with("/txs") {
            html.push_str(&asset_tx_links(body));
        } else if path.contains("/asset/") {
            html.push_str(&asset_markup(body, "", true));
        } else if path.contains("/utxo") {
            html.push_str(&address_rows_markup("", "", body));
        } else if path.contains("/txs") {
            html.push_str(&address_rows_markup("", body, ""));
        } else if path.contains("/scripthash/") {
            html.push_str(&scripthash_markup(body));
        } else if path.contains("/address-prefix/") {
            html.push_str(&address_prefix_markup(body));
        } else if path.starts_with("/address/") {
            html.push_str(&address_rows_markup(body, "", ""));
        } else {
            html.push_str(&tip_links_markup(body, ""));
        }
    }
    html
}

fn asset_tx_links(body: &str) -> String {
    let Ok(txs) = parse_address_txs(body) else {
        return String::new();
    };
    txs.iter()
        .filter(|tx| !tx.txid.is_empty())
        .map(|tx| {
            let escaped = html_escape(&tx.txid);
            format!("<a href=\"/tx/{escaped}\">{escaped}</a>")
        })
        .collect()
}

fn indexer_lines_markup(body: &str) -> Option<String> {
    let trimmed = body.trim();
    if trimmed.is_empty() || trimmed.starts_with('{') || trimmed.starts_with('[') {
        return None;
    }
    if !trimmed
        .lines()
        .any(|line| line.starts_with("indexer error "))
    {
        return None;
    }
    Some(
        trimmed
            .lines()
            .map(str::trim)
            .filter(|line| !line.is_empty())
            .map(notice)
            .collect(),
    )
}

/// HTML links from [`plan_search`], for the search control.
pub fn search_links_markup(raw: &str) -> String {
    let Some(plan) = plan_search(raw) else {
        return String::new();
    };
    let mut markup = String::new();
    for href in plan.hrefs {
        let escaped = html_escape(&href);
        markup.push_str(&format!("<a href=\"{escaped}\">{escaped}</a>"));
    }
    markup
}

fn height_plan(height: &str) -> SearchPlan {
    let href = match height.parse::<u64>() {
        Ok(value) => block_height_path(value),
        Err(_) => format!("/block-height/{height}"),
    };
    SearchPlan {
        hrefs: vec![href.clone()],
        requests: vec![get(href)],
    }
}

fn hex_plan(hex: &str, liquid: bool) -> SearchPlan {
    let hex = hex.to_ascii_lowercase();
    let mut requests = vec![
        get(scripthash_path(&hex)),
        get(scripthash_txs_path(&hex)),
        get(scripthash_utxo_path(&hex)),
        get(scripthash_txs_mempool_path(&hex)),
    ];
    if liquid {
        requests.push(get(asset_path(&hex)));
        requests.push(get(asset_txs_path(&hex)));
    }
    SearchPlan {
        hrefs: vec![tx_path(&hex), block_path(&hex)],
        requests,
    }
}

fn address_plan(address: &str) -> SearchPlan {
    let href = address_path(address);
    SearchPlan {
        hrefs: vec![href.clone()],
        requests: vec![get(href)],
    }
}

fn prefix_plan(prefix: &str) -> SearchPlan {
    // The address index stores mainnet bech32 prefixes in lowercase.
    let folded = prefix.to_ascii_lowercase();
    let requested = if folded.starts_with("bc1") {
        folded.as_str()
    } else {
        prefix
    };
    SearchPlan {
        hrefs: Vec::new(),
        requests: vec![get(address_prefix_path(requested))],
    }
}

fn get(path: String) -> IndexerRequest {
    IndexerRequest {
        method: "GET",
        path,
    }
}

fn is_ascii_digits(query: &str) -> bool {
    !query.is_empty() && query.chars().all(|ch| ch.is_ascii_digit())
}

fn is_hex_64(query: &str) -> bool {
    query.len() == 64 && query.chars().all(|ch| ch.is_ascii_hexdigit())
}

fn is_address(query: &str) -> bool {
    let lower = query.to_ascii_lowercase();
    if ADDRESS_STARTS.iter().any(|start| lower.starts_with(start)) {
        // Partial bc1p text is a prefix lookup. A full bech32m address still wins.
        if lower.starts_with("bc1p")
            && query.chars().count() < BECH32M_ADDRESS_LEN
            && is_prefix(query)
        {
            return false;
        }
        return true;
    }
    is_base58_address(query)
}

fn is_base58_address(query: &str) -> bool {
    let len = query.chars().count();
    if !(26..=90).contains(&len) {
        return false;
    }
    match query.chars().next() {
        Some('1' | '3' | 'm' | 'n' | '2') => query.chars().all(is_base58),
        _ => false,
    }
}

fn is_base58(ch: char) -> bool {
    BASE58.contains(ch)
}

fn is_prefix(query: &str) -> bool {
    let len = query.chars().count();
    (1..=64).contains(&len) && !query.chars().any(is_forbidden_in_prefix)
}

fn is_forbidden_in_prefix(ch: char) -> bool {
    matches!(ch, '/' | '?' | '#' | ' ') || ch.is_control()
}

fn address_strings(body: &str) -> Option<Vec<String>> {
    parse_address_prefix(body).ok()
}

fn single_non_json_line(body: &str) -> bool {
    !body.contains('\n') && !body.contains('\r') && serde_json::from_str::<Value>(body).is_err()
}

fn address_anchor(address: &str) -> String {
    let escaped = html_escape(address);
    format!("<a class=\"address\" href=\"/address/{escaped}\">{escaped}</a>")
}

fn notice(sentence: &str) -> String {
    paragraph("notice", sentence)
}

fn paragraph(class: &str, text: &str) -> String {
    format!("<p class=\"{class}\">{}</p>", html_escape(text))
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
