//! Calculator fixture fiat amounts. The displayed strings come from
//! `splora_frontend_shared`. This test does not compute them. The calculator
//! view has to call those exports, and the text it returns has to contain
//! every fixture string.

use splora_frontend_shared::{btc_fiat_jpy, btc_fiat_usd, sats_fiat_usd};
use splora_web::calculator_fiat_text;

fn views_source() -> &'static str {
    include_str!("../src/views.rs")
}

fn local_pages_source() -> &'static str {
    include_str!("../src/local_pages.rs")
}

/// Body inside the function braces, not the next function.
fn function_body<'a>(src: &'a str, name: &str) -> &'a str {
    let marker = format!("fn {name}(");
    let start = src
        .find(&marker)
        .unwrap_or_else(|| panic!("{name} is missing"));
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

#[test]
fn calculator_shows_shared_fixture_fiat_amounts() {
    let third_btc_usd = btc_fiat_usd("0.33");
    let one_sat_usd = btc_fiat_usd("0.00000001");
    let one_btc_jpy = btc_fiat_jpy("1");
    let ten_thousand_sats_usd = sats_fiat_usd(10_000);
    assert_eq!(third_btc_usd, "$40,740.48");
    // `0.00000001` BTC is one satoshi.
    assert_eq!(one_sat_usd, "$0.00");
    assert_eq!(one_btc_jpy, "\u{00A5}11,057,757");
    // 10000 sats is 0.0001 BTC. Shared rounding produces `$12.35`.
    assert_eq!(ten_thousand_sats_usd, "$12.35");

    let src = views_source();
    for name in ["btc_fiat_usd", "btc_fiat_jpy", "sats_fiat_usd"] {
        assert!(
            shared_use_imports(src, name),
            "{name} is not imported from splora_frontend_shared"
        );
        assert!(
            !src.contains(&format!("fn {name}(")),
            "{name} is copied in views.rs"
        );
    }

    let calculator = function_body(src, "CalculatorPage");
    for call in [
        "btc_fiat_usd(\"0.33\")",
        "btc_fiat_usd(\"0.00000001\")",
        "btc_fiat_jpy(\"1\")",
        "sats_fiat_usd(10_000)",
        "calculator_fiat_text()",
    ] {
        assert!(
            code_has_call(calculator, call),
            "CalculatorPage does not call {call}"
        );
    }
    for literal in ["$40,740.48", "$0.00", "$12.35", "11,057,757"] {
        assert!(
            !calculator.contains(literal),
            "CalculatorPage hardcodes {literal} instead of calling shared"
        );
    }

    let returned = function_body(local_pages_source(), "calculator_fiat_text");
    for call in [
        "btc_fiat_usd(\"0.33\")",
        "btc_fiat_usd(\"0.00000001\")",
        "btc_fiat_jpy(\"1\")",
        "sats_fiat_usd(10_000)",
    ] {
        assert!(
            code_has_call(returned, call),
            "calculator_fiat_text does not call {call}"
        );
    }
    for literal in ["$40,740.48", "$0.00", "$12.35", "11,057,757"] {
        assert!(
            !returned.contains(literal),
            "calculator_fiat_text hardcodes {literal}"
        );
    }

    let shown = calculator_fiat_text();
    for expected in [
        third_btc_usd,
        one_sat_usd,
        one_btc_jpy,
        ten_thousand_sats_usd,
    ] {
        assert!(
            shown.contains(&expected),
            "calculator text missing {expected}: {shown}"
        );
    }
}
