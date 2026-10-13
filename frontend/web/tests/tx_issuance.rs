//! Liquid issuance on a transaction input.
//!
//! `GET /tx/:txid` returns `TxInput.issuance`. The transaction page markup
//! reads that object after the input outpoint and its prevout. An issuance
//! object on `prevout` is not the input issuance. A missing amount is not
//! zero. A commitment hex is not an amount. The page does not unblind.
//! This test does not mark a Cypress row ported.

use splora_web::{parse_transaction_json, transaction_row_markup};

const ASSET_ID: &str = "ce091c998b83c78bb71a632313ba3760f1763d9cfcffae02258ffa9865a37bd2";
const CONTRACT_HASH: &str = "cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc";
const ASSET_ENTROPY: &str = "eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee";
const REISSUE_ENTROPY: &str = "dddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddd";
const BLINDING_NONCE: &str = "1111111111111111111111111111111111111111111111111111111111111111";
const ASSET_COMMITMENT: &str = "deadbeefassetcommitment";
const TOKEN_COMMITMENT: &str = "deadbeeftokencommitment";

fn output_with(address: &str, value: u64, extra: &str) -> String {
    let tail = if extra.is_empty() {
        String::new()
    } else {
        format!(",{extra}")
    };
    format!(
        r#"{{"scriptpubkey":"0014aa","scriptpubkey_asm":"OP_0","scriptpubkey_type":"v0_p2wpkh","scriptpubkey_address":"{address}","value":{value}{tail}}}"#
    )
}

fn input_with(txid: &str, prevout: &str, extra: &str) -> String {
    let tail = if extra.is_empty() {
        String::new()
    } else {
        format!(",{extra}")
    };
    format!(
        r#"{{"txid":"{txid}","vout":0,"prevout":{prevout},"scriptsig":"","scriptsig_asm":"","is_coinbase":false,"sequence":4294967295{tail}}}"#
    )
}

fn tx_body(txid: &str, vin: &str, vout: &str) -> String {
    format!(
        r#"{{"txid":"{txid}","version":1,"locktime":0,"vin":[{vin}],"vout":[{vout}],"size":10,"weight":40,"sigops":0,"fee":1,"status":{{"confirmed":true}}}}"#
    )
}

fn explicit_body() -> String {
    let issuance = format!(
        r#""issuance":{{"asset_id":"{ASSET_ID}","is_reissuance":false,"contract_hash":"{CONTRACT_HASH}","asset_entropy":"{ASSET_ENTROPY}","assetamount":5000,"tokenamount":1}}"#
    );
    tx_body(
        "issue-tx",
        &input_with("issue-prev", &output_with("bc1qissue", 10, ""), &issuance),
        &output_with("bc1qchange", 9, ""),
    )
}

fn blinded_body() -> String {
    let issuance = format!(
        r#""issuance":{{"asset_id":"{ASSET_ID}","is_reissuance":false,"asset_entropy":"{ASSET_ENTROPY}","assetamountcommitment":"{ASSET_COMMITMENT}","tokenamountcommitment":"{TOKEN_COMMITMENT}"}}"#
    );
    tx_body(
        "blind-tx",
        &input_with("blind-prev", &output_with("bc1qblind", 10, ""), &issuance),
        &output_with("bc1qchange", 9, ""),
    )
}

fn reissuance_body() -> String {
    let issuance = format!(
        r#""issuance":{{"asset_id":"{ASSET_ID}","is_reissuance":true,"asset_blinding_nonce":"{BLINDING_NONCE}","asset_entropy":"{REISSUE_ENTROPY}","assetamount":0,"tokenamount":100}}"#
    );
    tx_body(
        "reissue-tx",
        &input_with(
            "reissue-prev",
            &output_with("bc1qreissue", 10, ""),
            &issuance,
        ),
        &output_with("bc1qchange", 9, ""),
    )
}

fn plain_body() -> String {
    tx_body(
        "plain-tx",
        &input_with("plain-prev", &output_with("bc1qplain", 10, ""), ""),
        &output_with("bc1qchange", 9, ""),
    )
}

