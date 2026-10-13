//! Paths this indexer does not serve. They are not registered routes.

/// `"not-served"` for statistics, mining, prices, replace-by-fee paths, and
/// any path that contains `lightning`. `/api/v1/tx/<txid>/rbf` is not-served.
/// `GET /tx/<txid>` is not that path. Other paths return `"not-checked"`.
pub fn route_availability(path: &str) -> &'static str {
    if path == "/api/v1/statistics"
        || path == "/v1/mining"
        || path == "/v1/prices"
        || path.to_ascii_lowercase().contains("lightning")
        || path == "/api/v1/replacements"
        || path == "/api/v1/fullrbf/replacements"
        || is_api_v1_tx_rbf(path)
    {
        "not-served"
    } else {
        "not-checked"
    }
}

/// `/api/v1/tx/<txid>/rbf` only. A trailing extra segment does not match.
fn is_api_v1_tx_rbf(path: &str) -> bool {
    let Some(rest) = path.strip_prefix("/api/v1/tx/") else {
        return false;
    };
    let mut parts = rest.split('/');
    match (parts.next(), parts.next(), parts.next()) {
        (Some(txid), Some("rbf"), None) if !txid.is_empty() => true,
        _ => false,
    }
}
