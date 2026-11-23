//! Row 134. Show an unblinded Liquid transaction from the local `#blinded` fragment.
//!
//! Cypress: `ref/mempool/frontend/cypress/e2e/liquid/liquid.spec.ts`
//! `show unblinded TX`.
//! The fragment lists value, asset, value blinder, and asset blinder, twice.
//! 100000 sats is `0.00100000 LBTC`. 2364760 sats is `0.02364760 LBTC`.
//! The same spec also requires the first input amount `0.02465000 LBTC`
//! and the `assetBox` class on the input and output rows.
//! The native function under test is not implemented.

const BLINDED: &str = "100000,6f0279e9ed041c3d710a9f57d0c02928416460c4b722ae3457a11eec381c526d,0ab9f70650f16b1db8dfada05237f7d0d65191c3a13183da8a2ddddfbde9a2ad,fd98b2edc5530d76acd553f206a431f4c1fab27e10e290ad719582af878e98fc,2364760,6f0279e9ed041c3d710a9f57d0c02928416460c4b722ae3457a11eec381c526d,90c7a43b15b905bca045ca42a01271cfe71d2efe3133f4197792c24505cb32ed,12eb5959d9293b8842e7dd8bc9aa9639fd3fd031c5de3ba911adeca94eb57a3a";

#[test]
fn show_unblinded_tx() {
    let text = splora_native::show_unblinded_tx_text(BLINDED, 240);
    let vin_at = text.find("table-tx-vin").expect("vin table missing");
    let vout_at = text.find("table-tx-vout").expect("vout table missing");
    assert!(vin_at < vout_at, "{text}");
    let vin = &text[vin_at..vout_at];
    let vout = &text[vout_at..];
    assert!(
        vin.contains("0.02465000 LBTC"),
        "first input amount missing: {text}"
    );
    assert!(
        vin.contains("assetBox"),
        "input row is not an asset box: {text}"
    );
    let first = vout
        .find("0.00100000 LBTC")
        .expect("first output amount missing");
    let second = vout
        .find("0.02364760 LBTC")
        .expect("second output amount missing");
    assert!(first < second, "{text}");
    assert!(
        vout.contains("assetBox"),
        "output row is not an asset box: {text}"
    );

    let empty = splora_native::show_unblinded_tx_text("", 0);
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

/// An empty fragment is not an asset box, and a fee must not move the last
/// Confidential line onto the vin. A real blinded fragment still paints amounts.
#[test]
fn empty_fragment_is_not_an_asset_box() {
    let zero = splora_native::show_unblinded_tx_text("", 0);
    let with_fee = splora_native::show_unblinded_tx_text("", 240);
    assert!(
        !zero.contains("assetBox"),
        "show_unblinded_tx_text(\"\", 0) must not paint assetBox for an empty fragment, got {zero}"
    );
    assert!(
        !with_fee.contains("assetBox"),
        "show_unblinded_tx_text(\"\", 240) must not paint assetBox for an empty fragment, got {with_fee}"
    );
    assert_eq!(
        vin_amounts(&zero),
        vin_amounts(&with_fee),
        "an empty fragment must not paint the last Confidential line as the vin amount, fee 0 {zero:?} fee 240 {with_fee:?}"
    );

    let painted = splora_native::show_unblinded_tx_text(BLINDED, 240);
    assert!(
        painted.contains("0.02465000 LBTC"),
        "non-empty fragment must still paint the input amount: {painted}"
    );
    assert!(
        painted.contains("0.00100000 LBTC"),
        "non-empty fragment must still paint the first output amount: {painted}"
    );
    assert!(
        painted.contains("0.02364760 LBTC"),
        "non-empty fragment must still paint the second output amount: {painted}"
    );
    assert!(
        painted.contains("assetBox"),
        "non-empty fragment must still paint assetBox: {painted}"
    );
}

fn vin_amounts(text: &str) -> Vec<&str> {
    let vin = match (text.find("table-tx-vin"), text.find("table-tx-vout")) {
        (Some(vin_at), Some(vout_at)) if vin_at < vout_at => &text[vin_at..vout_at],
        (Some(vin_at), None) => &text[vin_at..],
        _ => "",
    };
    let mut amounts = Vec::new();
    let mut rest = vin;
    let key = "class=\"amount\">";
    while let Some(at) = rest.find(key) {
        rest = &rest[at + key.len()..];
        let end = rest.find('<').unwrap_or(rest.len());
        amounts.push(&rest[..end]);
        rest = &rest[end..];
    }
    amounts
}
