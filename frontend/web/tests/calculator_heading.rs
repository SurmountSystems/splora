//! Calculator page heading. No waiting-for-price sentence and no price feed.

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

#[test]
fn loads_the_calculator_page_with_heading_and_form() {
    let calculator = component_body(views_source(), "CalculatorPage");
    assert!(
        calculator.contains("<h2>\"Calculator\"</h2>"),
        "CalculatorPage is missing the Calculator heading"
    );
    assert!(
        calculator.contains("123456"),
        "CalculatorPage is missing the digits 123456"
    );
    assert!(
        calculator.contains("₿"),
        "CalculatorPage is missing the bitcoin symbol"
    );
    assert!(
        !calculator.contains("Waiting for price feed..."),
        "CalculatorPage contains Waiting for price feed..."
    );
    assert!(
        calculator.contains("fiat"),
        "CalculatorPage is missing the fiat label"
    );
    assert!(
        calculator.contains("BTC"),
        "CalculatorPage is missing the BTC label"
    );
    assert!(
        calculator.contains("sats"),
        "CalculatorPage is missing the sats label"
    );
    assert!(
        calculator.contains("<input"),
        "CalculatorPage is missing an input"
    );
    assert_eq!(
        calculator.matches("class=\"copy-control\"").count(),
        3,
        "views.rs does not contain three copy controls"
    );
}

#[test]
fn views_file_contains_incoming_virtual_bytes_rate_sentence() {
    let src = include_str!("../src/views.rs");
    assert!(
        src.contains("The incoming virtual-bytes-per-second rate is not in the payload."),
        "views.rs is missing the incoming virtual-bytes-per-second rate sentence"
    );
}
