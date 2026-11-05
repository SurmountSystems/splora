//! Cypress it() title: limits BTC to 8 decimals.
//!
//! Spec: ref/mempool/frontend/cypress/e2e/mainnet/calculator.spec.ts
//! The bitcoin field is cleared, then the typed text `1.123456789` is entered.
//! The field value splits on '.' into at most two parts. If a fractional part
//! exists, that part is at most 8 characters. This is a decimal-place cap.
//! It does not fetch a rate.

#[test]
fn limits_btc_to_8_decimals() {
    let typed = "1.123456789";
    let text = splora_frontend_shared::truncate_btc_decimals(typed);
    let parts: Vec<&str> = text.split('.').collect();
    assert!(
        parts.len() <= 2,
        "bitcoin field {text:?} from {typed:?} must split on '.' into at most 2 parts"
    );
    if let Some(frac) = parts.get(1) {
        assert!(
            frac.len() <= 8,
            "bitcoin fractional part {frac:?} from {typed:?} must be at most 8 characters"
        );
    }
}
