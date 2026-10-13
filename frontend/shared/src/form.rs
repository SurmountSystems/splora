//! Calculator heading and fiat, BTC, and sats inputs.
//! Amounts use the fixture rates in `fiat`. This module does not use the network.

use crate::calculator::{btc_to_sats, cap_btc_supply, sats_to_btc, truncate_btc_decimals};
use crate::fiat::{
    FiatCode, btc_fiat_jpy, btc_fiat_usd, decimal_comma_to_dot, jpy_to_btc_and_sats, sats_fiat_jpy,
    sats_fiat_usd, usd_to_btc_and_sats,
};

/// Shared calculator fields. Not a browser page and not a price request.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CalculatorForm {
    pub heading: &'static str,
    pub code: FiatCode,
    pub fiat: String,
    pub btc: String,
    pub sats: String,
    pub symbol: String,
    pub fiat_text: String,
    pub bitcoin_visual: &'static str,
    pub price_updated_label: &'static str,
    pub waiting_for_price: bool,
    pub copy_buttons: [&'static str; 3],
    pub label_btc: &'static str,
    pub label_sats: &'static str,
}

impl CalculatorForm {
    /// Fiat, BTC, and sats each have text.
    pub fn inputs_present(&self) -> bool {
        !self.fiat.is_empty() && !self.btc.is_empty() && !self.sats.is_empty()
    }
}

/// The three calculator amount field names.
pub fn calculator_field_names() -> [&'static str; 3] {
    ["fiat", "bitcoin", "satoshis"]
}

/// One BTC, with fiat filled from the fixture rate for `code`.
pub fn loaded_calculator(code: FiatCode) -> CalculatorForm {
    let symbol = match code {
        FiatCode::Usd => btc_fiat_usd("1"),
        FiatCode::Jpy => btc_fiat_jpy("1"),
    };
    let blank = CalculatorForm {
        heading: "Calculator",
        code,
        fiat: String::new(),
        btc: String::new(),
        sats: String::new(),
        symbol,
        fiat_text: String::new(),
        bitcoin_visual: "\u{20BF}",
        price_updated_label: "Fiat price last updated",
        waiting_for_price: false,
        copy_buttons: ["fiat", "btc", "sats"],
        label_btc: "BTC",
        label_sats: "sats",
    };
    enter_btc(&blank, "1")
}

/// Set the fiat field and update BTC and sats from the fixture rate.
pub fn enter_fiat(form: &CalculatorForm, amount: &str) -> Option<CalculatorForm> {
    let amount = decimal_comma_to_dot(amount);
    let sats = match form.code {
        FiatCode::Usd => usd_to_btc_and_sats(&amount)?.sats,
        FiatCode::Jpy => jpy_to_btc_and_sats(&amount)?.sats,
    };
    Some(with_amounts(form, amount, sats_to_btc(sats), sats))
}

/// Set the BTC field, cap it at 21000000, and update fiat and sats.
pub fn enter_btc(form: &CalculatorForm, amount: &str) -> CalculatorForm {
    let truncated = truncate_btc_decimals(amount);
    let capped = cap_btc_supply(&truncated);
    let btc = capped.btc;
    let sats = btc_to_sats(&btc).unwrap_or(0);
    let fiat = fiat_input_text(form.code, sats);
    with_amounts(form, fiat, btc, sats)
}

fn with_amounts(form: &CalculatorForm, fiat: String, btc: String, sats: u64) -> CalculatorForm {
    let fiat_text = match form.code {
        FiatCode::Usd => sats_fiat_usd(sats),
        FiatCode::Jpy => sats_fiat_jpy(sats),
    };
    CalculatorForm {
        fiat,
        btc,
        sats: sats.to_string(),
        fiat_text,
        ..form.clone()
    }
}

fn fiat_input_text(code: FiatCode, sats: u64) -> String {
    match code {
        FiatCode::Usd => {
            let shown = sats_fiat_usd(sats);
            let plain = shown.trim_start_matches('$').replace(',', "");
            plain.strip_suffix(".00").unwrap_or(&plain).to_string()
        }
        FiatCode::Jpy => {
            let shown = sats_fiat_jpy(sats);
            shown.trim_start_matches('\u{00A5}').replace(',', "")
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn calculator_field_names_are_fiat_bitcoin_and_satoshis() {
        assert_eq!(
            super::calculator_field_names(),
            ["fiat", "bitcoin", "satoshis"]
        );
    }
}
