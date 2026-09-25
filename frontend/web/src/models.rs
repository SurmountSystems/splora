pub use splora_frontend_shared::FeeEstimate;

pub fn parse_fee_estimates(
    body: &str,
) -> Result<Vec<FeeEstimate>, splora_frontend_shared::ParseError> {
    splora_frontend_shared::parse_fee_estimates(body)
}

pub fn parse_blocks_tip_hash(body: &str) -> Result<String, splora_frontend_shared::ParseError> {
    splora_frontend_shared::parse_blocks_tip_hash(body)
}

pub fn parse_blocks_tip_height(body: &str) -> Result<String, splora_frontend_shared::ParseError> {
    splora_frontend_shared::parse_blocks_tip_height(body)
}

pub fn recent_txids(body: &str) -> Vec<String> {
    splora_frontend_shared::parse_mempool_recent(body)
        .map(|rows| rows.into_iter().map(|row| row.txid).collect())
        .unwrap_or_default()
}
