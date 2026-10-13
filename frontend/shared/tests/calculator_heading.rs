use splora_frontend_shared::{
    FIXTURE_JPY_PER_BTC, FIXTURE_USD_PER_BTC, FiatCode, btc_fiat_jpy, btc_fiat_usd, enter_btc,
    enter_fiat, jpy_to_btc_and_sats, loaded_calculator, path_list, route_availability,
    usd_to_btc_and_sats,
};

fn assert_no_price_route() {
    assert_eq!(route_availability("/v1/prices"), "not-served");
    let routes = path_list();
    assert!(routes.iter().all(|route| route.pattern != "/v1/prices"));
}

#[test]
fn calculator_heading_and_fiat_btc_sats_inputs() {
    let form = loaded_calculator(FiatCode::Usd);
    assert_eq!(form.heading, "Calculator");
    assert!(form.inputs_present());
    assert_eq!(form.btc, "1");
    assert_eq!(form.sats, "100000000");
    assert_eq!(form.fiat, FIXTURE_USD_PER_BTC.to_string());
    assert!(!form.waiting_for_price);
    assert_no_price_route();
}

#[test]
fn symbol_includes_fixture_usd_rate() {
    let form = loaded_calculator(FiatCode::Usd);
    let symbol = form.symbol.replace(',', "");
    assert!(symbol.contains(&FIXTURE_USD_PER_BTC.to_string()));
    assert_eq!(form.symbol, btc_fiat_usd("1"));
    assert_no_price_route();
}

#[test]
fn three_copy_controls_for_the_inputs() {
    let form = loaded_calculator(FiatCode::Usd);
    assert_eq!(form.copy_buttons.len(), 3);
    assert!(form.copy_buttons.iter().all(|label| !label.is_empty()));
    assert_no_price_route();
}

#[test]
fn fiat_price_label_and_bitcoin_visual() {
    let form = loaded_calculator(FiatCode::Usd);
    assert_eq!(form.price_updated_label, "Fiat price last updated");
    assert!(form.bitcoin_visual.contains('\u{20BF}'));
    assert_eq!(form.fiat_text, btc_fiat_usd("1"));
    assert_no_price_route();
}

#[test]
fn btc_and_sats_input_labels() {
    let form = loaded_calculator(FiatCode::Usd);
    assert_eq!(form.label_btc, "BTC");
    assert_eq!(form.label_sats, "sats");
    assert!(form.inputs_present());
    assert_no_price_route();
}

#[test]
fn entering_one_hundred_usd_updates_btc_and_sats() {
    let quote = usd_to_btc_and_sats("100").expect("100 USD converts");
    assert_eq!(quote.sats, 81_001);
    let form = enter_fiat(&loaded_calculator(FiatCode::Usd), "100").expect("100 USD");
    assert_eq!(form.fiat, "100");
    assert_eq!(form.sats, "81001");
    assert_eq!(form.btc, "0.00081001");
    assert_no_price_route();
}

#[test]
fn three_copy_buttons_are_present() {
    let form = loaded_calculator(FiatCode::Usd);
    assert_eq!(form.copy_buttons, ["fiat", "btc", "sats"]);
    assert_no_price_route();
}

#[test]
fn btc_input_stays_usable_on_desktop() {
    let form = enter_btc(&loaded_calculator(FiatCode::Usd), "1");
    assert!(form.btc.contains('1'));
    assert_eq!(form.sats, "100000000");
    assert!(form.inputs_present());
    assert_no_price_route();
}

#[test]
fn fiat_btc_and_sats_inputs_stay_present_on_mobile() {
    let form = loaded_calculator(FiatCode::Usd);
    assert!(form.inputs_present());
    assert!(!form.fiat.is_empty());
    assert!(!form.btc.is_empty());
    assert!(!form.sats.is_empty());
    assert_no_price_route();
}

#[test]
fn fixture_rate_shows_the_form_without_a_price_route() {
    let form = loaded_calculator(FiatCode::Usd);
    assert!(!form.waiting_for_price);
    assert_ne!(form.heading, "Waiting for price feed...");
    assert!(!form.btc.is_empty());
    assert_no_price_route();
}

#[test]
fn symbol_includes_fixture_jpy_rate() {
    let form = loaded_calculator(FiatCode::Jpy);
    let symbol = form.symbol.replace(',', "");
    assert!(symbol.contains(&FIXTURE_JPY_PER_BTC.to_string()));
    assert_eq!(form.fiat_text, btc_fiat_jpy("1"));
    assert_eq!(form.fiat, FIXTURE_JPY_PER_BTC.to_string());
    assert_eq!(form.sats, "100000000");
    assert_no_price_route();
}

#[test]
fn entering_one_million_jpy_updates_btc_and_sats() {
    let quote = jpy_to_btc_and_sats("1000000").expect("1000000 JPY converts");
    assert_eq!(quote.numerator, 1_000_000);
    assert_eq!(quote.denominator, u128::from(FIXTURE_JPY_PER_BTC));
    assert_eq!(quote.sats, 9_043_425);
    let form = enter_fiat(&loaded_calculator(FiatCode::Jpy), "1000000").expect("yen");
    assert_eq!(form.sats, "9043425");
    assert_eq!(form.btc, "0.09043425");
    assert_no_price_route();
}
