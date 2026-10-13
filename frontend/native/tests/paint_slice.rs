//! Screen paint for peg badges, highlighted addresses, poison alerts,
//! liquid asset ids, and the block transaction sentence.
//!
//! Paths stay on the indexer suffix. No request path carries a query or a fragment.

use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use splora_api::{Network, api_root};
use splora_native::{
    ClientError, ClientRequest, ExplorerClient, ExplorerGet, ExplorerPost, Screen, ScreenHost,
};

const PEG_ADDRESS: &str = "1BxoGcMg14oaH3CwHD2hF4gU9VcfgX5yoR";
const VIEWED: &str = "bc1qhighlight";
const OTHER: &str = "bc1qotheraddress";
const POISON_WARNING: &str = "Warning! This transaction involves deceptively similar addresses.";
const PEGOUT: &str = r#""pegout":{"genesis_hash":"000000000019d6689c085ae165831e934ff763ae46a2a6c172b3f1b60a8ce26f","scriptpubkey":"76a91489abcdefabbaabbaabbaabbaabbaabbaabbaabba88ac","scriptpubkey_asm":"OP_DUP OP_HASH160 OP_EQUALVERIFY OP_CHECKSIG","scriptpubkey_address":"1BxoGcMg14oaH3CwHD2hF4gU9VcfgX5yoR"}"#;

const BLOCK_DETAILS: &[&str] = &[
    "block details",
    "id oneblock",
    "height 800000",
    "version 1",
    "timestamp 1700000000",
    "tx_count 1",
    "size 20",
    "weight 80",
    "merkle_root mm",
    "previousblockhash prevblock",
    "mediantime 1700000000",
    "nonce 1",
    "bits 2",
    "difficulty 1",
];

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

fn has_line(text: &str, line: &str) -> bool {
    text.lines().any(|row| row == line)
}

fn count_line(text: &str, line: &str) -> usize {
    text.lines().filter(|row| *row == line).count()
}

fn production_screen() -> &'static str {
    let src = include_str!("../src/screens.rs");
    src.split("#[cfg(test)]\nmod tests {")
        .next()
        .expect("screens tests module")
}

fn assert_plain_paths(paths: &[String]) {
    for path in paths {
        assert!(!path.contains('?'), "{path}");
        assert!(!path.contains('#'), "{path}");
    }
}

fn assert_lines(text: &str, lines: &[&str]) {
    for line in lines {
        assert!(has_line(text, line), "missing {line} in {text}");
    }
}

fn assert_absent(text: &str, lines: &[&str]) {
    for line in lines {
        assert!(!has_line(text, line), "still has {line} in {text}");
    }
}

fn output(address: &str, value: u64, extra: &str) -> String {
    let tail = if extra.is_empty() {
        String::new()
    } else {
        format!(",{extra}")
    };
    format!(
        r#"{{"scriptpubkey":"0014aa","scriptpubkey_asm":"OP_0","scriptpubkey_type":"v0_p2wpkh","scriptpubkey_address":"{address}","value":{value}{tail}}}"#
    )
}

fn spend(txid: &str, prevout: &str, extra: &str) -> String {
    let tail = if extra.is_empty() {
        String::new()
    } else {
        format!(",{extra}")
    };
    format!(
        r#"{{"txid":"{txid}","vout":0,"prevout":{prevout},"scriptsig":"","scriptsig_asm":"","is_coinbase":false,"sequence":4294967295{tail}}}"#
    )
}

fn coinbase() -> String {
    r#"{"txid":"0000000000000000000000000000000000000000000000000000000000000000","vout":4294967295,"prevout":null,"scriptsig":"","scriptsig_asm":"","is_coinbase":true,"sequence":4294967295}"#.to_string()
}

fn tx_body(txid: &str, vin: &str, vout: &str) -> String {
    format!(
        r#"{{"txid":"{txid}","version":1,"locktime":0,"vin":[{vin}],"vout":[{vout}],"size":10,"weight":40,"sigops":0,"fee":1,"status":{{"confirmed":true}}}}"#
    )
}

fn address_stats(address: &str) -> String {
    format!(
        r#"{{"address":"{address}","chain_stats":{{"tx_count":1,"funded_txo_count":1,"spent_txo_count":0,"funded_txo_sum":10,"spent_txo_sum":0}},"mempool_stats":{{"tx_count":0,"funded_txo_count":0,"spent_txo_count":0,"funded_txo_sum":0,"spent_txo_sum":0}}}}"#
    )
}

fn address_bodies(address: &str, txs: &str) -> BTreeMap<String, String> {
    BTreeMap::from([
        (format!("/address/{address}"), address_stats(address)),
        (format!("/address/{address}/txs"), txs.to_string()),
        (format!("/address/{address}/utxo"), "[]".to_string()),
    ])
}

