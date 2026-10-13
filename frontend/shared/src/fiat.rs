//! USD and JPY amount text. Amounts are decimal strings, not floating point.
//! Fixture conversion uses integer ratios at fixed rates. It does not use the network.

/// Currency code for [`format_fiat`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FiatCode {
    Usd,
    Jpy,
}

/// Replace each comma in a decimal field with `.`.
pub fn decimal_comma_to_dot(input: &str) -> String {
    input.replace(',', ".")
}

/// Format `amount` with `$` or `¥` (U+00A5).
///
/// A whole USD amount has no cents. A non-whole USD amount shows two cent
/// digits, and extra fractional digits are dropped. JPY has no decimal point.
pub fn format_fiat(code: FiatCode, amount: &str) -> String {
    let (whole, frac) = split_amount(amount);
    match code {
        FiatCode::Usd => {
            let cents = usd_cents(&frac);
            if cents == 0 {
                format!("${whole}")
            } else {
                format!("${whole}.{cents:02}")
            }
        }
        FiatCode::Jpy => format!("\u{00A5}{whole}"),
    }
}

/// Format one amount as USD, then JPY, then USD.
pub fn switch_usd_jpy_usd(amount: &str) -> [String; 3] {
    [
        format_fiat(FiatCode::Usd, amount),
        format_fiat(FiatCode::Jpy, amount),
        format_fiat(FiatCode::Usd, amount),
    ]
}

/// Fixture USD per one BTC. Not a live price.
pub const FIXTURE_USD_PER_BTC: u64 = 123_456;

/// Fixture JPY per one BTC. Not a live price.
pub const FIXTURE_JPY_PER_BTC: u64 = 11_057_757;

/// Exact BTC bought by a USD amount at [`FIXTURE_USD_PER_BTC`].
///
/// `numerator / denominator` is that BTC amount. For 100 USD it is
/// 100/123456, not a pre-rounded decimal. `sats` is that ratio times
/// 100000000, rounded half up.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UsdBtcQuote {
    pub numerator: u128,
    pub denominator: u128,
    pub sats: u64,
}

/// Exact BTC bought by a JPY amount at [`FIXTURE_JPY_PER_BTC`].
///
/// `sats` uses the same half-up rule as [`UsdBtcQuote`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct JpyBtcQuote {
    pub numerator: u128,
    pub denominator: u128,
    pub sats: u64,
}

/// Format a BTC decimal as USD at [`FIXTURE_USD_PER_BTC`].
///
/// USD always shows two fractional digits and thousands separators.
pub fn btc_fiat_usd(btc: &str) -> String {
    let sats = crate::calculator::btc_to_sats(btc).unwrap_or(0);
    sats_fiat_usd(sats)
}

/// Format a BTC decimal as JPY at [`FIXTURE_JPY_PER_BTC`].
///
/// Yen has no fractional digits. Thousands separators are commas.
pub fn btc_fiat_jpy(btc: &str) -> String {
    let sats = crate::calculator::btc_to_sats(btc).unwrap_or(0);
    format_jpy_yen(sats_to_yen(sats))
}

/// Format integer satoshis as USD at [`FIXTURE_USD_PER_BTC`].
pub fn sats_fiat_usd(sats: u64) -> String {
    format_usd_cents(sats_to_usd_cents(sats))
}

/// Format integer satoshis as JPY at [`FIXTURE_JPY_PER_BTC`].
pub fn sats_fiat_jpy(sats: u64) -> String {
    format_jpy_yen(sats_to_yen(sats))
}

/// Convert a USD decimal to BTC and satoshis at [`FIXTURE_USD_PER_BTC`].
///
/// The BTC amount stays the exact ratio of the entered units over the rate.
pub fn usd_to_btc_and_sats(amount: &str) -> Option<UsdBtcQuote> {
    let (numerator, frac_len) = parse_decimal_units(amount)?;
    let scale = 10_u128.checked_pow(frac_len)?;
    let denominator = u128::from(FIXTURE_USD_PER_BTC).checked_mul(scale)?;
    let sats_numer = numerator.checked_mul(u128::from(crate::calculator::SATS_PER_BTC))?;
    let sats = u64::try_from(div_round_half_up(sats_numer, denominator)).ok()?;
    Some(UsdBtcQuote {
        numerator,
        denominator,
        sats,
    })
}

