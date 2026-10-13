//! Peg-in and Liquid Peg Out come from `TxInput.is_pegin` and `TxOutput.pegout`.
//!
//! A missing `is_pegin` is false. A missing `pegout` is none. A key buried in
//! `prevout`, or a `pegout` object on an input, is not that field. The paint
//! does not scan the raw body, and it does not invent an asset amount.

use std::collections::BTreeMap;
use std::rc::Rc;

use splora_api::{Network, api_root};
use splora_frontend_shared::wire::Transaction;
use splora_native::{
    ClientError, ClientRequest, ExplorerClient, ExplorerGet, ExplorerPost, Screen, ScreenHost,
};

const PEG_ADDRESS: &str = "1BxoGcMg14oaH3CwHD2hF4gU9VcfgX5yoR";
const PEGOUT: &str = r#""pegout":{"genesis_hash":"000000000019d6689c085ae165831e934ff763ae46a2a6c172b3f1b60a8ce26f","scriptpubkey":"76a91489abcdefabbaabbaabbaabbaabbaabbaabbaabba88ac","scriptpubkey_asm":"OP_DUP OP_HASH160 OP_EQUALVERIFY OP_CHECKSIG","scriptpubkey_address":"1BxoGcMg14oaH3CwHD2hF4gU9VcfgX5yoR"}"#;

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

fn peg_lines_body() -> &'static str {
    let src = include_str!("../src/screens.rs");
    let start = src
        .find("fn peg_lines(")
        .expect("peg_lines paints Peg-in and Liquid Peg Out");
    let rest = &src[start..];
    let end = rest.find("\nfn ").unwrap_or(rest.len());
    &rest[..end]
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

fn address_stats(address: &str) -> String {
    format!(
        r#"{{"address":"{address}","chain_stats":{{"tx_count":1,"funded_txo_count":1,"spent_txo_count":0,"funded_txo_sum":10,"spent_txo_sum":0}},"mempool_stats":{{"tx_count":0,"funded_txo_count":0,"spent_txo_count":0,"funded_txo_sum":0,"spent_txo_sum":0}}}}"#
    )
}

fn open_tx(txid: &str, body: &str) -> ScreenHost<Rec> {
    let mut host = host_with(BTreeMap::from([(format!("/tx/{txid}"), body.to_string())]));
    host.open_transaction(txid).expect("tx");
    assert_eq!(host.screen, Screen::Tx);
    host
}

fn open_address(address: &str, txs: &str) -> ScreenHost<Rec> {
    let mut host = host_with(BTreeMap::from([
        (format!("/address/{address}"), address_stats(address)),
        (format!("/address/{address}/txs"), txs.to_string()),
        (format!("/address/{address}/utxo"), "[]".to_string()),
    ]));
    host.open_address(address).expect("address");
    assert_eq!(host.screen, Screen::Address);
    host
}

fn assert_no_invented_amount(text: &str) {
    assert!(!text.contains("Fiat price last updated"), "{text}");
    assert!(!text.contains("0.00100000"), "{text}");
    assert!(!text.contains("LBTC"), "{text}");
}

fn assert_no_peg_labels(text: &str) {
    assert!(!has_line(text, "Peg-in"), "{text}");
    assert!(!has_line(text, "Liquid Peg Out"), "{text}");
    assert_no_invented_amount(text);
}