fn prevout_decoy_body() -> String {
    let issuance = format!(
        r#""issuance":{{"asset_id":"{ASSET_ID}","is_reissuance":false,"contract_hash":"{CONTRACT_HASH}","asset_entropy":"{ASSET_ENTROPY}","assetamount":5000,"tokenamount":1}}"#
    );
    tx_body(
        "prevout-decoy-tx",
        &input_with("decoy-prev", &output_with("bc1qdecoy", 10, &issuance), ""),
        &output_with("bc1qchange", 9, ""),
    )
}

fn omitted_amount_body() -> String {
    let issuance = format!(
        r#""issuance":{{"asset_id":"{ASSET_ID}","is_reissuance":false,"asset_entropy":"{ASSET_ENTROPY}"}}"#
    );
    tx_body(
        "omit-tx",
        &input_with("omit-prev", &output_with("bc1qomit", 10, ""), &issuance),
        &output_with("bc1qchange", 9, ""),
    )
}

fn assert_in_order(markup: &str, phrases: &[&str]) {
    let mut start = 0;
    for phrase in phrases {
        let Some(found) = markup[start..].find(phrase) else {
            panic!("missing `{phrase}`\n{markup}");
        };
        start += found + phrase.len();
    }
}

fn assert_no_invented_paint(markup: &str) {
    assert!(!markup.contains("Confidential"), "{markup}");
    assert!(!markup.contains("0.00100000"), "{markup}");
    assert!(!markup.contains("LBTC"), "{markup}");
    assert!(!markup.contains('?'), "{markup}");
    assert!(!markup.contains('#'), "{markup}");
    assert!(!markup.contains("Issuance TX"), "{markup}");
    assert!(!markup.contains("issuance blinding nonce"), "{markup}");
}

fn assert_no_issuance_phrase(markup: &str) {
    assert!(
        !markup.contains("issuance "),
        "painted an issuance line\n{markup}"
    );
}

fn assert_confidential_not_an_integer(markup: &str, kind: &str) {
    let line = format!("issuance {kind} amount Confidential");
    let prefix = format!("issuance {kind} amount ");
    assert!(markup.contains(&line), "missing `{line}`\n{markup}");
    let mut rest = markup;
    let mut saw = false;
    while let Some(at) = rest.find(&prefix) {
        let after = &rest[at + prefix.len()..];
        assert!(
            after.starts_with("Confidential"),
            "invented integer amount for {kind}\n{markup}"
        );
        saw = true;
        rest = &after["Confidential".len()..];
    }
    assert!(saw, "missing `{line}`\n{markup}");
}

#[test]
fn explicit_issuance_paints_asset_contract_entropy_and_amounts() {
    let body = explicit_body();
    let tx = parse_transaction_json(&body).expect("typed tx");
    let issued = tx.vin[0].issuance.as_ref().expect("issuance");
    assert!(!issued.is_reissuance);
    assert_eq!(issued.asset_id, ASSET_ID);
    assert_eq!(issued.contract_hash.as_deref(), Some(CONTRACT_HASH));
    assert_eq!(issued.asset_entropy, ASSET_ENTROPY);
    assert_eq!(issued.asset_blinding_nonce, None);
    assert_eq!(issued.assetamount, Some(5000));
    assert_eq!(issued.tokenamount, Some(1));
    assert_eq!(issued.assetamountcommitment, None);
    assert_eq!(issued.tokenamountcommitment, None);
    let markup = transaction_row_markup(&body);
    assert_in_order(
        &markup,
        &[
            &format!("issuance asset {ASSET_ID}"),
            "issuance reissuance false",
            &format!("issuance contract {CONTRACT_HASH}"),
            &format!("issuance entropy {ASSET_ENTROPY}"),
            "issuance asset amount 5000",
            "issuance token amount 1",
        ],
    );
    assert_no_invented_paint(&markup);
}

