//! Liquid keys on `GET /address/:script/utxo` that `UtxoValue` emits.
//!
//! `src/rest.rs` emits `asset`, `valuecommitment`, `assetcommitment`, `nonce`,
//! `noncecommitment`, `surjection_proof`, and `range_proof` only when the
//! indexer `liquid` feature is on. A missing key is absent. Proofs are the
//! hex strings `serde_hex` already writes. This test does not unblind them.
//! A missing `value` is `None`, not zero. A non-liquid body still sends the
//! number. There is no unblinding step.

use splora_frontend_shared::wire::Utxo;

/// `asset_id` in `ref/mempool/frontend/cypress/fixtures/assets.json` (Tether USD).
const ASSET_ID: &str = "ce091c998b83c78bb71a632313ba3760f1763d9cfcffae02258ffa9865a37bd2";

/// Confidential consensus encodings are 33 bytes. The indexer hex-encodes
/// that serialization, so each commitment string is 66 hex characters.
const VALUE_COMMITMENT: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
const ASSET_COMMITMENT: &str = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
const NONCE_COMMITMENT: &str = "cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc";

/// Explicit nonce is `hex::encode` of the nonce bytes (`UtxoValue` in `src/rest.rs`).
const NONCE: &str = "1111111111111111111111111111111111111111111111111111111111111111";

/// `surjection_proof` and `range_proof` are hex via `serde_hex`. Empty proofs
/// are omitted. These stand in for that hex. They are not decoded.
const SURJECTION_PROOF: &str = "aa11";
const RANGE_PROOF: &str = "bb22";

fn utxo_body() -> String {
    format!(
        r#"[
            {{
                "txid": "explicit-utxo",
                "vout": 0,
                "status": {{
                    "confirmed": true,
                    "block_height": 100,
                    "block_hash": "block-id-1",
                    "block_time": 1600000000
                }},
                "value": 5000,
                "asset": "{ASSET_ID}",
                "nonce": "{NONCE}"
            }},
            {{
                "txid": "blinded-utxo",
                "vout": 1,
                "status": {{"confirmed": false}},
                "value": 1,
                "valuecommitment": "{VALUE_COMMITMENT}",
                "assetcommitment": "{ASSET_COMMITMENT}",
                "noncecommitment": "{NONCE_COMMITMENT}",
                "surjection_proof": "{SURJECTION_PROOF}",
                "range_proof": "{RANGE_PROOF}"
            }},
            {{
                "txid": "bitcoin-utxo",
                "vout": 2,
                "status": {{"confirmed": true, "block_height": 100}},
                "value": 42000
            }},
            {{
                "txid": "omitted-utxo",
                "vout": 3,
                "status": {{"confirmed": false}},
                "valuecommitment": "{VALUE_COMMITMENT}",
                "assetcommitment": "{ASSET_COMMITMENT}"
            }}
        ]"#
    )
}

