//! Cypress it() title: displays JPY values without decimal places.
//!
//! Spec: ref/mempool/frontend/cypress/e2e/mainnet/fiat-currency-formatting.spec.ts
//! The assertion is expect(trimmedText).to.not.match(/\.\d+$/).
//! This calls a shared function that is not defined. It does not read an indexer body.

#[test]
fn displays_jpy_values_without_decimal_places() {
    let text: String = splora_frontend_shared::displays_jpy_values_without_decimal_places();
    let trimmed = text.trim();
    let ends_with_dot_digits = match trimmed.rfind('.') {
        Some(index) => {
            let frac = &trimmed[index + 1..];
            !frac.is_empty() && frac.bytes().all(|byte| byte.is_ascii_digit())
        }
        None => false,
    };
    assert!(
        !ends_with_dot_digits,
        "trimmed fiat text must not match /\\.\\d+$/, got {trimmed:?}"
    );
}
