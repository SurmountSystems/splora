//! Screen JSON deserializes into `splora_frontend_shared::wire`.
//!
//! Tip hash and tip height stay plain text. `POST /tx` is plain text unless
//! the body is a `BroadcastResult` object.

use serde::de::DeserializeOwned;
use splora_frontend_shared::ParseError;
use splora_frontend_shared::wire::{
    AddressStats, Block, BroadcastResult, FeeEstimates, MempoolSummary, RecentTransaction,
    TestTxResult, Transaction, Utxo,
};

fn from_json<T: DeserializeOwned>(body: &str) -> Result<T, ParseError> {
    serde_json::from_str(body).map_err(ParseError::from)
}

pub fn parse_block_json(body: &str) -> Result<Block, ParseError> {
    from_json(body)
}

pub fn parse_blocks_json(body: &str) -> Result<Vec<Block>, ParseError> {
    from_json(body)
}

pub fn parse_transaction_json(body: &str) -> Result<Transaction, ParseError> {
    from_json(body)
}

pub fn parse_transactions_json(body: &str) -> Result<Vec<Transaction>, ParseError> {
    from_json(body)
}

pub fn parse_address_stats_json(body: &str) -> Result<AddressStats, ParseError> {
    from_json(body)
}

pub fn parse_utxos_json(body: &str) -> Result<Vec<Utxo>, ParseError> {
    from_json(body)
}

pub fn parse_recent_transactions_json(body: &str) -> Result<Vec<RecentTransaction>, ParseError> {
    from_json(body)
}

pub fn parse_mempool_summary_json(body: &str) -> Result<MempoolSummary, ParseError> {
    from_json(body)
}

pub fn parse_fee_estimates(body: &str) -> Result<FeeEstimates, ParseError> {
    from_json(body)
}

/// JSON object uses [`BroadcastResult`]. Any other body is the plain-text txid
/// from `POST /tx`.
pub fn parse_broadcast_result_json(body: &str) -> Result<BroadcastResult, ParseError> {
    let trimmed = body.trim();
    if let Ok(result) = serde_json::from_str::<BroadcastResult>(trimmed) {
        return Ok(result);
    }
    let txid = splora_frontend_shared::parse_broadcast(trimmed)?;
    Ok(BroadcastResult { txid })
}

pub fn parse_test_tx_results_json(body: &str) -> Result<Vec<TestTxResult>, ParseError> {
    from_json(body)
}

pub fn parse_blocks_tip_hash(body: &str) -> Result<String, ParseError> {
    splora_frontend_shared::parse_blocks_tip_hash(body)
}

pub fn parse_blocks_tip_height(body: &str) -> Result<String, ParseError> {
    splora_frontend_shared::parse_blocks_tip_height(body)
}

pub fn recent_txids(body: &str) -> Vec<String> {
    parse_recent_transactions_json(body)
        .map(|rows| rows.into_iter().map(|row| row.txid).collect())
        .unwrap_or_default()
}
