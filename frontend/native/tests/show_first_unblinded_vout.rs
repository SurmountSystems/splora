//! Cypress title: show first unblinded vout.
//!
//! Spec: ref/mempool/frontend/cypress/e2e/liquid/liquid.spec.ts lines 168-173.
//! The visit is one transaction with a single `#blinded` group. The first
//! `.table-tx-vout` row has class `assetBox`, and that row's `.amount`
//! contains `0.00100000 LBTC`. This test calls
//! `paint_show_first_unblinded_vout`. That paint opens one headless window
//! with `show: false` and quits before it returns. Class and amount come
//! from `splora_frontend_shared::show_first_unblinded_vout`. That return has
//! no height, no route, and no icon visibility, so this test does not invent
//! them. This test does not call a keypress paint, and it does not invent a
//! block hash, a fee, a price, or the heading `1 transaction`.

/// One complete `#blinded` group from the Cypress visit. Fee is not a field.
const BLINDED: &str = "100000,6f0279e9ed041c3d710a9f57d0c02928416460c4b722ae3457a11eec381c526d,0ab9f70650f16b1db8dfada05237f7d0d65191c3a13183da8a2ddddfbde9a2ad,fd98b2edc5530d76acd553f206a431f4c1fab27e10e290ad719582af878e98fc";

#[test]
fn show_first_unblinded_vout() {
    let painted = splora_native::paint_show_first_unblinded_vout(BLINDED);
    assert_eq!(
        painted.row_class, "assetBox",
        "cy.get('.table-tx-vout tr:nth-child(1)').should('have.class', 'assetBox')"
    );
    assert!(
        painted.amount.contains("0.00100000 LBTC"),
        "cy.get('.table-tx-vout tr:nth-child(1) .amount').should('contain.text', '0.00100000 LBTC'), got {}",
        painted.amount
    );
    assert!(
        painted.vout_row.contains("table-tx-vout"),
        "the first vout row is inside .table-tx-vout: {}",
        painted.vout_row
    );
    assert!(
        painted.vout_row.contains(painted.row_class.as_str()),
        "the laid-out row includes the shared class {}: {}",
        painted.row_class,
        painted.vout_row
    );
    assert!(
        painted.vout_row.contains(painted.amount.as_str()),
        "the laid-out row includes the shared amount {}: {}",
        painted.amount,
        painted.vout_row
    );
    let rows = painted.vout_row.lines().count();
    assert_eq!(
        painted.row_heights.len(),
        rows,
        "each laid-out line needs a row height, row {:?} heights {:?}",
        painted.vout_row,
        painted.row_heights
    );
    assert!(
        !painted.row_heights.is_empty() && painted.row_heights.iter().all(|height| *height > 0.0),
        "row heights come from laid-out bounds and must be positive, heights {:?}",
        painted.row_heights
    );
}
