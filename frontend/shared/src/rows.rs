//! List rows copied from [`crate::wire`] structs.
//!
//! These rows are not the explorer. Later rows are not in this module.

use crate::wire::{AddressStats, Block, Transaction, Utxo};

/// One block list row.
///
/// `id` is the block hash (`Block.id`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlockRow {
    pub height: u32,
    pub id: String,
    pub tx_count: u32,
    pub timestamp: u32,
}

impl From<&Block> for BlockRow {
    fn from(block: &Block) -> Self {
        Self {
            height: block.height,
            id: block.id.clone(),
            tx_count: block.tx_count,
            timestamp: block.timestamp,
        }
    }
}

impl From<Block> for BlockRow {
    fn from(block: Block) -> Self {
        Self {
            height: block.height,
            id: block.id,
            tx_count: block.tx_count,
            timestamp: block.timestamp,
        }
    }
}

/// One transaction list row.
///
/// `block_height` is `Transaction.status.block_height`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TxRow {
    pub txid: String,
    pub fee: u64,
    pub block_height: Option<u64>,
}

impl From<&Transaction> for TxRow {
    fn from(tx: &Transaction) -> Self {
        Self {
            txid: tx.txid.clone(),
            fee: tx.fee,
            block_height: tx.status.as_ref().and_then(|status| status.block_height),
        }
    }
}

impl From<Transaction> for TxRow {
    fn from(tx: Transaction) -> Self {
        Self {
            txid: tx.txid,
            fee: tx.fee,
            block_height: tx.status.and_then(|status| status.block_height),
        }
    }
}

/// One address list row.
///
/// `funded_sum` is `AddressStats.chain_stats.funded_txo_sum`.
/// `utxo_value` is `Utxo.value` when a UTXO is supplied.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AddressRow {
    pub address: String,
    pub funded_sum: u64,
    pub utxo_value: Option<u64>,
}

impl AddressRow {
    /// `None` when `stats.address` is absent.
    pub fn from_stats(stats: &AddressStats, utxo: Option<&Utxo>) -> Option<Self> {
        let address = stats.address.clone()?;
        Some(Self {
            address,
            funded_sum: stats.chain_stats.funded_txo_sum,
            utxo_value: utxo.map(|row| row.value),
        })
    }
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
        } = AddressRow::from_stats(&stats, Some(&utxos[0])).unwrap();
        assert_eq!(address, stats.address.clone().unwrap());
        assert_eq!(funded_sum, stats.chain_stats.funded_txo_sum);
        assert_eq!(utxo_value, Some(utxos[0].value));
        assert_eq!(address, "xyz");
        assert_eq!(funded_sum, 500_000);
        assert_eq!(utxo_value, Some(42_000));

        let AddressRow {
            address,
            funded_sum,
            utxo_value,
        } = AddressRow::from_stats(&stats, None).unwrap();
        assert_eq!(address, "xyz");
        assert_eq!(funded_sum, 500_000);
        assert_eq!(utxo_value, None);
    }
}
