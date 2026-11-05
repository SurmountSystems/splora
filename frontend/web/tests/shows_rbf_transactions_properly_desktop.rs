//! Cypress it() title: shows RBF transactions properly (desktop).
//!
//! Spec: ref/mempool/frontend/cypress/e2e/mainnet/mainnet.spec.ts
//! Viewport name `macbook-16`. The vendored Cypress config does not
//! define that preset's pixel size.
//!
//! Selectors and checks, from that it block:
//! `alertLocator` is `.alert`. `tableLocator` is `.container-xl > :nth-child(3)`.
//! `cy.get(tableLocator).invoke('css', 'width')` equals
//! `cy.get(alertLocator).invoke('css', 'width')`.
//! `getRectangle` is `getBoundingClientRect` on `.btn-warning` and `.alert`.
//! `areOverlapping` of those two rectangles is false. The expectation text
//! is `Confirmations box and RBF alert are overlapping`.
//!
//! Bodies are the Cypress fixtures for the visited cached transaction and
//! the `rbfTransaction` websocket file `mainnet_rbf.json` (`txReplaced`).

const CACHED_TX: &str =
    include_str!("../../../ref/mempool/frontend/cypress/fixtures/mainnet_tx_cached.json");
const REPLACEMENT_SOCKET: &str =
    include_str!("../../../ref/mempool/frontend/cypress/fixtures/mainnet_rbf.json");

#[test]
fn shows_rbf_transactions_properly_desktop() {
    let measured = splora_web::measure_desktop_rbf_transaction_paint(
        CACHED_TX,
        REPLACEMENT_SOCKET,
        "macbook-16",
    );
    println!(
        "alert_css_width={} third_child_css_width={} overlap={} alert_rect={:?} button_rect={:?} viewport={}",
        measured.alert_css_width,
        measured.third_child_css_width,
        measured.confirmations_and_alert_overlap,
        measured.alert_rect,
        measured.button_rect,
        measured.viewport,
    );
    assert_eq!(
        measured.third_child_css_width, measured.alert_css_width,
        "desktop replace-by-fee css widths differ"
    );
    assert!(
        !measured.confirmations_and_alert_overlap,
        "Confirmations box and RBF alert are overlapping"
    );
}
