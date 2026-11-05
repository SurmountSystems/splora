//! Cypress it() title: shows warning when entering 21M BTC.
//!
//! Spec: ref/mempool/frontend/cypress/e2e/mainnet/calculator.spec.ts
//! The bitcoin field is cleared, then the typed text `21000000` is entered.
//! The warning contains "Values were capped at the max supply of 21M BTC".
//! The bitcoin field equals `21000000`. The satoshis field equals
//! `2100000000000000`. Typed text `1` does not show that warning, and its
//! satoshi text equals `100000000`. This is the local supply cap.
//! It does not fetch a rate.

#[test]
fn shows_warning_when_entering_21m_btc() {
    let typed = "21000000";
    let entered = splora_frontend_shared::shows_warning_when_entering_21m_btc(typed);
    assert!(
        entered
            .warning
            .contains("Values were capped at the max supply of 21M BTC"),
        "warning for {typed:?} must contain the 21M supply cap sentence, got {:?}",
        entered.warning
    );
    assert_eq!(
        entered.bitcoin, "21000000",
        "bitcoin field from {typed:?} must equal \"21000000\""
    );
    assert_eq!(
        entered.satoshis, "2100000000000000",
        "satoshi field from {typed:?} must equal \"2100000000000000\""
    );

    let one = "1";
    let below = splora_frontend_shared::shows_warning_when_entering_21m_btc(one);
    assert!(
        !below
            .warning
            .contains("Values were capped at the max supply of 21M BTC"),
        "typed text {one:?} must not show the 21M supply cap warning, got {:?}",
        below.warning
    );
    assert_eq!(
        below.satoshis, "100000000",
        "satoshi field from {one:?} must equal \"100000000\""
    );
}
