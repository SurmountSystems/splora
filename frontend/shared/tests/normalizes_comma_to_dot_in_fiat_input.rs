//! Cypress it() title: normalizes comma to dot in fiat input.
//!
//! Spec: ref/mempool/frontend/cypress/e2e/mainnet/calculator.spec.ts
//! The fiat field is cleared, then the typed text `1,5` is entered.
//! The field value must match `/^1\.5/`.
//! This is the typed-text replace. It does not fetch a rate.

#[test]
fn normalizes_comma_to_dot_in_fiat_input() {
    let typed = "1,5";
    let text = splora_frontend_shared::decimal_comma_to_dot(typed);
    assert!(
        text.starts_with("1.5"),
        "fiat typed text {typed:?} must start with \"1.5\", got {text:?}"
    );
}
