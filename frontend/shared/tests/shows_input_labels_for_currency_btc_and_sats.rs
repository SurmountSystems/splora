//! Cypress it() title: shows input labels for currency, BTC, and sats.
//!
//! Spec: ref/mempool/frontend/cypress/e2e/mainnet/calculator.spec.ts line 66.
//! The it() requires `.input-group-text` to contain `BTC` and `sats`, and
//! each match to be visible. The calculator template has three of those
//! spans, in order: `currency$ | async`, the literal `BTC`, and the
//! literal `sats`. With no stored fiat preference, `fiatCurrency$` is
//! `USD`. This does not read a price feed.

#[test]
fn shows_input_labels_for_currency_btc_and_sats() {
    let labels = splora_frontend_shared::shows_input_labels_for_currency_btc_and_sats();
    assert_eq!(
        labels.btc, "BTC",
        "cy.get('.input-group-text').contains('BTC').should('be.visible')"
    );
    assert_eq!(
        labels.sats, "sats",
        "cy.get('.input-group-text').contains('sats').should('be.visible')"
    );
    assert!(labels.btc_visible, "the BTC input-group-text is visible");
    assert!(labels.sats_visible, "the sats input-group-text is visible");
    assert!(
        labels.currency_visible,
        "the currency input-group-text is visible"
    );
    assert_eq!(
        labels.currency, "USD",
        "fiatCurrency$ is fiatPreference or USD when the calculator page loads"
    );
    assert_eq!(
        labels.group_text(),
        [labels.currency, labels.btc, labels.sats],
        "template order is currency, BTC, then sats"
    );
    assert!(
        labels
            .group_text()
            .iter()
            .any(|text| text.contains(labels.btc)),
        "BTC is one .input-group-text"
    );
    assert!(
        labels
            .group_text()
            .iter()
            .any(|text| text.contains(labels.sats)),
        "sats is one .input-group-text"
    );
}
