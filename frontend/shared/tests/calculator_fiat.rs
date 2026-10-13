use splora_frontend_shared::{
    FiatCode, btc_fiat_jpy, btc_fiat_usd, btc_to_sats, cap_btc_supply, decimal_comma_to_dot,
    format_fiat, path_list, route_availability, sats_fiat_usd, sats_to_btc, strip_sats_decimal,
    switch_usd_jpy_usd, truncate_btc_decimals, usd_to_btc_and_sats,
};

#[test]
fn one_btc_converts_to_one_hundred_million_sats() {
    assert_eq!(btc_to_sats("1").unwrap(), 100_000_000);
}

#[test]
fn zero_point_three_three_btc_converts_to_thirty_three_million_sats() {
    assert_eq!(btc_to_sats("0.33").unwrap(), 33_000_000);
}

#[test]
fn one_sat_btc_input_converts_to_one_sat() {
    assert_eq!(btc_to_sats("0.00000001").unwrap(), 1);
}

#[test]
fn half_btc_converts_to_fifty_million_sats() {
    assert_eq!(btc_to_sats("0.5").unwrap(), 50_000_000);
}

#[test]
fn one_sat_formats_as_btc() {
    assert_eq!(sats_to_btc(1), "0.00000001");
}

#[test]
fn ten_thousand_sats_formats_as_btc() {
    assert_eq!(sats_to_btc(10_000), "0.00010000");
}

#[test]
fn btc_input_truncates_to_eight_decimals_without_rounding() {
    let limited = truncate_btc_decimals("1.123456789");
    assert_eq!(limited, "1.12345678");
    assert_ne!(limited, "1.12345679");
    let mut parts = limited.split('.');
    let _whole = parts.next().unwrap();
    let frac = parts.next().unwrap();
    assert!(parts.next().is_none());
    assert!(frac.len() <= 8);
    assert_eq!(frac, "12345678");
}

#[test]
fn shorter_btc_inputs_do_not_gain_trailing_zeros() {
    assert_eq!(truncate_btc_decimals("1"), "1");
    assert_eq!(truncate_btc_decimals("0.33"), "0.33");
    assert_eq!(truncate_btc_decimals("1.2"), "1.2");
}

#[test]
fn sats_input_strips_the_decimal_point_and_keeps_surrounding_characters() {
    assert_eq!(strip_sats_decimal("100.00"), "10000");
    let stripped = strip_sats_decimal("10000.99");
    assert!(!stripped.contains('.'));
    assert_eq!(stripped, "1000099");
}

#[test]
fn comma_in_a_decimal_field_becomes_a_dot() {
    assert_eq!(decimal_comma_to_dot("0,33"), "0.33");
    let typed = decimal_comma_to_dot("1,5");
    assert!(
        typed.starts_with("1.5"),
        "expected a 1.5 prefix, got {typed}"
    );
}

#[test]
fn twenty_one_million_btc_stays_and_is_flagged_as_the_cap() {
    let capped = cap_btc_supply("21000000");
    assert_eq!(capped.btc, "21000000");
    assert!(capped.capped);
    assert_eq!(btc_to_sats(&capped.btc).unwrap(), 2_100_000_000_000_000);
}

#[test]
fn twenty_five_million_btc_becomes_the_cap_and_is_flagged() {
    let capped = cap_btc_supply("25000000");
    assert_eq!(capped.btc, "21000000");
    assert!(capped.capped);
}

#[test]
fn btc_under_the_cap_is_not_flagged() {
    let one = cap_btc_supply("1");
    assert_eq!(one.btc, "1");
    assert!(!one.capped);
    let below = cap_btc_supply("20999999");
    assert_eq!(below.btc, "20999999");
    assert!(!below.capped);
    let fractional = cap_btc_supply("0.33");
    assert_eq!(fractional.btc, "0.33");
    assert!(!fractional.capped);
}