fn block_json(id: &str, tx_count: u32, previous: &str) -> String {
    format!(
        r#"{{"id":"{id}","height":800000,"version":1,"timestamp":1700000000,"tx_count":{tx_count},"size":20,"weight":80,"merkle_root":"mm","previousblockhash":"{previous}","mediantime":1700000000,"nonce":1,"bits":2,"difficulty":1.0}}"#
    )
}

fn insert_block(bodies: &mut BTreeMap<String, String>, id: &str, tx_count: u32, previous: &str) {
    bodies.insert(format!("/block/{id}"), block_json(id, tx_count, previous));
    bodies.insert(format!("/block/{id}/txs"), "[]".to_string());
    bodies.insert(format!("/block/{id}/txids"), "[]".to_string());
}

fn open_tx(txid: &str, body: &str) -> (ScreenHost<Rec>, Rec) {
    let (mut host, rec) = host_with(
        Network::Mainnet,
        BTreeMap::from([(format!("/tx/{txid}"), body.to_string())]),
    );
    host.open_transaction(txid).expect("tx");
    let paths = rec.gets();
    assert!(
        paths.iter().any(|path| path == &format!("/tx/{txid}")),
        "{paths:?}"
    );
    assert_plain_paths(&paths);
    (host, rec)
}

#[test]
fn peg_in_text_and_peg_out_badge_come_from_the_transaction_json() {
    let pegin = tx_body(
        "pegin-tx",
        &spend(
            "pegin-prev",
            &output("bc1qpegin", 10, ""),
            r#""is_pegin":true"#,
        ),
        &output("bc1qchange", 9, ""),
    );
    let (host, _) = open_tx("pegin-tx", &pegin);
    let text = host.rendered();
    assert!(has_line(&text, "Peg-in"), "{text}");
    assert!(!text.contains("Liquid Peg Out"), "{text}");

    let pegout = tx_body(
        "pegout-tx",
        &spend("pegout-prev", &output("bc1qsource", 10, ""), ""),
        &output("lq1qside", 9, PEGOUT),
    );
    let (host, _) = open_tx("pegout-tx", &pegout);
    let text = host.rendered();
    assert!(has_line(&text, "Liquid Peg Out"), "{text}");
}

#[test]
fn address_stats_do_not_invent_a_peg_badge() {
    let normal = tx_body(
        "normal-tx",
        &spend("prev-normal", &output(PEG_ADDRESS, 10, ""), ""),
        &output(PEG_ADDRESS, 9, ""),
    );
    let (mut host, _) = host_with(
        Network::Mainnet,
        address_bodies(PEG_ADDRESS, &format!("[{normal}]")),
    );
    host.open_address(PEG_ADDRESS).expect("address");
    let text = host.rendered();
    assert!(!text.contains("Peg-in"), "{text}");
    assert!(!text.contains("Liquid Peg Out"), "{text}");

    let flagged = tx_body(
        "flagged-tx",
        &spend(
            "prev-flag",
            &output("bc1qsource", 10, ""),
            r#""is_pegin":true"#,
        ),
        &output("lq1qside", 9, PEGOUT),
    );
    let (mut host, _) = host_with(
        Network::Mainnet,
        address_bodies(PEG_ADDRESS, &format!("[{flagged}]")),
    );
    host.open_address(PEG_ADDRESS).expect("flagged address");
    let text = host.rendered();
    assert!(has_line(&text, "Peg-in"), "{text}");
    assert!(has_line(&text, "Liquid Peg Out"), "{text}");
}

#[test]
fn address_highlight_uses_scriptpubkey_address() {
    let screen = production_screen();
    assert!(
        screen.contains("highlight_counts("),
        "screens.rs does not call highlight_counts"
    );
    let two_inputs = tx_body(
        "hl-two-in",
        &format!(
            "{},{}",
            spend("in-0", &output(VIEWED, 5, ""), ""),
            spend("in-1", &output(VIEWED, 4, ""), "")
        ),
        &output(OTHER, 8, ""),
    );
    let (mut host, _) = host_with(
        Network::Mainnet,
        address_bodies(VIEWED, &format!("[{two_inputs}]")),
    );
    host.open_address(VIEWED).expect("two inputs");
    let text = host.rendered();
    assert_eq!(
        count_line(&text, "highlight input bc1qhighlight"),
        2,
        "{text}"
    );
    assert_eq!(
        count_line(&text, "highlight output bc1qhighlight"),
        0,
        "{text}"
    );
    assert!(
        text.lines()
            .all(|line| !(line.starts_with("highlight ") && line.contains(OTHER))),
        "{text}"
    );

    let both = tx_body(
        "hl-both",
        &spend("in-0", &output(VIEWED, 6, ""), ""),
        &format!("{},{}", output(VIEWED, 3, ""), output(OTHER, 2, "")),
    );
    let (mut host, _) = host_with(
        Network::Mainnet,
        address_bodies(VIEWED, &format!("[{both}]")),
    );
    host.open_address(VIEWED).expect("input and output");
    let text = host.rendered();
    assert_eq!(
        count_line(&text, "highlight input bc1qhighlight"),
        1,
        "{text}"
    );
    assert_eq!(
        count_line(&text, "highlight output bc1qhighlight"),
        1,
        "{text}"
    );
}

