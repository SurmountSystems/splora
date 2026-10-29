//! Amount lines for a Liquid `#blinded` fragment.
//!
//! Field order matches `parseBlinders`: satoshi value, asset tag, value
//! blinder, asset blinder, repeated. The fee is not one of those fields.
//! When `fee_sats` is greater than zero, one more line is the parsed values
//! plus that fee. That line is the unknown input filled in after known
//! outputs and the fee.

use crate::calculator::sats_to_btc;

/// Format each parsed satoshi value as eight-decimal LBTC.
///
/// A positive `fee_sats` adds one line for the sum of those values plus the
/// fee. An empty or unusable fragment returns no amount lines.
pub fn unblinded_tx_preview(fragment: &str, fee_sats: u64) -> String {
    let values = parse_blinder_values(fragment);
    if values.is_empty() {
        return String::new();
    }
    let mut lines = Vec::with_capacity(values.len() + 1);
    let mut sum: u64 = 0;
    let mut sum_ok = true;
    for value in &values {
        lines.push(format_lbtc(*value));
        match sum.checked_add(*value) {
            Some(next) => sum = next,
            None => sum_ok = false,
        }
    }
    if fee_sats > 0
        && sum_ok
        && let Some(total) = sum.checked_add(fee_sats)
    {
        lines.push(format_lbtc(total));
    }
    lines.join("\n")
}

fn format_lbtc(sats: u64) -> String {
    format!("{} LBTC", sats_to_btc(sats))
}

/// Decimal value fields only. A group that is not a positive integer plus
/// three 64-character hex tokens ends the list.
fn parse_blinder_values(fragment: &str) -> Vec<u64> {
    if fragment.is_empty() {
        return Vec::new();
    }
    let parts: Vec<&str> = fragment.split(',').collect();
    let mut values = Vec::new();
    let mut index = 0;
    while index + 3 < parts.len() {
        let Some(value) = parse_positive_sats(parts[index]) else {
            break;
        };
        if !is_hex32(parts[index + 1]) || !is_hex32(parts[index + 2]) || !is_hex32(parts[index + 3])
        {
            break;
        }
        values.push(value);
        index += 4;
    }
    values
}

fn parse_positive_sats(token: &str) -> Option<u64> {
    if token.is_empty() || !token.bytes().all(|byte| byte.is_ascii_digit()) {
        return None;
    }
    let mut n: u64 = 0;
    for byte in token.bytes() {
        let digit = (byte - b'0') as u64;
        n = n.checked_mul(10)?.checked_add(digit)?;
    }
    if n == 0 { None } else { Some(n) }
}

fn is_hex32(token: &str) -> bool {
    token.len() == 64 && token.bytes().all(|byte| byte.is_ascii_hexdigit())
}