/// `is_pegin: true` sits on the prevout object. `TxInput.is_pegin` stays false.
fn prevout_decoy_body() -> String {
    let prevout = output_with("bc1qdecoy", 10, r#""is_pegin":true"#);
    tx_body(
        "decoy-pegin-tx",
        &input_with("decoy-prev", &prevout, ""),
        &output_with("bc1qchange", 9, ""),
    )
}

/// The peg-out object sits on the input. `TxOutput.pegout` stays none.
fn input_pegout_decoy_body() -> String {
    let pegout = r#""pegout":{"genesis_hash":"000000000019d6689c085ae165831e934ff763ae46a2a6c172b3f1b60a8ce26f","scriptpubkey":"76a91489abcdefabbaabbaabbaabbaabbaabbaabbaabba88ac","scriptpubkey_asm":"OP_DUP OP_HASH160 OP_EQUALVERIFY OP_CHECKSIG","scriptpubkey_address":"1BxoGcMg14oaH3CwHD2hF4gU9VcfgX5yoR"}"#;
    tx_body(
        "decoy-pegout-tx",
        &input_with("decoy-prev", &output_with("bc1qsource", 10, ""), pegout),
        &output_with("bc1qchange", 9, ""),
    )
}

fn typed_pegin_body() -> String {
    tx_body(
        "pegin-tx",
        &input_with(
            "pegin-prev",
            &output_with("bc1qpegin", 10, ""),
            r#""is_pegin":true"#,
        ),
        &output_with("bc1qchange", 9, ""),
    )
}

fn typed_pegout_body() -> String {
    tx_body(
        "pegout-tx",
        &input_with("pegout-prev", &output_with("bc1qsource", 10, ""), ""),
        &output_with("", 0, PEGOUT),
    )
}

fn omitted_body() -> String {
    tx_body(
        "omitted-tx",
        &input_with("omitted-prev", &output_with("bc1qsource", 10, ""), ""),
        &output_with("bc1qchange", 9, ""),
    )
}

fn false_and_null_body() -> String {
    tx_body(
        "false-tx",
        &input_with(
            "false-prev",
            &output_with("bc1qsource", 10, ""),
            r#""is_pegin":false"#,
        ),
        &output_with("bc1qchange", 9, r#""pegout":null"#),
    )
}

#[test]
fn peg_lines_reads_typed_fields_not_the_raw_body() {
    let body = peg_lines_body();
    assert!(
        body.contains(".is_pegin"),
        "peg_lines does not read TxInput.is_pegin: {body}"
    );
    assert!(
        body.contains(".pegout"),
        "peg_lines does not read TxOutput.pegout: {body}"
    );
    assert!(
        !body.contains("json_flag_true"),
        "peg_lines still scans the raw body for is_pegin: {body}"
    );
    assert!(
        !body.contains("json_field_object"),
        "peg_lines still scans the raw body for pegout: {body}"
    );
    assert!(!body.contains("Fiat price last updated"), "{body}");
    assert!(!body.contains("0.00100000"), "{body}");
    assert!(!body.contains("LBTC"), "{body}");
}

#[test]
fn prevout_is_pegin_does_not_paint_peg_in() {
    let body = prevout_decoy_body();
    let tx: Transaction = serde_json::from_str(&body).expect("typed tx");
    assert!(!tx.vin[0].is_pegin);
    assert!(tx.vout.iter().all(|output| output.pegout.is_none()));
    let host = open_tx("decoy-pegin-tx", &body);
    assert_no_peg_labels(&host.rendered());
}

#[test]
fn input_pegout_object_does_not_paint_liquid_peg_out() {
    let body = input_pegout_decoy_body();
    let tx: Transaction = serde_json::from_str(&body).expect("typed tx");
    assert!(!tx.vin[0].is_pegin);
    assert!(tx.vout.iter().all(|output| output.pegout.is_none()));
    let host = open_tx("decoy-pegout-tx", &body);
    assert_no_peg_labels(&host.rendered());
}

#[test]
fn typed_is_pegin_paints_peg_in() {
    let body = typed_pegin_body();
    let tx: Transaction = serde_json::from_str(&body).expect("typed tx");
    assert!(tx.vin[0].is_pegin);
    assert!(tx.vout.iter().all(|output| output.pegout.is_none()));
    let text = open_tx("pegin-tx", &body).rendered();
    assert!(has_line(&text, "Peg-in"), "{text}");
    assert!(!has_line(&text, "Liquid Peg Out"), "{text}");
    assert_no_invented_amount(&text);
}

#[test]
fn typed_pegout_paints_liquid_peg_out() {
    let body = typed_pegout_body();
    let tx: Transaction = serde_json::from_str(&body).expect("typed tx");
    assert!(!tx.vin[0].is_pegin);
    let pegout = tx.vout[0].pegout.as_ref().expect("pegout");
    assert_eq!(pegout.script_pubkey_address.as_deref(), Some(PEG_ADDRESS));
    let text = open_tx("pegout-tx", &body).rendered();
    assert!(has_line(&text, "Liquid Peg Out"), "{text}");
    assert!(!has_line(&text, "Peg-in"), "{text}");
    assert_no_invented_amount(&text);
}

#[test]
fn missing_is_pegin_and_pegout_paint_neither() {
    let body = omitted_body();
    let tx: Transaction = serde_json::from_str(&body).expect("typed tx");
    assert!(!tx.vin[0].is_pegin);
    assert!(tx.vout.iter().all(|output| output.pegout.is_none()));
    assert_no_peg_labels(&open_tx("omitted-tx", &body).rendered());
}

#[test]
fn false_is_pegin_and_null_pegout_paint_neither() {
    let body = false_and_null_body();
    let tx: Transaction = serde_json::from_str(&body).expect("typed tx");
    assert!(!tx.vin[0].is_pegin);
    assert!(tx.vout.iter().all(|output| output.pegout.is_none()));
    assert_no_peg_labels(&open_tx("false-tx", &body).rendered());
}

#[test]
fn address_list_ignores_nested_peg_keys() {
    let txs = format!("[{},{}]", prevout_decoy_body(), input_pegout_decoy_body());
    let parsed: Vec<Transaction> = serde_json::from_str(&txs).expect("typed list");
    assert!(
        parsed
            .iter()
            .all(|tx| tx.vin.iter().all(|input| !input.is_pegin))
    );
    assert!(
        parsed
            .iter()
            .all(|tx| tx.vout.iter().all(|output| output.pegout.is_none()))
    );
    assert_no_peg_labels(&open_address(PEG_ADDRESS, &txs).rendered());
}

#[test]
fn address_list_paints_typed_peg_labels() {
    let txs = format!("[{},{}]", typed_pegin_body(), typed_pegout_body());
    let parsed: Vec<Transaction> = serde_json::from_str(&txs).expect("typed list");
    assert!(
        parsed
            .iter()
            .any(|tx| tx.vin.iter().any(|input| input.is_pegin))
    );
    assert!(
        parsed
            .iter()
            .any(|tx| tx.vout.iter().any(|output| output.pegout.is_some()))
    );
    let text = open_address(PEG_ADDRESS, &txs).rendered();
    assert!(has_line(&text, "Peg-in"), "{text}");
    assert!(has_line(&text, "Liquid Peg Out"), "{text}");
    assert_no_invented_amount(&text);
}
