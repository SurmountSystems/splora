//! Web paint of the peg-in Bitcoin transaction URL.
//!
//! The target is the shared function's return value. This file does not
//! hard-code that target.

/// Paints the peg-in Bitcoin transaction URL. The string is the shared function.
pub fn paint_loads_peg_in_addresses_bitcoin_tx_url(txid: &str, vout: u32) -> String {
    splora_frontend_shared::loads_peg_in_addresses_bitcoin_tx_url(txid, vout)
}