#[test]
fn usd_whole_amount_has_no_cents() {
    let shown = format_fiat(FiatCode::Usd, "12");
    assert_eq!(shown, "$12");
    assert!(shown.starts_with('$'));
    assert!(!shown.contains('.'));
}

#[test]
fn usd_non_whole_amount_shows_two_cent_digits() {
    let shown = format_fiat(FiatCode::Usd, "1.50");
    assert_eq!(shown, "$1.50");
    assert!(shown.starts_with('$'));
    let frac = shown.split('.').nth(1).expect("two cent digits");
    assert_eq!(frac.len(), 2);
    assert_eq!(format_fiat(FiatCode::Usd, "1.5"), "$1.50");
}

#[test]
fn jpy_uses_yen_and_no_decimal_point() {
    let shown = format_fiat(FiatCode::Jpy, "1500");
    assert_eq!(shown, "\u{00A5}1500");
    assert!(shown.contains('\u{00A5}'));
    assert!(!shown.contains('.'));
    assert!(!shown.contains('\u{FFE5}'));
}

#[test]
fn switching_usd_to_jpy_and_back_changes_the_symbol() {
    let shown = switch_usd_jpy_usd("1500");
    assert_eq!(shown[0].chars().next(), Some('$'));
    assert_eq!(shown[1].chars().next(), Some('\u{00A5}'));
    assert_eq!(shown[2].chars().next(), Some('$'));
    assert!(!shown[1].contains('.'));
    assert_eq!(shown[0], "$1500");
    assert_eq!(shown[1], "\u{00A5}1500");
    assert_eq!(shown[2], "$1500");
}

#[test]
fn zero_point_three_three_btc_formats_as_forty_thousand_usd() {
    assert_eq!(btc_fiat_usd("0.33"), "$40,740.48");
}

#[test]
fn one_sat_formats_as_zero_usd() {
    assert_eq!(btc_fiat_usd("0.00000001"), "$0.00");
}

#[test]
fn one_btc_formats_as_jpy_with_thousands_separators() {
    assert_eq!(btc_fiat_jpy("1"), "\u{00A5}11,057,757");
}

#[test]
fn one_hundred_usd_converts_by_the_exact_rate_ratio() {
    let quote = usd_to_btc_and_sats("100").expect("100 USD converts");
    assert_eq!(quote.numerator, 100);
    assert_eq!(quote.denominator, 123_456);
    let numer = 100_u128 * 100_000_000;
    let denom = 123_456_u128;
    let quotient = numer / denom;
    let remainder = numer % denom;
    let sats = if remainder * 2 >= denom {
        quotient + 1
    } else {
        quotient
    };
    assert_eq!(quote.sats, u64::try_from(sats).unwrap());
}

#[test]
fn ten_thousand_sats_formats_as_rounded_usd_cents() {
    // 10000 sats is 0.0001 BTC. 0.0001 * 123456 = 12.3456, which is $12.35.
    let shown = sats_fiat_usd(10_000);
    assert_eq!(shown, "$12.35");
    assert_ne!(shown, "12.3456");
}

#[test]
fn statistics_mining_and_lightning_are_not_served() {
    for path in [
        "/api/v1/statistics",
        "/v1/mining",
        "/v1/prices",
        "/lightning",
        "/api/v1/lightning/statistics/latest",
        "/Lightning/nodes",
    ] {
        assert_eq!(route_availability(path), "not-served", "{path}");
    }
    assert_ne!(route_availability("/blocks"), "not-served");

    let routes = path_list();
    for path in [
        "/api/v1/statistics",
        "/v1/mining",
        "/v1/prices",
        "/lightning",
        "/api/v1/lightning/statistics/latest",
    ] {
        assert!(
            routes.iter().all(|route| route.pattern != path),
            "{path} must not be registered"
        );
    }
    assert!(
        routes
            .iter()
            .all(|route| !route.pattern.to_ascii_lowercase().contains("lightning")),
        "a lightning path must not be registered"
    );
}

