//! Cypress title: shows calculator form after price feed loads.
//!
//! Spec: ref/mempool/frontend/cypress/e2e/mainnet/calculator.spec.ts
//! lines 209-212. The loaded calculator form does not contain
//! `Waiting for price feed...`, and the bitcoin input is visible.
//! This test calls `paint_shows_calculator_form_after_price_feed_loads`.
//! That paint opens one headless window with `show: false` and quits
//! before it returns. Text and bitcoin visibility come from
//! `splora_frontend_shared::shows_calculator_form_after_price_feed_loads`.
//! This test does not invent a price, a block hash, a fee, or the heading
//! `1 transaction`. It does not call a keypress paint.

#[test]
fn shows_calculator_form_after_price_feed_loads() {
    let painted = splora_native::paint_shows_calculator_form_after_price_feed_loads();
    let shared = splora_frontend_shared::shows_calculator_form_after_price_feed_loads();
    assert!(
        !painted.text.contains("Waiting for price feed..."),
        "cy.contains('Waiting for price feed...').should('not.exist'), got {}",
        painted.text
    );
    assert!(
        painted.bitcoin_visible,
        "input[formControlName=\"bitcoin\"] is visible"
    );
    assert_eq!(
        painted.text, shared.text,
        "painted text equals the shared form text"
    );
    assert_eq!(
        painted.bitcoin_visible, shared.bitcoin_visible,
        "painted bitcoin visibility equals the shared flag"
    );
    assert!(
        painted.form_row.contains("formControlName=\"bitcoin\""),
        "the laid-out form includes the bitcoin input: {}",
        painted.form_row
    );
    assert!(
        !painted.form_row.contains("Waiting for price feed..."),
        "the laid-out form does not contain the waiting sentence: {}",
        painted.form_row
    );
    let rows = painted.form_row.lines().count();
    assert_eq!(
        painted.row_heights.len(),
        rows,
        "each laid-out line needs a row height, row {:?} heights {:?}",
        painted.form_row,
        painted.row_heights
    );
    assert!(
        !painted.row_heights.is_empty() && painted.row_heights.iter().all(|height| *height > 0.0),
        "row heights come from laid-out bounds and must be positive, heights {:?}",
        painted.row_heights
    );
}