#[test]
fn poison_alerts_use_the_shared_lookalike_pairs() {
    let screen = production_screen();
    assert!(
        !screen.contains("fn poison_split"),
        "screens.rs still defines poison_split"
    );
    assert!(
        screen.contains("poison_alerts("),
        "screens.rs does not call poison_alerts"
    );
    assert!(
        screen.contains("address_parts("),
        "screens.rs does not call address_parts"
    );
    let prefix_only = tx_body(
        "poison-prefix",
        &spend("in-0", &output("bc1qunrelated", 8, ""), ""),
        &format!(
            "{},{}",
            output("bc1qlookaaa", 3, ""),
            output("bc1qlookXY", 4, "")
        ),
    );
    let (host, _) = open_tx("poison-prefix", &prefix_only);
    let text = host.rendered();
    let poison = "poison prefix bc1qlook infix aaa infix XY";
    assert!(has_line(&text, poison), "{text}");
    assert!(has_line(&text, POISON_WARNING), "{text}");
    assert!(
        text.lines()
            .filter(|line| line.starts_with("poison "))
            .all(|line| !line.contains("suffix")),
        "{text}"
    );

    let incoming = "bc1qge86gqjnk8aqs3nvv7ejrvcd4zq6qur36p6e3r";
    let outgoing = "bc1qge8xyxjex6zzzx5g8hh65vsel4e548p26p6e3r";
    let with_suffix = tx_body(
        "poison-suffix",
        &spend("in-0", &output(incoming, 8, ""), ""),
        &output(outgoing, 3, ""),
    );
    let (host, _) = open_tx("poison-suffix", &with_suffix);
    let text = host.rendered();
    assert!(
        has_line(
            &text,
            "poison prefix bc1qge8 infix 6gqjnk8aqs3nvv7ejrvcd4zq6qur3 infix xyxjex6zzzx5g8hh65vsel4e548p2 suffix 6p6e3r"
        ),
        "{text}"
    );
    assert!(has_line(&text, POISON_WARNING), "{text}");

    let short_postfix = tx_body(
        "short-pair",
        &format!(
            "{},{}",
            spend("in-0", &output("bc1qMMMend", 8, ""), ""),
            spend("in-1", &output("bc1qZZZend", 7, ""), "")
        ),
        &format!(
            "{},{}",
            output("bc1qMMMend", 3, ""),
            output("bc1qZZZend", 4, "")
        ),
    );
    let (host, _) = open_tx("short-pair", &short_postfix);
    let text = host.rendered();
    assert!(
        text.lines().all(|line| !line.starts_with("poison ")),
        "{text}"
    );
    assert!(!text.contains("Warning!"), "{text}");

    let ordinary = tx_body(
        "plain-tx",
        &coinbase(),
        &format!(
            "{},{}",
            output("bc1qalice", 3, ""),
            output("bc1qbob", 4, "")
        ),
    );
    let (host, _) = open_tx("plain-tx", &ordinary);
    let text = host.rendered();
    assert!(!text.contains("poison"), "{text}");
    assert!(!text.contains("Warning!"), "{text}");
}

