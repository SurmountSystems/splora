//! Cypress it() title: normalizes comma to dot in fiat input.
//!
//! Mainnet spec
//! `ref/mempool/frontend/cypress/e2e/mainnet/calculator.spec.ts`
//! types `1,5` into the fiat input and expects the value to match `/^1\.5/`
//! (`expect(String(val)).to.match(/^1\.5/)`).
//! The painted text must start with `1.5` and must equal
//! `splora_frontend_shared::decimal_comma_to_dot("1,5")`.
//! A result that is still `1,5`, that does not start with `1.5`, or that
//! differs from the shared return fails.
//! `paint_normalizes_comma_to_dot_in_fiat_input` is not defined
//! on `splora_native`. This test does not add it.

#[test]
fn normalizes_comma_to_dot_in_fiat_input() {
    let painted: String = splora_native::paint_normalizes_comma_to_dot_in_fiat_input("1,5");
    let shared: String = splora_frontend_shared::decimal_comma_to_dot("1,5");
    assert!(
        painted.starts_with("1.5"),
        "painted fiat text must start with \"1.5\", got {painted:?}"
    );
    assert_eq!(
        painted, shared,
        "painted fiat text must equal decimal_comma_to_dot(\"1,5\")"
    );
}
