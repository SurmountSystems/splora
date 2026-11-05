//! Web paint of the peg-out Bitcoin address URL.
//!
//! The target is the shared function's return value. This file does not
//! hard-code that target.

/// Paints the peg-out Bitcoin address URL. The string is the shared function.
pub fn paint_loads_peg_out_addresses_bitcoin_address_url(address: &str) -> String {
    splora_frontend_shared::loads_peg_out_addresses_bitcoin_address_url(address)
}
