//! Row 134. Show an unblinded Liquid transaction from the local blinded fragment.
//!
//! Cypress: `ref/mempool/frontend/cypress/e2e/liquid/liquid.spec.ts`
//! `show unblinded TX`.
//! The shared function under test is not implemented.

const BLINDED: &str = "100000,6f0279e9ed041c3d710a9f57d0c02928416460c4b722ae3457a11eec381c526d,0ab9f70650f16b1db8dfada05237f7d0d65191c3a13183da8a2ddddfbde9a2ad,fd98b2edc5530d76acd553f206a431f4c1fab27e10e290ad719582af878e98fc,2364760,6f0279e9ed041c3d710a9f57d0c02928416460c4b722ae3457a11eec381c526d,90c7a43b15b905bca045ca42a01271cfe71d2efe3133f4197792c24505cb32ed,12eb5959d9293b8842e7dd8bc9aa9639fd3fd031c5de3ba911adeca94eb57a3a";

const AMOUNTS: [&str; 3] = ["0.02465000 LBTC", "0.00100000 LBTC", "0.02364760 LBTC"];

#[test]
fn show_unblinded_tx() {
    let preview = splora_frontend_shared::unblinded_tx_preview(BLINDED, 240);
    for amount in AMOUNTS {
        assert!(
            preview.contains(amount),
            "blinded fragment preview missing {amount}: {preview}"
        );
    }

    let empty = splora_frontend_shared::unblinded_tx_preview("", 0);
    for amount in AMOUNTS {
        assert!(
            !empty.contains(amount),
            "empty fragment preview contains {amount}: {empty}"
        );
    }
}
