pub fn regular_tx_unblinding_text(tx_hex: &str) -> String {
    match splora_frontend_shared::unblinding_error_for_regular_tx(tx_hex) {
        None => String::new(),
        Some(error) => error.to_string(),
    }
}
