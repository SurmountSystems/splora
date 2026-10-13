//! Asset id and the confidential label come from the parsed output.
//!
//! `TxOutput.asset`, `value`, `valuecommitment`, and `assetcommitment` are
//! typed fields. A missing value is `None`, not zero. Confidential is the
//! label when value is missing and a commitment is present. A commitment is
//! not an amount. An explicit asset id links to `/asset/{id}`. A numeric
//! value stays the satoshi text.

use splora_web::{
    address_rows_markup, block_rows_markup, parse_transaction_json, transaction_row_markup,
};

/// `asset_id` in `ref/mempool/frontend/cypress/fixtures/assets.json` (Tether USD).
const ASSET_ID: &str = "ce091c998b83c78bb71a632313ba3760f1763d9cfcffae02258ffa9865a37bd2";

/// Confidential consensus encodings are 33 bytes. The indexer hex-encodes them.
const VALUE_COMMITMENT: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
const ASSET_COMMITMENT: &str = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";

fn issued_and_blinded_tx() -> String {
    format!(
        r#"{{
            "txid": "asset-tx",
            "version": 2,
            "locktime": 0,
            "vin": [{{
                "txid": "prev",
                "vout": 0,
                "prevout": {{
                    "scriptpubkey": "76a914bb88ac",
                    "scriptpubkey_asm": "OP_DUP",
                    "scriptpubkey_type": "p2pkh",
                    "scriptpubkey_address": "lq1qsource",
                    "value": 6000,
                    "asset": "{ASSET_ID}"
                }},
                "scriptsig": "",
                "scriptsig_asm": "",
                "is_coinbase": false,
                "sequence": 1
            }}],
            "vout": [
                {{
                    "scriptpubkey": "76a914cc88ac",
                    "scriptpubkey_asm": "OP_DUP",
                    "scriptpubkey_type": "p2pkh",
                    "scriptpubkey_address": "lq1qdest",
                    "value": 5000,
                    "asset": "{ASSET_ID}"
                }},
                {{
                    "scriptpubkey": "51",
                    "scriptpubkey_asm": "OP_TRUE",
                    "scriptpubkey_type": "op_return",
                    "valuecommitment": "{VALUE_COMMITMENT}",
                    "assetcommitment": "{ASSET_COMMITMENT}"
                }}
            ],
            "size": 200,
            "weight": 800,
            "sigops": 1,
            "fee": 100
        }}"#
    )
}

fn output_without_amount(extra: &str) -> String {
    let extra = if extra.is_empty() {
        String::new()
    } else {
        format!(",{extra}")
    };
    format!(
        r#"{{"txid":"plain-tx","version":2,"locktime":0,"vin":[{{"txid":"prev","vout":0,"prevout":null,"scriptsig":"","scriptsig_asm":"","is_coinbase":false,"sequence":1}}],"vout":[{{"scriptpubkey":"6a","scriptpubkey_asm":"OP_RETURN","scriptpubkey_type":"op_return"{extra}}}],"size":10,"weight":40,"sigops":0,"fee":1}}"#
    )
}

fn block_json() -> String {
    r#"{"id":"abc","height":1,"version":1,"timestamp":1,"tx_count":1,"size":1,"weight":4,"merkle_root":"mm","previousblockhash":"00","mediantime":1,"nonce":1,"bits":1,"difficulty":1.0}"#.to_string()
}

fn address_stats() -> String {
    r#"{"address":"lq1qdest","chain_stats":{"tx_count":1,"funded_txo_count":1,"spent_txo_count":0,"funded_txo_sum":5000,"spent_txo_sum":0},"mempool_stats":{"tx_count":0,"funded_txo_count":0,"spent_txo_count":0,"funded_txo_sum":0,"spent_txo_sum":0}}"#.to_string()
}

fn assert_no_invented_amount(markup: &str) {
    assert!(!markup.contains("0.00100000"), "{markup}");
    assert!(!markup.contains("0.02364760"), "{markup}");
    assert!(!markup.contains(VALUE_COMMITMENT), "{markup}");
    assert!(!markup.contains(ASSET_COMMITMENT), "{markup}");
    assert!(!markup.contains("Fiat price last updated"), "{markup}");
    assert!(!markup.contains("Error: Invalid blinding data"), "{markup}");
}

