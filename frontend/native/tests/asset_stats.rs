//! Chain stats on the GPUI asset screen.
//!
//! `GET /asset/:id` in `src/rest.rs` returns the issued or pegged asset
//! document. The Angular asset page paints precision, the issuer domain,
//! the issuance transaction, and the chain amounts from that document.
//! A blinded issuance stays the word Confidential. A pegged asset has no
//! name in that document. The integers are the chain stats. This screen
//! does not format them into an unblinded amount.

use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use splora_api::{Network, api_root};
use splora_native::{
    ClientError, ClientRequest, ExplorerClient, ExplorerGet, ExplorerPost, Screen, ScreenHost,
};

const NATIVE_ASSET: &str = "6f0279e9ed041c3d710a9f57d0c02928416460c4b722ae3457a11eec381c526d";
const ISSUED: &str = "ac3e0ff248c5051ffd61e00155b7122e5ebc04fd397a0ecbdd4f4e4a56232926";
const ISSUANCE_TX: &str = "cdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcd";
const PREV_TX: &str = "efefefefefefefefefefefefefefefefefefefefefefefefefefefefefefefef";
const REISSUE: &str = "1111111111111111111111111111111111111111111111111111111111111111";

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

fn home() -> BTreeMap<String, String> {
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
    ])
}

fn open_asset(id: &str, document: &str) -> (ScreenHost<Rec>, Rec) {
    let mut bodies = home();
    bodies.insert(format!("/asset/{id}"), document.to_string());
    let rec = Rec::new(Network::Liquid, bodies);
    let mut host = ScreenHost::new(ExplorerClient::new(Network::Liquid, rec.clone()));
    host.activate(Screen::Dashboard).expect("dashboard");
    host.open_row(&format!("row asset {id}"))
        .expect("open asset");
    (host, rec)
}

fn has_line(text: &str, line: &str) -> bool {
    text.lines().any(|row| row == line)
}

fn assert_plain_paths(paths: &[String]) {
    for path in paths {
        assert!(!path.contains('?'), "{path}");
        assert!(!path.contains('#'), "{path}");
        assert!(
            path != "/assets/registry/search",
            "asset stats must not search the registry: {paths:?}"
        );
    }
}

fn assert_tip_first(text: &str) {
    let first = text.lines().next().unwrap_or("");
    assert_eq!(first, "tip height: 1 hash: liqtip", "{text}");
}

fn issued_stats(blinded: bool, issued_amount: u64, burned_amount: u64) -> String {
    format!(
        r#"{{"asset_id":"{ISSUED}","name":"Test Asset","ticker":"TEST","precision":2,"entity":{{"domain":"example.com"}},"contract":null,"issuance_txin":{{"txid":"{ISSUANCE_TX}","vin":0}},"issuance_prevout":{{"txid":"{PREV_TX}","vout":1}},"reissuance_token":"{REISSUE}","status":{{"confirmed":true}},"chain_stats":{{"tx_count":3,"issuance_count":1,"issued_amount":{issued_amount},"burned_amount":{burned_amount},"has_blinded_issuances":{blinded},"burned_reissuance_tokens":0}},"mempool_stats":{{"tx_count":0,"issuance_count":0,"issued_amount":0,"burned_amount":0,"has_blinded_issuances":false,"burned_reissuance_tokens":0}}}}"#
    )
}

fn pegged_stats() -> String {
    format!(
        r#"{{"asset_id":"{NATIVE_ASSET}","chain_stats":{{"tx_count":4,"peg_in_count":2,"peg_in_amount":1000,"peg_out_count":1,"peg_out_amount":200,"burn_count":1,"burned_amount":50}},"mempool_stats":{{"tx_count":0,"peg_in_count":0,"peg_in_amount":0,"peg_out_count":0,"peg_out_amount":0,"burn_count":0,"burned_amount":0}}}}"#
    )
}