#[test]
fn historical_price_is_not_checked_and_is_not_a_served_price_api() {
    let path = "/api/v1/historical-price";
    assert_eq!(route_availability(path), "not-checked");
    assert_eq!(route_availability("/v1/prices"), "not-served");
    let routes = path_list();
    assert!(
        routes.iter().all(|route| route.pattern != path),
        "{path} must not be registered"
    );
    assert!(
        routes.iter().all(|route| {
            !route
                .pattern
                .to_ascii_lowercase()
                .contains("historical-price")
        }),
        "historical-price must not be a served price API"
    );
}

#[test]
fn replacements_and_tx_rbf_paths_are_not_served() {
    let paths = [
        "/api/v1/replacements",
        "/api/v1/fullrbf/replacements",
        "/api/v1/tx/:txid/rbf",
        "/api/v1/tx/f81a08699b62b2070ad8fe0f2a076f8bea0386a2fdcd8124caee42cbc564a0d5/rbf",
    ];
    for path in paths {
        assert_eq!(route_availability(path), "not-served", "{path}");
    }
    let routes = path_list();
    for path in paths {
        assert!(
            routes.iter().all(|route| route.pattern != path),
            "{path} must not be registered"
        );
    }
    assert_ne!(
        route_availability("/tx/f81a08699b62b2070ad8fe0f2a076f8bea0386a2fdcd8124caee42cbc564a0d5"),
        "not-served"
    );
    assert_eq!(
        route_availability("/api/v1/historical-price"),
        "not-checked"
    );
    assert_eq!(route_availability("/api/v1/statistics"), "not-served");
    assert_eq!(route_availability("/v1/mining"), "not-served");
    assert_eq!(route_availability("/lightning"), "not-served");
}

#[test]
fn base_module_skip_guards_assert_no_indexer_route() {
    // Eight Cypress `it.skip` titles. The callback is absent, so no path is visited.
    let files = [
        "liquid/liquid.spec.ts",
        "liquidtestnet/liquidtestnet.spec.ts",
        "mainnet/calculator.spec.ts",
        "mainnet/fiat-currency-formatting.spec.ts",
        "mainnet/mainnet.spec.ts",
        "mainnet/mining.spec.ts",
        "signet/signet.spec.ts",
        "testnet4/testnet4.spec.ts",
    ];
    assert_eq!(files.len(), 8);
    let routes = path_list();
    for route in &routes {
        assert!(!route.pattern.contains("BASE_MODULE"), "{}", route.pattern);
        assert_ne!(route_availability(route.pattern), "not-served");
    }
}

#[test]
fn docs_click_is_not_an_electrs_route() {
    // The Cypress `it` clicks `#btn-docs`, which links to `/docs`, then waits.
    let path = "/docs";
    assert_eq!(route_availability(path), "not-checked");
    let routes = path_list();
    assert!(routes.iter().all(|route| route.pattern != path));
    assert!(routes.iter().all(|route| route.pattern != "/api"));
    for served in [
        "/blocks",
        "/tx/:txid",
        "/address/:script",
        "/mempool/recent",
    ] {
        assert!(
            routes.iter().any(|route| route.pattern == served),
            "{served} is an indexer route"
        );
    }
}

#[test]
fn skipped_dashboard_socket_reconnect_is_not_the_nip98_gate() {
    // The skipped Cypress case visits `/` and drops a dashboard socket.
    // GET /api/v1/ws is the NIP-98 gate. It is not that socket, and it is not
    // in the statistics, mining, or lightning not-served set.
    let routes = path_list();
    assert!(routes.iter().all(|route| route.pattern != "/"));
    assert!(routes.iter().all(|route| route.pattern != "/api/v1/ws"));
    assert_eq!(route_availability("/api/v1/ws"), "not-checked");
    assert_ne!(route_availability("/api/v1/ws"), "not-served");
}
