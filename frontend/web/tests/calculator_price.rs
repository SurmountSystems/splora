//! The calculator names `/v1/prices` through `route_availability`.
//! This indexer does not serve that feed, so the shown sentence is `not-served`.
//! Amount inputs call shared fixture conversion. A live price feed is not served.

use splora_frontend_shared::route_availability;

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
fn calculator_shows_v1_prices_not_served() {
    assert_eq!(route_availability("/v1/prices"), "not-served");

    let calculator = component_body(views_source(), "CalculatorPage");
    assert!(
        code_has_call(
            calculator,
            "let prices = route_availability(\"/v1/prices\")"
        ),
        "CalculatorPage does not call route_availability for /v1/prices"
    );
    assert_eq!(quoted_route_arg(calculator), "/v1/prices");
    assert!(
        code_has_call(calculator, "{prices}"),
        "CalculatorPage does not show the prices sentence"
    );
    assert!(
        !calculator.contains("\"not-served\""),
        "CalculatorPage hardcodes not-served"
    );
    assert!(
        calculator.to_ascii_lowercase().matches("<input").count() >= 3,
        "CalculatorPage does not show fiat, BTC, and sats inputs"
    );
    assert!(
        calculator.contains("A live price feed is not served."),
        "CalculatorPage does not say the live price feed is not served"
    );
    assert!(
        !code_has_call(calculator, "schedule("),
        "CalculatorPage fetches the indexer"
    );
    assert!(
        !code_has_call(calculator, "signed_get("),
        "CalculatorPage fetches the indexer"
    );
}
