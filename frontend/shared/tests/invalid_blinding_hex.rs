//! Cypress `show invalid unblinded TX hex`.
//! `ref/mempool/frontend/cypress/e2e/liquidtestnet/liquidtestnet.spec.ts` line 124.
//! `parseBlinders` splits on commas. The shared function is not implemented.

const INVALID_HEX: &str = "Error: Invalid blinding data (invalid hex)";

/// `show unblinded TX` fragment, same file line 104. Each hex token is 64 characters.
const PASSING: &str = "100000,144c654344aa716d6f3abcc1ca90e5641e4e2a7f633bc09fe3baf64585819a49,df290ead654d7d110ebc5aaf0bcf11d5b5d360431a467f1cde0a856fde986893,33cb3a2fd2e76643843691cf44a78c5cd28ec652a414da752160ad63fbd37bc9,49741,144c654344aa716d6f3abcc1ca90e5641e4e2a7f633bc09fe3baf64585819a49,edb0713bcbfcb3daabf601cb50978439667d208e15fed8a5ebbfea5696cda1d5,4de70115501e8c7d6bd763e229bf42781edeacf6e75e1d7bdfa4c63104bc508a";

#[test]
fn show_invalid_unblinded_tx_hex() {
    assert_eq!(
        splora_frontend_shared::invalid_blinding_data_hex_error("5000"),
        INVALID_HEX
    );
    assert_eq!(
        splora_frontend_shared::invalid_blinding_data_hex_error("abcd"),
        INVALID_HEX
    );
    let passed = splora_frontend_shared::invalid_blinding_data_hex_error(PASSING);
    assert!(
        !passed.contains(INVALID_HEX),
        "valid hex fragment must not report invalid hex: {passed}"
    );
}
