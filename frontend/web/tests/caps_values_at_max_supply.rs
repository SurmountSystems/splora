//! Cypress it() title: caps values at max supply.
//!
//! Spec: `ref/mempool/frontend/cypress/e2e/mainnet/calculator.spec.ts`
//! lines 176-180. Typed bitcoin text `25000000` shows the warning and
//! the bitcoin text equals `21000000`.
//! This test calls `splora_web::paint_shows_warning_when_entering_21m_btc`
//! and `splora_frontend_shared::shows_warning_when_entering_21m_btc`.
//! It does not add a paint function.

#[test]
fn caps_values_at_max_supply() {
    let painted = splora_web::paint_shows_warning_when_entering_21m_btc("25000000");
    let shared = splora_frontend_shared::shows_warning_when_entering_21m_btc("25000000");
    assert!(
        painted
            .warning
            .contains("Values were capped at the max supply of 21M BTC"),
        "warning for \"25000000\" must contain the 21M supply cap sentence, got {:?}",
        painted.warning
    );
    assert_eq!(
        painted.bitcoin, "21000000",
        "bitcoin text from \"25000000\" must equal \"21000000\""
    );
    assert_eq!(
        painted.warning, shared.warning,
        "painted warning for \"25000000\" must equal shared shows_warning_when_entering_21m_btc"
    );
    assert_eq!(
        painted.bitcoin, shared.bitcoin,
        "painted bitcoin text for \"25000000\" must equal shared shows_warning_when_entering_21m_btc"
    );
}
