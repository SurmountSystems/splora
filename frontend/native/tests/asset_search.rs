//! Liquid asset search.
//!
//! Angular `assets / allows searching assets` visits `/assets`, clicks the
//! search input, types `Tether USD`, and expects one `ngb-typeahead-window`.
//! The indexer match in `src/rest.rs` is `GET /assets/registry/search`. An
//! empty `q` returns `[]`. This client does not send `q`, because `?` and `#`
//! are rejected. One suggestion window. No second window, no price, and no
//! amount.

use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use splora_api::{Network, api_root};
use splora_native::{
    ClientError, ClientRequest, ExplorerClient, ExplorerGet, ExplorerPost, Screen, ScreenHost,
};

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

fn host_with(bodies: BTreeMap<String, String>) -> (ScreenHost<Rec>, Rec) {
    let rec = Rec::new(Network::Liquid, bodies);
    let host = ScreenHost::new(ExplorerClient::new(Network::Liquid, rec.clone()));
    (host, rec)
}

fn liquid_bodies(search: &str) -> BTreeMap<String, String> {
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
        (SEARCH_PATH.to_string(), search.to_string()),
    ])
}

fn search_hits() -> String {
    let tether = "12".repeat(32);
    let aud = "34".repeat(32);
    format!(
        r#"[{{"asset_id":"{tether}","name":"Tether USD","ticker":"USDt"}},{{"asset_id":"{aud}","name":"Liquid AUD","ticker":"L-AUD"}}]"#
    )
}

fn type_query(host: &mut ScreenHost<Rec>, text: &str) {
    for ch in text.chars() {
        host.push_str(&ch.to_string());
    }
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

fn assert_no_price_or_amount(text: &str) {
    assert!(!text.contains("price"), "{text}");
    assert!(!text.contains("amount"), "{text}");
    assert!(!text.contains("Fiat price last updated"), "{text}");
}

/// Typing `Tether USD` on the assets screen opens one suggestion window.
/// Two registry hits still paint one window. The request is
/// `GET /assets/registry/search` with no query string.
#[test]
fn typing_tether_usd_opens_one_suggestion_window() {
    let (mut host, rec) = host_with(liquid_bodies(&search_hits()));
    host.activate(Screen::Dashboard).expect("assets");
    assert!(
        rec.gets().iter().any(|path| path == "/assets/registry"),
        "assets screen did not request the registry: {:?}",
        rec.gets()
    );

    type_query(&mut host, "Tether USD");
    assert_eq!(host.query_text(), "Tether USD");
    host.reload().expect("search");

    let paths = rec.gets();
    assert!(
        paths.iter().any(|path| path == SEARCH_PATH),
        "liquid search was not requested: {paths:?}"
    );
    assert!(
        paths
            .iter()
            .all(|path| path != "/assets/registry/search?q=Tether%20USD"),
        "{paths:?}"
    );
    assert_plain_paths(&paths);

    let text = host.rendered();
    assert_eq!(suggestion_windows(&text), 1, "{text}");
    assert!(text.contains("Tether USD"), "{text}");
    assert!(text.contains("Liquid AUD"), "{text}");
    assert_eq!(
        host.painted_rows()
            .iter()
            .filter(|row| row.contains("suggestion window") || row.contains("window"))
            .count(),
        0,
        "{:?}",
        host.painted_rows()
    );
    assert_no_price_or_amount(&text);
}

/// An empty search body paints no suggestion window. The path is still
/// `GET /assets/registry/search` with no query string.
#[test]
fn empty_registry_search_paints_no_suggestion_window() {
    let (mut host, rec) = host_with(liquid_bodies("[]"));
    host.activate(Screen::Dashboard).expect("assets");
    type_query(&mut host, "Tether USD");
    host.reload().expect("empty search");
    let paths = rec.gets();
    assert!(paths.iter().any(|path| path == SEARCH_PATH), "{paths:?}");
    assert_plain_paths(&paths);
    let text = host.rendered();
    assert_eq!(suggestion_windows(&text), 0, "{text}");
    assert_no_price_or_amount(&text);
}

/// `?` and `#` never become the search path.
#[test]
fn asset_search_path_rejects_query_and_fragment() {
    let rec = Rec::new(Network::Liquid, liquid_bodies("[]"));
    let client = ExplorerClient::new(Network::Liquid, rec.clone());
    assert_eq!(
        client
            .get_path("/assets/registry/search?q=Tether%20USD")
            .unwrap_err(),
        ClientError::BadPath
    );
    assert_eq!(
        client
            .get_path("/assets/registry/search#Tether")
            .unwrap_err(),
        ClientError::BadPath
    );
    assert!(
        rec.gets()
            .iter()
            .all(|path| !path.contains('?') && !path.contains('#')),
        "{:?}",
        rec.gets()
    );
    client.get_path(SEARCH_PATH).expect("plain search path");
    assert!(rec.gets().iter().any(|path| path == SEARCH_PATH));
}

/// Mainnet does not open the liquid suggestion window.
#[test]
fn mainnet_does_not_open_a_suggestion_window() {
    let rec = Rec::new(Network::Mainnet, liquid_bodies(&search_hits()));
    let mut host = ScreenHost::new(ExplorerClient::new(Network::Mainnet, rec.clone()));
    host.activate(Screen::Dashboard).expect("mainnet");
    type_query(&mut host, "Tether USD");
    let _ = host.reload();
    let paths = rec.gets();
    assert!(paths.iter().all(|path| path != SEARCH_PATH), "{paths:?}");
    assert_plain_paths(&paths);
    assert_eq!(suggestion_windows(&host.rendered()), 0);
}
