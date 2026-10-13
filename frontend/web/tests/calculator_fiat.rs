//! Calculator, fiat format, and unserved pages on the Leptos explorer.
//! View source has to call `splora_frontend_shared`. Rendered strings come
//! from the wrappers re-exported by `splora_web`.

use splora_web::{
    capped_btc_text, jpy_fixture_text, lightning_page_text, mining_page_text, one_btc_sats_text,
    statistics_page_text, switched_fiat_symbols, truncated_btc_text, usd_fixture_text,
};

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

fn shared_use_imports(src: &str, name: &str) -> bool {
    let mut rest = src;
    let needle = "use splora_frontend_shared::";
    while let Some(at) = rest.find(needle) {
        let after = &rest[at..];
        let Some(end) = after.find(';') else {
            return false;
        };
        if after[..end].contains(name) {
            return true;
        }
        rest = &after[end + 1..];
    }
    false
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

fn assert_no_fake_drawing(name: &str, body: &str) {
    let lower = body.to_ascii_lowercase();
    for banned in ["chart", "canvas", "svg", "channel"] {
        assert!(!lower.contains(banned), "{name} contains {banned}");
    }
}

#[test]
fn calculator_fiat_and_unserved_pages_use_shared_functions() {
    let src = views_source();
    for name in [
        "btc_to_sats",
        "truncate_btc_decimals",
        "cap_btc_supply",
        "format_fiat",
        "switch_usd_jpy_usd",
        "route_availability",
    ] {
        assert!(
            shared_use_imports(src, name),
            "{name} is not imported from splora_frontend_shared"
        );
        assert!(
            !src.contains(&format!("fn {name}(")),
            "{name} is copied in views.rs"
        );
    }

    let calculator = component_body(src, "CalculatorPage");
    for call in [
        "btc_to_sats(",
        "truncate_btc_decimals(",
        "cap_btc_supply(",
        "one_btc_sats_text(",
        "truncated_btc_text(",
        "capped_btc_text(",
    ] {
        assert!(
            code_has_call(calculator, call),
            "CalculatorPage does not call {call}"
        );
    }

    let fiat = component_body(src, "FiatFormatPage");
    for call in [
        "format_fiat(",
        "switch_usd_jpy_usd(",
        "usd_fixture_text(",
        "jpy_fixture_text(",
        "switched_fiat_symbols(",
    ] {
        assert!(
            code_has_call(fiat, call),
            "FiatFormatPage does not call {call}"
        );
    }

    let statistics = component_body(src, "StatisticsPage");
    let mining = component_body(src, "MiningPage");
    let lightning = component_body(src, "LightningPage");
    assert_eq!(quoted_route_arg(statistics), "/api/v1/statistics");
    assert_eq!(quoted_route_arg(mining), "/v1/mining");
    let lightning_path = quoted_route_arg(lightning);
    assert!(
        lightning_path.to_ascii_lowercase().contains("lightning"),
        "LightningPage path does not contain lightning: {lightning_path}"
    );
    assert!(code_has_call(statistics, "route_availability("));
    assert!(code_has_call(mining, "route_availability("));
    assert!(code_has_call(lightning, "route_availability("));
    assert!(code_has_call(statistics, "statistics_page_text("));
    assert!(code_has_call(mining, "mining_page_text("));
    assert!(code_has_call(lightning, "lightning_page_text("));
    assert_no_fake_drawing("StatisticsPage", statistics);
    assert_no_fake_drawing("MiningPage", mining);
    assert_no_fake_drawing("LightningPage", lightning);
    for (name, body) in [
        ("StatisticsPage", statistics),
        ("MiningPage", mining),
        ("LightningPage", lightning),
    ] {
        assert!(
            !code_has_call(body, "schedule("),
            "{name} fetches the indexer"
        );
        assert!(
            !code_has_call(body, "signed_get("),
            "{name} fetches the indexer"
        );
    }

    let app = component_body(src, "App");
    for path in [
        "path!(\"blocks\")",
        "path!(\"calculator\")",
        "path!(\"fiat\")",
        "path!(\"statistics\")",
        "path!(\"mining\")",
        "path!(\"lightning\")",
    ] {
        assert!(code_has_call(app, path), "App dropped {path}");
    }

    assert_eq!(one_btc_sats_text(), "100000000");
    assert_eq!(truncated_btc_text("1.123456789"), "1.12345678");
    assert_eq!(capped_btc_text("25000000"), "21000000");
    assert_eq!(usd_fixture_text(), "$123456");
    let jpy = jpy_fixture_text();
    assert_eq!(jpy.chars().next(), Some('¥'));
    assert!(!jpy.contains('.'));
    assert!(!jpy.contains('\u{FFE5}'));
    assert_eq!(jpy, "\u{00A5}11057757");
    assert_eq!(switched_fiat_symbols(), ['$', '\u{00A5}', '$']);
    assert_eq!(statistics_page_text(), "not-served");
    assert_eq!(mining_page_text(), "not-served");
    assert_eq!(lightning_page_text(), "not-served");
}
