//! Typing in one calculator field writes the other two.
//! Source inspection only. No browser.

use splora_frontend_shared::btc_fiat_jpy;

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

fn field_slice<'a>(body: &'a str, start_name: &str, end_name: Option<&str>) -> &'a str {
    let start = body
        .find(start_name)
        .unwrap_or_else(|| panic!("CalculatorPage is missing {start_name}"));
    match end_name {
        Some(end_name) => {
            let end = body[start..]
                .find(end_name)
                .unwrap_or_else(|| panic!("CalculatorPage is missing {end_name}"));
            &body[start..start + end]
        }
        None => &body[start..],
    }
}

#[test]
fn typing_in_one_calculator_field_writes_the_other_two() {
    let calculator = component_body(views_source(), "CalculatorPage");
    let fiat_slice = field_slice(calculator, "name=fiat_name", Some("name=bitcoin_name"));
    let bitcoin_slice = field_slice(calculator, "name=bitcoin_name", Some("name=satoshis_name"));
    let satoshi_slice = field_slice(calculator, "name=satoshis_name", None);

    assert!(
        fiat_slice.contains("btc_in.set") && fiat_slice.contains("sats_in.set"),
        "entering fiat does not write the bitcoin and satoshi fields"
    );
    assert!(
        bitcoin_slice.contains("fiat_in.set") && bitcoin_slice.contains("sats_in.set"),
        "entering bitcoin does not write the fiat and satoshi fields"
    );
    assert!(
        satoshi_slice.contains("fiat_in.set") && satoshi_slice.contains("btc_in.set"),
        "entering satoshis does not write the fiat and bitcoin fields"
    );

    assert!(
        fiat_slice.contains("prop:value") && fiat_slice.contains("fiat_in"),
        "the fiat input does not show fiat_in"
    );
    assert!(
        bitcoin_slice.contains("prop:value") && bitcoin_slice.contains("btc_in"),
        "the bitcoin input does not show btc_in"
    );
    assert!(
        satoshi_slice.contains("prop:value") && satoshi_slice.contains("sats_in"),
        "the satoshi input does not show sats_in"
    );

    let stripped = calculator.replace("1.123456789", "");
    assert!(
        stripped.contains("btc_fiat_usd(\"1\")"),
        "the mocked USD rate is only a substring of 1.123456789"
    );

    assert!(
        calculator.contains("FiatCode::Jpy"),
        "CalculatorPage does not name FiatCode::Jpy"
    );
    assert!(
        calculator.contains("loaded_calculator"),
        "CalculatorPage does not call loaded_calculator"
    );
    assert_eq!(btc_fiat_jpy("1"), "\u{00A5}11,057,757");

    assert!(
        calculator.contains("enter_fiat("),
        "CalculatorPage does not call enter_fiat"
    );
    assert!(
        calculator.contains("enter_btc("),
        "CalculatorPage does not call enter_btc"
    );
    assert!(
        calculator.contains("sats_to_btc("),
        "CalculatorPage does not call sats_to_btc"
    );

    assert!(
        calculator.contains("A live price feed is not served."),
        "CalculatorPage dropped the live price feed sentence"
    );
    assert!(
        !calculator.contains("price_updated_label"),
        "CalculatorPage renders price_updated_label"
    );
    assert!(
        !calculator.contains("Fiat price last updated"),
        "CalculatorPage renders Fiat price last updated"
    );
    assert_eq!(calculator.matches("class=\"copy-control\"").count(), 3);
}
