//! Whole-line fee rate on the transaction screen from `GET /tx/:txid`.
//!
//! Fee 584 and weight 572 are the served body. The rate is fee divided by
//! weight divided by 4. Under 10 that rate keeps two fraction digits.
//! The screen must include one line equal to `Fee rate 4.08 sat/vB`.
//! A longer line that only contains those words does not count.

use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use splora_api::{Network, api_root};
use splora_native::{
    ClientError, ClientRequest, ExplorerClient, ExplorerGet, ExplorerPost, NpubPopupModel,
    ScreenHost, painted_shell_text,
};

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

fn painted(host: &ScreenHost<Rec>) -> String {
    let popup = NpubPopupModel::new("npub1restofindexer").expect("npub");
    painted_shell_text(host, &popup)
}

fn paint_tx_fee_rate(txid: &str, fee: u64, weight_field: &str) -> String {
    let weight_line = if weight_field.is_empty() {
        String::new()
    } else {
        format!("{weight_field},")
    };
    let tx = format!(
        r#"{{
        "txid":"{txid}",
        "version":1,
        "locktime":0,
        "vin":[{{
            "txid":"prev",
            "vout":0,
            "prevout":null,
            "scriptsig":"deadbeef",
            "scriptsig_asm":"OP_CHECKSIG",
            "witness":["abcdwitnesshex"],
            "is_coinbase":false,
            "sequence":1
        }}],
        "vout":[{{
            "scriptpubkey":"0014aa",
            "scriptpubkey_asm":"OP_RETURN",
            "scriptpubkey_type":"op_return",
            "value":9
        }}],
        "size":10,
        {weight_line}
        "sigops":1,
        "fee":{fee},
        "status":{{"confirmed":false}}
    }}"#
    );
    let (mut host, rec) = host_with(
        Network::Mainnet,
        BTreeMap::from([(format!("/tx/{txid}"), tx)]),
    );
    host.open_transaction(txid)
        .unwrap_or_else(|err| panic!("tx {txid} did not open: {err}"));
    let paths = rec.gets();
    assert!(
        paths.iter().any(|path| path == &format!("/tx/{txid}")),
        "GET /tx/{txid} was not requested: {paths:?}"
    );
    assert!(
        paths
            .iter()
            .all(|path| !path.contains('?') && !path.contains('#')),
        "a transaction open used a query or a fragment: {paths:?}"
    );
    painted(&host)
}

/// Fee 584 and weight 572 paint one line equal to `Fee rate 4.08 sat/vB`.
#[test]
fn transaction_screen_paints_fee_rate_4_08_whole_line() {
    const LINE: &str = "Fee rate 4.08 sat/vB";
    let text = paint_tx_fee_rate("rate408", 584, r#""weight":572"#);
    assert!(
        text.lines().any(|line| line == LINE),
        "fee 584 and weight 572 did not paint {LINE}: {text}"
    );
    assert_eq!(
        text.lines().filter(|line| *line == LINE).count(),
        1,
        "{LINE} was not one line: {text}"
    );
}
