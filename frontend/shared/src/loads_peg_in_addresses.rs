//! Static English label on a Liquid peg-in input.
//!
//! Cypress `loads peg in addresses` asserts `.table-tx-vin` contains `Peg-in`.
//! The peg-in link URL is not this string.

/// Static label `Peg-in` from a transaction input row.
pub fn loads_peg_in_addresses() -> &'static str {
    "Peg-in"
}
