//! Peg-in, peg-out, and burn rows on the GPUI asset screen.
//!
//! Leptos paints `GET /asset/:id` and `GET /asset/:id/txs`. The liquid native
//! asset heading is `Peg In/Out and Burn Transactions`. Any other asset,
//! including liquid testnet, uses `Issuance and Burn Transactions`. The rows
//! are `table-tx-vin` and `table-tx-vout`. They are not `assetBox`. A missing
//! amount is blank. `Confidential` is the label when value is missing and a
//! commitment is present. A commitment is not an amount.

use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use splora_api::{Network, api_root};
use splora_frontend_shared::wire::{Transaction, TxOutput};
use splora_native::{
    ClientError, ClientRequest, ExplorerClient, ExplorerGet, ExplorerPost, Screen, ScreenHost,
};

const NATIVE_ASSET: &str = "6f0279e9ed041c3d710a9f57d0c02928416460c4b722ae3457a11eec381c526d";
const NATIVE_TEST_ASSET: &str = "144c654344aa716d6f3abcc1ca90e5641e4e2a7f633bc09fe3baf64585819a49";
const ISSUED_ASSET: &str = "ac3e0ff248c5051ffd61e00155b7122e5ebc04fd397a0ecbdd4f4e4a56232926";
const TXID: &str = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
const INPUT_TXID: &str = "cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc";
const BLIND_TXID: &str = "dddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddd";
const INPUT_ADDRESS: &str = "1BxoGcMg14oaH3CwHD2hF4gU9VcfgX5yoR";
const OUTPUT_ADDRESS: &str = "1FeexV6bAHb8ybZjqQMjJrcCrHGW9sb6uF";
const BLIND_IN_ADDRESS: &str = "lq1qblindinput";
const BLIND_OUT_ADDRESS: &str = "lq1qblindoutput";
const BLIND_ASSET_ADDRESS: &str = "lq1qblindasset";
const ASSET_OUT_ADDRESS: &str = "lq1qissuedout";
const BLANK_ADDRESS: &str = "lq1qblankamount";
const EMPTY_COMMIT_ADDRESS: &str = "lq1qemptycommit";
const PEG_HEADING: &str = "Peg In/Out and Burn Transactions";
const ISSUANCE_HEADING: &str = "Issuance and Burn Transactions";

/// Confidential consensus encodings are 33 bytes. The indexer hex-encodes them.
const VALUE_COMMITMENT: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
const ASSET_COMMITMENT: &str = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
const DECOY_COMMITMENT: &str = "cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc";

#[derive(Clone)]
struct Rec {
    network: Network,
    gets: Rc<RefCell<Vec<String>>>,
    bodies: Rc<BTreeMap<String, String>>,
}

impl Rec {
    fn new(network: Network, bodies: BTreeMap<String, String>) -> Self {
        Self {
            network,
            gets: Rc::new(RefCell::new(Vec::new())),
            bodies: Rc::new(bodies),
        }
    }

    fn gets(&self) -> Vec<String> {
        self.gets.borrow().clone()
    }
}

fn indexer_path(network: Network, url: &str) -> String {
    let root = api_root(network);
    url.strip_prefix(&root).unwrap_or(url).to_string()
}

impl ExplorerGet for Rec {
    fn get_text(&self, url: &str) -> Result<String, ClientError> {
        let path = indexer_path(self.network, url);
        self.gets.borrow_mut().push(path.clone());
        self.bodies
            .get(&path)
            .cloned()
            .ok_or_else(|| ClientError::Http(format!("missing {path}")))
    }