/// An issued asset paints precision, the issuer, the issuance transaction,
/// and the chain issued, burned, and circulating integers.
#[test]
fn issued_asset_paints_precision_issuer_issuance_and_chain_amounts() {
    let (host, rec) = open_asset(ISSUED, &issued_stats(false, 5000, 200));
    let text = host.rendered();
    assert_tip_first(&text);
    assert!(has_line(&text, "Precision 2"), "{text}");
    assert!(has_line(&text, "Issuer example.com"), "{text}");
    assert!(
        has_line(&text, &format!("Issuance TX {ISSUANCE_TX}")),
        "{text}"
    );
    assert!(has_line(&text, "Issued amount 5000"), "{text}");
    assert!(has_line(&text, "Burned amount 200"), "{text}");
    assert!(has_line(&text, "Circulating amount 4800"), "{text}");
    assert!(text.contains("Test Asset"), "{text}");
    assert!(text.contains("TEST"), "{text}");
    assert!(!text.contains("0.00100000"), "{text}");
    assert!(!text.contains("LBTC"), "{text}");
    assert!(
        host.painted_rows()
            .iter()
            .any(|row| row == &format!("row tx txid {ISSUANCE_TX}")),
        "{:?}",
        host.painted_rows()
    );
    let paths = rec.gets();
    assert!(
        paths.iter().any(|path| path == &format!("/asset/{ISSUED}")),
        "{paths:?}"
    );
    assert_plain_paths(&paths);
}

/// A blinded issuance keeps Confidential on the amount rows and does not
/// paint the integers from the document.
#[test]
fn blinded_issuance_paints_confidential_not_the_integers() {
    let (host, rec) = open_asset(ISSUED, &issued_stats(true, 424242, 111));
    let text = host.rendered();
    assert_tip_first(&text);
    assert!(has_line(&text, "Issued amount Confidential"), "{text}");
    assert!(has_line(&text, "Burned amount Confidential"), "{text}");
    assert!(has_line(&text, "Circulating amount Confidential"), "{text}");
    assert!(!text.contains("424242"), "{text}");
    assert!(!text.contains("111"), "{text}");
    assert!(!text.contains("0.00100000"), "{text}");
    assert_plain_paths(&rec.gets());
}

/// A pegged asset document has no name. Pegged in, pegged out, burned, and
/// circulating come from `chain_stats`.
#[test]
fn pegged_asset_without_a_name_paints_pegged_and_circulating_amounts() {
    let (host, rec) = open_asset(NATIVE_ASSET, &pegged_stats());
    let text = host.rendered();
    assert_tip_first(&text);
    assert!(
        has_line(&text, &format!("asset id {NATIVE_ASSET}")),
        "{text}"
    );
    assert!(!has_line(&text, "asset name"), "{text}");
    assert!(
        !text.lines().any(|line| line.starts_with("ticker ")),
        "{text}"
    );
    assert!(has_line(&text, "Pegged in 1000"), "{text}");
    assert!(has_line(&text, "Pegged out 200"), "{text}");
    assert!(has_line(&text, "Burned amount 50"), "{text}");
    assert!(has_line(&text, "Circulating amount 750"), "{text}");
    assert!(!has_line(&text, "Issued amount 1000"), "{text}");
    assert!(!text.contains("Issuance TX"), "{text}");
    assert!(!text.contains("0.00100000"), "{text}");
    assert!(!text.contains("LBTC"), "{text}");
    assert!(
        has_line(&text, "Peg In/Out and Burn Transactions"),
        "{text}"
    );
    let paths = rec.gets();
    assert!(
        paths
            .iter()
            .any(|path| path == &format!("/asset/{NATIVE_ASSET}")),
        "{paths:?}"
    );
    assert_plain_paths(&paths);
}

/// A name and a ticker, with no chain stats, stay the open screen. No amount
/// is invented.
#[test]
fn name_and_ticker_only_does_not_invent_an_amount() {
    let id = "34".repeat(32);
    let document = format!(r#"{{"asset_id":"{id}","name":"Liquid AUD","ticker":"L-AUD"}}"#);
    let (host, rec) = open_asset(&id, &document);
    let text = host.rendered();
    assert_tip_first(&text);
    assert!(has_line(&text, &format!("asset id {id}")), "{text}");
    assert!(text.contains("Liquid AUD"), "{text}");
    assert!(text.contains("L-AUD"), "{text}");
    assert!(!text.contains("amount"), "{text}");
    assert!(!text.contains("Pegged"), "{text}");
    assert!(!text.contains("Precision"), "{text}");
    assert!(!text.contains("Issuer"), "{text}");
    assert!(!text.contains("Confidential"), "{text}");
    assert!(!text.contains("0.00100000"), "{text}");
    assert_plain_paths(&rec.gets());
}