#[test]
fn blinded_issuance_amount_paints_confidential() {
    let body = blinded_body();
    let tx = parse_transaction_json(&body).expect("typed tx");
    let issued = tx.vin[0].issuance.as_ref().expect("issuance");
    assert_eq!(issued.assetamount, None);
    assert_eq!(issued.tokenamount, None);
    assert_eq!(
        issued.assetamountcommitment.as_deref(),
        Some(ASSET_COMMITMENT)
    );
    assert_eq!(
        issued.tokenamountcommitment.as_deref(),
        Some(TOKEN_COMMITMENT)
    );
    let markup = transaction_row_markup(&body);
    assert!(
        markup.contains(&format!("issuance asset {ASSET_ID}")),
        "{markup}"
    );
    assert!(markup.contains("issuance reissuance false"), "{markup}");
    assert_confidential_not_an_integer(&markup, "asset");
    assert_confidential_not_an_integer(&markup, "token");
    assert!(!markup.contains(ASSET_COMMITMENT), "{markup}");
    assert!(!markup.contains(TOKEN_COMMITMENT), "{markup}");
    assert!(!markup.contains("0.00100000"), "{markup}");
    assert!(!markup.contains("LBTC"), "{markup}");
    assert!(!markup.contains('?'), "{markup}");
    assert!(!markup.contains('#'), "{markup}");
}

#[test]
fn reissuance_paints_blinding_nonce_and_not_a_contract() {
    let body = reissuance_body();
    let tx = parse_transaction_json(&body).expect("typed tx");
    let issued = tx.vin[0].issuance.as_ref().expect("issuance");
    assert!(issued.is_reissuance);
    assert_eq!(issued.contract_hash, None);
    assert_eq!(issued.asset_blinding_nonce.as_deref(), Some(BLINDING_NONCE));
    assert_eq!(issued.assetamount, Some(0));
    assert_eq!(issued.tokenamount, Some(100));
    let markup = transaction_row_markup(&body);
    assert_in_order(
        &markup,
        &[
            &format!("issuance asset {ASSET_ID}"),
            "issuance reissuance true",
            &format!("issuance blinding nonce {BLINDING_NONCE}"),
            "issuance asset amount 0",
            "issuance token amount 100",
        ],
    );
    assert!(!markup.contains("issuance contract"), "{markup}");
    assert!(!markup.contains("Confidential"), "{markup}");
    assert!(!markup.contains("0.00100000"), "{markup}");
    assert!(!markup.contains("LBTC"), "{markup}");
    assert!(!markup.contains('?'), "{markup}");
    assert!(!markup.contains('#'), "{markup}");
}

#[test]
fn missing_issuance_paints_no_issuance_line() {
    let body = plain_body();
    let tx = parse_transaction_json(&body).expect("typed tx");
    assert!(tx.vin[0].issuance.is_none());
    assert_no_issuance_phrase(&transaction_row_markup(&body));
}

#[test]
fn issuance_object_on_prevout_is_not_input_issuance() {
    let body = prevout_decoy_body();
    let tx = parse_transaction_json(&body).expect("typed tx");
    assert!(tx.vin[0].issuance.is_none());
    assert!(body.contains("\"issuance\""));
    assert_no_issuance_phrase(&transaction_row_markup(&body));
}

#[test]
fn omitted_amount_stays_blank_not_zero() {
    let body = omitted_amount_body();
    let tx = parse_transaction_json(&body).expect("typed tx");
    let issued = tx.vin[0].issuance.as_ref().expect("issuance");
    assert_eq!(issued.assetamount, None);
    assert_eq!(issued.tokenamount, None);
    assert_eq!(issued.assetamountcommitment, None);
    assert_eq!(issued.tokenamountcommitment, None);
    let markup = transaction_row_markup(&body);
    assert!(
        markup.contains(&format!("issuance asset {ASSET_ID}")),
        "{markup}"
    );
    assert!(markup.contains("issuance reissuance false"), "{markup}");
    assert!(
        !markup.contains("issuance asset amount"),
        "missing amount was not left blank\n{markup}"
    );
    assert!(
        !markup.contains("issuance token amount"),
        "missing amount was not left blank\n{markup}"
    );
    assert!(!markup.contains("Confidential"), "{markup}");
    assert!(!markup.contains("0.00100000"), "{markup}");
    assert!(!markup.contains("LBTC"), "{markup}");
}
