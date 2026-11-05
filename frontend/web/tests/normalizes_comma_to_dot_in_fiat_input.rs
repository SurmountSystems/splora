//! Cypress it() title: normalizes comma to dot in fiat input.
//!
//! Spec: `ref/mempool/frontend/cypress/e2e/mainnet/calculator.spec.ts`
//! around line 141. The fiat input is typed `1,5`. The field value must
//! match `/^1\.5/`.
//! `paint_normalizes_comma_to_dot_in_fiat_input` is not defined on
//! `splora_web`. This test does not add it.
//! The shared normalizer is `decimal_comma_to_dot` in
//! `frontend/shared/src/fiat.rs`, re-exported from `frontend/shared/src/lib.rs`.

#[test]
fn normalizes_comma_to_dot_in_fiat_input() {
    let painted: String = splora_web::paint_normalizes_comma_to_dot_in_fiat_input("1,5");
    let shared: String = splora_frontend_shared::decimal_comma_to_dot("1,5");
    assert!(
        painted.starts_with("1.5") && painted.starts_with(shared.as_str()),
        "painted fiat text must start with \"1.5\" and with decimal_comma_to_dot(\"1,5\") = {shared:?}, got {painted:?}"
    );
    assert_eq!(
        painted, shared,
        "painted fiat text must equal decimal_comma_to_dot(\"1,5\")"
    );
}
