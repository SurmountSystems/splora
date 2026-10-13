//! Liquid registry, split asset history, and supply parsers.
//!
//! These routes exist only when the indexer is built with the liquid feature.
//! Path strings live in [`crate::paths`]. This module does not register routes.
//! Compact history still uses [`crate::wire::TxSummary`], not a second summary.

use serde::Deserialize;
use serde_json::Value;

use crate::parse::{ParseError, Tx, parse_address_txs};
use crate::wire::TxStatus;

/// One hit from `GET /assets/registry/search?q=`.
///
/// `AssetRegistrySearchResult` in `src/rest.rs`. `ticker` and `domain` are
/// omitted when the registry entry has none.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct AssetRegistrySearchHit {
    pub asset_id: String,
    pub name: String,
    #[serde(default)]
    pub ticker: Option<String>,
    #[serde(default)]
    pub domain: Option<String>,
}

/// `GET /assets/registry/:asset`.
///
/// `AssetMeta` in `src/elements/registry.rs`. `contract` and `entity` are JSON
/// values. A missing or null value stays null. `ticker` is omitted when absent.
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct AssetRegistryEntry {
    #[serde(default)]
    pub contract: Value,
    #[serde(default)]
    pub entity: Value,
    pub precision: u8,
    pub name: String,
    #[serde(default)]
    pub ticker: Option<String>,
}

/// `issuance_txin` on an issued asset (`TxInput` in `src/util/transaction.rs`).
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct AssetTxIn {
    pub txid: String,
    pub vin: u32,
}

/// `issuance_prevout` on an issued asset.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct AssetOutPoint {
    pub txid: String,
    pub vout: u32,
}

/// `IssuedAssetStats` in `src/elements/asset.rs`.
///
/// `reissuance_tokens` is null when the token amount is confidential.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct IssuedAssetStats {
    pub tx_count: u64,
    pub issuance_count: u64,
    pub issued_amount: u64,
    pub burned_amount: u64,
    pub has_blinded_issuances: bool,
    #[serde(default)]
    pub reissuance_tokens: Option<u64>,
    pub burned_reissuance_tokens: u64,
}

/// `PeggedAssetStats` in `src/elements/asset.rs`.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct PeggedAssetStats {
    pub tx_count: u64,
    pub peg_in_count: u64,
    pub peg_in_amount: u64,
    pub peg_out_count: u64,
    pub peg_out_amount: u64,
    pub burn_count: u64,
    pub burned_amount: u64,
}

/// One issued row from `GET /assets/registry`.
///
/// `IssuedAsset` in `src/elements/asset.rs`, with `AssetMeta` flattened.
/// Registry keys are absent when the on-chain asset has no registry metadata.
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct IssuedRegistryAsset {
    pub asset_id: String,
    pub issuance_txin: AssetTxIn,
    pub issuance_prevout: AssetOutPoint,
    pub reissuance_token: String,
    #[serde(default)]
    pub contract_hash: Option<String>,
    pub status: TxStatus,
    pub chain_stats: IssuedAssetStats,
    pub mempool_stats: IssuedAssetStats,
    #[serde(default)]
    pub contract: Value,
    #[serde(default)]
    pub entity: Value,
    #[serde(default)]
    pub precision: Option<u8>,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub ticker: Option<String>,
}

/// One pegged row from `GET /assets/registry`.
///
/// `PeggedAsset` in `src/elements/asset.rs`.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct NativeRegistryAsset {
    pub asset_id: String,
    pub chain_stats: PeggedAssetStats,
    pub mempool_stats: PeggedAssetStats,
}

/// One element of `GET /assets/registry`.
///
/// `LiquidAsset` in `src/elements/asset.rs` is an untagged enum. An issued
/// asset has `issuance_txin`. A pegged asset does not.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(untagged)]
pub enum RegistryAsset {
    Issued(IssuedRegistryAsset),
    Native(NativeRegistryAsset),
}

/// `GET /assets/registry/search?q=`.
pub fn parse_assets_registry_search(body: &str) -> Result<Vec<AssetRegistrySearchHit>, ParseError> {
    Ok(serde_json::from_str(body)?)
}

/// `GET /assets/registry/:asset`.
pub fn parse_asset_registry(body: &str) -> Result<AssetRegistryEntry, ParseError> {
    Ok(serde_json::from_str(body)?)
}

/// `GET /assets/registry`. The JSON array is the page. The total is the
/// `X-Total-Results` header, parsed by [`parse_assets_registry_total`].
pub fn parse_assets_registry(body: &str) -> Result<Vec<RegistryAsset>, ParseError> {
    Ok(serde_json::from_str(body)?)
}

/// `X-Total-Results` on `GET /assets/registry`.
pub fn parse_assets_registry_total(header: &str) -> Result<u64, ParseError> {
    let text = header.trim();
    if text.is_empty() {
        return Err(ParseError::new("registry total was empty"));
    }
    text.parse()
        .map_err(|_| ParseError::new("registry total was not an integer"))
}

/// `GET /asset/:asset/txs/chain` and the `:last_seen_txid` page.
///
/// The body is the same transaction array as [`parse_address_txs`].
pub fn parse_asset_txs_chain(body: &str) -> Result<Vec<Tx>, ParseError> {
    parse_address_txs(body)
}

/// `GET /asset/:asset/txs/mempool`.
pub fn parse_asset_txs_mempool(body: &str) -> Result<Vec<Tx>, ParseError> {
    parse_address_txs(body)
}

/// `GET /asset/:asset/supply`. Plain integer text (`supply.to_string()`).
pub fn parse_asset_supply(body: &str) -> Result<String, ParseError> {
    let text = body.trim();
    if text.is_empty() || !text.bytes().all(|byte| byte.is_ascii_digit()) {
        return Err(ParseError::new("supply was not an integer"));
    }
    Ok(text.to_string())
}

/// `GET /asset/:asset/supply/decimal`.
///
/// Decimal text when precision is positive. The same route returns an integer
/// string when precision is zero (`src/rest.rs`).
pub fn parse_asset_supply_decimal(body: &str) -> Result<String, ParseError> {
    let text = body.trim();
    if text.is_empty() {
        return Err(ParseError::new("supply was empty"));
    }
    let value: f64 = text
        .parse()
        .map_err(|_| ParseError::new("supply was not a decimal"))?;
    if !value.is_finite() {
        return Err(ParseError::new("supply was not a decimal"));
    }
    Ok(text.to_string())
}
