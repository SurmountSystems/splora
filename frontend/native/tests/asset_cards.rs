//! Liquid assets screen.
//!
//! Angular `assets / shows the assets screen` visits `/assets` and expects
//! `.featuredBox .card` to have a length of at least five
//! (`ref/mempool/frontend/cypress/e2e/liquid/liquid.spec.ts`). The indexer
//! match in `src/rest.rs` is `GET /assets/registry`. This client does not
//! call `/assets/featured` and does not send `?` or `#`. One featured card
//! is one named issued row. A pegged row has no name, so it is not a card.
//! The card text is the name and the ticker. It is not a price, a hashrate,
//! an amount, or a block graphic. The suggestion window stays one, and the
//! tip line stays in front of that window.

use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use splora_api::{Network, api_root};
use splora_native::{
    ClientError, ClientRequest, ExplorerClient, ExplorerGet, ExplorerPost, Screen, ScreenHost,
};

const REGISTRY_PATH: &str = "/assets/registry";
const SEARCH_PATH: &str = "/assets/registry/search";

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

fn host_with(network: Network, bodies: BTreeMap<String, String>) -> (ScreenHost<Rec>, Rec) {
    let rec = Rec::new(network, bodies);
    let host = ScreenHost::new(ExplorerClient::new(network, rec.clone()));
    (host, rec)
}

fn home_bodies(registry: &str, search: &str) -> BTreeMap<String, String> {
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
        (REGISTRY_PATH.to_string(), registry.to_string()),
        (SEARCH_PATH.to_string(), search.to_string()),
    ])
}

