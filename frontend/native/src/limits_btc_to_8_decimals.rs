//! Native paint for the BTC amount field. Delegates to the shared truncator.

pub fn paint_limits_btc_to_8_decimals(input: &str) -> String {
    splora_frontend_shared::truncate_btc_decimals(input)
}
