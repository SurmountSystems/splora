//! Integer satoshi conversion. One BTC is 100000000 satoshis.
//! These functions do not use the network or floating point.

/// Scale used by [`btc_to_sats`] and [`sats_to_btc`].
pub const SATS_PER_BTC: u64 = 100_000_000;

/// Issuance cap in whole BTC.
pub const BTC_SUPPLY_CAP: u64 = 21_000_000;

/// A BTC amount after the issuance cap is applied.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CappedBtc {
    pub btc: String,
    pub capped: bool,
}

/// Convert a BTC decimal string to integer satoshis.
///
/// Extra digits past 8 decimal places are dropped, not rounded.
pub fn btc_to_sats(btc: &str) -> Option<u64> {
    let btc = btc.trim();
    if btc.is_empty() || btc.starts_with('+') || btc.starts_with('-') {
        return None;
    }
    let (whole, frac) = match btc.split_once('.') {
        Some((whole, frac)) => (whole, frac),
        None => (btc, ""),
    };
    if whole.is_empty() && frac.is_empty() {
        return None;
    }
    let whole_digits = if whole.is_empty() { "0" } else { whole };
    let whole_n = parse_digits(whole_digits)?;
    let frac_n = frac_as_sats(frac)?;
    let whole_sats = whole_n.checked_mul(SATS_PER_BTC)?;
    whole_sats.checked_add(frac_n)
}

/// Format integer satoshis as BTC with 8 decimal places.
pub fn sats_to_btc(sats: u64) -> String {
    let whole = sats / SATS_PER_BTC;
    let frac = sats % SATS_PER_BTC;
    format!("{whole}.{frac:08}")
}

/// Keep at most 8 digits after the decimal point.
///
/// Digits are cut off. A shorter value is not padded with zeros.
pub fn truncate_btc_decimals(input: &str) -> String {
    let Some((whole, frac)) = input.split_once('.') else {
        return input.to_string();
    };
    let truncated: String = frac.chars().take(8).collect();
    format!("{whole}.{truncated}")
}

/// Remove `.` and keep every other character, including digits on both sides.
pub fn strip_sats_decimal(input: &str) -> String {
    input.chars().filter(|c| *c != '.').collect()
}

/// Flag an amount that reaches 21000000 BTC, and clamp anything above it.
pub fn cap_btc_supply(btc: &str) -> CappedBtc {
    if reaches_cap(btc) {
        CappedBtc {
            btc: BTC_SUPPLY_CAP.to_string(),
            capped: true,
        }
    } else {
        CappedBtc {
            btc: btc.to_string(),
            capped: false,
        }
    }
}

fn reaches_cap(btc: &str) -> bool {
    let (whole, frac) = match btc.split_once('.') {
        Some((whole, frac)) => (whole, frac),
        None => (btc, ""),
    };
    if !whole.chars().all(|c| c.is_ascii_digit()) || !frac.chars().all(|c| c.is_ascii_digit()) {
        return false;
    }
    let whole = whole.trim_start_matches('0');
    if whole.is_empty() {
        return false;
    }
    if whole.len() > 8 {
        return true;
    }
    if whole.len() < 8 {
        return false;
    }
    whole.as_bytes() >= b"21000000"
}

fn parse_digits(digits: &str) -> Option<u64> {
    if digits.is_empty() {
        return None;
    }
    let mut n: u64 = 0;
    for c in digits.chars() {
        let d = c.to_digit(10)? as u64;
        n = n.checked_mul(10)?.checked_add(d)?;
    }
    Some(n)
}

fn frac_as_sats(frac: &str) -> Option<u64> {
    let mut digits = String::new();
    for c in frac.chars() {
        if !c.is_ascii_digit() {
            return None;
        }
        if digits.len() < 8 {
            digits.push(c);
        }
    }
    while digits.len() < 8 {
        digits.push('0');
    }
    parse_digits(&digits)
}

#[cfg(test)]
mod tests {
    #[test]
    fn native_calculator_fixture_helpers_return_existing_strings() {
        assert_eq!(crate::fiat::btc_fiat_usd("0.33"), "$40,740.48");
        assert_eq!(crate::fiat::btc_fiat_usd("0.00000001"), "$0.00");
        assert_eq!(crate::fiat::btc_fiat_jpy("1"), "\u{00A5}11,057,757");
        assert_eq!(crate::fiat::sats_fiat_usd(10_000), "$12.35");
    }
}
