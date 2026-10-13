//! Liquid issuance on a shared transaction input.
//!
//! The indexer emits `issuance` only when its `liquid` feature is on and the
//! input has an issuance. A Bitcoin body omits the key. The shared crate has
//! no `liquid` feature. Amounts stay integers. A missing amount is absent,
//! not zero. Commitment strings are copied, not unblinded.

use splora_frontend_shared::wire::Transaction;

/// `asset_id` in `ref/mempool/frontend/cypress/fixtures/assets.json` (Tether USD).
const ASSET_ID: &str = "ce091c998b83c78bb71a632313ba3760f1763d9cfcffae02258ffa9865a37bd2";

const CONTRACT_HASH: &str = "cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc";
const ASSET_ENTROPY: &str = "eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee";
const REISSUE_ENTROPY: &str = "dddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddd";
const BLINDING_NONCE: &str = "1111111111111111111111111111111111111111111111111111111111111111";

/// Confidential `Value` consensus encodings are 33 bytes. The indexer
/// hex-encodes that serialization, so each commitment string is 66 hex
/// characters. This test does not decode them.
const ASSET_AMOUNT_COMMITMENT: &str =
    "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
const TOKEN_AMOUNT_COMMITMENT: &str =
    "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";

fn transaction_body() -> String {
    format!(
        r#"{{
            "txid": "issuance-tx",
            "version": 2,
            "locktime": 0,
            "vin": [
                {{
                    "txid": "prev-issue",
                    "vout": 0,
                    "prevout": null,
                    "scriptsig": "",
                    "scriptsig_asm": "",
                    "is_coinbase": false,
                    "sequence": 4294967295,
                    "issuance": {{
                        "asset_id": "{ASSET_ID}",
                        "is_reissuance": false,
                        "contract_hash": "{CONTRACT_HASH}",
                        "asset_entropy": "{ASSET_ENTROPY}",
                        "assetamount": 25000000,
                        "tokenamount": 1
                    }}
                }},
                {{
                    "txid": "prev-reissue",
                    "vout": 1,
                    "prevout": null,
                    "scriptsig": "",
                    "scriptsig_asm": "",
                    "is_coinbase": false,
                    "sequence": 4294967295,
                    "issuance": {{
                        "asset_id": "{ASSET_ID}",
                        "is_reissuance": true,
                        "asset_blinding_nonce": "{BLINDING_NONCE}",
                        "asset_entropy": "{REISSUE_ENTROPY}",
                        "assetamountcommitment": "{ASSET_AMOUNT_COMMITMENT}",
                        "tokenamountcommitment": "{TOKEN_AMOUNT_COMMITMENT}"
                    }}
                }},
                {{
                    "txid": "prev-plain",
                    "vout": 2,
                    "prevout": null,
                    "scriptsig": "",
                    "scriptsig_asm": "",
                    "is_coinbase": false,
                    "sequence": 4294967295
                }}
            ],
            "vout": [
                {{
                    "scriptpubkey": "6a",
                    "scriptpubkey_asm": "OP_RETURN",
                    "scriptpubkey_type": "op_return",
                    "value": 0
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
fn transaction_parses_issuance() {
    assert_eq!(CONTRACT_HASH.len(), 64);
    assert_eq!(ASSET_ENTROPY.len(), 64);
    assert_eq!(REISSUE_ENTROPY.len(), 64);
    assert_eq!(BLINDING_NONCE.len(), 64);
    assert_eq!(ASSET_AMOUNT_COMMITMENT.len(), 66);
    assert_eq!(TOKEN_AMOUNT_COMMITMENT.len(), 66);

    let tx: Transaction = serde_json::from_str(&transaction_body()).expect("issuance transaction");

    let issued = tx.vin[0].issuance.as_ref().expect("initial issuance");
    assert_eq!(issued.asset_id, ASSET_ID);
    assert!(!issued.is_reissuance);
    assert_eq!(issued.asset_blinding_nonce, None);
    assert_eq!(issued.contract_hash.as_deref(), Some(CONTRACT_HASH));
    assert_eq!(issued.asset_entropy, ASSET_ENTROPY);
    assert_eq!(issued.assetamount, Some(25_000_000));
    assert_eq!(issued.assetamountcommitment, None);
    assert_eq!(issued.tokenamount, Some(1));
    assert_eq!(issued.tokenamountcommitment, None);

    let reissued = tx.vin[1].issuance.as_ref().expect("reissuance");
    assert_eq!(reissued.asset_id, ASSET_ID);
    assert!(reissued.is_reissuance);
    assert_eq!(
        reissued.asset_blinding_nonce.as_deref(),
        Some(BLINDING_NONCE)
    );
    assert_eq!(reissued.contract_hash, None);
    assert_eq!(reissued.asset_entropy, REISSUE_ENTROPY);
    assert_eq!(reissued.assetamount, None);
    assert_eq!(
        reissued.assetamountcommitment.as_deref(),
        Some(ASSET_AMOUNT_COMMITMENT)
    );
    assert_eq!(reissued.tokenamount, None);
    assert_eq!(
        reissued.tokenamountcommitment.as_deref(),
        Some(TOKEN_AMOUNT_COMMITMENT)
    );

    assert_eq!(tx.vin[2].issuance, None);

    let encoded = serde_json::to_value(&tx).expect("serialize");
    assert!(encoded["vin"][2].get("issuance").is_none());
    assert!(
        encoded["vin"][0]["issuance"]
            .get("asset_blinding_nonce")
            .is_none()
    );
    assert!(
        encoded["vin"][0]["issuance"]
            .get("assetamountcommitment")
            .is_none()
    );
    assert!(
        encoded["vin"][0]["issuance"]
            .get("tokenamountcommitment")
            .is_none()
    );
    assert!(encoded["vin"][1]["issuance"].get("contract_hash").is_none());
    assert!(encoded["vin"][1]["issuance"].get("assetamount").is_none());
    assert!(encoded["vin"][1]["issuance"].get("tokenamount").is_none());
}
