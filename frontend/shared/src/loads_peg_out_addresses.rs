//! Static English badge on a Liquid peg-out address.
//!
//! Cypress `loads peg out addresses` asserts `.badge` contains `Liquid Peg Out`.
//! The peg-out destination URL is not this string.

/// Static badge `Liquid Peg Out` from a Liquid peg-out address.
pub fn loads_peg_out_addresses() -> &'static str {
    "Liquid Peg Out"
}
