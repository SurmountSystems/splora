//! Cypress it() title: show second unblinded vout.
//!
//! The visit carries this fragment and does not fetch a transaction:
//! `#blinded=2364760,6f0279e9ed041c3d710a9f57d0c02928416460c4b722ae3457a11eec381c526d,90c7a43b15b905bca045ca42a01271cfe71d2efe3133f4197792c24505cb32ed,12eb5959d9293b8842e7dd8bc9aa9639fd3fd031c5de3ba911adeca94eb57a3a`
//!
//! Spec line 177 writes `tr:nth-child(2` and omits the closing parenthesis.
//! These asserts check child 2, not that broken selector text. The amount
//! is `sats_to_btc` plus ` LBTC`. No display string is the only check.

/// Cypress `#blinded` value for the second vout. Fee is not a field.
const SECOND: &str = "2364760,6f0279e9ed041c3d710a9f57d0c02928416460c4b722ae3457a11eec381c526d,90c7a43b15b905bca045ca42a01271cfe71d2efe3133f4197792c24505cb32ed,12eb5959d9293b8842e7dd8bc9aa9639fd3fd031c5de3ba911adeca94eb57a3a";

/// Same `#blinded` value as `show_first_unblinded_vout`. Not invented here.
const FIRST: &str = "100000,6f0279e9ed041c3d710a9f57d0c02928416460c4b722ae3457a11eec381c526d,0ab9f70650f16b1db8dfada05237f7d0d65191c3a13183da8a2ddddfbde9a2ad,fd98b2edc5530d76acd553f206a431f4c1fab27e10e290ad719582af878e98fc";

fn amount_lbtc(sats: u64) -> String {
    format!("{} LBTC", splora_frontend_shared::sats_to_btc(sats))
}

#[test]
fn show_second_unblinded_vout() {
    let second: splora_frontend_shared::SecondUnblindedVout =
        splora_frontend_shared::show_second_unblinded_vout(SECOND);
    let second_amount = amount_lbtc(2364760);
    assert_eq!(second.child, 2, "tr:nth-child(2)");
    assert_eq!(
        second.row_class, "assetBox",
        "cy.get('.table-tx-vout tr:nth-child(2)').should('have.class', 'assetBox')"
    );
    assert_eq!(second.amount, second_amount);
    assert!(
        second.amount.contains("0.02364760 LBTC"),
        "cy.get('.table-tx-vout tr:nth-child(2) .amount').should('contain.text', '0.02364760 LBTC'), got {}",
        second.amount
    );

    let first_on_child_2: splora_frontend_shared::SecondUnblindedVout =
        splora_frontend_shared::show_second_unblinded_vout(FIRST);
    let first_amount = amount_lbtc(100000);
    assert_eq!(first_on_child_2.child, 2, "tr:nth-child(2)");
    assert_eq!(first_on_child_2.amount, first_amount);
    assert!(
        first_on_child_2.amount.contains("0.00100000 LBTC"),
        "child 2 amount should contain 0.00100000 LBTC, got {}",
        first_on_child_2.amount
    );
    assert!(
        !first_on_child_2.amount.contains("0.02364760"),
        "the 100000-satoshi line must not contain 0.02364760, got {}",
        first_on_child_2.amount
    );

    let empty: splora_frontend_shared::SecondUnblindedVout =
        splora_frontend_shared::show_second_unblinded_vout("");
    assert_eq!(empty.child, 2, "tr:nth-child(2)");
    assert_ne!(
        empty.row_class, "assetBox",
        "an empty fragment must not paint assetBox on tr:nth-child(2)"
    );
}
