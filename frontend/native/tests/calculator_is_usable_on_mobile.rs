#[test]
fn calculator_is_usable_on_mobile() {
    // paint_calculator_is_usable_on_mobile is not defined yet.
    let painted = splora_native::paint_calculator_is_usable_on_mobile("iphone-6");
    let shared = splora_frontend_shared::calculator_is_usable_on_mobile("iphone-6");
    assert!(painted.bitcoin_visible);
    assert!(painted.fiat_visible);
    assert!(painted.satoshis_visible);
    assert_eq!(painted, shared);
}
