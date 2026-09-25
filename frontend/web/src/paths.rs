pub use splora_frontend_shared::{
    address_path, address_txs_path, block_path, block_txids_path, tx_path,
};

pub fn dashboard_paths() -> [&'static str; 5] {
    [
        splora_frontend_shared::blocks_tip_hash_path(),
        splora_frontend_shared::blocks_tip_height_path(),
        splora_frontend_shared::blocks_path(),
        splora_frontend_shared::fee_estimates_path(),
        splora_frontend_shared::mempool_path(),
    ]
}

pub fn block_txs_path(hash: &str, start: u32) -> Result<String, &'static str> {
    if start % 25 != 0 {
        return Err("page start must be a multiple of 25");
    }
    Ok(splora_frontend_shared::block_txs_start_index_path(
        hash,
        u64::from(start),
    ))
}

pub fn mempool_paths() -> [&'static str; 2] {
    [
        splora_frontend_shared::mempool_path(),
        splora_frontend_shared::mempool_recent_path(),
    ]
}

pub fn screen_routes() -> [&'static str; 17] {
    [
        "/",
        "/blocks",
        "/block/:hash",
        "/widget/wallet",
        "/tx/push",
        "/pushtx",
        "/tx/test",
        "/tx/:txid",
        "/address/:addr",
        "/mempool",
        "/terms-of-service",
        "/privacy-policy",
        "/trademark-policy",
        "/docs",
        "/docs/faq",
        "/docs/api/rest",
        "/docs/api/:type",
    ]
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IndexerRequest {
    pub method: &'static str,
    pub path: String,
}

pub fn allowed_post_path(path: &str) -> bool {
    path == splora_frontend_shared::broadcast_path()
        || path == splora_frontend_shared::test_txs_path()
}

pub fn broadcast_request() -> IndexerRequest {
    IndexerRequest {
        method: "POST",
        path: splora_frontend_shared::broadcast_path().to_string(),
    }
}

pub fn test_transactions_request() -> IndexerRequest {
    IndexerRequest {
        method: "POST",
        path: splora_frontend_shared::test_txs_path().to_string(),
    }
}

pub fn multi_address_screen_route() -> &'static str {
    "/widget/wallet"
}

pub fn static_page_routes() -> [&'static str; 7] {
    [
        "/terms-of-service",
        "/privacy-policy",
        "/trademark-policy",
        "/docs",
        "/docs/faq",
        "/docs/api/rest",
        "/docs/api/:type",
    ]
}

/// One concrete `/docs/api/:type` page. This is not an indexer path.
pub fn docs_api_type_page(type_segment: &str) -> Option<String> {
    if !is_one_doc_segment(type_segment) {
        return None;
    }
    Some(format!("/docs/api/{type_segment}"))
}

fn is_one_doc_segment(segment: &str) -> bool {
    let mut chars = segment.chars();
    match chars.next() {
        Some(first) if first.is_ascii_alphanumeric() => {}
        _ => return false,
    }
    chars.all(|ch| ch.is_ascii_alphanumeric() || ch == '-' || ch == '_')
}

/// Compiled page text. Static docs routes do not ask the indexer for a body.
/// A concrete `/docs/api/:type` segment is the same static page, not a proxy.
pub fn static_page_requests(route: &str) -> Vec<IndexerRequest> {
    if static_page_routes().iter().any(|known| *known == route) {
        return Vec::new();
    }
    if let Some(segment) = route.strip_prefix("/docs/api/") {
        if is_one_doc_segment(segment) {
            return Vec::new();
        }
    }
    Vec::new()
}

pub fn addresses_from_query(query: &str) -> Vec<String> {
    let query = query.trim().trim_start_matches('?');
    let mut scripts = Vec::new();
    if query.is_empty() {
        return scripts;
    }
    for pair in query.split('&') {
        if pair.is_empty() {
            continue;
        }
        let (key, value) = pair.split_once('=').unwrap_or((pair, ""));
        if percent_decode(key) != "addresses" {
            continue;
        }
        for script in percent_decode(value).split(',') {
            let script = script.trim();
            if !script.is_empty() {
                scripts.push(script.to_string());
            }
        }
    }
    scripts
}

pub fn multi_address_requests(query: &str) -> Vec<IndexerRequest> {
    addresses_from_query(query)
        .into_iter()
        .map(|script| IndexerRequest {
            method: "GET",
            path: address_path(&script),
        })
        .collect()
}

pub fn multi_address_requests_from_values(values: &[String]) -> Vec<IndexerRequest> {
    if values.is_empty() {
        return Vec::new();
    }
    let query = values
        .iter()
        .map(|value| format!("addresses={}", encode_query_component(value)))
        .collect::<Vec<_>>()
        .join("&");
    multi_address_requests(&query)
}

pub fn v1_client_requests() -> Vec<IndexerRequest> {
    let mut calls = Vec::new();
    for path in dashboard_paths() {
        calls.push(IndexerRequest {
            method: "GET",
            path: path.to_string(),
        });
    }
    calls.push(IndexerRequest {
        method: "GET",
        path: block_path("00"),
    });
    calls.push(IndexerRequest {
        method: "GET",
        path: block_txids_path("00"),
    });
    calls.push(IndexerRequest {
        method: "GET",
        path: block_txs_path("00", 0).unwrap_or_default(),
    });
    calls.push(IndexerRequest {
        method: "GET",
        path: tx_path("00"),
    });
    calls.push(IndexerRequest {
        method: "GET",
        path: address_path("addr"),
    });
    calls.push(IndexerRequest {
        method: "GET",
        path: address_txs_path("addr"),
    });
    calls.push(IndexerRequest {
        method: "GET",
        path: splora_frontend_shared::address_utxo_path("addr"),
    });
    calls.push(IndexerRequest {
        method: "GET",
        path: splora_frontend_shared::block_height_path(0),
    });
    calls.push(IndexerRequest {
        method: "GET",
        path: splora_frontend_shared::block_txs_path("00"),
    });
    for path in mempool_paths() {
        calls.push(IndexerRequest {
            method: "GET",
            path: path.to_string(),
        });
    }
    calls.extend(multi_address_requests("addresses=one,two"));
    calls.push(broadcast_request());
    calls.push(test_transactions_request());
    for route in static_page_routes() {
        calls.extend(static_page_requests(route));
    }
    calls
}

fn encode_query_component(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    for byte in value.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' | b',' => {
                out.push(byte as char);
            }
            _ => {
                const HEX: &[u8; 16] = b"0123456789ABCDEF";
                out.push('%');
                out.push(HEX[(byte >> 4) as usize] as char);
                out.push(HEX[(byte & 0x0f) as usize] as char);
            }
        }
    }
    out
}

fn percent_decode(input: &str) -> String {
    let bytes = input.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut index = 0;
    while index < bytes.len() {
        if bytes[index] == b'%' && index + 2 < bytes.len() {
            let hex = &input[index + 1..index + 3];
            if let Ok(value) = u8::from_str_radix(hex, 16) {
                out.push(value);
                index += 3;
                continue;
            }
        }
        if bytes[index] == b'+' {
            out.push(b' ');
        } else {
            out.push(bytes[index]);
        }
        index += 1;
    }
    String::from_utf8_lossy(&out).into_owned()
}

pub fn switcher_labels() -> [&'static str; 5] {
    ["mainnet", "testnet3", "testnet4", "mutinynet", "liquid"]
}