#[test]
fn asset_and_confidential_read_the_parsed_output_fields() {
    let src = include_str!("../src/rows.rs");
    assert!(
        !src.contains(".get(\"valuecommitment\")"),
        "rows.rs still scrapes the raw valuecommitment key"
    );
    assert!(
        !src.contains(".get(\"assetcommitment\")"),
        "rows.rs still scrapes the raw assetcommitment key"
    );
    assert!(
        !src.contains(".get(\"asset\")"),
        "rows.rs still scrapes the raw asset key"
    );
    assert!(
        !src.contains("fn raw_asset_and_confidential"),
        "rows.rs still has the raw asset scan"
    );

    let start = src
        .find("fn liquid_tx_labels")
        .expect("liquid_tx_labels is missing");
    let rest = &src[start..];
    let end = rest.find("\nfn ").unwrap_or(rest.len());
    let labels = &rest[..end];
    assert!(
        labels.contains("asset_labels_from_tx"),
        "liquid labels do not read asset fields from the parsed transaction: {labels}"
    );
    assert!(
        !labels.contains("raw_asset"),
        "liquid labels still scan the raw asset object: {labels}"
    );
    assert!(
        !labels.contains("serde_json"),
        "liquid labels still scan raw JSON: {labels}"
    );

    let reader_at = src
        .find("fn asset_labels_from_tx")
        .expect("asset labels are not read from the parsed transaction");
    let reader = &src[reader_at..];
    let confidential_at = reader
        .find("Confidential")
        .expect("parsed asset paint has no Confidential label");
    let after_label = confidential_at + "Confidential".len();
    let reader_end = reader[after_label..]
        .find("\nfn ")
        .map(|offset| after_label + offset)
        .unwrap_or(reader.len());
    let body = &reader[..reader_end];
    assert!(
        body.contains("valuecommitment"),
        "parsed asset paint does not read valuecommitment: {body}"
    );
    assert!(
        body.contains("assetcommitment"),
        "parsed asset paint does not read assetcommitment: {body}"
    );
    assert!(
        body.contains(".asset"),
        "parsed asset paint does not read asset: {body}"
    );
    assert!(
        body.contains(".value"),
        "parsed asset paint does not read value: {body}"
    );
    assert!(
        body.contains("/asset/"),
        "parsed asset paint does not link the asset id: {body}"
    );
    assert!(!body.contains("serde_json"), "{body}");
    assert!(!body.contains("0.00100000"), "{body}");
    assert!(!body.contains("Fiat price last updated"), "{body}");
}

#[test]
fn explicit_asset_id_links_and_the_value_stays_satoshis() {
    let body = issued_and_blinded_tx();
    let tx = parse_transaction_json(&body).expect("asset transaction");
    let prevout = tx.vin[0].prevout.as_ref().expect("prevout");
    assert_eq!(prevout.value, Some(6000));
    assert_eq!(prevout.asset.as_deref(), Some(ASSET_ID));
    assert_eq!(tx.vout[0].value, Some(5000));
    assert_eq!(tx.vout[0].asset.as_deref(), Some(ASSET_ID));
    assert_eq!(tx.vout[1].value, None);
    assert_ne!(tx.vout[1].value, Some(0));
    assert_eq!(
        tx.vout[1].valuecommitment.as_deref(),
        Some(VALUE_COMMITMENT)
    );
    assert_eq!(
        tx.vout[1].assetcommitment.as_deref(),
        Some(ASSET_COMMITMENT)
    );

    let markup = transaction_row_markup(&body);
    let link = format!("href=\"/asset/{ASSET_ID}\"");
    assert!(
        markup.matches(&link).count() >= 2,
        "prevout and output should each link the asset id: {markup}"
    );
    assert!(
        markup.contains("<span class=\"value\">6000</span>"),
        "{markup}"
    );
    assert!(
        markup.contains("<span class=\"value\">5000</span>"),
        "{markup}"
    );
    assert!(
        markup.contains("<span class=\"amount\">Confidential</span>"),
        "{markup}"
    );
    assert!(
        !markup.contains("<span class=\"value\">0</span>"),
        "{markup}"
    );
    assert_no_invented_amount(&markup);

    let listed = address_rows_markup(&address_stats(), &format!("[{body}]"), "[]");
    assert!(listed.contains(&link), "{listed}");
    assert!(
        listed.contains("<span class=\"amount\">Confidential</span>"),
        "{listed}"
    );
    assert_no_invented_amount(&listed);

    let block = block_rows_markup(&block_json(), &format!("[{body}]"));
    assert!(block.contains(&link), "{block}");
    assert!(
        block.contains("<span class=\"amount\">Confidential</span>"),
        "{block}"
    );
    assert_no_invented_amount(&block);
}