#[test]
fn assets_registry_lists_openable_asset_ids() {
    let issued = "12".repeat(32);
    let native = "ab".repeat(32);
    assert_eq!(issued.len(), 64);
    assert_eq!(native.len(), 64);
    let registry = format!(
        r#"[{{"asset_id":"{issued}","issuance_txin":{{"txid":"issuance-tx","vin":0}},"issuance_prevout":{{"txid":"issuance-prev","vout":1}},"reissuance_token":"reissue-token","status":{{"confirmed":true}},"chain_stats":{{"tx_count":4,"issuance_count":1,"issued_amount":1000,"burned_amount":2,"has_blinded_issuances":false,"burned_reissuance_tokens":0}},"mempool_stats":{{"tx_count":0,"issuance_count":0,"issued_amount":0,"burned_amount":0,"has_blinded_issuances":false,"burned_reissuance_tokens":0}},"name":"Tether USD","ticker":"USDt"}},{{"asset_id":"{native}","chain_stats":{{"tx_count":3,"peg_in_count":1,"peg_in_amount":50,"peg_out_count":1,"peg_out_amount":20,"burn_count":0,"burned_amount":0}},"mempool_stats":{{"tx_count":0,"peg_in_count":0,"peg_in_amount":0,"peg_out_count":0,"peg_out_amount":0,"burn_count":0,"burned_amount":0}}}}]"#
    );
    let asset_path = format!("/asset/{issued}");
    let (mut host, rec) = host_with(
        Network::Liquid,
        BTreeMap::from([
            ("/assets/registry".to_string(), registry),
            (
                asset_path.clone(),
                r#"{"name":"Tether USD","ticker":"USDt","precision":8}"#.to_string(),
            ),
        ]),
    );
    host.activate(Screen::Dashboard).expect("dashboard");
    let text = host.rendered();
    assert!(has_line(&text, &format!("asset id {issued}")), "{text}");
    assert!(has_line(&text, &format!("asset id {native}")), "{text}");
    let rows = host.painted_rows();
    let issued_row = format!("row asset {issued}");
    let native_row = format!("row asset {native}");
    assert!(rows.iter().any(|row| row == &issued_row), "{rows:?}");
    assert!(rows.iter().any(|row| row == &native_row), "{rows:?}");
    host.open_row(&issued_row).expect("open asset");
    let paths = rec.gets();
    assert!(paths.iter().any(|path| path == &asset_path), "{paths:?}");
    let text = host.rendered();
    assert!(text.contains("Tether USD"), "{text}");
    assert!(text.contains("USDt"), "{text}");
    assert!(
        paths.iter().all(|path| path != "/assets/registry/search"),
        "{paths:?}"
    );
    assert_plain_paths(&paths);
}

#[test]
fn one_transaction_sentence_and_block_details_toggle() {
    let screen = production_screen();
    assert!(
        screen.contains("transaction_count_heading("),
        "screens.rs does not call transaction_count_heading"
    );
    assert!(
        screen.contains("cap_recent_transactions("),
        "screens.rs does not call cap_recent_transactions"
    );
    assert!(
        !screen.contains("if block.tx_count == 1"),
        "screens.rs still has a private transaction plural"
    );
    let mut bodies = BTreeMap::new();
    insert_block(&mut bodies, "oneblock", 1, "prevblock");
    insert_block(&mut bodies, "prevblock", 2, "olderblock");
    let (mut host, rec) = host_with(Network::Mainnet, bodies);
    host.open_block("oneblock").expect("one block");
    let opened = host.rendered();
    assert!(has_line(&opened, "1 transaction"), "{opened}");
    assert!(!has_line(&opened, "block details"), "{opened}");
    assert!(
        !has_line(&opened, "previousblockhash prevblock"),
        "{opened}"
    );
    assert!(!opened.contains("vB/s"), "{opened}");
    assert!(!opened.contains("mempool-block"), "{opened}");
    assert!(!opened.contains("Fiat price last updated"), "{opened}");
    let rows = host.painted_rows();
    assert!(
        rows.iter().any(|row| row == "row block details"),
        "{rows:?}"
    );
    assert!(
        rows.iter()
            .any(|row| row.starts_with("row previous_hash prevblock")),
        "{rows:?}"
    );

    host.open_row("row block details").expect("show details");
    let text = host.rendered();
    assert_lines(&text, BLOCK_DETAILS);

    host.open_row("row block details").expect("hide details");
    let text = host.rendered();
    assert_absent(&text, BLOCK_DETAILS);
    assert!(has_line(&text, "1 transaction"), "{text}");
    assert!(
        host.painted_rows()
            .iter()
            .any(|row| row.starts_with("row previous_hash prevblock")),
        "{:?}",
        host.painted_rows()
    );

    let prev = host
        .painted_rows()
        .into_iter()
        .find(|row| row.starts_with("row previous_hash prevblock"))
        .expect("previous hash row");
    host.open_row(&prev).expect("open previous");
    assert_eq!(host.block.query, "prevblock");
    assert!(
        rec.gets().iter().any(|path| path == "/block/prevblock"),
        "{:?}",
        rec.gets()
    );

    let mut two = BTreeMap::new();
    insert_block(&mut two, "prevblock", 2, "olderblock");
    let (mut host, _) = host_with(Network::Mainnet, two);
    host.open_block("prevblock").expect("two transactions");
    assert!(
        !has_line(&host.rendered(), "1 transaction"),
        "{}",
        host.rendered()
    );
}
