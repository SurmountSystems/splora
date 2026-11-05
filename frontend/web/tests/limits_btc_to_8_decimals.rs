//! Cypress it() title: limits BTC to 8 decimals.
//!
//! Spec: `ref/mempool/frontend/cypress/e2e/mainnet/calculator.spec.ts`
//! around line 148. The bitcoin input is typed `1.123456789`. The painted
//! text splits on `.` into at most 2 parts. A fractional part is at most
//! 8 characters. The paint is not empty and is not the unchanged
//! 9-fraction input. This test calls
//! `splora_web::paint_limits_btc_to_8_decimals` and does not add that
//! function.

#[test]
fn limits_btc_to_8_decimals() {
    let painted: String = splora_web::paint_limits_btc_to_8_decimals("1.123456789");
    let parts: Vec<&str> = painted.split('.').collect();
    assert!(
        parts.len() <= 2,
        "painted BTC text must split on '.' into at most 2 parts, got {painted:?}"
    );
    if let Some(frac) = parts.get(1).filter(|frac| !frac.is_empty()) {
        assert!(
            frac.chars().count() <= 8,
            "fractional part must be at most 8 characters, got {frac:?} in {painted:?}"
        );
    }
    assert!(!painted.is_empty(), "painted BTC text must not be empty");
    assert_ne!(
        painted, "1.123456789",
        "painted BTC text must not be the unchanged 9-fraction input"
    );
}
