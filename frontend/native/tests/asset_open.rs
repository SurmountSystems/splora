//! Liquid asset open from the suggestion window.
//!
//! Angular `assets / shows a specific asset ID` visits `/assets`, types
//! `Liquid AUD`, and clicks the first button in `ngb-typeahead-window`
//! (`ref/mempool/frontend/cypress/e2e/liquid/liquid.spec.ts`). That click
//! opens `/assets/asset/{id}`. The indexer matches are `GET /assets/registry/search`
//! and `GET /asset/:id` in `src/rest.rs`. Search stays the plain path: no `?`,
//! no `#`, and no `?q=`. The opened screen shows that asset id. It does not
//! invent a price or an amount. One suggestion window, and the tip line stays
//! in front of it.

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

fn liquid_bodies(search: &str, aud: &str) -> BTreeMap<String, String> {
    let mut bodies = BTreeMap::from([
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
    ]);
    bodies.insert(
        format!("/asset/{aud}"),
        format!(r#"{{"asset_id":"{aud}","name":"Liquid AUD","ticker":"L-AUD"}}"#),
    );
    bodies
}

/// Liquid AUD is the first hit, which is the first typeahead button after
/// typing that name. Tether USD is the second button and must stay closed.
fn search_hits(aud: &str, tether: &str) -> String {
    format!(
        r#"[{{"asset_id":"{aud}","name":"Liquid AUD","ticker":"L-AUD"}},{{"asset_id":"{tether}","name":"Tether USD","ticker":"USDt"}}]"#
    )
}

fn type_query(host: &mut ScreenHost<Rec>, text: &str) {
    for ch in text.chars() {
        host.push_str(&ch.to_string());
    }
}

fn suggestion_windows(text: &str) -> usize {
    text.lines()
        .filter(|line| *line == "suggestion window")
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

fn assert_tip_first(text: &str) {
    let first = text.lines().next().unwrap_or("");
    assert_eq!(first, "tip height: 1 hash: liqtip", "{text}");
}

/// Typing `Liquid AUD` opens one suggestion window. The first button opens
/// that asset through `GET /asset/:id` and the screen shows that asset id.
#[test]
fn shows_a_specific_asset_id() {
    let aud = "34".repeat(32);
    let tether = "12".repeat(32);
    assert_eq!(aud.len(), 64);
    assert_eq!(tether.len(), 64);
    let (mut host, rec) = host_with(liquid_bodies(&search_hits(&aud, &tether), &aud));
    host.activate(Screen::Dashboard).expect("assets");
    type_query(&mut host, "Liquid AUD");
    assert_eq!(host.query_text(), "Liquid AUD");
    host.reload().expect("search");

    let paths = rec.gets();
    assert!(
        paths.iter().any(|path| path == SEARCH_PATH),
        "liquid search was not requested: {paths:?}"
    );
    assert!(
        paths
            .iter()
            .all(|path| path != "/assets/registry/search?q=Liquid%20AUD"),
        "{paths:?}"
    );
    assert_plain_paths(&paths);

    let text = host.rendered();
    assert_tip_first(&text);
    assert_eq!(suggestion_windows(&text), 1, "{text}");
    assert!(text.contains("Liquid AUD"), "{text}");
    assert!(text.contains("L-AUD"), "{text}");
    assert_no_price_or_amount(&text);
    assert_eq!(
        host.painted_rows()
            .iter()
            .filter(|row| row.contains("window"))
            .count(),
        0,
        "{:?}",
        host.painted_rows()
    );

    let aud_row = format!("row asset {aud}");
    let tether_row = format!("row asset {tether}");
    let buttons: Vec<String> = host
        .painted_rows()
        .into_iter()
        .filter(|row| row.starts_with("row asset "))
        .collect();
    assert_eq!(
        buttons.first().map(String::as_str),
        Some(aud_row.as_str()),
        "the first suggestion button is not Liquid AUD: {buttons:?}"
    );
    assert_eq!(
        buttons.get(1).map(String::as_str),
        Some(tether_row.as_str())
    );

    host.open_row(&aud_row).expect("open Liquid AUD");
    let paths = rec.gets();
    let asset_path = format!("/asset/{aud}");
    assert!(
        paths.iter().any(|path| path == &asset_path),
        "opening the suggestion did not request {asset_path}: {paths:?}"
    );
    assert!(
        paths.iter().all(|path| path != &format!("/asset/{tether}")),
        "the second suggestion was opened: {paths:?}"
    );
    assert_plain_paths(&paths);

    let opened = host.rendered();
    assert_tip_first(&opened);
    assert_eq!(suggestion_windows(&opened), 0, "{opened}");
    assert!(
        opened.lines().any(|line| line == format!("asset id {aud}")),
        "the asset screen does not show the Liquid AUD id: {opened}"
    );
    assert!(opened.contains("Liquid AUD"), "{opened}");
    assert!(opened.contains("L-AUD"), "{opened}");
    assert!(!opened.contains(&tether), "{opened}");
    assert_no_price_or_amount(&opened);
}
