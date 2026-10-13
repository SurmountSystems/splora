//! Block transaction pager. Page size is 25.
//! Five pages paint nine children: those five page numbers, plus first,
//! previous, next, and last. That is the Leptos five-page pager.
//! This screen does not read a viewport width. A Rust assertion of nine
//! children is not a Cypress viewport row.
//! Every other page count keeps the phone window of at most three page
//! numbers. A block with 200 transactions still paints seven children.

use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use splora_api::{Network, api_root};
use splora_native::{
    ClientError, ClientRequest, ExplorerClient, ExplorerGet, ExplorerPost, ScreenHost,
};

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

    fn gets(&self) -> Vec<String> {
        self.gets.borrow().clone()
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
        assert_eq!(request.method, "POST");
        let path = request.path.clone();
        self.posts
            .borrow_mut()
            .push((path.clone(), request.body.clone()));
        self.bodies
            .get(&path)
            .cloned()
            .ok_or_else(|| ClientError::Http(format!("missing {path}")))
    }
}

fn host_with(network: Network, bodies: BTreeMap<String, String>) -> (ScreenHost<Rec>, Rec) {
    let rec = Rec::new(network, bodies);
    let host = ScreenHost::new(ExplorerClient::new(network, rec.clone()));
    (host, rec)
}

fn has_line(text: &str, line: &str) -> bool {
    text.lines().any(|row| row == line)
}

fn assert_line(text: &str, line: &str) {
    assert!(has_line(text, line), "missing {line} in {text}");
}

fn lines_with_prefix<'a>(text: &'a str, prefix: &str) -> Vec<&'a str> {
    text.lines().filter(|row| row.starts_with(prefix)).collect()
}

fn assert_plain_paths(paths: &[String]) {
    for path in paths {
        assert!(!path.contains('?'), "{path}");
        assert!(!path.contains('#'), "{path}");
    }
}

fn assert_requests(rec: &Rec) {
    assert_plain_paths(&rec.gets());
    let posts: Vec<String> = rec.posts().into_iter().map(|(path, _)| path).collect();
    assert_plain_paths(&posts);
}

fn assert_no_banned(text: &str) {
    assert!(!text.contains("Fiat price last updated"), "{text}");
    assert!(!text.contains("vB/s"), "{text}");
    assert!(!text.contains("mempool-block"), "{text}");
    assert!(!text.contains("pagination desktop"), "{text}");
}

fn assert_has_row(rows: &[String], row: &str) {
    assert!(rows.iter().any(|painted| painted == row), "{rows:?}");
}

fn block_json(id: &str, tx_count: u32, previous: &str) -> String {
    format!(
        r#"{{"id":"{id}","height":800000,"version":1,"timestamp":1700000000,"tx_count":{tx_count},"size":20,"weight":80,"merkle_root":"mm","previousblockhash":"{previous}","mediantime":1700000000,"nonce":1,"bits":2,"difficulty":1.0}}"#
    )
}

fn insert_block(bodies: &mut BTreeMap<String, String>, id: &str, tx_count: u32, txs: &str) {
    bodies.insert(format!("/block/{id}"), block_json(id, tx_count, "older"));
    bodies.insert(format!("/block/{id}/txs"), txs.to_string());
    bodies.insert(format!("/block/{id}/txids"), "[]".to_string());
}

fn tx_list(txid: &str, size: u32, weight: u32, fee: u64) -> String {
    format!(
        r#"[{{"txid":"{txid}","version":1,"locktime":0,"vin":[],"vout":[],"size":{size},"weight":{weight},"sigops":0,"fee":{fee},"status":{{"confirmed":true}}}}]"#
    )
}

const PHONE_PAGE_ONE: &[&str] = &[
    "pagination control first",
    "pagination control previous",
    "pagination control page 1 start 0 active",
    "pagination control page 2 start 25",
    "pagination control page 3 start 50",
    "pagination control next",
    "pagination control last",
];

#[test]
fn phone_layout_paints_seven_pager_children() {
    let mut bodies = BTreeMap::new();
    insert_block(&mut bodies, "wide", 200, "[]");
    let (mut host, rec) = host_with(Network::Mainnet, bodies);
    host.open_block("wide").expect("wide");
    let text = host.rendered();
    assert_line(&text, "200 transactions");
    assert_line(&text, "pagination children 7");
    let controls = lines_with_prefix(&text, "pagination control ");
    assert_eq!(controls, PHONE_PAGE_ONE, "{text}");
    assert!(!text.contains("pagination control page 4"), "{text}");
    assert_has_row(&host.painted_rows(), "row page 25");
    assert_requests(&rec);
    assert_no_banned(&text);
}

