//! GPUI package result paints `other-wtxid` when that witness id is present.
//!
//! A missing key and an empty string are not a witness id. The txid is not
//! copied into that spot. The first token after `txid ` stays the txid.

use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use splora_api::{Network, api_root};
use splora_native::{
    ClientError, ClientRequest, ExplorerClient, ExplorerGet, ExplorerPost, NpubPopupModel, Screen,
    ScreenHost, painted_shell_text,
};

/// 32-byte witness id. Not the entry's `txid` and not the `tx-results` map key.
const OTHER_WTXID: &str = "cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc";
const TXID: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
/// A witness id this package never sends. The paint must not invent it.
const MADE_UP_WTXID: &str = "dddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddd";

#[derive(Clone)]
struct Rec {
    network: Network,
    gets: Rc<RefCell<Vec<String>>>,
    posts: Rc<RefCell<Vec<(String, String)>>>,
    bodies: Rc<BTreeMap<String, String>>,
}

impl Rec {
    fn new(network: Network, bodies: BTreeMap<String, String>) -> Self {
        Self {
            network,
            gets: Rc::new(RefCell::new(Vec::new())),
            posts: Rc::new(RefCell::new(Vec::new())),
            bodies: Rc::new(bodies),
        }
    }

    fn posts(&self) -> Vec<(String, String)> {
        self.posts.borrow().clone()
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
        self.posts
            .borrow_mut()
            .push((request.path.clone(), request.body.clone()));
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

fn painted(host: &ScreenHost<Rec>) -> String {
    let popup = NpubPopupModel::new("npub1restofindexer").expect("npub");
    painted_shell_text(host, &popup)
}

fn package_body(txid: &str, other_wtxid: Option<&str>) -> String {
    let fields = match other_wtxid {
        Some(wtxid) => format!(r#""txid":"{txid}","other-wtxid":"{wtxid}""#),
        None => format!(r#""txid":"{txid}""#),
    };
    format!(r#"{{"package_msg":"success","tx-results":{{"pkg-key":{{{fields}}}}}}}"#)
}

fn submit_package(body: &str) -> (ScreenHost<Rec>, Rec, String) {
    let (mut host, rec) = host_with(
        Network::Mainnet,
        BTreeMap::from([("/txs/package".to_string(), body.to_string())]),
    );
    host.activate(Screen::Broadcast).expect("broadcast");
    host.set_query("abcd ef01");
    host.submit().expect("package");
    let text = painted(&host);
    (host, rec, text)
}

fn open_txid_row(host: &mut ScreenHost<Rec>) {
    let opened = host
        .painted_rows()
        .into_iter()
        .find(|row| row.contains(TXID))
        .expect("txid row");
    host.open_row(&opened).expect("open txid");
    assert_eq!(host.screen, Screen::Tx);
    assert_eq!(host.tx.txid, TXID);
}

#[test]
fn package_result_paints_a_present_witness_id_and_does_not_invent_one() {
    assert_ne!(OTHER_WTXID, TXID);
    assert_ne!(MADE_UP_WTXID, TXID);
    assert_ne!(MADE_UP_WTXID, OTHER_WTXID);

    let (mut host, rec, present) = submit_package(&package_body(TXID, Some(OTHER_WTXID)));
    assert!(
        present.contains(OTHER_WTXID),
        "package result dropped the witness id: {present}"
    );
    assert!(present.contains(TXID), "{present}");
    let path = rec.posts()[0].0.clone();
    assert_eq!(path, "/txs/package");
    assert!(!path.contains('?'), "{path}");
    assert!(!path.contains('#'), "{path}");
    assert!(!present.contains("0.00100000"), "{present}");
    assert!(!present.contains("LBTC"), "{present}");
    open_txid_row(&mut host);

    let (mut missing_host, _, missing) = submit_package(&package_body(TXID, None));
    assert!(
        !missing.contains(OTHER_WTXID),
        "a missing other-wtxid invented a witness id: {missing}"
    );
    assert!(
        !missing.contains(MADE_UP_WTXID),
        "a missing other-wtxid invented a witness id: {missing}"
    );
    assert_eq!(
        missing.matches(TXID).count(),
        present.matches(TXID).count(),
        "a missing other-wtxid copied the txid into the witness slot: {missing}"
    );
    open_txid_row(&mut missing_host);

    let (_, _, empty) = submit_package(&package_body(TXID, Some("")));
    assert_eq!(empty, missing, "an empty other-wtxid is not a witness id");
    assert!(
        !empty.contains(OTHER_WTXID) && !empty.contains(MADE_UP_WTXID),
        "{empty}"
    );
    assert_eq!(empty.matches(TXID).count(), missing.matches(TXID).count());
}
