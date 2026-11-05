//! Cypress title: show second unblinded vout.
//!
//! Spec: ref/mempool/frontend/cypress/e2e/liquid/liquid.spec.ts lines 175-179.
//! The visit fragment's first field is 2364760 satoshis. Child 2 inside
//! `.table-tx-vout` has class `assetBox`, and that row's amount is
//! `sats_to_btc(2364760)` plus ` LBTC`. This test calls
//! `paint_show_second_unblinded_vout`. That paint opens one headless window
//! with `show: false` and quits before it returns. Child, class, and amount
//! come from `splora_frontend_shared::show_second_unblinded_vout`. This test
//! does not fetch a transaction.

/// Cypress `#blinded` value. The first field is 2364760 satoshis.
const SECOND: &str = "2364760,6f0279e9ed041c3d710a9f57d0c02928416460c4b722ae3457a11eec381c526d,90c7a43b15b905bca045ca42a01271cfe71d2efe3133f4197792c24505cb32ed,12eb5959d9293b8842e7dd8bc9aa9639fd3fd031c5de3ba911adeca94eb57a3a";

/// Copied from `frontend/shared/tests/show_first_unblinded_vout.rs`.
/// 100000 satoshis.
const FIRST: &str = "100000,6f0279e9ed041c3d710a9f57d0c02928416460c4b722ae3457a11eec381c526d,0ab9f70650f16b1db8dfada05237f7d0d65191c3a13183da8a2ddddfbde9a2ad,fd98b2edc5530d76acd553f206a431f4c1fab27e10e290ad719582af878e98fc";

fn amount_lbtc(sats: u64) -> String {
    format!("{} LBTC", splora_frontend_shared::sats_to_btc(sats))
}

#[test]
fn show_second_unblinded_vout() {
    let painted = splora_native::paint_show_second_unblinded_vout(SECOND);
    assert_eq!(painted.child, 2, "tr:nth-child(2)");
    assert_eq!(
        painted.row_class, "assetBox",
        "cy.get('.table-tx-vout tr:nth-child(2)').should('have.class', 'assetBox')"
    );
    let expected = amount_lbtc(2364760);
    assert_eq!(
        painted.amount, expected,
        "child 2 amount is sats_to_btc(2364760) plus \" LBTC\""
    );
    assert!(
        painted.amount.contains("0.02364760 LBTC"),
        "cy.get('.table-tx-vout tr:nth-child(2) .amount').should('contain.text', '0.02364760 LBTC'), got {}",
        painted.amount
    );

    let first = splora_native::paint_show_second_unblinded_vout(FIRST);
    assert_eq!(first.child, 2, "tr:nth-child(2)");
    assert!(
        first.amount.contains("0.00100000 LBTC"),
        "child 2 amount should contain 0.00100000 LBTC, got {}",
        first.amount
    );
    assert!(
        !first.amount.contains("0.02364760"),
        "the 100000-satoshi line must not contain 0.02364760, got {}",
        first.amount
    );

    let empty = splora_native::paint_show_second_unblinded_vout("");
    assert_eq!(empty.child, 2, "tr:nth-child(2)");
    assert_ne!(
        empty.row_class, "assetBox",
        "an empty fragment must not put class assetBox on child 2"
    );
}