#[test]
fn next_page_keeps_seven_children_and_shows_the_page() {
    let mut bodies = BTreeMap::new();
    insert_block(&mut bodies, "wide", 200, &tx_list("firstpage", 10, 40, 1));
    bodies.insert(
        "/block/wide/txs/25".to_string(),
        tx_list("secondpage", 11, 44, 2),
    );
    let (mut host, rec) = host_with(Network::Mainnet, bodies);
    host.open_block("wide").expect("wide");
    let text = host.rendered();
    assert!(text.contains("tx txid firstpage"), "{text}");
    assert_line(&text, "pagination control page 1 start 0 active");
    assert!(
        !text.contains("pagination control page 2 start 25 active"),
        "{text}"
    );
    assert_has_row(&host.painted_rows(), "row page 25");

    host.open_row("row page 25").expect("next page");
    assert!(
        rec.gets().iter().any(|path| path == "/block/wide/txs/25"),
        "{:?}",
        rec.gets()
    );
    let text = host.rendered();
    assert_line(&text, "pagination children 7");
    assert!(text.contains("tx txid secondpage"), "{text}");
    assert_line(&text, "pagination control page 2 start 25 active");
    assert!(
        !text.contains("pagination control page 1 start 0 active"),
        "{text}"
    );
    let controls = lines_with_prefix(&text, "pagination control ");
    assert_eq!(controls.len(), 7, "{controls:?}\n{text}");
    let pages = lines_with_prefix(&text, "pagination control page ");
    assert_eq!(
        pages,
        vec![
            "pagination control page 1 start 0",
            "pagination control page 2 start 25 active",
            "pagination control page 3 start 50",
        ],
        "{text}"
    );
    assert_has_row(&host.painted_rows(), "row page 50");
    assert_requests(&rec);
    assert_no_banned(&text);
}

#[test]
fn fewer_pages_do_not_invent_a_seventh_child() {
    let mut bodies = BTreeMap::new();
    insert_block(&mut bodies, "nineteen", 19, "[]");
    insert_block(&mut bodies, "paged", 41, &tx_list("firstpage", 10, 40, 1));
    bodies.insert(
        "/block/paged/txs/25".to_string(),
        tx_list("secondpage", 11, 44, 2),
    );
    let (mut host, rec) = host_with(Network::Mainnet, bodies);
    host.open_block("nineteen").expect("nineteen");
    let text = host.rendered();
    assert_line(&text, "19 transactions");
    assert_line(&text, "pagination children 5");
    let controls = lines_with_prefix(&text, "pagination control ");
    assert_eq!(
        controls,
        vec![
            "pagination control first",
            "pagination control previous",
            "pagination control page 1 start 0 active",
            "pagination control next",
            "pagination control last",
        ],
        "{text}"
    );
    assert_no_banned(&text);

    host.open_block("paged").expect("paged");
    let text = host.rendered();
    assert_line(&text, "41 transactions");
    assert_line(&text, "pagination children 6");
    let pages = lines_with_prefix(&text, "pagination control page ");
    assert_eq!(
        pages,
        vec![
            "pagination control page 1 start 0 active",
            "pagination control page 2 start 25",
        ],
        "{text}"
    );
    assert_eq!(
        lines_with_prefix(&text, "pagination control ").len(),
        6,
        "{text}"
    );
    host.open_row("row page 25").expect("second page");
    let text = host.rendered();
    assert_line(&text, "pagination children 6");
    assert_line(&text, "pagination control page 2 start 25 active");
    assert!(text.contains("tx txid secondpage"), "{text}");
    assert_requests(&rec);
    assert_no_banned(&text);
}

const FIVE_PAGE_ONE: &[&str] = &[
    "pagination control first",
    "pagination control previous",
    "pagination control page 1 start 0 active",
    "pagination control page 2 start 25",
    "pagination control page 3 start 50",
    "pagination control page 4 start 75",
    "pagination control page 5 start 100",
    "pagination control next",
    "pagination control last",
];

#[test]
fn five_pages_paint_nine_pager_children() {
    let mut bodies = BTreeMap::new();
    insert_block(&mut bodies, "five", 125, &tx_list("firstpage", 10, 40, 1));
    bodies.insert(
        "/block/five/txs/25".to_string(),
        tx_list("secondpage", 11, 44, 2),
    );
    let (mut host, rec) = host_with(Network::Mainnet, bodies);
    host.open_block("five").expect("five");
    let text = host.rendered();
    assert_line(&text, "125 transactions");
    assert_line(&text, "pagination children 9");
    let controls = lines_with_prefix(&text, "pagination control ");
    assert_eq!(controls, FIVE_PAGE_ONE, "{text}");
    assert!(!text.contains("pagination control page 6"), "{text}");
    assert_has_row(&host.painted_rows(), "row page 25");
    assert_requests(&rec);
    assert_no_banned(&text);

    host.open_row("row page 25").expect("second page");
    assert!(
        rec.gets().iter().any(|path| path == "/block/five/txs/25"),
        "{:?}",
        rec.gets()
    );
    let text = host.rendered();
    assert_line(&text, "pagination children 9");
    assert!(text.contains("tx txid secondpage"), "{text}");
    assert_line(&text, "pagination control page 2 start 25 active");
    assert!(
        !text.contains("pagination control page 1 start 0 active"),
        "{text}"
    );
    let controls = lines_with_prefix(&text, "pagination control ");
    assert_eq!(controls.len(), 9, "{controls:?}\n{text}");
    let pages = lines_with_prefix(&text, "pagination control page ");
    assert_eq!(
        pages,
        vec![
            "pagination control page 1 start 0",
            "pagination control page 2 start 25 active",
            "pagination control page 3 start 50",
            "pagination control page 4 start 75",
            "pagination control page 5 start 100",
        ],
        "{text}"
    );
    assert_has_row(&host.painted_rows(), "row page 50");
    assert_requests(&rec);
    assert_no_banned(&text);
}
