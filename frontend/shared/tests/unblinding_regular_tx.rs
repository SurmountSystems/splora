// Row 133. A regular (non-confidential) transaction must not produce an
// unblinding error message. Unblinding is local. The function under test
// is not implemented in this crate.

#[test]
fn regular_tx_does_not_show_an_unblinding_error() {
    // Bitcoin genesis coinbase. It is a regular transaction, not a
    // confidential Liquid transaction.
    let regular_tx_hex = "01000000010000000000000000000000000000000000000000000000000000000000000000ffffffff4d04ffff001d0104455468652054696d65732030332f4a616e2f32303039204368616e63656c6c6f72206f6e206272696e6b206f66207365636f6e64206261696c6f757420666f722062616e6b73ffffffff0100f2052a01000000434104678afdb0fe5548271967f1a67130b7105cd6a828e03909a67962e0ea1f61deb649f6bc3f4cef38c4f35504e51ec112de5c384df7ba0b8d578a4c702b6bf11d5fac00000000";
    let error_message = splora_frontend_shared::unblinding_error_for_regular_tx(regular_tx_hex);
    assert!(
        error_message.is_none(),
        "regular tx must not show an unblinding error message, got {error_message:?}"
    );
}
