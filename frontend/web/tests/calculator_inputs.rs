//! Calculator amount inputs call shared fiat conversion.
//! A live price feed is not served. No price route is added.

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

#[test]
fn calculator_inputs_use_shared_fiat_functions() {
    let src = views_source();
    let calculator = component_body(src, "CalculatorPage");
    assert!(
        calculator.to_ascii_lowercase().matches("<input").count() >= 3,
        "CalculatorPage does not show three amount inputs"
    );
    assert!(
        calculator.contains("\"fiat\""),
        "CalculatorPage is missing the fiat input"
    );
    assert!(
        calculator.contains("\"BTC\""),
        "CalculatorPage is missing the BTC input"
    );
    assert!(
        calculator.contains("\"sats\""),
        "CalculatorPage is missing the sats input"
    );
    for call in [
        "usd_to_btc_and_sats(",
        "btc_fiat_usd(",
        "btc_fiat_jpy(",
        "sats_fiat_usd(",
    ] {
        assert!(
            code_has_call(calculator, call),
            "CalculatorPage does not call {call}"
        );
    }
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
    let app = component_body(src, "App");
    assert!(
        code_has_call(app, "path!(\"calculator\")"),
        "App dropped the calculator route"
    );
    assert!(!app.contains("path!(\"prices\")"), "App adds a price route");
    assert!(
        !app.contains("path!(\"/v1/prices\")"),
        "App adds a price route"
    );
}
