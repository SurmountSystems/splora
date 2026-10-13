//! Liquid asset id and confidential commitment keys on the shared output type.
//!
//! The indexer emits `asset`, `valuecommitment`, and `assetcommitment` only
//! when its `liquid` feature is on. A numeric `value` is explicit. Omitting
//! `value` is the confidential case. The shared crate has no `liquid` feature.

use splora_frontend_shared::wire::Transaction;

/// `asset_id` in `ref/mempool/frontend/cypress/fixtures/assets.json` (Tether USD).
const ASSET_ID: &str = "ce091c998b83c78bb71a632313ba3760f1763d9cfcffae02258ffa9865a37bd2";

/// Confidential `Value` and `Asset` consensus encodings are 33 bytes
/// (`encoded_length` in elements `confidential.rs`). The indexer hex-encodes
/// that serialization, so each commitment string is 66 hex characters.
const VALUE_COMMITMENT: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
const ASSET_COMMITMENT: &str = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";

fn transaction_body() -> String {
    format!(
        r#"{{
            "txid": "asset-tx",
            "version": 2,
            "locktime": 0,
            "vin": [
                {{
                    "txid": "prev",
                    "vout": 0,
                    "prevout": null,
                    "scriptsig": "",
                    "scriptsig_asm": "",
                    "is_coinbase": false,
                    "sequence": 4294967295
                }}
            ],
            "vout": [
                {{
                    "scriptpubkey": "76a914bb88ac",
                    "scriptpubkey_asm": "OP_DUP",
                    "scriptpubkey_type": "p2pkh",
                    "value": 5000,
                    "asset": "{ASSET_ID}"
                }},
                {{
                    "scriptpubkey": "51",
                    "scriptpubkey_asm": "OP_TRUE",
                    "scriptpubkey_type": "op_return",
                    "valuecommitment": "{VALUE_COMMITMENT}",
                    "assetcommitment": "{ASSET_COMMITMENT}"
                }},
                {{
                    "scriptpubkey": "6a",
                    "scriptpubkey_asm": "OP_RETURN",
                    "scriptpubkey_type": "op_return",
                    "value": 1000
                }}
            ],
            "size": 200,
            "weight": 800,
            "sigops": 1,
            "fee": 100
        }}"#
    )
}

#[test]
fn transaction_parses_asset_and_commitment_fields() {
    assert_eq!(VALUE_COMMITMENT.len(), 66);
    assert_eq!(ASSET_COMMITMENT.len(), 66);

    let tx: Transaction = serde_json::from_str(&transaction_body()).expect("asset transaction");

    let issued = &tx.vout[0];
    assert_eq!(issued.value, Some(5000));
    assert_eq!(issued.asset.as_deref(), Some(ASSET_ID));
    assert_eq!(issued.valuecommitment, None);
    assert_eq!(issued.assetcommitment, None);

    let confidential = &tx.vout[1];
    assert_eq!(confidential.value, None);
    assert_eq!(confidential.asset, None);
    assert_eq!(
        confidential.valuecommitment.as_deref(),
        Some(VALUE_COMMITMENT)
    );
    assert_eq!(
        confidential.assetcommitment.as_deref(),
        Some(ASSET_COMMITMENT)
    );

    let plain = &tx.vout[2];
    assert_eq!(plain.value, Some(1000));
    assert_eq!(plain.asset, None);
    assert_eq!(plain.valuecommitment, None);
    assert_eq!(plain.assetcommitment, None);
}