/// Convert a JPY decimal to BTC and satoshis at [`FIXTURE_JPY_PER_BTC`].
pub fn jpy_to_btc_and_sats(amount: &str) -> Option<JpyBtcQuote> {
    let (numerator, frac_len) = parse_decimal_units(amount)?;
    let scale = 10_u128.checked_pow(frac_len)?;
    let denominator = u128::from(FIXTURE_JPY_PER_BTC).checked_mul(scale)?;
    let sats_numer = numerator.checked_mul(u128::from(crate::calculator::SATS_PER_BTC))?;
    let sats = u64::try_from(div_round_half_up(sats_numer, denominator)).ok()?;
    Some(JpyBtcQuote {
        numerator,
        denominator,
        sats,
    })
}

fn sats_to_usd_cents(sats: u64) -> u128 {
    // cents = round(sats * usd_per_btc / 1_000_000)
    let numer = u128::from(sats) * u128::from(FIXTURE_USD_PER_BTC);
    div_round_half_up(numer, 1_000_000)
}

fn sats_to_yen(sats: u64) -> u128 {
    let numer = u128::from(sats) * u128::from(FIXTURE_JPY_PER_BTC);
    div_round_half_up(numer, u128::from(crate::calculator::SATS_PER_BTC))
}

fn format_usd_cents(cents: u128) -> String {
    let dollars = cents / 100;
    let frac = (cents % 100) as u32;
    format!("${}.{:02}", group_thousands(&dollars.to_string()), frac)
}

fn format_jpy_yen(yen: u128) -> String {
    format!("\u{00A5}{}", group_thousands(&yen.to_string()))
}

fn group_thousands(digits: &str) -> String {
    let mut grouped = String::new();
    let len = digits.len();
    for (i, c) in digits.chars().enumerate() {
        if i > 0 && (len - i).is_multiple_of(3) {
            grouped.push(',');
        }
        grouped.push(c);
    }
    grouped
}

/// Value is `numerator / 10^frac_len`.
fn parse_decimal_units(amount: &str) -> Option<(u128, u32)> {
    let amount = amount.trim();
    if amount.is_empty() || amount.starts_with('+') || amount.starts_with('-') {
        return None;
    }
    let (whole, frac) = match amount.split_once('.') {
        Some((whole, frac)) => (whole, frac),
        None => (amount, ""),
    };
    if whole.is_empty() && frac.is_empty() {
        return None;
    }
    if !whole.chars().all(|c| c.is_ascii_digit()) || !frac.chars().all(|c| c.is_ascii_digit()) {
        return None;
    }
    if frac.len() > 18 {
        return None;
    }
    let mut numer = if whole.is_empty() {
        0
    } else {
        parse_u128(whole)?
    };
    for c in frac.chars() {
        let digit = c.to_digit(10)? as u128;
        numer = numer.checked_mul(10)?.checked_add(digit)?;
    }
    Some((numer, frac.len() as u32))
}

fn parse_u128(digits: &str) -> Option<u128> {
    if digits.is_empty() {
        return None;
    }
    let mut n: u128 = 0;
    for c in digits.chars() {
        let digit = c.to_digit(10)? as u128;
        n = n.checked_mul(10)?.checked_add(digit)?;
    }
    Some(n)
}

fn div_round_half_up(numer: u128, denom: u128) -> u128 {
    let quotient = numer / denom;
    let remainder = numer % denom;
    if remainder.saturating_mul(2) >= denom {
        quotient.saturating_add(1)
    } else {
        quotient
    }
}

fn split_amount(amount: &str) -> (String, String) {
    let amount = amount.trim();
    let (whole, frac) = match amount.split_once('.') {
        Some((whole, frac)) => (whole, frac),
        None => (amount, ""),
    };
    let whole_digits: String = whole.chars().filter(|c| c.is_ascii_digit()).collect();
    let frac_digits: String = frac.chars().filter(|c| c.is_ascii_digit()).collect();
    (normalize_whole(&whole_digits), frac_digits)
}

fn normalize_whole(digits: &str) -> String {
    let trimmed = digits.trim_start_matches('0');
    if trimmed.is_empty() {
        "0".to_string()
    } else {
        trimmed.to_string()
    }
}

fn usd_cents(frac: &str) -> u32 {
    let mut digits: String = frac.chars().take(2).collect();
    if digits.is_empty() {
        return 0;
    }
    if digits.len() == 1 {
        digits.push('0');
    }
    let mut n: u32 = 0;
    for c in digits.chars() {
        n = n * 10 + c.to_digit(10).unwrap_or(0);
    }
    n
}