#[test]
fn missing_value_with_a_commitment_is_confidential_not_zero() {
    let body = output_without_amount(&format!(
        r#""valuecommitment":"{VALUE_COMMITMENT}","assetcommitment":"{ASSET_COMMITMENT}""#
    ));
    assert!(
        !body.contains("\"value\""),
        "the fixture must omit value, not invent one: {body}"
    );
    let tx = parse_transaction_json(&body).expect("blinded output");
    assert_eq!(tx.vout[0].value, None);
    let markup = transaction_row_markup(&body);
    assert!(
        markup.contains("<span class=\"amount\">Confidential</span>"),
        "{markup}"
    );
    assert!(
        !markup.contains("<span class=\"value\">0</span>"),
        "{markup}"
    );
    assert!(!markup.contains("href=\"/asset/"), "{markup}");
    assert_no_invented_amount(&markup);
}

#[test]
fn empty_commitment_does_not_invent_confidential_or_zero() {
    let body = output_without_amount(r#""valuecommitment":"","assetcommitment":"","asset":"""#);
    let tx = parse_transaction_json(&body).expect("empty commitment");
    assert_eq!(tx.vout[0].value, None);
    assert_eq!(tx.vout[0].asset.as_deref(), Some(""));
    assert_eq!(tx.vout[0].valuecommitment.as_deref(), Some(""));
    assert_eq!(tx.vout[0].assetcommitment.as_deref(), Some(""));
    let markup = transaction_row_markup(&body);
    assert!(!markup.contains("Confidential"), "{markup}");
    assert!(
        !markup.contains("<span class=\"value\">0</span>"),
        "{markup}"
    );
    assert!(!markup.contains("href=\"/asset/"), "{markup}");
    assert_no_invented_amount(&markup);
}

#[test]
fn blinded_amount_keeps_the_explicit_asset_link() {
    let body = output_without_amount(&format!(
        r#""asset":"{ASSET_ID}","valuecommitment":"{VALUE_COMMITMENT}""#
    ));
    assert!(
        !body.contains("\"value\""),
        "the fixture must omit value, not invent one: {body}"
    );
    let tx = parse_transaction_json(&body).expect("blinded explicit asset");
    assert_eq!(tx.vout[0].value, None);
    assert_ne!(tx.vout[0].value, Some(0));
    assert_eq!(tx.vout[0].asset.as_deref(), Some(ASSET_ID));
    assert_eq!(
        tx.vout[0].valuecommitment.as_deref(),
        Some(VALUE_COMMITMENT)
    );
    assert_eq!(tx.vout[0].assetcommitment, None);
    let markup = transaction_row_markup(&body);
    assert!(
        markup.contains("<span class=\"amount\">Confidential</span>"),
        "{markup}"
    );
    assert!(
        markup.contains(&format!("href=\"/asset/{ASSET_ID}\"")),
        "{markup}"
    );
    assert!(
        !markup.contains("<span class=\"value\">0</span>"),
        "{markup}"
    );
    assert_no_invented_amount(&markup);
}

#[test]
fn explicit_zero_stays_the_satoshi_text() {
    let body = r#"{"txid":"zero-tx","version":2,"locktime":0,"vin":[{"txid":"prev","vout":0,"prevout":null,"scriptsig":"","scriptsig_asm":"","is_coinbase":false,"sequence":1}],"vout":[{"scriptpubkey":"6a","scriptpubkey_asm":"OP_RETURN","scriptpubkey_type":"op_return","value":0}],"size":10,"weight":40,"sigops":0,"fee":1}"#;
    let tx = parse_transaction_json(body).expect("explicit zero");
    assert_eq!(tx.vout[0].value, Some(0));
    let markup = transaction_row_markup(body);
    assert!(
        markup.contains("<span class=\"value\">0</span>"),
        "{markup}"
    );
    assert!(!markup.contains("Confidential"), "{markup}");
    assert!(!markup.contains("href=\"/asset/"), "{markup}");
    assert_no_invented_amount(&markup);
}
