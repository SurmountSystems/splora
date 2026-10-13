//! Empty unblinded transaction amounts.
//!
//! Angular `unblinded TX / show empty unblinded TX` visits a Liquid
//! transaction with an empty `blinded` fragment and expects Confidential on
//! the first input and on both outputs
//! (`ref/mempool/frontend/cypress/e2e/liquid/liquid.spec.ts`). An empty
//! fragment does not unblind. The indexer match in `src/rest.rs` is
//! `GET /tx/:txid`. This client does not send `?` or `#`. Confidential is
//! the label when value is missing and a commitment is present. It is not
//! an amount, and the commitment hex is not painted. The tip line stays.

use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use splora_api::{Network, api_root};
use splora_frontend_shared::wire::Transaction;
use splora_native::{
    ClientError, ClientRequest, ExplorerClient, ExplorerGet, ExplorerPost, Screen, ScreenHost,
};

const TXID: &str = "f2f41c0850e8e7e3f1af233161fd596662e67c11ef10ed15943884186fbb7f46";
const VIN_COMMITMENT: &str = "aa11";
const OUT1_COMMITMENT: &str = "bb22";
const OUT2_COMMITMENT: &str = "cc33";

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
        let path = request.path.clone();
        self.gets.borrow_mut().push(path.clone());
        self.bodies
            .get(&path)
            .cloned()
            .ok_or_else(|| ClientError::Http(format!("missing {path}")))
    }
}

impl ExplorerPost for Rec {
    fn post_text(&self, request: &ClientRequest) -> Result<String, ClientError> {
        Err(ClientError::Http(format!("missing {}", request.path)))
    }
}

fn host_with(bodies: BTreeMap<String, String>) -> (ScreenHost<Rec>, Rec) {
    let rec = Rec::new(Network::Liquid, bodies);
    let host = ScreenHost::new(ExplorerClient::new(Network::Liquid, rec.clone()));
    (host, rec)
}

fn home_bodies(tx_path: &str, tx_body: &str) -> BTreeMap<String, String> {
    BTreeMap::from([
        ("/blocks".to_string(), "[]".to_string()),
        ("/mempool/recent".to_string(), "[]".to_string()),
        ("/blocks/tip/height".to_string(), "1".to_string()),
        ("/blocks/tip/hash".to_string(), "liqtip".to_string()),
        ("/fee-estimates".to_string(), "{}".to_string()),
        (
            "/mempool".to_string(),
            r#"{"count":0,"vsize":0,"total_fee":0,"fee_histogram":[]}"#.to_string(),
        ),
        ("/assets/registry".to_string(), "[]".to_string()),
        (tx_path.to_string(), tx_body.to_string()),
    ])
}

