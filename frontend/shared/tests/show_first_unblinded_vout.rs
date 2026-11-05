//! Cypress it() title: show first unblinded vout.
//!
//! The spec visits one transaction whose `#blinded` fragment is a single
//! group. The skeleton wait is not an assertion. The first `.table-tx-vout`
//! row has class `assetBox`, and that row's `.amount` contains
//! `0.00100000 LBTC`. This file does not read a block hash, a price, or
//! the heading "1 transaction".

/// One complete `#blinded` group from the Cypress visit. Fee is not a field.
const BLINDED: &str = "100000,6f0279e9ed041c3d710a9f57d0c02928416460c4b722ae3457a11eec381c526d,0ab9f70650f16b1db8dfada05237f7d0d65191c3a13183da8a2ddddfbde9a2ad,fd98b2edc5530d76acd553f206a431f4c1fab27e10e290ad719582af878e98fc";

#[test]
fn show_first_unblinded_vout() {
    let row = splora_frontend_shared::show_first_unblinded_vout(BLINDED);
    assert_eq!(
        row.row_class, "assetBox",
        "cy.get('.table-tx-vout tr:nth-child(1)').should('have.class', 'assetBox')"
    );
    assert!(
        row.amount.contains("0.00100000 LBTC"),
        "cy.get('.table-tx-vout tr:nth-child(1) .amount').should('contain.text', '0.00100000 LBTC'), got {}",
        row.amount
    );
}
