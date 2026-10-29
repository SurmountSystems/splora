//! Row 134. Show an unblinded Liquid transaction from the local `#blinded` fragment.
//!
//! Cypress: `ref/mempool/frontend/cypress/e2e/liquid/liquid.spec.ts`
//! `show unblinded TX`.
//! The fragment lists value, asset, value blinder, and asset blinder, twice.
//! 100000 sats is `0.00100000 LBTC`. 2364760 sats is `0.02364760 LBTC`.
//! The same spec also requires the first input amount `0.02465000 LBTC`
//! and the `assetBox` class on the input and output rows.
//! The web function under test is not implemented.

const BLINDED: &str = "100000,6f0279e9ed041c3d710a9f57d0c02928416460c4b722ae3457a11eec381c526d,0ab9f70650f16b1db8dfada05237f7d0d65191c3a13183da8a2ddddfbde9a2ad,fd98b2edc5530d76acd553f206a431f4c1fab27e10e290ad719582af878e98fc,2364760,6f0279e9ed041c3d710a9f57d0c02928416460c4b722ae3457a11eec381c526d,90c7a43b15b905bca045ca42a01271cfe71d2efe3133f4197792c24505cb32ed,12eb5959d9293b8842e7dd8bc9aa9639fd3fd031c5de3ba911adeca94eb57a3a";

#[test]
fn show_unblinded_tx() {
    let markup = splora_web::show_unblinded_tx_markup(BLINDED, 240);
    let vin_at = markup.find("table-tx-vin").expect("vin table missing");
    let vout_at = markup.find("table-tx-vout").expect("vout table missing");
    assert!(vin_at < vout_at, "{markup}");
    let vin = &markup[vin_at..vout_at];
    let vout = &markup[vout_at..];
    assert!(
        vin.contains("0.02465000 LBTC"),
        "first input amount missing: {markup}"
    );
    assert!(
        vin.contains("assetBox"),
        "input row is not an asset box: {markup}"
    );
    let first = vout
        .find("0.00100000 LBTC")
        .expect("first output amount missing");
    let second = vout
        .find("0.02364760 LBTC")
        .expect("second output amount missing");
    assert!(first < second, "{markup}");
    assert!(
        vout.contains("assetBox"),
        "output row is not an asset box: {markup}"
    );

    let empty = splora_web::show_unblinded_tx_markup("", 0);
    assert!(
        !empty.contains("0.02465000 LBTC"),
        "empty fragment painted the input amount: {empty}"
    );
    assert!(
        !empty.contains("0.00100000 LBTC"),
        "empty fragment painted the first output amount: {empty}"
    );
    assert!(
        !empty.contains("0.02364760 LBTC"),
        "empty fragment painted the second output amount: {empty}"
    );
}
