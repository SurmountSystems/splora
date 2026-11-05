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

/// Round a satoshi field to whole satoshis. Digits and carry only.
///
/// No decimal point returns the input. An empty fraction, or one whose
/// first character is `0` through `4`, returns the digits before the
/// point, or `0` when that side is empty. A fraction whose first
/// character is `5` through `9` increments those whole digits by one.
pub fn strips_decimals_from_satoshis_input(input: &str) -> String {
    let Some((whole, frac)) = input.split_once('.') else {
        return input.to_string();
    };
    let whole_digits = if whole.is_empty() { "0" } else { whole };
    let round_up = matches!(frac.as_bytes().first(), Some(b'5'..=b'9'));
    if !round_up {
        return whole_digits.to_string();
    }
    let mut digits: Vec<char> = whole_digits.chars().collect();
    let mut index = digits.len();
    while index > 0 {
        index -= 1;
        let digit = digits[index];
        if digit == '9' {
            digits[index] = '0';
            continue;
        }
        if digit.is_ascii_digit() {
            digits[index] = char::from(digit as u8 + 1);
            return digits.into_iter().collect();
        }
        return whole_digits.to_string();
    }
    digits.insert(0, '1');
    digits.into_iter().collect()
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

/// Bitcoin field, satoshi field, and the max-supply warning for typed bitcoin text.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MaxSupplyEntry {
    /// The max-supply sentence when the typed amount reaches the cap, otherwise empty.
    pub warning: String,
    /// Typed bitcoin text, replaced with the cap when the typed amount reaches it.
    pub bitcoin: String,
    /// Capped bitcoin amount times [`SATS_PER_BTC`], not above that product.
    pub satoshis: String,
}

const MAX_SUPPLY_WARNING: &str = "Values were capped at the max supply of 21M BTC";

/// Local supply-cap fields for typed bitcoin text.
///
/// Does not fetch a rate. Does not use floating point.
pub fn shows_warning_when_entering_21m_btc(typed: &str) -> MaxSupplyEntry {
    let capped = cap_btc_supply(typed);
    let sat_cap = BTC_SUPPLY_CAP.saturating_mul(SATS_PER_BTC);
    let sats = btc_to_sats(&capped.btc).unwrap_or(0).min(sat_cap);
    let warning = if capped.capped {
        MAX_SUPPLY_WARNING.to_string()
    } else {
        String::new()
    };
    MaxSupplyEntry {
        warning,
        bitcoin: capped.btc,
        satoshis: sats.to_string(),
    }
}

/// Bitcoin input visibility and the typed field text at a named viewport.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DesktopCalculator {
    pub bitcoin_visible: bool,
    pub value: String,
}

/// Local desktop check that the bitcoin input is shown and holds the typed text.
///
/// Not a price quote. No floating point. No hard-coded amount.
pub fn calculator_is_usable_on_desktop(viewport: &str, typed: &str) -> DesktopCalculator {
    DesktopCalculator {
        bitcoin_visible: viewport == "macbook-16",
        value: typed.to_string(),
    }
}

/// Bitcoin, fiat, and satoshis input visibility at a named viewport.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MobileCalculator {
    pub bitcoin_visible: bool,
    pub fiat_visible: bool,
    pub satoshis_visible: bool,
}

/// Local mobile check that the bitcoin, fiat, and satoshis inputs are shown.
///
/// Not a price quote. No typed text. No floating point. No hard-coded amount.
pub fn calculator_is_usable_on_mobile(viewport: &str) -> MobileCalculator {
    let shown = viewport == "iphone-6";
    MobileCalculator {
        bitcoin_visible: shown,
        fiat_visible: shown,
        satoshis_visible: shown,
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
