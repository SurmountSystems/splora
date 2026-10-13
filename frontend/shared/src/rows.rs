//! List rows copied from [`crate::wire`] structs.
//!
//! These rows are not the explorer. Block, transaction, address, and fee-estimate fields are copied here.

use crate::wire::{AddressStats, Block, FeeEstimates, ScriptStats, Transaction, Utxo};

/// One block list row.
///
/// `id` is the block hash (`Block.id`).
#[derive(Debug, Clone, PartialEq)]
pub struct BlockRow {
    pub id: String,
    pub height: u32,
    pub version: u32,
    pub timestamp: u32,
    pub tx_count: u32,
    pub size: u32,
    pub weight: u32,
    pub merkle_root: String,
    pub previous_block_hash: Option<String>,
    pub median_time: u32,
    pub nonce: u32,
    pub bits: u32,
    pub difficulty: f64,
}

impl From<&Block> for BlockRow {
    fn from(block: &Block) -> Self {
        Self {
            id: block.id.clone(),
            height: block.height,
            version: block.version,
            timestamp: block.timestamp,
            tx_count: block.tx_count,
            size: block.size,
            weight: block.weight,
            merkle_root: block.merkle_root.clone(),
            previous_block_hash: block.previous_block_hash.clone(),
            median_time: block.median_time,
            nonce: block.nonce,
            bits: block.bits,
            difficulty: block.difficulty,
        }
    }
}

impl From<Block> for BlockRow {
    fn from(block: Block) -> Self {
        Self {
            id: block.id,
            height: block.height,
            version: block.version,
            timestamp: block.timestamp,
            tx_count: block.tx_count,
            size: block.size,
            weight: block.weight,
            merkle_root: block.merkle_root,
            previous_block_hash: block.previous_block_hash,
            median_time: block.median_time,
            nonce: block.nonce,
            bits: block.bits,
            difficulty: block.difficulty,
        }
    }
}

/// One transaction list row.
///
/// `block_height` is `Transaction.status.block_height`.
/// `confirmed` is `status.confirmed`, or false when `status` is absent.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TxRow {
    pub txid: String,
    pub version: u32,
    pub lock_time: u32,
    pub size: u32,
    pub weight: u32,
    pub sigops: u32,
    pub fee: u64,
    pub confirmed: bool,
    pub block_height: Option<u64>,
    pub block_hash: Option<String>,
    pub block_time: Option<u64>,
}

impl From<&Transaction> for TxRow {
    fn from(tx: &Transaction) -> Self {
        let (confirmed, block_height, block_hash, block_time) = match tx.status.as_ref() {
            Some(status) => (
                status.confirmed,
                status.block_height,
                status.block_hash.clone(),
                status.block_time,
            ),
            None => (false, None, None, None),
        };
        Self {
            txid: tx.txid.clone(),
            version: tx.version,
            lock_time: tx.lock_time,
            size: tx.size,
            weight: tx.weight,
            sigops: tx.sigops,
            fee: tx.fee,
            confirmed,
            block_height,
            block_hash,
            block_time,
        }
    }
}

impl From<Transaction> for TxRow {
    fn from(tx: Transaction) -> Self {
        let (confirmed, block_height, block_hash, block_time) = match tx.status {
            Some(status) => (
                status.confirmed,
                status.block_height,
                status.block_hash,
                status.block_time,
            ),
            None => (false, None, None, None),
        };
        Self {
            txid: tx.txid,
            version: tx.version,
            lock_time: tx.lock_time,
            size: tx.size,
            weight: tx.weight,
            sigops: tx.sigops,
            fee: tx.fee,
            confirmed,
            block_height,
            block_hash,
            block_time,
        }
    }
}

impl TxRow {
    pub fn sat_per_vbyte(&self) -> Option<f64> {
        sat_per_vbyte_from_weight(self.fee, self.weight)
    }
}

/// Virtual size is `weight / 4` (integer division), not `(weight + 3) / 4`.
pub fn sat_per_vbyte_from_weight(fee: u64, weight: u32) -> Option<f64> {
    let virtual_size = weight / 4;
    if virtual_size == 0 {
        None
    } else {
        Some(fee as f64 / virtual_size as f64)
    }
}