#[test]
fn utxo_parses_liquid_asset_nonce_and_proofs() {
    assert_eq!(VALUE_COMMITMENT.len(), 66);
    assert_eq!(ASSET_COMMITMENT.len(), 66);
    assert_eq!(NONCE_COMMITMENT.len(), 66);

    let rows: Vec<Utxo> = serde_json::from_str(&utxo_body()).expect("utxo list");
    assert_eq!(rows.len(), 4);

    let explicit = &rows[0];
    assert_eq!(explicit.value, Some(5000));
    assert_eq!(explicit.asset.as_deref(), Some(ASSET_ID));
    assert_eq!(explicit.nonce.as_deref(), Some(NONCE));
    assert_eq!(explicit.valuecommitment, None);
    assert_eq!(explicit.assetcommitment, None);
    assert_eq!(explicit.noncecommitment, None);
    assert_eq!(explicit.surjection_proof, None);
    assert_eq!(explicit.range_proof, None);

    let blinded = &rows[1];
    assert_eq!(blinded.value, Some(1));
    assert_eq!(blinded.asset, None);
    assert_eq!(blinded.nonce, None);
    assert_eq!(blinded.valuecommitment.as_deref(), Some(VALUE_COMMITMENT));
    assert_eq!(blinded.assetcommitment.as_deref(), Some(ASSET_COMMITMENT));
    assert_eq!(blinded.noncecommitment.as_deref(), Some(NONCE_COMMITMENT));
    assert_eq!(blinded.surjection_proof.as_deref(), Some(SURJECTION_PROOF));
    assert_eq!(blinded.range_proof.as_deref(), Some(RANGE_PROOF));

    let bitcoin = &rows[2];
    assert_eq!(bitcoin.value, Some(42000));
    assert_eq!(bitcoin.asset, None);
    assert_eq!(bitcoin.valuecommitment, None);
    assert_eq!(bitcoin.assetcommitment, None);
    assert_eq!(bitcoin.nonce, None);
    assert_eq!(bitcoin.noncecommitment, None);
    assert_eq!(bitcoin.surjection_proof, None);
    assert_eq!(bitcoin.range_proof, None);

    let omitted = &rows[3];
    assert_eq!(omitted.value, None);
    assert_eq!(omitted.asset, None);
    assert_eq!(omitted.nonce, None);
    assert_eq!(omitted.valuecommitment.as_deref(), Some(VALUE_COMMITMENT));
    assert_eq!(omitted.assetcommitment.as_deref(), Some(ASSET_COMMITMENT));
    assert_eq!(omitted.noncecommitment, None);
    assert_eq!(omitted.surjection_proof, None);
    assert_eq!(omitted.range_proof, None);

    let encoded = serde_json::to_value(&rows).expect("serialize");
    assert!(encoded[0].get("valuecommitment").is_none());
    assert!(encoded[0].get("assetcommitment").is_none());
    assert!(encoded[0].get("noncecommitment").is_none());
    assert!(encoded[0].get("surjection_proof").is_none());
    assert!(encoded[0].get("range_proof").is_none());
    assert!(encoded[1].get("asset").is_none());
    assert!(encoded[1].get("nonce").is_none());
    assert!(encoded[2].get("asset").is_none());
    assert!(encoded[2].get("nonce").is_none());
    assert!(encoded[2].get("valuecommitment").is_none());
    assert!(encoded[2].get("surjection_proof").is_none());
    assert!(encoded[2].get("range_proof").is_none());
    assert!(encoded[3].get("value").is_none());
    assert_eq!(encoded[0]["value"], 5000);
    assert_eq!(encoded[1]["value"], 1);
    assert_eq!(encoded[2]["value"], 42000);
}

/// A liquid UTXO may omit `value`. That is not a parse error and it is not zero.
#[test]
fn liquid_utxo_omitted_value_is_not_a_parse_error() {
    let body = r#"[
        {
            "txid": "blinded-omitted",
            "vout": 0,
            "status": {"confirmed": false},
            "valuecommitment": "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
        },
        {
            "txid": "explicit-zero",
            "vout": 1,
            "status": {"confirmed": true},
            "value": 0
        },
        {
            "txid": "present",
            "vout": 2,
            "status": {"confirmed": true},
            "value": 5000
        },
        {
            "txid": "json-null",
            "vout": 3,
            "status": {"confirmed": false},
            "value": null
        }
    ]"#;
    let rows: Vec<Utxo> = serde_json::from_str(body).expect("omitted value is not a parse error");
    assert_eq!(rows[0].value, None);
    assert_eq!(rows[1].value, Some(0));
    assert_eq!(rows[2].value, Some(5000));
    assert_eq!(rows[3].value, None);

    let encoded = serde_json::to_value(&rows).expect("serialize");
    assert!(encoded[0].get("value").is_none());
    assert_eq!(encoded[1]["value"], 0);
    assert_eq!(encoded[2]["value"], 5000);
    assert!(encoded[3].get("value").is_none());
}
