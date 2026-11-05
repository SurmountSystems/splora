//! Cypress it() title: limits BTC to 8 decimals.
//!
//! Mainnet spec
//! `ref/mempool/frontend/cypress/e2e/mainnet/calculator.spec.ts`
//! types `1.123456789` into the bitcoin input. The value must split on `.`
//! into at most two parts. When a fractional part is present, that part is
//! at most 8 characters.
//! The painted text must equal
//! `splora_frontend_shared::truncate_btc_decimals("1.123456789")`.
//! A second dot, a fractional part longer than 8 characters, or a result
//! that differs from the shared return fails.
//! `paint_limits_btc_to_8_decimals` is not defined
//! on `splora_native`. This test does not add it.

#[test]
fn limits_btc_to_8_decimals() {
    let painted: String = splora_native::paint_limits_btc_to_8_decimals("1.123456789");
    let shared: String = splora_frontend_shared::truncate_btc_decimals("1.123456789");
    let mut parts = painted.split('.');
    let _whole = parts.next();
    let frac = parts.next();
    assert!(
        parts.next().is_none(),
        "painted BTC text must contain at most one dot, got {painted:?}"
    );
    if let Some(frac) = frac.filter(|frac| !frac.is_empty()) {
        assert!(
            frac.chars().count() <= 8,
            "fractional part must be at most 8 characters, got {painted:?}"
        );
    }
    assert_eq!(
        painted, shared,
        "painted BTC text must equal truncate_btc_decimals(\"1.123456789\")"
    );
}
