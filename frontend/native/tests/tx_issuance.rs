//! Liquid issuance on a transaction input.
//!
//! `GET /tx/:txid` returns `TxInput.issuance`. The paint reads that object
//! after the input outpoint and its prevout line. An issuance object on
//! `prevout` is not the input issuance. A missing amount is not zero. A
//! commitment hex is not an amount. The paint does not unblind.

use std::collections::BTreeMap;
use std::rc::Rc;

use splora_api::{Network, api_root};
use splora_frontend_shared::wire::Transaction;
use splora_native::{
    ClientError, ClientRequest, ExplorerClient, ExplorerGet, ExplorerPost, Screen, ScreenHost,
};

const ASSET_ID: &str = "ce091c998b83c78bb71a632313ba3760f1763d9cfcffae02258ffa9865a37bd2";
const CONTRACT_HASH: &str = "cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc";
const ASSET_ENTROPY: &str = "eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee";
const REISSUE_ENTROPY: &str = "dddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddd";
const BLINDING_NONCE: &str = "1111111111111111111111111111111111111111111111111111111111111111";
const ASSET_COMMITMENT: &str = "deadbeefassetcommitment";
const TOKEN_COMMITMENT: &str = "deadbeeftokencommitment";

#[derive(Clone)]
struct Rec {
    network: Network,
    bodies: Rc<BTreeMap<String, String>>,
}

impl Rec {
    fn new(network: Network, bodies: BTreeMap<String, String>) -> Self {
        Self {
            network,
            bodies: Rc::new(bodies),
        }
    }
}

fn indexer_path(network: Network, url: &str) -> String {
    let root = api_root(network);
    url.strip_prefix(&root).unwrap_or(url).to_string()
}

impl ExplorerGet for Rec {
    fn get_text(&self, url: &str) -> Result<String, ClientError> {
        let path = indexer_path(self.network, url);
        self.bodies
            .get(&path)
            .cloned()
            .ok_or_else(|| ClientError::Http(format!("missing {path}")))
    }

    fn get_request(&self, request: &ClientRequest) -> Result<String, ClientError> {
        assert_eq!(request.method, "GET");
        self.bodies
            .get(&request.path)
            .cloned()
            .ok_or_else(|| ClientError::Http(format!("missing {}", request.path)))
    }
}

impl ExplorerPost for Rec {
    fn post_text(&self, request: &ClientRequest) -> Result<String, ClientError> {
        assert_eq!(request.method, "POST");
        self.bodies
            .get(&request.path)
            .cloned()
            .ok_or_else(|| ClientError::Http(format!("missing {}", request.path)))
    }
}

fn host_with(bodies: BTreeMap<String, String>) -> ScreenHost<Rec> {
    let rec = Rec::new(Network::Mainnet, bodies);
    ScreenHost::new(ExplorerClient::new(Network::Mainnet, rec))
}

fn has_line(text: &str, line: &str) -> bool {
    text.lines().any(|row| row == line)
}

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

fn open_transaction(txid: &str, body: &str) -> ScreenHost<Rec> {
    let mut host = host_with(BTreeMap::from([(format!("/tx/{txid}"), body.to_string())]));
    host.open_transaction(txid).expect("tx");
    assert_eq!(host.screen, Screen::Tx);
    host
}

fn assert_lines_in_order(text: &str, lines: &[&str]) {
    let rows: Vec<&str> = text.lines().collect();
    let mut start = 0;
    for line in lines {
        let Some(found) = rows[start..].iter().position(|row| row == line) else {
            panic!("missing line `{line}`\n{text}");
        };
        start += found + 1;
    }
}

fn assert_no_invented_paint(text: &str) {
    assert!(!text.contains("Confidential"), "{text}");
    assert!(!text.contains("0.00100000"), "{text}");
    assert!(!text.contains("LBTC"), "{text}");
    assert!(!text.contains('?'), "{text}");
    assert!(!text.contains('#'), "{text}");
    assert!(!text.contains("Issuance TX"), "{text}");
    assert!(
        !text
            .lines()
            .any(|row| row.starts_with("issuance blinding nonce")),
        "{text}"
    );
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

fn assert_no_issuance_line(text: &str) {
    assert!(
        !text.lines().any(|row| row.starts_with("issuance ")),
        "{text}"
    );
}

fn assert_confidential_not_an_integer(text: &str, label: &str) {
    let line = format!("issuance {label} amount Confidential");
    assert!(has_line(text, &line), "missing line `{line}`\n{text}");
    let prefix = format!("issuance {label} amount ");
    assert!(
        text.lines()
            .all(|row| !row.starts_with(&prefix) || row == line),
        "invented integer amount for {label}\n{text}"
    );
}

#[test]
fn explicit_issuance_paints_asset_contract_entropy_and_amounts() {
    let body = explicit_body();
    let tx: Transaction = serde_json::from_str(&body).expect("typed tx");
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
    let text = open_transaction("issue-tx", &body).rendered();
    assert_lines_in_order(
        &text,
        &[
            &format!("issuance asset {ASSET_ID}"),
            "issuance reissuance false",
            &format!("issuance contract {CONTRACT_HASH}"),
            &format!("issuance entropy {ASSET_ENTROPY}"),
            "issuance asset amount 5000",
            "issuance token amount 1",
        ],
    );
    assert_no_invented_paint(&text);
}

#[test]
fn blinded_issuance_amount_paints_confidential() {
    let body = blinded_body();
    let tx: Transaction = serde_json::from_str(&body).expect("typed tx");
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
    let text = open_transaction("blind-tx", &body).rendered();
    assert_confidential_not_an_integer(&text, "asset");
    assert_confidential_not_an_integer(&text, "token");
    assert!(!text.contains(ASSET_COMMITMENT), "{text}");
    assert!(!text.contains(TOKEN_COMMITMENT), "{text}");
}

#[test]
fn reissuance_paints_blinding_nonce_and_not_a_contract() {
    let body = reissuance_body();
    let tx: Transaction = serde_json::from_str(&body).expect("typed tx");
    let issued = tx.vin[0].issuance.as_ref().expect("issuance");
    assert!(issued.is_reissuance);
    assert_eq!(issued.contract_hash, None);
    assert_eq!(issued.asset_blinding_nonce.as_deref(), Some(BLINDING_NONCE));
    assert_eq!(issued.assetamount, Some(0));
    assert_eq!(issued.tokenamount, Some(100));
    let text = open_transaction("reissue-tx", &body).rendered();
    assert_lines_in_order(
        &text,
        &[
            "issuance reissuance true",
            &format!("issuance blinding nonce {BLINDING_NONCE}"),
            "issuance asset amount 0",
            "issuance token amount 100",
        ],
    );
    assert!(
        !text.lines().any(|row| row.starts_with("issuance contract")),
        "{text}"
    );
}

#[test]
fn missing_issuance_paints_no_issuance_line() {
    let body = plain_body();
    let tx: Transaction = serde_json::from_str(&body).expect("typed tx");
    assert!(tx.vin[0].issuance.is_none());
    assert_no_issuance_line(&open_transaction("plain-tx", &body).rendered());
}

#[test]
fn issuance_object_on_prevout_is_not_input_issuance() {
    let body = prevout_decoy_body();
    let tx: Transaction = serde_json::from_str(&body).expect("typed tx");
    assert!(tx.vin[0].issuance.is_none());
    assert!(body.contains("\"issuance\""));
    assert_no_issuance_line(&open_transaction("prevout-decoy-tx", &body).rendered());
}
