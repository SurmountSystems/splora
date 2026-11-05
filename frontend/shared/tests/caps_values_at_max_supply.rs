//! Cypress it() title: caps values at max supply.
//!
//! Spec: ref/mempool/frontend/cypress/e2e/mainnet/calculator.spec.ts
//! lines 176-180. The bitcoin field is cleared, then the typed text
//! `25000000` is entered. The warning alert is visible. The bitcoin
//! field equals `21000000`. This is local supply-cap formatting.
//! It does not assert a satoshi string. It does not assert a price.

#[test]
fn caps_values_at_max_supply() {
    let entered = splora_frontend_shared::shows_warning_when_entering_21m_btc("25000000");
    assert!(
        entered
            .warning
            .contains("Values were capped at the max supply of 21M BTC"),
        "warning for \"25000000\" must contain the 21M supply cap sentence, got {:?}",
        entered.warning
    );
    assert_eq!(
        entered.bitcoin, "21000000",
        "bitcoin field from \"25000000\" must equal \"21000000\""
    );
}
