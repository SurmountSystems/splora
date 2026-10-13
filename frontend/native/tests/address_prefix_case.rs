//! Mixed-case mainnet bech32 prefixes use the lowercase address-prefix route.
//!
//! `GET /address-prefix/:prefix` stores bech32 in lowercase. A search whose
//! ASCII lowercase form starts with `bc1` requests that lowercase path and
//! paints each returned address as `row prefix`. Base58 and other prefixes
//! keep the typed case. The paint does not say `Invalid Bitcoin address`.

use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use splora_api::{Network, api_root};
use splora_native::{
    ClientError, ClientRequest, ExplorerClient, ExplorerGet, ExplorerPost, Screen, ScreenHost,
};

const FOLDED: &str = "/address-prefix/bc1pqyqs";
const MATCH_A: &str = "bc1pqyqsmatchone000000000000000000";
const MATCH_B: &str = "bc1pqyqsmatchtwo000000000000000000";

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

fn prefix_body(addresses: &[&str]) -> String {
    let quoted: Vec<String> = addresses
        .iter()
        .map(|address| format!("\"{address}\""))
        .collect();
    format!("[{}]", quoted.join(","))
}

fn prefix_gets(rec: &Rec) -> Vec<String> {
    rec.gets()
        .into_iter()
        .filter(|path| path.starts_with("/address-prefix/"))
        .collect()
}

fn assert_plain_paths(paths: &[String]) {
    for path in paths {
        assert!(!path.contains('?'), "{path}");
        assert!(!path.contains('#'), "{path}");
    }
}

fn search(host: &mut ScreenHost<Rec>, query: &str) -> Result<(), ClientError> {
    host.activate(Screen::Dashboard)?;
    host.set_query(query);
    host.reload()
}

/// `BC1PQYQS` requests `/address-prefix/bc1pqyqs` and paints each match.
#[test]
fn mixed_case_bc1_prefix_requests_the_lowercase_path() {
    let (mut host, rec) = host_with(BTreeMap::from([(
        FOLDED.to_string(),
        prefix_body(&[MATCH_A, MATCH_B]),
    )]));
    search(&mut host, "BC1PQYQS").expect("lowercase bc1 prefix");
    let paths = prefix_gets(&rec);
    assert_eq!(paths, vec![FOLDED.to_string()], "{paths:?}");
    assert_plain_paths(&rec.gets());
    assert_eq!(
        host.painted_rows(),
        vec![
            format!("row prefix {MATCH_A}"),
            format!("row prefix {MATCH_B}"),
        ]
    );
    let text = host.rendered();
    assert!(text.contains(MATCH_A), "{text}");
    assert!(text.contains(MATCH_B), "{text}");
    assert!(!text.contains("Invalid Bitcoin address"), "{text}");
}

/// Interior capitals in a `bc1` prefix use that same lowercase path.
#[test]
fn interior_capitals_in_bc1_prefix_use_the_same_lowercase_path() {
    let (mut host, rec) = host_with(BTreeMap::from([(
        FOLDED.to_string(),
        prefix_body(&[MATCH_A]),
    )]));
    search(&mut host, "bc1PqYqS").expect("folded bc1 prefix");
    assert_eq!(prefix_gets(&rec), vec![FOLDED.to_string()]);
    assert_plain_paths(&rec.gets());
    assert_eq!(host.painted_rows(), vec![format!("row prefix {MATCH_A}")]);
    assert!(!host.rendered().contains("Invalid Bitcoin address"));
}

/// Base58 keeps the characters the operator typed, including `1wiz`.
#[test]
fn base58_prefix_keeps_the_typed_case() {
    let wiz = "1wizSAYSbuyXbt9d8JV8ytm5acqq2TorC";
    let mixed = "1WizSAYSbuyXbt9d8JV8ytm5acqq2TorC";
    let (mut host, rec) = host_with(BTreeMap::from([
        ("/address-prefix/1wiz".to_string(), prefix_body(&[wiz])),
        ("/address-prefix/1Wiz".to_string(), prefix_body(&[mixed])),
    ]));
    search(&mut host, "1wiz").expect("base58 prefix");
    assert_eq!(prefix_gets(&rec), vec!["/address-prefix/1wiz".to_string()]);
    assert_eq!(host.painted_rows(), vec![format!("row prefix {wiz}")]);
    assert!(!host.rendered().contains("Invalid Bitcoin address"));

    search(&mut host, "1Wiz").expect("mixed base58 prefix");
    assert_eq!(
        prefix_gets(&rec),
        vec![
            "/address-prefix/1wiz".to_string(),
            "/address-prefix/1Wiz".to_string(),
        ]
    );
    assert_plain_paths(&rec.gets());
    assert_eq!(host.painted_rows(), vec![format!("row prefix {mixed}")]);
}
