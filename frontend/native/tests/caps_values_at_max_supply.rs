//! Cypress it() title: caps values at max supply.
//!
//! Spec: `ref/mempool/frontend/cypress/e2e/mainnet/calculator.spec.ts`
//! lines 176-180. Typed bitcoin text `25000000` shows a warning containing
//! "Values were capped at the max supply of 21M BTC". The bitcoin text
//! equals `21000000`. Painted warning and bitcoin text equal
//! `splora_frontend_shared::shows_warning_when_entering_21m_btc`.
//! This test does not open a window.

#[test]
fn caps_values_at_max_supply() {
    let typed = "25000000";
    let painted = splora_native::paint_shows_warning_when_entering_21m_btc(typed);
    let shared = splora_frontend_shared::shows_warning_when_entering_21m_btc(typed);
    assert!(
        painted
            .warning
            .contains("Values were capped at the max supply of 21M BTC"),
        "warning for {typed:?} must contain the 21M supply cap sentence, got {:?}",
        painted.warning
    );
    assert_eq!(
        painted.bitcoin, "21000000",
        "bitcoin text from {typed:?} must equal \"21000000\""
    );
    assert_eq!(
        painted.warning, shared.warning,
        "painted warning for {typed:?} must equal shared shows_warning_when_entering_21m_btc"
    );
    assert_eq!(
        painted.bitcoin, shared.bitcoin,
        "painted bitcoin text for {typed:?} must equal shared shows_warning_when_entering_21m_btc"
    );
}