pub fn sat_per_vbyte_from_vsize(fee: u64, vsize: u32) -> Option<f64> {
    if vsize == 0 {
        None
    } else {
        Some(fee as f64 / vsize as f64)
    }
}

/// Chain or mempool script totals copied from [`ScriptStats`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScriptStatsRow {
    pub tx_count: u64,
    pub funded_txo_count: u64,
    pub spent_txo_count: u64,
    pub funded_txo_sum: u64,
    pub spent_txo_sum: u64,
}

impl From<&ScriptStats> for ScriptStatsRow {
    fn from(stats: &ScriptStats) -> Self {
        Self {
            tx_count: stats.tx_count,
            funded_txo_count: stats.funded_txo_count,
            spent_txo_count: stats.spent_txo_count,
            funded_txo_sum: stats.funded_txo_sum,
            spent_txo_sum: stats.spent_txo_sum,
        }
    }
}

/// One unspent output copied from [`Utxo`].
///
/// `confirmed` is `Utxo.status.confirmed`. `value` is the UTXO amount.
/// A missing amount is `None`, not zero.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UtxoRow {
    pub txid: String,
    pub vout: u32,
    pub value: Option<u64>,
    pub confirmed: bool,
}

/// One address list row.
///
/// `funded_sum` is `AddressStats.chain_stats.funded_txo_sum`.
/// `utxo_value` is the supplied UTXO's amount. A missing amount is `None`,
/// not zero. No supplied UTXO is also `None`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AddressRow {
    pub address: String,
    pub funded_sum: u64,
    pub utxo_value: Option<u64>,
    pub chain_stats: ScriptStatsRow,
    pub mempool_stats: ScriptStatsRow,
    pub utxo: Option<UtxoRow>,
}

impl AddressRow {
    /// `None` when `stats.address` is absent.
    pub fn from_stats(stats: &AddressStats, utxo: Option<&Utxo>) -> Option<Self> {
        let address = stats.address.clone()?;
        Some(Self {
            address,
            funded_sum: stats.chain_stats.funded_txo_sum,
            utxo_value: utxo.and_then(|row| row.value),
            chain_stats: ScriptStatsRow::from(&stats.chain_stats),
            mempool_stats: ScriptStatsRow::from(&stats.mempool_stats),
            utxo: utxo.map(|row| UtxoRow {
                txid: row.txid.clone(),
                vout: row.vout,
                value: row.value,
                confirmed: row.status.confirmed,
            }),
        })
    }
}

/// One confirmation-target fee estimate copied from [`FeeEstimates`].
#[derive(Debug, Clone, PartialEq)]
pub struct FeeEstimateRow {
    pub target: u16,
    pub sat_per_vbyte: f64,
}