fn output_fields(address: &str, value: &str, extra: &str) -> String {
    let value_field = if value.is_empty() {
        String::new()
    } else {
        format!(r#""value":{value},"#)
    };
    let extra_field = if extra.is_empty() {
        String::new()
    } else {
        format!(",{extra}")
    };
    format!(
        r#"{{"scriptpubkey":"0014aa","scriptpubkey_asm":"OP_0","scriptpubkey_type":"v0_p2wpkh","scriptpubkey_address":"{address}",{value_field}"scriptpubkey_address_tail":""{extra_field}}}"#
    )
}

fn confidential_output(address: &str, commitment: &str) -> String {
    output_fields(
        address,
        "",
        &format!(r#""valuecommitment":"{commitment}","assetcommitment":"{commitment}aa""#),
    )
}

fn clear_output(address: &str, value: u64) -> String {
    output_fields(address, &value.to_string(), "")
}

fn blank_output(address: &str) -> String {
    output_fields(address, "", "")
}

fn input_with(prevout: &str) -> String {
    format!(
        r#"{{"txid":"prevouttx","vout":0,"prevout":{prevout},"scriptsig":"","scriptsig_asm":"","is_coinbase":false,"sequence":4294967295}}"#
    )
}

fn tx_body(vin: &str, vout: &str) -> String {
    format!(
        r#"{{"txid":"{TXID}","version":1,"locktime":0,"vin":[{vin}],"vout":[{vout}],"size":10,"weight":40,"sigops":0,"fee":1,"status":{{"confirmed":true}}}}"#
    )
}

fn empty_unblinded_body() -> String {
    tx_body(
        &input_with(&confidential_output("lq1qqvin", VIN_COMMITMENT)),
        &format!(
            "{},{}",
            confidential_output("lq1qqout1", OUT1_COMMITMENT),
            confidential_output("lq1qqout2", OUT2_COMMITMENT)
        ),
    )
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

fn assert_no_invented_unblinding(text: &str) {
    assert!(!text.contains("0.00100000"), "{text}");
    assert!(!text.contains("0.02364760"), "{text}");
    assert!(!text.contains("0.02465000"), "{text}");
    assert!(!text.contains("LBTC"), "{text}");
    assert!(!text.contains("Error: Invalid blinding"), "{text}");
    assert!(!text.contains("error-unblinded"), "{text}");
    assert!(!text.contains("Fiat price last updated"), "{text}");
    assert!(!text.contains(VIN_COMMITMENT), "{text}");
    assert!(!text.contains(OUT1_COMMITMENT), "{text}");
    assert!(!text.contains(OUT2_COMMITMENT), "{text}");
}

fn open_liquid_tx(body: &str) -> (ScreenHost<Rec>, Rec) {
    let path = format!("/tx/{TXID}");
    let (mut host, rec) = host_with(home_bodies(&path, body));
    host.activate(Screen::Dashboard).expect("dashboard");
    assert_eq!(host.dashboard.tip_line(), "tip height: 1 hash: liqtip");
    host.open_transaction(TXID).expect("tx");
    assert_eq!(host.screen, Screen::Tx);
    assert_eq!(host.dashboard.tip_line(), "tip height: 1 hash: liqtip");
    let paths = rec.gets();
    assert!(
        paths.iter().any(|got| got == &path),
        "transaction was not requested: {paths:?}"
    );
    assert!(
        paths.iter().all(|got| got != "/assets/registry/search"),
        "search path was requested: {paths:?}"
    );
    assert_plain_paths(&paths);
    (host, rec)
}

/// An empty blinded fragment leaves the input and both outputs Confidential.
#[test]
fn show_empty_unblinded_tx() {
    let body = empty_unblinded_body();
    assert!(
        !body.contains("\"value\""),
        "the fixture must omit value: {body}"
    );
    let tx: Transaction = serde_json::from_str(&body).expect("typed tx");
    assert!(tx.vin[0].prevout.as_ref().unwrap().value.is_none());
    assert!(tx.vout.iter().all(|output| output.value.is_none()));
    assert!(
        tx.vout
            .iter()
            .all(|output| output.valuecommitment.is_some())
    );

    let (host, _) = open_liquid_tx(&body);
    let text = host.rendered();
    assert!(
        has_line(&text, "input address lq1qqvin value Confidential"),
        "{text}"
    );
    assert!(
        has_line(&text, "output address lq1qqout1 value Confidential"),
        "{text}"
    );
    assert!(
        has_line(&text, "output address lq1qqout2 value Confidential"),
        "{text}"
    );
    assert_no_invented_unblinding(&text);
}

/// A published value stays that number. It is not Confidential.
#[test]
fn explicit_value_is_not_confidential() {
    let body = tx_body(
        &input_with(&clear_output("lq1qqvin", 10)),
        &clear_output("lq1qqout1", 9),
    );
    let tx: Transaction = serde_json::from_str(&body).expect("typed tx");
    assert_eq!(tx.vout[0].value, Some(9));
    let (host, _) = open_liquid_tx(&body);
    let text = host.rendered();
    assert!(has_line(&text, "input address lq1qqvin value 10"), "{text}");
    assert!(
        has_line(&text, "output address lq1qqout1 value 9"),
        "{text}"
    );
    assert!(!text.contains("Confidential"), "{text}");
    assert_no_invented_unblinding(&text);
}

/// A missing value with no commitment stays blank.
#[test]
fn missing_value_without_commitment_stays_blank() {
    let body = tx_body(
        &input_with(&blank_output("lq1qqvin")),
        &blank_output("lq1qqout1"),
    );
    let tx: Transaction = serde_json::from_str(&body).expect("typed tx");
    assert!(tx.vout[0].value.is_none());
    assert!(tx.vout[0].valuecommitment.is_none());
    let (host, _) = open_liquid_tx(&body);
    let text = host.rendered();
    assert!(has_line(&text, "input address lq1qqvin value "), "{text}");
    assert!(has_line(&text, "output address lq1qqout1 value "), "{text}");
    assert!(!text.contains("Confidential"), "{text}");
    assert_no_invented_unblinding(&text);
}
