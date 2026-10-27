use elements::{AssetId, TxIn, confidential::Value, encode::serialize, issuance::ContractHash};

pub mod asset;
pub mod peg;
mod registry;

use asset::get_issuance_entropy;
pub use asset::{LiquidAsset, lookup_asset};
pub use registry::{AssetMeta, AssetRegistry, AssetSorting};

#[derive(Serialize, Deserialize, Clone)]
pub struct IssuanceValue {
    pub asset_id: String,
    pub is_reissuance: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub asset_blinding_nonce: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub contract_hash: Option<String>,
    pub asset_entropy: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub assetamount: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub assetamountcommitment: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tokenamount: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tokenamountcommitment: Option<String>,
}

impl From<&TxIn> for IssuanceValue {
    fn from(txin: &TxIn) -> Self {
        let issuance = &txin.asset_issuance;
        let is_reissuance = !issuance.asset_blinding_nonce.is_null();

        let asset_entropy = get_issuance_entropy(txin).expect("invalid issuance");
        let asset_id = AssetId::from_entropy(asset_entropy);

        let contract_hash = if !is_reissuance {
            Some(ContractHash::from_byte_array(
                issuance.asset_entropy.to_byte_array(),
            ))
        } else {
            None
        };

        IssuanceValue {
            asset_id: hex::encode(asset_id.as_byte_array()),
            asset_entropy: hex::encode(asset_entropy.as_byte_array()),
            contract_hash: contract_hash.map(|h| hex::encode(h.as_byte_array())),
            is_reissuance,
            asset_blinding_nonce: if is_reissuance {
                Some(hex::encode(issuance.asset_blinding_nonce.as_byte_array()))
            } else {
                None
            },
            assetamount: match issuance.amount {
                Value::Explicit(value) => Some(value),
                Value::Null => Some(0),
                Value::Confidential(..) => None,
            },
            assetamountcommitment: match issuance.amount {
                Value::Confidential(..) => Some(hex::encode(serialize(&issuance.amount))),
                _ => None,
            },
            tokenamount: match issuance.inflation_keys {
                Value::Explicit(value) => Some(value),
                Value::Null => Some(0),
                Value::Confidential(..) => None,
            },
            tokenamountcommitment: match issuance.inflation_keys {
                Value::Confidential(..) => Some(hex::encode(serialize(&issuance.inflation_keys))),
                _ => None,
            },
        }
    }
}