/// One row per map entry, in ascending confirmation-target order.
pub fn fee_estimate_rows(estimates: &FeeEstimates) -> Vec<FeeEstimateRow> {
    estimates
        .0
        .iter()
        .map(|(&target, &sat_per_vbyte)| FeeEstimateRow {
            target,
            sat_per_vbyte,
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture(name: &str) -> String {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures")
            .join(name);
        std::fs::read_to_string(path).unwrap()
    }

    #[test]
    fn block_row_copies_height_id_tx_count_and_timestamp() {
        let block: Block = serde_json::from_str(&fixture("block.json")).unwrap();
        let BlockRow {
            height,
            id,
            tx_count,
            timestamp,
            ..
        } = BlockRow::from(&block);
        assert_eq!(height, block.height);
        assert_eq!(id, block.id);
        assert_eq!(tx_count, block.tx_count);
        assert_eq!(timestamp, block.timestamp);
        assert_eq!(height, 100);
        assert_eq!(id, "block-id-1");
        assert_eq!(tx_count, 12);
        assert_eq!(timestamp, 1_600_000_000);

        let owned = block.clone();
        let BlockRow {
            height,
            id,
            tx_count,
            timestamp,
            ..
        } = BlockRow::from(owned);
        assert_eq!(height, block.height);
        assert_eq!(id, block.id);
        assert_eq!(tx_count, block.tx_count);
        assert_eq!(timestamp, block.timestamp);
    }

    #[test]
    fn tx_row_copies_txid_fee_and_block_height() {
        let tx: Transaction = serde_json::from_str(&fixture("tx.json")).unwrap();
        let TxRow {
            txid,
            fee,
            block_height,
            ..
        } = TxRow::from(&tx);
        assert_eq!(txid, tx.txid);
        assert_eq!(fee, tx.fee);
        assert_eq!(
            block_height,
            tx.status.as_ref().and_then(|status| status.block_height)
        );
        assert_eq!(txid, "tx-1");
        assert_eq!(fee, 4500);
        assert_eq!(block_height, Some(100));

        let owned = tx.clone();
        let TxRow {
            txid,
            fee,
            block_height,
            ..
        } = TxRow::from(owned);
        assert_eq!(txid, tx.txid);
        assert_eq!(fee, tx.fee);
        assert_eq!(block_height, Some(100));

        let unconfirmed: Vec<Transaction> =
            serde_json::from_str(&fixture("address_txs.json")).unwrap();
        let TxRow {
            txid,
            fee,
            block_height,
            ..
        } = TxRow::from(&unconfirmed[0]);
        assert_eq!(txid, unconfirmed[0].txid);
        assert_eq!(fee, unconfirmed[0].fee);
        assert_eq!(block_height, None);
        assert_eq!(txid, "addr-tx-1");
        assert_eq!(fee, 111);
    }

    #[test]
    fn address_row_copies_address_funded_sum_and_optional_utxo_value() {
        let stats: AddressStats = serde_json::from_str(&fixture("address.json")).unwrap();
        let utxos: Vec<Utxo> = serde_json::from_str(&fixture("address_utxos.json")).unwrap();
        let AddressRow {
            address,
            funded_sum,
            utxo_value,
            ..
        } = AddressRow::from_stats(&stats, Some(&utxos[0])).unwrap();
        assert_eq!(address, stats.address.clone().unwrap());
        assert_eq!(funded_sum, stats.chain_stats.funded_txo_sum);
        assert_eq!(utxo_value, utxos[0].value);
        assert_eq!(address, "xyz");
        assert_eq!(funded_sum, 500_000);
        assert_eq!(utxo_value, Some(42_000));

        let AddressRow {
            address,
            funded_sum,
            utxo_value,
            ..
        } = AddressRow::from_stats(&stats, None).unwrap();
        assert_eq!(address, "xyz");
        assert_eq!(funded_sum, 500_000);
        assert_eq!(utxo_value, None);
    }

    #[test]
    fn fee_rate_uses_weight_over_four_and_refuses_zero_weight() {
        assert_eq!(sat_per_vbyte_from_weight(4500, 720), Some(25.0));
        let tx: Transaction = serde_json::from_str(&fixture("tx.json")).unwrap();
        assert_eq!(tx.weight, 720);
        assert_eq!(tx.fee, 4500);
        assert_eq!(TxRow::from(&tx).sat_per_vbyte(), Some(25.0));
        assert_eq!(sat_per_vbyte_from_weight(4500, 0), None);
        assert_eq!(sat_per_vbyte_from_weight(1, 1), None);
        assert_eq!(sat_per_vbyte_from_weight(1, 2), None);
        assert_eq!(sat_per_vbyte_from_weight(1, 3), None);
        assert_eq!(sat_per_vbyte_from_weight(0, 4), Some(0.0));
    }

    #[test]
    fn recent_fee_rate_uses_vsize_and_refuses_zero_vsize() {
        let rows: Vec<crate::wire::RecentTransaction> =
            serde_json::from_str(&fixture("mempool_recent.json")).unwrap();
        assert_eq!(rows[0].fee, 800);
        assert_eq!(rows[0].vsize, 141);
        assert_eq!(
            sat_per_vbyte_from_vsize(rows[0].fee, rows[0].vsize),
            Some(800.0 / 141.0)
        );
        assert_eq!(sat_per_vbyte_from_vsize(rows[0].fee, 0), None);
    }

    #[test]
    fn block_row_keeps_difficulty_size_and_weight() {
        let block: Block = serde_json::from_str(&fixture("block.json")).unwrap();
        let row = BlockRow::from(&block);
        assert_eq!(row.difficulty, block.difficulty);
        assert_eq!(row.size, block.size);
        assert_eq!(row.weight, block.weight);
        assert_eq!(row.size, 1500);
        assert_eq!(row.weight, 4000);
        assert_eq!(row.difficulty, 1.0);
        assert_eq!(row.version, block.version);
        assert_eq!(row.version, 536_870_912);
        assert_eq!(row.merkle_root, block.merkle_root);
        assert_eq!(row.merkle_root, "merkle-root-1");
        assert_eq!(row.previous_block_hash, block.previous_block_hash);
        assert_eq!(row.previous_block_hash.as_deref(), Some("prev-block-1"));
        assert_eq!(row.median_time, block.median_time);
        assert_eq!(row.median_time, 1_599_990_000);
        assert_eq!(row.nonce, block.nonce);
        assert_eq!(row.nonce, 7);
        assert_eq!(row.bits, block.bits);
        assert_eq!(row.bits, 386_604_799);

        let owned = BlockRow::from(block.clone());
        assert_eq!(owned.difficulty, block.difficulty);
        assert_eq!(owned.size, block.size);
        assert_eq!(owned.weight, block.weight);
        assert_eq!(owned.version, block.version);
        assert_eq!(owned.merkle_root, block.merkle_root);
        assert_eq!(owned.previous_block_hash, block.previous_block_hash);
        assert_eq!(owned.median_time, block.median_time);
        assert_eq!(owned.nonce, block.nonce);
        assert_eq!(owned.bits, block.bits);

        let genesis = r#"{"id":"g","height":0,"version":1,"timestamp":0,"tx_count":1,"size":1,"weight":4,"merkle_root":"m","mediantime":0,"nonce":0,"bits":0,"difficulty":1}"#;
        let parsed: Block = serde_json::from_str(genesis).unwrap();
        let genesis_row = BlockRow::from(&parsed);
        assert_eq!(genesis_row.previous_block_hash, None);
        assert_eq!(genesis_row.difficulty, parsed.difficulty);
        assert_eq!(genesis_row.size, parsed.size);
        assert_eq!(genesis_row.weight, parsed.weight);
        assert_eq!(genesis_row.difficulty, 1.0);
        assert_eq!(genesis_row.size, 1);
        assert_eq!(genesis_row.weight, 4);
    }

    #[test]
    fn tx_row_copies_version_lock_time_size_weight_and_confirmation() {
        let tx: Transaction = serde_json::from_str(&fixture("tx.json")).unwrap();
        let row = TxRow::from(&tx);
        let status = tx.status.as_ref().expect("status");
        assert_eq!(row.version, tx.version);
        assert_eq!(row.lock_time, tx.lock_time);
        assert_eq!(row.size, tx.size);
        assert_eq!(row.weight, tx.weight);
        assert_eq!(row.sigops, tx.sigops);
        assert!(row.confirmed);
        assert_eq!(row.confirmed, status.confirmed);
        assert_eq!(row.block_hash, status.block_hash);
        assert_eq!(row.block_time, status.block_time);

        let owned = TxRow::from(tx.clone());
        assert_eq!(owned.version, tx.version);
        assert_eq!(owned.lock_time, tx.lock_time);
        assert_eq!(owned.size, tx.size);
        assert_eq!(owned.weight, tx.weight);
        assert_eq!(owned.sigops, tx.sigops);
        assert_eq!(owned.confirmed, status.confirmed);
        assert_eq!(owned.block_hash, status.block_hash);
        assert_eq!(owned.block_time, status.block_time);

        let unconfirmed: Vec<Transaction> =
            serde_json::from_str(&fixture("address_txs.json")).unwrap();
        let unconfirmed_row = TxRow::from(&unconfirmed[0]);
        assert!(!unconfirmed_row.confirmed);
        assert_eq!(unconfirmed_row.block_height, None);
        assert_eq!(unconfirmed_row.block_hash, None);
        assert_eq!(unconfirmed_row.block_time, None);

        let missing_status = r#"{"txid":"no-status","version":1,"locktime":0,"vin":[],"vout":[],"size":10,"weight":40,"sigops":0,"fee":1}"#;
        let parsed: Transaction = serde_json::from_str(missing_status).unwrap();
        assert!(parsed.status.is_none());
        let absent = TxRow::from(parsed);
        assert!(!absent.confirmed);
        assert_eq!(absent.block_height, None);
        assert_eq!(absent.block_hash, None);
        assert_eq!(absent.block_time, None);
    }

    #[test]
    fn address_row_copies_chain_mempool_stats_and_utxo() {
        let stats: AddressStats = serde_json::from_str(&fixture("address.json")).unwrap();
        let utxos: Vec<Utxo> = serde_json::from_str(&fixture("address_utxos.json")).unwrap();
        let row = AddressRow::from_stats(&stats, Some(&utxos[0])).unwrap();
        assert_eq!(row.funded_sum, stats.chain_stats.funded_txo_sum);
        assert_eq!(row.funded_sum, 500_000);
        assert_eq!(row.utxo_value, utxos[0].value);
        assert_eq!(row.utxo_value, Some(42_000));
        assert_eq!(row.chain_stats.tx_count, stats.chain_stats.tx_count);
        assert_eq!(
            row.chain_stats.funded_txo_count,
            stats.chain_stats.funded_txo_count
        );
        assert_eq!(
            row.chain_stats.spent_txo_count,
            stats.chain_stats.spent_txo_count
        );
        assert_eq!(
            row.chain_stats.funded_txo_sum,
            stats.chain_stats.funded_txo_sum
        );
        assert_eq!(
            row.chain_stats.spent_txo_sum,
            stats.chain_stats.spent_txo_sum
        );
        assert_eq!(row.mempool_stats.tx_count, stats.mempool_stats.tx_count);
        assert_eq!(
            row.mempool_stats.funded_txo_count,
            stats.mempool_stats.funded_txo_count
        );
        assert_eq!(
            row.mempool_stats.spent_txo_count,
            stats.mempool_stats.spent_txo_count
        );
        assert_eq!(
            row.mempool_stats.funded_txo_sum,
            stats.mempool_stats.funded_txo_sum
        );
        assert_eq!(
            row.mempool_stats.spent_txo_sum,
            stats.mempool_stats.spent_txo_sum
        );
        assert_eq!(row.chain_stats, ScriptStatsRow::from(&stats.chain_stats));
        assert_eq!(
            row.mempool_stats,
            ScriptStatsRow::from(&stats.mempool_stats)
        );
        let utxo = row.utxo.expect("utxo");
        assert_eq!(utxo.txid, utxos[0].txid);
        assert_eq!(utxo.vout, utxos[0].vout);
        assert_eq!(utxo.value, utxos[0].value);
        assert_eq!(utxo.confirmed, utxos[0].status.confirmed);
        assert_eq!(utxo.txid, "utxo-tx-1");
        assert_eq!(utxo.vout, 2);
        assert!(utxo.confirmed);

        let bare = AddressRow::from_stats(&stats, None).unwrap();
        assert_eq!(bare.utxo, None);
        assert_eq!(bare.utxo_value, None);
        assert_eq!(bare.funded_sum, stats.chain_stats.funded_txo_sum);
    }

    #[test]
    fn fee_estimate_rows_follow_btree_target_order() {
        let estimates: crate::wire::FeeEstimates =
            serde_json::from_str(r#"{"6":5,"1":12.5,"144":1}"#).unwrap();
        let rows = fee_estimate_rows(&estimates);
        assert_eq!(rows.len(), 3);
        assert_eq!(rows[0].target, 1);
        assert_eq!(rows[0].sat_per_vbyte, 12.5);
        assert_eq!(rows[1].target, 6);
        assert_eq!(rows[1].sat_per_vbyte, 5.0);
        assert_eq!(rows[2].target, 144);
        assert_eq!(rows[2].sat_per_vbyte, 1.0);
    }
}
