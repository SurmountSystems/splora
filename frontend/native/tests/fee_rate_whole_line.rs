//! Whole-line fee rate on the transaction screen from `GET /tx/:txid`.
//!
//! Fee 584 and weight 572 are the served body. A finite rate paints the bare
//! line `Fee rate`. The text `sat/vB` stays off. A longer line that contains
//! those words does not count as that bare line.

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

/// Fee 584 and weight 572 paint the bare line `Fee rate` once.
/// `sat/vB` stays off. The rate digits are not an equality gate.
#[test]
fn transaction_screen_paints_bare_fee_rate_line() {
    const LINE: &str = "Fee rate";
    let text = paint_tx_fee_rate("rate408", 584, r#""weight":572"#);
    assert!(
        text.lines().any(|line| line == LINE),
        "no line equals \"Fee rate\": {text}"
    );
    assert_eq!(
        text.lines().filter(|line| *line == LINE).count(),
        1,
        "Fee rate was not painted once: {text}"
    );
    assert!(
        !text.contains("sat/vB"),
        "the fee rate line painted sat/vB: {text}"
    );
}

/// Fee 1 and integer weight/4 == 0 omit the bare line `Fee rate`.
///
/// Weights 1, 2, and 3 take that path. The shipped transaction screen
/// omits that whole line. `sat/vB` stays off. A longer sentence that merely
/// contains those words is not this line.
#[test]
fn transaction_screen_omits_fee_rate_line_when_weight_under_four() {
    const LINE: &str = "Fee rate";
    for (txid, weight) in [("ratew1", 1u32), ("ratew2", 2), ("ratew3", 3)] {
        let text = paint_tx_fee_rate(txid, 1, &format!(r#""weight":{weight}"#));
        assert!(
            text.contains(&format!("tx txid {txid}")),
            "weight {weight} hid the transaction: {text}"
        );
        assert!(
            !text
                .lines()
                .any(|line| line == LINE || line.starts_with("Fee rate ")),
            "fee 1 and weight {weight} painted Fee rate: {text}"
        );
        assert!(
            !text.contains("sat/vB"),
            "fee 1 and weight {weight} painted sat/vB: {text}"
        );
    }
}
