//! Mainnet RBF replacements page.
//!
//! Cypress `ref/mempool/frontend/cypress/e2e/mainnet/mainnet.spec.ts`
//! describe `RBF transactions`, it `RBF page gets updated over websockets`,
//! intercepts `/api/v1/replacements` and `/api/v1/fullrbf/replacements` as
//! empty arrays, then visits `/rbf`. It expects `.no-replacements` and
//! `.tree` length 0. After socket fixture `rbf_page/rbf_01.json` the tree
//! length is 1. After `rbf_page/rbf_02.json` the tree length is 2.
//! The empty state sentence is the text inside `.no-replacements`.
//! The tree length is the line `tree count N`.

use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use splora_api::{Network, api_root};
use splora_native::{
    ClientError, ClientRequest, ExplorerClient, ExplorerGet, ExplorerPost, Screen, ScreenHost,
};

const NO_REPLACEMENTS: &str = "There are no replacements in the mempool yet!";
const RBF_01: &str =
    include_str!("../../../ref/mempool/frontend/cypress/fixtures/rbf_page/rbf_01.json");
const RBF_02: &str =
    include_str!("../../../ref/mempool/frontend/cypress/fixtures/rbf_page/rbf_02.json");

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
    let rec = Rec::new(Network::Mainnet, bodies);
    let host = ScreenHost::new(ExplorerClient::new(Network::Mainnet, rec.clone()));
    (host, rec)
}

fn empty_replacement_bodies() -> BTreeMap<String, String> {
    BTreeMap::from([
        ("/api/v1/replacements".to_string(), "[]".to_string()),
        ("/api/v1/fullrbf/replacements".to_string(), "[]".to_string()),
        ("/v1/replacements".to_string(), "[]".to_string()),
        ("/v1/fullrbf/replacements".to_string(), "[]".to_string()),
    ])
}

fn canonical_replacement(path: &str) -> Option<&'static str> {
    if path.ends_with("/fullrbf/replacements") {
        Some("/api/v1/fullrbf/replacements")
    } else if path.ends_with("/replacements") {
        Some("/api/v1/replacements")
    } else {
        None
    }
}

fn replacement_gets(rec: &Rec) -> String {
    let mut paths: Vec<&str> = rec
        .gets()
        .iter()
        .filter_map(|path| canonical_replacement(path))
        .collect();
    paths.sort();
    paths.dedup();
    paths.join(",")
}

fn tree_count_line(text: &str) -> &str {
    text.lines()
        .find(|line| line.starts_with("tree count "))
        .map(|line| line.trim_start_matches("tree count "))
        .unwrap_or("missing")
}

fn open_rbf(host: &mut ScreenHost<Rec>) {
    if let Some(screen) = Screen::from_link("rbf") {
        let opened = host.activate(screen);
        assert!(opened.is_ok(), "open /rbf: {opened:?}");
    }
}

/// Cypress visits `/rbf`, then checks `.no-replacements` and `.tree` length.
fn initial_state(host: &ScreenHost<Rec>, rec: &Rec) -> String {
    let text = host.rendered();
    let no_replacements = text.lines().any(|line| line == NO_REPLACEMENTS);
    format!(
        "screen={} no_replacements={no_replacements} tree_count={} gets={}",
        host.screen.label(),
        tree_count_line(&text),
        replacement_gets(rec)
    )
}

fn tree_state(host: &ScreenHost<Rec>) -> String {
    format!(
        "screen={} tree_count={}",
        host.screen.label(),
        tree_count_line(&host.rendered())
    )
}

fn deliver_socket(host: &mut ScreenHost<Rec>, fixture: &str) {
    host.set_query(fixture);
    let refreshed = host.reload();
    assert!(
        refreshed.is_ok(),
        "reload after the rbf socket fixture: {refreshed:?}"
    );
}

#[test]
fn rbf_page_gets_updated_over_websockets() {
    let (mut host, rec) = host_with(empty_replacement_bodies());
    open_rbf(&mut host);
    assert_eq!(
        initial_state(&host, &rec),
        "screen=rbf no_replacements=true tree_count=0 gets=/api/v1/fullrbf/replacements,/api/v1/replacements"
    );

    deliver_socket(&mut host, RBF_01);
    assert_eq!(tree_state(&host), "screen=rbf tree_count=1");

    deliver_socket(&mut host, RBF_02);
    assert_eq!(tree_state(&host), "screen=rbf tree_count=2");
}