fn issued(id: &str, name: &str, ticker: Option<&str>) -> String {
    let ticker_field = match ticker {
        Some(ticker) => format!(r#","ticker":"{ticker}""#),
        None => String::new(),
    };
    format!(
        r#"{{"asset_id":"{id}","issuance_txin":{{"txid":"issuance-tx","vin":0}},"issuance_prevout":{{"txid":"issuance-prev","vout":1}},"reissuance_token":"reissue-token","status":{{"confirmed":true}},"chain_stats":{{"tx_count":1,"issuance_count":1,"issued_amount":424242,"burned_amount":0,"has_blinded_issuances":false,"burned_reissuance_tokens":0}},"mempool_stats":{{"tx_count":0,"issuance_count":0,"issued_amount":0,"burned_amount":0,"has_blinded_issuances":false,"burned_reissuance_tokens":0}},"name":"{name}"{ticker_field}}}"#
    )
}

fn native_row(id: &str) -> String {
    format!(
        r#"{{"asset_id":"{id}","chain_stats":{{"tx_count":1,"peg_in_count":1,"peg_in_amount":424242,"peg_out_count":0,"peg_out_amount":0,"burn_count":0,"burned_amount":0}},"mempool_stats":{{"tx_count":0,"peg_in_count":0,"peg_in_amount":0,"peg_out_count":0,"peg_out_amount":0,"burn_count":0,"burned_amount":0}}}}"#
    )
}

fn featured_names() -> [(&'static str, &'static str); 5] {
    [
        ("Tether USD", "USDt"),
        ("Liquid AUD", "L-AUD"),
        ("Liquid CAD", "L-CAD"),
        ("Liquid EUR", "EUR"),
        ("Liquid JPY", "JPY"),
    ]
}

fn registry_of(rows: &[String]) -> String {
    format!("[{}]", rows.join(","))
}

fn five_registry() -> String {
    let rows: Vec<String> = featured_names()
        .iter()
        .enumerate()
        .map(|(index, (name, ticker))| {
            let id = format!("{index:02x}").repeat(32);
            issued(&id, name, Some(ticker))
        })
        .collect();
    registry_of(&rows)
}

fn search_hits() -> String {
    let tether = "00".repeat(32);
    let aud = "01".repeat(32);
    format!(
        r#"[{{"asset_id":"{tether}","name":"Tether USD","ticker":"USDt"}},{{"asset_id":"{aud}","name":"Liquid AUD","ticker":"L-AUD"}}]"#
    )
}

fn featured_cards(text: &str) -> Vec<&str> {
    text.lines()
        .filter(|line| line.starts_with("featured card "))
        .collect()
}

fn suggestion_windows(text: &str) -> usize {
    text.lines()
        .filter(|line| line.contains("suggestion window"))
        .count()
}

fn assert_plain_paths(paths: &[String]) {
    for path in paths {
        assert!(!path.contains('?'), "{path}");
        assert!(!path.contains('#'), "{path}");
    }
}

fn assert_no_invented_figures(text: &str) {
    assert!(!text.contains("424242"), "{text}");
    assert!(!text.contains("price"), "{text}");
    assert!(!text.contains("hashrate"), "{text}");
    assert!(!text.contains("amount"), "{text}");
    assert!(!text.contains("Fiat price last updated"), "{text}");
    assert!(!text.contains("mempool-block"), "{text}");
}

fn assert_tip_first(text: &str) {
    let first = text.lines().next().unwrap_or("");
    assert_eq!(first, "tip height: 1 hash: liqtip", "{text}");
}

/// Five named issued rows from `GET /assets/registry` paint five featured cards.
#[test]
fn assets_screen_shows_five_featured_cards() {
    let (mut host, rec) = host_with(Network::Liquid, home_bodies(&five_registry(), "[]"));
    host.activate(Screen::Dashboard).expect("assets");
    let paths = rec.gets();
    assert!(
        paths.iter().any(|path| path == REGISTRY_PATH),
        "assets screen did not request the registry: {paths:?}"
    );
    assert!(
        paths.iter().all(|path| path != "/assets/featured"),
        "{paths:?}"
    );
    assert_plain_paths(&paths);

    let text = host.rendered();
    assert_tip_first(&text);
    assert_eq!(suggestion_windows(&text), 0, "{text}");
    assert_eq!(
        featured_cards(&text),
        vec![
            "featured card Tether USD USDt",
            "featured card Liquid AUD L-AUD",
            "featured card Liquid CAD L-CAD",
            "featured card Liquid EUR EUR",
            "featured card Liquid JPY JPY",
        ],
        "{text}"
    );
    assert_no_invented_figures(&text);
    assert!(
        host.painted_rows()
            .iter()
            .all(|row| !row.contains("featured card") && !row.contains("window")),
        "{:?}",
        host.painted_rows()
    );
}

/// Four named rows stay four cards. The screen does not pad the list to five.
#[test]
fn four_named_assets_stay_four_cards() {
    let rows: Vec<String> = featured_names()
        .iter()
        .take(4)
        .enumerate()
        .map(|(index, (name, ticker))| {
            let id = format!("{index:02x}").repeat(32);
            issued(&id, name, Some(ticker))
        })
        .collect();
    let (mut host, rec) = host_with(Network::Liquid, home_bodies(&registry_of(&rows), "[]"));
    host.activate(Screen::Dashboard).expect("assets");
    assert!(rec.gets().iter().any(|path| path == REGISTRY_PATH));
    assert_plain_paths(&rec.gets());
    let text = host.rendered();
    assert_eq!(featured_cards(&text).len(), 4, "{text}");
    assert_no_invented_figures(&text);
}

/// A name with no ticker is still one card. The ticker is not invented.
#[test]
fn named_asset_without_ticker_is_one_card() {
    let row = issued(&"ab".repeat(32), "Bare Name", None);
    let (mut host, _) = host_with(Network::Liquid, home_bodies(&registry_of(&[row]), "[]"));
    host.activate(Screen::Dashboard).expect("assets");
    let text = host.rendered();
    assert_eq!(
        featured_cards(&text),
        vec!["featured card Bare Name"],
        "{text}"
    );
    assert_no_invented_figures(&text);
}

/// A pegged registry row has no name. It is not a featured card, and its peg
/// figures are not painted.
#[test]
fn pegged_row_is_not_a_featured_card() {
    let id = "cd".repeat(32);
    let (mut host, rec) = host_with(
        Network::Liquid,
        home_bodies(&registry_of(&[native_row(&id)]), "[]"),
    );
    host.activate(Screen::Dashboard).expect("assets");
    assert!(rec.gets().iter().any(|path| path == REGISTRY_PATH));
    let text = host.rendered();
    assert!(featured_cards(&text).is_empty(), "{text}");
    assert!(text.contains(&format!("asset id {id}")), "{text}");
    assert_no_invented_figures(&text);
}

/// An empty registry paints no featured card.
#[test]
fn empty_registry_paints_no_featured_card() {
    let (mut host, rec) = host_with(Network::Liquid, home_bodies("[]", "[]"));
    host.activate(Screen::Dashboard).expect("assets");
    assert!(rec.gets().iter().any(|path| path == REGISTRY_PATH));
    assert_plain_paths(&rec.gets());
    let text = host.rendered();
    assert!(featured_cards(&text).is_empty(), "{text}");
    assert_eq!(suggestion_windows(&text), 0, "{text}");
    assert_no_invented_figures(&text);
}

/// Mainnet does not request the liquid registry and does not paint a card.
#[test]
fn mainnet_does_not_paint_featured_cards() {
    let rec_bodies = home_bodies(&five_registry(), &search_hits());
    let (mut host, rec) = host_with(Network::Mainnet, rec_bodies);
    host.activate(Screen::Dashboard).expect("mainnet");
    let paths = rec.gets();
    assert!(paths.iter().all(|path| path != REGISTRY_PATH), "{paths:?}");
    assert!(paths.iter().all(|path| path != SEARCH_PATH), "{paths:?}");
    assert_plain_paths(&paths);
    let text = host.rendered();
    assert!(featured_cards(&text).is_empty(), "{text}");
    assert_eq!(suggestion_windows(&text), 0, "{text}");
}

/// Cards do not add a second suggestion window. The tip line stays first.
#[test]
fn featured_cards_keep_one_suggestion_window_behind_the_tip() {
    let (mut host, rec) = host_with(
        Network::Liquid,
        home_bodies(&five_registry(), &search_hits()),
    );
    host.activate(Screen::Dashboard).expect("assets");
    assert_eq!(featured_cards(&host.rendered()).len(), 5);

    for ch in "Tether USD".chars() {
        host.push_str(&ch.to_string());
    }
    host.reload().expect("search");

    let paths = rec.gets();
    assert!(paths.iter().any(|path| path == SEARCH_PATH), "{paths:?}");
    assert!(
        paths
            .iter()
            .all(|path| path != "/assets/registry/search?q=Tether%20USD"),
        "{paths:?}"
    );
    assert_plain_paths(&paths);
    let text = host.rendered();
    assert_tip_first(&text);
    assert_eq!(suggestion_windows(&text), 1, "{text}");
    assert_no_invented_figures(&text);
}
