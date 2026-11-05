//! Cypress it() title: strips decimals from satoshis input.
//!
//! Spec: ref/mempool/frontend/cypress/e2e/mainnet/calculator.spec.ts
//! The satoshis field is cleared, then the typed text `10000.99` is entered.
//! The field value must not include '.'. This is local field formatting.
//! It does not fetch a rate.

#[test]
fn strips_decimals_from_satoshis_input() {
    let typed = "10000.99";
    let text = splora_frontend_shared::strips_decimals_from_satoshis_input(typed);
    assert_eq!(
        text, "10001",
        "satoshis field from {typed:?} must equal \"10001\""
    );
    assert!(
        !text.contains('.'),
        "satoshis field {text:?} from {typed:?} must not contain '.'"
    );

    let trailing = "10000.";
    let trailing_text = splora_frontend_shared::strips_decimals_from_satoshis_input(trailing);
    assert_eq!(
        trailing_text, "10000",
        "satoshis field from {trailing:?} must equal \"10000\""
    );
    assert!(
        !trailing_text.contains('.'),
        "satoshis field {trailing_text:?} from {trailing:?} must not contain '.'"
    );
}
