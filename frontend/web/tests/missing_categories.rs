//! Graphs and hashrate render `not-served`. The price chart names
//! `/api/v1/historical-price` and renders `not-checked`.
//! No chart is drawn. No price is requested.

use splora_web::{graphs_page_text, hashrate_page_text, price_chart_page_text};

fn views_source() -> &'static str {
    include_str!("../src/views.rs")
}

/// Body inside the function braces, not the next function.
fn component_body<'a>(src: &'a str, name: &str) -> &'a str {
    let marker = format!("fn {name}(");
    let start = src
        .find(&marker)
        .unwrap_or_else(|| panic!("{name} is missing from views.rs"));
    let after_name = &src[start..];
    let brace_rel = after_name
        .find('{')
        .unwrap_or_else(|| panic!("{name} has no body"));
    let body_start = start + brace_rel + 1;
    let mut depth = 1i32;
    for (offset, ch) in src[body_start..].char_indices() {
        match ch {
            '{' => depth += 1,
            '}' => {
                depth -= 1;
                if depth == 0 {
                    return &src[body_start..body_start + offset];
                }
            }
            _ => {}
        }
    }
    panic!("{name} body is not closed");
}

fn code_has_call(body: &str, call: &str) -> bool {
    body.lines().any(|line| {
        let code = line.split("//").next().unwrap_or(line);
        code.contains(call)
    })
}

fn quoted_route_arg(body: &str) -> String {
    let code = body
        .lines()
        .filter_map(|line| line.split("//").next())
        .collect::<Vec<_>>()
        .join("\n");
    let key = "route_availability(";
    let start = code
        .find(key)
        .unwrap_or_else(|| panic!("missing route_availability call in {body}"));
    let after = &code[start + key.len()..];
    let quote = after
        .find('"')
        .unwrap_or_else(|| panic!("route_availability has no path string in {body}"));
    let rest = &after[quote + 1..];
    let end = rest
        .find('"')
        .unwrap_or_else(|| panic!("route path is not closed in {body}"));
    rest[..end].to_string()
}

#[test]
fn graphs_hashrate_and_price_chart_show_route_tokens() {
    assert_eq!(graphs_page_text(), "not-served");
    assert_eq!(hashrate_page_text(), "not-served");
    assert_eq!(price_chart_page_text(), "not-checked");

    let src = views_source();
    let graphs = component_body(src, "GraphsPage");
    let hashrate = component_body(src, "HashratePage");
    let price_chart = component_body(src, "PriceChartPage");
    assert_eq!(quoted_route_arg(graphs), "/api/v1/statistics");
    assert_eq!(quoted_route_arg(hashrate), "/v1/mining");
    assert_eq!(quoted_route_arg(price_chart), "/api/v1/historical-price");
    assert!(code_has_call(graphs, "route_availability("));
    assert!(code_has_call(hashrate, "route_availability("));
    assert!(code_has_call(price_chart, "route_availability("));
    assert!(code_has_call(graphs, "graphs_page_text("));
    assert!(code_has_call(hashrate, "hashrate_page_text("));
    assert!(code_has_call(price_chart, "price_chart_page_text("));

    let app = component_body(src, "App");
    for path in [
        "path!(\"graphs\")",
        "path!(\"hashrate\")",
        "path!(\"price-chart\")",
    ] {
        assert!(code_has_call(app, path), "App dropped {path}");
    }

    for (name, body) in [
        ("GraphsPage", graphs),
        ("HashratePage", hashrate),
        ("PriceChartPage", price_chart),
    ] {
        assert!(!code_has_call(body, "signed_get("), "{name} fetches");
        assert!(!code_has_call(body, "schedule("), "{name} fetches");
        assert!(
            !code_has_call(body, "btc_fiat_usd("),
            "{name} requests a price"
        );
        assert!(
            !code_has_call(body, "btc_fiat_jpy("),
            "{name} requests a price"
        );
        let lower = body.to_ascii_lowercase();
        for banned in ["canvas", "svg", "sparkline", "polyline"] {
            assert!(!lower.contains(banned), "{name} contains {banned}");
        }
    }
}