    fn get_request(&self, request: &ClientRequest) -> Result<String, ClientError> {
        assert_eq!(request.method, "GET");
        self.gets.borrow_mut().push(request.path.clone());
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

fn host_with(network: Network, bodies: BTreeMap<String, String>) -> (ScreenHost<Rec>, Rec) {
    let rec = Rec::new(network, bodies);
    let host = ScreenHost::new(ExplorerClient::new(network, rec.clone()));
    (host, rec)
}

fn has_line(text: &str, line: &str) -> bool {
    text.lines().any(|row| row == line)
}

fn assert_plain_paths(paths: &[String]) {
    for path in paths {
        assert!(!path.contains('?'), "{path}");
        assert!(!path.contains('#'), "{path}");
    }
}

fn asset_json(id: &str, name: &str, ticker: &str) -> String {
    format!(r#"{{"asset_id":"{id}","name":"{name}","ticker":"{ticker}"}}"#)
}

fn output_json(address: &str, extra: &str) -> String {
    let extra = if extra.is_empty() {
        String::new()
    } else {
        format!(",{extra}")
    };
    format!(
        r#"{{"scriptpubkey":"76a914cc88ac","scriptpubkey_asm":"OP_DUP","scriptpubkey_type":"p2pkh","scriptpubkey_address":"{address}"{extra}}}"#
    )
}

fn input_json(txid: &str, prevout: &str, extra: &str) -> String {
    let extra = if extra.is_empty() {
        String::new()
    } else {
        format!(",{extra}")
    };
    format!(
        r#"{{"txid":"{txid}","vout":0,"prevout":{prevout},"scriptsig":"","scriptsig_asm":"","is_coinbase":false,"sequence":1{extra}}}"#
    )
}

fn txs_json() -> String {
    let plain_in = input_json(
        INPUT_TXID,
        &output_json(INPUT_ADDRESS, r#""value":5000"#),
        &format!(r#""valuecommitment":"{DECOY_COMMITMENT}""#),
    );
    let blind_in = input_json(
        BLIND_TXID,
        &output_json(
            BLIND_IN_ADDRESS,
            &format!(r#""valuecommitment":"{VALUE_COMMITMENT}""#),
        ),
        "",
    );
    let plain_out = output_json(OUTPUT_ADDRESS, r#""value":4000"#);
    let blind_out = output_json(
        BLIND_OUT_ADDRESS,
        &format!(
            r#""valuecommitment":"{VALUE_COMMITMENT}","assetcommitment":"{ASSET_COMMITMENT}""#
        ),
    );
    let blind_asset = output_json(
        BLIND_ASSET_ADDRESS,
        &format!(r#""asset":"{ISSUED_ASSET}","valuecommitment":"{VALUE_COMMITMENT}""#),
    );
    let issued_out = output_json(
        ASSET_OUT_ADDRESS,
        &format!(r#""value":5000,"asset":"{ISSUED_ASSET}""#),
    );
    let blank_out = output_json(BLANK_ADDRESS, "");
    let empty_commit = output_json(
        EMPTY_COMMIT_ADDRESS,
        r#""valuecommitment":"","assetcommitment":"","asset":"""#,
    );
    format!(
        r#"[{{"txid":"{TXID}","version":2,"locktime":0,"vin":[{plain_in},{blind_in}],"vout":[{plain_out},{blind_out},{blind_asset},{issued_out},{blank_out},{empty_commit}],"size":10,"weight":40,"sigops":0,"fee":1}}]"#
    )
}

fn party_line(
    class: &str,
    address: &str,
    amount: &str,
    confidential: bool,
    asset: Option<&str>,
) -> String {
    let mut line = format!("{class} {address} value {amount}");
    if confidential {
        line.push_str(" Confidential");
    }
    if let Some(asset) = asset {
        line.push_str(" asset ");
        line.push_str(asset);
    }
    line
}

fn assert_plain_tables(text: &str) {
    let vin = party_line("table-tx-vin", INPUT_ADDRESS, "5000", false, None);
    let blind_vin = party_line("table-tx-vin", BLIND_IN_ADDRESS, "", true, None);
    let vout = party_line("table-tx-vout", OUTPUT_ADDRESS, "4000", false, None);
    let blind_vout = party_line("table-tx-vout", BLIND_OUT_ADDRESS, "", true, None);
    let blind_asset = party_line(
        "table-tx-vout",
        BLIND_ASSET_ADDRESS,
        "",
        true,
        Some(ISSUED_ASSET),
    );
    let issued = party_line(
        "table-tx-vout",
        ASSET_OUT_ADDRESS,
        "5000",
        false,
        Some(ISSUED_ASSET),
    );
    let blank = party_line("table-tx-vout", BLANK_ADDRESS, "", false, None);
    let empty_commit = party_line("table-tx-vout", EMPTY_COMMIT_ADDRESS, "", false, None);
    for line in [
        vin.as_str(),
        blind_vin.as_str(),
        vout.as_str(),
        blind_vout.as_str(),
        blind_asset.as_str(),
        issued.as_str(),
        blank.as_str(),
        empty_commit.as_str(),
    ] {
        assert!(has_line(text, line), "missing {line} in {text}");
    }
    assert!(!blank.contains('0'), "{blank}");
    assert!(!text.contains("assetBox"), "{text}");
    assert!(text.contains(TXID), "{text}");
    assert!(!text.contains("0.00100000"), "{text}");
    assert!(!text.contains(VALUE_COMMITMENT), "{text}");
    assert!(!text.contains(ASSET_COMMITMENT), "{text}");
    assert!(!text.contains(DECOY_COMMITMENT), "{text}");
    assert!(!text.contains("Error: Invalid blinding"), "{text}");
    assert!(!text.contains("Fiat price last updated"), "{text}");
    let vin_count = text
        .lines()
        .filter(|line| line.starts_with("table-tx-vin "))
        .count();
    let vout_count = text
        .lines()
        .filter(|line| line.starts_with("table-tx-vout "))
        .count();
    assert_eq!(vin_count, 2, "{text}");
    assert_eq!(vout_count, 6, "{text}");
}

fn open_asset(id: &str, name: &str, ticker: &str) -> (ScreenHost<Rec>, Rec) {
    let (mut host, rec) = host_with(
        Network::Liquid,
        BTreeMap::from([
            (format!("/tx/{id}"), "Transaction not found".to_string()),
            (format!("/block/{id}"), "Block not found".to_string()),
            (format!("/asset/{id}"), asset_json(id, name, ticker)),
            (format!("/asset/{id}/txs"), txs_json()),
        ]),
    );
    host.activate(Screen::Dashboard).expect("dashboard");
    host.push_str(id);
    host.reload().expect("asset");
    let paths = rec.gets();
    assert!(
        paths.iter().any(|path| path == &format!("/asset/{id}")),
        "{paths:?}"
    );
    assert!(
        paths.iter().any(|path| path == &format!("/asset/{id}/txs")),
        "{paths:?}"
    );
    assert_plain_paths(&paths);
    (host, rec)
}

#[test]
fn asset_paint_reads_typed_output_fields() {
    let src = include_str!("../src/screens.rs");
    assert!(
        !src.contains(".get(\"valuecommitment\")"),
        "screens.rs still scrapes the raw valuecommitment key"
    );
    assert!(
        !src.contains(".get(\"assetcommitment\")"),
        "screens.rs still scrapes the raw assetcommitment key"
    );
    assert!(
        !src.contains(".get(\"asset\")"),
        "screens.rs still scrapes the raw asset key"
    );
    let start = src
        .find("fn asset_party_line(")
        .expect("asset_party_line paints TxOutput fields");
    let rest = &src[start..];
    let end = rest.find("\nfn ").unwrap_or(rest.len());
    let body = &rest[..end];
    assert!(
        body.contains(".valuecommitment"),
        "asset paint does not read TxOutput.valuecommitment: {body}"
    );
    assert!(
        body.contains(".assetcommitment"),
        "asset paint does not read TxOutput.assetcommitment: {body}"
    );
    assert!(
        body.contains(".asset"),
        "asset paint does not read TxOutput.asset: {body}"
    );
    assert!(
        body.contains(".value"),
        "asset paint does not read TxOutput.value: {body}"
    );
    assert!(body.contains("Confidential"), "{body}");
    assert!(!body.contains("serde_json"), "{body}");
    assert!(!body.contains(".get("), "{body}");
    assert!(!body.contains("0.00100000"), "{body}");
    assert!(!body.contains("LBTC"), "{body}");
    assert!(!body.contains("assetBox"), "{body}");
    assert!(!body.contains("Fiat price last updated"), "{body}");

    let lines_at = src
        .find("fn asset_peg_burn_lines(")
        .expect("asset_peg_burn_lines paints the asset tables");
    let lines_rest = &src[lines_at..];
    let lines_end = lines_rest.find("\nfn ").unwrap_or(lines_rest.len());
    let lines = &lines_rest[..lines_end];
    assert!(lines.contains("table-tx-vin"), "{lines}");
    assert!(lines.contains("table-tx-vout"), "{lines}");
    assert!(lines.contains("asset_transaction_heading"), "{lines}");
    assert!(!lines.contains("assetBox"), "{lines}");

    let heading_at = src
        .find("fn asset_transaction_heading(")
        .expect("asset_transaction_heading is missing");
    let heading_rest = &src[heading_at..];
    let heading_end = heading_rest.find("\nfn ").unwrap_or(heading_rest.len());
    let heading = &heading_rest[..heading_end];
    assert!(heading.contains(PEG_HEADING), "{heading}");
    assert!(heading.contains(ISSUANCE_HEADING), "{heading}");
    assert!(heading.contains(NATIVE_ASSET), "{heading}");

    let apply = src
        .find("fn apply_hex_hit(")
        .expect("apply_hex_hit opens the asset screen");
    let apply_rest = &src[apply..];
    let apply_end = apply_rest.find("\n    fn ").unwrap_or(apply_rest.len());
    let apply_body = &apply_rest[..apply_end];
    assert!(
        apply_body.contains("asset_peg_burn_lines("),
        "the asset screen does not paint peg and burn rows: {apply_body}"
    );
}

#[test]
fn liquid_native_asset_shows_peg_in_out_and_burn_rows() {
    let txs = txs_json();
    let parsed: Vec<Transaction> = serde_json::from_str(&txs).expect("asset txs");
    let prevout = parsed[0].vin[0].prevout.as_ref().expect("prevout");
    assert_eq!(prevout.value, Some(5000));
    assert_eq!(prevout.valuecommitment, None);
    let blind_prev = parsed[0].vin[1].prevout.as_ref().expect("blind prevout");
    assert_eq!(blind_prev.value, None);
    assert_eq!(
        blind_prev.valuecommitment.as_deref(),
        Some(VALUE_COMMITMENT)
    );
    let blind_out: &TxOutput = &parsed[0].vout[1];
    assert_eq!(blind_out.value, None);
    assert_eq!(blind_out.valuecommitment.as_deref(), Some(VALUE_COMMITMENT));
    assert_eq!(blind_out.assetcommitment.as_deref(), Some(ASSET_COMMITMENT));
    assert_eq!(VALUE_COMMITMENT.len(), 66);
    assert_eq!(ASSET_COMMITMENT.len(), 66);

    let (host, _) = open_asset(NATIVE_ASSET, "Liquid Bitcoin", "L-BTC");
    let text = host.rendered();
    assert!(has_line(&text, PEG_HEADING), "{text}");
    assert!(!has_line(&text, ISSUANCE_HEADING), "{text}");
    assert!(text.contains("Liquid Bitcoin"), "{text}");
    assert!(text.contains("L-BTC"), "{text}");
    assert_plain_tables(&text);

    let (test_host, _) = open_asset(NATIVE_TEST_ASSET, "Liquid Bitcoin", "tLBTC");
    let test_native = test_host.rendered();
    assert!(has_line(&test_native, ISSUANCE_HEADING), "{test_native}");
    assert!(!has_line(&test_native, PEG_HEADING), "{test_native}");
    assert_plain_tables(&test_native);
}

#[test]
fn other_assets_use_the_issuance_heading() {
    let (host, _) = open_asset(ISSUED_ASSET, "Test Asset", "TEST");
    let text = host.rendered();
    assert!(has_line(&text, ISSUANCE_HEADING), "{text}");
    assert!(!has_line(&text, PEG_HEADING), "{text}");
    assert!(text.contains("Test Asset"), "{text}");
    assert!(text.contains("TEST"), "{text}");
    assert_plain_tables(&text);
}

#[test]
fn headings_stay_off_when_the_asset_screen_is_closed() {
    let id = NATIVE_ASSET;
    let (mut host, rec) = host_with(
        Network::Mainnet,
        BTreeMap::from([
            (format!("/tx/{id}"), "Transaction not found".to_string()),
            (format!("/block/{id}"), "Block not found".to_string()),
            (
                format!("/asset/{id}"),
                asset_json(id, "Liquid Bitcoin", "L-BTC"),
            ),
            (format!("/asset/{id}/txs"), txs_json()),
            (
                format!("/scripthash/{id}"),
                "address search disabled".to_string(),
            ),
        ]),
    );
    host.activate(Screen::Dashboard).expect("dashboard");
    host.push_str(id);
    let _ = host.reload();
    let text = host.rendered();
    assert!(!text.contains(PEG_HEADING), "{text}");
    assert!(!text.contains(ISSUANCE_HEADING), "{text}");
    assert!(!text.contains("table-tx-vin"), "{text}");
    assert!(!text.contains("table-tx-vout"), "{text}");
    assert!(
        rec.gets().iter().all(|path| !path.contains("/asset/")),
        "{:?}",
        rec.gets()
    );
    assert_plain_paths(&rec.gets());
}
