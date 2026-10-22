//! GPUI screen for the rest of the public indexer.
//!
//! A 64-digit id is a block height. Failed broadcast and hex bodies stay
//! sentences. Compact history, package broadcast, mempool pages, and liquid
//! registry stay on path forms. The client rejects `?`.

use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use splora_api::{Network, api_root};
use splora_native::{
    ClientError, ClientRequest, ExplorerClient, ExplorerGet, ExplorerPost, NpubPopupModel, Screen,
    ScreenHost, painted_shell_text,
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

fn assert_no_closed(paths: &[String]) {
    for path in paths {
        assert!(!path.contains('?'), "{path}");
        assert!(!path.contains("/internal"), "{path}");
        assert!(!path.contains("/electrum"), "{path}");
        assert!(!path.contains("/api/v1/ws"), "{path}");
        assert!(!path.contains("/block-template"), "{path}");
    }
}

fn block_json(id: &str) -> String {
    format!(
        r#"{{"id":"{id}","height":800000,"version":1,"timestamp":1700000000,"tx_count":1,"size":20,"weight":80,"merkle_root":"mm","previousblockhash":"pp","mediantime":1700000000,"nonce":1,"bits":2,"difficulty":1.0}}"#
    )
}

fn address_stats(address: &str) -> String {
    format!(
        r#"{{"address":"{address}","chain_stats":{{"tx_count":1,"funded_txo_count":1,"spent_txo_count":0,"funded_txo_sum":10,"spent_txo_sum":0}},"mempool_stats":{{"tx_count":0,"funded_txo_count":0,"spent_txo_count":0,"funded_txo_sum":0,"spent_txo_sum":0}}}}"#
    )
}

fn paint_address(address: &str, stats: &str) -> String {
    let (mut host, rec) = host_with(
        Network::Mainnet,
        BTreeMap::from([
            (format!("/address/{address}"), stats.to_string()),
            (format!("/address/{address}/txs"), "[]".to_string()),
            (format!("/address/{address}/utxo"), "[]".to_string()),
        ]),
    );
    host.open_address(address).expect("address");
    let paths = rec.gets();
    assert!(
        paths
            .iter()
            .all(|path| !path.contains('?') && !path.contains('#')),
        "{paths:?}"
    );
    painted(&host)
}

/// `open_address` sends a 64-hex id to `GET /scripthash/:hash`, not `/address/`.
fn paint_scripthash(hash: &str, stats: &str) -> String {
    assert_eq!(hash.len(), 64, "{hash}");
    assert!(hash.bytes().all(|byte| byte.is_ascii_hexdigit()), "{hash}");
    let (mut host, rec) = host_with(
        Network::Mainnet,
        BTreeMap::from([
            (format!("/scripthash/{hash}"), stats.to_string()),
            (format!("/scripthash/{hash}/txs"), "[]".to_string()),
            (format!("/scripthash/{hash}/utxo"), "[]".to_string()),
        ]),
    );
    host.open_address(hash).expect("scripthash");
    let paths = rec.gets();
    assert!(
        paths
            .iter()
            .any(|path| path == &format!("/scripthash/{hash}")),
        "{paths:?}"
    );
    assert!(
        paths.iter().all(|path| {
            !path.contains('?') && !path.contains('#') && !path.starts_with("/address/")
        }),
        "{paths:?}"
    );
    painted(&host)
}

const NOTICE_TX: &str = r#"{"txid":"noticetx","version":1,"locktime":0,"vin":[],"vout":[{"scriptpubkey":"0014aa","scriptpubkey_asm":"OP_0","scriptpubkey_type":"v0_p2wpkh","scriptpubkey_address":"bc1qout","value":9}],"size":10,"weight":40,"sigops":0,"fee":1,"status":{"confirmed":true,"block_height":4,"block_hash":"tiphash1","block_time":8}}"#;

#[test]
fn rest_of_indexer_screen_shows_height_broadcast_history_and_liquid() {
    let digits = "9".repeat(64);
    assert!(digits.len() == 64);
    assert!(digits.parse::<u64>().is_err());
    let hash = "heightblock1";
    let (mut host, rec) = host_with(
        Network::Mainnet,
        BTreeMap::from([
            (format!("/block-height/{digits}"), hash.to_string()),
            (format!("/block/{hash}"), block_json(hash)),
            (format!("/block/{hash}/txs"), "[]".to_string()),
            (format!("/block/{hash}/txids"), "[]".to_string()),
            (format!("/block/{hash}/header"), "aabb".to_string()),
            (format!("/block/{hash}/raw"), "ccdd".to_string()),
            (format!("/block/{hash}/txid/0"), "indextxid01".to_string()),
        ]),
    );
    host.activate(Screen::Dashboard).expect("dashboard");
    host.set_query(&digits);
    let _ = host.reload();
    let paths = rec.gets();
    assert!(
        paths
            .iter()
            .any(|path| path == &format!("/block-height/{digits}")),
        "64-digit id was not a block height: {paths:?}"
    );
    assert!(
        paths.iter().all(|path| {
            path != &format!("/tx/{digits}")
                && path != &format!("/block/{digits}")
                && !path.contains("/asset/")
                && !path.contains("/assets/registry")
                && !path.contains("/scripthash/")
        }),
        "64-digit id was probed as something other than a height: {paths:?}"
    );
    assert_no_closed(&paths);
    let block_paint = painted(&host);
    assert!(block_paint.contains("block header aabb"), "{block_paint}");
    assert!(block_paint.contains("block raw ccdd"), "{block_paint}");
    assert!(
        block_paint.contains("indextxid01"),
        "txid at an index is missing: {block_paint}"
    );
    assert!(
        paths
            .iter()
            .any(|path| path == &format!("/block/{hash}/txid/0")),
        "{paths:?}"
    );
    assert!(
        paths.iter().all(|path| !path.contains("/txs/outspends")),
        "batch spend query is not a path the client can send: {paths:?}"
    );

    let sentence = "Transaction not found";
    let (mut host, rec) = host_with(
        Network::Mainnet,
        BTreeMap::from([("/tx".to_string(), sentence.to_string())]),
    );
    host.activate(Screen::Broadcast).expect("broadcast");
    host.broadcast.raw_tx = "abcd".to_string();
    host.submit().expect("post tx");
    let failed = painted(&host);
    assert!(failed.contains(sentence), "{failed}");
    assert!(
        !failed.contains("broadcast txid"),
        "failed POST /tx was painted as a txid: {failed}"
    );
    assert_eq!(rec.posts()[0].0, "/tx");

    let rejection = "package rejected";
    let (mut host, _) = host_with(
        Network::Mainnet,
        BTreeMap::from([("/txs/package".to_string(), rejection.to_string())]),
    );
    host.activate(Screen::Broadcast).expect("broadcast");
    host.broadcast.raw_tx = "abcd,ef01".to_string();
    host.submit().expect("package");
    let rejected = painted(&host);
    assert!(rejected.contains(rejection), "{rejected}");
    assert!(
        !rejected.contains("broadcast package"),
        "unparsed package replaced the rejection: {rejected}"
    );

    let package = r#"{"package_msg":"success","tx-results":{"aa":{"txid":"pkgspace","vsize":10}}}"#;
    let (mut host, rec) = host_with(
        Network::Mainnet,
        BTreeMap::from([("/txs/package".to_string(), package.to_string())]),
    );
    host.activate(Screen::Broadcast).expect("broadcast");
    host.set_query("abcd ef01");
    host.submit().expect("space package");
    assert_eq!(rec.posts()[0].0, "/txs/package", "{:?}", rec.posts());
    assert!(rec.posts()[0].1.contains("abcd"), "{:?}", rec.posts());
    assert!(rec.posts()[0].1.contains("ef01"), "{:?}", rec.posts());
    let opened = host
        .painted_rows()
        .into_iter()
        .find(|row| row.contains("pkgspace"))
        .expect("broadcast txid row");
    host.open_row(&opened).expect("open broadcast txid");
    assert_eq!(host.screen, Screen::Tx);
    assert_eq!(host.tx.txid, "pkgspace");

    let (mut host, rec) = host_with(
        Network::Mainnet,
        BTreeMap::from([("/txs/package".to_string(), package.to_string())]),
    );
    host.activate(Screen::Broadcast).expect("broadcast");
    host.set_query("abcd\nef01");
    host.submit().expect("newline package");
    assert_eq!(rec.posts()[0].0, "/txs/package", "{:?}", rec.posts());

    let (mut host, rec) = host_with(
        Network::Mainnet,
        BTreeMap::from([
            ("/tx/noticetx".to_string(), NOTICE_TX.to_string()),
            (
                "/tx/noticetx/outspends".to_string(),
                r#"[{"spent":false},{"spent":true,"txid":"spendtx111","vin":1}]"#.to_string(),
            ),
            (
                "/tx/noticetx/outspend/0".to_string(),
                r#"{"spent":false}"#.to_string(),
            ),
            (
                "/tx/noticetx/merkle-proof".to_string(),
                r#"{"block_height":4,"merkle":["abcmerklehash","defmerklehash"],"pos":1}"#
                    .to_string(),
            ),
            ("/tx/noticetx/hex".to_string(), "not-served".to_string()),
            ("/tx/noticetx/raw".to_string(), "abcd".to_string()),
            (
                "/tx/noticetx/merkleblock-proof".to_string(),
                "eeff".to_string(),
            ),
            (
                "/tx/pkgspace".to_string(),
                r#"{"txid":"pkgspace","version":1,"locktime":0,"vin":[],"vout":[],"size":1,"weight":4,"sigops":0,"fee":1,"status":{"confirmed":false}}"#.to_string(),
            ),
        ]),
    );
    host.open_transaction("noticetx").expect("tx");
    let tx_paint = painted(&host);
    assert!(tx_paint.contains("not-served"), "{tx_paint}");
    assert!(
        !tx_paint.contains("tx hex"),
        "failed hex was not a notice: {tx_paint}"
    );
    assert!(tx_paint.contains("abcmerklehash"), "{tx_paint}");
    assert!(tx_paint.contains("defmerklehash"), "{tx_paint}");
    assert!(
        tx_paint.contains("unspent") || tx_paint.contains("spent false"),
        "unspent output is missing: {tx_paint}"
    );
    assert!(tx_paint.contains("tx raw abcd"), "{tx_paint}");
    assert!(tx_paint.contains("merkleblock eeff"), "{tx_paint}");
    let tx_paths = rec.gets();
    assert!(
        tx_paths
            .iter()
            .any(|path| path == "/tx/noticetx/outspend/0"),
        "per-output spend was not requested: {tx_paths:?}"
    );
    assert!(
        tx_paths.iter().any(|path| path == "/tx/noticetx/raw"),
        "{tx_paths:?}"
    );
    assert!(
        tx_paths
            .iter()
            .any(|path| path == "/tx/noticetx/merkleblock-proof"),
        "{tx_paths:?}"
    );
    assert_no_closed(&tx_paths);
    let merkle_row = host
        .painted_rows()
        .into_iter()
        .find(|row| row.contains("abcmerklehash"))
        .expect("merkle hash row");
    host.open_row(&merkle_row).expect("open merkle");
    assert_eq!(host.tx.txid, "abcmerklehash");

    let script = "bc1qsummaryaddr000000000000000000000000";
    let summary =
        r#"[{"txid":"sumtx1","height":90,"value":-15,"time":1700000000,"tx_position":3}]"#;
    let (mut host, rec) = host_with(
        Network::Mainnet,
        BTreeMap::from([
            (format!("/address/{script}"), address_stats(script)),
            (format!("/address/{script}/txs"), "[]".to_string()),
            (format!("/address/{script}/utxo"), "[]".to_string()),
            (
                format!("/address/{script}/txs/summary"),
                summary.to_string(),
            ),
            (
                "/tx/sumtx1".to_string(),
                r#"{"txid":"sumtx1","version":1,"locktime":0,"vin":[],"vout":[],"size":1,"weight":4,"sigops":0,"fee":1,"status":{"confirmed":true,"block_height":90}}"#.to_string(),
            ),
        ]),
    );
    host.open_address(script).expect("address");
    let addr_paths = rec.gets();
    assert!(
        addr_paths
            .iter()
            .any(|path| path == &format!("/address/{script}/txs/summary")),
        "compact history was not requested: {addr_paths:?}"
    );
    let summary_row = host
        .painted_rows()
        .into_iter()
        .find(|row| row.contains("sumtx1"))
        .expect("summary row");
    assert!(summary_row.contains("height 90"), "{summary_row}");
    assert!(summary_row.contains("value -15"), "{summary_row}");
    assert!(summary_row.contains("time 1700000000"), "{summary_row}");
    assert!(summary_row.contains("tx_position 3"), "{summary_row}");
    host.open_row(&summary_row).expect("open summary txid");
    assert_eq!(host.screen, Screen::Tx);
    assert_eq!(host.tx.txid, "sumtx1");

    let prefix_addr = "bc1qprefixmatch0000000000000000000000";
    let (mut host, _) = host_with(
        Network::Mainnet,
        BTreeMap::from([
            (
                "/address-prefix/bc1qzz".to_string(),
                format!(r#"["{prefix_addr}"]"#),
            ),
            (
                format!("/address/{prefix_addr}"),
                address_stats(prefix_addr),
            ),
            (format!("/address/{prefix_addr}/txs"), "[]".to_string()),
            (format!("/address/{prefix_addr}/utxo"), "[]".to_string()),
        ]),
    );
    host.activate(Screen::Dashboard).expect("dashboard");
    host.set_query("bc1qzz");
    host.reload().expect("prefix");
    let prefix_row = host
        .painted_rows()
        .into_iter()
        .find(|row| row.contains(prefix_addr))
        .expect("prefix match row");
    host.open_row(&prefix_row).expect("open prefix");
    assert_eq!(host.screen, Screen::Address);
    assert_eq!(host.address.address, prefix_addr);

    let summary_posts = r#"[{"txid":"multi1","height":3,"value":8,"time":11,"tx_position":1}]"#;
    let (mut host, rec) = host_with(
        Network::Mainnet,
        BTreeMap::from([
            ("/address/addrA".to_string(), address_stats("addrA")),
            ("/address/addrB".to_string(), address_stats("addrB")),
            ("/address/addrA/utxo".to_string(), "[]".to_string()),
            ("/address/addrB/utxo".to_string(), "[]".to_string()),
            (
                "/addresses/txs/summary".to_string(),
                summary_posts.to_string(),
            ),
            (format!("/address/addrA"), address_stats("addrA")),
            ("/address/addrA/txs".to_string(), "[]".to_string()),
        ]),
    );
    host.multi_address.input = "addrA, addrB".to_string();
    host.activate(Screen::MultiAddress).expect("addresses");
    let posted = rec.posts();
    assert!(
        posted.iter().any(|(path, body)| {
            path == "/addresses/txs/summary" && body.contains("addrA") && body.contains("addrB")
        }),
        "several addresses did not post the summary: {posted:?}"
    );
    let entered = host
        .painted_rows()
        .into_iter()
        .find(|row| row.contains("addrA") && !row.contains("funded_sum"))
        .expect("entered address row");
    host.open_row(&entered).expect("open entered address");
    assert_eq!(host.screen, Screen::Address);
    assert_eq!(host.address.address, "addrA");

    let (mut host, rec) = host_with(
        Network::Mainnet,
        BTreeMap::from([
            (
                "/mempool".to_string(),
                r#"{"count":1,"vsize":10,"total_fee":2,"fee_histogram":[]}"#.to_string(),
            ),
            ("/mempool/recent".to_string(), "[]".to_string()),
            (
                "/mempool/txids".to_string(),
                r#"["mempooltxid1","mempooltxid2"]"#.to_string(),
            ),
            (
                "/mempool/txids/page/mempooltxid2".to_string(),
                r#"["olderpooltx"]"#.to_string(),
            ),
            (
                "/tx/mempooltxid1".to_string(),
                r#"{"txid":"mempooltxid1","version":1,"locktime":0,"vin":[],"vout":[],"size":1,"weight":4,"sigops":0,"fee":1,"status":{"confirmed":false}}"#.to_string(),
            ),
        ]),
    );
    host.activate(Screen::Mempool).expect("mempool");
    let mem_paths = rec.gets();
    assert!(
        mem_paths.iter().any(|path| path == "/mempool/txids"),
        "mempool txid list was not requested: {mem_paths:?}"
    );
    let mem_row = host
        .painted_rows()
        .into_iter()
        .find(|row| row.contains("mempooltxid1"))
        .expect("mempool txid row");
    host.open_row(&mem_row).expect("open mempool txid");
    assert_eq!(host.screen, Screen::Tx);
    assert_eq!(host.tx.txid, "mempooltxid1");
    host.activate(Screen::Mempool).expect("mempool again");
    let page_row = host
        .painted_rows()
        .into_iter()
        .find(|row| row.contains("mempool page") || row.contains("older"))
        .expect("older mempool page row");
    host.open_row(&page_row).expect("older page");
    assert!(
        rec.gets()
            .iter()
            .any(|path| path == "/mempool/txids/page/mempooltxid2"),
        "older mempool page was not requested: {:?}",
        rec.gets()
    );
    assert!(painted(&host).contains("olderpooltx"), "{}", painted(&host));

    let (mut host, rec) = host_with(
        Network::Mainnet,
        BTreeMap::from([
            ("/blocks/tip/height".to_string(), "11".to_string()),
            ("/blocks/tip/hash".to_string(), "tiphash1".to_string()),
            ("/blocks".to_string(), "[]".to_string()),
            ("/mempool/recent".to_string(), "[]".to_string()),
            ("/fee-estimates".to_string(), "{}".to_string()),
            (
                "/mempool".to_string(),
                r#"{"count":0,"vsize":0,"total_fee":0,"fee_histogram":[]}"#.to_string(),
            ),
            ("/block/tiphash1".to_string(), block_json("tiphash1")),
            ("/block/tiphash1/txs".to_string(), "[]".to_string()),
            ("/block/tiphash1/txids".to_string(), "[]".to_string()),
        ]),
    );
    host.activate(Screen::Dashboard).expect("tip");
    let tip_row = host
        .painted_rows()
        .into_iter()
        .find(|row| row.contains("tiphash1"))
        .expect("tip hash row");
    host.open_row(&tip_row).expect("open tip");
    assert_eq!(host.screen, Screen::Block);
    assert_eq!(host.block.query, "tiphash1");
    assert!(
        rec.gets().iter().any(|path| path == "/block/tiphash1"),
        "{:?}",
        rec.gets()
    );

    let asset = "12".repeat(32);
    let registry = format!(
        r#"[{{"asset_id":"{asset}","name":"Tether USD","ticker":"USDt","precision":8,"issuance_txin":{{"txid":"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa","vin":0}},"issuance_prevout":{{"txid":"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa","vout":0}},"reissuance_token":"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa","status":{{"confirmed":true}},"chain_stats":{{"tx_count":1,"issuance_count":1,"issued_amount":1,"burned_amount":0,"has_blinded_issuances":false,"burned_reissuance_tokens":0}},"mempool_stats":{{"tx_count":0,"issuance_count":0,"issued_amount":0,"burned_amount":0,"has_blinded_issuances":false,"burned_reissuance_tokens":0}}}}]"#
    );
    let chain_tx = r#"[{"txid":"assetchaintx","version":1,"locktime":0,"vin":[],"vout":[],"size":1,"weight":4,"sigops":0,"fee":1,"status":{"confirmed":true}}]"#;
    let mem_tx = r#"[{"txid":"assetmempooltx","version":1,"locktime":0,"vin":[],"vout":[],"size":1,"weight":4,"sigops":0,"fee":1,"status":{"confirmed":false}}]"#;
    let (mut host, rec) = host_with(
        Network::Liquid,
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
            ("/assets/registry".to_string(), registry.clone()),
            (
                "/assets/registry/search".to_string(),
                r#"[{"asset_id":"bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb","name":"Liquid Bitcoin","ticker":"L-BTC"}]"#.to_string(),
            ),
            (
                format!("/tx/{asset}"),
                "Transaction not found".to_string(),
            ),
            (format!("/block/{asset}"), "Block not found".to_string()),
            (
                format!("/asset/{asset}"),
                r#"{"asset_id":"x","name":"Tether USD","ticker":"USDt","precision":8,"status":{"confirmed":true}}"#.to_string(),
            ),
            (format!("/asset/{asset}/txs"), chain_tx.to_string()),
            (format!("/asset/{asset}/txs/chain"), chain_tx.to_string()),
            (format!("/asset/{asset}/txs/mempool"), mem_tx.to_string()),
            (format!("/asset/{asset}/supply"), "21000000".to_string()),
            (
                format!("/asset/{asset}/supply/decimal"),
                "210000.5".to_string(),
            ),
            (format!("/assets/registry/{asset}"), r#"{"contract":null,"entity":null,"precision":8,"name":"Tether USD","ticker":"USDt"}"#.to_string()),
        ]),
    );
    host.activate(Screen::Dashboard).expect("liquid dashboard");
    let liquid_paths = rec.gets();
    assert!(
        liquid_paths.iter().any(|path| path == "/assets/registry"),
        "liquid registry was not requested: {liquid_paths:?}"
    );
    let registry_paint = painted(&host);
    assert!(registry_paint.contains("Tether USD"), "{registry_paint}");
    assert_no_closed(&liquid_paths);
    host.set_query("USDt");
    host.reload().expect("registry search");
    let search_paths = rec.gets();
    assert!(
        search_paths
            .iter()
            .any(|path| path == "/assets/registry/search"),
        "liquid search was not requested without a query string: {search_paths:?}"
    );
    assert!(
        search_paths.iter().all(|path| !path.contains('?')),
        "{search_paths:?}"
    );
    let search_paint = painted(&host);
    assert!(search_paint.contains("Liquid Bitcoin"), "{search_paint}");
    assert!(search_paint.contains("L-BTC"), "{search_paint}");
    host.activate(Screen::Dashboard).expect("clear search");
    host.set_query(&asset);
    host.reload().expect("asset");
    let asset_paths = rec.gets();
    assert!(
        asset_paths
            .iter()
            .any(|path| path == &format!("/asset/{asset}/txs/chain")),
        "confirmed asset history missing: {asset_paths:?}"
    );
    assert!(
        asset_paths
            .iter()
            .any(|path| path == &format!("/asset/{asset}/txs/mempool")),
        "mempool asset history missing: {asset_paths:?}"
    );
    assert!(
        asset_paths
            .iter()
            .any(|path| path == &format!("/asset/{asset}/supply")),
        "{asset_paths:?}"
    );
    assert!(
        asset_paths
            .iter()
            .any(|path| path == &format!("/asset/{asset}/supply/decimal")),
        "{asset_paths:?}"
    );
    let asset_paint = painted(&host);
    assert!(asset_paint.contains("assetchaintx"), "{asset_paint}");
    assert!(asset_paint.contains("assetmempooltx"), "{asset_paint}");
    assert!(asset_paint.contains("21000000"), "{asset_paint}");
    assert!(asset_paint.contains("210000.5"), "{asset_paint}");
    assert_no_closed(&asset_paths);

    let (mut host, rec) = host_with(
        Network::Mainnet,
        BTreeMap::from([
            ("/blocks".to_string(), "[]".to_string()),
            ("/mempool/recent".to_string(), "[]".to_string()),
            ("/blocks/tip/height".to_string(), "1".to_string()),
            ("/blocks/tip/hash".to_string(), "maintip".to_string()),
            ("/fee-estimates".to_string(), "{}".to_string()),
            (
                "/mempool".to_string(),
                r#"{"count":0,"vsize":0,"total_fee":0,"fee_histogram":[]}"#.to_string(),
            ),
            ("/assets/registry".to_string(), registry),
            (format!("/asset/{asset}"), r#"{"name":"Tether USD","ticker":"USDt"}"#.to_string()),
            (
                "/assets/registry/search".to_string(),
                r#"[{"asset_id":"bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb","name":"Liquid Bitcoin","ticker":"L-BTC"}]"#.to_string(),
            ),
        ]),
    );
    host.activate(Screen::Dashboard).expect("mainnet dashboard");
    host.set_query("USDt");
    let _ = host.reload();
    host.set_query(&asset);
    let _ = host.reload();
    let main_paths = rec.gets();
    assert!(
        main_paths
            .iter()
            .all(|path| !path.contains("/asset/") && !path.contains("/assets/registry")),
        "mainnet requested liquid routes: {main_paths:?}"
    );
    assert!(!painted(&host).contains("Fiat price last updated"));

    let (mut host, _) = host_with(Network::Mainnet, BTreeMap::new());
    host.activate(Screen::Calculator).expect("calculator");
    let calc = painted(&host);
    assert!(calc.contains('\u{00A5}'), "{calc}");
    assert!(!calc.contains("Fiat price last updated"), "{calc}");
    host.activate(Screen::Fiat).expect("fiat");
    let fiat = painted(&host);
    assert!(fiat.contains("\u{00A5}11057757"), "{fiat}");
    assert!(!fiat.contains("Fiat price last updated"), "{fiat}");
}

fn full_tx_json(txid: &str) -> String {
    format!(
        r#"[{{"txid":"{txid}","version":1,"locktime":0,"vin":[],"vout":[],"size":2,"weight":8,"sigops":0,"fee":4,"status":{{"confirmed":true,"block_height":12}}}}]"#
    )
}

/// Several addresses post `POST /addresses/txs`. Several scripthashes post
/// `POST /scripthashes/txs`. Compact summary stays. Neither post uses `?`.
#[test]
fn multi_address_posts_full_tx_history_for_addresses_and_scripthashes() {
    let (mut host, rec) = host_with(
        Network::Mainnet,
        BTreeMap::from([
            ("/address/addrA".to_string(), address_stats("addrA")),
            ("/address/addrB".to_string(), address_stats("addrB")),
            ("/address/addrA/utxo".to_string(), "[]".to_string()),
            ("/address/addrB/utxo".to_string(), "[]".to_string()),
            ("/addresses/txs".to_string(), full_tx_json("fulltx1")),
            ("/addresses/txs/summary".to_string(), "[]".to_string()),
        ]),
    );
    host.multi_address.input = "addrA, addrB".to_string();
    host.activate(Screen::MultiAddress).expect("addresses");
    let posted = rec.posts();
    assert!(
        posted.iter().any(|(path, body)| {
            path == "/addresses/txs/summary" && body.contains("addrA") && body.contains("addrB")
        }),
        "summary post was dropped: {posted:?}"
    );
    assert!(
        posted.iter().any(|(path, body)| {
            path == "/addresses/txs" && !path.contains('?') && body == r#"["addrA","addrB"]"#
        }),
        "several addresses did not post full history: {posted:?}"
    );
    assert!(
        posted
            .iter()
            .all(|(path, _)| path != "/scripthashes/txs" && !path.contains('?')),
        "address history used a scripthash post or a query string: {posted:?}"
    );
    assert!(
        host.painted_rows()
            .iter()
            .any(|row| row.contains("fulltx1")),
        "full address history was not shown: {:?}",
        host.painted_rows()
    );

    let hash_a = "a".repeat(64);
    let hash_b = "b".repeat(64);
    let (mut host, rec) = host_with(
        Network::Mainnet,
        BTreeMap::from([
            (format!("/address/{hash_a}"), address_stats(&hash_a)),
            (format!("/address/{hash_b}"), address_stats(&hash_b)),
            (format!("/address/{hash_a}/utxo"), "[]".to_string()),
            (format!("/address/{hash_b}/utxo"), "[]".to_string()),
            (
                "/scripthashes/txs".to_string(),
                full_tx_json("scripthistory1"),
            ),
            ("/addresses/txs/summary".to_string(), "[]".to_string()),
        ]),
    );
    host.multi_address.input = format!("{hash_a}, {hash_b}");
    host.activate(Screen::MultiAddress).expect("scripthashes");
    let posted = rec.posts();
    let expected = format!(r#"["{hash_a}","{hash_b}"]"#);
    assert!(
        posted.iter().any(|(path, body)| {
            path == "/scripthashes/txs" && !path.contains('?') && body == expected.as_str()
        }),
        "several scripthashes did not post full history: {posted:?}"
    );
    assert!(
        posted
            .iter()
            .all(|(path, _)| path != "/addresses/txs" && !path.contains('?')),
        "scripthash history used an address post or a query string: {posted:?}"
    );
    assert!(
        host.painted_rows()
            .iter()
            .any(|row| row.contains("scripthistory1")),
        "full scripthash history was not shown: {:?}",
        host.painted_rows()
    );
}

/// Mainnet bech32 pay-to-witness-pubkey-hash paints the badge `P2WPKH`
/// beside the row label `Type`. The address JSON has no type field. The
/// badge is the address string, not an uppercased output `Type V0_P2WPKH`.
/// A different address shape does not paint `P2WPKH`.
#[test]
fn address_screen_paints_p2wpkh_badge() {
    let address = "bc1q8yj0herd4r4yxszw3nkfvt53433thk0f5qst4g";
    let stats = format!(
        r#"{{"address":"{address}","chain_stats":{{"tx_count":1,"funded_txo_count":1,"spent_txo_count":0,"funded_txo_sum":10,"spent_txo_sum":0}},"mempool_stats":{{"tx_count":0,"funded_txo_count":0,"spent_txo_count":0,"funded_txo_sum":0,"spent_txo_sum":0}}}}"#
    );
    assert!(
        !stats.contains("\"type\""),
        "address JSON must not carry a type field: {stats}"
    );
    let painted = paint_address(address, &stats);
    assert!(
        painted.lines().any(|line| line == "Type P2WPKH"),
        "Type did not sit beside the P2WPKH badge: {painted}"
    );
    assert_eq!(
        painted.matches("Type P2WPKH").count(),
        1,
        "P2WPKH badge was not painted once: {painted}"
    );
    assert!(
        !painted.contains("V0_P2WPKH"),
        "address badge was uppercased into the output type: {painted}"
    );
    for absent in ["P2PKH", "P2SH", "P2TR", "P2WSH"] {
        assert!(
            !painted.contains(absent),
            "{absent} was painted on the p2wpkh address: {painted}"
        );
    }

    let other = "bc1qnotp2wpkhshape";
    let other_stats = address_stats(other);
    assert!(
        !other_stats.contains("\"type\""),
        "address JSON must not carry a type field: {other_stats}"
    );
    let other_painted = paint_address(other, &other_stats);
    assert!(
        !other_painted.contains("P2WPKH"),
        "a different address shape painted P2WPKH: {other_painted}"
    );
    assert!(
        !other_painted.lines().any(|line| line.starts_with("Type ")),
        "a different address shape painted a type badge: {other_painted}"
    );
}

/// Mainnet bech32 pay-to-witness-script-hash paints the badge `P2WSH`
/// beside the row label `Type`. The address starts with `bc1q` and then
/// has 58 bech32 characters. The address JSON has no type field. The
/// badge is the address string, not an uppercased output `Type V0_P2WSH`.
/// A shorter `bc1q` pay-to-witness-pubkey-hash address stays `Type P2WPKH`
/// and does not paint `P2WSH`.
#[test]
fn address_screen_paints_p2wsh_badge() {
    let address = "bc1qjasf9z3h7w3jspkhtgatgpyvvzgpa2wwd2lr0eh5tx44reyn2k7sfc27a4";
    let rest = address
        .strip_prefix("bc1q")
        .expect("longer mainnet address starts with bc1q");
    assert_eq!(rest.len(), 58, "{address}");
    assert!(
        rest.bytes().all(|byte| {
            matches!(
                byte,
                b'a' | b'c'..=b'h' | b'j'..=b'n' | b'p'..=b'z' | b'0' | b'2'..=b'9'
            )
        }),
        "longer bc1q suffix is not bech32: {address}"
    );
    let stats = address_stats(address);
    assert!(
        !stats.contains("\"type\""),
        "address JSON must not carry a type field: {stats}"
    );
    let painted = paint_address(address, &stats);
    assert!(
        painted.lines().any(|line| line == "Type P2WSH"),
        "Type did not sit beside the P2WSH badge: {painted}"
    );
    assert_eq!(
        painted.matches("Type P2WSH").count(),
        1,
        "P2WSH badge was not painted once: {painted}"
    );
    assert!(
        !painted.contains("V0_P2WSH"),
        "address badge was uppercased into the output type: {painted}"
    );
    assert!(
        !painted.contains("P2WPKH"),
        "the longer bc1q address painted P2WPKH: {painted}"
    );

    let short = "bc1q8yj0herd4r4yxszw3nkfvt53433thk0f5qst4g";
    let short_rest = short
        .strip_prefix("bc1q")
        .expect("shorter mainnet address starts with bc1q");
    assert_eq!(short_rest.len(), 38, "{short}");
    let short_stats = address_stats(short);
    assert!(
        !short_stats.contains("\"type\""),
        "address JSON must not carry a type field: {short_stats}"
    );
    let short_painted = paint_address(short, &short_stats);
    assert!(
        short_painted.lines().any(|line| line == "Type P2WPKH"),
        "Type did not sit beside the P2WPKH badge: {short_painted}"
    );
    assert!(
        !short_painted.contains("P2WSH"),
        "the shorter bc1q address painted P2WSH: {short_painted}"
    );
    assert!(
        !short_painted.contains("V0_P2WSH"),
        "the shorter bc1q address painted V0_P2WSH: {short_painted}"
    );
}

/// Mainnet bech32m pay-to-taproot paints the badge `P2TR` beside the row
/// label `Type`. The address starts with `bc1` and then `p` plus 58 bech32
/// characters. The address JSON has no type field. The badge is the address
/// string, not the label `P2TR tapscript`, and not an uppercased output type.
/// A `bc1q` pay-to-witness-pubkey-hash address stays `Type P2WPKH` and does
/// not paint `P2TR`. A longer `bc1q` pay-to-witness-script-hash address stays
/// `Type P2WSH` and does not paint `P2TR`.
#[test]
fn address_screen_paints_p2tr_badge() {
    let address = "bc1pahzjr2vnfqnqwl0h8jps2aa4zqe29wqxn2nagvd5a2z982k7ttdq8ry370";
    let rest = address
        .strip_prefix("bc1p")
        .expect("longer mainnet address starts with bc1p");
    assert_eq!(rest.len(), 58, "{address}");
    assert!(
        rest.bytes().all(|byte| {
            matches!(
                byte,
                b'a' | b'c'..=b'h' | b'j'..=b'n' | b'p'..=b'z' | b'0' | b'2'..=b'9'
            )
        }),
        "longer bc1p suffix is not bech32: {address}"
    );
    let stats = address_stats(address);
    assert!(
        !stats.contains("\"type\""),
        "address JSON must not carry a type field: {stats}"
    );
    let painted = paint_address(address, &stats);
    assert!(
        painted.lines().any(|line| line == "Type P2TR"),
        "Type did not sit beside the P2TR badge: {painted}"
    );
    assert_eq!(
        painted.matches("Type P2TR").count(),
        1,
        "P2TR badge was not painted once: {painted}"
    );
    assert!(
        !painted.contains("P2TR tapscript"),
        "the address badge was the tapscript label: {painted}"
    );
    assert!(
        !painted.contains("V1_P2TR"),
        "address badge was uppercased into the output type: {painted}"
    );
    assert!(
        !painted.contains("P2WPKH"),
        "the longer bc1p address painted P2WPKH: {painted}"
    );
    assert!(
        !painted.contains("P2WSH"),
        "the longer bc1p address painted P2WSH: {painted}"
    );

    let short = "bc1q8yj0herd4r4yxszw3nkfvt53433thk0f5qst4g";
    let short_rest = short
        .strip_prefix("bc1q")
        .expect("shorter mainnet address starts with bc1q");
    assert_eq!(short_rest.len(), 38, "{short}");
    let short_stats = address_stats(short);
    assert!(
        !short_stats.contains("\"type\""),
        "address JSON must not carry a type field: {short_stats}"
    );
    let short_painted = paint_address(short, &short_stats);
    assert!(
        short_painted.lines().any(|line| line == "Type P2WPKH"),
        "Type did not sit beside the P2WPKH badge: {short_painted}"
    );
    assert!(
        !short_painted.contains("P2TR"),
        "the shorter bc1q address painted P2TR: {short_painted}"
    );

    let script = "bc1qjasf9z3h7w3jspkhtgatgpyvvzgpa2wwd2lr0eh5tx44reyn2k7sfc27a4";
    let script_rest = script
        .strip_prefix("bc1q")
        .expect("longer mainnet address starts with bc1q");
    assert_eq!(script_rest.len(), 58, "{script}");
    let script_stats = address_stats(script);
    assert!(
        !script_stats.contains("\"type\""),
        "address JSON must not carry a type field: {script_stats}"
    );
    let script_painted = paint_address(script, &script_stats);
    assert!(
        script_painted.lines().any(|line| line == "Type P2WSH"),
        "Type did not sit beside the P2WSH badge: {script_painted}"
    );
    assert!(
        !script_painted.contains("P2TR"),
        "the longer bc1q address painted P2TR: {script_painted}"
    );
}

/// Mainnet base58 pay-to-pubkey-hash paints the badge `P2PKH` beside the
/// row label `Type`. The address is the Cypress fixture
/// `1stAprEZapjCYGUACUcXohQqSHF9MU5Kj` (`scriptpubkey_type` `p2pkh`).
/// Its first character is `1`. `detectAddressType` returns `p2pkh`.
/// Angular has no dedicated p2pkh case. The default arm uppercases the
/// type, so the visible text is `P2PKH`. The address JSON has no type
/// field. This is not an output type line. A `bc1q`
/// pay-to-witness-pubkey-hash address stays `Type P2WPKH` and does not
/// paint `P2PKH`. A longer `bc1q` pay-to-witness-script-hash address stays
/// `Type P2WSH` and does not paint `P2PKH`. A `bc1p` pay-to-taproot
/// address stays `Type P2TR` and does not paint `P2PKH`. A mainnet
/// address that starts with `3` paints `Type P2SH` once and does not
/// paint `P2PKH`. A failed address
/// load keeps `Error loading address data.` and does not show this badge.
/// The indexer body `Invalid Bitcoin address` stays.
#[test]
fn address_screen_paints_p2pkh_badge() {
    let address = "1stAprEZapjCYGUACUcXohQqSHF9MU5Kj";
    let rest = address
        .strip_prefix('1')
        .expect("mainnet p2pkh address starts with 1");
    assert_eq!(rest.len(), 32, "{address}");
    assert!(
        rest.bytes().all(|byte| {
            matches!(
                byte,
                b'1'..=b'9'
                    | b'A'..=b'H'
                    | b'J'..=b'N'
                    | b'P'..=b'Z'
                    | b'a'..=b'k'
                    | b'm'..=b'z'
            )
        }),
        "p2pkh remainder is not base58: {address}"
    );
    let stats = address_stats(address);
    assert!(
        !stats.contains("\"type\""),
        "address JSON must not carry a type field: {stats}"
    );
    let loaded = paint_address(address, &stats);
    assert!(
        loaded.lines().any(|line| line == "Type P2PKH"),
        "Type did not sit beside the P2PKH badge: {loaded}"
    );
    assert_eq!(
        loaded.matches("Type P2PKH").count(),
        1,
        "P2PKH badge was not painted once: {loaded}"
    );
    assert!(
        !loaded.contains("Error loading address data."),
        "a parsed address that returns stats showed the load error: {loaded}"
    );
    for absent in ["script hex", "script asm", "witness", "tooltip"] {
        assert!(
            !loaded.contains(absent),
            "{absent} was painted on the p2pkh address: {loaded}"
        );
    }

    let short = "bc1q8yj0herd4r4yxszw3nkfvt53433thk0f5qst4g";
    let short_rest = short
        .strip_prefix("bc1q")
        .expect("shorter mainnet address starts with bc1q");
    assert_eq!(short_rest.len(), 38, "{short}");
    let short_stats = address_stats(short);
    assert!(
        !short_stats.contains("\"type\""),
        "address JSON must not carry a type field: {short_stats}"
    );
    let short_painted = paint_address(short, &short_stats);
    assert!(
        short_painted.lines().any(|line| line == "Type P2WPKH"),
        "Type did not sit beside the P2WPKH badge: {short_painted}"
    );
    assert!(
        !short_painted.contains("P2PKH"),
        "the shorter bc1q address painted P2PKH: {short_painted}"
    );

    let script = "bc1qjasf9z3h7w3jspkhtgatgpyvvzgpa2wwd2lr0eh5tx44reyn2k7sfc27a4";
    let script_rest = script
        .strip_prefix("bc1q")
        .expect("longer mainnet address starts with bc1q");
    assert_eq!(script_rest.len(), 58, "{script}");
    let script_stats = address_stats(script);
    assert!(
        !script_stats.contains("\"type\""),
        "address JSON must not carry a type field: {script_stats}"
    );
    let script_painted = paint_address(script, &script_stats);
    assert!(
        script_painted.lines().any(|line| line == "Type P2WSH"),
        "Type did not sit beside the P2WSH badge: {script_painted}"
    );
    assert!(
        !script_painted.contains("P2PKH"),
        "the longer bc1q address painted P2PKH: {script_painted}"
    );

    let taproot = "bc1pahzjr2vnfqnqwl0h8jps2aa4zqe29wqxn2nagvd5a2z982k7ttdq8ry370";
    let taproot_rest = taproot
        .strip_prefix("bc1p")
        .expect("longer mainnet address starts with bc1p");
    assert_eq!(taproot_rest.len(), 58, "{taproot}");
    let taproot_stats = address_stats(taproot);
    assert!(
        !taproot_stats.contains("\"type\""),
        "address JSON must not carry a type field: {taproot_stats}"
    );
    let taproot_painted = paint_address(taproot, &taproot_stats);
    assert!(
        taproot_painted.lines().any(|line| line == "Type P2TR"),
        "Type did not sit beside the P2TR badge: {taproot_painted}"
    );
    assert!(
        !taproot_painted.contains("P2PKH"),
        "the longer bc1p address painted P2PKH: {taproot_painted}"
    );

    let nested = "3Bmb9Jig8A5kHdDSxvDZ6eryj3AXd3swuJ";
    assert!(nested.starts_with('3'), "{nested}");
    let nested_rest = nested
        .strip_prefix('3')
        .expect("mainnet address starts with 3");
    assert_eq!(nested_rest.len(), 33, "{nested}");
    assert!(
        nested_rest.bytes().all(|byte| {
            matches!(
                byte,
                b'1'..=b'9'
                    | b'A'..=b'H'
                    | b'J'..=b'N'
                    | b'P'..=b'Z'
                    | b'a'..=b'k'
                    | b'm'..=b'z'
            )
        }),
        "address that starts with 3 is not base58: {nested}"
    );
    let nested_stats = address_stats(nested);
    assert!(
        !nested_stats.contains("\"type\""),
        "address JSON must not carry a type field: {nested_stats}"
    );
    let nested_painted = paint_address(nested, &nested_stats);
    assert!(
        !nested_painted.contains("P2PKH"),
        "a mainnet address that starts with 3 painted P2PKH: {nested_painted}"
    );
    assert_eq!(
        nested_painted
            .lines()
            .filter(|line| *line == "Type P2SH")
            .count(),
        1,
        "Type did not sit beside the P2SH badge once: {nested_painted}"
    );

    let (mut host, rec) = host_with(
        Network::Mainnet,
        BTreeMap::from([(
            format!("/address/{address}"),
            "Invalid Bitcoin address".to_string(),
        )]),
    );
    let err = host
        .open_address(address)
        .expect_err("invalid address must fail");
    let paths = rec.gets();
    assert!(
        paths
            .iter()
            .any(|path| path == &format!("/address/{address}")),
        "address load did not GET /address/{address}: {paths:?}"
    );
    assert!(
        paths
            .iter()
            .all(|path| !path.contains('?') && !path.contains('#')),
        "address load used a query string: {paths:?}"
    );
    let failed = painted(&host);
    assert!(
        failed
            .lines()
            .any(|line| line == "Error loading address data."),
        "Error loading address data. is absent: {failed}"
    );
    assert!(
        err.to_string().contains("Invalid Bitcoin address"),
        "indexer body left the error: {err}"
    );
    assert!(
        failed.lines().any(|line| line == "Invalid Bitcoin address"),
        "Invalid Bitcoin address was replaced or dropped: {failed}"
    );
    assert!(
        !failed.contains("P2PKH"),
        "a failed address load showed the P2PKH badge: {failed}"
    );
    assert!(
        !failed.lines().any(|line| line.starts_with("Type ")),
        "a failed address load painted a type badge: {failed}"
    );
}

/// Mainnet base58 pay-to-script-hash paints the badge `P2SH` beside the
/// row label `Type` when no spend has rewritten the type. The address is
/// the Cypress pools fixture `3Bmb9Jig8A5kHdDSxvDZ6eryj3AXd3swuJ`. Its
/// first character is `3`. The remainder is 26 to 34 base58 characters.
/// `detectAddressType` returns `p2sh`. Angular has no dedicated p2sh case.
/// The default arm uppercases the type, so the visible text is `P2SH`.
/// The address JSON has no type field. This is the address Type line, not
/// an output type line. The letters `P2SH` also appear inside
/// `P2SH-P2WPKH`, so the check is the exact line. A `bc1q`
/// pay-to-witness-pubkey-hash address stays `Type P2WPKH` and does not
/// paint `Type P2SH`. A longer `bc1q` pay-to-witness-script-hash address
/// stays `Type P2WSH` and does not paint `Type P2SH`. A `bc1p`
/// pay-to-taproot address stays `Type P2TR` and does not paint `Type P2SH`.
/// A leading `1` address stays `Type P2PKH` and does not paint `Type P2SH`.
/// A failed address load keeps `Error loading address data.` and does not
/// show this badge. The indexer body `Invalid Bitcoin address` stays.
#[test]
fn address_screen_paints_p2sh_badge() {
    let address = "3Bmb9Jig8A5kHdDSxvDZ6eryj3AXd3swuJ";
    let rest = address
        .strip_prefix('3')
        .expect("mainnet p2sh address starts with 3");
    assert!(
        (26..=34).contains(&rest.len()),
        "p2sh remainder length is not 26 to 34: {address}"
    );
    assert_eq!(rest.len(), 33, "{address}");
    assert!(
        rest.bytes().all(|byte| {
            matches!(
                byte,
                b'1'..=b'9'
                    | b'A'..=b'H'
                    | b'J'..=b'N'
                    | b'P'..=b'Z'
                    | b'a'..=b'k'
                    | b'm'..=b'z'
            )
        }),
        "p2sh remainder is not base58: {address}"
    );
    let stats = address_stats(address);
    assert!(
        !stats.contains("\"type\""),
        "address JSON must not carry a type field: {stats}"
    );
    let loaded = paint_address(address, &stats);
    assert!(
        loaded.lines().any(|line| line == "Type P2SH"),
        "Type did not sit beside the P2SH badge: {loaded}"
    );
    assert_eq!(
        loaded.lines().filter(|line| *line == "Type P2SH").count(),
        1,
        "P2SH badge was not painted once: {loaded}"
    );
    assert!(
        !loaded.contains("P2SH-P2WPKH"),
        "the no-spend page painted P2SH-P2WPKH: {loaded}"
    );
    assert!(
        !loaded.contains("P2SH-P2WSH"),
        "the no-spend page painted P2SH-P2WSH: {loaded}"
    );
    assert!(
        !loaded.contains("Error loading address data."),
        "a parsed address that returns stats showed the load error: {loaded}"
    );
    for absent in ["script hex", "script asm", "witness", "tooltip"] {
        assert!(
            !loaded.contains(absent),
            "{absent} was painted on the p2sh address: {loaded}"
        );
    }

    let short = "bc1q8yj0herd4r4yxszw3nkfvt53433thk0f5qst4g";
    let short_rest = short
        .strip_prefix("bc1q")
        .expect("shorter mainnet address starts with bc1q");
    assert_eq!(short_rest.len(), 38, "{short}");
    let short_stats = address_stats(short);
    assert!(
        !short_stats.contains("\"type\""),
        "address JSON must not carry a type field: {short_stats}"
    );
    let short_painted = paint_address(short, &short_stats);
    assert!(
        short_painted.lines().any(|line| line == "Type P2WPKH"),
        "Type did not sit beside the P2WPKH badge: {short_painted}"
    );
    assert!(
        !short_painted.lines().any(|line| line == "Type P2SH"),
        "the shorter bc1q address painted Type P2SH: {short_painted}"
    );

    let script = "bc1qjasf9z3h7w3jspkhtgatgpyvvzgpa2wwd2lr0eh5tx44reyn2k7sfc27a4";
    let script_rest = script
        .strip_prefix("bc1q")
        .expect("longer mainnet address starts with bc1q");
    assert_eq!(script_rest.len(), 58, "{script}");
    let script_stats = address_stats(script);
    assert!(
        !script_stats.contains("\"type\""),
        "address JSON must not carry a type field: {script_stats}"
    );
    let script_painted = paint_address(script, &script_stats);
    assert!(
        script_painted.lines().any(|line| line == "Type P2WSH"),
        "Type did not sit beside the P2WSH badge: {script_painted}"
    );
    assert!(
        !script_painted.lines().any(|line| line == "Type P2SH"),
        "the longer bc1q address painted Type P2SH: {script_painted}"
    );

    let taproot = "bc1pahzjr2vnfqnqwl0h8jps2aa4zqe29wqxn2nagvd5a2z982k7ttdq8ry370";
    let taproot_rest = taproot
        .strip_prefix("bc1p")
        .expect("longer mainnet address starts with bc1p");
    assert_eq!(taproot_rest.len(), 58, "{taproot}");
    let taproot_stats = address_stats(taproot);
    assert!(
        !taproot_stats.contains("\"type\""),
        "address JSON must not carry a type field: {taproot_stats}"
    );
    let taproot_painted = paint_address(taproot, &taproot_stats);
    assert!(
        taproot_painted.lines().any(|line| line == "Type P2TR"),
        "Type did not sit beside the P2TR badge: {taproot_painted}"
    );
    assert!(
        !taproot_painted.lines().any(|line| line == "Type P2SH"),
        "the longer bc1p address painted Type P2SH: {taproot_painted}"
    );

    let pubkey = "1stAprEZapjCYGUACUcXohQqSHF9MU5Kj";
    let pubkey_rest = pubkey
        .strip_prefix('1')
        .expect("mainnet p2pkh address starts with 1");
    assert_eq!(pubkey_rest.len(), 32, "{pubkey}");
    let pubkey_stats = address_stats(pubkey);
    assert!(
        !pubkey_stats.contains("\"type\""),
        "address JSON must not carry a type field: {pubkey_stats}"
    );
    let pubkey_painted = paint_address(pubkey, &pubkey_stats);
    assert!(
        pubkey_painted.lines().any(|line| line == "Type P2PKH"),
        "Type did not sit beside the P2PKH badge: {pubkey_painted}"
    );
    assert!(
        !pubkey_painted.lines().any(|line| line == "Type P2SH"),
        "the p2pkh address painted Type P2SH: {pubkey_painted}"
    );

    let (mut host, rec) = host_with(
        Network::Mainnet,
        BTreeMap::from([(
            format!("/address/{address}"),
            "Invalid Bitcoin address".to_string(),
        )]),
    );
    let err = host
        .open_address(address)
        .expect_err("invalid address must fail");
    let paths = rec.gets();
    assert!(
        paths
            .iter()
            .any(|path| path == &format!("/address/{address}")),
        "address load did not GET /address/{address}: {paths:?}"
    );
    assert!(
        paths
            .iter()
            .all(|path| !path.contains('?') && !path.contains('#')),
        "address load used a query string: {paths:?}"
    );
    let failed = painted(&host);
    assert!(
        failed
            .lines()
            .any(|line| line == "Error loading address data."),
        "Error loading address data. is absent: {failed}"
    );
    assert!(
        err.to_string().contains("Invalid Bitcoin address"),
        "indexer body left the error: {err}"
    );
    assert!(
        failed.lines().any(|line| line == "Invalid Bitcoin address"),
        "Invalid Bitcoin address was replaced or dropped: {failed}"
    );
    assert!(
        !failed.lines().any(|line| line == "Type P2SH"),
        "a failed address load showed the P2SH badge: {failed}"
    );
    assert!(
        !failed.lines().any(|line| line.starts_with("Type ")),
        "a failed address load painted a type badge: {failed}"
    );
}

/// Scriptsig from the Angular `transactions-random.json` p2sh-p2wpkh spend.
const P2SH_P2WPKH_SCRIPTSIG: &str = "1600149b27f072e4b972927c445d1946162a550b0914d8";
const P2SH_P2WPKH_WITNESS_0: &str = "3040021c23902a01d4c5cff2c33c8bdb778a5aadea78a9a0d6d4db60aaa0fba1022069237d9dbf2db8cff9c260ba71250493682d01a746f4a45c5c7ea386e56d2bc902";
const P2SH_P2WPKH_WITNESS_1: &str =
    "0240187acd3e2fd3d8e1acffefa85907b6550730c24f78dfd3301c829fc4daf3cc";
/// Scriptsig from the same fixture's p2sh-p2wsh spend.
/// Address `3GcrZrbUuvE4UtUdSbKTXcRnTqmfMdyMAC` in `transactions-random.json`.
/// A non-empty witness paints `Type P2SH-P2WSH`. An empty witness does not.
const P2SH_P2WSH_SCRIPTSIG: &str =
    "220020a18160de7291554f349c7d5cbee4ab97fb542e94cf302ce8d7e9747e4188ca75";
const P2SH_P2WSH_ADDRESS: &str = "3GcrZrbUuvE4UtUdSbKTXcRnTqmfMdyMAC";
const P2SH_P2WSH_WITNESS_0: &str = "303f021c65aee6696e80be6e14545cfd64b44f17b0514c150eefdb090c0f0bd9021f3fef4aa95c252a225622aba99e4d5af5a6fe40d177acd593e64cf2f8557ccc03";
const P2SH_P2WSH_WITNESS_1: &str =
    "03b55c6f0749e0f3e2caeca05f68e3699f1b3c62a550730f704985a6a9aae437a1";
const P2SH_P2WSH_WITNESS_2: &str = "76a914db865fd920959506111079995f1e4017b489bfe38763ac6721024d560f7f5d28aae5e1a8aa2b7ba615d7fc48e4ea27e5d27336e6a8f5fa0f5c8c7c820120876475527c2103443e8834fa7d79d7b5e95e0e9d0847f6b03ac3ea977979858b4104947fca87ca52ae67a91446c3747322b220fdb925c9802f0e949c1feab99988ac6868";
/// Address and 2-of-3 redeem script from the Angular track-address sample.
const MULTISIG_2_OF_3_ADDRESS: &str = "3L4YUynB4X44rJBY9CmiLMN8Wjti49JCYB";
const MULTISIG_2_OF_3_REDEEM: &str = "OP_PUSHNUM_2 OP_PUSHBYTES_33 027f2a0df8e86535d08ca3e766a178f90c813d2dd1d55b0166e82518efbffb18c0 OP_PUSHBYTES_33 03556a35844b517e2fc8216701e2e0a64dbcbe62ad420ac6dd73dc79e69efeb695 OP_PUSHBYTES_33 031ef21bd55171032b7aec21ec82932735fb986f1d4d8611feee62ab38acf4a6bb OP_PUSHNUM_3 OP_CHECKMULTISIG";
const MULTISIG_2_OF_3_SCRIPTSIG: &str = "0048304502210099219ee0cd5da341650078e3c63885b3cc2211069f2551cf436e0100f421e1760220349b4ec284255b458d6da539fa17314e8330459e0a653c254f775d4ec8f32b3f0147304402203a8353c5ee76a2e266432e5f993f882e05725297e64c0833cf44719f7dda8d3b022058a2f72e7739efd21657b4943cac60a0a3c749e712787f0e85726da4c3adcf8e014c695221027f2a0df8e86535d08ca3e766a178f90c813d2dd1d55b0166e82518efbffb18c02103556a35844b517e2fc8216701e2e0a64dbcbe62ad420ac6dd73dc79e69efeb69521031ef21bd55171032b7aec21ec82932735fb986f1d4d8611feee62ab38acf4a6bb53ae";
const MULTISIG_2_OF_3_PUBKEY: &str =
    "027f2a0df8e86535d08ca3e766a178f90c813d2dd1d55b0166e82518efbffb18c0";
/// Wrapped-script redeem from the same Angular fixture. It is not 2-of-3.
const NOT_2_OF_3_REDEEM: &str =
    "OP_0 OP_PUSHBYTES_32 a18160de7291554f349c7d5cbee4ab97fb542e94cf302ce8d7e9747e4188ca75";
/// Lightning witness script from `transactions-random.json`. It is not 2-of-3.
const NOT_MULTISIG_REDEEM: &str = "OP_DUP OP_HASH160 OP_PUSHBYTES_20 db865fd920959506111079995f1e4017b489bfe3 OP_EQUAL OP_IF OP_CHECKSIG OP_ELSE OP_PUSHBYTES_33 024d560f7f5d28aae5e1a8aa2b7ba615d7fc48e4ea27e5d27336e6a8f5fa0f5c8c OP_SWAP OP_SIZE OP_PUSHBYTES_1 20 OP_EQUAL OP_NOTIF OP_DROP OP_PUSHNUM_2 OP_SWAP OP_PUSHBYTES_33 03443e8834fa7d79d7b5e95e0e9d0847f6b03ac3ea977979858b4104947fca87ca OP_PUSHNUM_2 OP_CHECKMULTISIG OP_ELSE OP_HASH160 OP_PUSHBYTES_20 46c3747322b220fdb925c9802f0e949c1feab999 OP_EQUALVERIFY OP_CHECKSIG OP_ENDIF OP_ENDIF";

fn p2sh_address_txs(
    prevout_address: &str,
    scriptsig: &str,
    scriptsig_asm: &str,
    witness: &str,
) -> String {
    address_txs_with_redeem(
        prevout_address,
        "p2sh",
        scriptsig,
        scriptsig_asm,
        witness,
        None,
    )
}

fn address_txs_with_redeem(
    prevout_address: &str,
    prevout_type: &str,
    scriptsig: &str,
    scriptsig_asm: &str,
    witness: &str,
    redeem_asm: Option<&str>,
) -> String {
    let redeem = match redeem_asm {
        Some(asm) => format!(r#","inner_redeemscript_asm":"{asm}""#),
        None => String::new(),
    };
    format!(
        r#"[{{"txid":"b10c0000004da5a9d1d9b4ae32e09f0b3e62d21a5cce5428d4ad714fb444eb5d","version":1,"locktime":1231006505,"vin":[{{"txid":"cb9b47ac04023b29fb633a8ef04af351ac9fd74c57c9a2163f683516274767e3","vout":2,"prevout":{{"scriptpubkey":"a9143b13a1f71c20c799d86bb624b3898c826d6c82da87","scriptpubkey_asm":"OP_HASH160 OP_PUSHBYTES_20 3b13a1f71c20c799d86bb624b3898c826d6c82da OP_EQUAL","scriptpubkey_type":"{prevout_type}","scriptpubkey_address":"{prevout_address}","value":5139}},"scriptsig":"{scriptsig}","scriptsig_asm":"{scriptsig_asm}","witness":{witness},"is_coinbase":false,"sequence":141{redeem}}}],"vout":[],"size":10,"weight":40,"sigops":0,"fee":1,"status":{{"confirmed":true,"block_height":1}}}}]"#
    )
}

fn assert_mainnet_script_address(address: &str) {
    assert!(address.starts_with('3'), "{address}");
    let rest = address
        .strip_prefix('3')
        .expect("mainnet address starts with 3");
    assert!(
        (26..=34).contains(&rest.len()),
        "p2sh remainder length is not 26 to 34: {address}"
    );
    assert!(
        rest.bytes().all(|byte| {
            matches!(
                byte,
                b'1'..=b'9'
                    | b'A'..=b'H'
                    | b'J'..=b'N'
                    | b'P'..=b'Z'
                    | b'a'..=b'k'
                    | b'm'..=b'z'
            )
        }),
        "p2sh remainder is not base58: {address}"
    );
}

fn assert_no_type_badge(painted: &str) {
    assert!(
        !painted.contains("P2SH-P2WPKH"),
        "P2SH-P2WPKH was painted: {painted}"
    );
    assert!(
        !painted.contains("P2SH-P2WSH"),
        "P2SH-P2WSH was painted: {painted}"
    );
    assert!(
        !painted.lines().any(|line| line == "Type P2SH"),
        "a plain Type P2SH line was painted: {painted}"
    );
    assert!(
        !painted.lines().any(|line| line.starts_with("Type ")),
        "a type badge was painted: {painted}"
    );
}

/// The type was not rewritten, so the address line is `Type P2SH` once.
/// `P2SH-P2WPKH` contains the letters `P2SH`, so the check is the exact line.
fn assert_plain_p2sh_once(painted: &str) {
    assert_eq!(
        painted.lines().filter(|line| *line == "Type P2SH").count(),
        1,
        "Type did not sit beside the P2SH badge once: {painted}"
    );
    assert!(
        !painted
            .lines()
            .any(|line| line.starts_with("Type ") && line != "Type P2SH"),
        "another type badge was painted: {painted}"
    );
    assert!(
        !painted.contains("P2SH-P2WPKH"),
        "P2SH-P2WPKH was painted: {painted}"
    );
    assert!(
        !painted.contains("P2SH-P2WSH"),
        "P2SH-P2WSH was painted: {painted}"
    );
}

/// Mainnet pay-to-script-hash that spends as wrapped pay-to-witness-pubkey-hash
/// paints the badge `P2SH-P2WPKH` beside the row label `Type`. The address is
/// the Angular fixture `375PJxsKRtAq4WoS6u82jvgZW94R8Wx3iH`. Its first character
/// is `3`. `detectAddressType` starts it as `p2sh`. `processInputs` changes the
/// type to `p2sh-p2wpkh` when a vin spending this address has a non-empty
/// witness and a scriptsig hex string that starts with `160014`. Angular has
/// no dedicated case. The default arm uppercases the type, so the visible text
/// is `P2SH-P2WPKH`. The address JSON has no type field. The Type row appears
/// only after the no-query `GET /address/:script/txs` succeeds. This is not
/// `Type P2SH` and not `P2SH-P2WSH`. The scriptsig and the witness stay off
/// the screen. A `bc1q` pay-to-witness-pubkey-hash address stays `Type P2WPKH`.
/// A longer `bc1q` address stays `Type P2WSH`. A `bc1p` address stays `Type P2TR`.
/// A leading `1` address stays `Type P2PKH`. None of those paint `P2SH-P2WPKH`.
/// A mainnet address that starts with `3` and has no matching spend does not
/// paint this badge. A spend whose scriptsig starts with `160014` and whose
/// witness is empty does not paint it. A spend with a witness whose scriptsig
/// does not start with `160014` does not paint it. That `220020` spend paints
/// `Type P2SH-P2WSH` once. An input that does not
/// spend this address does not paint it. A failed address load keeps
/// `Error loading address data.` and `Invalid Bitcoin address`.
#[test]
fn address_screen_paints_p2sh_p2wpkh_badge() {
    let address = "375PJxsKRtAq4WoS6u82jvgZW94R8Wx3iH";
    assert_mainnet_script_address(address);
    let witness = format!(r#"["{P2SH_P2WPKH_WITNESS_0}","{P2SH_P2WPKH_WITNESS_1}"]"#);
    let txs = p2sh_address_txs(
        address,
        P2SH_P2WPKH_SCRIPTSIG,
        "OP_PUSHBYTES_22 00149b27f072e4b972927c445d1946162a550b0914d8",
        &witness,
    );
    assert!(
        txs.contains(P2SH_P2WPKH_SCRIPTSIG) && txs.contains(P2SH_P2WPKH_WITNESS_0),
        "the fixture spend left the transaction list"
    );
    let stats = loaded_address_stats(address, 1, 0);
    assert!(
        !stats.contains("\"type\""),
        "address JSON must not carry a type field: {stats}"
    );
    let loaded = paint_address_loaded(address, 1, 0, &txs);
    assert!(
        loaded.lines().any(|line| line == "Type P2SH-P2WPKH"),
        "Type did not sit beside the P2SH-P2WPKH badge: {loaded}"
    );
    assert_eq!(
        loaded.matches("Type P2SH-P2WPKH").count(),
        1,
        "P2SH-P2WPKH badge was not painted once: {loaded}"
    );
    assert!(
        !loaded.lines().any(|line| line == "Type P2SH"),
        "a plain Type P2SH line was painted: {loaded}"
    );
    assert!(
        !loaded.contains("P2SH-P2WSH"),
        "the wrapped pubkey spend painted P2SH-P2WSH: {loaded}"
    );
    assert!(
        !loaded.contains("Error loading address data."),
        "a parsed address that returns stats showed the load error: {loaded}"
    );
    for absent in [
        P2SH_P2WPKH_SCRIPTSIG,
        P2SH_P2WPKH_WITNESS_0,
        P2SH_P2WPKH_WITNESS_1,
        "script hex",
        "script asm",
        "witness",
        "tooltip",
    ] {
        assert!(
            !loaded.contains(absent),
            "{absent} was painted on the p2sh-p2wpkh address: {loaded}"
        );
    }

    let short = "bc1q8yj0herd4r4yxszw3nkfvt53433thk0f5qst4g";
    let short_painted = paint_address(short, &address_stats(short));
    assert!(
        short_painted.lines().any(|line| line == "Type P2WPKH"),
        "Type did not sit beside the P2WPKH badge: {short_painted}"
    );
    assert!(
        !short_painted.contains("P2SH-P2WPKH"),
        "the shorter bc1q address painted P2SH-P2WPKH: {short_painted}"
    );

    let script = "bc1qjasf9z3h7w3jspkhtgatgpyvvzgpa2wwd2lr0eh5tx44reyn2k7sfc27a4";
    let script_painted = paint_address(script, &address_stats(script));
    assert!(
        script_painted.lines().any(|line| line == "Type P2WSH"),
        "Type did not sit beside the P2WSH badge: {script_painted}"
    );
    assert!(
        !script_painted.contains("P2SH-P2WPKH"),
        "the longer bc1q address painted P2SH-P2WPKH: {script_painted}"
    );

    let taproot = "bc1pahzjr2vnfqnqwl0h8jps2aa4zqe29wqxn2nagvd5a2z982k7ttdq8ry370";
    let taproot_painted = paint_address(taproot, &address_stats(taproot));
    assert!(
        taproot_painted.lines().any(|line| line == "Type P2TR"),
        "Type did not sit beside the P2TR badge: {taproot_painted}"
    );
    assert!(
        !taproot_painted.contains("P2SH-P2WPKH"),
        "the longer bc1p address painted P2SH-P2WPKH: {taproot_painted}"
    );

    let pubkey = "1stAprEZapjCYGUACUcXohQqSHF9MU5Kj";
    let pubkey_painted = paint_address(pubkey, &address_stats(pubkey));
    assert!(
        pubkey_painted.lines().any(|line| line == "Type P2PKH"),
        "Type did not sit beside the P2PKH badge: {pubkey_painted}"
    );
    assert!(
        !pubkey_painted.contains("P2SH-P2WPKH"),
        "the p2pkh address painted P2SH-P2WPKH: {pubkey_painted}"
    );

    let nested = "3Bmb9Jig8A5kHdDSxvDZ6eryj3AXd3swuJ";
    assert_mainnet_script_address(nested);
    let nested_painted = paint_address(nested, &address_stats(nested));
    assert_plain_p2sh_once(&nested_painted);

    let idle = paint_address(address, &address_stats(address));
    assert_plain_p2sh_once(&idle);

    let empty_witness = p2sh_address_txs(
        address,
        P2SH_P2WPKH_SCRIPTSIG,
        "OP_PUSHBYTES_22 00149b27f072e4b972927c445d1946162a550b0914d8",
        "[]",
    );
    let empty_witness_painted = paint_address_loaded(address, 1, 0, &empty_witness);
    assert_plain_p2sh_once(&empty_witness_painted);

    let other = "3GcrZrbUuvE4UtUdSbKTXcRnTqmfMdyMAC";
    assert_mainnet_script_address(other);
    let other_witness = r#"["303f021c65aee6696e80be6e14545cfd64b44f17b0514c150eefdb090c0f0bd9021f3fef4aa95c252a225622aba99e4d5af5a6fe40d177acd593e64cf2f8557ccc03","03b55c6f0749e0f3e2caeca05f68e3699f1b3c62a550730f704985a6a9aae437a1"]"#;
    let other_txs = p2sh_address_txs(
        other,
        P2SH_P2WSH_SCRIPTSIG,
        "OP_PUSHBYTES_34 0020a18160de7291554f349c7d5cbee4ab97fb542e94cf302ce8d7e9747e4188ca75",
        other_witness,
    );
    let other_painted = paint_address_loaded(other, 1, 0, &other_txs);
    assert!(
        other_painted.lines().any(|line| line == "Type P2SH-P2WSH"),
        "Type did not sit beside the P2SH-P2WSH badge: {other_painted}"
    );
    assert_eq!(
        other_painted.matches("Type P2SH-P2WSH").count(),
        1,
        "P2SH-P2WSH badge was not painted once: {other_painted}"
    );
    assert!(
        !other_painted.contains("P2SH-P2WPKH"),
        "the wrapped script spend painted P2SH-P2WPKH: {other_painted}"
    );
    assert!(
        !other_painted.lines().any(|line| line == "Type P2SH"),
        "a plain Type P2SH line was painted: {other_painted}"
    );
    assert!(
        !other_painted.contains(P2SH_P2WSH_SCRIPTSIG),
        "the scriptsig was painted: {other_painted}"
    );

    let foreign = p2sh_address_txs(
        nested,
        P2SH_P2WPKH_SCRIPTSIG,
        "OP_PUSHBYTES_22 00149b27f072e4b972927c445d1946162a550b0914d8",
        &witness,
    );
    let foreign_painted = paint_address_loaded(address, 1, 0, &foreign);
    assert_plain_p2sh_once(&foreign_painted);

    let (mut host, rec) = host_with(
        Network::Mainnet,
        BTreeMap::from([(
            format!("/address/{address}"),
            "Invalid Bitcoin address".to_string(),
        )]),
    );
    let err = host
        .open_address(address)
        .expect_err("invalid address must fail");
    let paths = rec.gets();
    assert!(
        paths
            .iter()
            .any(|path| path == &format!("/address/{address}")),
        "address load did not GET /address/{address}: {paths:?}"
    );
    assert!(
        paths
            .iter()
            .all(|path| !path.contains('?') && !path.contains('#')),
        "address load used a query string: {paths:?}"
    );
    let failed = painted(&host);
    assert!(
        failed
            .lines()
            .any(|line| line == "Error loading address data."),
        "Error loading address data. is absent: {failed}"
    );
    assert!(
        err.to_string().contains("Invalid Bitcoin address"),
        "indexer body left the error: {err}"
    );
    assert!(
        failed.lines().any(|line| line == "Invalid Bitcoin address"),
        "Invalid Bitcoin address was replaced or dropped: {failed}"
    );
    assert_no_type_badge(&failed);
}

/// Mainnet pay-to-script-hash that spends as wrapped pay-to-witness-script-hash
/// paints the badge `P2SH-P2WSH` beside the row label `Type`. The address is
/// the Angular fixture `3GcrZrbUuvE4UtUdSbKTXcRnTqmfMdyMAC`. Its first character
/// is `3`. `detectAddressType` starts it as `p2sh`. `processInputs` changes the
/// type to `p2sh-p2wsh` when a vin spending this address has a non-empty
/// witness and a scriptsig hex string that starts with `220020`. Angular has
/// no dedicated case. The default arm uppercases the type, so the visible text
/// is `P2SH-P2WSH`. The address JSON has no type field. The Type row appears
/// only after the no-query `GET /address/:script/txs` succeeds. This is not
/// `Type P2SH` and not `P2SH-P2WPKH`. The scriptsig and the witness stay off
/// the screen. A `160014` spend stays `Type P2SH-P2WPKH` and does not paint
/// `P2SH-P2WSH`. A `bc1q` pay-to-witness-pubkey-hash address stays `Type P2WPKH`.
/// A longer `bc1q` address stays `Type P2WSH`. A `bc1p` address stays `Type P2TR`.
/// A leading `1` address stays `Type P2PKH`. None of those paint `P2SH-P2WSH`.
/// A mainnet address that starts with `3` and has no matching spend does not
/// paint this badge. A spend whose scriptsig starts with `220020` and whose
/// witness is empty does not paint it. An input that does not spend this
/// address does not paint it. A failed address load keeps
/// `Error loading address data.` and `Invalid Bitcoin address`.
#[test]
fn address_screen_paints_p2sh_p2wsh_badge() {
    let address = P2SH_P2WSH_ADDRESS;
    assert_mainnet_script_address(address);
    assert!(
        P2SH_P2WSH_SCRIPTSIG.starts_with("220020"),
        "fixture scriptsig does not start with 220020"
    );
    let witness =
        format!(r#"["{P2SH_P2WSH_WITNESS_0}","{P2SH_P2WSH_WITNESS_1}","{P2SH_P2WSH_WITNESS_2}"]"#);
    let txs = p2sh_address_txs(
        address,
        P2SH_P2WSH_SCRIPTSIG,
        "OP_PUSHBYTES_34 0020a18160de7291554f349c7d5cbee4ab97fb542e94cf302ce8d7e9747e4188ca75",
        &witness,
    );
    assert!(
        txs.contains(P2SH_P2WSH_SCRIPTSIG) && txs.contains(P2SH_P2WSH_WITNESS_0),
        "the fixture spend left the transaction list"
    );
    let stats = loaded_address_stats(address, 1, 0);
    assert!(
        !stats.contains("\"type\""),
        "address JSON must not carry a type field: {stats}"
    );
    let loaded = paint_address_loaded(address, 1, 0, &txs);
    assert!(
        loaded.lines().any(|line| line == "Type P2SH-P2WSH"),
        "Type did not sit beside the P2SH-P2WSH badge: {loaded}"
    );
    assert_eq!(
        loaded.matches("Type P2SH-P2WSH").count(),
        1,
        "P2SH-P2WSH badge was not painted once: {loaded}"
    );
    assert!(
        !loaded.lines().any(|line| line == "Type P2SH"),
        "a plain Type P2SH line was painted: {loaded}"
    );
    assert!(
        !loaded.contains("P2SH-P2WPKH"),
        "the wrapped script spend painted P2SH-P2WPKH: {loaded}"
    );
    assert!(
        !loaded.contains("Error loading address data."),
        "a parsed address that returns stats showed the load error: {loaded}"
    );
    for absent in [
        P2SH_P2WSH_SCRIPTSIG,
        P2SH_P2WSH_WITNESS_0,
        P2SH_P2WSH_WITNESS_1,
        P2SH_P2WSH_WITNESS_2,
        "script hex",
        "script asm",
        "witness",
        "tooltip",
    ] {
        assert!(
            !loaded.contains(absent),
            "{absent} was painted on the p2sh-p2wsh address: {loaded}"
        );
    }

    let wrapped_pubkey = "375PJxsKRtAq4WoS6u82jvgZW94R8Wx3iH";
    assert_mainnet_script_address(wrapped_pubkey);
    let pubkey_witness = format!(r#"["{P2SH_P2WPKH_WITNESS_0}","{P2SH_P2WPKH_WITNESS_1}"]"#);
    let pubkey_txs = p2sh_address_txs(
        wrapped_pubkey,
        P2SH_P2WPKH_SCRIPTSIG,
        "OP_PUSHBYTES_22 00149b27f072e4b972927c445d1946162a550b0914d8",
        &pubkey_witness,
    );
    let pubkey_painted = paint_address_loaded(wrapped_pubkey, 1, 0, &pubkey_txs);
    assert!(
        pubkey_painted
            .lines()
            .any(|line| line == "Type P2SH-P2WPKH"),
        "Type did not sit beside the P2SH-P2WPKH badge: {pubkey_painted}"
    );
    assert_eq!(
        pubkey_painted.matches("Type P2SH-P2WPKH").count(),
        1,
        "P2SH-P2WPKH badge was not painted once: {pubkey_painted}"
    );
    assert!(
        !pubkey_painted.contains("P2SH-P2WSH"),
        "the wrapped pubkey spend painted P2SH-P2WSH: {pubkey_painted}"
    );
    assert!(
        !pubkey_painted.lines().any(|line| line == "Type P2SH"),
        "a plain Type P2SH line was painted: {pubkey_painted}"
    );

    let short = "bc1q8yj0herd4r4yxszw3nkfvt53433thk0f5qst4g";
    let short_painted = paint_address(short, &address_stats(short));
    assert!(
        short_painted.lines().any(|line| line == "Type P2WPKH"),
        "Type did not sit beside the P2WPKH badge: {short_painted}"
    );
    assert!(
        !short_painted.contains("P2SH-P2WSH"),
        "the shorter bc1q address painted P2SH-P2WSH: {short_painted}"
    );

    let script = "bc1qjasf9z3h7w3jspkhtgatgpyvvzgpa2wwd2lr0eh5tx44reyn2k7sfc27a4";
    let script_painted = paint_address(script, &address_stats(script));
    assert!(
        script_painted.lines().any(|line| line == "Type P2WSH"),
        "Type did not sit beside the P2WSH badge: {script_painted}"
    );
    assert!(
        !script_painted.contains("P2SH-P2WSH"),
        "the longer bc1q address painted P2SH-P2WSH: {script_painted}"
    );

    let taproot = "bc1pahzjr2vnfqnqwl0h8jps2aa4zqe29wqxn2nagvd5a2z982k7ttdq8ry370";
    let taproot_painted = paint_address(taproot, &address_stats(taproot));
    assert!(
        taproot_painted.lines().any(|line| line == "Type P2TR"),
        "Type did not sit beside the P2TR badge: {taproot_painted}"
    );
    assert!(
        !taproot_painted.contains("P2SH-P2WSH"),
        "the longer bc1p address painted P2SH-P2WSH: {taproot_painted}"
    );

    let pubkey = "1stAprEZapjCYGUACUcXohQqSHF9MU5Kj";
    let pubkey_address_painted = paint_address(pubkey, &address_stats(pubkey));
    assert!(
        pubkey_address_painted
            .lines()
            .any(|line| line == "Type P2PKH"),
        "Type did not sit beside the P2PKH badge: {pubkey_address_painted}"
    );
    assert!(
        !pubkey_address_painted.contains("P2SH-P2WSH"),
        "the p2pkh address painted P2SH-P2WSH: {pubkey_address_painted}"
    );

    let nested = "3Bmb9Jig8A5kHdDSxvDZ6eryj3AXd3swuJ";
    assert_mainnet_script_address(nested);
    let nested_painted = paint_address(nested, &address_stats(nested));
    assert_plain_p2sh_once(&nested_painted);

    let idle = paint_address(address, &address_stats(address));
    assert_plain_p2sh_once(&idle);

    let empty_witness = p2sh_address_txs(
        address,
        P2SH_P2WSH_SCRIPTSIG,
        "OP_PUSHBYTES_34 0020a18160de7291554f349c7d5cbee4ab97fb542e94cf302ce8d7e9747e4188ca75",
        "[]",
    );
    let empty_witness_painted = paint_address_loaded(address, 1, 0, &empty_witness);
    assert_plain_p2sh_once(&empty_witness_painted);

    let foreign = p2sh_address_txs(
        nested,
        P2SH_P2WSH_SCRIPTSIG,
        "OP_PUSHBYTES_34 0020a18160de7291554f349c7d5cbee4ab97fb542e94cf302ce8d7e9747e4188ca75",
        &witness,
    );
    let foreign_painted = paint_address_loaded(address, 1, 0, &foreign);
    assert_plain_p2sh_once(&foreign_painted);
    assert!(
        !foreign_painted.contains(P2SH_P2WSH_SCRIPTSIG),
        "the scriptsig was painted: {foreign_painted}"
    );

    let (mut host, rec) = host_with(
        Network::Mainnet,
        BTreeMap::from([(
            format!("/address/{address}"),
            "Invalid Bitcoin address".to_string(),
        )]),
    );
    let err = host
        .open_address(address)
        .expect_err("invalid address must fail");
    let paths = rec.gets();
    assert!(
        paths
            .iter()
            .any(|path| path == &format!("/address/{address}")),
        "address load did not GET /address/{address}: {paths:?}"
    );
    assert!(
        paths
            .iter()
            .all(|path| !path.contains('?') && !path.contains('#')),
        "address load used a query string: {paths:?}"
    );
    let failed = painted(&host);
    assert!(
        failed
            .lines()
            .any(|line| line == "Error loading address data."),
        "Error loading address data. is absent: {failed}"
    );
    assert!(
        err.to_string().contains("Invalid Bitcoin address"),
        "indexer body left the error: {err}"
    );
    assert!(
        failed.lines().any(|line| line == "Invalid Bitcoin address"),
        "Invalid Bitcoin address was replaced or dropped: {failed}"
    );
    assert_no_type_badge(&failed);
}

/// Mainnet pay-to-script-hash whose spend carries a 2-of-3 redeem script paints
/// the warning `Multisig 2 of 3`. The address and the redeem script are the
/// Angular track-address sample. The label is the word Multisig, the required
/// signature count, the word of, and the key count. The address JSON has no
/// type field. The line appears only after the no-query `GET /address/:script/txs`
/// succeeds. A `160014` spend with a witness still paints `Type P2SH-P2WPKH`
/// and does not show this warning, even when the same redeem script is present.
/// An empty witness keeps the redeem script, so that `160014` spend does show
/// the warning and does not paint `Type P2SH-P2WPKH`. A `220020` spend still
/// paints `Type P2SH-P2WSH`. A tapscript address does not show the warning.
/// A redeem script that is not 2-of-3 does not show this exact text. The asm,
/// the script hex, and the witness stay off the successful screen. A failed
/// address load keeps `Error loading address data.` and `Invalid Bitcoin address`.
#[test]
fn address_screen_paints_multisig_badge() {
    let address = MULTISIG_2_OF_3_ADDRESS;
    assert_mainnet_script_address(address);
    assert!(
        MULTISIG_2_OF_3_REDEEM.contains("OP_PUSHNUM_2")
            && MULTISIG_2_OF_3_REDEEM.contains("OP_PUSHNUM_3")
            && MULTISIG_2_OF_3_REDEEM.ends_with("OP_CHECKMULTISIG"),
        "the Angular redeem script left the fixture"
    );
    assert!(
        !MULTISIG_2_OF_3_SCRIPTSIG.starts_with("160014")
            && !MULTISIG_2_OF_3_SCRIPTSIG.starts_with("220020"),
        "the Angular scriptsig is a wrapped segwit spend"
    );
    let txs = address_txs_with_redeem(
        address,
        "p2sh",
        MULTISIG_2_OF_3_SCRIPTSIG,
        "OP_0",
        "[]",
        Some(MULTISIG_2_OF_3_REDEEM),
    );
    assert!(
        txs.contains(MULTISIG_2_OF_3_REDEEM) && txs.contains(MULTISIG_2_OF_3_PUBKEY),
        "the fixture spend left the transaction list"
    );
    let stats = loaded_address_stats(address, 1, 0);
    assert!(
        !stats.contains("\"type\""),
        "address JSON must not carry a type field: {stats}"
    );
    let loaded = paint_address_loaded(address, 1, 0, &txs);
    assert!(
        loaded.lines().any(|line| line == "Multisig 2 of 3"),
        "the screen did not show Multisig 2 of 3: {loaded}"
    );
    assert!(
        !loaded.contains("Error loading address data."),
        "a parsed address that returns stats showed the load error: {loaded}"
    );
    assert!(
        !loaded.lines().any(|line| line == "Type P2SH-P2WPKH"),
        "the plain multisig spend painted Type P2SH-P2WPKH: {loaded}"
    );
    assert!(
        !loaded.lines().any(|line| line == "Type P2SH-P2WSH"),
        "the plain multisig spend painted Type P2SH-P2WSH: {loaded}"
    );
    assert!(
        !loaded.lines().any(|line| line == "Type MULTISIG"),
        "the warning replaced the output type line: {loaded}"
    );
    for absent in [
        MULTISIG_2_OF_3_REDEEM,
        MULTISIG_2_OF_3_SCRIPTSIG,
        MULTISIG_2_OF_3_PUBKEY,
        "OP_CHECKMULTISIG",
        "OP_PUSHBYTES_33",
        "OP_PUSHNUM_2",
        "script hex",
        "script asm",
        "witness",
        "tooltip",
    ] {
        assert!(
            !loaded.contains(absent),
            "{absent} was painted on the multisig address: {loaded}"
        );
    }

    let kept = address_txs_with_redeem(
        address,
        "p2sh",
        P2SH_P2WPKH_SCRIPTSIG,
        "OP_PUSHBYTES_22 00149b27f072e4b972927c445d1946162a550b0914d8",
        "[]",
        Some(MULTISIG_2_OF_3_REDEEM),
    );
    let kept_painted = paint_address_loaded(address, 1, 0, &kept);
    assert!(
        kept_painted.lines().any(|line| line == "Multisig 2 of 3"),
        "an empty witness dropped the redeem script: {kept_painted}"
    );
    assert!(
        !kept_painted.lines().any(|line| line == "Type P2SH-P2WPKH"),
        "an empty witness painted Type P2SH-P2WPKH: {kept_painted}"
    );

    let wrapped_pubkey = "375PJxsKRtAq4WoS6u82jvgZW94R8Wx3iH";
    assert_mainnet_script_address(wrapped_pubkey);
    let pubkey_witness = format!(r#"["{P2SH_P2WPKH_WITNESS_0}","{P2SH_P2WPKH_WITNESS_1}"]"#);
    let pubkey_txs = address_txs_with_redeem(
        wrapped_pubkey,
        "p2sh",
        P2SH_P2WPKH_SCRIPTSIG,
        "OP_PUSHBYTES_22 00149b27f072e4b972927c445d1946162a550b0914d8",
        &pubkey_witness,
        Some(MULTISIG_2_OF_3_REDEEM),
    );
    let pubkey_painted = paint_address_loaded(wrapped_pubkey, 1, 0, &pubkey_txs);
    assert!(
        pubkey_painted
            .lines()
            .any(|line| line == "Type P2SH-P2WPKH"),
        "Type did not sit beside the P2SH-P2WPKH badge: {pubkey_painted}"
    );
    assert!(
        !pubkey_painted.lines().any(|line| line == "Multisig 2 of 3"),
        "the wrapped pubkey spend showed Multisig 2 of 3: {pubkey_painted}"
    );

    let wrapped_script = P2SH_P2WSH_ADDRESS;
    assert_mainnet_script_address(wrapped_script);
    let script_witness =
        format!(r#"["{P2SH_P2WSH_WITNESS_0}","{P2SH_P2WSH_WITNESS_1}","{P2SH_P2WSH_WITNESS_2}"]"#);
    let script_txs = p2sh_address_txs(
        wrapped_script,
        P2SH_P2WSH_SCRIPTSIG,
        "OP_PUSHBYTES_34 0020a18160de7291554f349c7d5cbee4ab97fb542e94cf302ce8d7e9747e4188ca75",
        &script_witness,
    );
    let script_painted = paint_address_loaded(wrapped_script, 1, 0, &script_txs);
    assert!(
        script_painted.lines().any(|line| line == "Type P2SH-P2WSH"),
        "Type did not sit beside the P2SH-P2WSH badge: {script_painted}"
    );
    assert!(
        !script_painted.lines().any(|line| line == "Multisig 2 of 3"),
        "the wrapped script spend showed Multisig 2 of 3: {script_painted}"
    );

    let taproot = "bc1pahzjr2vnfqnqwl0h8jps2aa4zqe29wqxn2nagvd5a2z982k7ttdq8ry370";
    let tap_witness = format!(r#"["{P2SH_P2WPKH_WITNESS_0}"]"#);
    let tap_txs = address_txs_with_redeem(
        taproot,
        "v1_p2tr",
        MULTISIG_2_OF_3_SCRIPTSIG,
        "OP_0",
        &tap_witness,
        Some(MULTISIG_2_OF_3_REDEEM),
    );
    let tap_painted = paint_address_loaded(taproot, 1, 0, &tap_txs);
    assert!(
        tap_painted.lines().any(|line| line == "Type P2TR"),
        "Type did not sit beside the P2TR badge: {tap_painted}"
    );
    assert!(
        !tap_painted.lines().any(|line| line == "Multisig 2 of 3"),
        "a tapscript address showed Multisig 2 of 3: {tap_painted}"
    );

    let other = "3Bmb9Jig8A5kHdDSxvDZ6eryj3AXd3swuJ";
    assert_mainnet_script_address(other);
    let other_txs =
        address_txs_with_redeem(other, "p2sh", "00", "OP_0", "[]", Some(NOT_MULTISIG_REDEEM));
    let other_painted = paint_address_loaded(other, 1, 0, &other_txs);
    assert!(
        !other_painted.lines().any(|line| line == "Multisig 2 of 3"),
        "a redeem script that is not 2-of-3 showed Multisig 2 of 3: {other_painted}"
    );
    let program_txs = address_txs_with_redeem(
        wrapped_script,
        "p2sh",
        "00",
        "OP_0",
        "[]",
        Some(NOT_2_OF_3_REDEEM),
    );
    let program_painted = paint_address_loaded(wrapped_script, 1, 0, &program_txs);
    assert!(
        !program_painted
            .lines()
            .any(|line| line == "Multisig 2 of 3"),
        "the wrapped-script redeem showed Multisig 2 of 3: {program_painted}"
    );

    let short = "bc1q8yj0herd4r4yxszw3nkfvt53433thk0f5qst4g";
    let short_painted = paint_address(short, &address_stats(short));
    assert!(
        short_painted.lines().any(|line| line == "Type P2WPKH"),
        "Type did not sit beside the P2WPKH badge: {short_painted}"
    );
    assert!(
        !short_painted.lines().any(|line| line == "Multisig 2 of 3"),
        "the shorter bc1q address showed Multisig 2 of 3: {short_painted}"
    );

    let script = "bc1qjasf9z3h7w3jspkhtgatgpyvvzgpa2wwd2lr0eh5tx44reyn2k7sfc27a4";
    let script_hash_painted = paint_address(script, &address_stats(script));
    assert!(
        script_hash_painted.lines().any(|line| line == "Type P2WSH"),
        "Type did not sit beside the P2WSH badge: {script_hash_painted}"
    );
    assert!(
        !script_hash_painted
            .lines()
            .any(|line| line == "Multisig 2 of 3"),
        "the longer bc1q address showed Multisig 2 of 3: {script_hash_painted}"
    );

    let pubkey = "1stAprEZapjCYGUACUcXohQqSHF9MU5Kj";
    let pubkey_address_painted = paint_address(pubkey, &address_stats(pubkey));
    assert!(
        pubkey_address_painted
            .lines()
            .any(|line| line == "Type P2PKH"),
        "Type did not sit beside the P2PKH badge: {pubkey_address_painted}"
    );
    assert!(
        !pubkey_address_painted
            .lines()
            .any(|line| line == "Multisig 2 of 3"),
        "the p2pkh address showed Multisig 2 of 3: {pubkey_address_painted}"
    );

    let idle = paint_address(address, &address_stats(address));
    assert!(
        !idle.lines().any(|line| line == "Multisig 2 of 3"),
        "a p2sh address with no spend showed Multisig 2 of 3: {idle}"
    );
    assert_plain_p2sh_once(&idle);

    let (mut host, rec) = host_with(
        Network::Mainnet,
        BTreeMap::from([(
            format!("/address/{address}"),
            "Invalid Bitcoin address".to_string(),
        )]),
    );
    let err = host
        .open_address(address)
        .expect_err("invalid address must fail");
    let paths = rec.gets();
    assert!(
        paths
            .iter()
            .any(|path| path == &format!("/address/{address}")),
        "address load did not GET /address/{address}: {paths:?}"
    );
    assert!(
        paths
            .iter()
            .all(|path| !path.contains('?') && !path.contains('#')),
        "address load used a query string: {paths:?}"
    );
    let failed = painted(&host);
    assert!(
        failed
            .lines()
            .any(|line| line == "Error loading address data."),
        "Error loading address data. is absent: {failed}"
    );
    assert!(
        err.to_string().contains("Invalid Bitcoin address"),
        "indexer body left the error: {err}"
    );
    assert!(
        failed.lines().any(|line| line == "Invalid Bitcoin address"),
        "Invalid Bitcoin address was replaced or dropped: {failed}"
    );
    assert!(
        !failed.lines().any(|line| line == "Multisig 2 of 3"),
        "a failed load showed Multisig 2 of 3: {failed}"
    );
    assert_no_type_badge(&failed);
}

#[test]
fn address_screen_paints_confirmed_chain_balance() {
    let funded = paint_address(
        "bc1qfunded",
        r#"{"address":"bc1qfunded","chain_stats":{"tx_count":4,"funded_txo_count":4,"spent_txo_count":1,"funded_txo_sum":9000,"spent_txo_sum":1000},"mempool_stats":{"tx_count":1,"funded_txo_count":1,"spent_txo_count":2,"funded_txo_sum":20,"spent_txo_sum":7}}"#,
    );
    assert!(funded.contains("funded_sum 9000"), "{funded}");
    assert!(funded.contains("row address funded_sum 9000"), "{funded}");
    assert!(funded.contains("Confirmed balance 8000"), "{funded}");

    let omitted = paint_address(
        "bc1qomit",
        r#"{"address":"bc1qomit","chain_stats":{"tx_count":4,"funded_txo_count":4,"spent_txo_count":1,"funded_txo_sum":9000},"mempool_stats":{"tx_count":1,"funded_txo_count":1,"spent_txo_count":2,"funded_txo_sum":20,"spent_txo_sum":7}}"#,
    );
    assert!(!omitted.contains("Confirmed balance"), "{omitted}");
    assert!(
        !omitted.split_whitespace().any(|token| token == "0"),
        "missing chain sum was painted as 0: {omitted}"
    );
    assert!(omitted.contains("funded_sum 9000"), "{omitted}");

    let zero = paint_address(
        "bc1qzero",
        r#"{"address":"bc1qzero","chain_stats":{"tx_count":4,"funded_txo_count":4,"spent_txo_count":1,"funded_txo_sum":0,"spent_txo_sum":0},"mempool_stats":{"tx_count":1,"funded_txo_count":1,"spent_txo_count":2,"funded_txo_sum":20,"spent_txo_sum":7}}"#,
    );
    assert!(zero.contains("funded_sum 0"), "{zero}");
    assert!(zero.contains("Confirmed balance 0"), "{zero}");
}

/// Chain funded count minus chain spent count. A missing count is not zero.
#[test]
fn address_screen_paints_confirmed_utxo_count() {
    let funded = paint_address(
        "bc1qutxos",
        r#"{"address":"bc1qutxos","chain_stats":{"tx_count":4,"funded_txo_count":4,"spent_txo_count":1,"funded_txo_sum":9000,"spent_txo_sum":1000},"mempool_stats":{"tx_count":1,"funded_txo_count":1,"spent_txo_count":2,"funded_txo_sum":20,"spent_txo_sum":7}}"#,
    );
    assert!(funded.contains("Confirmed UTXOs 3"), "{funded}");
    assert!(funded.contains("Confirmed balance 8000"), "{funded}");
    let balance_at = funded.find("Confirmed balance 8000").expect("balance");
    let utxo_at = funded.find("Confirmed UTXOs 3").expect("utxos");
    assert!(
        utxo_at > balance_at,
        "utxo words do not follow the balance in {funded}"
    );
    assert!(
        !funded.contains("Confirmed UTXOs -1"),
        "mempool count difference painted: {funded}"
    );

    let omitted = paint_address(
        "bc1qnocount",
        r#"{"address":"bc1qnocount","chain_stats":{"tx_count":4,"funded_txo_count":4,"funded_txo_sum":9000,"spent_txo_sum":1000},"mempool_stats":{"tx_count":1,"funded_txo_count":1,"spent_txo_count":2,"funded_txo_sum":20,"spent_txo_sum":7}}"#,
    );
    assert!(!omitted.contains("Confirmed UTXOs"), "{omitted}");
    assert!(
        !omitted.split_whitespace().any(|token| token == "0"),
        "missing chain count was painted as 0: {omitted}"
    );

    let zero = paint_address(
        "bc1qzerocount",
        r#"{"address":"bc1qzerocount","chain_stats":{"tx_count":4,"funded_txo_count":0,"spent_txo_count":0,"funded_txo_sum":9000,"spent_txo_sum":1000},"mempool_stats":{"tx_count":1,"funded_txo_count":1,"spent_txo_count":2,"funded_txo_sum":20,"spent_txo_sum":7}}"#,
    );
    assert!(zero.contains("Confirmed UTXOs 0"), "{zero}");
    assert!(zero.contains("Confirmed balance 8000"), "{zero}");
}

/// Chain funded sum. A missing sum is not zero. This is not the balance.
#[test]
fn address_screen_paints_total_received() {
    let funded = paint_address(
        "bc1qrecv",
        r#"{"address":"bc1qrecv","chain_stats":{"tx_count":4,"funded_txo_count":4,"spent_txo_count":1,"funded_txo_sum":9000,"spent_txo_sum":1000},"mempool_stats":{"tx_count":1,"funded_txo_count":1,"spent_txo_count":2,"funded_txo_sum":20,"spent_txo_sum":7}}"#,
    );
    assert!(funded.contains("Confirmed balance 8000"), "{funded}");
    assert!(funded.contains("Confirmed UTXOs 3"), "{funded}");
    assert!(funded.contains("Total received 9000"), "{funded}");
    assert!(
        !funded.contains("Total received 8000"),
        "balance was painted as total received: {funded}"
    );
    let balance_at = funded.find("Confirmed balance 8000").expect("balance");
    let utxo_at = funded.find("Confirmed UTXOs 3").expect("utxos");
    let received_at = funded.find("Total received 9000").expect("received");
    assert!(
        utxo_at > balance_at && received_at > utxo_at,
        "total received does not follow the balance and the utxo count in {funded}"
    );

    let omitted = paint_address(
        "bc1qnorecv",
        r#"{"address":"bc1qnorecv","chain_stats":{"tx_count":4,"funded_txo_count":4,"spent_txo_count":1,"spent_txo_sum":1000},"mempool_stats":{"tx_count":1,"funded_txo_count":1,"spent_txo_count":2,"funded_txo_sum":20,"spent_txo_sum":7}}"#,
    );
    assert!(!omitted.contains("Total received"), "{omitted}");
    assert!(
        !omitted.split_whitespace().any(|token| token == "0"),
        "missing funded sum was painted as 0: {omitted}"
    );

    let zero = paint_address(
        "bc1qrecvzero",
        r#"{"address":"bc1qrecvzero","chain_stats":{"tx_count":4,"funded_txo_count":4,"spent_txo_count":1,"funded_txo_sum":0,"spent_txo_sum":0},"mempool_stats":{"tx_count":1,"funded_txo_count":1,"spent_txo_count":2,"funded_txo_sum":20,"spent_txo_sum":7}}"#,
    );
    assert!(zero.contains("Total received 0"), "{zero}");
}

/// `GET /scripthash/:hash` uses the same chain stats as an address.
/// Confirmed balance is chain funded minus chain spent. Confirmed UTXOs is
/// chain funded count minus chain spent count. Total received is the chain
/// funded sum. A missing number is not zero. Explicit zero stays zero.
#[test]
fn scripthash_screen_paints_confirmed_balance_utxos_and_total_received() {
    let hash = "ab".repeat(32);
    let funded = paint_scripthash(
        &hash,
        &format!(
            r#"{{"scripthash":"{hash}","chain_stats":{{"tx_count":4,"funded_txo_count":4,"spent_txo_count":1,"funded_txo_sum":9000,"spent_txo_sum":1000}},"mempool_stats":{{"tx_count":1,"funded_txo_count":1,"spent_txo_count":2,"funded_txo_sum":20,"spent_txo_sum":7}}}}"#
        ),
    );
    assert!(funded.contains("Confirmed balance 8000"), "{funded}");
    assert!(funded.contains("Confirmed UTXOs 3"), "{funded}");
    assert!(funded.contains("Total received 9000"), "{funded}");
    assert!(
        !funded.contains("Total received 8000"),
        "balance was painted as total received: {funded}"
    );
    assert!(
        !funded.contains("Confirmed UTXOs -1"),
        "mempool count difference painted: {funded}"
    );
    let balance_at = funded.find("Confirmed balance 8000").expect("balance");
    let utxo_at = funded.find("Confirmed UTXOs 3").expect("utxos");
    let received_at = funded.find("Total received 9000").expect("received");
    assert!(
        utxo_at > balance_at && received_at > utxo_at,
        "total received does not follow the balance and the utxo count in {funded}"
    );

    let omit_hash = "cd".repeat(32);
    let omitted = paint_scripthash(
        &omit_hash,
        &format!(
            r#"{{"scripthash":"{omit_hash}","chain_stats":{{"tx_count":4,"funded_txo_count":4,"spent_txo_sum":1000}},"mempool_stats":{{"tx_count":1,"funded_txo_count":1,"spent_txo_count":2,"funded_txo_sum":20,"spent_txo_sum":7}}}}"#
        ),
    );
    assert!(!omitted.contains("Confirmed balance"), "{omitted}");
    assert!(!omitted.contains("Confirmed UTXOs"), "{omitted}");
    assert!(!omitted.contains("Total received"), "{omitted}");
    assert!(
        !omitted.split_whitespace().any(|token| token == "0"),
        "missing chain number was painted as 0: {omitted}"
    );

    let zero_hash = "ef".repeat(32);
    let zero = paint_scripthash(
        &zero_hash,
        &format!(
            r#"{{"scripthash":"{zero_hash}","chain_stats":{{"tx_count":4,"funded_txo_count":0,"spent_txo_count":0,"funded_txo_sum":0,"spent_txo_sum":0}},"mempool_stats":{{"tx_count":1,"funded_txo_count":1,"spent_txo_count":2,"funded_txo_sum":20,"spent_txo_sum":7}}}}"#
        ),
    );
    assert!(zero.contains("Confirmed balance 0"), "{zero}");
    assert!(zero.contains("Confirmed UTXOs 0"), "{zero}");
    assert!(zero.contains("Total received 0"), "{zero}");
}

/// Chain spent sum plus mempool spent sum. A missing sum is not zero.
/// Both explicit zeros paint `Total sent 0`. The line follows Total received.
/// Confirmed balance stays chain funded minus chain spent.
#[test]
fn address_screen_paints_total_sent() {
    let funded = paint_address(
        "bc1qsent",
        r#"{"address":"bc1qsent","chain_stats":{"tx_count":4,"funded_txo_count":4,"spent_txo_count":1,"funded_txo_sum":9000,"spent_txo_sum":1000},"mempool_stats":{"tx_count":1,"funded_txo_count":1,"spent_txo_count":2,"funded_txo_sum":20,"spent_txo_sum":7}}"#,
    );
    assert!(funded.contains("Confirmed balance 8000"), "{funded}");
    assert!(
        !funded.contains("Confirmed balance 7993"),
        "mempool spent was subtracted from the confirmed balance: {funded}"
    );
    assert!(funded.contains("Confirmed UTXOs 3"), "{funded}");
    assert!(funded.contains("Total received 9000"), "{funded}");
    assert!(funded.contains("Total sent 1007"), "{funded}");
    assert!(
        !funded.contains("Total sent 1000"),
        "total sent ignored the mempool spent sum: {funded}"
    );
    let balance_at = funded.find("Confirmed balance 8000").expect("balance");
    let utxo_at = funded.find("Confirmed UTXOs 3").expect("utxos");
    let received_at = funded.find("Total received 9000").expect("received");
    let sent_at = funded.find("Total sent 1007").expect("sent");
    assert!(
        utxo_at > balance_at && received_at > utxo_at && sent_at > received_at,
        "total sent does not follow total received in {funded}"
    );

    let omitted = paint_address(
        "bc1qnosent",
        r#"{"address":"bc1qnosent","chain_stats":{"tx_count":4,"funded_txo_count":4,"spent_txo_count":1,"funded_txo_sum":9000},"mempool_stats":{"tx_count":1,"funded_txo_count":1,"spent_txo_count":2,"funded_txo_sum":20,"spent_txo_sum":7}}"#,
    );
    assert!(omitted.contains("Total received 9000"), "{omitted}");
    assert!(!omitted.contains("Total sent"), "{omitted}");
    assert!(
        !omitted.split_whitespace().any(|token| token == "0"),
        "missing spent sum was painted as 0: {omitted}"
    );

    let zero = paint_address(
        "bc1qsentzero",
        r#"{"address":"bc1qsentzero","chain_stats":{"tx_count":4,"funded_txo_count":4,"spent_txo_count":1,"funded_txo_sum":9000,"spent_txo_sum":0},"mempool_stats":{"tx_count":1,"funded_txo_count":1,"spent_txo_count":2,"funded_txo_sum":20,"spent_txo_sum":0}}"#,
    );
    assert!(zero.contains("Confirmed balance 9000"), "{zero}");
    assert!(zero.contains("Total received 9000"), "{zero}");
    assert!(zero.contains("Total sent 0"), "{zero}");
    let zero_received = zero.find("Total received 9000").expect("received");
    let zero_sent = zero.find("Total sent 0").expect("sent");
    assert!(
        zero_sent > zero_received,
        "total sent does not follow total received in {zero}"
    );

    let chain_spent = u64::MAX;
    let mempool_spent = 1u64;
    let sum = i128::from(chain_spent) + i128::from(mempool_spent);
    let wide = paint_address(
        "bc1qsentwide",
        &format!(
            r#"{{"address":"bc1qsentwide","chain_stats":{{"tx_count":4,"funded_txo_count":4,"spent_txo_count":1,"funded_txo_sum":9000,"spent_txo_sum":{chain_spent}}},"mempool_stats":{{"tx_count":1,"funded_txo_count":1,"spent_txo_count":2,"funded_txo_sum":20,"spent_txo_sum":{mempool_spent}}}}}"#
        ),
    );
    assert!(wide.contains(&format!("Total sent {sum}")), "{wide}");
    assert!(!wide.contains("Total sent 0"), "{wide}");
}

/// `GET /scripthash/:hash` uses the same spent sums as an address.
/// Total sent is chain spent plus mempool spent. A missing sum omits the line.
#[test]
fn scripthash_screen_paints_total_sent() {
    let hash = "11".repeat(32);
    let funded = paint_scripthash(
        &hash,
        &format!(
            r#"{{"scripthash":"{hash}","chain_stats":{{"tx_count":4,"funded_txo_count":4,"spent_txo_count":1,"funded_txo_sum":9000,"spent_txo_sum":1000}},"mempool_stats":{{"tx_count":1,"funded_txo_count":1,"spent_txo_count":2,"funded_txo_sum":20,"spent_txo_sum":7}}}}"#
        ),
    );
    assert!(funded.contains("Confirmed balance 8000"), "{funded}");
    assert!(
        !funded.contains("Confirmed balance 7993"),
        "mempool spent was subtracted from the confirmed balance: {funded}"
    );
    assert!(funded.contains("Confirmed UTXOs 3"), "{funded}");
    assert!(funded.contains("Total received 9000"), "{funded}");
    assert!(funded.contains("Total sent 1007"), "{funded}");
    assert!(
        !funded.contains("Total sent 1000"),
        "total sent ignored the mempool spent sum: {funded}"
    );
    let balance_at = funded.find("Confirmed balance 8000").expect("balance");
    let utxo_at = funded.find("Confirmed UTXOs 3").expect("utxos");
    let received_at = funded.find("Total received 9000").expect("received");
    let sent_at = funded.find("Total sent 1007").expect("sent");
    assert!(
        utxo_at > balance_at && received_at > utxo_at && sent_at > received_at,
        "total sent does not follow total received in {funded}"
    );

    let omit_hash = "22".repeat(32);
    let omitted = paint_scripthash(
        &omit_hash,
        &format!(
            r#"{{"scripthash":"{omit_hash}","chain_stats":{{"tx_count":4,"funded_txo_count":4,"spent_txo_count":1,"funded_txo_sum":9000,"spent_txo_sum":1000}},"mempool_stats":{{"tx_count":1,"funded_txo_count":1,"spent_txo_count":2,"funded_txo_sum":20}}}}"#
        ),
    );
    assert!(omitted.contains("Total received 9000"), "{omitted}");
    assert!(omitted.contains("Confirmed balance 8000"), "{omitted}");
    assert!(!omitted.contains("Total sent"), "{omitted}");
    assert!(
        !omitted.split_whitespace().any(|token| token == "0"),
        "missing spent sum was painted as 0: {omitted}"
    );

    let zero_hash = "33".repeat(32);
    let zero = paint_scripthash(
        &zero_hash,
        &format!(
            r#"{{"scripthash":"{zero_hash}","chain_stats":{{"tx_count":4,"funded_txo_count":4,"spent_txo_count":1,"funded_txo_sum":9000,"spent_txo_sum":0}},"mempool_stats":{{"tx_count":1,"funded_txo_count":1,"spent_txo_count":2,"funded_txo_sum":20,"spent_txo_sum":0}}}}"#
        ),
    );
    assert!(zero.contains("Confirmed balance 9000"), "{zero}");
    assert!(zero.contains("Total received 9000"), "{zero}");
    assert!(zero.contains("Total sent 0"), "{zero}");
    let zero_received = zero.find("Total received 9000").expect("received");
    let zero_sent = zero.find("Total sent 0").expect("sent");
    assert!(
        zero_sent > zero_received,
        "total sent does not follow total received in {zero}"
    );

    let chain_spent = u64::MAX;
    let mempool_spent = 1u64;
    let sum = i128::from(chain_spent) + i128::from(mempool_spent);
    let wide_hash = "44".repeat(32);
    let wide = paint_scripthash(
        &wide_hash,
        &format!(
            r#"{{"scripthash":"{wide_hash}","chain_stats":{{"tx_count":4,"funded_txo_count":4,"spent_txo_count":1,"funded_txo_sum":9000,"spent_txo_sum":{chain_spent}}},"mempool_stats":{{"tx_count":1,"funded_txo_count":1,"spent_txo_count":2,"funded_txo_sum":20,"spent_txo_sum":{mempool_spent}}}}}"#
        ),
    );
    assert!(wide.contains(&format!("Total sent {sum}")), "{wide}");
    assert!(!wide.contains("Total sent 0"), "{wide}");
}

/// Mempool funded count minus mempool spent count. A missing count is not zero.
/// Both explicit zeros paint `Pending UTXOs 0`. The line follows Total sent.
/// The difference is signed. Confirmed UTXOs stays the chain counts.
#[test]
fn address_screen_paints_pending_utxos() {
    let funded = paint_address(
        "bc1qpending",
        r#"{"address":"bc1qpending","chain_stats":{"tx_count":4,"funded_txo_count":4,"spent_txo_count":1,"funded_txo_sum":9000,"spent_txo_sum":1000},"mempool_stats":{"tx_count":1,"funded_txo_count":1,"spent_txo_count":2,"funded_txo_sum":20,"spent_txo_sum":7}}"#,
    );
    assert!(funded.contains("Confirmed balance 8000"), "{funded}");
    assert!(funded.contains("Confirmed UTXOs 3"), "{funded}");
    assert!(
        !funded.contains("Confirmed UTXOs -1"),
        "mempool count difference painted as confirmed utxos: {funded}"
    );
    assert!(funded.contains("Total sent 1007"), "{funded}");
    assert!(funded.contains("Pending UTXOs -1"), "{funded}");
    let sent_at = funded.find("Total sent 1007").expect("sent");
    let pending_at = funded.find("Pending UTXOs -1").expect("pending");
    assert!(
        pending_at > sent_at,
        "pending utxos do not follow total sent in {funded}"
    );

    let omitted = paint_address(
        "bc1qnopend",
        r#"{"address":"bc1qnopend","chain_stats":{"tx_count":4,"funded_txo_count":4,"spent_txo_count":1,"funded_txo_sum":9000,"spent_txo_sum":1000},"mempool_stats":{"tx_count":1,"funded_txo_count":1,"funded_txo_sum":20,"spent_txo_sum":7}}"#,
    );
    assert!(omitted.contains("Total sent 1007"), "{omitted}");
    assert!(omitted.contains("Confirmed UTXOs 3"), "{omitted}");
    assert!(!omitted.contains("Pending UTXOs"), "{omitted}");
    assert!(
        !omitted.split_whitespace().any(|token| token == "0"),
        "missing mempool count was painted as 0: {omitted}"
    );

    let zero = paint_address(
        "bc1qpendzero",
        r#"{"address":"bc1qpendzero","chain_stats":{"tx_count":4,"funded_txo_count":4,"spent_txo_count":1,"funded_txo_sum":9000,"spent_txo_sum":1000},"mempool_stats":{"tx_count":1,"funded_txo_count":0,"spent_txo_count":0,"funded_txo_sum":20,"spent_txo_sum":7}}"#,
    );
    assert!(zero.contains("Confirmed UTXOs 3"), "{zero}");
    assert!(zero.contains("Total sent 1007"), "{zero}");
    assert!(zero.contains("Pending UTXOs 0"), "{zero}");
    assert!(!zero.contains("Pending UTXOs -1"), "{zero}");
    let zero_sent = zero.find("Total sent 1007").expect("sent");
    let zero_pending = zero.find("Pending UTXOs 0").expect("pending");
    assert!(
        zero_pending > zero_sent,
        "pending utxos do not follow total sent in {zero}"
    );
}

/// `GET /scripthash/:hash` uses the same mempool counts as an address.
/// Pending UTXOs is mempool funded count minus mempool spent count.
/// A missing count omits the line. Both explicit zeros paint `Pending UTXOs 0`.
#[test]
fn scripthash_screen_paints_pending_utxos() {
    let hash = "55".repeat(32);
    let funded = paint_scripthash(
        &hash,
        &format!(
            r#"{{"scripthash":"{hash}","chain_stats":{{"tx_count":4,"funded_txo_count":4,"spent_txo_count":1,"funded_txo_sum":9000,"spent_txo_sum":1000}},"mempool_stats":{{"tx_count":1,"funded_txo_count":1,"spent_txo_count":2,"funded_txo_sum":20,"spent_txo_sum":7}}}}"#
        ),
    );
    assert!(funded.contains("Confirmed balance 8000"), "{funded}");
    assert!(funded.contains("Confirmed UTXOs 3"), "{funded}");
    assert!(
        !funded.contains("Confirmed UTXOs -1"),
        "mempool count difference painted as confirmed utxos: {funded}"
    );
    assert!(funded.contains("Total sent 1007"), "{funded}");
    assert!(funded.contains("Pending UTXOs -1"), "{funded}");
    let sent_at = funded.find("Total sent 1007").expect("sent");
    let pending_at = funded.find("Pending UTXOs -1").expect("pending");
    assert!(
        pending_at > sent_at,
        "pending utxos do not follow total sent in {funded}"
    );

    let omit_hash = "66".repeat(32);
    let omitted = paint_scripthash(
        &omit_hash,
        &format!(
            r#"{{"scripthash":"{omit_hash}","chain_stats":{{"tx_count":4,"funded_txo_count":4,"spent_txo_count":1,"funded_txo_sum":9000,"spent_txo_sum":1000}},"mempool_stats":{{"tx_count":1,"spent_txo_count":2,"funded_txo_sum":20,"spent_txo_sum":7}}}}"#
        ),
    );
    assert!(omitted.contains("Total sent 1007"), "{omitted}");
    assert!(omitted.contains("Confirmed UTXOs 3"), "{omitted}");
    assert!(!omitted.contains("Pending UTXOs"), "{omitted}");
    assert!(
        !omitted.split_whitespace().any(|token| token == "0"),
        "missing mempool count was painted as 0: {omitted}"
    );

    let zero_hash = "77".repeat(32);
    let zero = paint_scripthash(
        &zero_hash,
        &format!(
            r#"{{"scripthash":"{zero_hash}","chain_stats":{{"tx_count":4,"funded_txo_count":4,"spent_txo_count":1,"funded_txo_sum":9000,"spent_txo_sum":1000}},"mempool_stats":{{"tx_count":1,"funded_txo_count":0,"spent_txo_count":0,"funded_txo_sum":20,"spent_txo_sum":7}}}}"#
        ),
    );
    assert!(zero.contains("Confirmed UTXOs 3"), "{zero}");
    assert!(zero.contains("Total sent 1007"), "{zero}");
    assert!(zero.contains("Pending UTXOs 0"), "{zero}");
    assert!(!zero.contains("Pending UTXOs -1"), "{zero}");
    let zero_sent = zero.find("Total sent 1007").expect("sent");
    let zero_pending = zero.find("Pending UTXOs 0").expect("pending");
    assert!(
        zero_pending > zero_sent,
        "pending utxos do not follow total sent in {zero}"
    );
}

/// Chain tx count plus mempool tx count. A missing count is not zero.
/// Both explicit zeros paint `Transactions 0`. The line follows Pending UTXOs.
/// `chain_tx_count` and `mempool_tx_count` stay on the line as separate text.
#[test]
fn address_screen_paints_transaction_count_sum() {
    let funded = paint_address(
        "bc1qtxsum",
        r#"{"address":"bc1qtxsum","chain_stats":{"tx_count":4,"funded_txo_count":4,"spent_txo_count":1,"funded_txo_sum":9000,"spent_txo_sum":1000},"mempool_stats":{"tx_count":1,"funded_txo_count":1,"spent_txo_count":2,"funded_txo_sum":20,"spent_txo_sum":7}}"#,
    );
    assert!(funded.contains("chain_tx_count 4"), "{funded}");
    assert!(funded.contains("mempool_tx_count 1"), "{funded}");
    assert!(funded.contains("Pending UTXOs -1"), "{funded}");
    assert!(funded.contains("Transactions 5"), "{funded}");
    let pending_at = funded.find("Pending UTXOs -1").expect("pending");
    let tx_at = funded.find("Transactions 5").expect("transactions");
    assert!(
        tx_at > pending_at,
        "transactions do not follow pending utxos in {funded}"
    );

    let omitted = paint_address(
        "bc1qtxomit",
        r#"{"address":"bc1qtxomit","chain_stats":{"funded_txo_count":4,"spent_txo_count":1,"funded_txo_sum":9000,"spent_txo_sum":1000},"mempool_stats":{"tx_count":1,"funded_txo_count":1,"spent_txo_count":2,"funded_txo_sum":20,"spent_txo_sum":7}}"#,
    );
    assert!(omitted.contains("Confirmed balance 8000"), "{omitted}");
    assert!(omitted.contains("Pending UTXOs -1"), "{omitted}");
    assert!(!omitted.contains("Transactions"), "{omitted}");
    assert!(
        !omitted.split_whitespace().any(|token| token == "0"),
        "missing tx count was painted as 0: {omitted}"
    );

    let zero = paint_address(
        "bc1qtxzero",
        r#"{"address":"bc1qtxzero","chain_stats":{"tx_count":0,"funded_txo_count":4,"spent_txo_count":1,"funded_txo_sum":9000,"spent_txo_sum":1000},"mempool_stats":{"tx_count":0,"funded_txo_count":1,"spent_txo_count":2,"funded_txo_sum":20,"spent_txo_sum":7}}"#,
    );
    assert!(zero.contains("chain_tx_count 0"), "{zero}");
    assert!(zero.contains("mempool_tx_count 0"), "{zero}");
    assert!(zero.contains("Pending UTXOs -1"), "{zero}");
    assert!(zero.contains("Transactions 0"), "{zero}");
    let zero_pending = zero.find("Pending UTXOs -1").expect("pending");
    let zero_tx = zero.find("Transactions 0").expect("transactions");
    assert!(
        zero_tx > zero_pending,
        "transactions do not follow pending utxos in {zero}"
    );
}

/// `GET /scripthash/:hash` uses the same tx counts as an address.
/// Transactions is the chain count plus the mempool count.
/// A missing count omits the line. Both explicit zeros paint `Transactions 0`.
#[test]
fn scripthash_screen_paints_transaction_count_sum() {
    let hash = "81".repeat(32);
    let funded = paint_scripthash(
        &hash,
        &format!(
            r#"{{"scripthash":"{hash}","chain_stats":{{"tx_count":4,"funded_txo_count":4,"spent_txo_count":1,"funded_txo_sum":9000,"spent_txo_sum":1000}},"mempool_stats":{{"tx_count":1,"funded_txo_count":1,"spent_txo_count":2,"funded_txo_sum":20,"spent_txo_sum":7}}}}"#
        ),
    );
    assert!(funded.contains("chain_tx_count 4"), "{funded}");
    assert!(funded.contains("mempool_tx_count 1"), "{funded}");
    assert!(funded.contains("Pending UTXOs -1"), "{funded}");
    assert!(funded.contains("Transactions 5"), "{funded}");
    let pending_at = funded.find("Pending UTXOs -1").expect("pending");
    let tx_at = funded.find("Transactions 5").expect("transactions");
    assert!(
        tx_at > pending_at,
        "transactions do not follow pending utxos in {funded}"
    );

    let omit_hash = "82".repeat(32);
    let omitted = paint_scripthash(
        &omit_hash,
        &format!(
            r#"{{"scripthash":"{omit_hash}","chain_stats":{{"tx_count":4,"funded_txo_count":4,"spent_txo_count":1,"funded_txo_sum":9000,"spent_txo_sum":1000}},"mempool_stats":{{"funded_txo_count":1,"spent_txo_count":2,"funded_txo_sum":20,"spent_txo_sum":7}}}}"#
        ),
    );
    assert!(omitted.contains("Confirmed balance 8000"), "{omitted}");
    assert!(omitted.contains("Pending UTXOs -1"), "{omitted}");
    assert!(!omitted.contains("Transactions"), "{omitted}");
    assert!(
        !omitted.split_whitespace().any(|token| token == "0"),
        "missing tx count was painted as 0: {omitted}"
    );

    let zero_hash = "83".repeat(32);
    let zero = paint_scripthash(
        &zero_hash,
        &format!(
            r#"{{"scripthash":"{zero_hash}","chain_stats":{{"tx_count":0,"funded_txo_count":4,"spent_txo_count":1,"funded_txo_sum":9000,"spent_txo_sum":1000}},"mempool_stats":{{"tx_count":0,"funded_txo_count":1,"spent_txo_count":2,"funded_txo_sum":20,"spent_txo_sum":7}}}}"#
        ),
    );
    assert!(zero.contains("chain_tx_count 0"), "{zero}");
    assert!(zero.contains("mempool_tx_count 0"), "{zero}");
    assert!(zero.contains("Pending UTXOs -1"), "{zero}");
    assert!(zero.contains("Transactions 0"), "{zero}");
    let zero_pending = zero.find("Pending UTXOs -1").expect("pending");
    let zero_tx = zero.find("Transactions 0").expect("transactions");
    assert!(
        zero_tx > zero_pending,
        "transactions do not follow pending utxos in {zero}"
    );
}

/// `Pending <integer>` is the mempool sum difference. `Pending UTXOs` is not that label.
fn pending_amount_tokens(text: &str) -> Vec<String> {
    text.split_whitespace()
        .collect::<Vec<_>>()
        .windows(2)
        .filter(|pair| pair[0] == "Pending" && signed_integer_token(pair[1]))
        .map(|pair| format!("{} {}", pair[0], pair[1]))
        .collect()
}

fn signed_integer_token(token: &str) -> bool {
    let digits = token.strip_prefix('-').unwrap_or(token);
    !digits.is_empty() && digits.bytes().all(|byte| byte.is_ascii_digit())
}

/// Mempool funded sum minus mempool spent sum. A missing sum is not zero.
/// Both explicit zeros paint `Pending 0`. The label follows Transactions.
/// Pending UTXOs stays the mempool count difference. The difference is signed.
#[test]
fn address_screen_paints_pending_amount() {
    let funded = paint_address(
        "bc1qpendamt",
        r#"{"address":"bc1qpendamt","chain_stats":{"tx_count":4,"funded_txo_count":4,"spent_txo_count":1,"funded_txo_sum":9000,"spent_txo_sum":1000},"mempool_stats":{"tx_count":1,"funded_txo_count":1,"spent_txo_count":2,"funded_txo_sum":20,"spent_txo_sum":7}}"#,
    );
    assert!(funded.contains("Confirmed balance 8000"), "{funded}");
    assert!(funded.contains("Pending UTXOs -1"), "{funded}");
    assert!(funded.contains("Transactions 5"), "{funded}");
    assert!(funded.contains("Pending 13"), "{funded}");
    assert!(
        !funded.contains("Pending UTXOs 13"),
        "pending amount was painted as Pending UTXOs: {funded}"
    );
    let utxo_at = funded.find("Pending UTXOs -1").expect("pending utxos");
    let tx_at = funded.find("Transactions 5").expect("transactions");
    let amount_at = funded.find("Pending 13").expect("pending amount");
    assert!(
        tx_at > utxo_at && amount_at > tx_at,
        "pending amount does not follow transactions in {funded}"
    );

    let negative = paint_address(
        "bc1qpendneg",
        r#"{"address":"bc1qpendneg","chain_stats":{"tx_count":4,"funded_txo_count":4,"spent_txo_count":1,"funded_txo_sum":9000,"spent_txo_sum":1000},"mempool_stats":{"tx_count":1,"funded_txo_count":1,"spent_txo_count":2,"funded_txo_sum":7,"spent_txo_sum":20}}"#,
    );
    assert!(negative.contains("Pending UTXOs -1"), "{negative}");
    assert!(negative.contains("Pending -13"), "{negative}");
    assert!(negative.contains("Confirmed balance 8000"), "{negative}");

    let omitted = paint_address(
        "bc1qpendomit",
        r#"{"address":"bc1qpendomit","chain_stats":{"tx_count":4,"funded_txo_count":4,"spent_txo_count":1,"funded_txo_sum":9000,"spent_txo_sum":1000},"mempool_stats":{"tx_count":1,"funded_txo_count":1,"spent_txo_count":2,"funded_txo_sum":20}}"#,
    );
    assert!(omitted.contains("Pending UTXOs -1"), "{omitted}");
    assert!(omitted.contains("Confirmed balance 8000"), "{omitted}");
    assert!(
        pending_amount_tokens(&omitted).is_empty(),
        "missing mempool sum painted a pending amount: {omitted}"
    );
    assert!(
        !omitted.split_whitespace().any(|token| token == "0"),
        "missing mempool sum was painted as 0: {omitted}"
    );

    let zero = paint_address(
        "bc1qpendzeroamt",
        r#"{"address":"bc1qpendzeroamt","chain_stats":{"tx_count":4,"funded_txo_count":4,"spent_txo_count":1,"funded_txo_sum":9000,"spent_txo_sum":1000},"mempool_stats":{"tx_count":1,"funded_txo_count":1,"spent_txo_count":2,"funded_txo_sum":0,"spent_txo_sum":0}}"#,
    );
    assert!(zero.contains("Pending UTXOs -1"), "{zero}");
    assert!(zero.contains("Transactions 5"), "{zero}");
    assert!(zero.contains("Pending 0"), "{zero}");
    assert!(zero.contains("Confirmed balance 8000"), "{zero}");
    let zero_tx = zero.find("Transactions 5").expect("transactions");
    let zero_amount = zero.find("Pending 0").expect("pending amount");
    assert!(
        zero_amount > zero_tx,
        "pending zero does not follow transactions in {zero}"
    );
}

/// `GET /scripthash/:hash` uses the same mempool sums as an address.
/// Pending is mempool funded sum minus mempool spent sum.
/// A missing sum omits the amount. Both explicit zeros paint `Pending 0`.
#[test]
fn scripthash_screen_paints_pending_amount() {
    let hash = "91".repeat(32);
    let funded = paint_scripthash(
        &hash,
        &format!(
            r#"{{"scripthash":"{hash}","chain_stats":{{"tx_count":4,"funded_txo_count":4,"spent_txo_count":1,"funded_txo_sum":9000,"spent_txo_sum":1000}},"mempool_stats":{{"tx_count":1,"funded_txo_count":1,"spent_txo_count":2,"funded_txo_sum":20,"spent_txo_sum":7}}}}"#
        ),
    );
    assert!(funded.contains("Confirmed balance 8000"), "{funded}");
    assert!(funded.contains("Pending UTXOs -1"), "{funded}");
    assert!(funded.contains("Transactions 5"), "{funded}");
    assert!(funded.contains("Pending 13"), "{funded}");
    assert!(
        !funded.contains("Pending UTXOs 13"),
        "pending amount was painted as Pending UTXOs: {funded}"
    );
    let utxo_at = funded.find("Pending UTXOs -1").expect("pending utxos");
    let tx_at = funded.find("Transactions 5").expect("transactions");
    let amount_at = funded.find("Pending 13").expect("pending amount");
    assert!(
        tx_at > utxo_at && amount_at > tx_at,
        "pending amount does not follow transactions in {funded}"
    );

    let negative_hash = "94".repeat(32);
    let negative = paint_scripthash(
        &negative_hash,
        &format!(
            r#"{{"scripthash":"{negative_hash}","chain_stats":{{"tx_count":4,"funded_txo_count":4,"spent_txo_count":1,"funded_txo_sum":9000,"spent_txo_sum":1000}},"mempool_stats":{{"tx_count":1,"funded_txo_count":1,"spent_txo_count":2,"funded_txo_sum":7,"spent_txo_sum":20}}}}"#
        ),
    );
    assert!(negative.contains("Pending UTXOs -1"), "{negative}");
    assert!(negative.contains("Pending -13"), "{negative}");
    assert!(negative.contains("Confirmed balance 8000"), "{negative}");

    let omit_hash = "92".repeat(32);
    let omitted = paint_scripthash(
        &omit_hash,
        &format!(
            r#"{{"scripthash":"{omit_hash}","chain_stats":{{"tx_count":4,"funded_txo_count":4,"spent_txo_count":1,"funded_txo_sum":9000,"spent_txo_sum":1000}},"mempool_stats":{{"tx_count":1,"funded_txo_count":1,"spent_txo_count":2,"spent_txo_sum":7}}}}"#
        ),
    );
    assert!(omitted.contains("Pending UTXOs -1"), "{omitted}");
    assert!(omitted.contains("Confirmed balance 8000"), "{omitted}");
    assert!(
        pending_amount_tokens(&omitted).is_empty(),
        "missing mempool sum painted a pending amount: {omitted}"
    );
    assert!(
        !omitted.split_whitespace().any(|token| token == "0"),
        "missing mempool sum was painted as 0: {omitted}"
    );

    let zero_hash = "93".repeat(32);
    let zero = paint_scripthash(
        &zero_hash,
        &format!(
            r#"{{"scripthash":"{zero_hash}","chain_stats":{{"tx_count":4,"funded_txo_count":4,"spent_txo_count":1,"funded_txo_sum":9000,"spent_txo_sum":1000}},"mempool_stats":{{"tx_count":1,"funded_txo_count":1,"spent_txo_count":2,"funded_txo_sum":0,"spent_txo_sum":0}}}}"#
        ),
    );
    assert!(zero.contains("Pending UTXOs -1"), "{zero}");
    assert!(zero.contains("Transactions 5"), "{zero}");
    assert!(zero.contains("Pending 0"), "{zero}");
    assert!(zero.contains("Confirmed balance 8000"), "{zero}");
    let zero_tx = zero.find("Transactions 5").expect("transactions");
    let zero_amount = zero.find("Pending 0").expect("pending amount");
    assert!(
        zero_amount > zero_tx,
        "pending zero does not follow transactions in {zero}"
    );
}

/// Chain funded plus mempool funded, minus chain spent plus mempool spent.
/// A missing sum omits `Balance`. All four explicit zeros paint `Balance 0`.
/// Confirmed balance and Pending stay their own numbers. The label follows Pending.
#[test]
fn address_screen_paints_combined_balance() {
    let funded = paint_address(
        "bc1qbal",
        r#"{"address":"bc1qbal","chain_stats":{"tx_count":4,"funded_txo_count":4,"spent_txo_count":1,"funded_txo_sum":9000,"spent_txo_sum":1000},"mempool_stats":{"tx_count":1,"funded_txo_count":1,"spent_txo_count":2,"funded_txo_sum":20,"spent_txo_sum":7}}"#,
    );
    assert!(funded.contains("Balance 8013"), "{funded}");
    assert!(funded.contains("Confirmed balance 8000"), "{funded}");
    assert!(funded.contains("Pending 13"), "{funded}");
    assert!(funded.contains("Pending UTXOs -1"), "{funded}");
    let pending_at = funded.find("Pending 13").expect("pending amount");
    let balance_at = funded.find("Balance 8013").expect("balance");
    assert!(
        balance_at > pending_at,
        "balance does not follow pending amount in {funded}"
    );

    let omitted = paint_address(
        "bc1qbalomit",
        r#"{"address":"bc1qbalomit","chain_stats":{"tx_count":4,"funded_txo_count":4,"spent_txo_count":1,"funded_txo_sum":9000,"spent_txo_sum":1000},"mempool_stats":{"tx_count":1,"funded_txo_count":1,"spent_txo_count":2,"funded_txo_sum":20}}"#,
    );
    assert!(omitted.contains("Confirmed balance 8000"), "{omitted}");
    assert!(
        !omitted.contains("Balance "),
        "missing sum painted a balance: {omitted}"
    );
    assert!(
        !omitted.split_whitespace().any(|token| token == "0"),
        "missing sum was painted as 0: {omitted}"
    );

    let zero = paint_address(
        "bc1qbalzero",
        r#"{"address":"bc1qbalzero","chain_stats":{"tx_count":4,"funded_txo_count":4,"spent_txo_count":1,"funded_txo_sum":0,"spent_txo_sum":0},"mempool_stats":{"tx_count":1,"funded_txo_count":1,"spent_txo_count":2,"funded_txo_sum":0,"spent_txo_sum":0}}"#,
    );
    assert!(zero.contains("Balance 0"), "{zero}");
    assert!(zero.contains("Confirmed balance 0"), "{zero}");
    assert!(zero.contains("Pending 0"), "{zero}");
    let zero_pending = zero.find("Pending 0").expect("pending amount");
    let zero_balance = zero.find("Balance 0").expect("balance");
    assert!(
        zero_balance > zero_pending,
        "balance zero does not follow pending amount in {zero}"
    );
}

/// Chain funded count minus chain spent count. Mempool counts are not in it.
/// Both zeros paint `Unspent TXOs 0`. A failed address-stats parse omits the
/// line. The scripthash summary does not paint it. Confirmed UTXOs stays.
#[test]
fn address_screen_paints_unspent_txos() {
    let funded = paint_address(
        "bc1qunspent",
        r#"{"address":"bc1qunspent","chain_stats":{"tx_count":4,"funded_txo_count":5,"spent_txo_count":2,"funded_txo_sum":9000,"spent_txo_sum":1000},"mempool_stats":{"tx_count":1,"funded_txo_count":9,"spent_txo_count":1,"funded_txo_sum":20,"spent_txo_sum":7}}"#,
    );
    assert!(funded.contains("Transactions 5"), "{funded}");
    assert!(funded.contains("Confirmed UTXOs 3"), "{funded}");
    assert!(funded.contains("mempool_funded_txo_count 9"), "{funded}");
    assert!(funded.contains("mempool_spent_txo_count 1"), "{funded}");
    assert_eq!(
        funded.matches("Unspent TXOs 3").count(),
        1,
        "chain funded 5 minus chain spent 2 was not painted once: {funded}"
    );
    assert!(
        !funded.contains("Unspent TXOs 8"),
        "mempool count difference painted: {funded}"
    );
    assert!(
        !funded.contains("Unspent TXOs 5"),
        "chain funded count was painted without subtracting chain spent: {funded}"
    );
    assert!(
        !funded.contains("Unspent TXOs 11"),
        "mempool counts were folded into Unspent TXOs: {funded}"
    );
    assert!(!funded.contains("<p"), "html tag painted: {funded}");
    assert!(
        !funded.contains("unspent-txos"),
        "html class painted: {funded}"
    );
    let transactions_at = funded.find("Transactions 5").expect("transactions");
    let unspent_at = funded.find("Unspent TXOs 3").expect("unspent");
    assert!(
        unspent_at > transactions_at,
        "Unspent TXOs does not come after Transactions: {funded}"
    );

    let zero = paint_address(
        "bc1qunspent0",
        r#"{"address":"bc1qunspent0","chain_stats":{"tx_count":4,"funded_txo_count":0,"spent_txo_count":0,"funded_txo_sum":9000,"spent_txo_sum":1000},"mempool_stats":{"tx_count":1,"funded_txo_count":9,"spent_txo_count":1,"funded_txo_sum":20,"spent_txo_sum":7}}"#,
    );
    assert!(zero.contains("Unspent TXOs 0"), "{zero}");
    assert!(
        !zero.contains("Unspent TXOs 8"),
        "mempool count difference painted: {zero}"
    );
    assert!(zero.contains("Confirmed UTXOs 0"), "{zero}");

    let omitted = paint_address(
        "bc1qunspentomit",
        r#"{"address":"bc1qunspentomit","chain_stats":{"tx_count":4,"funded_txo_count":5,"funded_txo_sum":9000,"spent_txo_sum":1000},"mempool_stats":{"tx_count":1,"funded_txo_count":9,"spent_txo_count":1,"funded_txo_sum":20,"spent_txo_sum":7}}"#,
    );
    assert!(omitted.contains("chain_funded_txo_count 5"), "{omitted}");
    assert!(!omitted.contains("Unspent TXOs"), "{omitted}");
    assert!(!omitted.contains("Unspent TXOs 0"), "{omitted}");
    assert!(!omitted.contains("Unspent TXOs -"), "{omitted}");
    assert!(
        !omitted.split_whitespace().any(|token| token == "0"),
        "failed address-stats parse was painted as 0: {omitted}"
    );

    let (mut host, _) = host_with(
        Network::Mainnet,
        BTreeMap::from([
            (
                "/address/bc1qunspentbad".to_string(),
                "not-json".to_string(),
            ),
            ("/address/bc1qunspentbad/txs".to_string(), "[]".to_string()),
            ("/address/bc1qunspentbad/utxo".to_string(), "[]".to_string()),
        ]),
    );
    let opened = host.open_address("bc1qunspentbad");
    assert!(opened.is_err(), "not-json parsed as address stats");
    let failed = painted(&host);
    assert!(!failed.contains("Unspent TXOs"), "{failed}");
    assert!(!failed.contains("Unspent TXOs 0"), "{failed}");
    assert!(!failed.contains("Unspent TXOs -"), "{failed}");

    let hash = "d1".repeat(32);
    let script = paint_scripthash(
        &hash,
        &format!(
            r#"{{"scripthash":"{hash}","chain_stats":{{"tx_count":4,"funded_txo_count":5,"spent_txo_count":2,"funded_txo_sum":9000,"spent_txo_sum":1000}},"mempool_stats":{{"tx_count":1,"funded_txo_count":9,"spent_txo_count":1,"funded_txo_sum":20,"spent_txo_sum":7}}}}"#
        ),
    );
    assert!(script.contains("Confirmed UTXOs 3"), "{script}");
    assert!(script.contains("Transactions 5"), "{script}");
    assert!(
        !script.contains("Unspent TXOs"),
        "scripthash summary painted Unspent TXOs: {script}"
    );
}

/// `GET /scripthash/:hash` uses the same four sums as an address.
/// Balance is the signed difference. A missing sum omits the label.
#[test]
fn scripthash_screen_paints_combined_balance() {
    let hash = "b1".repeat(32);
    let funded = paint_scripthash(
        &hash,
        &format!(
            r#"{{"scripthash":"{hash}","chain_stats":{{"tx_count":4,"funded_txo_count":4,"spent_txo_count":1,"funded_txo_sum":9000,"spent_txo_sum":1000}},"mempool_stats":{{"tx_count":1,"funded_txo_count":1,"spent_txo_count":2,"funded_txo_sum":20,"spent_txo_sum":7}}}}"#
        ),
    );
    assert!(funded.contains("Balance 8013"), "{funded}");
    assert!(funded.contains("Confirmed balance 8000"), "{funded}");
    assert!(funded.contains("Pending 13"), "{funded}");
    assert!(funded.contains("Pending UTXOs -1"), "{funded}");
    let pending_at = funded.find("Pending 13").expect("pending amount");
    let balance_at = funded.find("Balance 8013").expect("balance");
    assert!(
        balance_at > pending_at,
        "balance does not follow pending amount in {funded}"
    );

    let omit_hash = "b2".repeat(32);
    let omitted = paint_scripthash(
        &omit_hash,
        &format!(
            r#"{{"scripthash":"{omit_hash}","chain_stats":{{"tx_count":4,"funded_txo_count":4,"spent_txo_count":1,"funded_txo_sum":9000,"spent_txo_sum":1000}},"mempool_stats":{{"tx_count":1,"funded_txo_count":1,"spent_txo_count":2,"funded_txo_sum":20}}}}"#
        ),
    );
    assert!(omitted.contains("Confirmed balance 8000"), "{omitted}");
    assert!(
        !omitted.contains("Balance "),
        "missing sum painted a balance: {omitted}"
    );
    assert!(
        !omitted.split_whitespace().any(|token| token == "0"),
        "missing sum was painted as 0: {omitted}"
    );

    let zero_hash = "b3".repeat(32);
    let zero = paint_scripthash(
        &zero_hash,
        &format!(
            r#"{{"scripthash":"{zero_hash}","chain_stats":{{"tx_count":4,"funded_txo_count":4,"spent_txo_count":1,"funded_txo_sum":0,"spent_txo_sum":0}},"mempool_stats":{{"tx_count":1,"funded_txo_count":1,"spent_txo_count":2,"funded_txo_sum":0,"spent_txo_sum":0}}}}"#
        ),
    );
    assert!(zero.contains("Balance 0"), "{zero}");
    assert!(zero.contains("Confirmed balance 0"), "{zero}");
    assert!(zero.contains("Pending 0"), "{zero}");
    let zero_pending = zero.find("Pending 0").expect("pending amount");
    let zero_balance = zero.find("Balance 0").expect("balance");
    assert!(
        zero_balance > zero_pending,
        "balance zero does not follow pending amount in {zero}"
    );
}

/// `scriptpubkey_type` from `GET /tx/:txid`. The JSON spelling stays.
/// An empty type is omitted. Script hex and asm stay off the screen.
#[test]
fn transaction_screen_paints_script_pubkey_type() {
    let tx = r#"{
        "txid":"typetx",
        "version":1,
        "locktime":0,
        "vin":[{
            "txid":"prev",
            "vout":0,
            "prevout":{
                "scriptpubkey":"0014aa",
                "scriptpubkey_asm":"OP_0 OP_PUSHBYTES_20",
                "scriptpubkey_type":"v0_p2wpkh",
                "scriptpubkey_address":"bc1qinput",
                "value":5
            },
            "scriptsig":"deadbeef",
            "scriptsig_asm":"OP_CHECKSIG",
            "is_coinbase":false,
            "sequence":1
        }],
        "vout":[{
            "scriptpubkey":"6a",
            "scriptpubkey_asm":"OP_RETURN",
            "scriptpubkey_type":"op_return",
            "value":0
        },{
            "scriptpubkey":"51",
            "scriptpubkey_asm":"OP_TRUE",
            "scriptpubkey_type":"",
            "value":1
        }],
        "size":10,
        "weight":40,
        "sigops":0,
        "fee":1,
        "status":{"confirmed":false}
    }"#;
    let (mut host, _) = host_with(
        Network::Mainnet,
        BTreeMap::from([("/tx/typetx".to_string(), tx.to_string())]),
    );
    host.open_transaction("typetx").expect("tx");
    let text = painted(&host);
    assert!(
        text.contains("input script type v0_p2wpkh"),
        "transaction dropped the previous output type: {text}"
    );
    assert!(
        text.contains("output script type op_return"),
        "transaction dropped the output type: {text}"
    );
    let output_types: Vec<&str> = text
        .lines()
        .filter(|line| line.contains("output script type"))
        .collect();
    assert_eq!(
        output_types,
        vec!["output script type op_return"],
        "empty script type was painted: {text}"
    );
    assert!(!text.contains("0014aa"), "{text}");
    assert!(!text.contains("deadbeef"), "{text}");
    assert_op_return_asm_stays_off(&text);
    assert!(!text.contains("scriptpubkey"), "{text}");
}

/// Sum of output `value` fields from `GET /tx/:txid`.
///
/// Every present value is added, including zero. An empty output list is 0.
/// A missing value omits the total. A commitment is not an amount.
/// The add is `u128`. Script hex and asm stay off the screen.
#[test]
fn transaction_screen_paints_output_total() {
    let sum = paint_tx_outputs(
        "outsum",
        &format!(
            "{},{}",
            tx_output_value(Some(9), None),
            tx_output_value(Some(1), None)
        ),
    );
    assert!(
        sum.contains("output total 10"),
        "transaction dropped the output total: {sum}"
    );
    assert!(
        sum.contains("value 9"),
        "per-output value 9 was dropped: {sum}"
    );
    assert!(
        sum.contains("value 1"),
        "per-output value 1 was dropped: {sum}"
    );
    assert!(!sum.contains("0014aa"), "{sum}");
    assert!(!sum.contains("deadbeef"), "{sum}");
    assert_op_return_asm_stays_off(&sum);
    assert!(!sum.contains("scriptpubkey"), "{sum}");

    let missing = paint_tx_outputs(
        "outmiss",
        &format!(
            "{},{}",
            tx_output_value(Some(40), None),
            tx_output_value(None, None)
        ),
    );
    assert!(
        !missing.contains("output total"),
        "missing value painted an output total: {missing}"
    );
    assert!(
        !missing.contains("output total 0"),
        "missing value was painted as 0: {missing}"
    );
    assert!(
        !missing.contains("output total 40"),
        "missing value kept only the present amount: {missing}"
    );

    let zero = paint_tx_outputs(
        "outzero",
        &format!(
            "{},{}",
            tx_output_value(Some(0), None),
            tx_output_value(Some(0), None)
        ),
    );
    assert!(
        zero.contains("output total 0"),
        "explicit zero values omitted the output total: {zero}"
    );

    let empty = paint_tx_outputs("outempty", "");
    assert!(
        empty.contains("output total 0"),
        "empty output list omitted the output total: {empty}"
    );

    let commitment = paint_tx_outputs(
        "outcommit",
        &tx_output_value(None, Some("ab11cd22ef334455")),
    );
    assert!(
        !commitment.contains("output total"),
        "valuecommitment painted an output total: {commitment}"
    );
    assert!(
        !commitment.contains("ab11cd22ef334455"),
        "valuecommitment hex was painted: {commitment}"
    );

    let wide = paint_tx_outputs(
        "outwide",
        &format!(
            "{},{}",
            tx_output_value(Some(u64::MAX), None),
            tx_output_value(Some(u64::MAX), None)
        ),
    );
    assert!(
        wide.contains("output total 36893488147419103230"),
        "output total wrapped: {wide}"
    );
}

fn tx_output_value(value: Option<u64>, commitment: Option<&str>) -> String {
    let mut fields = String::from(
        r#""scriptpubkey":"0014aa","scriptpubkey_asm":"OP_RETURN","scriptpubkey_type":"op_return""#,
    );
    if let Some(value) = value {
        fields.push_str(&format!(r#","value":{value}"#));
    }
    if let Some(commitment) = commitment {
        fields.push_str(&format!(r#","valuecommitment":"{commitment}""#));
    }
    format!("{{{fields}}}")
}

fn paint_tx_outputs(txid: &str, vout: &str) -> String {
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
            "is_coinbase":false,
            "sequence":1
        }}],
        "vout":[{vout}],
        "size":10,
        "weight":40,
        "sigops":0,
        "fee":1,
        "status":{{"confirmed":false}}
    }}"#
    );
    let (mut host, _) = host_with(
        Network::Mainnet,
        BTreeMap::from([(format!("/tx/{txid}"), tx)]),
    );
    host.open_transaction(txid).expect("tx");
    painted(&host)
}

/// `is_coinbase` and `sequence` from `GET /tx/:txid`.
///
/// A coinbase input paints `input coinbase`. False and a missing key do not.
/// A present sequence paints that integer, including zero. A missing sequence
/// omits the label. Script hex and asm stay off the screen.
#[test]
fn transaction_screen_paints_coinbase_and_sequence() {
    let coinbase = paint_tx_input("cointx", r#","is_coinbase":true,"sequence":0"#);
    assert!(
        coinbase.contains("input coinbase"),
        "coinbase input was not painted: {coinbase}"
    );
    assert!(
        coinbase.contains("sequence 0"),
        "sequence 0 was not painted: {coinbase}"
    );
    assert!(!coinbase.contains("deadbeef"), "{coinbase}");
    assert!(!coinbase.contains("OP_CHECKSIG"), "{coinbase}");
    assert!(!coinbase.contains("scriptpubkey"), "{coinbase}");

    let ordinary = paint_tx_input("ordtx", r#","is_coinbase":false,"sequence":7"#);
    assert!(
        !ordinary.contains("input coinbase"),
        "false coinbase was painted: {ordinary}"
    );
    assert!(
        ordinary.contains("sequence 7"),
        "sequence 7 was not painted: {ordinary}"
    );

    let omitted_coinbase = paint_tx_input("omitcb", r#","sequence":7"#);
    assert!(
        !omitted_coinbase.contains("input coinbase"),
        "missing is_coinbase was painted: {omitted_coinbase}"
    );
    assert!(
        omitted_coinbase.contains("sequence 7"),
        "sequence next to a missing coinbase was dropped: {omitted_coinbase}"
    );

    let omitted_sequence = paint_tx_input("omitseq", r#","is_coinbase":false"#);
    assert!(
        omitted_sequence.contains("input outpoint prev:1"),
        "transaction did not paint without sequence: {omitted_sequence}"
    );
    assert!(
        !omitted_sequence.contains("sequence"),
        "missing sequence was painted: {omitted_sequence}"
    );
    assert!(
        !omitted_sequence
            .split_whitespace()
            .any(|token| token == "0"),
        "missing sequence was painted as 0: {omitted_sequence}"
    );
    assert!(
        !omitted_sequence.contains("input coinbase"),
        "false coinbase was painted: {omitted_sequence}"
    );
}

/// Plaintext parenthetical, no HTML, no liquid switch.
///
/// `is_coinbase` true paints the whole line `input coinbase` and the whole
/// line `(Newly Generated Coins)` once each. False paints neither line.
#[test]
fn transaction_screen_paints_newly_generated_coins() {
    let coinbase = paint_tx_input("newcoins", r#","is_coinbase":true,"sequence":0"#);
    assert_eq!(
        coinbase
            .lines()
            .filter(|&line| line == "input coinbase")
            .count(),
        1,
        "input coinbase was not exactly one whole line: {coinbase}"
    );
    assert_eq!(
        coinbase.matches("(Newly Generated Coins)").count(),
        1,
        "(Newly Generated Coins) is absent: {coinbase}"
    );
    assert_eq!(
        coinbase
            .lines()
            .filter(|&line| line == "(Newly Generated Coins)")
            .count(),
        1,
        "(Newly Generated Coins) was not exactly one whole line: {coinbase}"
    );
    for needle in [
        "UNKNOWN",
        "<span",
        "newly-generated-coins",
        "unknown-input",
        "deadbeef",
        "OP_CHECKSIG",
    ] {
        assert!(
            !coinbase.contains(needle),
            "{needle} was painted: {coinbase}"
        );
    }

    let ordinary = paint_tx_input("newcoinsf", r#","is_coinbase":false,"sequence":7"#);
    assert!(
        !ordinary.contains("(Newly Generated Coins)"),
        "false coinbase painted the parenthetical: {ordinary}"
    );
    assert!(
        !ordinary.contains("input coinbase"),
        "false coinbase was painted: {ordinary}"
    );
}

/// `nSequence` hex next to the decimal sequence line from `GET /tx/:txid`.
///
/// Angular formatHex: lowercase hex, prefixed with `0x`, and one leading
/// zero when the hex digit count is odd. A present sequence paints both
/// lines. A missing sequence omits `nSequence` and still paints the
/// transaction. Script hex and RBF stay off the screen.
#[test]
fn transaction_screen_paints_nsequence() {
    let zero = paint_tx_input("nseq0", r#","is_coinbase":false,"sequence":0"#);
    let zero_hex = zero.lines().any(|line| line == "nSequence 0x00");
    let zero_decimal = zero.lines().any(|line| line == "sequence 0");
    assert!(
        zero_hex,
        "nSequence 0x00 was absent while sequence 0 was present ({zero_decimal}): {zero}"
    );
    assert!(zero_decimal, "sequence 0 was dropped: {zero}");
    assert!(!zero.contains("deadbeef"), "{zero}");
    assert!(!zero.contains("RBF"), "{zero}");

    let padded = paint_tx_input("nseq256", r#","is_coinbase":false,"sequence":256"#);
    assert!(
        padded.lines().any(|line| line == "nSequence 0x0100"),
        "sequence 256 did not paint nSequence 0x0100: {padded}"
    );
    assert!(
        padded.lines().any(|line| line == "sequence 256"),
        "sequence 256 decimal line was dropped: {padded}"
    );
    assert!(
        !padded.lines().any(|line| line == "nSequence 0x100"),
        "sequence 256 painted the unpadded hex line: {padded}"
    );
    assert!(!padded.contains("deadbeef"), "{padded}");
    assert!(!padded.contains("RBF"), "{padded}");

    let max = paint_tx_input("nseqmax", r#","is_coinbase":false,"sequence":4294967295"#);
    assert!(
        max.lines().any(|line| line == "nSequence 0xffffffff"),
        "sequence 4294967295 did not paint nSequence 0xffffffff: {max}"
    );
    assert!(
        max.lines().any(|line| line == "sequence 4294967295"),
        "sequence 4294967295 decimal line was dropped: {max}"
    );
    assert!(
        !max.contains("FFFFFFFF") && !max.contains("0xFFFFFFFF"),
        "nSequence used uppercase hex: {max}"
    );
    assert!(!max.contains("deadbeef"), "{max}");
    assert!(!max.contains("RBF"), "{max}");

    let omitted = paint_tx_input("nseqomit", r#","is_coinbase":false"#);
    assert!(
        omitted.contains("input outpoint prev:1"),
        "transaction did not paint without sequence: {omitted}"
    );
    assert!(
        !omitted.lines().any(|line| line == "nSequence 0x00"),
        "missing sequence painted nSequence 0x00: {omitted}"
    );
    assert!(
        !omitted.contains("nSequence"),
        "missing sequence painted nSequence: {omitted}"
    );
    assert!(!omitted.contains("deadbeef"), "{omitted}");
    assert!(!omitted.contains("RBF"), "{omitted}");
}

const UNK_SIG: &str = "UNINSIGHEXDEAD";
const UNK_SIG_ASM: &str = "UNINSIGASMDEAD";
const UNK_WIT: &str = "UNINWITDEAD";
const UNK_OUT_SCRIPT: &str = "UNINOUTHEXDEAD";
const UNK_OUT_ASM: &str = "UNINOUTASMDEAD";
const UNK_PREV_SCRIPT: &str = "UNINPREVHEXDEAD";
const UNK_PREV_ASM: &str = "UNINPREVASMDEAD";
const UNK_ASSET: &str = "ce091c998b83c78bb71a632313ba3760f1763d9cfcffae02258ffa9865a37bd2";
const UNK_ENTROPY: &str = "eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee";

fn assert_unknown_sentinels_absent(painted: &str) {
    for needle in [
        UNK_SIG,
        UNK_SIG_ASM,
        UNK_WIT,
        UNK_OUT_SCRIPT,
        UNK_OUT_ASM,
        UNK_PREV_SCRIPT,
        UNK_PREV_ASM,
    ] {
        assert!(
            !painted.contains(needle),
            "{needle} leaked into the screen: {painted}"
        );
    }
}

fn assert_no_unknown_chrome(painted: &str) {
    for needle in [
        "graph",
        "tooltip",
        "skeleton",
        "mining",
        "USD",
        "$",
        "<span",
        "unknown-input",
        "pool",
        "ago",
    ] {
        assert!(!painted.contains(needle), "{needle} was painted: {painted}");
    }
}

/// `UNKNOWN` once when `prevout` is missing, from `GET /tx/:txid`.
///
/// JSON null and an omitted key are the same missing prevout. A present
/// prevout, coinbase, issuance, or peg-in does not paint it. Script hex,
/// script asm, scriptsig, and witness stay off the screen.
fn paint_unknown_input(txid: &str, prevout_field: &str, is_coinbase: bool, extra: &str) -> String {
    assert!(
        !txid.is_empty() && !txid.contains('/') && !txid.contains('?') && !txid.contains('#'),
        "{txid}"
    );
    let tx = format!(
        r#"{{"txid":"{txid}","version":1,"locktime":0,"vin":[{{"txid":"prev","vout":1,{prevout_field}"scriptsig":"{UNK_SIG}","scriptsig_asm":"{UNK_SIG_ASM}","witness":["{UNK_WIT}"],"is_coinbase":{is_coinbase},"sequence":1{extra}}}],"vout":[{{"scriptpubkey":"{UNK_OUT_SCRIPT}","scriptpubkey_asm":"{UNK_OUT_ASM}","scriptpubkey_type":"p2pkh","value":1}}],"size":10,"weight":7,"sigops":0,"fee":1,"status":{{"confirmed":false}}}}"#
    );
    let (mut host, rec) = host_with(
        Network::Mainnet,
        BTreeMap::from([(format!("/tx/{txid}"), tx)]),
    );
    host.open_transaction(txid).expect("tx");
    let paths = rec.gets();
    assert!(
        paths.iter().any(|path| path == &format!("/tx/{txid}")),
        "GET /tx/{txid} was not registered: {paths:?}"
    );
    assert!(
        paths
            .iter()
            .all(|path| !path.contains('?') && !path.contains('#')),
        "{paths:?}"
    );
    painted(&host)
}

#[test]
fn transaction_screen_paints_unknown_input() {
    let assert_one_unknown_line = |painted: &str| {
        assert_eq!(
            painted.lines().filter(|&line| line == "UNKNOWN").count(),
            1,
            "UNKNOWN was not exactly one whole line: {painted}"
        );
    };

    let null_prevout = paint_unknown_input("unknull", r#""prevout":null,"#, false, "");
    assert!(
        null_prevout.contains("input outpoint prev:1"),
        "null prevout dropped the input outpoint: {null_prevout}"
    );
    assert_eq!(
        null_prevout.matches("UNKNOWN").count(),
        1,
        "UNKNOWN is absent while the input outpoint is present: {null_prevout}"
    );
    assert_one_unknown_line(&null_prevout);
    assert_unknown_sentinels_absent(&null_prevout);
    assert_no_unknown_chrome(&null_prevout);

    let omitted = paint_unknown_input("unkomit", "", false, "");
    assert!(
        !omitted.is_empty(),
        "omitted prevout key returned an empty screen"
    );
    assert!(
        omitted.contains("input outpoint prev:1"),
        "omitted prevout key dropped the input outpoint: {omitted}"
    );
    assert_eq!(
        omitted.matches("UNKNOWN").count(),
        1,
        "omitted prevout key did not paint UNKNOWN once: {omitted}"
    );
    assert_one_unknown_line(&omitted);
    assert_unknown_sentinels_absent(&omitted);
    assert_no_unknown_chrome(&omitted);

    let typed = paint_unknown_input(
        "unktyped",
        &format!(
            r#""prevout":{{"scriptpubkey":"{UNK_PREV_SCRIPT}","scriptpubkey_asm":"{UNK_PREV_ASM}","scriptpubkey_type":"v0_p2wpkh"}},"#
        ),
        false,
        "",
    );
    assert!(
        !typed.contains("UNKNOWN"),
        "present prevout painted UNKNOWN: {typed}"
    );
    assert!(
        typed.contains("input script type v0_p2wpkh"),
        "present prevout did not paint input script type: {typed}"
    );
    assert_unknown_sentinels_absent(&typed);
    assert_no_unknown_chrome(&typed);

    let coinbase = paint_unknown_input("unkcoin", r#""prevout":null,"#, true, "");
    assert!(
        !coinbase.contains("UNKNOWN"),
        "coinbase painted UNKNOWN: {coinbase}"
    );
    assert!(
        coinbase.contains("input coinbase"),
        "coinbase input was not painted: {coinbase}"
    );
    assert_unknown_sentinels_absent(&coinbase);
    assert_no_unknown_chrome(&coinbase);

    let issuance = paint_unknown_input(
        "unkiss",
        r#""prevout":null,"#,
        false,
        &format!(
            r#","issuance":{{"asset_id":"{UNK_ASSET}","is_reissuance":false,"asset_entropy":"{UNK_ENTROPY}"}}"#
        ),
    );
    assert!(
        !issuance.contains("UNKNOWN"),
        "issuance painted UNKNOWN: {issuance}"
    );
    assert!(
        issuance.contains(&format!("issuance asset {UNK_ASSET}")),
        "issuance asset was not painted: {issuance}"
    );
    assert_unknown_sentinels_absent(&issuance);
    assert_no_unknown_chrome(&issuance);

    let peg_in = paint_unknown_input(
        "unkpegin",
        r#""prevout":null,"#,
        false,
        r#","is_pegin":true"#,
    );
    assert!(
        !peg_in.contains("UNKNOWN"),
        "peg-in painted UNKNOWN: {peg_in}"
    );
    assert_unknown_sentinels_absent(&peg_in);
    assert_no_unknown_chrome(&peg_in);

    let peg_false = paint_unknown_input(
        "unkpegfalse",
        r#""prevout":null,"#,
        false,
        r#","is_pegin":false"#,
    );
    assert!(
        peg_false.contains("input outpoint prev:1"),
        "is_pegin false dropped the input outpoint: {peg_false}"
    );
    assert_eq!(
        peg_false.matches("UNKNOWN").count(),
        1,
        "is_pegin false did not paint UNKNOWN once: {peg_false}"
    );
    assert_one_unknown_line(&peg_false);
    assert_unknown_sentinels_absent(&peg_false);
    assert_no_unknown_chrome(&peg_false);
}

const P2SH_SIG: &str = "P2SHSIGHEXDEAD";
const P2SH_SIG_ASM: &str = "P2SHSIGASMDEAD";
const P2SH_WIT: &str = "P2SHWITDEAD";
const P2SH_PUB: &str = "P2SHPUBHEXDEAD";
const P2SH_PUB_ASM: &str = "P2SHPUBASMDEAD";
const P2SH_WIT_ASM: &str = "P2SHWITASMDEAD";

enum RedeemField {
    Present(&'static str),
    Missing,
    Null,
    Empty,
}

fn p2sh_redeem_body(txid: &str, redeem: RedeemField) -> String {
    let redeem_field = match redeem {
        RedeemField::Present(asm) => format!(r#","inner_redeemscript_asm":"{asm}""#),
        RedeemField::Missing => String::new(),
        RedeemField::Null => r#","inner_redeemscript_asm":null"#.to_string(),
        RedeemField::Empty => r#","inner_redeemscript_asm":"""#.to_string(),
    };
    format!(
        r#"{{"txid":"{txid}","version":1,"locktime":0,"vin":[{{"txid":"bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb","vout":0,"prevout":null,"scriptsig":"{P2SH_SIG}","scriptsig_asm":"{P2SH_SIG_ASM}","witness":["{P2SH_WIT}"],"is_coinbase":false,"sequence":1,"inner_witnessscript_asm":"{P2SH_WIT_ASM}"{redeem_field}}}],"vout":[{{"scriptpubkey":"{P2SH_PUB}","scriptpubkey_asm":"{P2SH_PUB_ASM}","scriptpubkey_type":"p2pkh","value":1}}],"size":10,"weight":7,"sigops":0,"fee":1}}"#
    )
}

fn paint_p2sh_redeem(txid: &str, redeem: RedeemField) -> String {
    assert!(
        !txid.is_empty() && !txid.contains('/') && !txid.contains('?') && !txid.contains('#'),
        "{txid}"
    );
    let body = p2sh_redeem_body(txid, redeem);
    for needle in [
        P2SH_SIG,
        P2SH_SIG_ASM,
        P2SH_WIT,
        P2SH_PUB,
        P2SH_PUB_ASM,
        P2SH_WIT_ASM,
    ] {
        assert!(body.contains(needle), "fixture omitted {needle}: {body}");
    }
    let (mut host, rec) = host_with(
        Network::Mainnet,
        BTreeMap::from([(format!("/tx/{txid}"), body)]),
    );
    host.open_transaction(txid).expect("tx");
    let paths = rec.gets();
    assert!(
        paths.iter().any(|path| path == &format!("/tx/{txid}")),
        "GET /tx/{txid} was not registered: {paths:?}"
    );
    assert!(
        paths
            .iter()
            .all(|path| !path.contains('?') && !path.contains('#')),
        "{paths:?}"
    );
    painted(&host)
}

fn p2sh_line_count(painted: &str, line: &str) -> usize {
    painted.lines().filter(|row| *row == line).count()
}

fn assert_p2sh_sentinels_absent(painted: &str) {
    for needle in [
        P2SH_SIG,
        P2SH_SIG_ASM,
        P2SH_WIT,
        P2SH_PUB,
        P2SH_PUB_ASM,
        P2SH_WIT_ASM,
    ] {
        assert!(
            !painted.contains(needle),
            "{needle} leaked into the screen: {painted}"
        );
    }
}

fn assert_no_p2sh_html(painted: &str) {
    for needle in ["<span", "p2sh-redeem-label", "&lt;", "unknown-input"] {
        assert!(!painted.contains(needle), "{needle} was painted: {painted}");
    }
}

/// `P2SH redeem script` from `GET /tx/:txid` when `inner_redeemscript_asm` is present.
///
/// A non-empty asm paints one plaintext line, and angle brackets stay as text.
/// A missing key, JSON null, and an empty string omit the label. Script hex,
/// scriptsig, scriptsig asm, witness, scriptpubkey, scriptpubkey asm, and the
/// inner witness script stay off the screen.
#[test]
fn transaction_screen_paints_p2sh_redeem_script() {
    let present = paint_p2sh_redeem("p2shdup", RedeemField::Present("OP_DUP OP_HASH160"));
    assert!(
        present.contains("input outpoint"),
        "present asm dropped the input outpoint: {present}"
    );
    assert_eq!(
        p2sh_line_count(&present, "P2SH redeem script OP_DUP OP_HASH160"),
        1,
        "present asm was not exactly one line `P2SH redeem script OP_DUP OP_HASH160`: {present}"
    );
    assert_eq!(
        present.matches("P2SH redeem script").count(),
        1,
        "the label was not painted once: {present}"
    );
    assert_p2sh_sentinels_absent(&present);
    assert_no_p2sh_html(&present);

    let brackets = paint_p2sh_redeem("p2shbrack", RedeemField::Present("OP_DUP <OP_HASH160>"));
    assert_eq!(
        p2sh_line_count(&brackets, "P2SH redeem script OP_DUP <OP_HASH160>"),
        1,
        "angle brackets were not one plaintext line `P2SH redeem script OP_DUP <OP_HASH160>`: {brackets}"
    );
    assert_eq!(
        brackets.matches("P2SH redeem script").count(),
        1,
        "the angle-bracket label was not painted once: {brackets}"
    );
    assert_no_p2sh_html(&brackets);
    assert_p2sh_sentinels_absent(&brackets);

    let missing = paint_p2sh_redeem("p2shmiss", RedeemField::Missing);
    assert!(
        missing.contains("input outpoint"),
        "missing key dropped the input outpoint: {missing}"
    );
    assert!(
        !missing.contains("P2SH redeem script"),
        "missing key painted the label: {missing}"
    );
    assert_p2sh_sentinels_absent(&missing);
    assert_no_p2sh_html(&missing);

    let null = paint_p2sh_redeem("p2shnull", RedeemField::Null);
    assert!(
        null.contains("input outpoint"),
        "JSON null dropped the input outpoint: {null}"
    );
    assert!(
        !null.contains("P2SH redeem script"),
        "JSON null painted the label: {null}"
    );
    assert_p2sh_sentinels_absent(&null);
    assert_no_p2sh_html(&null);

    let empty = paint_p2sh_redeem("p2shempty", RedeemField::Empty);
    assert!(
        empty.contains("input outpoint"),
        "empty string dropped the input outpoint: {empty}"
    );
    assert!(
        !empty.contains("P2SH redeem script"),
        "empty string painted the label: {empty}"
    );
    assert_p2sh_sentinels_absent(&empty);
    assert_no_p2sh_html(&empty);
}

const P2TR_LABEL: &str = "P2TR tapscript";
const P2TR_ASM: &str = "TAPSCRIPTASMDEAD";
const P2TR_SIG: &str = "TAPSIGHEXDEAD";
const P2TR_SIG_ASM: &str = "TAPSIGASMDEAD";
const P2TR_WIT: &str = "TAPWITDEAD";
const P2TR_PUB: &str = "TAPPUBHEXDEAD";
const P2TR_PUB_ASM: &str = "TAPPUBASMDEAD";
const P2TR_LEAF: &str = "TAPLEAFVERDEAD";

fn p2tr_tx_body(
    txid: &str,
    prevout_type: Option<&str>,
    inner_witness: bool,
    redeem_asm: Option<&str>,
    vout_type: &str,
) -> String {
    let prevout_field = match prevout_type {
        Some(kind) => format!(
            r#""prevout":{{"scriptpubkey":"{P2TR_PUB}","scriptpubkey_asm":"{P2TR_PUB_ASM}","scriptpubkey_type":"{kind}","value":5}},"#
        ),
        None => r#""prevout":null,"#.to_string(),
    };
    let witness_field = if inner_witness {
        format!(r#","inner_witnessscript_asm":"{P2TR_ASM}""#)
    } else {
        String::new()
    };
    let redeem_field = match redeem_asm {
        Some(asm) => format!(r#","inner_redeemscript_asm":"{asm}""#),
        None => String::new(),
    };
    let vin = format!(
        r#"{{"txid":"cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc","vout":0,{prevout_field}"scriptsig":"{P2TR_SIG}","scriptsig_asm":"{P2TR_SIG_ASM}","witness":["{P2TR_WIT}"],"is_coinbase":false,"sequence":1,"leaf_version":"{P2TR_LEAF}"{witness_field}{redeem_field}}}"#
    );
    format!(
        r#"{{"txid":"{txid}","version":1,"locktime":0,"vin":[{vin}],"vout":[{{"scriptpubkey":"{P2TR_PUB}","scriptpubkey_asm":"{P2TR_PUB_ASM}","scriptpubkey_type":"{vout_type}","value":1}}],"size":10,"weight":7,"sigops":0,"fee":1}}"#
    )
}

fn paint_p2tr(
    txid: &str,
    prevout_type: Option<&str>,
    inner_witness: bool,
    redeem_asm: Option<&str>,
    vout_type: &str,
) -> String {
    assert!(
        !txid.is_empty() && !txid.contains('/') && !txid.contains('?') && !txid.contains('#'),
        "{txid}"
    );
    let body = p2tr_tx_body(txid, prevout_type, inner_witness, redeem_asm, vout_type);
    for needle in [
        P2TR_SIG,
        P2TR_SIG_ASM,
        P2TR_WIT,
        P2TR_PUB,
        P2TR_PUB_ASM,
        P2TR_LEAF,
    ] {
        assert!(body.contains(needle), "fixture omitted {needle}: {body}");
    }
    if inner_witness {
        assert!(
            body.contains(P2TR_ASM),
            "fixture omitted {P2TR_ASM}: {body}"
        );
    } else {
        assert!(
            !body.contains(P2TR_ASM),
            "fixture included {P2TR_ASM}: {body}"
        );
    }
    let (mut host, rec) = host_with(
        Network::Mainnet,
        BTreeMap::from([(format!("/tx/{txid}"), body)]),
    );
    host.open_transaction(txid).expect("tx");
    let paths = rec.gets();
    assert!(
        paths.iter().any(|path| path == &format!("/tx/{txid}")),
        "GET /tx/{txid} was not registered: {paths:?}"
    );
    assert!(
        paths
            .iter()
            .all(|path| !path.contains('?') && !path.contains('#')),
        "{paths:?}"
    );
    painted(&host)
}

fn p2tr_line_count(painted: &str) -> usize {
    painted.lines().filter(|line| *line == P2TR_LABEL).count()
}

fn assert_p2tr_script_stays_off(painted: &str) {
    for needle in [
        P2TR_ASM,
        P2TR_SIG,
        P2TR_SIG_ASM,
        P2TR_WIT,
        P2TR_PUB,
        P2TR_PUB_ASM,
        P2TR_LEAF,
        "leaf version",
        "Leaf version",
        "P2TR control block",
        "P2TR Simplicity script",
        "P2WSH witness script",
    ] {
        assert!(
            !painted.contains(needle),
            "{needle} leaked into the screen: {painted}"
        );
    }
}

/// `P2TR tapscript` from `GET /tx/:txid` when `prevout.scriptpubkey_type` is `v1_p2tr`.
///
/// The line is exactly `P2TR tapscript`, once for that input. A p2sh input does
/// not gain it. Any other prevout type does not gain it. An output whose type
/// is `v1_p2tr` does not gain it. The inner witness script, script asm, script
/// hex, scriptsig, witness, and leaf version stay off the screen. `P2SH redeem
/// script` stays on a p2sh input that has a redeem script.
#[test]
fn transaction_screen_paints_p2tr_tapscript_label() {
    let present = paint_p2tr("p2trtap", Some("v1_p2tr"), true, None, "p2pkh");
    assert!(
        present.contains("input outpoint"),
        "v1_p2tr dropped the input outpoint: {present}"
    );
    assert_eq!(
        p2tr_line_count(&present),
        1,
        "v1_p2tr was not exactly one line `P2TR tapscript`: {present}"
    );
    assert_eq!(
        present.matches(P2TR_LABEL).count(),
        1,
        "the label was not painted once: {present}"
    );
    assert!(
        present
            .lines()
            .any(|line| line == "Previous output type V1_P2TR"),
        "Previous output type was dropped: {present}"
    );
    assert!(
        !present.contains("P2SH redeem script"),
        "a v1_p2tr input gained P2SH redeem script: {present}"
    );
    assert_p2tr_script_stays_off(&present);

    let bare = paint_p2tr("p2trbare", Some("v1_p2tr"), false, None, "p2pkh");
    assert_eq!(
        p2tr_line_count(&bare),
        1,
        "a v1_p2tr input without an inner script was not one line `P2TR tapscript`: {bare}"
    );
    assert_eq!(
        bare.matches(P2TR_LABEL).count(),
        1,
        "the bare v1_p2tr label was not painted once: {bare}"
    );
    assert_p2tr_script_stays_off(&bare);

    let p2sh = paint_p2tr(
        "p2trnotsh",
        Some("p2sh"),
        true,
        Some("OP_DUP OP_HASH160"),
        "p2pkh",
    );
    assert_eq!(
        p2tr_line_count(&p2sh),
        0,
        "a p2sh input gained `P2TR tapscript`: {p2sh}"
    );
    assert!(
        !p2sh.contains(P2TR_LABEL),
        "a p2sh input gained `P2TR tapscript`: {p2sh}"
    );
    assert_eq!(
        p2sh.lines()
            .filter(|line| *line == "P2SH redeem script OP_DUP OP_HASH160")
            .count(),
        1,
        "P2SH redeem script was dropped: {p2sh}"
    );
    assert_p2tr_script_stays_off(&p2sh);

    let other = paint_p2tr("p2trwsh", Some("v0_p2wsh"), true, None, "p2pkh");
    assert_eq!(
        p2tr_line_count(&other),
        0,
        "a non-v1_p2tr input gained `P2TR tapscript`: {other}"
    );
    assert!(
        !other.contains(P2TR_LABEL),
        "a non-v1_p2tr input gained `P2TR tapscript`: {other}"
    );
    assert!(
        other.contains("input outpoint"),
        "a non-v1_p2tr input dropped the outpoint: {other}"
    );
    assert_p2tr_script_stays_off(&other);

    let missing_prevout = paint_p2tr("p2trnoprev", None, true, None, "p2pkh");
    assert_eq!(
        p2tr_line_count(&missing_prevout),
        0,
        "a missing prevout gained `P2TR tapscript`: {missing_prevout}"
    );
    assert!(
        !missing_prevout.contains(P2TR_LABEL),
        "a missing prevout gained `P2TR tapscript`: {missing_prevout}"
    );
    assert!(
        missing_prevout.contains("input outpoint"),
        "a missing prevout dropped the outpoint: {missing_prevout}"
    );
    assert_p2tr_script_stays_off(&missing_prevout);

    let output_only = paint_p2tr("p2trout", Some("v0_p2wpkh"), true, None, "v1_p2tr");
    assert_eq!(
        p2tr_line_count(&output_only),
        0,
        "an output of type v1_p2tr gained `P2TR tapscript`: {output_only}"
    );
    assert!(
        !output_only.contains(P2TR_LABEL),
        "an output of type v1_p2tr gained `P2TR tapscript`: {output_only}"
    );
    assert!(
        output_only.lines().any(|line| line == "Type V1_P2TR"),
        "the output type line was dropped: {output_only}"
    );
    assert_p2tr_script_stays_off(&output_only);
}

const PREVIOUS_OUTPUT_SCRIPT: &str = "Previous output script";
const POS_ASM: &str = "POSASMDEAD";
const POS_HEX: &str = "POSHEXDEAD";
const POS_SIG: &str = "POSSIGHEXDEAD";
const POS_SIG_ASM: &str = "POSSIGASMDEAD";
const POS_WIT: &str = "POSWITDEAD";
const POS_INNER: &str = "POSINNERDEAD";
const POS_LEAF: &str = "POSLEAFDEAD";

fn pos_line_count(painted: &str) -> usize {
    painted
        .lines()
        .filter(|line| *line == PREVIOUS_OUTPUT_SCRIPT)
        .count()
}

fn assert_pos_bytes_stay_off(painted: &str) {
    for needle in [
        POS_ASM,
        POS_HEX,
        POS_SIG,
        POS_SIG_ASM,
        POS_WIT,
        POS_INNER,
        POS_LEAF,
        "scriptpubkey_asm",
        "script asm",
        "leaf version",
        "Leaf version",
    ] {
        assert!(
            !painted.contains(needle),
            "{needle} leaked into the screen: {painted}"
        );
    }
}

fn pos_prevout(kind: &str) -> String {
    format!(
        r#""prevout":{{"scriptpubkey":"{POS_HEX}","scriptpubkey_asm":"{POS_ASM}","scriptpubkey_type":"{kind}","value":5}},"#
    )
}

fn pos_vin(prevout_field: &str, extra: &str) -> String {
    format!(
        r#"{{"txid":"eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee","vout":0,{prevout_field}"scriptsig":"{POS_SIG}","scriptsig_asm":"{POS_SIG_ASM}","witness":["{POS_WIT}"],"is_coinbase":false,"sequence":1{extra}}}"#
    )
}

fn paint_previous_output_script(txid: &str, vin: &str) -> String {
    assert!(
        !txid.is_empty() && !txid.contains('/') && !txid.contains('?') && !txid.contains('#'),
        "{txid}"
    );
    let body = format!(
        r#"{{"txid":"{txid}","version":1,"locktime":0,"vin":[{vin}],"vout":[{{"scriptpubkey":"{POS_HEX}","scriptpubkey_asm":"{POS_ASM}","scriptpubkey_type":"p2pkh","value":1}}],"size":10,"weight":7,"sigops":0,"fee":1}}"#
    );
    for needle in [POS_ASM, POS_HEX, POS_SIG, POS_SIG_ASM, POS_WIT] {
        assert!(body.contains(needle), "fixture omitted {needle}: {body}");
    }
    let (mut host, rec) = host_with(
        Network::Mainnet,
        BTreeMap::from([(format!("/tx/{txid}"), body)]),
    );
    host.open_transaction(txid).expect("tx");
    let paths = rec.gets();
    assert!(
        paths.iter().any(|path| path == &format!("/tx/{txid}")),
        "GET /tx/{txid} was not registered: {paths:?}"
    );
    assert!(
        paths
            .iter()
            .all(|path| !path.contains('?') && !path.contains('#')),
        "{paths:?}"
    );
    painted(&host)
}

/// `Previous output script` from `GET /tx/:txid` when the input has a prevout.
///
/// The line is exactly those words, once per previous output. A missing prevout
/// does not get it. Script hex, script asm, scriptsig, witness, the inner
/// tapscript string, and leaf version stay off the screen. A p2sh input still
/// paints `P2SH redeem script`. A v1_p2tr input still paints `P2TR tapscript`.
/// The push screen does not gain the words.
#[test]
fn transaction_screen_paints_previous_output_script() {
    let present = paint_previous_output_script("posone", &pos_vin(&pos_prevout("v0_p2wpkh"), ""));
    assert!(
        present.contains("input outpoint"),
        "a previous output dropped the input outpoint: {present}"
    );
    assert_eq!(
        pos_line_count(&present),
        1,
        "a previous output was not exactly one line `Previous output script`: {present}"
    );
    assert_eq!(
        present.matches(PREVIOUS_OUTPUT_SCRIPT).count(),
        1,
        "the label was not painted once: {present}"
    );
    assert!(
        present
            .lines()
            .any(|line| line == "Previous output type V0_P2WPKH"),
        "Previous output type was dropped: {present}"
    );
    assert!(!present.contains("P2TR tapscript"), "{present}");
    assert!(!present.contains("P2SH redeem script"), "{present}");
    assert_pos_bytes_stay_off(&present);

    let two = paint_previous_output_script(
        "postwo",
        &format!(
            "{},{}",
            pos_vin(&pos_prevout("v0_p2wpkh"), ""),
            pos_vin(&pos_prevout("p2sh"), "")
        ),
    );
    assert_eq!(
        pos_line_count(&two),
        2,
        "two previous outputs were not two lines `Previous output script`: {two}"
    );
    assert_eq!(
        two.matches(PREVIOUS_OUTPUT_SCRIPT).count(),
        2,
        "the label was not painted twice: {two}"
    );
    assert_pos_bytes_stay_off(&two);

    let missing = paint_previous_output_script("posnull", &pos_vin(r#""prevout":null,"#, ""));
    assert!(
        missing.contains("input outpoint"),
        "a missing prevout dropped the input outpoint: {missing}"
    );
    assert_eq!(
        pos_line_count(&missing),
        0,
        "a missing prevout gained `Previous output script`: {missing}"
    );
    assert!(
        !missing.contains(PREVIOUS_OUTPUT_SCRIPT),
        "a missing prevout gained `Previous output script`: {missing}"
    );
    assert_pos_bytes_stay_off(&missing);

    let p2sh = paint_previous_output_script(
        "posp2sh",
        &pos_vin(
            &pos_prevout("p2sh"),
            r#","inner_redeemscript_asm":"OP_DUP OP_HASH160""#,
        ),
    );
    assert_eq!(
        pos_line_count(&p2sh),
        1,
        "a p2sh previous output was not one line `Previous output script`: {p2sh}"
    );
    assert_eq!(
        p2sh.lines()
            .filter(|line| *line == "P2SH redeem script OP_DUP OP_HASH160")
            .count(),
        1,
        "P2SH redeem script was dropped: {p2sh}"
    );
    assert!(!p2sh.contains("P2TR tapscript"), "{p2sh}");
    assert_pos_bytes_stay_off(&p2sh);

    let p2sh_missing = paint_previous_output_script(
        "posp2shnull",
        &pos_vin(
            r#""prevout":null,"#,
            r#","inner_redeemscript_asm":"OP_DUP OP_HASH160""#,
        ),
    );
    assert_eq!(
        pos_line_count(&p2sh_missing),
        0,
        "a p2sh input with no prevout gained `Previous output script`: {p2sh_missing}"
    );
    assert_eq!(
        p2sh_missing
            .lines()
            .filter(|line| *line == "P2SH redeem script OP_DUP OP_HASH160")
            .count(),
        1,
        "P2SH redeem script was dropped: {p2sh_missing}"
    );
    assert_pos_bytes_stay_off(&p2sh_missing);

    let p2tr = paint_previous_output_script(
        "posp2tr",
        &pos_vin(
            &pos_prevout("v1_p2tr"),
            &format!(r#","inner_witnessscript_asm":"{POS_INNER}","leaf_version":"{POS_LEAF}""#),
        ),
    );
    assert_eq!(
        pos_line_count(&p2tr),
        1,
        "a v1_p2tr previous output was not one line `Previous output script`: {p2tr}"
    );
    assert_eq!(
        p2tr.lines()
            .filter(|line| *line == "P2TR tapscript")
            .count(),
        1,
        "P2TR tapscript was dropped: {p2tr}"
    );
    assert!(!p2tr.contains("P2SH redeem script"), "{p2tr}");
    assert_pos_bytes_stay_off(&p2tr);

    let (mut host, _) = host_with(Network::Mainnet, BTreeMap::new());
    host.activate(Screen::Broadcast).expect("broadcast");
    let push = painted(&host);
    assert!(
        !push.contains(PREVIOUS_OUTPUT_SCRIPT),
        "the push screen painted Previous output script: {push}"
    );
    assert_eq!(
        push.lines()
            .filter(|line| *line == "Broadcast Transaction")
            .count(),
        1,
        "Broadcast Transaction was dropped: {push}"
    );
}

fn paint_tx_input(txid: &str, input_tail: &str) -> String {
    let tx = format!(
        r#"{{
        "txid":"{txid}",
        "version":2,
        "locktime":1,
        "vin":[{{
            "txid":"prev",
            "vout":1,
            "prevout":null,
            "scriptsig":"deadbeef",
            "scriptsig_asm":"OP_CHECKSIG"{input_tail}
        }}],
        "vout":[{{
            "scriptpubkey":"51",
            "scriptpubkey_asm":"OP_TRUE",
            "scriptpubkey_type":"op_return",
            "value":9
        }}],
        "size":10,
        "weight":40,
        "sigops":1,
        "fee":1,
        "status":{{"confirmed":false}}
    }}"#
    );
    let (mut host, _) = host_with(
        Network::Mainnet,
        BTreeMap::from([(format!("/tx/{txid}"), tx)]),
    );
    host.open_transaction(txid).expect("tx");
    painted(&host)
}

/// Virtual size is `weight / 4` from `GET /tx/:txid`, integer division.
///
/// A present weight paints that quotient, including zero. Weight 3, 5, and 7
/// do not round up. A missing key or JSON null omits the label and is not zero.
/// The header weight comes from that same document. A missing key or JSON null
/// does not paint `weight 0`. An explicit zero still does. Script hex and asm
/// stay off the screen.
#[test]
fn transaction_screen_paints_virtual_size() {
    let five = paint_tx_weight("vsize5", r#""weight":5"#);
    assert!(
        five.lines().any(|line| line == "virtual size 1"),
        "weight 5 did not paint virtual size 1: {five}"
    );
    assert!(
        !five.lines().any(|line| line == "virtual size 2"),
        "weight 5 rounded up: {five}"
    );
    assert!(
        five.contains("weight 5"),
        "weight 5 text was dropped: {five}"
    );
    assert!(!five.contains("0014aa"), "{five}");
    assert!(!five.contains("deadbeef"), "{five}");
    assert_op_return_asm_stays_off(&five);
    assert!(!five.contains("OP_CHECKSIG"), "{five}");
    assert!(!five.contains("abcdwitnesshex"), "{five}");
    assert!(!five.contains("scriptpubkey"), "{five}");

    let seven = paint_tx_weight("vsize7", r#""weight":7"#);
    assert!(
        seven.lines().any(|line| line == "virtual size 1"),
        "weight 7 did not paint virtual size 1: {seven}"
    );
    assert!(
        !seven.lines().any(|line| line == "virtual size 2"),
        "weight 7 rounded up: {seven}"
    );

    let zero = paint_tx_weight("vsize0", r#""weight":0"#);
    assert!(
        zero.lines().any(|line| line == "virtual size 0"),
        "weight 0 omitted virtual size: {zero}"
    );
    assert!(
        zero.contains("weight 0"),
        "explicit weight 0 was not painted: {zero}"
    );
    assert!(
        !zero.contains("fee_rate"),
        "weight 0 painted a fee rate: {zero}"
    );

    let three = paint_tx_weight("vsize3", r#""weight":3"#);
    assert!(
        three.lines().any(|line| line == "virtual size 0"),
        "weight 3 did not paint virtual size 0: {three}"
    );
    assert!(
        !three.lines().any(|line| line == "virtual size 1"),
        "weight 3 rounded up: {three}"
    );

    let absent = paint_tx_weight("vsizeabs", "");
    assert!(
        absent.contains("tx txid vsizeabs"),
        "missing weight hid the transaction: {absent}"
    );
    assert!(
        !absent.contains("virtual size"),
        "missing weight painted virtual size: {absent}"
    );
    assert!(
        !absent.contains("virtual size 0"),
        "missing weight was painted as 0: {absent}"
    );
    assert!(
        !absent.contains("weight 0"),
        "missing weight was painted as weight 0: {absent}"
    );

    let null = paint_tx_weight("vsizenull", r#""weight":null"#);
    assert!(
        null.contains("tx txid vsizenull"),
        "null weight hid the transaction: {null}"
    );
    assert!(
        !null.contains("virtual size"),
        "null weight painted virtual size: {null}"
    );
    assert!(
        !null.contains("virtual size 0"),
        "null weight was painted as 0: {null}"
    );
    assert!(
        !null.contains("weight 0"),
        "null weight was painted as weight 0: {null}"
    );
}

fn paint_tx_weight(txid: &str, weight_field: &str) -> String {
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
        "sigops":0,
        "fee":1,
        "status":{{"confirmed":false}}
    }}"#
    );
    let (mut host, _) = host_with(
        Network::Mainnet,
        BTreeMap::from([(format!("/tx/{txid}"), tx)]),
    );
    host.open_transaction(txid)
        .unwrap_or_else(|err| panic!("tx {txid} did not open: {err}"));
    painted(&host)
}

/// Confirmation count is the tip height minus `status.block_height`, plus 1.
///
/// The label is `Confirmations`. Both heights are present counts, including
/// zero. A missing block height is unconfirmed and omits the line. It is not
/// zero. A missing tip, an empty tip body, or a non-integer tip omits the
/// line and does not fail the open. The subtraction is signed. Script hex
/// and asm stay off the screen.
#[test]
fn transaction_screen_paints_confirmation_count() {
    let in_tip = paint_tx_confirmations("confintip", Some("800000"), Some(800000));
    assert!(
        in_tip.lines().any(|line| line == "Confirmations 1"),
        "tip height did not paint Confirmations 1: {in_tip}"
    );

    let behind = paint_tx_confirmations("confbehind", Some("800000"), Some(799998));
    assert!(
        behind.lines().any(|line| line == "Confirmations 3"),
        "height 799998 did not paint Confirmations 3: {behind}"
    );
    assert!(
        !behind.lines().any(|line| line == "Confirmations 2"),
        "height 799998 painted Confirmations 2: {behind}"
    );

    let unconfirmed = paint_tx_confirmations("confunconf", Some("800000"), None);
    assert!(
        unconfirmed.contains("tx txid confunconf"),
        "missing block height hid the transaction: {unconfirmed}"
    );
    assert!(
        !unconfirmed.contains("Confirmations"),
        "missing block height painted Confirmations: {unconfirmed}"
    );
    assert!(
        !unconfirmed.contains("Confirmations 800001"),
        "missing block height was painted as zero: {unconfirmed}"
    );

    let no_tip = paint_tx_confirmations("confnotip", None, Some(800000));
    assert!(
        no_tip.contains("tx txid confnotip"),
        "missing tip hid the transaction: {no_tip}"
    );
    assert!(
        !no_tip.contains("Confirmations"),
        "missing tip painted Confirmations: {no_tip}"
    );
    assert!(
        !no_tip.contains("Confirmations 0"),
        "missing tip was painted as 0: {no_tip}"
    );

    let zeros = paint_tx_confirmations("confzero", Some("0"), Some(0));
    assert!(
        zeros.lines().any(|line| line == "Confirmations 1"),
        "present zero heights omitted Confirmations 1: {zeros}"
    );

    let gap = paint_tx_confirmations("confgap", Some("100"), Some(90));
    assert!(
        gap.lines().any(|line| line == "Confirmations 11"),
        "tip 100 and height 90 did not paint Confirmations 11: {gap}"
    );

    let bad_tip = paint_tx_confirmations("confbadtip", Some("not-a-height"), Some(90));
    assert!(
        bad_tip.contains("tx txid confbadtip"),
        "a non-integer tip hid the transaction: {bad_tip}"
    );
    assert!(
        !bad_tip.contains("Confirmations"),
        "a non-integer tip painted Confirmations: {bad_tip}"
    );

    let empty_tip = paint_tx_confirmations("confemptytip", Some(""), Some(90));
    assert!(
        empty_tip.contains("tx txid confemptytip"),
        "an empty tip hid the transaction: {empty_tip}"
    );
    assert!(
        !empty_tip.contains("Confirmations"),
        "an empty tip painted Confirmations: {empty_tip}"
    );
}

fn paint_tx_confirmations(txid: &str, tip: Option<&str>, block_height: Option<u64>) -> String {
    let status = match block_height {
        Some(height) => format!(r#""status":{{"confirmed":true,"block_height":{height}}}"#),
        None => r#""status":{"confirmed":false}"#.to_string(),
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
            "is_coinbase":false,
            "sequence":1
        }}],
        "vout":[{{
            "scriptpubkey":"0014aa",
            "scriptpubkey_asm":"OP_RETURN",
            "scriptpubkey_type":"op_return",
            "value":9,
            "weight":4
        }}],
        "size":10,
        "weight":40,
        "sigops":0,
        "fee":1,
        {status}
    }}"#
    );
    let mut bodies = BTreeMap::from([(format!("/tx/{txid}"), tx)]);
    if let Some(tip) = tip {
        bodies.insert("/blocks/tip/height".to_string(), tip.to_string());
    }
    let (mut host, _) = host_with(Network::Mainnet, bodies);
    host.open_transaction(txid)
        .unwrap_or_else(|err| panic!("tx {txid} did not open: {err}"));
    painted(&host)
}

/// A block height whose tip does not parse paints the plaintext word Confirmed once.
///
/// A parsed tip with a block height paints only the count. A parsed tip with
/// no block height does not paint Confirmed.
#[test]
fn transaction_screen_paints_confirmed_when_tip_does_not_parse() {
    let bad_tip = paint_tx_confirmations("confwordbad", Some("not-a-height"), Some(90));
    assert_eq!(
        bad_tip.lines().filter(|line| *line == "Confirmed").count(),
        1,
        "a non-integer tip with a block height did not paint Confirmed once: {bad_tip}"
    );
    assert!(
        bad_tip.contains("tx txid confwordbad"),
        "a non-integer tip hid the transaction: {bad_tip}"
    );
    assert!(
        !bad_tip.lines().any(|line| {
            line.starts_with("confirmations ") || line.starts_with("Confirmations ")
        }),
        "a non-integer tip painted the count: {bad_tip}"
    );

    let empty_tip = paint_tx_confirmations("confwordempty", Some(""), Some(90));
    assert_eq!(
        empty_tip
            .lines()
            .filter(|line| *line == "Confirmed")
            .count(),
        1,
        "an empty tip with a block height did not paint Confirmed once: {empty_tip}"
    );

    let missing_tip = paint_tx_confirmations("confwordmiss", None, Some(90));
    assert_eq!(
        missing_tip
            .lines()
            .filter(|line| *line == "Confirmed")
            .count(),
        1,
        "a missing tip with a block height did not paint Confirmed once: {missing_tip}"
    );

    let counted = paint_tx_confirmations("confwordgap", Some("100"), Some(90));
    assert!(
        counted.lines().any(|line| line == "confirmations 11"),
        "tip 100 and height 90 did not paint only the count: {counted}"
    );
    assert!(
        !counted.lines().any(|line| line == "Confirmed"),
        "a parsed tip with a block height painted Confirmed: {counted}"
    );

    let no_height = paint_tx_confirmations("confwordnone", Some("100"), None);
    assert!(
        no_height.contains("tx txid confwordnone"),
        "a missing block height hid the transaction: {no_height}"
    );
    assert!(
        !no_height.lines().any(|line| line == "Confirmed"),
        "a parsed tip with no block height painted Confirmed: {no_height}"
    );
}

/// A parsed tip with no block height paints the plaintext word Unconfirmed once.
///
/// No tip and no block height stay empty. A parsed tip with a block height
/// still paints only the count. A block height whose tip does not parse still
/// paints Confirmed, not Unconfirmed.
#[test]
fn transaction_screen_paints_unconfirmed() {
    let no_height = paint_tx_confirmations("unconfword", Some("100"), None);
    assert_eq!(
        no_height
            .lines()
            .filter(|line| *line == "Unconfirmed")
            .count(),
        1,
        "a parsed tip with no block height did not paint Unconfirmed once: {no_height}"
    );
    assert!(
        no_height.contains("tx txid unconfword"),
        "a missing block height hid the transaction: {no_height}"
    );
    assert!(
        !no_height.lines().any(|line| line == "Confirmed"),
        "a parsed tip with no block height painted Confirmed: {no_height}"
    );

    let neither = paint_tx_confirmations("unconfneither", None, None);
    assert!(
        neither.contains("tx txid unconfneither"),
        "a missing tip and a missing block height hid the transaction: {neither}"
    );
    assert!(
        !neither
            .lines()
            .any(|line| line == "Unconfirmed" || line == "Confirmed"),
        "no tip and no block height painted a confirmation word: {neither}"
    );

    let counted = paint_tx_confirmations("unconfgap", Some("100"), Some(90));
    assert!(
        counted.lines().any(|line| line == "confirmations 11"),
        "tip 100 and height 90 did not paint only the count: {counted}"
    );
    assert!(
        !counted
            .lines()
            .any(|line| line == "Unconfirmed" || line == "Confirmed"),
        "a parsed tip with a block height painted a confirmation word: {counted}"
    );

    let bad_tip = paint_tx_confirmations("unconfbad", Some("not-a-height"), Some(90));
    assert_eq!(
        bad_tip.lines().filter(|line| *line == "Confirmed").count(),
        1,
        "a non-integer tip with a block height did not paint Confirmed once: {bad_tip}"
    );
    assert!(
        !bad_tip.lines().any(|line| line == "Unconfirmed"),
        "a non-integer tip painted Unconfirmed: {bad_tip}"
    );
}

const INPUTS_OUTPUTS_HEADING: &str = "Inputs & Outputs";

fn tx_with_parties(txid: &str, vin: &str, vout: &str) -> String {
    format!(
        r#"{{
        "txid":"{txid}",
        "version":1,
        "locktime":0,
        "vin":[{vin}],
        "vout":[{vout}],
        "size":10,
        "weight":40,
        "sigops":0,
        "fee":1,
        "status":{{"confirmed":false}}
    }}"#
    )
}

fn paint_tx_body(txid: &str, body: &str) -> String {
    let (mut host, _) = host_with(
        Network::Mainnet,
        BTreeMap::from([(format!("/tx/{txid}"), body.to_string())]),
    );
    let _opened = host.open_transaction(txid);
    painted(&host)
}

fn assert_inputs_outputs_heading_once(painted: &str, txid: &str) {
    assert!(
        painted.contains(&format!("tx txid {txid}")),
        "the parsed transaction was hidden: {painted}"
    );
    assert_eq!(
        painted
            .lines()
            .filter(|line| *line == INPUTS_OUTPUTS_HEADING)
            .count(),
        1,
        "the parsed transaction did not paint {INPUTS_OUTPUTS_HEADING} once: {painted}"
    );
    assert_eq!(
        painted.matches(INPUTS_OUTPUTS_HEADING).count(),
        1,
        "the heading was not exactly once: {painted}"
    );
    assert_heading_stays_plaintext(painted);
}

fn assert_heading_absent(painted: &str) {
    assert!(
        !painted.contains(INPUTS_OUTPUTS_HEADING),
        "the heading leaked: {painted}"
    );
    assert_heading_stays_plaintext(painted);
}

fn assert_heading_stays_plaintext(painted: &str) {
    assert!(!painted.contains("<h2"), "an h2 tag leaked: {painted}");
    assert!(!painted.contains("<td"), "a td tag leaked: {painted}");
    for banned in ["Allowed?", "Previous output type", "Transaction hex"] {
        assert!(
            !painted.contains(banned),
            "{banned} leaked into the painted shell: {painted}"
        );
    }
}

/// Angular subtitle on the parsed transaction from `GET /tx/:txid`.
///
/// The plaintext line is `Inputs & Outputs`, once, including empty `vin`
/// and empty `vout`. An empty body, invalid JSON, and `[]` do not paint it.
/// The blocks list does not. There is no HTML.
#[test]
fn transaction_screen_paints_inputs_outputs_heading() {
    let vin = r#"{"txid":"prev","vout":0,"prevout":null,"scriptsig":"","scriptsig_asm":"","is_coinbase":false,"sequence":1}"#;
    let vout = r#"{"scriptpubkey":"","scriptpubkey_asm":"","scriptpubkey_type":"","value":9}"#;
    let one = paint_tx_body("inoutone", &tx_with_parties("inoutone", vin, vout));
    assert_inputs_outputs_heading_once(&one, "inoutone");

    let empty = paint_tx_body("inoutempty", &tx_with_parties("inoutempty", "", ""));
    assert_inputs_outputs_heading_once(&empty, "inoutempty");
    assert!(
        !empty.lines().any(|line| {
            line.starts_with("input outpoint ") || line.starts_with("output address ")
        }),
        "empty vin and vout painted a party: {empty}"
    );

    for (txid, body) in [
        ("inoutblank", ""),
        ("inoutbad", "not-json"),
        ("inoutarr", "[]"),
    ] {
        let painted = paint_tx_body(txid, body);
        assert!(
            painted.contains(txid),
            "the unparsed body hid the transaction screen: {painted}"
        );
        assert!(
            !painted.contains(&format!("tx txid {txid}")),
            "an unparsed body painted a transaction row: {painted}"
        );
        assert_heading_absent(&painted);
    }

    let (mut host, _) = host_with(
        Network::Mainnet,
        BTreeMap::from([("/blocks".to_string(), format!("[{}]", block_json("blkio1")))]),
    );
    host.activate(Screen::Blocks)
        .unwrap_or_else(|err| panic!("blocks list did not open: {err}"));
    let blocks = painted(&host);
    assert!(
        blocks.contains("blkio1"),
        "the blocks list hid the block: {blocks}"
    );
    assert_heading_absent(&blocks);
}

fn show_more_input(index: usize) -> String {
    format!(
        r#"{{"txid":"prev","vout":{index},"prevout":null,"scriptsig":"","scriptsig_asm":"","is_coinbase":false,"sequence":1}}"#
    )
}

fn show_more_output(index: usize) -> String {
    format!(r#"{{"scriptpubkey":"","scriptpubkey_asm":"","scriptpubkey_type":"","value":{index}}}"#)
}

fn show_more_tx(txid: &str, inputs: usize, outputs: usize) -> String {
    let vin = (0..inputs)
        .map(show_more_input)
        .collect::<Vec<_>>()
        .join(",");
    let vout = (0..outputs)
        .map(show_more_output)
        .collect::<Vec<_>>()
        .join(",");
    format!(
        r#"{{"txid":"{txid}","version":1,"locktime":0,"vin":[{vin}],"vout":[{vout}],"size":10,"weight":40,"sigops":0,"fee":1,"status":{{"confirmed":false}}}}"#
    )
}

fn paint_show_more(txid: &str, inputs: usize, outputs: usize) -> String {
    let (mut host, rec) = host_with(
        Network::Mainnet,
        BTreeMap::from([(format!("/tx/{txid}"), show_more_tx(txid, inputs, outputs))]),
    );
    host.open_transaction(txid).expect("tx");
    let paths = rec.gets();
    assert!(
        paths.iter().any(|path| path == &format!("/tx/{txid}")),
        "{paths:?}"
    );
    assert!(
        paths
            .iter()
            .all(|path| !path.contains('?') && !path.contains('#')),
        "{paths:?}"
    );
    painted(&host)
}

fn assert_no_show_more_words(painted: &str) {
    for word in ["Show all", "Show more", "remaining"] {
        assert!(
            !painted.lines().any(|line| line.contains(word)),
            "{word} was painted: {painted}"
        );
    }
}

fn assert_input_outpoints(painted: &str, shown: usize, total: usize) {
    for index in 0..shown {
        let line = format!("input outpoint prev:{index}");
        assert!(
            painted.lines().any(|row| row == line),
            "input {index} was hidden: {painted}"
        );
    }
    for index in shown..total {
        let line = format!("input outpoint prev:{index}");
        assert!(
            !painted.lines().any(|row| row == line),
            "input {index} was painted past the cutoff: {painted}"
        );
    }
}

fn assert_output_values(painted: &str, shown: usize, total: usize) {
    for index in 0..shown {
        let line = format!("output address  value {index}");
        assert!(
            painted.lines().any(|row| row == line),
            "output {index} was hidden: {painted}"
        );
    }
    for index in shown..total {
        let line = format!("output address  value {index}");
        assert!(
            !painted.lines().any(|row| row == line),
            "output {index} was painted past the cutoff: {painted}"
        );
    }
}

fn assert_show_more_line(painted: &str, line: &str) {
    assert_eq!(
        painted.lines().filter(|row| *row == line).count(),
        1,
        "the show more line is absent: {painted}"
    );
    assert!(!painted.contains("<button"), "{painted}");
    assert!(!painted.contains("<span"), "{painted}");
}

/// Transactions-list button from `GET /tx/:txid` with no query string.
///
/// The button is painted when `tx.vin.length > getVinLimit(tx)`.
/// `inputRowLimit` is 12. When `vin.length - limit <= 5`, `getVinLimit`
/// returns `vin.length` and the button stays off. The label is `Show all`
/// when `getVinLimit(tx, true) >= tx.vin.length`, and `Show more` otherwise,
/// then `(N remaining)` where N is `tx.vin.length - getVinLimit(tx)`.
/// `showMoreIncrement` is 1000. Outputs use `getVoutLimit` the same way.
#[test]
fn transaction_screen_paints_show_more_inputs() {
    let eighteen = paint_show_more("showin18", 18, 1);
    assert_show_more_line(&eighteen, "Show all (6 remaining)");
    assert!(!eighteen.contains("Show more"), "{eighteen}");
    assert_input_outpoints(&eighteen, 12, 18);
    assert_output_values(&eighteen, 1, 1);
    let rows: Vec<&str> = eighteen.lines().collect();
    let last_input = rows
        .iter()
        .position(|line| *line == "input outpoint prev:11")
        .expect("input 11");
    let button = rows
        .iter()
        .position(|line| *line == "Show all (6 remaining)")
        .expect("button");
    let output = rows
        .iter()
        .position(|line| *line == "output address  value 0")
        .expect("output");
    assert!(last_input < button && button < output, "{eighteen}");

    let dozen = paint_show_more("showin12", 12, 1);
    assert_no_show_more_words(&dozen);
    assert_input_outpoints(&dozen, 12, 12);
    assert_output_values(&dozen, 1, 1);

    let seventeen = paint_show_more("showin17", 17, 1);
    assert_no_show_more_words(&seventeen);
    assert_input_outpoints(&seventeen, 17, 17);
    assert_output_values(&seventeen, 1, 1);

    let within_tail = paint_show_more("showin13", 13, 1);
    assert_no_show_more_words(&within_tail);
    assert_input_outpoints(&within_tail, 13, 13);

    let show_more = paint_show_more("showin1006", 1006, 1);
    assert_show_more_line(&show_more, "Show more (994 remaining)");
    assert!(!show_more.contains("Show all"), "{show_more}");
    assert_input_outpoints(&show_more, 12, 1006);
    assert_output_values(&show_more, 1, 1);

    let show_all_next = paint_show_more("showin1005", 1005, 1);
    assert_show_more_line(&show_all_next, "Show all (993 remaining)");
    assert_input_outpoints(&show_all_next, 12, 1005);

    let outputs = paint_show_more("showout18", 1, 18);
    assert_show_more_line(&outputs, "Show all (6 remaining)");
    assert_input_outpoints(&outputs, 1, 1);
    assert_output_values(&outputs, 12, 18);
    let out_rows: Vec<&str> = outputs.lines().collect();
    let last_output = out_rows
        .iter()
        .position(|line| *line == "output address  value 11")
        .expect("output 11");
    let out_button = out_rows
        .iter()
        .position(|line| *line == "Show all (6 remaining)")
        .expect("output button");
    assert!(last_output < out_button, "{outputs}");

    let out_seventeen = paint_show_more("showout17", 1, 17);
    assert_no_show_more_words(&out_seventeen);
    assert_output_values(&out_seventeen, 17, 17);
    assert_input_outpoints(&out_seventeen, 1, 1);

    let dozen_outputs = paint_show_more("showout12", 1, 12);
    assert_no_show_more_words(&dozen_outputs);
    assert_output_values(&dozen_outputs, 12, 12);

    let out_more = paint_show_more("showout1006", 1, 1006);
    assert_show_more_line(&out_more, "Show more (994 remaining)");
    assert!(!out_more.contains("Show all"), "{out_more}");
    assert_output_values(&out_more, 12, 1006);
    assert_input_outpoints(&out_more, 1, 1);

    let inputs_only = paint_show_more("showin18out12", 18, 12);
    assert_show_more_line(&inputs_only, "Show all (6 remaining)");
    assert_input_outpoints(&inputs_only, 12, 18);
    assert_output_values(&inputs_only, 12, 12);

    let both = paint_show_more("showboth", 18, 1006);
    assert_show_more_line(&both, "Show all (6 remaining)");
    assert_show_more_line(&both, "Show more (994 remaining)");
    assert_input_outpoints(&both, 12, 18);
    assert_output_values(&both, 12, 1006);
    let both_rows: Vec<&str> = both.lines().collect();
    let input_button = both_rows
        .iter()
        .position(|line| *line == "Show all (6 remaining)")
        .expect("input button");
    let output_button = both_rows
        .iter()
        .position(|line| *line == "Show more (994 remaining)")
        .expect("output button");
    assert!(input_button < output_button, "{both}");
}

const PREVIOUS_OUTPUT_TYPE: &str = "Previous output type";

fn pot_prevout(kind: &str) -> String {
    format!(
        r#"{{"scriptpubkey":"POTPUBHEX","scriptpubkey_asm":"POTASM","scriptpubkey_type":"{kind}","value":5}}"#
    )
}

fn pot_input(prevout: &str) -> String {
    format!(
        r#"{{"txid":"prev","vout":0,"prevout":{prevout},"scriptsig":"","scriptsig_asm":"","is_coinbase":false,"sequence":1}}"#
    )
}

fn pot_output(kind: &str) -> String {
    format!(
        r#"{{"scriptpubkey":"POTOUTHEX","scriptpubkey_asm":"POTOUTASM","scriptpubkey_type":"{kind}","value":1}}"#
    )
}

fn previous_output_type_lines(painted: &str) -> Vec<&str> {
    painted
        .lines()
        .filter(|line| line.starts_with(PREVIOUS_OUTPUT_TYPE))
        .collect()
}

fn assert_previous_output_lines(painted: &str, expected: &[&str]) {
    assert_eq!(previous_output_type_lines(painted), expected, "{painted}");
    assert_eq!(
        painted.matches(PREVIOUS_OUTPUT_TYPE).count(),
        expected.len(),
        "the label appeared outside its own line: {painted}"
    );
}

fn assert_previous_output_omitted(painted: &str) {
    assert_previous_output_lines(painted, &[]);
    assert_previous_output_stays_plaintext(painted);
}

fn assert_previous_output_stays_plaintext(painted: &str) {
    assert!(
        !painted.contains("&amp;"),
        "the type was HTML-escaped: {painted}"
    );
    assert!(
        !painted.contains("&lt;"),
        "the type was HTML-escaped: {painted}"
    );
    assert!(
        !painted.contains("<span"),
        "an HTML span was painted for this label: {painted}"
    );
    assert!(
        !painted.contains("previous-output-type"),
        "an HTML class was painted for this label: {painted}"
    );
}

fn assert_output_type_stays_lowercase(painted: &str, kind: &str) {
    assert!(
        painted.contains(&format!("output script type {kind}")),
        "the lowercase output type was dropped: {painted}"
    );
    let upper = kind.to_uppercase();
    assert_ne!(upper, kind, "{kind}");
    // The script-type line stays as the indexer spelled it. The separate
    // output Type line is the one uppercase form.
    let type_line = format!("Type {upper}");
    assert!(
        painted
            .lines()
            .all(|line| line == type_line || !line.contains(&upper)),
        "an output type was painted in uppercase: {painted}"
    );
}

fn assert_pot_sentinels_off(painted: &str) {
    for needle in ["POTPUBHEX", "POTASM", "POTOUTHEX", "POTOUTASM"] {
        assert!(!painted.contains(needle), "{needle} leaked: {painted}");
    }
}

/// `prevout.scriptpubkey_type` on the transaction screen from `GET /tx/:txid`.
///
/// A nonempty type paints one plaintext line: `Previous output type` and the
/// uppercased type. Two prevouts paint that line twice, in input order. An
/// empty type, JSON null `prevout`, a missing `prevout` key, and a missing or
/// null `scriptpubkey_type` omit the label. `&` and `<` stay characters.
/// An output type stays lowercase. This is not that output row.
#[test]
fn transaction_screen_paints_previous_output_type() {
    let output = pot_output("p2pkh");
    let one = paint_tx_body(
        "potone",
        &tx_with_parties("potone", &pot_input(&pot_prevout("v0_p2wpkh")), &output),
    );
    assert!(
        one.contains("tx txid potone"),
        "the parsed transaction was hidden: {one}"
    );
    assert!(
        one.contains(INPUTS_OUTPUTS_HEADING),
        "Inputs & Outputs was dropped: {one}"
    );
    assert_previous_output_lines(&one, &["Previous output type V0_P2WPKH"]);
    assert!(
        one.contains("input script type v0_p2wpkh"),
        "the lowercase input script type line was dropped: {one}"
    );
    assert_output_type_stays_lowercase(&one, "p2pkh");
    assert_previous_output_stays_plaintext(&one);
    assert_pot_sentinels_off(&one);

    let two_vin = format!(
        "{},{}",
        pot_input(&pot_prevout("v0_p2wpkh")),
        pot_input(&pot_prevout("p2sh"))
    );
    let two = paint_tx_body("pottwo", &tx_with_parties("pottwo", &two_vin, &output));
    assert_previous_output_lines(
        &two,
        &[
            "Previous output type V0_P2WPKH",
            "Previous output type P2SH",
        ],
    );
    assert_output_type_stays_lowercase(&two, "p2pkh");
    assert_previous_output_stays_plaintext(&two);
    assert_pot_sentinels_off(&two);

    let empty = paint_tx_body(
        "potempty",
        &tx_with_parties("potempty", &pot_input(&pot_prevout("")), &output),
    );
    assert!(
        empty.contains("tx txid potempty"),
        "an empty type hid the transaction: {empty}"
    );
    assert_previous_output_omitted(&empty);
    assert_output_type_stays_lowercase(&empty, "p2pkh");
    assert_pot_sentinels_off(&empty);

    let null_prevout = paint_tx_body(
        "potnullprev",
        &tx_with_parties(
            "potnullprev",
            r#"{"txid":"prev","vout":0,"prevout":null,"scriptsig":"","scriptsig_asm":"","is_coinbase":false,"sequence":1}"#,
            &output,
        ),
    );
    assert!(
        null_prevout.contains("tx txid potnullprev"),
        "JSON null prevout hid the transaction: {null_prevout}"
    );
    assert_previous_output_omitted(&null_prevout);
    assert_output_type_stays_lowercase(&null_prevout, "p2pkh");

    let missing_prevout = paint_tx_body(
        "potmissprev",
        &tx_with_parties(
            "potmissprev",
            r#"{"txid":"prev","vout":0,"scriptsig":"","scriptsig_asm":"","is_coinbase":false,"sequence":1}"#,
            &output,
        ),
    );
    assert!(
        missing_prevout.contains("tx txid potmissprev"),
        "a missing prevout key hid the transaction: {missing_prevout}"
    );
    assert_previous_output_omitted(&missing_prevout);
    assert_output_type_stays_lowercase(&missing_prevout, "p2pkh");

    let null_type = paint_tx_body(
        "potnulltype",
        &tx_with_parties(
            "potnulltype",
            &pot_input(
                r#"{"scriptpubkey":"POTPUBHEX","scriptpubkey_asm":"POTASM","scriptpubkey_type":null,"value":5}"#,
            ),
            &output,
        ),
    );
    assert!(
        null_type.contains("potnulltype"),
        "JSON null scriptpubkey_type hid the transaction screen: {null_type}"
    );
    assert_previous_output_omitted(&null_type);

    let missing_type = paint_tx_body(
        "potmisstype",
        &tx_with_parties(
            "potmisstype",
            &pot_input(r#"{"scriptpubkey":"POTPUBHEX","scriptpubkey_asm":"POTASM","value":5}"#),
            &output,
        ),
    );
    assert!(
        missing_type.contains("potmisstype"),
        "a missing scriptpubkey_type hid the transaction screen: {missing_type}"
    );
    assert_previous_output_omitted(&missing_type);

    let marked = paint_tx_body(
        "potmark",
        &tx_with_parties("potmark", &pot_input(&pot_prevout("a&b<c")), &output),
    );
    assert!(
        marked
            .lines()
            .any(|line| line == "Previous output type A&B<C"),
        "markup characters were not painted as plaintext: {marked}"
    );
    assert_previous_output_lines(&marked, &["Previous output type A&B<C"]);
    assert!(
        marked.contains("input script type a&b<c"),
        "the lowercase input script type line was dropped: {marked}"
    );
    assert_output_type_stays_lowercase(&marked, "p2pkh");
    assert_previous_output_stays_plaintext(&marked);
    assert_pot_sentinels_off(&marked);
}

fn otype_output(kind: &str) -> String {
    format!(
        r#"{{"scriptpubkey":"OTYPUBHEX","scriptpubkey_asm":"OTYASM","scriptpubkey_type":"{kind}","value":1}}"#
    )
}

fn otype_prevout(kind: &str) -> String {
    format!(
        r#"{{"scriptpubkey":"OTYPREVPUB","scriptpubkey_asm":"OTYPREVASM","scriptpubkey_type":"{kind}","value":5}}"#
    )
}

fn otype_input(prevout: &str) -> String {
    format!(
        r#"{{"txid":"prev","vout":0,"prevout":{prevout},"scriptsig":"OTYSIG","scriptsig_asm":"OTYSIGASM","witness":["OTYWIT"],"is_coinbase":false,"sequence":1}}"#
    )
}

fn otype_input_null() -> &'static str {
    r#"{"txid":"prev","vout":0,"prevout":null,"scriptsig":"OTYSIG","scriptsig_asm":"OTYSIGASM","witness":["OTYWIT"],"is_coinbase":false,"sequence":1}"#
}

fn output_type_lines(painted: &str) -> Vec<&str> {
    painted
        .lines()
        .filter(|line| line.starts_with("Type "))
        .collect()
}

fn assert_output_type_lines(painted: &str, expected: &[&str]) {
    assert_eq!(output_type_lines(painted), expected, "{painted}");
    assert_eq!(
        painted.matches("Type ").count(),
        expected.len(),
        "Type appeared outside its own line: {painted}"
    );
}

fn assert_output_type_plaintext(painted: &str) {
    assert!(
        !painted.contains("&amp;"),
        "the type was HTML-escaped: {painted}"
    );
    assert!(
        !painted.contains("&lt;"),
        "the type was HTML-escaped: {painted}"
    );
    assert!(
        !painted.contains("<span"),
        "an HTML span was painted: {painted}"
    );
    assert!(
        !painted.contains("output-type"),
        "an HTML class was painted: {painted}"
    );
    assert!(
        !painted.contains("Type:"),
        "the label used a colon: {painted}"
    );
}

fn assert_otype_sentinels_off(painted: &str) {
    for needle in [
        "OTYPUBHEX",
        "OTYASM",
        "OTYSIG",
        "OTYSIGASM",
        "OTYPREVPUB",
        "OTYPREVASM",
        "OTYWIT",
    ] {
        assert!(!painted.contains(needle), "{needle} leaked: {painted}");
    }
}

/// `vout.scriptpubkey_type` on the transaction screen from `GET /tx/:txid`.
///
/// A nonempty type paints one plaintext line: `Type` and the uppercased type.
/// Two outputs paint that line twice. An empty string omits it for that output.
/// A missing key, JSON null, an empty body, `not-json`, and `[]` omit it.
/// `&` and `<` stay characters. This is not Previous output type.
#[test]
fn transaction_screen_paints_output_type() {
    let marked = paint_tx_body(
        "otypemark",
        &tx_with_parties("otypemark", otype_input_null(), &otype_output("a&b<c")),
    );
    assert!(
        marked.contains("tx txid otypemark"),
        "the parsed transaction was hidden: {marked}"
    );
    assert!(
        marked.contains(INPUTS_OUTPUTS_HEADING),
        "Inputs & Outputs was dropped: {marked}"
    );
    assert_output_type_lines(&marked, &["Type A&B<C"]);
    assert!(
        marked.lines().any(|line| line == "Type A&B<C"),
        "markup characters were not painted as plaintext: {marked}"
    );
    assert!(
        marked.contains("output script type a&b<c"),
        "the lowercase output script type was dropped: {marked}"
    );
    let address_at = marked.find("output address").expect("output row");
    let type_at = marked.find("\nType A&B<C").expect("type line");
    assert!(
        type_at > address_at,
        "Type was not in the output row: {marked}"
    );
    assert_output_type_plaintext(&marked);
    assert_otype_sentinels_off(&marked);
    assert!(
        !marked.contains("Previous output type"),
        "a null prevout painted Previous output type: {marked}"
    );

    let two_vout = format!("{},{}", otype_output("v0_p2wpkh"), otype_output("p2sh"));
    let two = paint_tx_body(
        "otypetwo",
        &tx_with_parties("otypetwo", otype_input_null(), &two_vout),
    );
    assert_output_type_lines(&two, &["Type V0_P2WPKH", "Type P2SH"]);
    assert!(
        two.contains("output script type v0_p2wpkh"),
        "the first lowercase output script type was dropped: {two}"
    );
    assert!(
        two.contains("output script type p2sh"),
        "the second lowercase output script type was dropped: {two}"
    );
    assert_output_type_plaintext(&two);
    assert_otype_sentinels_off(&two);

    let mixed_vout = format!("{},{}", otype_output(""), otype_output("p2pkh"));
    let mixed = paint_tx_body(
        "otypemix",
        &tx_with_parties("otypemix", otype_input_null(), &mixed_vout),
    );
    assert!(
        mixed.contains("tx txid otypemix"),
        "an empty type hid the transaction: {mixed}"
    );
    assert!(
        mixed.contains(INPUTS_OUTPUTS_HEADING),
        "Inputs & Outputs was dropped: {mixed}"
    );
    assert_output_type_lines(&mixed, &["Type P2PKH"]);
    assert!(
        mixed.contains("output script type p2pkh"),
        "the typed output lost its lowercase script type: {mixed}"
    );
    assert_eq!(
        mixed
            .lines()
            .filter(|line| line.contains("output script type"))
            .count(),
        1,
        "the empty type painted a script type line: {mixed}"
    );
    assert_output_type_plaintext(&mixed);
    assert_otype_sentinels_off(&mixed);

    let only_empty = paint_tx_body(
        "otypeempty",
        &tx_with_parties("otypeempty", otype_input_null(), &otype_output("")),
    );
    assert!(
        only_empty.contains("tx txid otypeempty"),
        "an empty type hid the transaction: {only_empty}"
    );
    assert!(
        only_empty.contains(INPUTS_OUTPUTS_HEADING),
        "Inputs & Outputs was dropped: {only_empty}"
    );
    assert_output_type_lines(&only_empty, &[]);
    assert!(
        !only_empty.contains("output script type"),
        "an empty type painted a script type line: {only_empty}"
    );
    assert_output_type_plaintext(&only_empty);

    let op_return = paint_tx_body(
        "otypeop",
        &tx_with_parties("otypeop", otype_input_null(), &otype_output("op_return")),
    );
    assert!(
        op_return.contains("tx txid otypeop"),
        "op_return hid the transaction: {op_return}"
    );
    assert_output_type_lines(&op_return, &["Type OP_RETURN"]);
    assert!(
        op_return.contains("output script type op_return"),
        "the lowercase op_return script type was dropped: {op_return}"
    );
    assert_eq!(
        op_return
            .lines()
            .filter(|line| *line == "OP_RETURN")
            .count(),
        1,
        "the op_return address cell did not paint OP_RETURN once: {op_return}"
    );
    assert_output_type_plaintext(&op_return);
    assert_otype_sentinels_off(&op_return);

    let beside = paint_tx_body(
        "otypeboth",
        &tx_with_parties(
            "otypeboth",
            &otype_input(&otype_prevout("v0_p2wpkh")),
            &otype_output("p2pkh"),
        ),
    );
    assert!(
        beside.contains("tx txid otypeboth"),
        "the parsed transaction was hidden: {beside}"
    );
    assert_output_type_lines(&beside, &["Type P2PKH"]);
    assert!(
        beside
            .lines()
            .any(|line| line == "Previous output type V0_P2WPKH"),
        "Previous output type was dropped or reworded: {beside}"
    );
    assert!(
        beside.contains("output script type p2pkh"),
        "the lowercase output script type was dropped: {beside}"
    );
    assert!(
        beside.contains("input script type v0_p2wpkh"),
        "the lowercase input script type was dropped: {beside}"
    );
    assert_output_type_plaintext(&beside);
    assert_otype_sentinels_off(&beside);

    let missing = paint_tx_body(
        "otypemiss",
        &tx_with_parties(
            "otypemiss",
            otype_input_null(),
            r#"{"scriptpubkey":"OTYPUBHEX","scriptpubkey_asm":"OTYASM","value":1}"#,
        ),
    );
    assert!(
        missing.contains("otypemiss"),
        "a missing scriptpubkey_type hid the screen: {missing}"
    );
    assert_output_type_lines(&missing, &[]);
    assert_output_type_plaintext(&missing);

    let null_type = paint_tx_body(
        "otypenull",
        &tx_with_parties(
            "otypenull",
            otype_input_null(),
            r#"{"scriptpubkey":"OTYPUBHEX","scriptpubkey_asm":"OTYASM","scriptpubkey_type":null,"value":1}"#,
        ),
    );
    assert!(
        null_type.contains("otypenull"),
        "JSON null scriptpubkey_type hid the screen: {null_type}"
    );
    assert_output_type_lines(&null_type, &[]);
    assert_output_type_plaintext(&null_type);

    for (txid, body) in [
        ("otypeblank", ""),
        ("otypebad", "not-json"),
        ("otypearr", "[]"),
    ] {
        let painted = paint_tx_body(txid, body);
        assert!(
            painted.contains(txid),
            "the unparsed body hid the screen: {painted}"
        );
        assert_output_type_lines(&painted, &[]);
        assert_output_type_plaintext(&painted);
        assert!(
            !painted.contains(&format!("tx txid {txid}")),
            "an unparsed body painted a transaction row: {painted}"
        );
    }
}

const ANC_HEX: &str = "ancpubhexdead";
const ANC_ASM: &str = "ancasmdead";
const ANC_SIG: &str = "ancsighexdead";
const ANC_SIG_ASM: &str = "ancsigasmdead";
const ANC_WIT: &str = "ancwitdead";
const ANC_INNER: &str = "ancinnerdead";
const ANC_LEAF: &str = "ancleafdead";

fn anchor_vin() -> String {
    format!(
        r#"{{"txid":"prev","vout":0,"prevout":{{"scriptpubkey":"{ANC_HEX}","scriptpubkey_asm":"{ANC_ASM}","scriptpubkey_type":"v0_p2wpkh","value":5}},"scriptsig":"{ANC_SIG}","scriptsig_asm":"{ANC_SIG_ASM}","witness":["{ANC_WIT}"],"is_coinbase":false,"sequence":1,"inner_witnessscript_asm":"{ANC_INNER}","leaf_version":"{ANC_LEAF}"}}"#
    )
}

fn anchor_vout(type_field: &str) -> String {
    format!(
        r#"{{"scriptpubkey":"{ANC_HEX}","scriptpubkey_asm":"{ANC_ASM}",{type_field}"value":1}}"#
    )
}

fn anchor_tx(txid: &str, vout: &str) -> String {
    format!(
        r#"{{"txid":"{txid}","version":1,"locktime":0,"vin":[{}],"vout":[{vout}],"size":10,"weight":400,"sigops":0,"fee":50,"status":{{"confirmed":true,"block_height":4}}}}"#,
        anchor_vin()
    )
}

fn paint_anchor_tx(txid: &str, vout: &str) -> (Result<(), ClientError>, String) {
    assert!(
        !txid.is_empty() && !txid.contains('/') && !txid.contains('?') && !txid.contains('#'),
        "{txid}"
    );
    assert!(
        !txid.to_ascii_lowercase().contains("anchor"),
        "the txid must not contain the type word: {txid}"
    );
    let body = anchor_tx(txid, vout);
    for needle in [
        ANC_HEX,
        ANC_ASM,
        ANC_SIG,
        ANC_SIG_ASM,
        ANC_WIT,
        ANC_INNER,
        ANC_LEAF,
    ] {
        assert!(body.contains(needle), "fixture omitted {needle}: {body}");
    }
    let (mut host, rec) = host_with(
        Network::Mainnet,
        BTreeMap::from([(format!("/tx/{txid}"), body)]),
    );
    let opened = host.open_transaction(txid);
    let paths = rec.gets();
    assert!(
        paths.iter().any(|path| path == &format!("/tx/{txid}")),
        "GET /tx/{txid} was not requested: {paths:?}"
    );
    assert!(
        paths.iter().all(|path| !path.contains('?')
            && !path.contains('#')
            && !path.starts_with("/address/")),
        "a transaction open invented a route, a query, or a fragment: {paths:?}"
    );
    (opened, painted(&host))
}

fn assert_anchor_bytes_off(painted: &str) {
    for needle in [
        ANC_HEX,
        ANC_ASM,
        ANC_SIG,
        ANC_SIG_ASM,
        ANC_WIT,
        ANC_INNER,
        ANC_LEAF,
        "scriptpubkey",
        "script asm",
        "leaf version",
        "Leaf version",
        "<span",
        "output-type",
        "scriptpubkey-type",
    ] {
        assert!(
            !painted.contains(needle),
            "{needle} leaked into the screen: {painted}"
        );
    }
}

fn assert_one_fee_and_confirmed(painted: &str) {
    assert_eq!(
        painted
            .lines()
            .filter(|line| *line == "Fee 50 sats")
            .count(),
        1,
        "the details Fee row was dropped or repeated: {painted}"
    );
    assert_eq!(
        painted
            .lines()
            .filter(|line| line.starts_with("Fee ") && !line.starts_with("Fee rate "))
            .count(),
        1,
        "a second Fee row was painted: {painted}"
    );
    assert_eq!(
        painted
            .lines()
            .filter(|line| *line == "Fee rate 0.50 sat/vB")
            .count(),
        1,
        "Fee rate was dropped or repeated: {painted}"
    );
    assert_eq!(
        painted.lines().filter(|line| *line == "Confirmed").count(),
        1,
        "Confirmed was dropped or the letter case changed: {painted}"
    );
    assert!(
        !painted.lines().any(|line| {
            line == "confirmed"
                || line == "CONFIRMED"
                || line == "Unconfirmed"
                || line == "unconfirmed"
        }),
        "confirmation letter case changed: {painted}"
    );
}

/// Output script type `anchor` from `GET /tx/:txid`.
///
/// A nonempty `vout.scriptpubkey_type` of `anchor` paints the exact line
/// `Type ANCHOR`. An empty string, a missing key, and JSON null omit that
/// line. `output script type anchor` stays the indexer spelling. Script hex,
/// script asm, scriptsig, witness, the inner tapscript, and leaf version stay
/// off the screen. This is not an address span. `GET /address/:script` is not
/// called. The path has no query and no fragment.
#[test]
fn transaction_screen_paints_anchor_output_type() {
    let (opened, painted) =
        paint_anchor_tx("atypeone", &anchor_vout(r#""scriptpubkey_type":"anchor","#));
    opened.expect("anchor output");
    assert!(
        painted.contains("tx txid atypeone"),
        "the parsed transaction was hidden: {painted}"
    );
    assert_eq!(
        painted
            .lines()
            .filter(|line| *line == INPUTS_OUTPUTS_HEADING)
            .count(),
        1,
        "Inputs & Outputs was dropped: {painted}"
    );
    assert_eq!(
        painted
            .lines()
            .filter(|line| *line == "Type ANCHOR")
            .count(),
        1,
        "scriptpubkey_type anchor did not paint the exact line Type ANCHOR: {painted}"
    );
    assert_eq!(
        painted.matches("Type ANCHOR").count(),
        1,
        "Type ANCHOR was not once: {painted}"
    );
    assert_eq!(
        painted.matches("ANCHOR").count(),
        1,
        "ANCHOR leaked outside Type ANCHOR: {painted}"
    );
    assert_eq!(
        painted
            .lines()
            .filter(|line| *line == "output script type anchor")
            .count(),
        1,
        "the indexer spelling was dropped: {painted}"
    );
    assert_eq!(
        painted.matches("anchor").count(),
        1,
        "lowercase anchor leaked outside the indexer spelling: {painted}"
    );
    assert!(
        !painted
            .lines()
            .any(|line| line == "ANCHOR" || line == "anchor"),
        "an address classification line was painted: {painted}"
    );
    let rows: Vec<&str> = painted.lines().collect();
    let previous = rows
        .iter()
        .position(|line| *line == "Previous output script");
    let address = rows
        .iter()
        .position(|line| *line == "output address  value 1");
    let raw = rows
        .iter()
        .position(|line| *line == "output script type anchor");
    let upper = rows.iter().position(|line| *line == "Type ANCHOR");
    assert!(
        previous.is_some() && address.is_some() && raw.is_some() && upper.is_some(),
        "Previous output script or the anchor type lines were dropped: {painted}"
    );
    assert!(
        previous.unwrap() < address.unwrap()
            && address.unwrap() < raw.unwrap()
            && raw.unwrap() < upper.unwrap(),
        "Type ANCHOR was not after the indexer spelling on the output row: {painted}"
    );
    assert!(
        !painted.contains("P2TR tapscript"),
        "an anchor output painted P2TR tapscript: {painted}"
    );
    assert!(
        !painted.contains("P2SH redeem script"),
        "an anchor output painted P2SH redeem script: {painted}"
    );
    assert_one_fee_and_confirmed(&painted);
    assert_anchor_bytes_off(&painted);

    let (empty_opened, empty) =
        paint_anchor_tx("atypeempty", &anchor_vout(r#""scriptpubkey_type":"","#));
    empty_opened.expect("empty output type");
    assert!(
        empty.contains("tx txid atypeempty"),
        "an empty type hid the transaction: {empty}"
    );
    assert!(
        empty.lines().any(|line| line == "Previous output script"),
        "an empty output type dropped Previous output script: {empty}"
    );
    assert_output_type_lines(&empty, &[]);
    assert!(
        !empty.contains("output script type"),
        "an empty type painted a script type line: {empty}"
    );
    assert!(!empty.to_ascii_lowercase().contains("anchor"), "{empty}");
    assert_one_fee_and_confirmed(&empty);
    assert_anchor_bytes_off(&empty);

    for (txid, type_field) in [
        ("atypemiss", ""),
        ("atypenull", r#""scriptpubkey_type":null,"#),
    ] {
        let (_opened, omitted) = paint_anchor_tx(txid, &anchor_vout(type_field));
        assert!(
            omitted.contains(txid),
            "a missing scriptpubkey_type hid the screen: {omitted}"
        );
        assert_output_type_lines(&omitted, &[]);
        assert!(
            !omitted.contains("Type ANCHOR"),
            "a missing scriptpubkey_type painted Type ANCHOR: {omitted}"
        );
        assert!(
            !omitted.to_ascii_lowercase().contains("anchor"),
            "{omitted}"
        );
        assert_anchor_bytes_off(&omitted);
    }
}

const WSH_HEX: &str = "wshpubhexdead";
const WSH_ASM: &str = "wshasmdead";
const WSH_SIG: &str = "wshsighexdead";
const WSH_SIG_ASM: &str = "wshsigasmdead";
const WSH_WIT: &str = "wshwitdead";
const WSH_INNER: &str = "wshinnerdead";
const WSH_LEAF: &str = "wshleafdead";

fn v0_p2wsh_vin() -> String {
    format!(
        r#"{{"txid":"prev","vout":0,"prevout":{{"scriptpubkey":"{WSH_HEX}","scriptpubkey_asm":"{WSH_ASM}","scriptpubkey_type":"v0_p2wpkh","value":5}},"scriptsig":"{WSH_SIG}","scriptsig_asm":"{WSH_SIG_ASM}","witness":["{WSH_WIT}"],"is_coinbase":false,"sequence":1,"inner_witnessscript_asm":"{WSH_INNER}","leaf_version":"{WSH_LEAF}"}}"#
    )
}

fn v0_p2wsh_vout(type_field: &str) -> String {
    format!(
        r#"{{"scriptpubkey":"{WSH_HEX}","scriptpubkey_asm":"{WSH_ASM}",{type_field}"value":1}}"#
    )
}

fn v0_p2wsh_tx(txid: &str, vout: &str) -> String {
    format!(
        r#"{{"txid":"{txid}","version":1,"locktime":0,"vin":[{}],"vout":[{vout}],"size":10,"weight":400,"sigops":0,"fee":50,"status":{{"confirmed":true,"block_height":4}}}}"#,
        v0_p2wsh_vin()
    )
}

fn paint_v0_p2wsh_tx(txid: &str, vout: &str) -> (Result<(), ClientError>, String) {
    assert!(
        !txid.is_empty() && !txid.contains('/') && !txid.contains('?') && !txid.contains('#'),
        "{txid}"
    );
    assert!(
        !txid.to_ascii_lowercase().contains("v0_p2wsh")
            && !txid.to_ascii_lowercase().contains("p2wsh"),
        "the txid must not contain the type word: {txid}"
    );
    let body = v0_p2wsh_tx(txid, vout);
    for needle in [
        WSH_HEX,
        WSH_ASM,
        WSH_SIG,
        WSH_SIG_ASM,
        WSH_WIT,
        WSH_INNER,
        WSH_LEAF,
    ] {
        assert!(body.contains(needle), "fixture omitted {needle}: {body}");
    }
    let (mut host, rec) = host_with(
        Network::Mainnet,
        BTreeMap::from([(format!("/tx/{txid}"), body)]),
    );
    let opened = host.open_transaction(txid);
    let paths = rec.gets();
    assert!(
        paths.iter().any(|path| path == &format!("/tx/{txid}")),
        "GET /tx/{txid} was not requested: {paths:?}"
    );
    assert!(
        paths.iter().all(|path| !path.contains('?')
            && !path.contains('#')
            && !path.starts_with("/address/")),
        "a transaction open invented a route, a query, or a fragment: {paths:?}"
    );
    (opened, painted(&host))
}

fn assert_v0_p2wsh_bytes_off(painted: &str) {
    for needle in [
        WSH_HEX,
        WSH_ASM,
        WSH_SIG,
        WSH_SIG_ASM,
        WSH_WIT,
        WSH_INNER,
        WSH_LEAF,
        "scriptpubkey",
        "script asm",
        "leaf version",
        "Leaf version",
        "<span",
        "output-type",
        "scriptpubkey-type",
    ] {
        assert!(
            !painted.contains(needle),
            "{needle} leaked into the screen: {painted}"
        );
    }
}

fn assert_v0_p2wsh_leaves_other_rows(painted: &str) {
    for line in [
        "Transaction fee",
        "P2PK",
        "OP_RETURN",
        "Type ANCHOR",
        "Show all",
        "Show more",
        "P2TR tapscript",
        "P2SH redeem script",
    ] {
        assert!(
            !painted.contains(line),
            "{line} changed on a v0_p2wsh screen: {painted}"
        );
    }
}

/// Output script type `v0_p2wsh` from `GET /tx/:txid`.
///
/// A nonempty `vout.scriptpubkey_type` of `v0_p2wsh` paints the exact line
/// `Type V0_P2WSH`. An empty string, a missing key, and JSON null omit that
/// line. `output script type v0_p2wsh` stays the indexer spelling. Script hex,
/// script asm, scriptsig, witness, the inner tapscript, and leaf version stay
/// off the screen. This is not an address span. `GET /address/:script` is not
/// called. The path has no query and no fragment. The confirmation line on
/// this fixture stays `Confirmed`. The count line `confirmations 1` is not
/// this fixture.
#[test]
fn transaction_screen_paints_v0_p2wsh_output_type() {
    let (opened, painted) = paint_v0_p2wsh_tx(
        "wshone",
        &v0_p2wsh_vout(r#""scriptpubkey_type":"v0_p2wsh","#),
    );
    opened.expect("v0_p2wsh output");
    assert!(
        painted.contains("tx txid wshone"),
        "the parsed transaction was hidden: {painted}"
    );
    assert_eq!(
        painted
            .lines()
            .filter(|line| *line == INPUTS_OUTPUTS_HEADING)
            .count(),
        1,
        "Inputs & Outputs was dropped: {painted}"
    );
    assert_eq!(
        painted
            .lines()
            .filter(|line| *line == "Type V0_P2WSH")
            .count(),
        1,
        "scriptpubkey_type v0_p2wsh did not paint the exact line Type V0_P2WSH: {painted}"
    );
    assert_output_type_lines(&painted, &["Type V0_P2WSH"]);
    assert_eq!(
        painted.matches("Type V0_P2WSH").count(),
        1,
        "Type V0_P2WSH was not once: {painted}"
    );
    assert_eq!(
        painted.matches("V0_P2WSH").count(),
        1,
        "V0_P2WSH leaked outside Type V0_P2WSH: {painted}"
    );
    assert_eq!(
        painted
            .lines()
            .filter(|line| *line == "output script type v0_p2wsh")
            .count(),
        1,
        "the indexer spelling was dropped: {painted}"
    );
    assert_eq!(
        painted.matches("v0_p2wsh").count(),
        1,
        "lowercase v0_p2wsh leaked outside the indexer spelling: {painted}"
    );
    assert!(
        !painted
            .lines()
            .any(|line| line == "V0_P2WSH" || line == "v0_p2wsh"),
        "an address classification line was painted: {painted}"
    );
    let rows: Vec<&str> = painted.lines().collect();
    let previous = rows
        .iter()
        .position(|line| *line == "Previous output script");
    let address = rows
        .iter()
        .position(|line| *line == "output address  value 1");
    let raw = rows
        .iter()
        .position(|line| *line == "output script type v0_p2wsh");
    let upper = rows.iter().position(|line| *line == "Type V0_P2WSH");
    assert!(
        previous.is_some() && address.is_some() && raw.is_some() && upper.is_some(),
        "Previous output script or the v0_p2wsh type lines were dropped: {painted}"
    );
    assert!(
        previous.unwrap() < address.unwrap()
            && address.unwrap() < raw.unwrap()
            && raw.unwrap() < upper.unwrap(),
        "Type V0_P2WSH was not after the indexer spelling on the output row: {painted}"
    );
    assert_one_fee_and_confirmed(&painted);
    assert_v0_p2wsh_leaves_other_rows(&painted);
    assert_v0_p2wsh_bytes_off(&painted);

    let (empty_opened, empty) =
        paint_v0_p2wsh_tx("wshempty", &v0_p2wsh_vout(r#""scriptpubkey_type":"","#));
    empty_opened.expect("empty output type");
    assert!(
        empty.contains("tx txid wshempty"),
        "an empty type hid the transaction: {empty}"
    );
    assert!(
        empty.lines().any(|line| line == "Previous output script"),
        "an empty output type dropped Previous output script: {empty}"
    );
    assert_output_type_lines(&empty, &[]);
    assert!(
        !empty.contains("output script type"),
        "an empty type painted a script type line: {empty}"
    );
    assert!(
        !empty.contains("Type V0_P2WSH"),
        "an empty type painted Type V0_P2WSH: {empty}"
    );
    assert!(!empty.to_ascii_lowercase().contains("v0_p2wsh"), "{empty}");
    assert_one_fee_and_confirmed(&empty);
    assert_v0_p2wsh_leaves_other_rows(&empty);
    assert_v0_p2wsh_bytes_off(&empty);

    for (txid, type_field) in [("wshmiss", ""), ("wshnull", r#""scriptpubkey_type":null,"#)] {
        let (_opened, omitted) = paint_v0_p2wsh_tx(txid, &v0_p2wsh_vout(type_field));
        assert!(
            omitted.contains(txid),
            "a missing scriptpubkey_type hid the screen: {omitted}"
        );
        assert_output_type_lines(&omitted, &[]);
        assert!(
            !omitted.contains("Type V0_P2WSH"),
            "a missing scriptpubkey_type painted Type V0_P2WSH: {omitted}"
        );
        assert!(
            !omitted.to_ascii_lowercase().contains("v0_p2wsh"),
            "{omitted}"
        );
        assert_v0_p2wsh_leaves_other_rows(&omitted);
        assert_v0_p2wsh_bytes_off(&omitted);
    }
}

const MSIG_HEX: &str = "msigpubhexdead";
const MSIG_ASM: &str = "msigasmdead";
const MSIG_SIG: &str = "msigsighexdead";
const MSIG_SIG_ASM: &str = "msigsigasmdead";
const MSIG_WIT: &str = "msigwitdead";
const MSIG_INNER: &str = "msiginnerdead";
const MSIG_LEAF: &str = "msigleafdead";

fn multisig_vin() -> String {
    format!(
        r#"{{"txid":"prev","vout":0,"prevout":{{"scriptpubkey":"{MSIG_HEX}","scriptpubkey_asm":"{MSIG_ASM}","scriptpubkey_type":"v0_p2wpkh","value":5}},"scriptsig":"{MSIG_SIG}","scriptsig_asm":"{MSIG_SIG_ASM}","witness":["{MSIG_WIT}"],"is_coinbase":false,"sequence":1,"inner_witnessscript_asm":"{MSIG_INNER}","leaf_version":"{MSIG_LEAF}"}}"#
    )
}

fn multisig_vout(type_field: &str) -> String {
    format!(
        r#"{{"scriptpubkey":"{MSIG_HEX}","scriptpubkey_asm":"{MSIG_ASM}",{type_field}"value":1}}"#
    )
}

fn multisig_tx(txid: &str, vout: &str) -> String {
    format!(
        r#"{{"txid":"{txid}","version":1,"locktime":0,"vin":[{}],"vout":[{vout}],"size":10,"weight":400,"sigops":0,"fee":50,"status":{{"confirmed":true,"block_height":4}}}}"#,
        multisig_vin()
    )
}

fn paint_multisig_tx(txid: &str, vout: &str) -> (Result<(), ClientError>, String) {
    assert!(
        !txid.is_empty() && !txid.contains('/') && !txid.contains('?') && !txid.contains('#'),
        "{txid}"
    );
    assert!(
        !txid.to_ascii_lowercase().contains("multisig"),
        "the txid must not contain the type word: {txid}"
    );
    let body = multisig_tx(txid, vout);
    for needle in [
        MSIG_HEX,
        MSIG_ASM,
        MSIG_SIG,
        MSIG_SIG_ASM,
        MSIG_WIT,
        MSIG_INNER,
        MSIG_LEAF,
    ] {
        assert!(body.contains(needle), "fixture omitted {needle}: {body}");
    }
    let (mut host, rec) = host_with(
        Network::Mainnet,
        BTreeMap::from([(format!("/tx/{txid}"), body)]),
    );
    let opened = host.open_transaction(txid);
    let paths = rec.gets();
    assert!(
        paths.iter().any(|path| path == &format!("/tx/{txid}")),
        "GET /tx/{txid} was not requested: {paths:?}"
    );
    assert!(
        paths.iter().all(|path| !path.contains('?')
            && !path.contains('#')
            && !path.starts_with("/address/")),
        "a transaction open invented a route, a query, or a fragment: {paths:?}"
    );
    (opened, painted(&host))
}

fn assert_multisig_bytes_off(painted: &str) {
    for needle in [
        MSIG_HEX,
        MSIG_ASM,
        MSIG_SIG,
        MSIG_SIG_ASM,
        MSIG_WIT,
        MSIG_INNER,
        MSIG_LEAF,
        "scriptpubkey",
        "script asm",
        "leaf version",
        "Leaf version",
        "<span",
        "output-type",
        "scriptpubkey-type",
    ] {
        assert!(
            !painted.contains(needle),
            "{needle} leaked into the screen: {painted}"
        );
    }
}

fn assert_multisig_leaves_other_rows(painted: &str) {
    for line in [
        "Transaction fee",
        "P2PK",
        "OP_RETURN",
        "Type ANCHOR",
        "Type V0_P2WSH",
        "Show all",
        "Show more",
        "P2TR tapscript",
        "P2SH redeem script",
    ] {
        assert!(
            !painted.contains(line),
            "{line} changed on a multisig screen: {painted}"
        );
    }
}

/// Output script type `multisig` from `GET /tx/:txid`.
///
/// A nonempty `vout.scriptpubkey_type` of `multisig` paints the exact line
/// `Type MULTISIG`. An empty string, a missing key, and JSON null omit that
/// line. `output script type multisig` stays the indexer spelling. Script hex,
/// script asm, scriptsig, witness, the inner tapscript, and leaf version stay
/// off the screen. This is not an address span. `GET /address/:script` is not
/// called. The path has no query and no fragment. The confirmation line on
/// this fixture stays `Confirmed`. The count line `confirmations 1` is not
/// this fixture.
#[test]
fn transaction_screen_paints_multisig_output_type() {
    let (opened, painted) = paint_multisig_tx(
        "msigone",
        &multisig_vout(r#""scriptpubkey_type":"multisig","#),
    );
    opened.expect("multisig output");
    assert!(
        painted.contains("tx txid msigone"),
        "the parsed transaction was hidden: {painted}"
    );
    assert_eq!(
        painted
            .lines()
            .filter(|line| *line == INPUTS_OUTPUTS_HEADING)
            .count(),
        1,
        "Inputs & Outputs was dropped: {painted}"
    );
    assert_eq!(
        painted
            .lines()
            .filter(|line| *line == "Type MULTISIG")
            .count(),
        1,
        "scriptpubkey_type multisig did not paint the exact line Type MULTISIG: {painted}"
    );
    assert_output_type_lines(&painted, &["Type MULTISIG"]);
    assert_eq!(
        painted.matches("Type MULTISIG").count(),
        1,
        "Type MULTISIG was not once: {painted}"
    );
    assert_eq!(
        painted.matches("MULTISIG").count(),
        1,
        "MULTISIG leaked outside Type MULTISIG: {painted}"
    );
    assert_eq!(
        painted
            .lines()
            .filter(|line| *line == "output script type multisig")
            .count(),
        1,
        "the indexer spelling was dropped: {painted}"
    );
    assert_eq!(
        painted.matches("multisig").count(),
        1,
        "lowercase multisig leaked outside the indexer spelling: {painted}"
    );
    assert!(
        !painted
            .lines()
            .any(|line| line == "MULTISIG" || line == "multisig"),
        "an address classification line was painted: {painted}"
    );
    let rows: Vec<&str> = painted.lines().collect();
    let previous = rows
        .iter()
        .position(|line| *line == "Previous output script");
    let address = rows
        .iter()
        .position(|line| *line == "output address  value 1");
    let raw = rows
        .iter()
        .position(|line| *line == "output script type multisig");
    let upper = rows.iter().position(|line| *line == "Type MULTISIG");
    assert!(
        previous.is_some() && address.is_some() && raw.is_some() && upper.is_some(),
        "Previous output script or the multisig type lines were dropped: {painted}"
    );
    assert!(
        previous.unwrap() < address.unwrap()
            && address.unwrap() < raw.unwrap()
            && raw.unwrap() < upper.unwrap(),
        "Type MULTISIG was not after the indexer spelling on the output row: {painted}"
    );
    assert_one_fee_and_confirmed(&painted);
    assert_multisig_leaves_other_rows(&painted);
    assert_multisig_bytes_off(&painted);

    let (empty_opened, empty) =
        paint_multisig_tx("msigempty", &multisig_vout(r#""scriptpubkey_type":"","#));
    empty_opened.expect("empty output type");
    assert!(
        empty.contains("tx txid msigempty"),
        "an empty type hid the transaction: {empty}"
    );
    assert!(
        empty.lines().any(|line| line == "Previous output script"),
        "an empty output type dropped Previous output script: {empty}"
    );
    assert_output_type_lines(&empty, &[]);
    assert!(
        !empty.contains("output script type"),
        "an empty type painted a script type line: {empty}"
    );
    assert!(
        !empty.contains("Type MULTISIG"),
        "an empty type painted Type MULTISIG: {empty}"
    );
    assert!(!empty.to_ascii_lowercase().contains("multisig"), "{empty}");
    assert_one_fee_and_confirmed(&empty);
    assert_multisig_leaves_other_rows(&empty);
    assert_multisig_bytes_off(&empty);

    for (txid, type_field) in [
        ("msigmiss", ""),
        ("msignull", r#""scriptpubkey_type":null,"#),
    ] {
        let (_opened, omitted) = paint_multisig_tx(txid, &multisig_vout(type_field));
        assert!(
            omitted.contains(txid),
            "a missing scriptpubkey_type hid the screen: {omitted}"
        );
        assert_output_type_lines(&omitted, &[]);
        assert!(
            !omitted.contains("Type MULTISIG"),
            "a missing scriptpubkey_type painted Type MULTISIG: {omitted}"
        );
        assert!(
            !omitted.to_ascii_lowercase().contains("multisig"),
            "{omitted}"
        );
        assert_multisig_leaves_other_rows(&omitted);
        assert_multisig_bytes_off(&omitted);
    }
}

const ETYP_HEX: &str = "etyppubhexdead";
const ETYP_ASM: &str = "etypasmdead";
const ETYP_SIG: &str = "etypsighexdead";
const ETYP_SIG_ASM: &str = "etypsigasmdead";
const ETYP_WIT: &str = "etypwitdead";
const ETYP_INNER: &str = "etypinnerdead";
const ETYP_LEAF: &str = "etypleafdead";

fn empty_type_vin() -> String {
    format!(
        r#"{{"txid":"prev","vout":0,"prevout":{{"scriptpubkey":"{ETYP_HEX}","scriptpubkey_asm":"{ETYP_ASM}","scriptpubkey_type":"v0_p2wpkh","value":5}},"scriptsig":"{ETYP_SIG}","scriptsig_asm":"{ETYP_SIG_ASM}","witness":["{ETYP_WIT}"],"is_coinbase":false,"sequence":1,"inner_witnessscript_asm":"{ETYP_INNER}","leaf_version":"{ETYP_LEAF}"}}"#
    )
}

fn empty_type_vout(type_field: &str) -> String {
    format!(
        r#"{{"scriptpubkey":"{ETYP_HEX}","scriptpubkey_asm":"{ETYP_ASM}",{type_field}"value":1}}"#
    )
}

fn empty_type_tx(txid: &str, vout: &str) -> String {
    format!(
        r#"{{"txid":"{txid}","version":1,"locktime":0,"vin":[{}],"vout":[{vout}],"size":10,"weight":400,"sigops":0,"fee":50,"status":{{"confirmed":true,"block_height":4}}}}"#,
        empty_type_vin()
    )
}

fn paint_empty_type_tx(txid: &str, vout: &str) -> (Result<(), ClientError>, String) {
    assert!(
        !txid.is_empty() && !txid.contains('/') && !txid.contains('?') && !txid.contains('#'),
        "{txid}"
    );
    assert!(
        !txid.to_ascii_lowercase().contains("empty"),
        "the txid must not contain the type word: {txid}"
    );
    let body = empty_type_tx(txid, vout);
    for needle in [
        ETYP_HEX,
        ETYP_ASM,
        ETYP_SIG,
        ETYP_SIG_ASM,
        ETYP_WIT,
        ETYP_INNER,
        ETYP_LEAF,
    ] {
        assert!(body.contains(needle), "fixture omitted {needle}: {body}");
    }
    let (mut host, rec) = host_with(
        Network::Mainnet,
        BTreeMap::from([(format!("/tx/{txid}"), body)]),
    );
    let opened = host.open_transaction(txid);
    let paths = rec.gets();
    assert!(
        paths.iter().any(|path| path == &format!("/tx/{txid}")),
        "GET /tx/{txid} was not requested: {paths:?}"
    );
    assert!(
        paths.iter().all(|path| !path.contains('?')
            && !path.contains('#')
            && !path.starts_with("/address/")),
        "a transaction open invented a route, a query, or a fragment: {paths:?}"
    );
    (opened, painted(&host))
}

fn assert_empty_type_bytes_off(painted: &str) {
    for needle in [
        ETYP_HEX,
        ETYP_ASM,
        ETYP_SIG,
        ETYP_SIG_ASM,
        ETYP_WIT,
        ETYP_INNER,
        ETYP_LEAF,
        "scriptpubkey",
        "script asm",
        "leaf version",
        "Leaf version",
        "<span",
        "output-type",
        "scriptpubkey-type",
    ] {
        assert!(
            !painted.contains(needle),
            "{needle} leaked into the screen: {painted}"
        );
    }
}

fn assert_empty_type_leaves_other_rows(painted: &str) {
    for line in [
        "Transaction fee",
        "P2PK",
        "OP_RETURN",
        "Type ANCHOR",
        "Type V0_P2WSH",
        "Type MULTISIG",
        "Show all",
        "Show more",
        "P2TR tapscript",
        "P2SH redeem script",
        "Why is this block empty?",
    ] {
        assert!(
            !painted.contains(line),
            "{line} changed on an empty-type screen: {painted}"
        );
    }
}

/// Output script type `empty` from `GET /tx/:txid`.
///
/// A nonempty `vout.scriptpubkey_type` of `empty` paints the exact line
/// `Type EMPTY`. An empty string, a missing key, and JSON null omit that
/// line. Those three are not the type value `empty`. `output script type empty`
/// stays the indexer spelling. Script hex, script asm, scriptsig, witness,
/// the inner tapscript, and leaf version stay off the screen. This is not an
/// address span. `GET /address/:script` is not called. The path has no query
/// and no fragment. The confirmation line on this fixture stays `Confirmed`.
/// The count line `confirmations 1` is not this fixture. `Why is this block
/// empty?` is a different string and is not this screen.
#[test]
fn transaction_screen_paints_empty_output_type() {
    let (opened, painted) = paint_empty_type_tx(
        "etypeone",
        &empty_type_vout(r#""scriptpubkey_type":"empty","#),
    );
    opened.expect("empty output type");
    assert!(
        painted.contains("tx txid etypeone"),
        "the parsed transaction was hidden: {painted}"
    );
    assert_eq!(
        painted
            .lines()
            .filter(|line| *line == INPUTS_OUTPUTS_HEADING)
            .count(),
        1,
        "Inputs & Outputs was dropped: {painted}"
    );
    assert_eq!(
        painted.lines().filter(|line| *line == "Type EMPTY").count(),
        1,
        "scriptpubkey_type empty did not paint the exact line Type EMPTY: {painted}"
    );
    assert_output_type_lines(&painted, &["Type EMPTY"]);
    assert_eq!(
        painted.matches("Type EMPTY").count(),
        1,
        "Type EMPTY was not once: {painted}"
    );
    assert_eq!(
        painted.matches("EMPTY").count(),
        1,
        "EMPTY leaked outside Type EMPTY: {painted}"
    );
    assert_eq!(
        painted
            .lines()
            .filter(|line| *line == "output script type empty")
            .count(),
        1,
        "the indexer spelling was dropped: {painted}"
    );
    assert_eq!(
        painted.matches("empty").count(),
        1,
        "lowercase empty leaked outside the indexer spelling: {painted}"
    );
    assert!(
        !painted
            .lines()
            .any(|line| line == "EMPTY" || line == "empty"),
        "an address classification line was painted: {painted}"
    );
    let rows: Vec<&str> = painted.lines().collect();
    let previous = rows
        .iter()
        .position(|line| *line == "Previous output script");
    let address = rows
        .iter()
        .position(|line| *line == "output address  value 1");
    let raw = rows
        .iter()
        .position(|line| *line == "output script type empty");
    let upper = rows.iter().position(|line| *line == "Type EMPTY");
    assert!(
        previous.is_some() && address.is_some() && raw.is_some() && upper.is_some(),
        "Previous output script or the empty type lines were dropped: {painted}"
    );
    assert!(
        previous.unwrap() < address.unwrap()
            && address.unwrap() < raw.unwrap()
            && raw.unwrap() < upper.unwrap(),
        "Type EMPTY was not after the indexer spelling on the output row: {painted}"
    );
    assert_one_fee_and_confirmed(&painted);
    assert_empty_type_leaves_other_rows(&painted);
    assert_empty_type_bytes_off(&painted);

    let (blank_opened, blank) =
        paint_empty_type_tx("etypeblank", &empty_type_vout(r#""scriptpubkey_type":"","#));
    blank_opened.expect("blank output type");
    assert!(
        blank.contains("tx txid etypeblank"),
        "an empty type string hid the transaction: {blank}"
    );
    assert!(
        blank.lines().any(|line| line == "Previous output script"),
        "an empty output type dropped Previous output script: {blank}"
    );
    assert_output_type_lines(&blank, &[]);
    assert!(
        !blank.contains("output script type"),
        "an empty type string painted a script type line: {blank}"
    );
    assert!(
        !blank.contains("Type EMPTY"),
        "an empty type string painted Type EMPTY: {blank}"
    );
    assert!(!blank.to_ascii_lowercase().contains("empty"), "{blank}");
    assert_one_fee_and_confirmed(&blank);
    assert_empty_type_leaves_other_rows(&blank);
    assert_empty_type_bytes_off(&blank);

    for (txid, type_field) in [
        ("etypemiss", ""),
        ("etypenull", r#""scriptpubkey_type":null,"#),
    ] {
        let (_opened, omitted) = paint_empty_type_tx(txid, &empty_type_vout(type_field));
        assert!(
            omitted.contains(txid),
            "a missing scriptpubkey_type hid the screen: {omitted}"
        );
        assert_output_type_lines(&omitted, &[]);
        assert!(
            !omitted.contains("Type EMPTY"),
            "a missing scriptpubkey_type painted Type EMPTY: {omitted}"
        );
        assert!(!omitted.to_ascii_lowercase().contains("empty"), "{omitted}");
        assert_empty_type_leaves_other_rows(&omitted);
        assert_empty_type_bytes_off(&omitted);
    }
}

const PACKAGE_TXID: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";

fn paint_package(error_field: &str) -> String {
    let body = format!(
        r#"{{"package_msg":"success","tx-results":{{"pkg-key":{{"txid":"{PACKAGE_TXID}"{error_field}}}}}}}"#
    );
    let (mut host, rec) = host_with(
        Network::Mainnet,
        BTreeMap::from([("/txs/package".to_string(), body)]),
    );
    host.activate(Screen::Broadcast).expect("broadcast");
    host.broadcast.raw_tx = "abcd,ef01".to_string();
    host.submit().expect("package");
    assert_eq!(rec.posts()[0].0, "/txs/package", "{:?}", rec.posts());
    painted(&host)
}

/// `PackageTx.error` from `POST /txs/package`.
///
/// A non-empty error prints on its own line next to that result's txid,
/// including the text `0`. A missing key, JSON null, and an empty string omit
/// the label. This is not `reject-reason`.
#[test]
fn package_error_prints_when_present_and_omits_when_missing() {
    let present = paint_package(r#","error":"txn-mempool-conflict""#);
    assert!(
        present
            .lines()
            .any(|line| line == "error txn-mempool-conflict"),
        "a present error did not show the line error txn-mempool-conflict: {present}"
    );
    assert!(
        present.contains("broadcast package success"),
        "package success was dropped: {present}"
    );
    assert!(
        present.contains(PACKAGE_TXID),
        "package txid was dropped: {present}"
    );

    let zero = paint_package(r#","error":"0""#);
    assert!(
        zero.lines().any(|line| line == "error 0"),
        "the text 0 was omitted: {zero}"
    );
    assert!(
        zero.contains("broadcast package success"),
        "package success was dropped: {zero}"
    );
    assert!(
        zero.contains(PACKAGE_TXID),
        "package txid was dropped: {zero}"
    );

    let absent = paint_package("");
    assert!(
        absent.contains("broadcast package success"),
        "package success was dropped: {absent}"
    );
    assert!(
        absent.contains(PACKAGE_TXID),
        "package txid was dropped: {absent}"
    );
    assert!(
        !absent.lines().any(|line| line.starts_with("error")),
        "a missing error key painted a line: {absent}"
    );

    let null = paint_package(r#","error":null"#);
    assert!(
        null.contains("broadcast package success"),
        "package success was dropped: {null}"
    );
    assert!(
        null.contains(PACKAGE_TXID),
        "package txid was dropped: {null}"
    );
    assert!(
        !null.lines().any(|line| line.starts_with("error")),
        "a null error painted a line: {null}"
    );

    let empty = paint_package(r#","error":"""#);
    assert!(
        empty.contains("broadcast package success"),
        "package success was dropped: {empty}"
    );
    assert!(
        empty.contains(PACKAGE_TXID),
        "package txid was dropped: {empty}"
    );
    assert!(
        !empty.lines().any(|line| line.starts_with("error")),
        "an empty error painted a line: {empty}"
    );
}

fn paint_tx_adjusted(txid: &str, weight_field: &str, sigops_field: &str) -> String {
    let weight_line = if weight_field.is_empty() {
        String::new()
    } else {
        format!("{weight_field},")
    };
    let sigops_line = if sigops_field.is_empty() {
        String::new()
    } else {
        format!("{sigops_field},")
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
        {sigops_line}
        "fee":1,
        "status":{{"confirmed":false}}
    }}"#
    );
    let (mut host, _) = host_with(
        Network::Mainnet,
        BTreeMap::from([(format!("/tx/{txid}"), tx)]),
    );
    host.open_transaction(txid)
        .unwrap_or_else(|err| panic!("tx {txid} did not open: {err}"));
    painted(&host)
}

/// Script asm `OP_RETURN` stays off the screen. `Type OP_RETURN` is the
/// uppercased output `scriptpubkey_type`, not that asm. `Previous output type
/// OP_RETURN` is the same uppercase spelling on an input. The address cell
/// paints one exact line `OP_RETURN` for an `op_return` output. Any other
/// line that contains `OP_RETURN` is asm or script text and stays off.
fn assert_op_return_asm_stays_off(painted: &str) {
    assert!(
        painted.lines().all(|line| {
            line == "Type OP_RETURN"
                || line == "Previous output type OP_RETURN"
                || line == "OP_RETURN"
                || !line.contains("OP_RETURN")
        }),
        "OP_RETURN leaked outside the type lines and the address cell: {painted}"
    );
}

fn assert_script_hex_stays_off(painted: &str) {
    assert!(!painted.contains("0014aa"), "{painted}");
    assert!(!painted.contains("deadbeef"), "{painted}");
    assert_op_return_asm_stays_off(painted);
    assert!(!painted.contains("OP_CHECKSIG"), "{painted}");
    assert!(!painted.contains("scriptpubkey"), "{painted}");
}

/// Adjusted vsize is `max(weight / 4, sigops * 5)` from `GET /tx/:txid`.
///
/// The division is floating point, then rounded to 2 decimal places, half away
/// from zero. This is not the integer virtual-size line. A missing sigops key,
/// JSON null sigops, or a missing weight omits the label. Explicit sigops 0
/// still paints. Script hex and asm stay off the screen.
#[test]
fn transaction_screen_paints_adjusted_vsize() {
    let seven = paint_tx_adjusted("adjv7", r#""weight":7"#, r#""sigops":0"#);
    assert!(
        seven.lines().any(|line| line == "Adjusted vsize 1.75"),
        "weight 7 and sigops 0 did not paint the line Adjusted vsize 1.75: {seven}"
    );
    assert!(
        seven.lines().any(|line| line == "virtual size 1"),
        "weight 7 dropped virtual size 1: {seven}"
    );
    assert!(
        !seven.lines().any(|line| line == "Adjusted vsize 1"),
        "weight 7 painted Adjusted vsize 1: {seven}"
    );
    assert!(
        !seven.lines().any(|line| line == "Adjusted vsize 1.00"),
        "weight 7 painted Adjusted vsize 1.00: {seven}"
    );
    assert_script_hex_stays_off(&seven);

    let twenty = paint_tx_adjusted("adjv20", r#""weight":20"#, r#""sigops":2"#);
    assert!(
        twenty.lines().any(|line| line == "Adjusted vsize 10.00"),
        "weight 20 and sigops 2 did not paint Adjusted vsize 10.00: {twenty}"
    );
    assert!(
        !twenty.lines().any(|line| line == "Adjusted vsize 5.00"),
        "weight 20 painted the weight side: {twenty}"
    );
    assert_script_hex_stays_off(&twenty);

    let eight = paint_tx_adjusted("adjv8", r#""weight":8"#, r#""sigops":3"#);
    assert!(
        eight.lines().any(|line| line == "Adjusted vsize 15.00"),
        "weight 8 and sigops 3 did not paint Adjusted vsize 15.00: {eight}"
    );
    assert_script_hex_stays_off(&eight);

    let absent = paint_tx_adjusted("adjsigabs", r#""weight":7"#, "");
    assert!(
        absent.contains("tx txid adjsigabs"),
        "missing sigops hid the transaction: {absent}"
    );
    assert!(
        !absent.contains("Adjusted vsize"),
        "missing sigops painted Adjusted vsize: {absent}"
    );
    assert!(
        !absent.contains("1.75"),
        "missing sigops painted 1.75: {absent}"
    );
    assert_script_hex_stays_off(&absent);

    let null = paint_tx_adjusted("adjsignull", r#""weight":7"#, r#""sigops":null"#);
    assert!(
        null.contains("tx txid adjsignull"),
        "null sigops hid the transaction: {null}"
    );
    assert!(
        !null.contains("Adjusted vsize"),
        "null sigops painted Adjusted vsize: {null}"
    );
    assert!(!null.contains("1.75"), "null sigops painted 1.75: {null}");
    assert_script_hex_stays_off(&null);

    let no_weight = paint_tx_adjusted("adjwabs", "", r#""sigops":1"#);
    assert!(
        no_weight.contains("tx txid adjwabs"),
        "missing weight hid the transaction: {no_weight}"
    );
    assert!(
        !no_weight.contains("Adjusted vsize"),
        "missing weight painted Adjusted vsize: {no_weight}"
    );
    assert_script_hex_stays_off(&no_weight);
}

fn paint_tx_size(txid: &str, size: u64) -> String {
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
        "size":{size},
        "weight":40,
        "sigops":1,
        "fee":1,
        "status":{{"confirmed":false}}
    }}"#
    );
    let (mut host, _) = host_with(
        Network::Mainnet,
        BTreeMap::from([(format!("/tx/{txid}"), tx)]),
    );
    host.open_transaction(txid)
        .unwrap_or_else(|err| panic!("tx {txid} did not open: {err}"));
    painted(&host)
}

fn assert_size_line(painted: &str, size: u64, line: &str) {
    assert!(
        painted.lines().any(|row| row == line),
        "size {size} did not paint the line {line}: {painted}"
    );
    assert!(
        painted.contains(&format!("size {size} weight")),
        "size {size} dropped the raw header: {painted}"
    );
    assert!(
        painted.lines().any(|row| row == "virtual size 10"),
        "size {size} dropped virtual size 10: {painted}"
    );
    assert!(
        painted.lines().any(|row| row == "Adjusted vsize 10.00"),
        "size {size} dropped Adjusted vsize 10.00: {painted}"
    );
    assert!(!painted.contains("abcdwitnesshex"), "{painted}");
    assert_script_hex_stays_off(painted);
}

/// Transaction size from `GET /tx/:txid`, scaled like the Angular `bytes` pipe.
///
/// The unit is chosen from the raw byte count before rounding. Thresholds are
/// powers of 1000, not 1024. The scaled value is rounded to 2 decimal places,
/// half away from zero, then printed the way JavaScript `Number.toString` does.
/// The raw header `size N` stays. This is not the block or address list.
/// Script hex, asm, scriptsig, and witness stay off the screen.
#[test]
fn transaction_screen_paints_size() {
    let fifteen = paint_tx_size("size1500", 1500);
    assert!(
        fifteen.lines().any(|line| line == "Size 1.5 kB"),
        "size 1500 did not paint the line Size 1.5 kB: {fifteen}"
    );
    assert!(
        !fifteen.lines().any(|line| line == "Size 1 kB"),
        "size 1500 painted Size 1 kB: {fifteen}"
    );
    assert!(
        fifteen.contains("size 1500"),
        "size 1500 dropped the raw header: {fifteen}"
    );
    assert_size_line(&fifteen, 1500, "Size 1.5 kB");

    assert_size_line(&paint_tx_size("size10", 10), 10, "Size 10 B");
    assert_size_line(&paint_tx_size("size0", 0), 0, "Size 0 B");
    assert_size_line(&paint_tx_size("size1000", 1000), 1000, "Size 1 kB");

    let kib = paint_tx_size("size1024", 1024);
    assert_size_line(&kib, 1024, "Size 1.02 kB");
    assert!(
        !kib.lines().any(|line| line == "Size 1024 B"),
        "size 1024 stayed in bytes: {kib}"
    );
    assert!(
        !kib.contains("KiB"),
        "size 1024 used a 1024-based unit: {kib}"
    );

    let almost_mb = paint_tx_size("size999999", 999_999);
    assert_size_line(&almost_mb, 999_999, "Size 1000 kB");
    assert!(
        !almost_mb.lines().any(|line| line == "Size 1 MB"),
        "size 999999 promoted to MB: {almost_mb}"
    );

    let mb = paint_tx_size("size1000000", 1_000_000);
    assert_size_line(&mb, 1_000_000, "Size 1 MB");
    assert!(
        !mb.lines().any(|line| line == "Size 1000 kB"),
        "size 1000000 stayed in kB: {mb}"
    );

    let five = paint_tx_size("size1005", 1005);
    assert_size_line(&five, 1005, "Size 1 kB");
    assert!(
        !five.lines().any(|line| line == "Size 1.01 kB"),
        "size 1005 painted the line Size 1.01 kB: {five}"
    );
    assert!(!five.contains("1.01"), "size 1005 painted 1.01: {five}");
}

fn paint_tx_fee(txid: &str, fee: u64) -> String {
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
        "weight":40,
        "sigops":1,
        "fee":{fee},
        "status":{{"confirmed":false}}
    }}"#
    );
    let (mut host, _) = host_with(
        Network::Mainnet,
        BTreeMap::from([(format!("/tx/{txid}"), tx)]),
    );
    host.open_transaction(txid)
        .unwrap_or_else(|err| panic!("tx {txid} did not open: {err}"));
    painted(&host)
}

/// Transaction preview fee from `GET /tx/:txid`.
///
/// Angular paints `Fee`, the en-US `number` pipe, then `sats`. An integer
/// gets thousands separators and no fraction digits. Explicit 0 paints.
/// The raw header `fee N` stays ungrouped. This line is not the list row.
/// Script hex, asm, scriptsig, and witness stay off the screen.
#[test]
fn transaction_screen_paints_fee() {
    let zero = paint_tx_fee("fee0", 0);
    assert!(
        zero.lines().any(|line| line == "Fee 0 sats"),
        "fee 0 did not paint the line Fee 0 sats: {zero}"
    );
    assert!(
        zero.contains("fee 0"),
        "fee 0 dropped the raw header: {zero}"
    );
    assert!(!zero.contains("abcdwitnesshex"), "{zero}");
    assert_script_hex_stays_off(&zero);

    let one = paint_tx_fee("fee1", 1);
    assert!(
        one.lines().any(|line| line == "Fee 1 sats"),
        "fee 1 did not paint the line Fee 1 sats: {one}"
    );
    assert!(one.contains("fee 1"), "fee 1 dropped the raw header: {one}");
    assert_script_hex_stays_off(&one);

    let fifteen = paint_tx_fee("fee1500", 1500);
    assert!(
        fifteen.lines().any(|line| line == "Fee 1,500 sats"),
        "fee 1500 did not paint the line Fee 1,500 sats: {fifteen}"
    );
    assert!(
        fifteen.contains("fee 1500"),
        "fee 1500 dropped the raw header: {fifteen}"
    );
    assert!(
        !fifteen.lines().any(|line| line == "Fee 1500 sats"),
        "fee 1500 painted an ungrouped Fee line: {fifteen}"
    );
    assert_script_hex_stays_off(&fifteen);

    let million = paint_tx_fee("fee1000000", 1_000_000);
    assert!(
        million.lines().any(|line| line == "Fee 1,000,000 sats"),
        "fee 1000000 did not paint the line Fee 1,000,000 sats: {million}"
    );
    assert!(
        million.contains("fee 1000000"),
        "fee 1000000 dropped the raw header: {million}"
    );
    assert_script_hex_stays_off(&million);
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

/// Transaction fee rate from `GET /tx/:txid`.
///
/// Angular paints `Fee rate`, then `fee / (weight / 4)` through the default
/// `feeRounding` pipe, then `sat/vB`. A rate under 10 keeps two fraction
/// digits. A rate from 10 up to but not including 100 keeps one. A rate of
/// 100 or more is an integer with thousands separators. Explicit fee 0
/// paints. A missing weight, JSON null, or zero weight omits the line.
/// The raw header `fee_rate N` stays and uses integer virtual size. This
/// line is not that header, not Effective fee rate, and not Maximum fee
/// rate. Script hex, asm, scriptsig, and witness stay off the screen.
#[test]
fn transaction_screen_paints_fee_rate() {
    let half = paint_tx_fee_rate("ratehalf", 50, r#""weight":400"#);
    assert!(
        half.lines().any(|line| line == "Fee rate 0.50 sat/vB"),
        "fee 50 and weight 400 did not paint Fee rate 0.50 sat/vB: {half}"
    );
    assert!(
        half.contains("fee_rate 0.5"),
        "fee 50 and weight 400 dropped the raw fee_rate header: {half}"
    );
    assert!(
        half.lines().any(|line| line == "Fee 50 sats"),
        "fee 50 dropped Fee 50 sats: {half}"
    );
    assert_script_hex_stays_off(&half);
    assert!(!half.contains("abcdwitnesshex"), "{half}");

    let zero = paint_tx_fee_rate("ratezero", 0, r#""weight":400"#);
    assert!(
        zero.lines().any(|line| line == "Fee rate 0.00 sat/vB"),
        "fee 0 did not paint Fee rate 0.00 sat/vB: {zero}"
    );
    assert!(
        zero.contains("fee_rate 0"),
        "fee 0 dropped the raw fee_rate header: {zero}"
    );
    assert_script_hex_stays_off(&zero);

    let thirds = paint_tx_fee_rate("ratethird", 1, r#""weight":7"#);
    assert!(
        thirds.lines().any(|line| line == "Fee rate 0.57 sat/vB"),
        "fee 1 and weight 7 did not paint Fee rate 0.57 sat/vB: {thirds}"
    );
    assert!(
        thirds.contains("fee_rate 1"),
        "fee 1 and weight 7 dropped the integer fee_rate header: {thirds}"
    );
    assert!(
        !thirds.lines().any(|line| line == "Fee rate 1 sat/vB"),
        "fee 1 and weight 7 painted the integer rate: {thirds}"
    );
    assert_script_hex_stays_off(&thirds);

    let fifteen = paint_tx_fee_rate("rate15", 15, r#""weight":4"#);
    assert!(
        fifteen.lines().any(|line| line == "Fee rate 15.0 sat/vB"),
        "fee 15 and weight 4 did not paint Fee rate 15.0 sat/vB: {fifteen}"
    );
    assert_script_hex_stays_off(&fifteen);

    let grouped = paint_tx_fee_rate("rate1500", 1500, r#""weight":4"#);
    assert!(
        grouped.lines().any(|line| line == "Fee rate 1,500 sat/vB"),
        "fee 1500 and weight 4 did not paint Fee rate 1,500 sat/vB: {grouped}"
    );
    assert!(
        !grouped.lines().any(|line| line == "Fee rate 1500 sat/vB"),
        "fee 1500 painted an ungrouped Fee rate line: {grouped}"
    );
    assert!(
        grouped.contains("fee_rate 1500"),
        "fee 1500 dropped the raw fee_rate header: {grouped}"
    );
    assert_script_hex_stays_off(&grouped);

    let absent = paint_tx_fee_rate("rateabs", 50, "");
    assert!(
        absent.contains("tx txid rateabs"),
        "missing weight hid the transaction: {absent}"
    );
    assert!(
        !absent.lines().any(|line| line.starts_with("Fee rate ")),
        "missing weight painted Fee rate: {absent}"
    );
    assert_script_hex_stays_off(&absent);

    let null = paint_tx_fee_rate("ratenull", 50, r#""weight":null"#);
    assert!(
        null.contains("tx txid ratenull"),
        "null weight hid the transaction: {null}"
    );
    assert!(
        !null.lines().any(|line| line.starts_with("Fee rate ")),
        "null weight painted Fee rate: {null}"
    );
    assert_script_hex_stays_off(&null);

    let zero_weight = paint_tx_fee_rate("ratew0", 50, r#""weight":0"#);
    assert!(
        zero_weight.contains("tx txid ratew0"),
        "weight 0 hid the transaction: {zero_weight}"
    );
    assert!(
        !zero_weight
            .lines()
            .any(|line| line.starts_with("Fee rate ")),
        "weight 0 painted Fee rate: {zero_weight}"
    );
    assert!(
        !zero_weight.contains("fee_rate"),
        "weight 0 painted a fee rate: {zero_weight}"
    );
    assert_script_hex_stays_off(&zero_weight);
}

/// Transaction sigops from `GET /tx/:txid`.
///
/// Angular paints `Sigops` and the en-US `number` pipe. An integer gets
/// thousands separators and no fraction digits. Explicit 0 paints. A missing
/// key or JSON null omits the row and is not zero. The raw header is lowercase
/// `sigops N` and stays when the JSON has the field. This is not that header.
/// The FAQ tooltip and Transaction hex stay off the screen. Script hex and asm
/// stay off the screen.
#[test]
fn transaction_screen_paints_sigops() {
    let zero = paint_tx_adjusted("sig0", r#""weight":7"#, r#""sigops":0"#);
    assert!(
        zero.lines().any(|line| line == "Sigops 0"),
        "sigops 0 did not paint the line Sigops 0: {zero}"
    );
    assert!(
        zero.lines().any(|line| line == "Adjusted vsize 1.75"),
        "weight 7 and sigops 0 dropped Adjusted vsize 1.75: {zero}"
    );
    assert!(
        zero.lines().any(|line| line == "virtual size 1"),
        "weight 7 and sigops 0 dropped virtual size 1: {zero}"
    );
    assert!(
        zero.contains("sigops 0"),
        "sigops 0 dropped the raw header: {zero}"
    );
    assert_script_hex_stays_off(&zero);

    let one = paint_tx_adjusted("sig1", r#""weight":40"#, r#""sigops":1"#);
    assert!(
        one.lines().any(|line| line == "Sigops 1"),
        "sigops 1 did not paint the line Sigops 1: {one}"
    );
    assert!(
        one.contains("sigops 1"),
        "sigops 1 dropped the raw header: {one}"
    );
    assert_script_hex_stays_off(&one);

    let fifteen = paint_tx_adjusted("sig1500", r#""weight":40"#, r#""sigops":1500"#);
    assert!(
        fifteen.lines().any(|line| line == "Sigops 1,500"),
        "sigops 1500 did not paint the line Sigops 1,500: {fifteen}"
    );
    assert!(
        !fifteen.lines().any(|line| line == "Sigops 1500"),
        "sigops 1500 painted an ungrouped Sigops line: {fifteen}"
    );
    assert!(
        fifteen.contains("sigops 1500"),
        "sigops 1500 dropped the raw header: {fifteen}"
    );
    assert_script_hex_stays_off(&fifteen);

    let absent = paint_tx_adjusted("sigabs", r#""weight":7"#, "");
    assert!(
        absent.contains("tx txid sigabs"),
        "missing sigops hid the transaction: {absent}"
    );
    assert!(
        !absent.lines().any(|line| line == "Sigops 0"),
        "missing sigops painted the line Sigops 0: {absent}"
    );
    assert!(
        !absent.contains("Sigops"),
        "missing sigops painted Sigops: {absent}"
    );
    assert!(
        !absent.contains("Adjusted vsize"),
        "missing sigops painted Adjusted vsize: {absent}"
    );
    assert_script_hex_stays_off(&absent);

    let null = paint_tx_adjusted("signull", r#""weight":7"#, r#""sigops":null"#);
    assert!(
        null.contains("tx txid signull"),
        "null sigops hid the transaction: {null}"
    );
    assert!(
        !null.lines().any(|line| line == "Sigops 0"),
        "null sigops painted the line Sigops 0: {null}"
    );
    assert!(
        !null.contains("Sigops"),
        "null sigops painted Sigops: {null}"
    );
    assert!(
        !null.contains("Adjusted vsize"),
        "null sigops painted Adjusted vsize: {null}"
    );
    assert_script_hex_stays_off(&null);
}

fn paint_tx_version(txid: &str, version: u32) -> String {
    let tx = format!(
        r#"{{
        "txid":"{txid}",
        "version":{version},
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
        "weight":40,
        "sigops":1,
        "fee":1,
        "status":{{"confirmed":false}}
    }}"#
    );
    let (mut host, _) = host_with(
        Network::Mainnet,
        BTreeMap::from([(format!("/tx/{txid}"), tx)]),
    );
    host.open_transaction(txid)
        .unwrap_or_else(|err| panic!("tx {txid} did not open: {err}"));
    painted(&host)
}

/// Transaction version from `GET /tx/:txid`.
///
/// Angular paints `Version` and the en-US `number` pipe. An integer gets
/// thousands separators and no fraction digits. The row is always shown.
/// Explicit 0 paints. The raw header is lowercase `version N` and stays.
/// This is not that header. Script hex, asm, scriptsig, and witness stay off.
#[test]
fn transaction_screen_paints_version() {
    let zero = paint_tx_version("ver0", 0);
    assert!(
        zero.lines().any(|line| line == "Version 0"),
        "version 0 did not paint the line Version 0: {zero}"
    );
    assert!(
        zero.contains("version 0"),
        "version 0 dropped the raw header: {zero}"
    );
    assert!(!zero.contains("abcdwitnesshex"), "{zero}");
    assert_script_hex_stays_off(&zero);

    let one = paint_tx_version("ver1", 1);
    assert!(
        one.lines().any(|line| line == "Version 1"),
        "version 1 did not paint the line Version 1: {one}"
    );
    assert!(
        one.contains("version 1"),
        "version 1 dropped the raw header: {one}"
    );
    assert_script_hex_stays_off(&one);

    let two = paint_tx_version("ver2", 2);
    assert!(
        two.lines().any(|line| line == "Version 2"),
        "version 2 did not paint the line Version 2: {two}"
    );
    assert!(
        two.contains("version 2"),
        "version 2 dropped the raw header: {two}"
    );
    assert_script_hex_stays_off(&two);

    let fifteen = paint_tx_version("ver1500", 1500);
    assert!(
        fifteen.lines().any(|line| line == "Version 1,500"),
        "version 1500 did not paint the line Version 1,500: {fifteen}"
    );
    assert!(
        !fifteen.lines().any(|line| line == "Version 1500"),
        "version 1500 painted an ungrouped Version line: {fifteen}"
    );
    assert!(
        fifteen.contains("version 1500"),
        "version 1500 dropped the raw header: {fifteen}"
    );
    assert_script_hex_stays_off(&fifteen);
}

fn paint_tx_locktime(txid: &str, locktime: u32) -> String {
    let tx = format!(
        r#"{{
        "txid":"{txid}",
        "version":1,
        "locktime":{locktime},
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
        "weight":7,
        "sigops":0,
        "fee":1,
        "status":{{"confirmed":false}}
    }}"#
    );
    let (mut host, _) = host_with(
        Network::Mainnet,
        BTreeMap::from([(format!("/tx/{txid}"), tx)]),
    );
    host.open_transaction(txid)
        .unwrap_or_else(|err| panic!("tx {txid} did not open: {err}"));
    painted(&host)
}

/// Transaction locktime from `GET /tx/:txid`.
///
/// Angular paints `Locktime` whenever the transaction is shown, including
/// locktime 0. The value is the raw integer, with no thousands separator
/// and no left-to-right mark. The raw header is lowercase `locktime N`
/// and stays. This is not that header. Script hex, asm, scriptsig, and
/// witness stay off.
#[test]
fn transaction_screen_paints_locktime() {
    let zero = paint_tx_locktime("lt0", 0);
    assert!(
        zero.lines().any(|line| line == "Locktime 0"),
        "locktime 0 did not paint the line Locktime 0: {zero}"
    );
    assert!(
        zero.lines().any(|line| line == "virtual size 1"),
        "locktime 0 dropped virtual size 1: {zero}"
    );
    assert!(
        zero.lines().any(|line| line == "Adjusted vsize 1.75"),
        "locktime 0 dropped Adjusted vsize 1.75: {zero}"
    );
    assert!(
        zero.lines().any(|line| line == "Sigops 0"),
        "locktime 0 dropped Sigops 0: {zero}"
    );
    assert!(
        zero.contains("locktime 0"),
        "locktime 0 dropped the raw header: {zero}"
    );
    assert!(
        !zero.contains('\u{200e}'),
        "locktime painted a left-to-right mark: {zero}"
    );
    assert!(!zero.contains("abcdwitnesshex"), "{zero}");
    assert_script_hex_stays_off(&zero);

    let raw = paint_tx_locktime("lt500m", 500_000_000);
    assert!(
        raw.lines().any(|line| line == "Locktime 500000000"),
        "locktime 500000000 did not paint the line Locktime 500000000: {raw}"
    );
    assert!(
        !raw.contains("500,000,000"),
        "locktime 500000000 painted a thousands separator: {raw}"
    );
    assert!(
        !raw.lines().any(|line| line == "Locktime 500,000,000"),
        "locktime 500000000 painted the line Locktime 500,000,000: {raw}"
    );
    assert!(
        raw.contains("locktime 500000000"),
        "locktime 500000000 dropped the raw header: {raw}"
    );
    assert!(
        !raw.contains('\u{200e}'),
        "locktime painted a left-to-right mark: {raw}"
    );
    assert!(!raw.contains("abcdwitnesshex"), "{raw}");
    assert_script_hex_stays_off(&raw);
}

/// Address word `Issuance` on one input from `GET /tx/:txid`.
///
/// The line is exact and appears only when that input has an issuance object
/// and no prevout. JSON null and an omitted prevout key are both no prevout.
/// An ordinary prevout does not gain the line. A missing issuance key and
/// JSON null issuance do not gain it. An issuance object that also has a
/// prevout keeps the field lines and does not gain this word. Existing input
/// lines stay. No amount is invented. The path has no query and no fragment.
#[test]
fn transaction_screen_paints_issuance_label() {
    const LABEL: &str = "Issuance";
    const ASSET: &str = "ce091c998b83c78bb71a632313ba3760f1763d9cfcffae02258ffa9865a37bd2";
    const ENTROPY: &str = "eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee";

    fn issuance_object() -> String {
        format!(r#"{{"asset_id":"{ASSET}","is_reissuance":false,"asset_entropy":"{ENTROPY}"}}"#)
    }

    fn vin(prevout_field: &str, issuance_field: &str) -> String {
        format!(
            r#"{{"txid":"prev","vout":0,{prevout_field}{issuance_field}"scriptsig":"deadbeef","scriptsig_asm":"OP_CHECKSIG","witness":["abcdwitnesshex"],"is_coinbase":false,"sequence":1}}"#
        )
    }

    fn tx_body(txid: &str, vins: &str) -> String {
        format!(
            r#"{{"txid":"{txid}","version":1,"locktime":0,"vin":[{vins}],"vout":[{{"scriptpubkey":"0014aa","scriptpubkey_asm":"OP_RETURN","scriptpubkey_type":"op_return","value":9}}],"size":10,"weight":400,"sigops":0,"fee":50,"status":{{"confirmed":true,"block_height":4,"block_hash":"tiphash1","block_time":1600000000}}}}"#
        )
    }

    fn paint(txid: &str, vins: &str) -> String {
        assert!(
            !txid.is_empty() && !txid.contains('/') && !txid.contains('?') && !txid.contains('#'),
            "{txid}"
        );
        let (mut host, rec) = host_with(
            Network::Mainnet,
            BTreeMap::from([(format!("/tx/{txid}"), tx_body(txid, vins))]),
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

    fn label_count(text: &str) -> usize {
        text.lines().filter(|line| *line == LABEL).count()
    }

    fn assert_screen_keeps_existing_rows(text: &str, txid: &str) {
        let lines: Vec<&str> = text.lines().collect();
        let marker = "block_time 1600000000 ";
        assert!(
            lines.windows(2).any(|pair| {
                pair[0] == "Timestamp"
                    && pair[1].contains(&format!("tx txid {txid} "))
                    && pair[1].contains(marker)
                    && !pair[1].contains("Timestamp")
                    && !pair[1].contains(',')
            }),
            "Timestamp was not immediately before the raw block_time line: {text}"
        );
        assert!(
            text.lines().any(|line| line == "Fee rate 0.50 sat/vB"),
            "Fee rate was dropped: {text}"
        );
        assert!(
            text.contains("fee_rate 0.5"),
            "the raw fee_rate header was dropped: {text}"
        );
        assert!(
            text.lines().any(|line| line == "Confirmed"),
            "Confirmed was dropped or the letter case changed: {text}"
        );
        assert!(
            text.contains("confirmed true"),
            "the raw confirmed header changed letter case: {text}"
        );
        assert!(
            text.lines().any(|line| line == "Inputs & Outputs"),
            "Inputs & Outputs was dropped: {text}"
        );
        assert!(!text.contains("1,600,000,000"), "{text}");
        assert!(!text.contains("abcdwitnesshex"), "{text}");
        assert_script_hex_stays_off(text);
        assert_paint_is_not_html(text);
    }

    fn assert_no_amount(text: &str) {
        assert!(
            !text.lines().any(|line| {
                line.starts_with("issuance asset amount")
                    || line.starts_with("issuance token amount")
                    || line.contains("Confidential")
            }),
            "an amount was invented: {text}"
        );
    }

    let issued_vin = vin(
        r#""prevout":null,"#,
        &format!(r#""issuance":{},"#, issuance_object()),
    );
    let ordinary_vin = vin(
        r#""prevout":{"scriptpubkey":"0014aa","scriptpubkey_asm":"OP_RETURN","scriptpubkey_type":"v0_p2wpkh","scriptpubkey_address":"bc1qordinary","value":5},"#,
        "",
    );
    let issued = paint("issuelabel", &format!("{issued_vin},{ordinary_vin}"));
    assert_eq!(
        label_count(&issued),
        1,
        "{LABEL} was not one exact line on the issuance input with no prevout: {issued}"
    );
    let rows: Vec<&str> = issued.lines().collect();
    let outpoint = rows
        .iter()
        .position(|line| *line == "input outpoint prev:0");
    let label_at = rows.iter().position(|line| *line == LABEL);
    let asset_line = format!("issuance asset {ASSET}");
    let asset_at = rows.iter().position(|line| *line == asset_line);
    assert!(
        outpoint.is_some() && label_at.is_some() && asset_at.is_some(),
        "the outpoint, {LABEL}, or the asset line was missing: {issued}"
    );
    assert!(
        outpoint.unwrap() < label_at.unwrap() && label_at.unwrap() < asset_at.unwrap(),
        "{LABEL} was not on the issuance input before its field lines: {issued}"
    );
    assert!(
        issued
            .lines()
            .any(|line| line == "issuance reissuance false"),
        "the reissuance line was dropped: {issued}"
    );
    assert!(
        issued
            .lines()
            .any(|line| line == format!("issuance entropy {ENTROPY}")),
        "the entropy line was dropped: {issued}"
    );
    assert!(
        issued.lines().any(|line| line == "sequence 1"),
        "sequence was dropped: {issued}"
    );
    assert!(
        issued
            .lines()
            .any(|line| line == "input address bc1qordinary value 5"),
        "the ordinary input address was dropped: {issued}"
    );
    assert!(
        issued
            .lines()
            .any(|line| line == "input script type v0_p2wpkh"),
        "the ordinary input script type was dropped: {issued}"
    );
    assert!(
        issued
            .lines()
            .any(|line| line == "Previous output type V0_P2WPKH"),
        "Previous output type was dropped: {issued}"
    );
    assert!(
        !issued.lines().any(|line| line == "UNKNOWN"),
        "the issuance input painted UNKNOWN: {issued}"
    );
    assert_no_amount(&issued);
    assert_screen_keeps_existing_rows(&issued, "issuelabel");

    let omitted = paint(
        "issueomit",
        &vin("", &format!(r#""issuance":{},"#, issuance_object())),
    );
    assert_eq!(
        label_count(&omitted),
        1,
        "an omitted prevout key did not paint {LABEL} once: {omitted}"
    );
    assert!(
        omitted.contains(&format!("issuance asset {ASSET}")),
        "an omitted prevout key dropped the asset line: {omitted}"
    );
    assert!(
        omitted.lines().any(|line| line == "input outpoint prev:0"),
        "an omitted prevout key dropped the outpoint: {omitted}"
    );
    assert_no_amount(&omitted);
    assert_screen_keeps_existing_rows(&omitted, "issueomit");

    let missing = paint("issuemiss", &vin(r#""prevout":null,"#, ""));
    assert_eq!(
        label_count(&missing),
        0,
        "a missing issuance key painted {LABEL}: {missing}"
    );
    assert_eq!(
        missing.lines().filter(|line| *line == "UNKNOWN").count(),
        1,
        "a missing issuance key did not keep UNKNOWN: {missing}"
    );
    assert!(
        missing.lines().any(|line| line == "input outpoint prev:0"),
        "a missing issuance key dropped the outpoint: {missing}"
    );
    assert_screen_keeps_existing_rows(&missing, "issuemiss");

    let null_issuance = paint(
        "issuenull",
        &vin(r#""prevout":null,"#, r#""issuance":null,"#),
    );
    assert_eq!(
        label_count(&null_issuance),
        0,
        "JSON null issuance painted {LABEL}: {null_issuance}"
    );
    assert_eq!(
        null_issuance
            .lines()
            .filter(|line| *line == "UNKNOWN")
            .count(),
        1,
        "JSON null issuance did not keep UNKNOWN: {null_issuance}"
    );
    assert_screen_keeps_existing_rows(&null_issuance, "issuenull");

    let with_prevout = paint(
        "issueprev",
        &vin(
            r#""prevout":{"scriptpubkey":"0014aa","scriptpubkey_asm":"OP_RETURN","scriptpubkey_type":"v0_p2wpkh","scriptpubkey_address":"bc1qwithprev","value":8},"#,
            &format!(r#""issuance":{},"#, issuance_object()),
        ),
    );
    assert_eq!(
        label_count(&with_prevout),
        0,
        "an issuance input that has a prevout painted {LABEL}: {with_prevout}"
    );
    assert!(
        with_prevout.contains(&format!("issuance asset {ASSET}")),
        "a prevout dropped the issuance field lines: {with_prevout}"
    );
    assert!(
        with_prevout
            .lines()
            .any(|line| line == "input address bc1qwithprev value 8"),
        "the prevout address was dropped: {with_prevout}"
    );
    assert!(
        !with_prevout.lines().any(|line| line == "UNKNOWN"),
        "a prevout painted UNKNOWN: {with_prevout}"
    );
    assert_screen_keeps_existing_rows(&with_prevout, "issueprev");
}

/// Output address cell from `GET /tx/:txid` when `scriptpubkey_type` is `fee`.
///
/// The scriptpubkeyType pipe returns the exact line `Transaction fee` for the
/// lowercase type `fee`. The output cell calls that pipe only when the output
/// has no address, is not a peg-out, and is not OP_RETURN. An omitted address,
/// an empty address, and JSON null `pegout` are that case. An address, a
/// peg-out object, an OP_RETURN output, and an ordinary output do not gain the
/// line. An input whose prevout type is `fee` stays `Previous output type FEE`
/// and does not gain this line. `output script type fee` and `Type FEE` stay.
/// The details Fee row and the Fee rate row stay one each. The path has no
/// query and no fragment.
#[test]
fn transaction_screen_paints_transaction_fee_label() {
    const LABEL: &str = "Transaction fee";

    fn vin(prevout_field: &str) -> String {
        format!(
            r#"{{"txid":"prev","vout":0,{prevout_field}"scriptsig":"deadbeef","scriptsig_asm":"OP_CHECKSIG","witness":["abcdwitnesshex"],"is_coinbase":false,"sequence":1}}"#
        )
    }

    fn tx_body(txid: &str, vin_json: &str, vout_json: &str) -> String {
        format!(
            r#"{{"txid":"{txid}","version":1,"locktime":0,"vin":[{vin_json}],"vout":[{vout_json}],"size":10,"weight":400,"sigops":0,"fee":50,"status":{{"confirmed":true,"block_height":4,"block_hash":"tiphash1","block_time":1600000000}}}}"#
        )
    }

    fn paint(txid: &str, vin_json: &str, vout_json: &str) -> String {
        assert!(
            !txid.is_empty() && !txid.contains('/') && !txid.contains('?') && !txid.contains('#'),
            "{txid}"
        );
        let (mut host, rec) = host_with(
            Network::Mainnet,
            BTreeMap::from([(format!("/tx/{txid}"), tx_body(txid, vin_json, vout_json))]),
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

    fn label_count(text: &str) -> usize {
        text.lines().filter(|line| *line == LABEL).count()
    }

    fn assert_one_fee_row_and_one_fee_rate(text: &str) {
        assert_eq!(
            text.lines().filter(|line| *line == "Fee 50 sats").count(),
            1,
            "the details Fee row was dropped or repeated: {text}"
        );
        assert_eq!(
            text.lines()
                .filter(|line| *line == "Fee rate 0.50 sat/vB")
                .count(),
            1,
            "the Fee rate row was dropped or repeated: {text}"
        );
        assert!(
            text.contains("fee_rate 0.5"),
            "the raw fee_rate header was dropped: {text}"
        );
        assert_eq!(
            text.lines()
                .filter(|line| line.starts_with("Fee ") && !line.starts_with("Fee rate "))
                .count(),
            1,
            "a second Fee row was painted: {text}"
        );
    }

    fn assert_screen_keeps_timestamp(text: &str, txid: &str) {
        let lines: Vec<&str> = text.lines().collect();
        let marker = "block_time 1600000000 ";
        assert!(
            lines.windows(2).any(|pair| {
                pair[0] == "Timestamp"
                    && pair[1].contains(&format!("tx txid {txid} "))
                    && pair[1].contains(marker)
                    && !pair[1].contains("Timestamp")
                    && !pair[1].contains(',')
            }),
            "Timestamp was not immediately before the raw block_time line: {text}"
        );
        assert!(
            text.lines().any(|line| line == "Inputs & Outputs"),
            "Inputs & Outputs was dropped: {text}"
        );
        assert!(
            text.lines().any(|line| line == "Confirmed"),
            "Confirmed was dropped: {text}"
        );
        assert!(!text.contains("1,600,000,000"), "{text}");
        assert!(
            !text.contains("2020") && !text.contains("ago"),
            "the block time was formatted: {text}"
        );
        assert!(!text.contains("abcdwitnesshex"), "{text}");
        assert!(
            !text.lines().any(|line| line.contains("Confidential")),
            "a value was unblinded or invented as Confidential: {text}"
        );
        assert_script_hex_stays_off(text);
        assert_paint_is_not_html(text);
    }

    let ordinary_vin = vin(
        r#""prevout":{"scriptpubkey":"0014aa","scriptpubkey_asm":"OP_0","scriptpubkey_type":"v0_p2wpkh","scriptpubkey_address":"bc1qin","value":8},"#,
    );
    let fee_vout =
        r#"{"scriptpubkey":"","scriptpubkey_asm":"","scriptpubkey_type":"fee","value":0}"#;
    let ordinary_vout = r#"{"scriptpubkey":"0014aa","scriptpubkey_asm":"OP_0","scriptpubkey_type":"v0_p2wpkh","scriptpubkey_address":"bc1qordinary","value":9}"#;
    let fee_out = paint(
        "feelabel",
        &ordinary_vin,
        &format!("{fee_vout},{ordinary_vout}"),
    );
    assert_eq!(
        label_count(&fee_out),
        1,
        "{LABEL} was not one exact line on the fee output with no address: {fee_out}"
    );
    let rows: Vec<&str> = fee_out.lines().collect();
    let fee_address = rows
        .iter()
        .position(|line| *line == "output address  value 0");
    let script_type = rows
        .iter()
        .position(|line| *line == "output script type fee");
    let type_fee = rows.iter().position(|line| *line == "Type FEE");
    let label_at = rows.iter().position(|line| *line == LABEL);
    let ordinary_at = rows
        .iter()
        .position(|line| *line == "output address bc1qordinary value 9");
    assert!(
        fee_address.is_some()
            && script_type.is_some()
            && type_fee.is_some()
            && label_at.is_some()
            && ordinary_at.is_some(),
        "the fee output lines or the ordinary output were dropped: {fee_out}"
    );
    assert!(
        fee_address.unwrap() < script_type.unwrap()
            && script_type.unwrap() < type_fee.unwrap()
            && type_fee.unwrap() < label_at.unwrap()
            && label_at.unwrap() < ordinary_at.unwrap(),
        "{LABEL} was not on the fee output after Type FEE and before the ordinary output: {fee_out}"
    );
    assert!(
        fee_out
            .lines()
            .any(|line| line == "output script type v0_p2wpkh"),
        "the ordinary output script type was dropped: {fee_out}"
    );
    assert!(
        fee_out.lines().any(|line| line == "Type V0_P2WPKH"),
        "the ordinary output type was dropped: {fee_out}"
    );
    assert_eq!(
        fee_out
            .lines()
            .filter(|line| *line == "output script type fee")
            .count(),
        1,
        "output script type fee was dropped or repeated: {fee_out}"
    );
    assert_eq!(
        fee_out.lines().filter(|line| *line == "Type FEE").count(),
        1,
        "Type FEE was dropped or repeated: {fee_out}"
    );
    assert_one_fee_row_and_one_fee_rate(&fee_out);
    assert_screen_keeps_timestamp(&fee_out, "feelabel");

    let empty_address = r#"{"scriptpubkey":"","scriptpubkey_asm":"","scriptpubkey_type":"fee","scriptpubkey_address":"","value":2}"#;
    let empty = paint("feeempty", &ordinary_vin, empty_address);
    assert_eq!(
        label_count(&empty),
        1,
        "an empty address did not paint {LABEL} once: {empty}"
    );
    assert!(
        empty.lines().any(|line| line == "output address  value 2"),
        "the empty address output row was dropped: {empty}"
    );
    assert!(
        empty.lines().any(|line| line == "output script type fee"),
        "output script type fee was dropped for an empty address: {empty}"
    );
    assert!(
        empty.lines().any(|line| line == "Type FEE"),
        "Type FEE was dropped for an empty address: {empty}"
    );
    assert_one_fee_row_and_one_fee_rate(&empty);
    assert_screen_keeps_timestamp(&empty, "feeempty");

    let null_peg = r#"{"scriptpubkey":"","scriptpubkey_asm":"","scriptpubkey_type":"fee","value":0,"pegout":null}"#;
    let null_pegout = paint("feenullpeg", &ordinary_vin, null_peg);
    assert_eq!(
        label_count(&null_pegout),
        1,
        "JSON null pegout did not paint {LABEL} once: {null_pegout}"
    );
    assert!(
        !null_pegout.lines().any(|line| line == "Liquid Peg Out"),
        "JSON null pegout painted Liquid Peg Out: {null_pegout}"
    );
    assert!(
        null_pegout
            .lines()
            .any(|line| line == "output script type fee"),
        "output script type fee was dropped for null pegout: {null_pegout}"
    );
    assert!(
        null_pegout.lines().any(|line| line == "Type FEE"),
        "Type FEE was dropped for null pegout: {null_pegout}"
    );
    assert_one_fee_row_and_one_fee_rate(&null_pegout);

    let with_address = r#"{"scriptpubkey":"0014bb","scriptpubkey_asm":"OP_0","scriptpubkey_type":"fee","scriptpubkey_address":"bc1qfeeaddr","value":3}"#;
    let addressed = paint("feeaddr", &ordinary_vin, with_address);
    assert_eq!(
        label_count(&addressed),
        0,
        "a fee output with an address painted {LABEL}: {addressed}"
    );
    assert!(
        addressed
            .lines()
            .any(|line| line == "output address bc1qfeeaddr value 3"),
        "the addressed fee output was dropped: {addressed}"
    );
    assert!(
        addressed
            .lines()
            .any(|line| line == "output script type fee"),
        "output script type fee was dropped when the output has an address: {addressed}"
    );
    assert!(
        addressed.lines().any(|line| line == "Type FEE"),
        "Type FEE was dropped when the output has an address: {addressed}"
    );
    assert_one_fee_row_and_one_fee_rate(&addressed);
    assert_screen_keeps_timestamp(&addressed, "feeaddr");

    let pegout = r#"{"scriptpubkey":"","scriptpubkey_asm":"","scriptpubkey_type":"fee","value":0,"pegout":{"genesis_hash":"aa","scriptpubkey":"bb","scriptpubkey_asm":"OP_TRUE"}}"#;
    let pegged = paint("feepeg", &ordinary_vin, pegout);
    assert_eq!(
        label_count(&pegged),
        0,
        "a peg-out painted {LABEL}: {pegged}"
    );
    assert!(
        pegged.lines().any(|line| line == "Liquid Peg Out"),
        "Liquid Peg Out was dropped: {pegged}"
    );
    assert!(
        pegged.lines().any(|line| line == "output script type fee"),
        "output script type fee was dropped on a peg-out: {pegged}"
    );
    assert!(
        pegged.lines().any(|line| line == "Type FEE"),
        "Type FEE was dropped on a peg-out: {pegged}"
    );
    assert_one_fee_row_and_one_fee_rate(&pegged);
    assert_screen_keeps_timestamp(&pegged, "feepeg");

    let fee_prevout = vin(
        r#""prevout":{"scriptpubkey":"","scriptpubkey_asm":"","scriptpubkey_type":"fee","value":5},"#,
    );
    let input_fee = paint("feein", &fee_prevout, ordinary_vout);
    assert_eq!(
        label_count(&input_fee),
        0,
        "an input whose prevout type is fee painted {LABEL}: {input_fee}"
    );
    assert!(
        input_fee
            .lines()
            .any(|line| line == "input script type fee"),
        "input script type fee was dropped: {input_fee}"
    );
    assert!(
        input_fee
            .lines()
            .any(|line| line == "Previous output type FEE"),
        "Previous output type FEE was dropped: {input_fee}"
    );
    assert!(
        !input_fee.lines().any(|line| line == "Type FEE"),
        "Type FEE was painted on the input path: {input_fee}"
    );
    assert!(
        input_fee
            .lines()
            .any(|line| line == "output address bc1qordinary value 9"),
        "the ordinary output was dropped beside a fee input: {input_fee}"
    );
    assert_one_fee_row_and_one_fee_rate(&input_fee);
    assert_screen_keeps_timestamp(&input_fee, "feein");

    let op_return = r#"{"scriptpubkey":"6a","scriptpubkey_asm":"OP_RETURN","scriptpubkey_type":"op_return","value":0}"#;
    let ordinary = paint("feeord", &ordinary_vin, op_return);
    assert_eq!(
        label_count(&ordinary),
        0,
        "an OP_RETURN output painted {LABEL}: {ordinary}"
    );
    assert!(
        ordinary.lines().any(|line| line == "Type OP_RETURN"),
        "Type OP_RETURN was dropped: {ordinary}"
    );
    assert!(
        ordinary
            .lines()
            .any(|line| line == "output script type op_return"),
        "output script type op_return was dropped: {ordinary}"
    );
    assert!(
        !ordinary.lines().any(|line| line == "Type FEE"),
        "an ordinary output painted Type FEE: {ordinary}"
    );
    assert_one_fee_row_and_one_fee_rate(&ordinary);
    assert_screen_keeps_timestamp(&ordinary, "feeord");
}

/// Address-type caption from `GET /tx/:txid` when `scriptpubkey_type` is `fee`.
///
/// The address-type element body is the exact line `fee`. `Type FEE`,
/// `Transaction fee`, and `output script type fee` stay. An ordinary output
/// does not gain this line. An input whose prevout type is `fee` does not
/// gain this line. The path has no query and no fragment.
#[test]
fn transaction_screen_paints_address_type_fee() {
    const LABEL: &str = "fee";

    fn tx_body(txid: &str, vin_json: &str, vout_json: &str) -> String {
        format!(
            r#"{{"txid":"{txid}","version":1,"locktime":0,"vin":[{vin_json}],"vout":[{vout_json}],"size":10,"weight":400,"sigops":0,"fee":50,"status":{{"confirmed":true,"block_height":4,"block_hash":"tiphash1","block_time":1600000000}}}}"#
        )
    }

    fn paint(txid: &str, vin_json: &str, vout_json: &str) -> String {
        assert!(
            !txid.is_empty() && !txid.contains('/') && !txid.contains('?') && !txid.contains('#'),
            "{txid}"
        );
        let (mut host, rec) = host_with(
            Network::Mainnet,
            BTreeMap::from([(format!("/tx/{txid}"), tx_body(txid, vin_json, vout_json))]),
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

    let fee_vout =
        r#"{"scriptpubkey":"","scriptpubkey_asm":"","scriptpubkey_type":"fee","value":0}"#;
    let fee_out = paint("addrtypefee", "", fee_vout);
    assert!(
        fee_out.lines().any(|line| line == LABEL),
        "the exact line is absent: {fee_out}"
    );
    assert_eq!(
        fee_out.lines().filter(|line| *line == LABEL).count(),
        1,
        "the address-type fee caption was not once: {fee_out}"
    );
    assert!(
        fee_out.lines().any(|line| line == "Type FEE"),
        "Type FEE was dropped: {fee_out}"
    );
    assert!(
        fee_out.lines().any(|line| line == "Transaction fee"),
        "Transaction fee was dropped: {fee_out}"
    );
    assert!(
        fee_out.lines().any(|line| line == "output script type fee"),
        "output script type fee was dropped: {fee_out}"
    );

    let ordinary = r#"{"scriptpubkey":"0014aa","scriptpubkey_asm":"OP_0","scriptpubkey_type":"v0_p2wpkh","scriptpubkey_address":"bc1qordinary","value":9}"#;
    let ordinary_out = paint("addrtypeord", "", ordinary);
    assert!(
        !ordinary_out.lines().any(|line| line == LABEL),
        "an ordinary output painted the address-type fee caption: {ordinary_out}"
    );

    let fee_input = r#""prevout":{"scriptpubkey":"","scriptpubkey_asm":"","scriptpubkey_type":"fee","value":5},"#;
    let input_out = paint(
        "addrtypein",
        &format!(
            r#"{{"txid":"prev","vout":0,{fee_input}"scriptsig":"deadbeef","scriptsig_asm":"OP_CHECKSIG","witness":["abcdwitnesshex"],"is_coinbase":false,"sequence":1}}"#
        ),
        ordinary,
    );
    assert!(
        !input_out.lines().any(|line| line == LABEL),
        "an input whose prevout type is fee painted the address-type fee caption: {input_out}"
    );
    assert!(
        input_out
            .lines()
            .any(|line| line == "Previous output type FEE"),
        "Previous output type FEE was dropped: {input_out}"
    );
}

/// Output address cell from `GET /tx/:txid` when `scriptpubkey_type` is `p2pk`.
///
/// The scriptpubkeyType pipe returns the exact line `P2PK` for the lowercase
/// type `p2pk`. The output address cell prints that word when the output has
/// no address. That branch is earlier than peg-out and earlier than the pipe
/// call, so a peg-out object does not remove the line. An omitted address, an
/// empty address, and JSON null address are that case. An address does not
/// gain the line. An input whose prevout type is `p2pk` stays `Previous output
/// type P2PK` and does not gain this line. `output script type p2pk` and
/// `Type P2PK` stay. The details Fee row and the Fee rate row stay one each.
/// `Transaction fee` stays only on a fee output. The path has no query and no
/// fragment.
#[test]
fn transaction_screen_paints_p2pk_label() {
    const LABEL: &str = "P2PK";

    fn vin(prevout_field: &str) -> String {
        format!(
            r#"{{"txid":"prev","vout":0,{prevout_field}"scriptsig":"deadbeef","scriptsig_asm":"OP_CHECKSIG","witness":["abcdwitnesshex"],"is_coinbase":false,"sequence":1}}"#
        )
    }

    fn tx_body(txid: &str, vin_json: &str, vout_json: &str) -> String {
        format!(
            r#"{{"txid":"{txid}","version":1,"locktime":0,"vin":[{vin_json}],"vout":[{vout_json}],"size":10,"weight":400,"sigops":0,"fee":50,"status":{{"confirmed":true,"block_height":4,"block_hash":"tiphash1","block_time":1600000000}}}}"#
        )
    }

    fn paint(txid: &str, vin_json: &str, vout_json: &str) -> String {
        assert!(
            !txid.is_empty() && !txid.contains('/') && !txid.contains('?') && !txid.contains('#'),
            "{txid}"
        );
        let (mut host, rec) = host_with(
            Network::Mainnet,
            BTreeMap::from([(format!("/tx/{txid}"), tx_body(txid, vin_json, vout_json))]),
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

    fn label_count(text: &str) -> usize {
        text.lines().filter(|line| *line == LABEL).count()
    }

    fn assert_one_fee_row_and_one_fee_rate(text: &str) {
        assert_eq!(
            text.lines().filter(|line| *line == "Fee 50 sats").count(),
            1,
            "the details Fee row was dropped or repeated: {text}"
        );
        assert_eq!(
            text.lines()
                .filter(|line| *line == "Fee rate 0.50 sat/vB")
                .count(),
            1,
            "the Fee rate row was dropped or repeated: {text}"
        );
        assert!(
            text.contains("fee_rate 0.5"),
            "the raw fee_rate header was dropped: {text}"
        );
        assert_eq!(
            text.lines()
                .filter(|line| line.starts_with("Fee ") && !line.starts_with("Fee rate "))
                .count(),
            1,
            "a second Fee row was painted: {text}"
        );
    }

    fn assert_screen_keeps_timestamp(text: &str, txid: &str) {
        let lines: Vec<&str> = text.lines().collect();
        let marker = "block_time 1600000000 ";
        assert!(
            lines.windows(2).any(|pair| {
                pair[0] == "Timestamp"
                    && pair[1].contains(&format!("tx txid {txid} "))
                    && pair[1].contains(marker)
                    && !pair[1].contains("Timestamp")
                    && !pair[1].contains(',')
            }),
            "Timestamp was not immediately before the raw block_time line: {text}"
        );
        assert!(
            text.lines().any(|line| line == "Inputs & Outputs"),
            "Inputs & Outputs was dropped: {text}"
        );
        assert!(
            text.lines().any(|line| line == "Confirmed"),
            "Confirmed was dropped: {text}"
        );
        assert!(!text.contains("1,600,000,000"), "{text}");
        assert!(
            !text.contains("2020") && !text.contains("ago"),
            "the block time was formatted: {text}"
        );
        assert!(!text.contains("abcdwitnesshex"), "{text}");
        assert!(
            !text.lines().any(|line| line.contains("Confidential")),
            "a value was unblinded or invented as Confidential: {text}"
        );
        assert_script_hex_stays_off(text);
        assert_paint_is_not_html(text);
    }

    fn assert_p2pk_type_lines(text: &str) {
        assert_eq!(
            text.lines()
                .filter(|line| *line == "output script type p2pk")
                .count(),
            1,
            "output script type p2pk was dropped or repeated: {text}"
        );
        assert_eq!(
            text.lines().filter(|line| *line == "Type P2PK").count(),
            1,
            "Type P2PK was dropped or repeated: {text}"
        );
    }

    let ordinary_vin = vin(
        r#""prevout":{"scriptpubkey":"0014aa","scriptpubkey_asm":"OP_0","scriptpubkey_type":"v0_p2wpkh","scriptpubkey_address":"bc1qin","value":8},"#,
    );
    let p2pk_vout = r#"{"scriptpubkey":"0014aa","scriptpubkey_asm":"OP_0","scriptpubkey_type":"p2pk","value":1}"#;
    let ordinary_vout = r#"{"scriptpubkey":"0014aa","scriptpubkey_asm":"OP_0","scriptpubkey_type":"v0_p2wpkh","scriptpubkey_address":"bc1qordinary","value":9}"#;
    let p2pk_out = paint(
        "p2pklabel",
        &ordinary_vin,
        &format!("{p2pk_vout},{ordinary_vout}"),
    );
    assert_eq!(
        label_count(&p2pk_out),
        1,
        "{LABEL} was not one exact line on the p2pk output with no address: {p2pk_out}"
    );
    let rows: Vec<&str> = p2pk_out.lines().collect();
    let p2pk_address = rows
        .iter()
        .position(|line| *line == "output address  value 1");
    let script_type = rows
        .iter()
        .position(|line| *line == "output script type p2pk");
    let type_p2pk = rows.iter().position(|line| *line == "Type P2PK");
    let label_at = rows.iter().position(|line| *line == LABEL);
    let ordinary_at = rows
        .iter()
        .position(|line| *line == "output address bc1qordinary value 9");
    assert!(
        p2pk_address.is_some()
            && script_type.is_some()
            && type_p2pk.is_some()
            && label_at.is_some()
            && ordinary_at.is_some(),
        "the p2pk output lines or the ordinary output were dropped: {p2pk_out}"
    );
    assert!(
        p2pk_address.unwrap() < script_type.unwrap()
            && script_type.unwrap() < type_p2pk.unwrap()
            && type_p2pk.unwrap() < label_at.unwrap()
            && label_at.unwrap() < ordinary_at.unwrap(),
        "{LABEL} was not on the p2pk output after Type P2PK and before the ordinary output: {p2pk_out}"
    );
    assert!(
        p2pk_out
            .lines()
            .any(|line| line == "output script type v0_p2wpkh"),
        "the ordinary output script type was dropped: {p2pk_out}"
    );
    assert!(
        p2pk_out.lines().any(|line| line == "Type V0_P2WPKH"),
        "the ordinary output type was dropped: {p2pk_out}"
    );
    assert_eq!(
        p2pk_out
            .lines()
            .filter(|line| *line == "Transaction fee")
            .count(),
        0,
        "a p2pk output painted Transaction fee: {p2pk_out}"
    );
    assert_p2pk_type_lines(&p2pk_out);
    assert_one_fee_row_and_one_fee_rate(&p2pk_out);
    assert_screen_keeps_timestamp(&p2pk_out, "p2pklabel");

    let empty_address = r#"{"scriptpubkey":"0014aa","scriptpubkey_asm":"OP_0","scriptpubkey_type":"p2pk","scriptpubkey_address":"","value":2}"#;
    let empty = paint("p2pkempty", &ordinary_vin, empty_address);
    assert_eq!(
        label_count(&empty),
        1,
        "an empty address did not paint {LABEL} once: {empty}"
    );
    assert!(
        empty.lines().any(|line| line == "output address  value 2"),
        "the empty address output row was dropped: {empty}"
    );
    assert_p2pk_type_lines(&empty);
    assert_one_fee_row_and_one_fee_rate(&empty);
    assert_screen_keeps_timestamp(&empty, "p2pkempty");

    let null_address = r#"{"scriptpubkey":"0014aa","scriptpubkey_asm":"OP_0","scriptpubkey_type":"p2pk","scriptpubkey_address":null,"value":4}"#;
    let null_addr = paint("p2pknulladdr", &ordinary_vin, null_address);
    assert_eq!(
        label_count(&null_addr),
        1,
        "JSON null address did not paint {LABEL} once: {null_addr}"
    );
    assert!(
        null_addr
            .lines()
            .any(|line| line == "output address  value 4"),
        "the null address output row was dropped: {null_addr}"
    );
    assert_p2pk_type_lines(&null_addr);
    assert_one_fee_row_and_one_fee_rate(&null_addr);

    let null_peg = r#"{"scriptpubkey":"0014aa","scriptpubkey_asm":"OP_0","scriptpubkey_type":"p2pk","value":1,"pegout":null}"#;
    let null_pegout = paint("p2pknullpeg", &ordinary_vin, null_peg);
    assert_eq!(
        label_count(&null_pegout),
        1,
        "JSON null pegout did not paint {LABEL} once: {null_pegout}"
    );
    assert!(
        !null_pegout.lines().any(|line| line == "Liquid Peg Out"),
        "JSON null pegout painted Liquid Peg Out: {null_pegout}"
    );
    assert_p2pk_type_lines(&null_pegout);
    assert_one_fee_row_and_one_fee_rate(&null_pegout);

    let pegout = r#"{"scriptpubkey":"0014aa","scriptpubkey_asm":"OP_0","scriptpubkey_type":"p2pk","value":1,"pegout":{"genesis_hash":"aa","scriptpubkey":"bb","scriptpubkey_asm":"OP_TRUE"}}"#;
    let pegged = paint("p2pkpeg", &ordinary_vin, pegout);
    assert_eq!(
        label_count(&pegged),
        1,
        "a peg-out p2pk output did not paint {LABEL} once: {pegged}"
    );
    assert!(
        pegged.lines().any(|line| line == "Liquid Peg Out"),
        "Liquid Peg Out was dropped: {pegged}"
    );
    assert_p2pk_type_lines(&pegged);
    assert_one_fee_row_and_one_fee_rate(&pegged);
    assert_screen_keeps_timestamp(&pegged, "p2pkpeg");

    let with_address = r#"{"scriptpubkey":"0014aa","scriptpubkey_asm":"OP_0","scriptpubkey_type":"p2pk","scriptpubkey_address":"bc1qp2pk","value":3}"#;
    let addressed = paint("p2pkaddr", &ordinary_vin, with_address);
    assert_eq!(
        label_count(&addressed),
        0,
        "a p2pk output with an address painted {LABEL}: {addressed}"
    );
    assert!(
        addressed
            .lines()
            .any(|line| line == "output address bc1qp2pk value 3"),
        "the addressed p2pk output was dropped: {addressed}"
    );
    assert_p2pk_type_lines(&addressed);
    assert_one_fee_row_and_one_fee_rate(&addressed);
    assert_screen_keeps_timestamp(&addressed, "p2pkaddr");

    let p2pk_prevout = vin(
        r#""prevout":{"scriptpubkey":"0014aa","scriptpubkey_asm":"OP_0","scriptpubkey_type":"p2pk","value":5},"#,
    );
    let input_p2pk = paint("p2pkin", &p2pk_prevout, ordinary_vout);
    assert_eq!(
        label_count(&input_p2pk),
        0,
        "an input whose prevout type is p2pk painted {LABEL}: {input_p2pk}"
    );
    assert!(
        input_p2pk
            .lines()
            .any(|line| line == "input script type p2pk"),
        "input script type p2pk was dropped: {input_p2pk}"
    );
    assert!(
        input_p2pk
            .lines()
            .any(|line| line == "Previous output type P2PK"),
        "Previous output type P2PK was dropped: {input_p2pk}"
    );
    assert!(
        !input_p2pk.lines().any(|line| line == "Type P2PK"),
        "Type P2PK was painted on the input path: {input_p2pk}"
    );
    assert!(
        input_p2pk
            .lines()
            .any(|line| line == "output address bc1qordinary value 9"),
        "the ordinary output was dropped beside a p2pk input: {input_p2pk}"
    );
    assert_one_fee_row_and_one_fee_rate(&input_p2pk);
    assert_screen_keeps_timestamp(&input_p2pk, "p2pkin");

    let op_return = r#"{"scriptpubkey":"6a","scriptpubkey_asm":"OP_RETURN","scriptpubkey_type":"op_return","value":0}"#;
    let op_return_out = paint("p2pkopret", &ordinary_vin, op_return);
    assert_eq!(
        label_count(&op_return_out),
        0,
        "an OP_RETURN output painted {LABEL}: {op_return_out}"
    );
    assert!(
        op_return_out.lines().any(|line| line == "Type OP_RETURN"),
        "Type OP_RETURN was dropped: {op_return_out}"
    );
    assert!(
        op_return_out
            .lines()
            .any(|line| line == "output script type op_return"),
        "output script type op_return was dropped: {op_return_out}"
    );
    assert_eq!(
        op_return_out
            .lines()
            .filter(|line| *line == "OP_RETURN")
            .count(),
        1,
        "the op_return address cell did not paint OP_RETURN once: {op_return_out}"
    );
    assert_one_fee_row_and_one_fee_rate(&op_return_out);
    assert_screen_keeps_timestamp(&op_return_out, "p2pkopret");

    let fee_vout =
        r#"{"scriptpubkey":"","scriptpubkey_asm":"","scriptpubkey_type":"fee","value":0}"#;
    let fee_out = paint("p2pkfee", &ordinary_vin, fee_vout);
    assert_eq!(
        label_count(&fee_out),
        0,
        "a fee output painted {LABEL}: {fee_out}"
    );
    assert_eq!(
        fee_out
            .lines()
            .filter(|line| *line == "Transaction fee")
            .count(),
        1,
        "Transaction fee was dropped or repeated on the fee output: {fee_out}"
    );
    assert!(
        fee_out.lines().any(|line| line == "Type FEE"),
        "Type FEE was dropped: {fee_out}"
    );
    assert!(
        fee_out.lines().any(|line| line == "output script type fee"),
        "output script type fee was dropped: {fee_out}"
    );
    assert_one_fee_row_and_one_fee_rate(&fee_out);
    assert_screen_keeps_timestamp(&fee_out, "p2pkfee");

    let p2pkh = r#"{"scriptpubkey":"0014aa","scriptpubkey_asm":"OP_0","scriptpubkey_type":"p2pkh","value":6}"#;
    let other = paint("p2pkother", &ordinary_vin, p2pkh);
    assert_eq!(
        label_count(&other),
        0,
        "a p2pkh output painted {LABEL}: {other}"
    );
    assert!(
        other.lines().any(|line| line == "Type P2PKH"),
        "Type P2PKH was dropped: {other}"
    );
    assert!(
        !other.lines().any(|line| line == "P2PKH"),
        "a bare P2PKH line was invented: {other}"
    );
    assert!(
        other.lines().any(|line| line == "output script type p2pkh"),
        "output script type p2pkh was dropped: {other}"
    );
    assert_one_fee_row_and_one_fee_rate(&other);
    assert_screen_keeps_timestamp(&other, "p2pkother");
}

/// Output address cell from `GET /tx/:txid` when `scriptpubkey_type` is `op_return`.
///
/// The scriptpubkeyType pipe returns the exact line `OP_RETURN` for the
/// lowercase type `op_return`. The output address cell prints that word when
/// the output has no address and is not a peg-out. An omitted address, an
/// empty address, JSON null address, and JSON null `pegout` are that case.
/// An address or a peg-out object does not gain the line. An input whose
/// prevout type is `op_return` stays `Previous output type OP_RETURN` and
/// does not gain this line. `output script type op_return` and `Type
/// OP_RETURN` stay. `P2PK` stays only on a p2pk output. `Transaction fee`
/// stays only on a fee output. The details Fee row and the Fee rate row stay
/// one each. `Issuance` stays on an issuance input with no prevout. The path
/// has no query and no fragment. Script asm, script hex, and the raw script
/// stay off. The block time stays the raw integer.
#[test]
fn transaction_screen_paints_op_return_label() {
    const LABEL: &str = "OP_RETURN";
    const ASM: &str = "OP_RETURN 68656c6c6f";
    const SCRIPT: &str = "6a0468656c6c6f";

    fn vin(prevout_field: &str) -> String {
        format!(
            r#"{{"txid":"prev","vout":0,{prevout_field}"scriptsig":"deadbeef","scriptsig_asm":"OP_CHECKSIG","witness":["abcdwitnesshex"],"is_coinbase":false,"sequence":1}}"#
        )
    }

    fn tx_body(txid: &str, vin_json: &str, vout_json: &str) -> String {
        format!(
            r#"{{"txid":"{txid}","version":1,"locktime":0,"vin":[{vin_json}],"vout":[{vout_json}],"size":10,"weight":400,"sigops":0,"fee":50,"status":{{"confirmed":true,"block_height":4,"block_hash":"tiphash1","block_time":1600000000}}}}"#
        )
    }

    fn paint(txid: &str, vin_json: &str, vout_json: &str) -> String {
        assert!(
            !txid.is_empty() && !txid.contains('/') && !txid.contains('?') && !txid.contains('#'),
            "{txid}"
        );
        let (mut host, rec) = host_with(
            Network::Mainnet,
            BTreeMap::from([(format!("/tx/{txid}"), tx_body(txid, vin_json, vout_json))]),
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

    fn label_count(text: &str) -> usize {
        text.lines().filter(|line| *line == LABEL).count()
    }

    fn assert_one_fee_row_and_one_fee_rate(text: &str) {
        assert_eq!(
            text.lines().filter(|line| *line == "Fee 50 sats").count(),
            1,
            "the details Fee row was dropped or repeated: {text}"
        );
        assert_eq!(
            text.lines()
                .filter(|line| *line == "Fee rate 0.50 sat/vB")
                .count(),
            1,
            "the Fee rate row was dropped or repeated: {text}"
        );
        assert!(
            text.contains("fee_rate 0.5"),
            "the raw fee_rate header was dropped: {text}"
        );
        assert_eq!(
            text.lines()
                .filter(|line| line.starts_with("Fee ") && !line.starts_with("Fee rate "))
                .count(),
            1,
            "a second Fee row was painted: {text}"
        );
    }

    fn assert_screen_keeps_timestamp(text: &str, txid: &str) {
        const ASM: &str = "OP_RETURN 68656c6c6f";
        const SCRIPT: &str = "6a0468656c6c6f";
        let lines: Vec<&str> = text.lines().collect();
        let marker = "block_time 1600000000 ";
        assert!(
            lines.windows(2).any(|pair| {
                pair[0] == "Timestamp"
                    && pair[1].contains(&format!("tx txid {txid} "))
                    && pair[1].contains(marker)
                    && !pair[1].contains("Timestamp")
                    && !pair[1].contains(',')
            }),
            "Timestamp was not immediately before the raw block_time line: {text}"
        );
        assert!(
            text.lines().any(|line| line == "Inputs & Outputs"),
            "Inputs & Outputs was dropped: {text}"
        );
        assert!(
            text.lines().any(|line| line == "Confirmed"),
            "Confirmed was dropped: {text}"
        );
        assert!(!text.contains("1,600,000,000"), "{text}");
        assert!(
            !text.contains("2020") && !text.contains("ago"),
            "the block time was formatted: {text}"
        );
        assert!(!text.contains("abcdwitnesshex"), "{text}");
        assert!(
            !text.lines().any(|line| line.contains("Confidential")),
            "a value was unblinded or invented as Confidential: {text}"
        );
        assert!(!text.contains(ASM), "script asm was painted: {text}");
        assert!(!text.contains(SCRIPT), "script hex was painted: {text}");
        assert_script_hex_stays_off(text);
        assert_paint_is_not_html(text);
    }

    fn assert_op_return_type_lines(text: &str) {
        assert_eq!(
            text.lines()
                .filter(|line| *line == "output script type op_return")
                .count(),
            1,
            "output script type op_return was dropped or repeated: {text}"
        );
        assert_eq!(
            text.lines()
                .filter(|line| *line == "Type OP_RETURN")
                .count(),
            1,
            "Type OP_RETURN was dropped or repeated: {text}"
        );
    }

    fn op_return_vout(extra: &str) -> String {
        const ASM: &str = "OP_RETURN 68656c6c6f";
        const SCRIPT: &str = "6a0468656c6c6f";
        format!(
            r#"{{"scriptpubkey":"{SCRIPT}","scriptpubkey_asm":"{ASM}","scriptpubkey_type":"op_return"{extra}}}"#
        )
    }

    let ordinary_vin = vin(
        r#""prevout":{"scriptpubkey":"0014aa","scriptpubkey_asm":"OP_0","scriptpubkey_type":"v0_p2wpkh","scriptpubkey_address":"bc1qin","value":8},"#,
    );
    let issued_vin = r#"{"txid":"issueprev","vout":0,"prevout":null,"issuance":{"asset_id":"issuanceasset","is_reissuance":false,"asset_entropy":"entropyvalue"},"scriptsig":"deadbeef","scriptsig_asm":"OP_CHECKSIG","witness":["abcdwitnesshex"],"is_coinbase":false,"sequence":1}"#;
    let op_return_vout_bare = op_return_vout(r#","value":0"#);
    let ordinary_vout = r#"{"scriptpubkey":"0014aa","scriptpubkey_asm":"OP_0","scriptpubkey_type":"v0_p2wpkh","scriptpubkey_address":"bc1qordinary","value":9}"#;
    let op_return_out = paint(
        "opretlabel",
        &format!("{issued_vin},{ordinary_vin}"),
        &format!("{op_return_vout_bare},{ordinary_vout}"),
    );
    assert_eq!(
        label_count(&op_return_out),
        1,
        "{LABEL} was not one exact line on the op_return output with no address: {op_return_out}"
    );
    let rows: Vec<&str> = op_return_out.lines().collect();
    let op_address = rows
        .iter()
        .position(|line| *line == "output address  value 0");
    let script_type = rows
        .iter()
        .position(|line| *line == "output script type op_return");
    let type_op_return = rows.iter().position(|line| *line == "Type OP_RETURN");
    let label_at = rows.iter().position(|line| *line == LABEL);
    let ordinary_at = rows
        .iter()
        .position(|line| *line == "output address bc1qordinary value 9");
    assert!(
        op_address.is_some()
            && script_type.is_some()
            && type_op_return.is_some()
            && label_at.is_some()
            && ordinary_at.is_some(),
        "the op_return output lines or the ordinary output were dropped: {op_return_out}"
    );
    assert!(
        op_address.unwrap() < script_type.unwrap()
            && script_type.unwrap() < type_op_return.unwrap()
            && type_op_return.unwrap() < label_at.unwrap()
            && label_at.unwrap() < ordinary_at.unwrap(),
        "{LABEL} was not on the op_return output after Type OP_RETURN and before the ordinary output: {op_return_out}"
    );
    assert!(
        op_return_out
            .lines()
            .any(|line| line == "output script type v0_p2wpkh"),
        "the ordinary output script type was dropped: {op_return_out}"
    );
    assert!(
        op_return_out.lines().any(|line| line == "Type V0_P2WPKH"),
        "the ordinary output type was dropped: {op_return_out}"
    );
    assert_eq!(
        op_return_out
            .lines()
            .filter(|line| *line == "Issuance")
            .count(),
        1,
        "Issuance was dropped or repeated: {op_return_out}"
    );
    assert!(
        op_return_out
            .lines()
            .any(|line| line == "issuance asset issuanceasset"),
        "the issuance asset line was dropped: {op_return_out}"
    );
    assert_eq!(
        op_return_out.lines().filter(|line| *line == "P2PK").count(),
        0,
        "an op_return output painted P2PK: {op_return_out}"
    );
    assert_eq!(
        op_return_out
            .lines()
            .filter(|line| *line == "Transaction fee")
            .count(),
        0,
        "an op_return output painted Transaction fee: {op_return_out}"
    );
    assert_op_return_type_lines(&op_return_out);
    assert_one_fee_row_and_one_fee_rate(&op_return_out);
    assert_screen_keeps_timestamp(&op_return_out, "opretlabel");

    let empty = paint(
        "opretempty",
        &ordinary_vin,
        &op_return_vout(r#","scriptpubkey_address":"","value":2"#),
    );
    assert_eq!(
        label_count(&empty),
        1,
        "an empty address did not paint {LABEL} once: {empty}"
    );
    assert!(
        empty.lines().any(|line| line == "output address  value 2"),
        "the empty address output row was dropped: {empty}"
    );
    assert_op_return_type_lines(&empty);
    assert_one_fee_row_and_one_fee_rate(&empty);
    assert_screen_keeps_timestamp(&empty, "opretempty");

    let null_addr = paint(
        "opretnulladdr",
        &ordinary_vin,
        &op_return_vout(r#","scriptpubkey_address":null,"value":4"#),
    );
    assert_eq!(
        label_count(&null_addr),
        1,
        "JSON null address did not paint {LABEL} once: {null_addr}"
    );
    assert!(
        null_addr
            .lines()
            .any(|line| line == "output address  value 4"),
        "the null address output row was dropped: {null_addr}"
    );
    assert_op_return_type_lines(&null_addr);
    assert_one_fee_row_and_one_fee_rate(&null_addr);

    let null_pegout = paint(
        "opretnullpeg",
        &ordinary_vin,
        &op_return_vout(r#","value":0,"pegout":null"#),
    );
    assert_eq!(
        label_count(&null_pegout),
        1,
        "JSON null pegout did not paint {LABEL} once: {null_pegout}"
    );
    assert!(
        !null_pegout.lines().any(|line| line == "Liquid Peg Out"),
        "JSON null pegout painted Liquid Peg Out: {null_pegout}"
    );
    assert_op_return_type_lines(&null_pegout);
    assert_one_fee_row_and_one_fee_rate(&null_pegout);

    let addressed = paint(
        "opretaddr",
        &ordinary_vin,
        &op_return_vout(r#","scriptpubkey_address":"bc1qopret","value":3"#),
    );
    assert_eq!(
        label_count(&addressed),
        0,
        "an op_return output with an address painted {LABEL}: {addressed}"
    );
    assert!(
        addressed
            .lines()
            .any(|line| line == "output address bc1qopret value 3"),
        "the addressed op_return output was dropped: {addressed}"
    );
    assert_op_return_type_lines(&addressed);
    assert_one_fee_row_and_one_fee_rate(&addressed);
    assert_screen_keeps_timestamp(&addressed, "opretaddr");

    let pegged = paint(
        "opretpeg",
        &ordinary_vin,
        &op_return_vout(
            r#","value":0,"pegout":{"genesis_hash":"aa","scriptpubkey":"bb","scriptpubkey_asm":"OP_TRUE"}"#,
        ),
    );
    assert_eq!(
        label_count(&pegged),
        0,
        "a peg-out painted {LABEL}: {pegged}"
    );
    assert!(
        pegged.lines().any(|line| line == "Liquid Peg Out"),
        "Liquid Peg Out was dropped: {pegged}"
    );
    assert_op_return_type_lines(&pegged);
    assert_one_fee_row_and_one_fee_rate(&pegged);
    assert_screen_keeps_timestamp(&pegged, "opretpeg");

    let op_return_prevout = vin(&format!(
        r#""prevout":{{"scriptpubkey":"{SCRIPT}","scriptpubkey_asm":"{ASM}","scriptpubkey_type":"op_return","value":5}},"#
    ));
    let input_op_return = paint("opretin", &op_return_prevout, ordinary_vout);
    assert_eq!(
        label_count(&input_op_return),
        0,
        "an input whose prevout type is op_return painted {LABEL}: {input_op_return}"
    );
    assert!(
        input_op_return
            .lines()
            .any(|line| line == "input script type op_return"),
        "input script type op_return was dropped: {input_op_return}"
    );
    assert!(
        input_op_return
            .lines()
            .any(|line| line == "Previous output type OP_RETURN"),
        "Previous output type OP_RETURN was dropped: {input_op_return}"
    );
    assert!(
        !input_op_return.lines().any(|line| line == "Type OP_RETURN"),
        "Type OP_RETURN was painted on the input path: {input_op_return}"
    );
    assert!(
        input_op_return
            .lines()
            .any(|line| line == "output address bc1qordinary value 9"),
        "the ordinary output was dropped beside an op_return input: {input_op_return}"
    );
    assert_one_fee_row_and_one_fee_rate(&input_op_return);
    assert_screen_keeps_timestamp(&input_op_return, "opretin");

    let fee_vout =
        r#"{"scriptpubkey":"","scriptpubkey_asm":"","scriptpubkey_type":"fee","value":0}"#;
    let fee_out = paint("opretfee", &ordinary_vin, fee_vout);
    assert_eq!(
        label_count(&fee_out),
        0,
        "a fee output painted {LABEL}: {fee_out}"
    );
    assert_eq!(
        fee_out
            .lines()
            .filter(|line| *line == "Transaction fee")
            .count(),
        1,
        "Transaction fee was dropped or repeated on the fee output: {fee_out}"
    );
    assert!(
        fee_out.lines().any(|line| line == "Type FEE"),
        "Type FEE was dropped: {fee_out}"
    );
    assert!(
        fee_out.lines().any(|line| line == "output script type fee"),
        "output script type fee was dropped: {fee_out}"
    );
    assert_one_fee_row_and_one_fee_rate(&fee_out);
    assert_screen_keeps_timestamp(&fee_out, "opretfee");

    let p2pk_vout = r#"{"scriptpubkey":"0014aa","scriptpubkey_asm":"OP_0","scriptpubkey_type":"p2pk","value":1}"#;
    let p2pk_out = paint("opretp2pk", &ordinary_vin, p2pk_vout);
    assert_eq!(
        label_count(&p2pk_out),
        0,
        "a p2pk output painted {LABEL}: {p2pk_out}"
    );
    assert_eq!(
        p2pk_out.lines().filter(|line| *line == "P2PK").count(),
        1,
        "P2PK was dropped or repeated on the p2pk output: {p2pk_out}"
    );
    assert!(
        p2pk_out.lines().any(|line| line == "Type P2PK"),
        "Type P2PK was dropped: {p2pk_out}"
    );
    assert!(
        p2pk_out
            .lines()
            .any(|line| line == "output script type p2pk"),
        "output script type p2pk was dropped: {p2pk_out}"
    );
    assert_one_fee_row_and_one_fee_rate(&p2pk_out);
    assert_screen_keeps_timestamp(&p2pk_out, "opretp2pk");

    let p2pkh = r#"{"scriptpubkey":"0014aa","scriptpubkey_asm":"OP_0","scriptpubkey_type":"p2pkh","value":6}"#;
    let other = paint("opretother", &ordinary_vin, p2pkh);
    assert_eq!(
        label_count(&other),
        0,
        "a p2pkh output painted {LABEL}: {other}"
    );
    assert!(
        other.lines().any(|line| line == "Type P2PKH"),
        "Type P2PKH was dropped: {other}"
    );
    assert!(
        !other.lines().any(|line| line == "P2PKH"),
        "a bare P2PKH line was invented: {other}"
    );
    assert!(
        other.lines().any(|line| line == "output script type p2pkh"),
        "output script type p2pkh was dropped: {other}"
    );
    assert_one_fee_row_and_one_fee_rate(&other);
    assert_screen_keeps_timestamp(&other, "opretother");
}

/// Runestone beside `OP_RETURN` from a no-query `GET /tx/:txid`.
///
/// An `op_return` output whose `scriptpubkey` hex starts with `6a5d` paints
/// the exact line `Runestone`. `OP_RETURN` and `Type OP_RETURN` stay. An
/// `op_return` output whose script does not start with `6a5d` does not paint
/// `Runestone`. A non-`op_return` output does not, even when its script hex
/// starts with `6a5d`. Script hex, script asm, scriptsig, and witness stay off.
#[test]
fn transaction_screen_paints_runestone() {
    const STONE_SCRIPT: &str = "6a5ddeadbeef";
    const PLAIN_SCRIPT: &str = "6a046869";
    const OTHER_SCRIPT: &str = "6a5dnotopreturn";
    const ASM: &str = "asmsentinel";
    const SIG: &str = "sigsentinel";
    const SIG_ASM: &str = "sigasmsentinel";
    const WIT: &str = "witsentinel";

    fn vout(script: &str, kind: &str) -> String {
        format!(
            r#"{{"scriptpubkey":"{script}","scriptpubkey_asm":"{ASM}","scriptpubkey_type":"{kind}","value":0}}"#
        )
    }

    fn tx_body(txid: &str, vout_json: &str) -> String {
        format!(
            r#"{{"txid":"{txid}","version":1,"locktime":0,"vin":[{{"txid":"prev","vout":0,"prevout":null,"scriptsig":"{SIG}","scriptsig_asm":"{SIG_ASM}","witness":["{WIT}"],"is_coinbase":false,"sequence":1}}],"vout":[{vout_json}],"size":10,"weight":40,"sigops":0,"fee":1,"status":{{"confirmed":false}}}}"#
        )
    }

    fn paint(txid: &str, vout_json: &str) -> String {
        let (mut host, rec) = host_with(
            Network::Mainnet,
            BTreeMap::from([(format!("/tx/{txid}"), tx_body(txid, vout_json))]),
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

    fn assert_script_stays_off(text: &str) {
        for needle in [
            STONE_SCRIPT,
            PLAIN_SCRIPT,
            OTHER_SCRIPT,
            ASM,
            SIG,
            SIG_ASM,
            WIT,
        ] {
            assert!(!text.contains(needle), "{needle} was painted: {text}");
        }
    }

    let stone = paint("runestone", &vout(STONE_SCRIPT, "op_return"));
    assert!(
        stone.contains("tx txid runestone"),
        "the parsed transaction was hidden: {stone}"
    );
    assert!(
        stone.lines().any(|line| line == "Runestone"),
        "Runestone is absent: {stone}"
    );
    assert_eq!(
        stone.lines().filter(|line| *line == "Runestone").count(),
        1,
        "Runestone was not painted once: {stone}"
    );
    assert_eq!(
        stone.lines().filter(|line| *line == "OP_RETURN").count(),
        1,
        "OP_RETURN was dropped or repeated: {stone}"
    );
    assert_eq!(
        stone
            .lines()
            .filter(|line| *line == "Type OP_RETURN")
            .count(),
        1,
        "Type OP_RETURN was dropped or repeated: {stone}"
    );
    assert!(
        stone
            .lines()
            .any(|line| line == "output script type op_return"),
        "output script type op_return was dropped: {stone}"
    );
    assert_script_stays_off(&stone);

    let plain = paint("runestoneplain", &vout(PLAIN_SCRIPT, "op_return"));
    assert!(
        plain.contains("tx txid runestoneplain"),
        "the parsed transaction was hidden: {plain}"
    );
    assert!(
        !plain.lines().any(|line| line.contains("Runestone")),
        "an OP_RETURN that does not start with 6a5d painted Runestone: {plain}"
    );
    assert_eq!(
        plain.lines().filter(|line| *line == "OP_RETURN").count(),
        1,
        "OP_RETURN was dropped: {plain}"
    );
    assert!(
        plain.lines().any(|line| line == "Type OP_RETURN"),
        "Type OP_RETURN was dropped: {plain}"
    );
    assert_script_stays_off(&plain);

    let other = paint("runestoneother", &vout(OTHER_SCRIPT, "v0_p2wpkh"));
    assert!(
        other.contains("tx txid runestoneother"),
        "the parsed transaction was hidden: {other}"
    );
    assert!(
        !other.lines().any(|line| line.contains("Runestone")),
        "a non-OP_RETURN output painted Runestone: {other}"
    );
    assert!(
        other.lines().any(|line| line == "Type V0_P2WPKH"),
        "Type V0_P2WPKH was dropped: {other}"
    );
    assert!(
        !other.lines().any(|line| line == "OP_RETURN"),
        "a non-OP_RETURN output painted OP_RETURN: {other}"
    );
    assert_script_stays_off(&other);
}

/// Transaction screen from `GET /tx/:txid`, field `status.block_time`.
///
/// One exact line `Timestamp` sits immediately before the existing raw
/// `block_time` line. The integer on that line stays unformatted. Explicit 0
/// still paints the label and `block_time 0`. A missing key, JSON null, or a
/// missing status omits the label and does not invent 0. The blocks list and
/// the block page do not gain this line. The path has no query and no fragment.
#[test]
fn transaction_screen_paints_timestamp_label() {
    const LABEL: &str = "Timestamp";

    fn tx_body(txid: &str, status: &str) -> String {
        let status_field = if status.is_empty() {
            String::new()
        } else {
            format!(r#","status":{status}"#)
        };
        format!(
            r#"{{"txid":"{txid}","version":1,"locktime":0,"vin":[{{"txid":"prev","vout":0,"prevout":null,"scriptsig":"deadbeef","scriptsig_asm":"OP_CHECKSIG","witness":["abcdwitnesshex"],"is_coinbase":false,"sequence":1}}],"vout":[{{"scriptpubkey":"0014aa","scriptpubkey_asm":"OP_RETURN","scriptpubkey_type":"op_return","value":9}}],"size":10,"weight":400,"sigops":0,"fee":50{status_field}}}"#
        )
    }

    fn paint_tx(txid: &str, status: &str) -> (String, Vec<String>) {
        let (mut host, rec) = host_with(
            Network::Mainnet,
            BTreeMap::from([(format!("/tx/{txid}"), tx_body(txid, status))]),
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
        (painted(&host), paths)
    }

    fn assert_label_before_raw(painted: &str, txid: &str, time: u64) {
        let lines: Vec<&str> = painted.lines().collect();
        let marker = format!("block_time {time} ");
        let before = lines.windows(2).any(|pair| {
            pair[0] == LABEL
                && pair[1].contains(&format!("tx txid {txid} "))
                && pair[1].contains(&marker)
                && !pair[1].contains("Timestamp")
                && !pair[1].contains(',')
        });
        assert!(
            before,
            "{LABEL} was not immediately before the raw {marker} line: {painted}"
        );
        assert_eq!(
            painted.lines().filter(|line| *line == LABEL).count(),
            1,
            "{LABEL} was not one exact line: {painted}"
        );
        assert!(
            !painted.contains("1,600,000,000"),
            "the block time gained a thousands separator: {painted}"
        );
        assert!(
            !painted.contains("2020") && !painted.contains("ago"),
            "the block time was formatted: {painted}"
        );
        assert!(
            painted.lines().any(|line| line == "Fee rate 0.50 sat/vB"),
            "Fee rate was dropped: {painted}"
        );
        assert!(
            painted.contains("fee_rate 0.5"),
            "the raw fee_rate header was dropped: {painted}"
        );
        assert!(
            painted.lines().any(|line| line == "Confirmed"),
            "Confirmed was dropped or the letter case changed: {painted}"
        );
        assert!(
            painted.contains("confirmed true"),
            "the raw confirmed header changed letter case: {painted}"
        );
        assert!(
            painted.lines().any(|line| line == "Inputs & Outputs"),
            "Inputs & Outputs was dropped: {painted}"
        );
        assert!(!painted.contains("abcdwitnesshex"), "{painted}");
        assert_script_hex_stays_off(painted);
        assert_paint_is_not_html(painted);
    }

    fn assert_label_omitted(painted: &str, txid: &str) {
        assert!(
            painted.contains(&format!("tx txid {txid} ")),
            "the transaction was hidden: {painted}"
        );
        assert!(
            !painted.lines().any(|line| line == LABEL),
            "absent block_time painted {LABEL}: {painted}"
        );
        assert!(
            !painted.contains("block_time 0"),
            "absent block_time invented 0: {painted}"
        );
        assert!(
            painted.lines().any(|line| line == "Fee rate 0.50 sat/vB"),
            "Fee rate was dropped: {painted}"
        );
        assert!(!painted.contains("abcdwitnesshex"), "{painted}");
        assert_script_hex_stays_off(painted);
        assert_paint_is_not_html(painted);
    }

    let present = paint_tx(
        "timepresent",
        r#"{"confirmed":true,"block_height":4,"block_hash":"tiphash1","block_time":1600000000}"#,
    );
    assert_label_before_raw(&present.0, "timepresent", 1_600_000_000);

    let zero = paint_tx(
        "timezero",
        r#"{"confirmed":true,"block_height":4,"block_hash":"tiphash1","block_time":0}"#,
    );
    assert_label_before_raw(&zero.0, "timezero", 0);

    let missing = paint_tx(
        "timemiss",
        r#"{"confirmed":true,"block_height":4,"block_hash":"tiphash1"}"#,
    );
    assert_label_omitted(&missing.0, "timemiss");

    let null = paint_tx(
        "timenull",
        r#"{"confirmed":true,"block_height":4,"block_hash":"tiphash1","block_time":null}"#,
    );
    assert_label_omitted(&null.0, "timenull");

    let no_status = paint_tx("timenostatus", "");
    assert_label_omitted(&no_status.0, "timenostatus");

    let list_id = "timeblocklist";
    let (mut host, rec) = host_with(
        Network::Mainnet,
        BTreeMap::from([("/blocks".to_string(), format!("[{}]", block_json(list_id)))]),
    );
    host.activate(Screen::Blocks)
        .unwrap_or_else(|err| panic!("blocks list did not open: {err}"));
    let paths = rec.gets();
    assert!(
        paths.iter().any(|path| path == "/blocks"),
        "list did not GET /blocks: {paths:?}"
    );
    assert!(
        paths
            .iter()
            .all(|path| !path.contains('?') && !path.contains('#')),
        "{paths:?}"
    );
    let list = painted(&host);
    assert!(
        !list.lines().any(|line| line == LABEL),
        "the blocks list painted {LABEL}: {list}"
    );
    assert!(
        list.lines().any(|line| line == "TXs"),
        "TXs was dropped from the blocks list: {list}"
    );
    assert!(
        list.contains("timestamp 1700000000"),
        "the blocks list dropped its raw timestamp: {list}"
    );
    assert!(
        list.contains("difficulty 1") && !list.contains("difficulty 1.0"),
        "blocks list difficulty left plain_number: {list}"
    );

    let page_id = "timeblockpage";
    let (mut host, rec) = host_with(
        Network::Mainnet,
        BTreeMap::from([
            (format!("/block/{page_id}"), block_json(page_id)),
            (format!("/block/{page_id}/txs"), "[]".to_string()),
            (format!("/block/{page_id}/txids"), "[]".to_string()),
        ]),
    );
    host.block.query = page_id.to_string();
    host.activate(Screen::Block)
        .unwrap_or_else(|err| panic!("block {page_id} did not open: {err}"));
    let paths = rec.gets();
    assert!(
        paths
            .iter()
            .any(|path| path == &format!("/block/{page_id}")),
        "load did not GET /block/{page_id}: {paths:?}"
    );
    assert!(
        paths
            .iter()
            .all(|path| !path.contains('?') && !path.contains('#')),
        "{paths:?}"
    );
    let closed = painted(&host);
    assert!(
        !closed.lines().any(|line| line == LABEL),
        "the block page painted {LABEL}: {closed}"
    );
    assert!(
        closed.lines().any(|line| line == "TXs"),
        "TXs was dropped from the block page: {closed}"
    );
    host.open_row("row block details")
        .unwrap_or_else(|err| panic!("block details did not open: {err}"));
    let opened = painted(&host);
    assert!(
        !opened.lines().any(|line| line == LABEL),
        "block details painted {LABEL}: {opened}"
    );
    assert!(
        opened.lines().any(|line| line == "Merkle root mm"),
        "Merkle root was dropped: {opened}"
    );
    assert!(
        opened.lines().any(|line| line == "Difficulty 1"),
        "Difficulty was dropped: {opened}"
    );
    assert!(
        !opened
            .lines()
            .any(|line| line == "Difficulty 1.0" || line == "difficulty 1.0"),
        "plain_number painted 1.0: {opened}"
    );
}

fn paint_tx_screen_weight(txid: &str, weight_field: &str, sigops_field: &str) -> String {
    let weight_line = if weight_field.is_empty() {
        String::new()
    } else {
        format!("{weight_field},")
    };
    let sigops_line = if sigops_field.is_empty() {
        String::new()
    } else {
        format!("{sigops_field},")
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
        {sigops_line}
        "fee":1,
        "status":{{"confirmed":false}}
    }}"#
    );
    let (mut host, _) = host_with(
        Network::Mainnet,
        BTreeMap::from([(format!("/tx/{txid}"), tx)]),
    );
    host.open_transaction(txid)
        .unwrap_or_else(|err| panic!("tx {txid} did not open: {err}"));
    painted(&host)
}

/// Transaction weight from `GET /tx/:txid`, scaled like the Angular `wuBytes` pipe.
///
/// The unit is chosen from the raw weight before rounding. Thresholds are
/// powers of 1000, not 1024. The scaled value is rounded to 2 decimal places,
/// half away from zero, then printed the way JavaScript `Number.toString` does.
/// Units are `WU`, `kWU`, `MWU`, `GWU`, and `TWU`. A missing key or JSON null
/// omits the row. An explicit zero paints. The raw header `weight N` stays.
/// This is not the integer virtual-size line and not adjusted vsize.
#[test]
fn transaction_screen_paints_weight() {
    let fifteen = paint_tx_screen_weight("wu1500", r#""weight":1500"#, "");
    assert!(
        fifteen.lines().any(|line| line == "Weight 1.5 kWU"),
        "weight 1500 did not paint the line Weight 1.5 kWU: {fifteen}"
    );
    assert!(
        fifteen.contains("weight 1500"),
        "weight 1500 dropped the raw header: {fifteen}"
    );
    assert!(
        !fifteen.lines().any(|line| line == "Weight 1 kWU"),
        "weight 1500 painted Weight 1 kWU: {fifteen}"
    );
    assert_script_hex_stays_off(&fifteen);

    let seven = paint_tx_screen_weight("wu7", r#""weight":7"#, r#""sigops":0"#);
    assert!(
        seven.lines().any(|line| line == "Weight 7 WU"),
        "weight 7 did not paint the line Weight 7 WU: {seven}"
    );
    assert!(
        seven.lines().any(|line| line == "virtual size 1"),
        "weight 7 dropped virtual size 1: {seven}"
    );
    assert!(
        seven.lines().any(|line| line == "Adjusted vsize 1.75"),
        "weight 7 dropped Adjusted vsize 1.75: {seven}"
    );
    assert_script_hex_stays_off(&seven);

    let zero = paint_tx_screen_weight("wu0", r#""weight":0"#, "");
    assert!(
        zero.lines().any(|line| line == "Weight 0 WU"),
        "weight 0 omitted the line Weight 0 WU: {zero}"
    );
    assert!(
        zero.contains("weight 0"),
        "explicit weight 0 dropped the raw header: {zero}"
    );
    assert!(
        zero.lines().any(|line| line == "virtual size 0"),
        "weight 0 dropped virtual size 0: {zero}"
    );

    assert!(
        paint_tx_screen_weight("wu1000", r#""weight":1000"#, "")
            .lines()
            .any(|line| line == "Weight 1 kWU"),
        "weight 1000 did not paint the line Weight 1 kWU"
    );

    let kib = paint_tx_screen_weight("wu1024", r#""weight":1024"#, "");
    assert!(
        kib.lines().any(|line| line == "Weight 1.02 kWU"),
        "weight 1024 did not paint the line Weight 1.02 kWU: {kib}"
    );
    assert!(
        !kib.lines().any(|line| line == "Weight 1024 WU"),
        "weight 1024 stayed in WU: {kib}"
    );

    let almost = paint_tx_screen_weight("wu999999", r#""weight":999999"#, "");
    assert!(
        almost.lines().any(|line| line == "Weight 1000 kWU"),
        "weight 999999 did not paint the line Weight 1000 kWU: {almost}"
    );
    assert!(
        !almost.lines().any(|line| line == "Weight 1 MWU"),
        "weight 999999 promoted to MWU: {almost}"
    );

    let mega = paint_tx_screen_weight("wu1000000", r#""weight":1000000"#, "");
    assert!(
        mega.lines().any(|line| line == "Weight 1 MWU"),
        "weight 1000000 did not paint the line Weight 1 MWU: {mega}"
    );
    assert!(
        !mega.lines().any(|line| line == "Weight 1000 kWU"),
        "weight 1000000 stayed in kWU: {mega}"
    );

    let absent = paint_tx_screen_weight("wuabs", "", "");
    assert!(
        absent.contains("tx txid wuabs"),
        "missing weight hid the transaction: {absent}"
    );
    assert!(
        !absent.contains("Weight"),
        "missing weight painted Weight: {absent}"
    );
    assert!(
        !absent.contains("weight 0"),
        "missing weight was painted as weight 0: {absent}"
    );
    assert!(
        !absent.contains("virtual size"),
        "missing weight painted virtual size: {absent}"
    );

    let null = paint_tx_screen_weight("wunull", r#""weight":null"#, "");
    assert!(
        null.contains("tx txid wunull"),
        "null weight hid the transaction: {null}"
    );
    assert!(
        !null.contains("Weight"),
        "null weight painted Weight: {null}"
    );
    assert!(
        !null.contains("weight 0"),
        "null weight was painted as weight 0: {null}"
    );
    assert!(
        !null.contains("virtual size"),
        "null weight painted virtual size: {null}"
    );
}

fn paint_block_details(id: &str) -> String {
    let (mut host, rec) = host_with(
        Network::Mainnet,
        BTreeMap::from([
            (format!("/block/{id}"), block_json(id)),
            (format!("/block/{id}/txs"), "[]".to_string()),
            (format!("/block/{id}/txids"), "[]".to_string()),
        ]),
    );
    host.block.query = id.to_string();
    host.activate(Screen::Block)
        .unwrap_or_else(|err| panic!("block {id} did not open: {err}"));
    host.open_row("row block details")
        .unwrap_or_else(|err| panic!("block details did not open: {err}"));
    let paths = rec.gets();
    assert!(
        paths
            .iter()
            .all(|path| !path.contains('?') && !path.contains('#')),
        "{paths:?}"
    );
    painted(&host)
}

fn assert_hash_line_has_no_mark(painted: &str, expected: &str) {
    let line = painted
        .lines()
        .find(|line| *line == expected)
        .unwrap_or_else(|| panic!("missing line {expected} in {painted}"));
    assert!(
        !line.contains('\u{2026}'),
        "Hash line used a unicode ellipsis: {line}"
    );
    assert!(!line.contains('\u{200e}'), "Hash line used U+200E: {line}");
}

/// A failed no-query block load paints Angular's span above the indexer body.
/// The span is `Error loading block data.` The italic line stays `Block not found`.
#[test]
fn block_screen_paints_load_error() {
    let id = "nope";
    let (mut host, rec) = host_with(
        Network::Mainnet,
        BTreeMap::from([(format!("/block/{id}"), "Block not found".to_string())]),
    );
    let err = host.open_block(id).expect_err("missing block must fail");
    assert!(
        err.to_string().contains("Block not found"),
        "indexer body left the error: {err}"
    );
    let paths = rec.gets();
    assert!(
        paths.iter().any(|path| path == "/block/nope"),
        "block load did not GET /block/nope: {paths:?}"
    );
    assert!(
        paths
            .iter()
            .all(|path| !path.contains('?') && !path.contains('#')),
        "block load used a query string: {paths:?}"
    );
    let painted = painted(&host);
    assert!(
        painted.lines().any(|line| line == "Block not found"),
        "Block not found was replaced or dropped: {painted}"
    );
    assert!(
        painted
            .lines()
            .any(|line| line == "Error loading block data."),
        "Error loading block data. is absent: {painted}"
    );
    let span_at = painted.find("Error loading block data.").expect("span");
    let body_at = painted.find("Block not found").expect("indexer body");
    assert!(
        span_at < body_at,
        "the span did not sit above Block not found: {painted}"
    );
    assert_eq!(
        painted.matches("Error loading block data.").count(),
        1,
        "the span was not painted once: {painted}"
    );
    assert_eq!(host.notice, "Block not found");
}

/// A failed no-query `GET /address/:addr` paints Angular's span above the indexer body.
/// The span is `Error loading address data.` The body stays `Invalid Bitcoin address`.
/// A parsed address that returns stats does not show the span, and `Type P2WPKH` stays.
#[test]
fn address_screen_paints_load_error() {
    let address = "notanaddress";
    let (mut host, rec) = host_with(
        Network::Mainnet,
        BTreeMap::from([(
            format!("/address/{address}"),
            "Invalid Bitcoin address".to_string(),
        )]),
    );
    let err = host
        .open_address(address)
        .expect_err("invalid address must fail");
    let paths = rec.gets();
    assert!(
        paths.iter().any(|path| path == "/address/notanaddress"),
        "address load did not GET /address/notanaddress: {paths:?}"
    );
    assert!(
        paths
            .iter()
            .all(|path| !path.contains('?') && !path.contains('#')),
        "address load used a query string: {paths:?}"
    );
    let painted = painted(&host);
    assert!(
        painted
            .lines()
            .any(|line| line == "Error loading address data."),
        "Error loading address data. is absent: {painted}"
    );
    assert!(
        err.to_string().contains("Invalid Bitcoin address"),
        "indexer body left the error: {err}"
    );
    assert!(
        painted
            .lines()
            .any(|line| line == "Invalid Bitcoin address"),
        "Invalid Bitcoin address was replaced or dropped: {painted}"
    );
    let span_at = painted.find("Error loading address data.").expect("span");
    let body_at = painted
        .find("Invalid Bitcoin address")
        .expect("indexer body");
    assert!(
        span_at < body_at,
        "the span did not sit above Invalid Bitcoin address: {painted}"
    );
    assert_eq!(
        painted.matches("Error loading address data.").count(),
        1,
        "the span was not painted once: {painted}"
    );
    assert!(
        !painted.contains("400 Bad Request"),
        "a 400 Bad Request prefix was added: {painted}"
    );
    assert!(
        !painted.contains("Error loading block data."),
        "the block sentence replaced the address span: {painted}"
    );
    assert_eq!(host.notice, "Invalid Bitcoin address");

    let good = "bc1q8yj0herd4r4yxszw3nkfvt53433thk0f5qst4g";
    let good_painted = paint_address(good, &address_stats(good));
    assert!(
        !good_painted.contains("Error loading address data."),
        "a parsed address that returns stats showed the load error: {good_painted}"
    );
    assert!(
        good_painted.lines().any(|line| line == "Type P2WPKH"),
        "Type P2WPKH left a successful load: {good_painted}"
    );
}

/// Block Hash from `GET /block/:hash`, shortened like Angular `shortenString` of length 13.
///
/// Prefix length is integer `length / 2` (6). Suffix length is `length - prefix` (7).
/// The joiner is three ASCII dots. An id of length 13 or less stays whole.
/// The raw `id` line and the `block hash` summary stay the full id.
#[test]
fn block_screen_paints_hash() {
    let long = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";
    assert_eq!(long.len(), 64);
    let painted = paint_block_details(long);
    let id_line = format!("id {long}");
    assert!(
        painted.lines().any(|line| line == "Hash 012345...9abcdef"),
        "Hash 012345...9abcdef is absent; id line present={}: {painted}",
        painted.lines().any(|line| line == id_line)
    );
    assert!(
        !painted.contains("012345...abcdef"),
        "6-and-6 split leaked into the painted text: {painted}"
    );
    assert!(
        painted.lines().any(|line| line == id_line),
        "64-char id dropped the line {id_line}: {painted}"
    );
    assert!(
        painted.contains(&format!("block hash {long}")),
        "summary dropped the full id on block hash: {painted}"
    );
    assert_hash_line_has_no_mark(&painted, "Hash 012345...9abcdef");

    let fourteen = "abcdefghijklmn";
    assert_eq!(fourteen.len(), 14);
    let painted = paint_block_details(fourteen);
    assert!(
        painted.lines().any(|line| line == "Hash abcdef...hijklmn"),
        "14-char id did not paint the line Hash abcdef...hijklmn: {painted}"
    );
    assert_hash_line_has_no_mark(&painted, "Hash abcdef...hijklmn");

    let thirteen = "abcdefghijklm";
    assert_eq!(thirteen.len(), 13);
    let painted = paint_block_details(thirteen);
    assert!(
        painted.lines().any(|line| line == "Hash abcdefghijklm"),
        "13-char id did not paint the line Hash abcdefghijklm: {painted}"
    );
    let thirteen_line = painted
        .lines()
        .find(|line| *line == "Hash abcdefghijklm")
        .expect("13-char Hash line");
    assert!(
        !thirteen_line.contains("..."),
        "13-char Hash line was shortened: {thirteen_line}"
    );
    assert_hash_line_has_no_mark(&painted, "Hash abcdefghijklm");

    let twelve = "abcdefghijkl";
    assert_eq!(twelve.len(), 12);
    let painted = paint_block_details(twelve);
    assert!(
        painted.lines().any(|line| line == "Hash abcdefghijkl"),
        "12-char id did not paint the line Hash abcdefghijkl: {painted}"
    );
    let twelve_line = painted
        .lines()
        .find(|line| *line == "Hash abcdefghijkl")
        .expect("12-char Hash line");
    assert!(
        !twelve_line.contains("..."),
        "12-char Hash line was shortened: {twelve_line}"
    );
    assert_hash_line_has_no_mark(&painted, "Hash abcdefghijkl");

    // A missing id fails serde before block_details_text. txs and txids are
    // registered so a later read cannot hide that parse miss.
    let missing = "missingidblock";
    let missing_body = r#"{"height":800000,"version":1,"timestamp":1700000000,"tx_count":1,"size":20,"weight":80,"merkle_root":"mm","mediantime":1700000000}"#;
    let (mut host, rec) = host_with(
        Network::Mainnet,
        BTreeMap::from([
            (format!("/block/{missing}"), missing_body.to_string()),
            (format!("/block/{missing}/txs"), "[]".to_string()),
            (format!("/block/{missing}/txids"), "[]".to_string()),
        ]),
    );
    host.block.query = missing.to_string();
    let err = host
        .activate(Screen::Block)
        .expect_err("missing id must fail the block parse");
    let message = err.to_string();
    assert!(
        message.contains("missing field `id`"),
        "missing id was not a parse error: {message}"
    );
    let paths = rec.gets();
    assert!(
        paths
            .iter()
            .all(|path| !path.contains('?') && !path.contains('#')),
        "{paths:?}"
    );
    let popup = NpubPopupModel::new("npub1restofindexer").expect("npub");
    let missing_painted = painted_shell_text(&host, &popup);
    assert!(
        !missing_painted.contains("Hash"),
        "missing id painted Hash: {missing_painted}"
    );
}

fn block_version_json(id: &str, version: u32) -> String {
    format!(
        r#"{{"id":"{id}","height":2,"version":{version},"timestamp":1700000000,"tx_count":1,"size":100,"weight":400,"merkle_root":"merkle-root","mediantime":1699990000,"bits":2,"nonce":9,"difficulty":1}}"#
    )
}

/// Open block details for one version fixture. The id is not all digits, so
/// load uses `GET /block/:id` and not the height route.
fn paint_block_version(id: &str, version: u32) -> String {
    assert!(
        !id.is_empty() && !id.bytes().all(|byte| byte.is_ascii_digit()),
        "id must not be all digits: {id}"
    );
    assert!(!id.contains('?') && !id.contains('#'), "{id}");
    let (mut host, rec) = host_with(
        Network::Mainnet,
        BTreeMap::from([
            (format!("/block/{id}"), block_version_json(id, version)),
            (format!("/block/{id}/txs"), "[]".to_string()),
            (format!("/block/{id}/txids"), "[]".to_string()),
        ]),
    );
    host.block.query = id.to_string();
    host.activate(Screen::Block)
        .unwrap_or_else(|err| panic!("block {id} did not open: {err}"));
    host.open_row("row block details")
        .unwrap_or_else(|err| panic!("block details did not open: {err}"));
    let paths = rec.gets();
    assert!(
        paths
            .iter()
            .all(|path| !path.contains('?') && !path.contains('#')),
        "{paths:?}"
    );
    painted(&host)
}

fn assert_bits_stay_decimal(painted: &str) {
    assert!(
        painted.lines().any(|line| line == "bits 2 0x00000002"),
        "bits 2 0x00000002 was not a details line: {painted}"
    );
    assert!(
        painted
            .lines()
            .any(|line| line.split_whitespace().any(|word| word == "0x00000002")),
        "0x00000002 was not the whole bits hex: {painted}"
    );
}

fn assert_summary_version_stays_decimal(painted: &str, version: &str) {
    let summary = painted
        .lines()
        .find(|line| line.starts_with("block hash "))
        .unwrap_or_else(|| panic!("summary line missing: {painted}"));
    assert!(
        summary.contains(&format!(" version {version} ")),
        "summary dropped the decimal version: {summary}"
    );
    assert!(
        !summary.contains("0x"),
        "summary line painted version hex: {summary}"
    );
}

/// Block version hex beside the decimal on the details line.
///
/// Angular `decimal2hex` defaults to 4 bytes: `0x` plus exactly 8 lowercase
/// hex digits. The summary line stays the decimal version only. Bits details
/// are the decimal beside eight hex digits. Nonce and difficulty stay decimal.
/// A missing version key fails serde before the
/// details text, and that paint has no `0x`.
#[test]
fn block_screen_paints_version_hex() {
    let segwit = paint_block_version("verhex536", 536870912);
    let decimal_line = segwit.lines().any(|line| line == "version 536870912");
    assert!(
        segwit
            .lines()
            .any(|line| line == "version 536870912 0x20000000"),
        "version 536870912 0x20000000 is absent while the decimal version line is present ({decimal_line}): {segwit}"
    );
    assert_summary_version_stays_decimal(&segwit, "536870912");
    assert!(
        !segwit.contains("0X"),
        "version hex was uppercase: {segwit}"
    );
    assert_bits_stay_decimal(&segwit);

    let zero = paint_block_version("verhex0", 0);
    assert!(
        zero.lines().any(|line| line == "version 0 0x00000000"),
        "version 0 did not paint version 0 0x00000000: {zero}"
    );
    assert_summary_version_stays_decimal(&zero, "0");
    assert_bits_stay_decimal(&zero);

    let one = paint_block_version("verhex1", 1);
    assert!(
        one.lines().any(|line| line == "version 1 0x00000001"),
        "version 1 did not paint version 1 0x00000001: {one}"
    );
    assert!(
        !one.lines()
            .any(|line| line.split_whitespace().any(|word| word == "0x1")),
        "version 1 painted 0x1: {one}"
    );
    assert_summary_version_stays_decimal(&one, "1");
    assert_bits_stay_decimal(&one);
    assert!(
        one.lines().any(|line| line == "nonce 9 0x00000009"),
        "nonce 9 0x00000009 was not a details line: {one}"
    );
    assert!(
        one.lines().any(|line| line == "difficulty 1"),
        "difficulty 1 was not a details line: {one}"
    );

    let missing = "verhexmiss";
    assert!(
        !missing.bytes().all(|byte| byte.is_ascii_digit()),
        "{missing}"
    );
    let missing_body = format!(
        r#"{{"id":"{missing}","height":2,"timestamp":1700000000,"tx_count":1,"size":100,"weight":400,"merkle_root":"merkle-root","mediantime":1699990000}}"#
    );
    let (mut host, rec) = host_with(
        Network::Mainnet,
        BTreeMap::from([
            (format!("/block/{missing}"), missing_body),
            (format!("/block/{missing}/txs"), "[]".to_string()),
            (format!("/block/{missing}/txids"), "[]".to_string()),
        ]),
    );
    host.block.query = missing.to_string();
    let err = host
        .activate(Screen::Block)
        .expect_err("missing version must fail the block parse");
    let message = err.to_string();
    assert!(
        message.contains("missing field `version`"),
        "missing version was not a parse error: {message}"
    );
    let paths = rec.gets();
    assert!(
        paths
            .iter()
            .all(|path| !path.contains('?') && !path.contains('#')),
        "{paths:?}"
    );
    let missing_painted = painted(&host);
    assert!(
        !missing_painted.contains("0x"),
        "missing version painted hex: {missing_painted}"
    );
}

fn block_bits_json(id: &str, bits: &str) -> String {
    format!(
        r#"{{"id":"{id}","height":2,"version":1,"timestamp":1700000000,"tx_count":1,"size":100,"weight":400,"merkle_root":"merkle-root","previousblockhash":null,"mediantime":1699990000,"nonce":9,"bits":{bits},"difficulty":1}}"#
    )
}

fn missing_bits_json(id: &str) -> String {
    format!(
        r#"{{"id":"{id}","height":2,"version":1,"timestamp":1700000000,"tx_count":1,"size":100,"weight":400,"merkle_root":"merkle-root","previousblockhash":null,"mediantime":1699990000,"nonce":9,"difficulty":1}}"#
    )
}

/// Open block details for one bits fixture. The id is not all digits, so
/// load uses `GET /block/:id` and not the height route. Version is 1 so the
/// version hex is `0x00000001` and `0x00000000` is not already on that line.
fn paint_block_bits(id: &str, body: String) -> String {
    assert!(
        !id.is_empty() && !id.bytes().all(|byte| byte.is_ascii_digit()),
        "id must not be all digits: {id}"
    );
    assert!(!id.contains('?') && !id.contains('#'), "{id}");
    let (mut host, rec) = host_with(
        Network::Mainnet,
        BTreeMap::from([
            (format!("/block/{id}"), body),
            (format!("/block/{id}/txs"), "[]".to_string()),
            (format!("/block/{id}/txids"), "[]".to_string()),
        ]),
    );
    host.block.query = id.to_string();
    host.activate(Screen::Block)
        .unwrap_or_else(|err| panic!("block {id} did not open: {err}"));
    host.open_row("row block details")
        .unwrap_or_else(|err| panic!("block details did not open: {err}"));
    let paths = rec.gets();
    assert!(
        paths
            .iter()
            .all(|path| !path.contains('?') && !path.contains('#')),
        "{paths:?}"
    );
    painted(&host)
}

fn assert_summary_bits_stays_decimal(painted: &str, bits: &str) {
    let summary = painted
        .lines()
        .find(|line| line.starts_with("block hash "))
        .unwrap_or_else(|| panic!("summary line missing: {painted}"));
    assert!(
        summary.contains(&format!(" bits {bits} ")),
        "summary dropped the decimal bits: {summary}"
    );
    assert!(
        !summary.contains("0x"),
        "summary line painted bits hex: {summary}"
    );
}

fn assert_bits_omitted(painted: &str) {
    assert!(
        painted.lines().all(|line| !line.starts_with("bits")),
        "a line starts with bits: {painted}"
    );
    assert!(
        !painted.contains("bits 0"),
        "absent bits painted bits 0: {painted}"
    );
    assert!(
        !painted.contains("bits 0 0x00000000"),
        "absent bits painted bits 0 0x00000000: {painted}"
    );
    assert!(
        !painted.contains("0x00000000"),
        "absent bits painted 0x00000000: {painted}"
    );
    assert!(
        painted.lines().any(|line| line == "version 1 0x00000001"),
        "version 1 fixture did not paint version 1 0x00000001: {painted}"
    );
}

/// Block bits hex beside the decimal on the details line.
///
/// Angular `decimal2hex` defaults to 4 bytes: `0x` plus exactly 8 lowercase
/// hex digits. The summary line stays the decimal bits only. Nonce stays
/// decimal. Difficulty stays `plain_number`. A missing bits key and JSON
/// null are both `None` and omit the bits line. They do not substitute zero.
#[test]
fn block_screen_paints_bits_hex() {
    let wide = paint_block_bits("bitshex386", block_bits_json("bitshex386", "386547056"));
    let decimal_line = wide.lines().any(|line| line == "bits 386547056");
    assert!(
        wide.lines().any(|line| line == "bits 386547056 0x170a3d70"),
        "bits 386547056 0x170a3d70 is absent while the decimal bits line is present ({decimal_line}): {wide}"
    );
    assert!(
        !wide.lines().any(|line| line == "bits 386547056"),
        "bare decimal bits line remained beside the hex line: {wide}"
    );
    assert!(
        wide.lines()
            .any(|line| line.split_whitespace().any(|word| word == "0x170a3d70")),
        "0x170a3d70 was not the whole hex word: {wide}"
    );
    assert!(!wide.contains("0X"), "bits hex was uppercase: {wide}");
    assert_summary_bits_stays_decimal(&wide, "386547056");
    assert!(
        wide.lines().any(|line| line == "nonce 9 0x00000009"),
        "nonce 9 0x00000009 was not a details line: {wide}"
    );
    assert!(
        wide.lines().any(|line| line == "difficulty 1"),
        "difficulty 1 was not a details line: {wide}"
    );

    let zero = paint_block_bits("bitshex0", block_bits_json("bitshex0", "0"));
    assert!(
        zero.lines().any(|line| line == "bits 0 0x00000000"),
        "bits 0 did not paint bits 0 0x00000000: {zero}"
    );
    assert!(
        zero.lines()
            .any(|line| line.split_whitespace().any(|word| word == "0x00000000")),
        "0x00000000 was not the whole hex word beside bits 0: {zero}"
    );
    assert_summary_bits_stays_decimal(&zero, "0");
    assert!(
        zero.lines().any(|line| line == "version 1 0x00000001"),
        "bits 0 fixture did not keep version 1 0x00000001: {zero}"
    );

    let one = paint_block_bits("bitshex1", block_bits_json("bitshex1", "1"));
    assert!(
        one.lines().any(|line| line == "bits 1 0x00000001"),
        "bits 1 did not paint bits 1 0x00000001: {one}"
    );
    assert!(
        !one.lines()
            .any(|line| line.split_whitespace().any(|word| word == "0x1")),
        "bits 1 painted 0x1: {one}"
    );
    assert_summary_bits_stays_decimal(&one, "1");

    let missing = paint_block_bits("bitshexmiss", missing_bits_json("bitshexmiss"));
    assert_bits_omitted(&missing);

    let null = paint_block_bits("bitshexnull", block_bits_json("bitshexnull", "null"));
    assert_bits_omitted(&null);
}

fn block_nonce_json(id: &str, nonce: &str) -> String {
    format!(
        r#"{{"id":"{id}","height":2,"version":1,"timestamp":1700000000,"tx_count":1,"size":100,"weight":400,"merkle_root":"merkle-root","previousblockhash":null,"mediantime":1699990000,"nonce":{nonce},"bits":386547056,"difficulty":1}}"#
    )
}

fn missing_nonce_json(id: &str) -> String {
    format!(
        r#"{{"id":"{id}","height":2,"version":1,"timestamp":1700000000,"tx_count":1,"size":100,"weight":400,"merkle_root":"merkle-root","previousblockhash":null,"mediantime":1699990000,"bits":386547056,"difficulty":1}}"#
    )
}

fn assert_summary_nonce_stays_decimal(painted: &str, nonce: &str) {
    let summary = painted
        .lines()
        .find(|line| line.starts_with("block hash "))
        .unwrap_or_else(|| panic!("summary line missing: {painted}"));
    assert!(
        summary.contains(&format!(" nonce {nonce} ")),
        "summary dropped the decimal nonce: {summary}"
    );
    assert!(
        !summary.contains("0x"),
        "summary line painted nonce hex: {summary}"
    );
}

fn assert_nonce_omitted(painted: &str) {
    assert!(
        painted.lines().all(|line| !line.starts_with("nonce")),
        "a line starts with nonce: {painted}"
    );
    assert!(
        !painted.contains("nonce 0"),
        "absent nonce painted nonce 0: {painted}"
    );
    assert!(
        !painted.contains("nonce 0 0x00000000"),
        "absent nonce painted nonce 0 0x00000000: {painted}"
    );
    assert!(
        !painted.contains("0x00000000"),
        "absent nonce painted 0x00000000: {painted}"
    );
    assert!(
        painted.lines().any(|line| line == "version 1 0x00000001"),
        "missing nonce fixture did not paint version 1 0x00000001: {painted}"
    );
    assert!(
        painted
            .lines()
            .any(|line| line == "bits 386547056 0x170a3d70"),
        "missing nonce fixture did not paint bits 386547056 0x170a3d70: {painted}"
    );
}

/// Block nonce hex beside the decimal on the details line.
///
/// Angular `decimal2hex` defaults to 4 bytes: `0x` plus exactly 8 lowercase
/// hex digits. The summary line stays the decimal nonce only. Bits details
/// stay the decimal beside eight hex digits. Difficulty stays `plain_number`.
/// A missing nonce key and JSON null are both `None` and omit the nonce line.
/// They do not substitute zero. Version 1 and bits 386547056 keep
/// `0x00000000` off those lines, so nonce 0 is the only source of that hex.
#[test]
fn block_screen_paints_nonce_hex() {
    let nine = paint_block_bits("noncehex9", block_nonce_json("noncehex9", "9"));
    let decimal_line = nine.lines().any(|line| line == "nonce 9");
    assert!(
        nine.lines().any(|line| line == "nonce 9 0x00000009"),
        "nonce 9 0x00000009 is absent while the decimal nonce line is present ({decimal_line}): {nine}"
    );
    assert!(
        !nine.lines().any(|line| line == "nonce 9"),
        "bare decimal nonce line remained beside the hex line: {nine}"
    );
    assert!(
        nine.lines()
            .any(|line| line.split_whitespace().any(|word| word == "0x00000009")),
        "0x00000009 was not the whole hex word: {nine}"
    );
    assert!(!nine.contains("0X"), "nonce hex was uppercase: {nine}");
    assert_summary_nonce_stays_decimal(&nine, "9");
    assert!(
        nine.lines().any(|line| line == "version 1 0x00000001"),
        "nonce 9 fixture did not keep version 1 0x00000001: {nine}"
    );
    assert!(
        nine.lines().any(|line| line == "bits 386547056 0x170a3d70"),
        "nonce 9 fixture did not keep bits 386547056 0x170a3d70: {nine}"
    );
    assert!(
        nine.lines().any(|line| line == "difficulty 1"),
        "difficulty 1 was not a details line: {nine}"
    );

    let zero = paint_block_bits("noncehex0", block_nonce_json("noncehex0", "0"));
    assert!(
        zero.lines().any(|line| line == "nonce 0 0x00000000"),
        "nonce 0 did not paint nonce 0 0x00000000: {zero}"
    );
    assert!(
        zero.lines()
            .any(|line| line.split_whitespace().any(|word| word == "0x00000000")),
        "0x00000000 was not the whole hex word beside nonce 0: {zero}"
    );
    let zero_hex_lines: Vec<_> = zero
        .lines()
        .filter(|line| line.contains("0x00000000"))
        .collect();
    assert_eq!(
        zero_hex_lines,
        vec!["Nonce 0x00000000", "nonce 0 0x00000000"],
        "0x00000000 was not only the nonce lines: {zero}"
    );
    assert!(
        zero.lines().any(|line| line == "version 1 0x00000001"),
        "nonce 0 fixture did not keep version 1 0x00000001: {zero}"
    );
    assert!(
        zero.lines().any(|line| line == "bits 386547056 0x170a3d70"),
        "nonce 0 fixture did not keep bits 386547056 0x170a3d70: {zero}"
    );
    assert_summary_nonce_stays_decimal(&zero, "0");

    let one = paint_block_bits("noncehex1", block_nonce_json("noncehex1", "1"));
    assert!(
        one.lines().any(|line| line == "nonce 1 0x00000001"),
        "nonce 1 did not paint nonce 1 0x00000001: {one}"
    );
    assert!(
        !one.lines()
            .any(|line| line.split_whitespace().any(|word| word == "0x1")),
        "nonce 1 painted 0x1: {one}"
    );
    assert_summary_nonce_stays_decimal(&one, "1");

    let missing = paint_block_bits("noncehexmiss", missing_nonce_json("noncehexmiss"));
    assert_nonce_omitted(&missing);

    let null = paint_block_bits("noncehexnull", block_nonce_json("noncehexnull", "null"));
    assert_nonce_omitted(&null);
}

/// Open one block whose `GET /block/:hash/header` body is the given hex.
/// The id is not all digits, so load uses `GET /block/:id` and not the height route.
fn paint_block_header_hex(id: &str, header: &str) -> (String, Vec<String>) {
    assert!(
        !id.is_empty() && !id.bytes().all(|byte| byte.is_ascii_digit()),
        "id must not be all digits: {id}"
    );
    assert!(!id.contains('?') && !id.contains('#'), "{id}");
    assert!(!header.contains('?') && !header.contains('#'), "{header}");
    let (mut host, rec) = host_with(
        Network::Mainnet,
        BTreeMap::from([
            (format!("/block/{id}"), block_json(id)),
            (format!("/block/{id}/txs"), "[]".to_string()),
            (format!("/block/{id}/txids"), "[]".to_string()),
            (format!("/block/{id}/header"), header.to_string()),
        ]),
    );
    host.block.query = id.to_string();
    host.activate(Screen::Block)
        .unwrap_or_else(|err| panic!("block {id} did not open: {err}"));
    let paths = rec.gets();
    assert!(
        paths
            .iter()
            .all(|path| !path.contains('?') && !path.contains('#')),
        "{paths:?}"
    );
    (painted(&host), paths)
}

/// Eighty-byte block header from `GET /block/:hash/header`.
///
/// Angular `i18n="block.header"` paints the plaintext label `Block Header Hex`.
/// The value is 160 hex characters on that same line. It is not transaction
/// hex and it is not escaped. The path has no query string.
#[test]
fn block_screen_paints_header_hex() {
    const LABEL: &str = "Block Header Hex";
    // Serialized header: version, previous block, merkle root, time, bits, nonce.
    let header = [
        "01000000",
        "0000000000000000000000000000000000000000000000000000000000000000",
        "3ba3edfd7a7b12b27ac72c3e67768f617fc81bc3888a51323a9fb8aa4b1e5e4a",
        "29ab5f49",
        "ffff001d",
        "1dac2b7c",
    ]
    .concat();
    assert_eq!(header.len(), 160, "header hex is not 80 bytes");
    assert!(
        header.bytes().all(|byte| byte.is_ascii_hexdigit()),
        "header is not hex: {header}"
    );
    let id = "hdrhexblock1";
    let (painted, paths) = paint_block_header_hex(id, &header);
    let expected = format!("{LABEL} {header}");
    assert!(
        painted.lines().any(|line| line == expected),
        "{expected} is absent from the block body: {painted}"
    );
    assert_eq!(
        painted.lines().filter(|line| *line == expected).count(),
        1,
        "{LABEL} was not one plaintext line: {painted}"
    );
    assert!(
        paths
            .iter()
            .any(|path| path == &format!("/block/{id}/header")),
        "load did not GET /block/{id}/header: {paths:?}"
    );
    assert!(
        paths
            .iter()
            .all(|path| !path.contains('?') && !path.contains('#')),
        "{paths:?}"
    );
}

fn block_merkle_json(id: &str, merkle_root: &str) -> String {
    format!(
        r#"{{"id":"{id}","height":2,"version":1,"timestamp":1700000000,"tx_count":1,"size":100,"weight":400,"merkle_root":"{merkle_root}","previousblockhash":null,"mediantime":1699990000,"nonce":9,"bits":2,"difficulty":1}}"#
    )
}

/// Block Merkle root from `GET /block/:hash`.
///
/// Angular `i18n="block.merkle-root"` paints the plaintext label `Merkle root`
/// and the full `merkle_root` string on that same line. The value is not
/// shortened. The raw `merkle_root` line stays. Details stay closed until the
/// details row opens. A missing key fails serde before details text, and that
/// paint has no `Merkle root`. This line is not Difficulty.
#[test]
fn block_screen_paints_merkle_root() {
    const LABEL: &str = "Merkle root";
    let root = "4a5e1e4baab89f3a32518a88c31bc87f618f76673e2cc77ab2127b7afdeda33b";
    assert_eq!(root.len(), 64);
    assert!(root.bytes().all(|byte| byte.is_ascii_hexdigit()), "{root}");
    let id = "merkleblock1";
    assert!(!id.bytes().all(|byte| byte.is_ascii_digit()), "{id}");
    assert!(!id.contains('?') && !id.contains('#'), "{id}");
    let (mut host, rec) = host_with(
        Network::Mainnet,
        BTreeMap::from([
            (format!("/block/{id}"), block_merkle_json(id, root)),
            (format!("/block/{id}/txs"), "[]".to_string()),
            (format!("/block/{id}/txids"), "[]".to_string()),
        ]),
    );
    host.block.query = id.to_string();
    host.activate(Screen::Block)
        .unwrap_or_else(|err| panic!("block {id} did not open: {err}"));
    let closed = painted(&host);
    let expected = format!("{LABEL} {root}");
    assert!(
        !closed.lines().any(|line| line == expected),
        "closed details painted {expected}: {closed}"
    );
    host.open_row("row block details")
        .unwrap_or_else(|err| panic!("block details did not open: {err}"));
    let opened = painted(&host);
    assert!(
        opened.lines().any(|line| line == expected),
        "{expected} is absent from the block details: {opened}"
    );
    assert_eq!(
        opened.lines().filter(|line| *line == expected).count(),
        1,
        "{LABEL} was not one plaintext line: {opened}"
    );
    assert!(
        !expected.contains("..."),
        "Merkle root was shortened: {expected}"
    );
    let raw = format!("merkle_root {root}");
    assert!(
        opened.lines().any(|line| line == raw),
        "raw {raw} was dropped: {opened}"
    );
    assert!(
        opened.lines().any(|line| line == "difficulty 1"),
        "difficulty 1 was not a details line: {opened}"
    );
    let paths = rec.gets();
    assert!(
        paths.iter().any(|path| path == &format!("/block/{id}")),
        "load did not GET /block/{id}: {paths:?}"
    );
    assert!(
        paths
            .iter()
            .all(|path| !path.contains('?') && !path.contains('#')),
        "{paths:?}"
    );

    let short_id = "merklemm";
    let short = paint_block_bits(short_id, block_merkle_json(short_id, "mm"));
    assert!(
        short.lines().any(|line| line == "Merkle root mm"),
        "short merkle root did not paint Merkle root mm: {short}"
    );
    assert!(
        short.lines().any(|line| line == "merkle_root mm"),
        "short merkle root dropped merkle_root mm: {short}"
    );
    assert!(
        !short.lines().any(|line| line.contains("Merkle root m...")),
        "short merkle root was shortened: {short}"
    );

    let missing = "merklemiss";
    let missing_body = format!(
        r#"{{"id":"{missing}","height":2,"version":1,"timestamp":1700000000,"tx_count":1,"size":100,"weight":400,"previousblockhash":null,"mediantime":1699990000,"nonce":9,"bits":2,"difficulty":1}}"#
    );
    let (mut host, rec) = host_with(
        Network::Mainnet,
        BTreeMap::from([
            (format!("/block/{missing}"), missing_body),
            (format!("/block/{missing}/txs"), "[]".to_string()),
            (format!("/block/{missing}/txids"), "[]".to_string()),
        ]),
    );
    host.block.query = missing.to_string();
    let err = host
        .activate(Screen::Block)
        .expect_err("missing merkle_root must fail the block parse");
    let message = err.to_string();
    assert!(
        message.contains("missing field `merkle_root`"),
        "missing merkle_root was not a parse error: {message}"
    );
    let paths = rec.gets();
    assert!(
        paths
            .iter()
            .all(|path| !path.contains('?') && !path.contains('#')),
        "{paths:?}"
    );
    let missing_painted = painted(&host);
    assert!(
        !missing_painted.contains("Merkle root"),
        "missing merkle_root painted Merkle root: {missing_painted}"
    );
}

fn block_difficulty_json(id: &str, difficulty: &str) -> String {
    format!(
        r#"{{"id":"{id}","height":2,"version":1,"timestamp":1700000000,"tx_count":1,"size":100,"weight":400,"merkle_root":"merkle-root","previousblockhash":null,"mediantime":1699990000,"nonce":9,"bits":2,"difficulty":{difficulty}}}"#
    )
}

/// Block difficulty from `GET /block/:hash`.
///
/// Angular `i18n="block.difficulty"` paints the plaintext label `Difficulty`
/// and the existing `plain_number` text on that same line. Whole values stay
/// integers (`1`, not `1.0`). The raw `difficulty` line stays. Details stay
/// closed until the details row opens. A missing key omits both lines.
#[test]
fn block_screen_paints_difficulty_label() {
    const LABEL: &str = "Difficulty";
    let id = "diffblock1";
    assert!(!id.bytes().all(|byte| byte.is_ascii_digit()), "{id}");
    assert!(!id.contains('?') && !id.contains('#'), "{id}");
    let (mut host, rec) = host_with(
        Network::Mainnet,
        BTreeMap::from([
            (format!("/block/{id}"), block_difficulty_json(id, "1.0")),
            (format!("/block/{id}/txs"), "[]".to_string()),
            (format!("/block/{id}/txids"), "[]".to_string()),
        ]),
    );
    host.block.query = id.to_string();
    host.activate(Screen::Block)
        .unwrap_or_else(|err| panic!("block {id} did not open: {err}"));
    let closed = painted(&host);
    let expected = format!("{LABEL} 1");
    assert!(
        !closed.lines().any(|line| line == expected),
        "closed details painted {expected}: {closed}"
    );
    host.open_row("row block details")
        .unwrap_or_else(|err| panic!("block details did not open: {err}"));
    let opened = painted(&host);
    assert!(
        opened.lines().any(|line| line == expected),
        "{expected} is absent from the block details: {opened}"
    );
    assert_eq!(
        opened.lines().filter(|line| *line == expected).count(),
        1,
        "{LABEL} was not one plaintext line: {opened}"
    );
    assert!(
        opened.lines().any(|line| line == "difficulty 1"),
        "difficulty 1 was not a details line: {opened}"
    );
    assert!(
        !opened
            .lines()
            .any(|line| line == "Difficulty 1.0" || line == "difficulty 1.0"),
        "plain_number painted 1.0: {opened}"
    );
    assert!(
        opened.lines().any(|line| line == "Merkle root merkle-root"),
        "Merkle root was dropped: {opened}"
    );
    let paths = rec.gets();
    assert!(
        paths.iter().any(|path| path == &format!("/block/{id}")),
        "load did not GET /block/{id}: {paths:?}"
    );
    assert!(
        paths
            .iter()
            .all(|path| !path.contains('?') && !path.contains('#')),
        "{paths:?}"
    );

    let fraction_id = "difffraction";
    let fraction = paint_block_bits(fraction_id, block_difficulty_json(fraction_id, "1.5"));
    assert!(
        fraction.lines().any(|line| line == "Difficulty 1.5"),
        "difficulty 1.5 did not paint Difficulty 1.5: {fraction}"
    );
    assert!(
        fraction.lines().any(|line| line == "difficulty 1.5"),
        "difficulty 1.5 dropped the raw line: {fraction}"
    );

    let missing = "diffmiss";
    let missing_body = format!(
        r#"{{"id":"{missing}","height":2,"version":1,"timestamp":1700000000,"tx_count":1,"size":100,"weight":400,"merkle_root":"merkle-root","previousblockhash":null,"mediantime":1699990000,"nonce":9,"bits":2}}"#
    );
    let missing_painted = paint_block_bits(missing, missing_body);
    assert!(
        missing_painted
            .lines()
            .all(|line| !line.starts_with("Difficulty") && !line.starts_with("difficulty")),
        "missing difficulty painted a difficulty line: {missing_painted}"
    );
}

/// Single block page paints one plaintext line, `Why is this block empty?`,
/// only when height is above 100000 and `tx_count` is at most 1.
///
/// Details stay closed. The sentence is not HTML. Height 100000, `tx_count`
/// 2, a missing height key, a missing `tx_count` key, and height 0 stay off.
/// Height 0 paints the plaintext line Genesis exactly once.
#[test]
fn block_screen_paints_empty_block() {
    const SENTENCE: &str = "Why is this block empty?";

    fn empty_block_json(id: &str, height: Option<u32>, tx_count: Option<u32>) -> String {
        let mut fields = vec![format!(r#""id":"{id}""#)];
        if let Some(height) = height {
            fields.push(format!(r#""height":{height}"#));
        }
        fields.push(r#""version":1"#.to_string());
        fields.push(r#""timestamp":1"#.to_string());
        if let Some(tx_count) = tx_count {
            fields.push(format!(r#""tx_count":{tx_count}"#));
        }
        fields.push(r#""size":20"#.to_string());
        fields.push(r#""weight":80"#.to_string());
        fields.push(r#""merkle_root":"merkle""#.to_string());
        fields.push(r#""mediantime":1"#.to_string());
        format!("{{{}}}", fields.join(","))
    }

    fn paint_empty_block(id: &str, body: String) -> Result<String, (ClientError, String)> {
        assert!(
            !id.is_empty() && !id.bytes().all(|byte| byte.is_ascii_digit()),
            "id must not be all digits: {id}"
        );
        assert!(!id.contains('?') && !id.contains('#'), "{id}");
        let (mut host, rec) = host_with(
            Network::Mainnet,
            BTreeMap::from([
                (format!("/block/{id}"), body),
                (format!("/block/{id}/txs"), "[]".to_string()),
                (format!("/block/{id}/txids"), "[]".to_string()),
            ]),
        );
        host.block.query = id.to_string();
        let opened = host.activate(Screen::Block);
        let paths = rec.gets();
        assert!(
            paths.iter().any(|path| path == &format!("/block/{id}")),
            "load did not GET /block/{id}: {paths:?}"
        );
        assert!(
            paths.iter().all(|path| !path.contains("/block-height/")),
            "load used /block-height/: {paths:?}"
        );
        assert!(
            paths
                .iter()
                .all(|path| !path.contains('?') && !path.contains('#')),
            "{paths:?}"
        );
        let text = painted(&host);
        match opened {
            Ok(()) => Ok(text),
            Err(err) => Err((err, text)),
        }
    }

    fn assert_sentence_once(painted: &str) {
        assert_eq!(
            painted.matches(SENTENCE).count(),
            1,
            "Why is this block empty? is absent: {painted}"
        );
        assert_eq!(
            painted.lines().filter(|line| *line == SENTENCE).count(),
            1,
            "the sentence was not one plaintext line: {painted}"
        );
        for banned in [
            "<span",
            "empty-block",
            "Previous Block",
            "Genesis",
            "tooltip",
            "skeleton",
        ] {
            assert!(
                !painted.contains(banned),
                "{banned} leaked into the painted shell: {painted}"
            );
        }
    }

    fn assert_sentence_absent(painted: &str) {
        assert_eq!(
            painted.matches(SENTENCE).count(),
            0,
            "Why is this block empty? leaked: {painted}"
        );
    }

    fn assert_height_painted(painted: &str, height: u32) {
        let needle = format!("height {height}");
        assert!(
            painted.lines().any(|line| line.contains(&needle)),
            "the page did not paint {needle}: {painted}"
        );
    }

    let zero = paint_empty_block(
        "emptyh100001z",
        empty_block_json("emptyh100001z", Some(100_001), Some(0)),
    )
    .unwrap_or_else(|(err, painted)| {
        panic!("height 100001 tx_count 0 did not open: {err}\n{painted}")
    });
    assert_sentence_once(&zero);

    let one = paint_empty_block(
        "emptyh100001o",
        empty_block_json("emptyh100001o", Some(100_001), Some(1)),
    )
    .unwrap_or_else(|(err, painted)| {
        panic!("height 100001 tx_count 1 did not open: {err}\n{painted}")
    });
    assert_sentence_once(&one);

    let boundary = paint_empty_block(
        "emptyh100000",
        empty_block_json("emptyh100000", Some(100_000), Some(1)),
    )
    .unwrap_or_else(|(err, painted)| {
        panic!("height 100000 tx_count 1 did not open: {err}\n{painted}")
    });
    assert_sentence_absent(&boundary);
    assert_height_painted(&boundary, 100_000);

    let two = paint_empty_block(
        "emptyh100001t",
        empty_block_json("emptyh100001t", Some(100_001), Some(2)),
    )
    .unwrap_or_else(|(err, painted)| {
        panic!("height 100001 tx_count 2 did not open: {err}\n{painted}")
    });
    assert_sentence_absent(&two);
    assert_height_painted(&two, 100_001);

    let (missing_height, missing_height_paint) = paint_empty_block(
        "emptyhmissh",
        empty_block_json("emptyhmissh", None, Some(0)),
    )
    .expect_err("omitted height must fail the block parse");
    let missing_height_message = missing_height.to_string();
    assert!(
        missing_height_message.contains("missing field `height`"),
        "activate error did not mention missing field `height`: {missing_height_message}"
    );
    assert_sentence_absent(&missing_height_paint);

    let (missing_tx_count, missing_tx_count_paint) = paint_empty_block(
        "emptyhmisstx",
        empty_block_json("emptyhmisstx", Some(100_001), None),
    )
    .expect_err("omitted tx_count must fail the block parse");
    let missing_tx_count_message = missing_tx_count.to_string();
    assert!(
        missing_tx_count_message.contains("missing field `tx_count`"),
        "activate error did not mention missing field `tx_count`: {missing_tx_count_message}"
    );
    assert_sentence_absent(&missing_tx_count_paint);

    let genesis_zero = paint_empty_block(
        "emptyhzeroz",
        empty_block_json("emptyhzeroz", Some(0), Some(0)),
    )
    .unwrap_or_else(|(err, painted)| panic!("height 0 tx_count 0 did not open: {err}\n{painted}"));
    assert_sentence_absent(&genesis_zero);
    assert_eq!(
        genesis_zero.matches("Genesis").count(),
        1,
        "height 0 tx_count 0 did not paint the plaintext line Genesis once: {genesis_zero}"
    );
    assert_eq!(
        genesis_zero
            .lines()
            .filter(|line| *line == "Genesis")
            .count(),
        1,
        "height 0 tx_count 0 Genesis was not one plaintext line: {genesis_zero}"
    );
    assert_height_painted(&genesis_zero, 0);

    let genesis_one = paint_empty_block(
        "emptyhzeroo",
        empty_block_json("emptyhzeroo", Some(0), Some(1)),
    )
    .unwrap_or_else(|(err, painted)| panic!("height 0 tx_count 1 did not open: {err}\n{painted}"));
    assert_sentence_absent(&genesis_one);
    assert_eq!(
        genesis_one.matches("Genesis").count(),
        1,
        "height 0 tx_count 1 did not paint the plaintext line Genesis once: {genesis_one}"
    );
    assert_eq!(
        genesis_one
            .lines()
            .filter(|line| *line == "Genesis")
            .count(),
        1,
        "height 0 tx_count 1 Genesis was not one plaintext line: {genesis_one}"
    );
    assert_height_painted(&genesis_one, 0);
}

/// Single block page paints the plaintext line `Genesis` only when height is 0.
///
/// Height 1 does not. The blocks list does not. Height 100001 with `tx_count`
/// 0 or 1 still paints `Why is this block empty?` and does not paint Genesis.
/// A missing height key fails parse, so neither sentence appears. The line is
/// not a span and not HTML.
#[test]
fn block_screen_paints_genesis() {
    const SENTENCE: &str = "Why is this block empty?";
    const GENESIS: &str = "Genesis";

    fn block_json(id: &str, height: Option<u32>, tx_count: Option<u32>) -> String {
        let mut fields = vec![format!(r#""id":"{id}""#)];
        if let Some(height) = height {
            fields.push(format!(r#""height":{height}"#));
        }
        fields.push(r#""version":1"#.to_string());
        fields.push(r#""timestamp":1"#.to_string());
        if let Some(tx_count) = tx_count {
            fields.push(format!(r#""tx_count":{tx_count}"#));
        }
        fields.push(r#""size":20"#.to_string());
        fields.push(r#""weight":80"#.to_string());
        fields.push(r#""merkle_root":"merkle""#.to_string());
        fields.push(r#""mediantime":1"#.to_string());
        format!("{{{}}}", fields.join(","))
    }

    fn assert_id_ok(id: &str) {
        assert!(
            !id.is_empty() && !id.bytes().all(|byte| byte.is_ascii_digit()),
            "id must not be all digits: {id}"
        );
        assert!(!id.contains('?') && !id.contains('#'), "{id}");
        assert!(
            !id.to_ascii_lowercase().contains("genesis"),
            "id must not contain Genesis: {id}"
        );
    }

    fn assert_plain(painted: &str) {
        for banned in ["<span", "genesis", "Previous Block"] {
            assert!(
                !painted.contains(banned),
                "{banned} leaked into the painted shell: {painted}"
            );
        }
        assert!(
            !painted.contains('<'),
            "HTML leaked into the painted shell: {painted}"
        );
    }

    fn assert_height(painted: &str, height: u32) {
        let needle = format!("height {height}");
        assert!(
            painted.lines().any(|line| line.contains(&needle)),
            "the page did not paint {needle}: {painted}"
        );
    }

    fn assert_genesis_once(label: &str, painted: &str) {
        assert_eq!(
            painted.matches(GENESIS).count(),
            1,
            "{label} did not paint Genesis once: {painted}"
        );
        assert_eq!(
            painted.lines().filter(|line| *line == GENESIS).count(),
            1,
            "{label} Genesis was not one plaintext line: {painted}"
        );
        assert_eq!(
            painted.matches(SENTENCE).count(),
            0,
            "{label} painted Why is this block empty?: {painted}"
        );
        assert_height(painted, 0);
        assert_plain(painted);
    }

    fn assert_no_genesis(label: &str, painted: &str) {
        assert!(
            !painted.contains(GENESIS),
            "{label} painted Genesis: {painted}"
        );
        assert_eq!(
            painted.matches(SENTENCE).count(),
            0,
            "{label} painted Why is this block empty?: {painted}"
        );
        assert_plain(painted);
    }

    fn assert_empty_once(label: &str, painted: &str) {
        assert_eq!(
            painted.matches(SENTENCE).count(),
            1,
            "{label} did not paint Why is this block empty? once: {painted}"
        );
        assert_eq!(
            painted.lines().filter(|line| *line == SENTENCE).count(),
            1,
            "{label} empty sentence was not one plaintext line: {painted}"
        );
        assert!(
            !painted.contains(GENESIS),
            "{label} painted Genesis: {painted}"
        );
        assert_plain(painted);
    }

    fn paint_block(id: &str, body: String) -> Result<String, (ClientError, String)> {
        assert_id_ok(id);
        let (mut host, rec) = host_with(
            Network::Mainnet,
            BTreeMap::from([
                (format!("/block/{id}"), body),
                (format!("/block/{id}/txs"), "[]".to_string()),
                (format!("/block/{id}/txids"), "[]".to_string()),
            ]),
        );
        host.block.query = id.to_string();
        let opened = host.activate(Screen::Block);
        let paths = rec.gets();
        assert!(
            paths.iter().any(|path| path == &format!("/block/{id}")),
            "load did not GET /block/{id}: {paths:?}"
        );
        assert!(
            paths.iter().all(|path| !path.contains("/block-height/")),
            "load used /block-height/: {paths:?}"
        );
        assert!(
            paths
                .iter()
                .all(|path| !path.contains('?') && !path.contains('#')),
            "{paths:?}"
        );
        let text = painted(&host);
        match opened {
            Ok(()) => Ok(text),
            Err(err) => Err((err, text)),
        }
    }

    let zero = paint_block("blk0tx0", block_json("blk0tx0", Some(0), Some(0))).unwrap_or_else(
        |(err, painted)| panic!("height 0 tx_count 0 did not open: {err}\n{painted}"),
    );
    assert_genesis_once("height 0 tx_count 0", &zero);

    let one = paint_block("blk0tx1", block_json("blk0tx1", Some(0), Some(1))).unwrap_or_else(
        |(err, painted)| panic!("height 0 tx_count 1 did not open: {err}\n{painted}"),
    );
    assert_genesis_once("height 0 tx_count 1", &one);

    let height_one = paint_block("blkheight1", block_json("blkheight1", Some(1), Some(1)))
        .unwrap_or_else(|(err, painted)| panic!("height 1 did not open: {err}\n{painted}"));
    assert_height(&height_one, 1);
    assert_no_genesis("height 1", &height_one);

    let list_id = "listblk0";
    assert_id_ok(list_id);
    let list_body = format!("[{}]", block_json(list_id, Some(0), Some(0)));
    let (mut list_host, list_rec) = host_with(
        Network::Mainnet,
        BTreeMap::from([("/blocks".to_string(), list_body)]),
    );
    list_host
        .activate(Screen::Blocks)
        .unwrap_or_else(|err| panic!("blocks list did not open: {err}"));
    let list_paths = list_rec.gets();
    assert!(
        list_paths.iter().any(|path| path == "/blocks"),
        "list did not GET /blocks: {list_paths:?}"
    );
    assert!(
        list_paths
            .iter()
            .all(|path| !path.contains('?') && !path.contains('#')),
        "{list_paths:?}"
    );
    let list = painted(&list_host);
    assert!(
        list.contains(list_id) && list.contains("height 0"),
        "the blocks list did not paint the height 0 block: {list}"
    );
    assert_no_genesis("blocks list", &list);

    let high_zero = paint_block(
        "blk100001z",
        block_json("blk100001z", Some(100_001), Some(0)),
    )
    .unwrap_or_else(|(err, painted)| {
        panic!("height 100001 tx_count 0 did not open: {err}\n{painted}")
    });
    assert_empty_once("height 100001 tx_count 0", &high_zero);

    let high_one = paint_block(
        "blk100001o",
        block_json("blk100001o", Some(100_001), Some(1)),
    )
    .unwrap_or_else(|(err, painted)| {
        panic!("height 100001 tx_count 1 did not open: {err}\n{painted}")
    });
    assert_empty_once("height 100001 tx_count 1", &high_one);

    let (missing, missing_paint) = paint_block("blkmissh", block_json("blkmissh", None, Some(0)))
        .expect_err("omitted height must fail the block parse");
    let missing_message = missing.to_string();
    assert!(
        missing_message.contains("missing field `height`"),
        "activate error did not mention missing field `height`: {missing_message}"
    );
    assert_no_genesis("missing height", &missing_paint);
}

fn block_summary_json(id: &str, size: u32) -> String {
    format!(
        r#"{{"id":"{id}","height":1,"version":1,"timestamp":1,"tx_count":1,"size":{size},"weight":4,"merkle_root":"merkle","mediantime":1}}"#
    )
}

/// Summary line only. Block details stay closed so a raw `size N` details
/// integer cannot satisfy the check.
fn paint_block_summary(id: &str, size: u32) -> String {
    assert!(
        !id.is_empty() && !id.bytes().all(|byte| byte.is_ascii_digit()),
        "id must not be all digits: {id}"
    );
    assert!(!id.contains('?') && !id.contains('#'), "{id}");
    let (mut host, rec) = host_with(
        Network::Mainnet,
        BTreeMap::from([
            (format!("/block/{id}"), block_summary_json(id, size)),
            (format!("/block/{id}/txs"), "[]".to_string()),
            (format!("/block/{id}/txids"), "[]".to_string()),
        ]),
    );
    host.block.query = id.to_string();
    host.activate(Screen::Block)
        .unwrap_or_else(|err| panic!("block {id} did not open: {err}"));
    let paths = rec.gets();
    assert!(
        paths
            .iter()
            .all(|path| !path.contains('?') && !path.contains('#')),
        "{paths:?}"
    );
    painted(&host)
}

fn block_hash_summary_line(painted: &str) -> String {
    painted
        .lines()
        .find(|line| line.starts_with("block hash "))
        .unwrap_or_else(|| panic!("summary line missing: {painted}"))
        .to_string()
}

fn assert_summary_size_has_no_mark(line: &str) {
    assert!(
        !line.contains("Size "),
        "title-case Size leaked into the summary line: {line}"
    );
    assert!(!line.contains("&lrm;"), "summary line has &lrm;: {line}");
    assert!(
        !line.contains('\u{200e}'),
        "summary line has U+200E: {line}"
    );
}

/// Block summary size from `GET /block/:id`, scaled like Angular `bytes: 2`.
///
/// The value is the number, a space, and the unit. Powers of 1000. Divide in
/// f64, then `(value * 100).round() / 100`. Size 1005 is `1` kB, not `1.01`.
/// The word Size stays off this line. No left-to-right mark. A missing size
/// key fails serde before the summary line and does not paint `size 0 B`.
#[test]
fn block_summary_paints_size() {
    let one_kb = paint_block_summary("sizesum1005", 1005);
    let line = block_hash_summary_line(&one_kb);
    assert!(
        line.contains("size 1 kB"),
        "size 1005 still shows the raw integer and size 1 kB is absent: {line}"
    );
    assert!(
        !line.contains("1.01"),
        "size 1005 painted 1.01 on the summary line: {line}"
    );
    assert!(
        !line.contains("size 1005"),
        "size 1005 stayed the raw integer on the summary line: {line}"
    );
    assert_summary_size_has_no_mark(&line);

    let nine = block_hash_summary_line(&paint_block_summary("sizesum999", 999));
    assert!(
        nine.contains("size 999 B"),
        "size 999 did not paint size 999 B on the summary line: {nine}"
    );
    assert!(
        !nine.contains("kB"),
        "size 999 painted kB on the summary line: {nine}"
    );
    assert_summary_size_has_no_mark(&nine);

    let zero = block_hash_summary_line(&paint_block_summary("sizesum0", 0));
    assert!(
        zero.contains("size 0 B"),
        "size 0 did not paint size 0 B on the summary line: {zero}"
    );
    assert_summary_size_has_no_mark(&zero);

    let kib = block_hash_summary_line(&paint_block_summary("sizesum1234", 1234));
    assert!(
        kib.contains("size 1.23 kB"),
        "size 1234 did not paint size 1.23 kB on the summary line: {kib}"
    );
    assert!(
        !kib.contains("size 1234"),
        "size 1234 stayed the raw integer on the summary line: {kib}"
    );
    assert_summary_size_has_no_mark(&kib);

    let missing = "sizesummiss";
    assert!(
        !missing.bytes().all(|byte| byte.is_ascii_digit()),
        "{missing}"
    );
    let missing_body = format!(
        r#"{{"id":"{missing}","height":1,"version":1,"timestamp":1,"tx_count":1,"weight":4,"merkle_root":"merkle","mediantime":1}}"#
    );
    let (mut host, rec) = host_with(
        Network::Mainnet,
        BTreeMap::from([
            (format!("/block/{missing}"), missing_body),
            (format!("/block/{missing}/txs"), "[]".to_string()),
            (format!("/block/{missing}/txids"), "[]".to_string()),
        ]),
    );
    host.block.query = missing.to_string();
    let err = host
        .activate(Screen::Block)
        .expect_err("missing size must fail the block parse");
    let message = err.to_string();
    assert!(
        message.contains("missing field `size`"),
        "missing size was not a parse error: {message}"
    );
    let paths = rec.gets();
    assert!(
        paths
            .iter()
            .all(|path| !path.contains('?') && !path.contains('#')),
        "{paths:?}"
    );
    let missing_painted = painted(&host);
    assert!(
        !missing_painted.contains("size 0 B"),
        "missing size painted size 0 B: {missing_painted}"
    );
}

fn block_summary_weight_json(id: &str, weight: u32) -> String {
    format!(
        r#"{{"id":"{id}","height":1,"version":1,"timestamp":1,"tx_count":1,"size":1,"weight":{weight},"merkle_root":"merkle","mediantime":1}}"#
    )
}

/// Summary line only. Block details stay closed so a raw `weight N` details
/// integer cannot satisfy the check.
fn paint_block_weight_summary(id: &str, weight: u32) -> String {
    assert!(
        !id.is_empty() && !id.bytes().all(|byte| byte.is_ascii_digit()),
        "id must not be all digits: {id}"
    );
    assert!(!id.contains('?') && !id.contains('#'), "{id}");
    let (mut host, rec) = host_with(
        Network::Mainnet,
        BTreeMap::from([
            (
                format!("/block/{id}"),
                block_summary_weight_json(id, weight),
            ),
            (format!("/block/{id}/txs"), "[]".to_string()),
            (format!("/block/{id}/txids"), "[]".to_string()),
        ]),
    );
    host.block.query = id.to_string();
    host.activate(Screen::Block)
        .unwrap_or_else(|err| panic!("block {id} did not open: {err}"));
    let paths = rec.gets();
    assert!(
        paths
            .iter()
            .all(|path| !path.contains('?') && !path.contains('#')),
        "{paths:?}"
    );
    painted(&host)
}

fn assert_summary_weight_has_no_mark(line: &str) {
    assert!(
        !line.contains("Weight "),
        "title-case Weight leaked into the summary line: {line}"
    );
    assert!(!line.contains("&lrm;"), "summary line has &lrm;: {line}");
    assert!(
        !line.contains('\u{200e}'),
        "summary line has U+200E: {line}"
    );
}

/// Block summary weight from `GET /block/:id`, scaled like Angular `wuBytes: 2`.
///
/// The value is the number, a space, and the unit. Powers of 1000, not 1024.
/// The unit is chosen from the raw weight before rounding. The word Weight
/// stays off this line. No left-to-right mark. A missing weight key fails
/// serde before the summary line and does not paint `weight 0 WU`.
#[test]
fn block_summary_paints_weight() {
    let one_and_half = paint_block_weight_summary("weightsum1500", 1500);
    let line = block_hash_summary_line(&one_and_half);
    assert!(
        line.contains("weight 1.5 kWU"),
        "weight 1500 still shows the raw integer and weight 1.5 kWU is absent: {line}"
    );
    assert!(
        !line.contains("weight 1500"),
        "weight 1500 stayed the raw integer on the summary line: {line}"
    );
    assert_summary_weight_has_no_mark(&line);

    let zero = block_hash_summary_line(&paint_block_weight_summary("weightsum0", 0));
    assert!(
        zero.contains("weight 0 WU"),
        "weight 0 did not paint weight 0 WU on the summary line: {zero}"
    );
    assert_summary_weight_has_no_mark(&zero);

    let nine = block_hash_summary_line(&paint_block_weight_summary("weightsum999", 999));
    assert!(
        nine.contains("weight 999 WU"),
        "weight 999 did not paint weight 999 WU on the summary line: {nine}"
    );
    assert!(
        !nine.contains("kWU"),
        "weight 999 painted kWU on the summary line: {nine}"
    );
    assert_summary_weight_has_no_mark(&nine);

    let almost = block_hash_summary_line(&paint_block_weight_summary("weightsum999999", 999999));
    assert!(
        almost.contains("weight 1000 kWU"),
        "weight 999999 did not paint weight 1000 kWU on the summary line: {almost}"
    );
    assert!(
        !almost.contains("MWU"),
        "weight 999999 painted MWU on the summary line: {almost}"
    );
    assert!(
        !almost.contains("weight 999999"),
        "weight 999999 stayed the raw integer on the summary line: {almost}"
    );
    assert_summary_weight_has_no_mark(&almost);

    let missing = "weightsummiss";
    assert!(
        !missing.bytes().all(|byte| byte.is_ascii_digit()),
        "{missing}"
    );
    let missing_body = format!(
        r#"{{"id":"{missing}","height":1,"version":1,"timestamp":1,"tx_count":1,"size":1,"merkle_root":"merkle","mediantime":1}}"#
    );
    let (mut host, rec) = host_with(
        Network::Mainnet,
        BTreeMap::from([
            (format!("/block/{missing}"), missing_body),
            (format!("/block/{missing}/txs"), "[]".to_string()),
            (format!("/block/{missing}/txids"), "[]".to_string()),
        ]),
    );
    host.block.query = missing.to_string();
    let err = host
        .activate(Screen::Block)
        .expect_err("missing weight must fail the block parse");
    let message = err.to_string();
    assert!(
        message.contains("missing field `weight`"),
        "missing weight was not a parse error: {message}"
    );
    let paths = rec.gets();
    assert!(
        paths
            .iter()
            .all(|path| !path.contains('?') && !path.contains('#')),
        "{paths:?}"
    );
    let missing_painted = painted(&host);
    assert!(
        !missing_painted.contains("weight 0 WU"),
        "missing weight painted weight 0 WU: {missing_painted}"
    );
}

/// Blocks list paints the word Height once beside height 1 and height 0.
///
/// A missing height key or JSON null drops that element. One bad element does
/// not fail the list. The list does not paint Genesis. The row stays
/// `row block height`, so Height is not counted a second time.
#[test]
fn blocks_list_paints_height_header() {
    fn assert_id_ok(id: &str) {
        assert!(
            !id.is_empty() && !id.bytes().all(|byte| byte.is_ascii_digit()),
            "id must not be all digits: {id}"
        );
        assert!(!id.contains('?') && !id.contains('#'), "{id}");
        let lower = id.to_ascii_lowercase();
        for letter in ["h", "e", "i", "g", "t", "n", "s"] {
            assert!(
                !lower.contains(letter),
                "id must not contain the letters of Height or Genesis: {id}"
            );
        }
    }

    fn block_json(id: &str, height: Option<&str>) -> String {
        let mut fields = vec![format!(r#""id":"{id}""#)];
        if let Some(height) = height {
            fields.push(format!(r#""height":{height}"#));
        }
        fields.push(r#""version":1"#.to_string());
        fields.push(r#""timestamp":1"#.to_string());
        fields.push(r#""tx_count":1"#.to_string());
        fields.push(r#""size":20"#.to_string());
        fields.push(r#""weight":80"#.to_string());
        fields.push(r#""merkle_root":"mm""#.to_string());
        fields.push(r#""mediantime":1"#.to_string());
        format!("{{{}}}", fields.join(","))
    }

    fn paint_blocks(body: String) -> Result<String, ClientError> {
        let (mut host, rec) = host_with(
            Network::Mainnet,
            BTreeMap::from([("/blocks".to_string(), body)]),
        );
        let opened = host.activate(Screen::Blocks);
        let paths = rec.gets();
        assert!(
            paths.iter().any(|path| path == "/blocks"),
            "list did not GET /blocks: {paths:?}"
        );
        assert!(
            paths.iter().all(|path| {
                !path.contains('?') && !path.contains('#') && !path.contains("/block-height/")
            }),
            "{paths:?}"
        );
        opened?;
        Ok(painted(&host))
    }

    fn assert_plain(painted: &str) {
        for banned in ["<td", "height-label", "<", "Previous Block", "Genesis"] {
            assert!(
                !painted.contains(banned),
                "{banned} leaked into the painted shell: {painted}"
            );
        }
    }

    fn assert_no_extra(painted: &str) {
        for banned in [
            "Previous Block",
            "tooltip",
            "graph",
            "mining",
            "pool",
            "fiat",
            "USD",
            "$",
        ] {
            assert!(
                !painted.contains(banned),
                "{banned} leaked into the painted shell: {painted}"
            );
        }
    }

    let id_one = "abcd1";
    assert_id_ok(id_one);
    let one = paint_blocks(format!("[{}]", block_json(id_one, Some("1"))))
        .unwrap_or_else(|err| panic!("height 1 blocks list did not open: {err}"));
    assert!(
        one.contains("Height height 1"),
        "the blocks list did not paint Height height 1: {one}"
    );
    assert_eq!(
        one.matches("Height").count(),
        1,
        "Height was not once on the blocks list: {one}"
    );
    assert!(
        one.contains(id_one) && one.contains("height 1"),
        "the blocks list dropped the block id or height 1: {one}"
    );
    assert_plain(&one);

    let id_zero = "abcd0";
    assert_id_ok(id_zero);
    let zero = paint_blocks(format!("[{}]", block_json(id_zero, Some("0"))))
        .unwrap_or_else(|err| panic!("height 0 blocks list did not open: {err}"));
    assert!(
        zero.contains("Height height 0"),
        "the blocks list did not paint Height height 0: {zero}"
    );
    assert_eq!(
        zero.matches("Height").count(),
        1,
        "Height was not once on the height 0 blocks list: {zero}"
    );
    assert!(
        zero.contains(id_zero) && zero.contains("height 0"),
        "the blocks list dropped the block id or height 0: {zero}"
    );
    assert!(
        !zero.contains("Genesis"),
        "the blocks list painted Genesis: {zero}"
    );
    assert_plain(&zero);

    let id_miss = "qqqbad";
    assert_id_ok(id_miss);
    let missing = paint_blocks(format!("[{}]", block_json(id_miss, None)))
        .unwrap_or_else(|err| panic!("missing height did not open the blocks list: {err}"));
    assert!(
        !missing.contains(id_miss),
        "missing height stayed on the blocks list: {missing}"
    );
    assert!(
        !missing.contains("Height"),
        "missing height painted Height: {missing}"
    );
    assert!(
        !missing.contains("Genesis"),
        "missing height painted Genesis: {missing}"
    );
    assert_plain(&missing);

    let id_null = "zzzbad";
    assert_id_ok(id_null);
    let null_height = paint_blocks(format!("[{}]", block_json(id_null, Some("null"))))
        .unwrap_or_else(|err| panic!("null height did not open the blocks list: {err}"));
    assert!(
        !null_height.contains(id_null),
        "null height stayed on the blocks list: {null_height}"
    );
    assert!(
        !null_height.contains("Height"),
        "null height painted Height: {null_height}"
    );
    assert!(
        !null_height.contains("Genesis"),
        "null height painted Genesis: {null_height}"
    );
    assert_plain(&null_height);

    let id_keep = "blkok1";
    assert_id_ok(id_keep);
    let mixed = paint_blocks(format!(
        "[{},{}]",
        block_json(id_keep, Some("1")),
        block_json(id_miss, None)
    ))
    .unwrap_or_else(|err| panic!("mixed blocks list did not open: {err}"));
    assert_eq!(
        mixed.matches("Height").count(),
        1,
        "mixed list did not paint Height once: {mixed}"
    );
    assert!(
        mixed.contains("Height height 1"),
        "mixed list did not paint Height height 1: {mixed}"
    );
    assert!(
        mixed.contains(id_keep),
        "mixed list dropped the valid block: {mixed}"
    );
    assert!(
        !mixed.contains(id_miss),
        "mixed list painted the bad block: {mixed}"
    );
    assert!(
        !mixed.contains("Genesis"),
        "mixed list painted Genesis: {mixed}"
    );
    assert_plain(&mixed);

    for painted in [&one, &zero, &missing, &null_height, &mixed] {
        assert_no_extra(painted);
    }
}

/// Blocks list paints one exact line `TXs` immediately before the count line.
///
/// The count line stays the existing `tx_count` text. Count 0 stays `0`.
/// Count 1234 stays `1234` with no thousands separator. A missing `tx_count`
/// key omits that row and does not paint `TXs`. `GET /blocks` has no query.
#[test]
fn blocks_list_paints_txs_label() {
    const LABEL: &str = "TXs";

    fn block_json(id: &str, tx_count: Option<&str>) -> String {
        let mut fields = vec![
            format!(r#""id":"{id}""#),
            r#""height":2"#.to_string(),
            r#""version":1"#.to_string(),
            r#""timestamp":9"#.to_string(),
        ];
        if let Some(tx_count) = tx_count {
            fields.push(format!(r#""tx_count":{tx_count}"#));
        }
        fields.push(r#""size":20"#.to_string());
        fields.push(r#""weight":80"#.to_string());
        fields.push(r#""merkle_root":"mm""#.to_string());
        fields.push(r#""mediantime":8"#.to_string());
        fields.push(r#""difficulty":1.0"#.to_string());
        format!("{{{}}}", fields.join(","))
    }

    fn paint_blocks(body: String) -> Result<String, ClientError> {
        let (mut host, rec) = host_with(
            Network::Mainnet,
            BTreeMap::from([("/blocks".to_string(), body)]),
        );
        let opened = host.activate(Screen::Blocks);
        let paths = rec.gets();
        assert!(
            paths.iter().any(|path| path == "/blocks"),
            "list did not GET /blocks: {paths:?}"
        );
        assert!(
            paths
                .iter()
                .all(|path| !path.contains('?') && !path.contains('#')),
            "{paths:?}"
        );
        opened?;
        Ok(painted(&host))
    }

    fn assert_label_before_count(painted: &str, id: &str, count: &str) {
        let lines: Vec<&str> = painted.lines().collect();
        assert!(
            lines.iter().any(|line| *line == LABEL),
            "no line is exactly {LABEL}: {painted}"
        );
        let marker = format!("tx_count {count}");
        let before = lines.windows(2).any(|pair| {
            pair[0] == LABEL
                && pair[1].contains(&format!("block hash {id}"))
                && pair[1].contains(&marker)
                && !pair[1].contains("1,234")
        });
        assert!(
            before,
            "{LABEL} was not immediately before the count line {marker}: {painted}"
        );
        let row = format!("row block height 2 hash {id} tx_count {count} time 9");
        assert!(
            lines.iter().any(|line| *line == row),
            "the existing count row was dropped: {painted}"
        );
        assert!(
            painted.contains("merkle_root mm"),
            "merkle root was dropped: {painted}"
        );
        assert!(
            painted.contains("difficulty 1") && !painted.contains("difficulty 1.0"),
            "difficulty left plain_number: {painted}"
        );
        assert!(
            !painted.contains("1,234"),
            "the count gained a thousands separator: {painted}"
        );
    }

    let shown = paint_blocks(format!("[{}]", block_json("txshow", Some("17"))))
        .unwrap_or_else(|err| panic!("tx_count 17 did not open: {err}"));
    assert!(
        shown.lines().any(|line| line == LABEL),
        "no line is exactly {LABEL}: {shown}"
    );
    assert_label_before_count(&shown, "txshow", "17");
    assert_eq!(
        shown.lines().filter(|line| *line == LABEL).count(),
        1,
        "{LABEL} was not once: {shown}"
    );

    let zero = paint_blocks(format!("[{}]", block_json("txzero", Some("0"))))
        .unwrap_or_else(|err| panic!("tx_count 0 did not open: {err}"));
    assert_label_before_count(&zero, "txzero", "0");
    assert!(zero.contains("tx_count 0"), "count 0 was omitted: {zero}");

    let wide = paint_blocks(format!("[{}]", block_json("txwide", Some("1234"))))
        .unwrap_or_else(|err| panic!("tx_count 1234 did not open: {err}"));
    assert_label_before_count(&wide, "txwide", "1234");
    assert!(
        wide.contains("tx_count 1234"),
        "count 1234 was not the raw integer: {wide}"
    );

    let missing = paint_blocks(format!("[{}]", block_json("txmiss", None)))
        .unwrap_or_else(|err| panic!("missing tx_count did not open: {err}"));
    assert!(
        !missing.lines().any(|line| line == LABEL),
        "missing tx_count painted {LABEL}: {missing}"
    );
    assert!(
        !missing.contains("txmiss"),
        "missing tx_count kept the row: {missing}"
    );
    assert!(
        !missing.contains("tx_count"),
        "missing tx_count still painted a count: {missing}"
    );

    let null_count = paint_blocks(format!("[{}]", block_json("txnull", Some("null"))))
        .unwrap_or_else(|err| panic!("null tx_count did not open: {err}"));
    assert!(
        !null_count.lines().any(|line| line == LABEL),
        "null tx_count painted {LABEL}: {null_count}"
    );
    assert!(
        !null_count.contains("txnull"),
        "null tx_count kept the row: {null_count}"
    );

    let page_id = "txpage";
    let (mut host, rec) = host_with(
        Network::Mainnet,
        BTreeMap::from([
            (format!("/block/{page_id}"), block_json(page_id, Some("17"))),
            (format!("/block/{page_id}/txs"), "[]".to_string()),
            (format!("/block/{page_id}/txids"), "[]".to_string()),
        ]),
    );
    host.block.query = page_id.to_string();
    host.activate(Screen::Block)
        .unwrap_or_else(|err| panic!("block {page_id} did not open: {err}"));
    let paths = rec.gets();
    assert!(
        paths
            .iter()
            .any(|path| path == &format!("/block/{page_id}")),
        "load did not GET /block/{page_id}: {paths:?}"
    );
    assert!(
        paths
            .iter()
            .all(|path| !path.contains('?') && !path.contains('#')),
        "{paths:?}"
    );
    let closed = painted(&host);
    let closed_lines: Vec<&str> = closed.lines().collect();
    assert!(
        closed_lines.windows(2).any(|pair| {
            pair[0] == LABEL
                && pair[1].contains("block hash txpage")
                && pair[1].contains("tx_count 17")
        }),
        "the block page did not paint {LABEL} before tx_count 17: {closed}"
    );
    host.open_row("row block details")
        .unwrap_or_else(|err| panic!("block details did not open: {err}"));
    let opened = painted(&host);
    assert!(
        opened
            .lines()
            .collect::<Vec<_>>()
            .windows(2)
            .any(|pair| { pair[0] == LABEL && pair[1] == "tx_count 17" }),
        "open details did not paint {LABEL} before the tx_count line: {opened}"
    );
    assert!(
        opened.lines().any(|line| line == "Merkle root mm"),
        "Merkle root was dropped: {opened}"
    );
    assert!(
        opened.lines().any(|line| line == "Difficulty 1"),
        "Difficulty was dropped: {opened}"
    );

    let missing_id = "txpagemiss";
    let (mut host, _) = host_with(
        Network::Mainnet,
        BTreeMap::from([(format!("/block/{missing_id}"), block_json(missing_id, None))]),
    );
    host.block.query = missing_id.to_string();
    let err = host
        .activate(Screen::Block)
        .expect_err("missing tx_count must fail the block parse");
    assert!(
        err.to_string().contains("missing field `tx_count`"),
        "missing tx_count was not a parse error: {err}"
    );
    let missing_page = painted(&host);
    assert!(
        !missing_page.lines().any(|line| line == LABEL),
        "missing tx_count painted {LABEL} on the block page: {missing_page}"
    );
}

const ALLOWED_HEADER: &str = "Allowed?";

fn assert_paint_is_not_html(text: &str) {
    for tag in ["<th", "<td", "<h2"] {
        assert!(!text.contains(tag), "{tag} leaked into the paint: {text}");
    }
}

fn assert_path_has_no_query_or_hash(path: &str) {
    assert!(!path.contains('?'), "{path}");
    assert!(!path.contains('#'), "{path}");
    assert!(!path.contains(ALLOWED_HEADER), "{path}");
}

fn assert_allowed_header_once(rendered: &str, painted: &str) {
    assert_eq!(
        rendered
            .lines()
            .filter(|line| *line == ALLOWED_HEADER)
            .count(),
        1,
        "the result did not paint {ALLOWED_HEADER} once: {rendered}"
    );
    assert_eq!(
        rendered.matches(ALLOWED_HEADER).count(),
        1,
        "the header text was not once: {rendered}"
    );
    assert_eq!(
        painted
            .lines()
            .filter(|line| *line == ALLOWED_HEADER)
            .count(),
        1,
        "the painted shell did not paint {ALLOWED_HEADER} once: {painted}"
    );
    assert_paint_is_not_html(rendered);
    assert_paint_is_not_html(painted);
}

fn assert_allowed_header_absent(rendered: &str, painted: &str) {
    assert_eq!(rendered, "", "expected empty paint: {rendered}");
    assert!(
        !rendered.contains(ALLOWED_HEADER),
        "{ALLOWED_HEADER} leaked into an empty result: {rendered}"
    );
    assert!(
        !painted.contains(ALLOWED_HEADER),
        "{ALLOWED_HEADER} leaked into the painted shell: {painted}"
    );
    assert_paint_is_not_html(rendered);
    assert_paint_is_not_html(painted);
}

/// `POST /txs/test` result list. Returns the screen body, the painted shell,
/// and the posted path.
fn paint_test_transactions(body: &str) -> (String, String, String) {
    let (mut host, rec) = host_with(
        Network::Mainnet,
        BTreeMap::from([("/txs/test".to_string(), body.to_string())]),
    );
    host.activate(Screen::TestTransactions)
        .unwrap_or_else(|err| panic!("test transactions did not open: {err}"));
    host.test_txs.input = "abcd".to_string();
    host.submit()
        .unwrap_or_else(|err| panic!("test transactions did not submit: {err}"));
    let posts = rec.posts();
    assert_eq!(posts.len(), 1, "{posts:?}");
    assert_eq!(posts[0].0, "/txs/test", "{posts:?}");
    assert_path_has_no_query_or_hash(&posts[0].0);
    assert_path_has_no_query_or_hash(&posts[0].1);
    let request = host.test_txs.request.as_ref().expect("posted request");
    assert_eq!(request.method, "POST");
    assert_eq!(request.path, "/txs/test");
    assert_path_has_no_query_or_hash(&request.path);
    assert_path_has_no_query_or_hash(&request.url);
    let path = request.path.clone();
    assert!(rec.gets().is_empty(), "{:?}", rec.gets());
    (host.rendered(), painted(&host), path)
}

/// Plaintext `Allowed?` on the test-transactions result, once, when a txid is present.
///
/// An empty array, an empty string, invalid JSON, and a parse with no txid
/// paint nothing and do not contain the header. `allowed: false` with no
/// reject-reason still shows the header when a txid is present. The ordinary
/// transaction screen keeps `Inputs & Outputs` and does not show the header.
/// The paint is not an HTML table cell or heading.
#[test]
fn test_transactions_paint_allowed_header() {
    let tx = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
    let spend = "eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee";

    let (rendered, shell, path) =
        paint_test_transactions(&format!(r#"[{{"txid":"{tx}","allowed":true}}]"#));
    assert_eq!(path, "/txs/test");
    assert_allowed_header_once(&rendered, &shell);
    assert!(
        rendered.contains(tx),
        "the result list hid the txid: {rendered}"
    );

    let with_witness = format!(r#"[{{"txid":"{tx}","wtxid":"wwww","allowed":true,"vsize":80}}]"#);
    let (rendered, shell, _) = paint_test_transactions(&with_witness);
    assert_allowed_header_once(&rendered, &shell);
    assert!(
        rendered.contains(&format!("test txid {tx} allowed true")),
        "the parsed result dropped the txid row: {rendered}"
    );

    let fallback =
        format!(r#"[{{"txid":"{tx}","allowed":true}},{{"txid":"{spend}","allowed":false}}]"#);
    let (rendered, shell, _) = paint_test_transactions(&fallback);
    assert_allowed_header_once(&rendered, &shell);
    assert!(rendered.contains(tx), "{rendered}");
    assert!(rendered.contains(spend), "{rendered}");

    let rejected = format!(r#"[{{"txid":"{spend}","allowed":false}}]"#);
    let (rendered, shell, _) = paint_test_transactions(&rejected);
    assert_allowed_header_once(&rendered, &shell);
    assert!(
        rendered.contains(spend),
        "allowed false with no reject-reason hid the txid: {rendered}"
    );

    for body in ["[]", "", "{not-json", "[{}]", r#"[{"txid":""}]"#] {
        let (rendered, shell, path) = paint_test_transactions(body);
        assert_eq!(path, "/txs/test");
        assert_allowed_header_absent(&rendered, &shell);
    }

    let vin = r#"{"txid":"prev","vout":0,"prevout":null,"scriptsig":"","scriptsig_asm":"","is_coinbase":false,"sequence":1}"#;
    let vout = r#"{"scriptpubkey":"","scriptpubkey_asm":"","scriptpubkey_type":"","value":9}"#;
    let txid = "allowrow1";
    let (mut host, rec) = host_with(
        Network::Mainnet,
        BTreeMap::from([(format!("/tx/{txid}"), tx_with_parties(txid, vin, vout))]),
    );
    let _opened = host.open_transaction(txid);
    let ordinary = painted(&host);
    assert!(
        ordinary.contains(INPUTS_OUTPUTS_HEADING),
        "the transaction screen dropped {INPUTS_OUTPUTS_HEADING}: {ordinary}"
    );
    assert!(
        ordinary.contains(&format!("tx txid {txid}"))
            || ordinary.contains(&format!("row tx txid {txid}")),
        "the ordinary transaction row was hidden: {ordinary}"
    );
    assert!(
        !ordinary.contains(ALLOWED_HEADER),
        "the ordinary transaction screen painted {ALLOWED_HEADER}: {ordinary}"
    );
    assert!(
        !ordinary.lines().any(|line| line.contains(ALLOWED_HEADER)),
        "the ordinary transaction row painted {ALLOWED_HEADER}: {ordinary}"
    );
    assert_paint_is_not_html(&ordinary);
    for path in rec.gets() {
        assert_path_has_no_query_or_hash(&path);
    }
}

const ISSUED_ASSET: &str = "ac3e0ff248c5051ffd61e00155b7122e5ebc04fd397a0ecbdd4f4e4a56232926";
const PEGGED_ASSET: &str = "6f0279e9ed041c3d710a9f57d0c02928416460c4b722ae3457a11eec381c526d";
const ISSUANCE_TX: &str = "cdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcd";
const PREV_TX: &str = "efefefefefefefefefefefefefefefefefefefefefefefefefefefefefefefef";
const REISSUE_TOKEN: &str = "1111111111111111111111111111111111111111111111111111111111111111";

enum AssetNameField<'a> {
    Omit,
    Null,
    Text(&'a str),
}

fn asset_json_string_field(key: &str, field: AssetNameField<'_>) -> String {
    match field {
        AssetNameField::Omit => String::new(),
        AssetNameField::Null => format!(r#""{key}":null,"#),
        AssetNameField::Text(value) => format!(r#""{key}":"{value}","#),
    }
}

fn issued_name_document(name: AssetNameField<'_>, ticker: AssetNameField<'_>) -> String {
    let name_field = asset_json_string_field("name", name);
    let ticker_field = asset_json_string_field("ticker", ticker);
    format!(
        r#"{{"asset_id":"{ISSUED_ASSET}",{name_field}{ticker_field}"precision":2,"entity":{{"domain":"registry.example"}},"contract":null,"issuance_txin":{{"txid":"{ISSUANCE_TX}","vin":0}},"issuance_prevout":{{"txid":"{PREV_TX}","vout":1}},"reissuance_token":"{REISSUE_TOKEN}","status":{{"confirmed":true}},"chain_stats":{{"tx_count":3,"issuance_count":1,"issued_amount":5000,"burned_amount":200,"has_blinded_issuances":false,"burned_reissuance_tokens":0}},"mempool_stats":{{"tx_count":9,"issuance_count":4,"issued_amount":999999,"burned_amount":888888,"has_blinded_issuances":false,"burned_reissuance_tokens":0}}}}"#
    )
}

fn pegged_name_document() -> String {
    format!(
        r#"{{"asset_id":"{PEGGED_ASSET}","chain_stats":{{"tx_count":4,"peg_in_count":2,"peg_in_amount":1000,"peg_out_count":1,"peg_out_amount":200,"burn_count":1,"burned_amount":50}},"mempool_stats":{{"tx_count":8,"peg_in_count":3,"peg_in_amount":777777,"peg_out_count":2,"peg_out_amount":666666,"burn_count":1,"burned_amount":555555}}}}"#
    )
}

fn open_named_asset(id: &str, document: &str) -> ScreenHost<Rec> {
    let (mut host, rec) = host_with(
        Network::Liquid,
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
            (format!("/asset/{id}"), document.to_string()),
        ]),
    );
    host.activate(Screen::Dashboard).expect("dashboard");
    host.set_query(id);
    host.reload().expect("asset");
    assert_no_closed(&rec.gets());
    host
}

fn assert_name_before_precision(paint: &str, name_line: &str) {
    let name_at = paint.lines().position(|line| line == name_line);
    let precision_at = paint.lines().position(|line| line == "Precision 2");
    assert!(
        name_at.is_some(),
        "Name line absent: expected {name_line:?}\n{paint}"
    );
    assert!(
        precision_at.is_some(),
        "Precision 2 absent after {name_line:?}\n{paint}"
    );
    assert!(
        name_at.unwrap() < precision_at.unwrap(),
        "{name_line:?} at {name_at:?} is not before Precision 2 at {precision_at:?}\n{paint}"
    );
}

fn assert_name_omitted(paint: &str) {
    assert!(
        !paint.lines().any(|line| line.starts_with("Name ")),
        "a blank name still painted a Name line\n{paint}"
    );
    assert!(
        paint.lines().any(|line| line == "Precision 2"),
        "Precision 2 was dropped with the Name line\n{paint}"
    );
}

/// Plaintext Name on the issued asset screen, before Precision.
///
/// The stored name and ticker are not HTML-escaped. A ticker that is missing,
/// empty, or only whitespace leaves the name alone. A missing, null, empty, or
/// whitespace name omits the row. A pegged asset has no Name row.
#[test]
fn asset_screen_paints_name() {
    let named = open_named_asset(
        ISSUED_ASSET,
        &issued_name_document(
            AssetNameField::Text("Test Asset"),
            AssetNameField::Text("TEST"),
        ),
    );
    let paint = painted(&named);
    assert_name_before_precision(&paint, "Name Test Asset (TEST)");
    assert!(
        paint.lines().any(|line| line == "asset name Test Asset"),
        "the asset name header was removed\n{paint}"
    );
    assert!(
        paint.lines().any(|line| line == "ticker TEST"),
        "the ticker header was removed\n{paint}"
    );
    assert!(!paint.contains("clipboard"), "{paint}");
    assert!(paint.contains("5000"), "{paint}");
    assert!(
        paint.lines().any(|line| line == "Issued amount 5000"),
        "the issued amount was scaled or dropped\n{paint}"
    );

    let raw = open_named_asset(
        ISSUED_ASSET,
        &issued_name_document(AssetNameField::Text("a&b<c"), AssetNameField::Text("d>e")),
    );
    let paint = painted(&raw);
    assert_name_before_precision(&paint, "Name a&b<c (d>e)");
    assert!(!paint.contains("&amp;"), "{paint}");
    assert!(!paint.contains("&lt;"), "{paint}");
    assert!(!paint.contains("&gt;"), "{paint}");

    for ticker in [
        AssetNameField::Omit,
        AssetNameField::Text(""),
        AssetNameField::Text("   "),
    ] {
        let host = open_named_asset(
            ISSUED_ASSET,
            &issued_name_document(AssetNameField::Text("OnlyName"), ticker),
        );
        let paint = painted(&host);
        assert_name_before_precision(&paint, "Name OnlyName");
        assert!(
            !paint.contains("OnlyName ("),
            "a blank ticker still painted parentheses\n{paint}"
        );
    }

    for name in [
        AssetNameField::Omit,
        AssetNameField::Null,
        AssetNameField::Text(""),
        AssetNameField::Text("   "),
    ] {
        let host = open_named_asset(
            ISSUED_ASSET,
            &issued_name_document(name, AssetNameField::Text("TEST")),
        );
        assert_name_omitted(&painted(&host));
    }

    let pegged = open_named_asset(PEGGED_ASSET, &pegged_name_document());
    let paint = painted(&pegged);
    assert!(
        !paint.lines().any(|line| line.starts_with("Name ")),
        "a pegged asset painted a Name line\n{paint}"
    );
    assert!(
        paint.lines().any(|line| line == "Pegged in 1000"),
        "Pegged in was dropped\n{paint}"
    );
}

/// Asset page title. Angular `i18n="asset|Liquid Asset page title"` paints the
/// plaintext line `Asset` once when `GET /asset/:id` is shown.
///
/// The line is the five letters Asset. It is not the registry column `Asset ID`,
/// not the `asset id` hex row, and not the asset-page line
/// `Name Test Asset (TEST)`. A pegged asset has the title and no Name row.
/// The request path has no query string.
#[test]
fn asset_screen_paints_asset_title() {
    const LABEL: &str = "Asset";
    const NAME_LINE: &str = "Name Test Asset (TEST)";
    let issued_row = format!("asset id {ISSUED_ASSET}");
    let (mut host, rec) = host_with(
        Network::Liquid,
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
            (
                format!("/asset/{ISSUED_ASSET}"),
                issued_name_document(
                    AssetNameField::Text("Test Asset"),
                    AssetNameField::Text("TEST"),
                ),
            ),
        ]),
    );
    host.activate(Screen::Dashboard).expect("dashboard");
    host.set_query(ISSUED_ASSET);
    host.reload().expect("issued asset");
    let paths = rec.gets();
    let asset_path = format!("/asset/{ISSUED_ASSET}");
    assert!(
        paths.iter().any(|path| path == &asset_path),
        "the asset document was not requested: {paths:?}"
    );
    for path in &paths {
        assert_path_has_no_query_or_hash(path);
    }
    let paint = painted(&host);
    let title_at = paint.lines().position(|line| line == LABEL);
    let name_at = paint.lines().position(|line| line == NAME_LINE);
    let id_at = paint.lines().position(|line| line == issued_row);
    assert!(
        title_at.is_some(),
        "the asset title was not painted\n{paint}"
    );
    assert!(name_at.is_some(), "the Name line was dropped\n{paint}");
    assert!(id_at.is_some(), "the asset id hex row was dropped\n{paint}");
    assert!(
        title_at.unwrap() < name_at.unwrap(),
        "Asset at {title_at:?} is not before {NAME_LINE} at {name_at:?}\n{paint}"
    );
    assert!(
        title_at.unwrap() < id_at.unwrap(),
        "Asset at {title_at:?} is not before {issued_row} at {id_at:?}\n{paint}"
    );
    assert_ne!(issued_row, LABEL);
    assert_ne!(NAME_LINE, LABEL);
    assert!(
        paint
            .lines()
            .filter(|line| *line == LABEL)
            .all(|line| line != "Asset ID" && line != NAME_LINE && line != issued_row),
        "Asset ID, the Name line, or the hex row was counted as the title: {paint}"
    );
    assert_eq!(
        paint.lines().filter(|line| *line == LABEL).count(),
        1,
        "the asset title was missing or repeated: {paint}"
    );
    assert!(
        paint.lines().any(|line| line == "Precision 2"),
        "Precision 2 was dropped\n{paint}"
    );
    assert_paint_is_not_html(&paint);

    let pegged_row = format!("asset id {PEGGED_ASSET}");
    let pegged = open_named_asset(PEGGED_ASSET, &pegged_name_document());
    let paint = painted(&pegged);
    assert_eq!(
        paint.lines().filter(|line| *line == LABEL).count(),
        1,
        "a pegged asset did not paint the title once: {paint}"
    );
    assert!(
        !paint.lines().any(|line| line.starts_with("Name ")),
        "a pegged asset painted a Name line\n{paint}"
    );
    assert!(
        paint.lines().any(|line| line == pegged_row),
        "the pegged asset id was dropped\n{paint}"
    );
    assert!(
        paint.lines().any(|line| line == "Pegged in 1000"),
        "Pegged in was dropped\n{paint}"
    );
    let title_at = paint.lines().position(|line| line == LABEL).unwrap();
    let id_at = paint.lines().position(|line| line == pegged_row).unwrap();
    assert!(
        title_at < id_at,
        "Asset at {title_at} is not before {pegged_row} at {id_at}\n{paint}"
    );
}

/// Asset h2 count. Angular `i18n="asset.M_of_N"` paints `{loaded} of {txCount}`
/// when the loaded list is not empty. `txCount` is chain `tx_count` plus
/// mempool `tx_count` from `GET /asset/:id`. The h2 has no noun after the count.
/// An empty list does not paint the line. The request path has no query string.
#[test]
fn screen_paints_asset_loaded_of() {
    const LABEL: &str = "1 of 12";
    let txs = r#"[{"txid":"assetchaintx","version":1,"locktime":0,"vin":[],"vout":[],"size":1,"weight":4,"sigops":0,"fee":1,"status":{"confirmed":true}}]"#;
    let (mut host, rec) = host_with(
        Network::Liquid,
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
            (
                format!("/asset/{ISSUED_ASSET}"),
                issued_name_document(
                    AssetNameField::Text("Test Asset"),
                    AssetNameField::Text("TEST"),
                ),
            ),
            (format!("/asset/{ISSUED_ASSET}/txs"), txs.to_string()),
        ]),
    );
    host.activate(Screen::Dashboard).expect("dashboard");
    host.set_query(ISSUED_ASSET);
    host.reload().expect("issued asset");
    let paths = rec.gets();
    assert!(
        paths
            .iter()
            .any(|path| path == &format!("/asset/{ISSUED_ASSET}")),
        "the asset document was not requested: {paths:?}"
    );
    for path in &paths {
        assert_path_has_no_query_or_hash(path);
    }
    let paint = painted(&host);
    assert!(
        paint.lines().any(|line| line == LABEL),
        "1 of 12 absent\n{paint}"
    );
    assert_eq!(
        paint.lines().filter(|line| *line == LABEL).count(),
        1,
        "1 of 12 was missing or repeated: {paint}"
    );
    assert_paint_is_not_html(&paint);
}

/// Plaintext broadcast form heading. One line, not the button and not HTML.
#[test]
fn broadcast_screen_paints_broadcast_transaction() {
    const LABEL: &str = "Broadcast Transaction";

    let (mut host, rec) = host_with(Network::Mainnet, BTreeMap::new());
    host.activate(Screen::Broadcast).expect("broadcast");
    let idle = painted(&host);
    assert_eq!(
        idle.lines().filter(|line| *line == LABEL).count(),
        1,
        "the broadcast heading was missing or repeated: {idle}"
    );
    assert!(
        !idle.contains('<'),
        "the broadcast screen painted HTML: {idle}"
    );
    for path in rec.gets() {
        assert_path_has_no_query_or_hash(&path);
    }
    for (path, _body) in rec.posts() {
        assert_path_has_no_query_or_hash(&path);
    }

    let sentence = "Transaction not found";
    let (mut host, rec) = host_with(
        Network::Mainnet,
        BTreeMap::from([("/tx".to_string(), sentence.to_string())]),
    );
    host.activate(Screen::Broadcast).expect("broadcast");
    host.broadcast.raw_tx = "abcd".to_string();
    host.submit().expect("post tx");
    assert_eq!(rec.posts()[0].0, "/tx");
    assert_path_has_no_query_or_hash(&rec.posts()[0].0);
    for path in rec.gets() {
        assert_path_has_no_query_or_hash(&path);
    }
    let failed = painted(&host);
    assert!(
        failed.contains(sentence),
        "the indexer sentence was dropped: {failed}"
    );
    assert!(
        !failed.contains("broadcast txid"),
        "failed POST /tx was painted as a txid: {failed}"
    );
    assert!(
        !failed.contains("/tx/"),
        "failed POST /tx was painted as a link: {failed}"
    );
    assert!(
        !failed.contains('<'),
        "the broadcast screen painted HTML: {failed}"
    );
    assert_eq!(
        failed.lines().filter(|line| *line == LABEL).count(),
        1,
        "the broadcast heading was missing or repeated after failure: {failed}"
    );
}

/// Assets list column header. Angular `i18n="Asset ticker header"` paints the
/// plaintext word `Ticker` once when `GET /assets/registry` is shown.
///
/// The line is the six letters Ticker. It is not the parenthetical on an
/// issued asset Name line (`Name Test Asset (TEST)`), and it is not escaped.
/// The request path has no query string.
#[test]
fn assets_registry_paints_ticker_column_header() {
    const LABEL: &str = "Ticker";
    const PARENTHETICAL: &str = "Name Test Asset (TEST)";
    let asset = "12".repeat(32);
    let registry = format!(
        r#"[{{"asset_id":"{asset}","name":"Test Asset","ticker":"TEST","precision":2,"issuance_txin":{{"txid":"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa","vin":0}},"issuance_prevout":{{"txid":"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa","vout":0}},"reissuance_token":"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa","status":{{"confirmed":true}},"chain_stats":{{"tx_count":1,"issuance_count":1,"issued_amount":1,"burned_amount":0,"has_blinded_issuances":false,"burned_reissuance_tokens":0}},"mempool_stats":{{"tx_count":0,"issuance_count":0,"issued_amount":0,"burned_amount":0,"has_blinded_issuances":false,"burned_reissuance_tokens":0}}}}]"#
    );
    let (mut host, rec) = host_with(
        Network::Liquid,
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
            ("/assets/registry".to_string(), registry),
        ]),
    );
    host.activate(Screen::Dashboard).expect("assets registry");
    let paths = rec.gets();
    assert!(
        paths.iter().any(|path| path == "/assets/registry"),
        "the assets registry was not requested: {paths:?}"
    );
    for path in &paths {
        assert_path_has_no_query_or_hash(path);
    }
    let paint = painted(&host);
    assert!(
        paint.contains("Test Asset"),
        "the assets registry did not show the issued asset: {paint}"
    );
    assert!(
        paint.lines().any(|line| line == "TEST"),
        "the ticker value was not painted, so the registry row is missing: {paint}"
    );
    assert!(
        paint
            .lines()
            .filter(|line| *line == LABEL)
            .all(|line| line != PARENTHETICAL && !line.contains("(TEST)")),
        "the Name line parenthetical was counted as the column header: {paint}"
    );
    assert_eq!(
        paint.lines().filter(|line| *line == LABEL).count(),
        1,
        "the assets registry column header {LABEL} was missing or repeated: {paint}"
    );
    assert_eq!(
        paint.matches(LABEL).count(),
        1,
        "the column header text {LABEL} was not once: {paint}"
    );
    assert_paint_is_not_html(&paint);
    assert!(!paint.contains("&amp;"), "{paint}");
    assert!(!paint.contains("&#"), "{paint}");
}

/// Assets list column header. Angular `i18n="Asset Issuer Domain header"` paints
/// the plaintext line `Issuer domain` once when `GET /assets/registry` is shown.
///
/// The line is those two words. It is not the parenthetical on an issued asset
/// Name line (`Name Test Asset (TEST)`), and it is not the Ticker header or the
/// ticker value. The request path has no query string.
#[test]
fn assets_registry_paints_issuer_domain_column_header() {
    const LABEL: &str = "Issuer domain";
    const PARENTHETICAL: &str = "Name Test Asset (TEST)";
    let asset = "12".repeat(32);
    let registry = format!(
        r#"[{{"asset_id":"{asset}","name":"Test Asset","ticker":"TEST","precision":2,"issuance_txin":{{"txid":"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa","vin":0}},"issuance_prevout":{{"txid":"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa","vout":0}},"reissuance_token":"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa","status":{{"confirmed":true}},"chain_stats":{{"tx_count":1,"issuance_count":1,"issued_amount":1,"burned_amount":0,"has_blinded_issuances":false,"burned_reissuance_tokens":0}},"mempool_stats":{{"tx_count":0,"issuance_count":0,"issued_amount":0,"burned_amount":0,"has_blinded_issuances":false,"burned_reissuance_tokens":0}}}}]"#
    );
    let (mut host, rec) = host_with(
        Network::Liquid,
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
            ("/assets/registry".to_string(), registry),
        ]),
    );
    host.activate(Screen::Dashboard).expect("assets registry");
    let paths = rec.gets();
    assert!(
        paths.iter().any(|path| path == "/assets/registry"),
        "the assets registry was not requested: {paths:?}"
    );
    for path in &paths {
        assert_path_has_no_query_or_hash(path);
    }
    let paint = painted(&host);
    assert!(
        paint.contains("Test Asset"),
        "the assets registry did not show the issued asset: {paint}"
    );
    assert!(
        paint.lines().any(|line| line == "TEST"),
        "the ticker value was not painted, so the registry row is missing: {paint}"
    );
    assert!(
        paint.lines().any(|line| line == "Ticker"),
        "the Ticker column header was dropped: {paint}"
    );
    assert!(
        paint
            .lines()
            .filter(|line| *line == LABEL)
            .all(|line| line != PARENTHETICAL && !line.contains("(TEST)")),
        "the Name line parenthetical was counted as the column header: {paint}"
    );
    assert_eq!(
        paint.lines().filter(|line| *line == LABEL).count(),
        1,
        "the assets registry column header {LABEL} was missing or repeated: {paint}"
    );
    assert_eq!(
        paint.matches(LABEL).count(),
        1,
        "the column header text {LABEL} was not once: {paint}"
    );
    assert_paint_is_not_html(&paint);
    assert!(!paint.contains("&amp;"), "{paint}");
    assert!(!paint.contains("&#"), "{paint}");
}

/// Assets list cell from `GET /assets/registry`. `entity.domain` is its own
/// line. The column header stays `Issuer domain` once.
///
/// The line is the stored domain. It is not `Issuer example.com` and it is
/// not a second copy of the header. A missing domain omits the line. The
/// request path has no query string.
#[test]
fn assets_registry_paints_the_issuer_domain_value() {
    const DOMAIN: &str = "example.com";
    let asset = "12".repeat(32);
    let registry = format!(
        r#"[{{"asset_id":"{asset}","name":"Test Asset","ticker":"TEST","entity":{{"domain":"{DOMAIN}"}},"precision":2,"issuance_txin":{{"txid":"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa","vin":0}},"issuance_prevout":{{"txid":"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa","vout":0}},"reissuance_token":"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa","status":{{"confirmed":true}},"chain_stats":{{"tx_count":1,"issuance_count":1,"issued_amount":1,"burned_amount":0,"has_blinded_issuances":false,"burned_reissuance_tokens":0}},"mempool_stats":{{"tx_count":0,"issuance_count":0,"issued_amount":0,"burned_amount":0,"has_blinded_issuances":false,"burned_reissuance_tokens":0}}}}]"#
    );
    let (mut host, rec) = host_with(
        Network::Liquid,
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
            ("/assets/registry".to_string(), registry),
        ]),
    );
    host.activate(Screen::Dashboard).expect("assets registry");
    let paths = rec.gets();
    assert!(
        paths.iter().any(|path| path == "/assets/registry"),
        "the assets registry was not requested: {paths:?}"
    );
    for path in &paths {
        assert_path_has_no_query_or_hash(path);
    }
    let paint = painted(&host);
    assert!(
        paint.lines().any(|line| line == "Test Asset"),
        "the issued name was dropped: {paint}"
    );
    assert!(
        paint.lines().any(|line| line == "TEST"),
        "the ticker value was dropped: {paint}"
    );
    assert_eq!(
        paint
            .lines()
            .filter(|line| *line == "Issuer domain")
            .count(),
        1,
        "the domain value was counted as the column header: {paint}"
    );
    assert_eq!(
        paint.lines().filter(|line| *line == DOMAIN).count(),
        1,
        "the issuer domain value was not one line {DOMAIN}: {paint}"
    );
    assert!(
        !paint.lines().any(|line| line == format!("Issuer {DOMAIN}")),
        "the asset-page issuer line was painted on the registry list: {paint}"
    );
    assert_paint_is_not_html(&paint);

    let bare = format!(
        r#"[{{"asset_id":"{asset}","name":"Test Asset","ticker":"TEST","precision":2,"issuance_txin":{{"txid":"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa","vin":0}},"issuance_prevout":{{"txid":"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa","vout":0}},"reissuance_token":"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa","status":{{"confirmed":true}},"chain_stats":{{"tx_count":1,"issuance_count":1,"issued_amount":1,"burned_amount":0,"has_blinded_issuances":false,"burned_reissuance_tokens":0}},"mempool_stats":{{"tx_count":0,"issuance_count":0,"issued_amount":0,"burned_amount":0,"has_blinded_issuances":false,"burned_reissuance_tokens":0}}}}]"#
    );
    let (mut host, _) = host_with(
        Network::Liquid,
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
            ("/assets/registry".to_string(), bare),
        ]),
    );
    host.activate(Screen::Dashboard)
        .expect("registry without a domain");
    let missing = painted(&host);
    assert!(
        !missing.lines().any(|line| line == DOMAIN),
        "a missing domain painted {DOMAIN}: {missing}"
    );
}

/// Assets list column header. Angular `i18n="Asset ID header"` paints the
/// plaintext line `Asset ID` once when `GET /assets/registry` is shown.
///
/// The line is those two words. It is not the `asset id` hex row, not the
/// parenthetical on an issued asset Name line (`Name Test Asset (TEST)`),
/// and it is not the Ticker or Issuer domain header. The issued name and
/// the ticker value stay. The request path has no query string.
#[test]
fn assets_registry_paints_asset_id_column_header() {
    const LABEL: &str = "Asset ID";
    const PARENTHETICAL: &str = "Name Test Asset (TEST)";
    let asset = "12".repeat(32);
    let hex_row = format!("asset id {asset}");
    let registry = format!(
        r#"[{{"asset_id":"{asset}","name":"Test Asset","ticker":"TEST","precision":2,"issuance_txin":{{"txid":"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa","vin":0}},"issuance_prevout":{{"txid":"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa","vout":0}},"reissuance_token":"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa","status":{{"confirmed":true}},"chain_stats":{{"tx_count":1,"issuance_count":1,"issued_amount":1,"burned_amount":0,"has_blinded_issuances":false,"burned_reissuance_tokens":0}},"mempool_stats":{{"tx_count":0,"issuance_count":0,"issued_amount":0,"burned_amount":0,"has_blinded_issuances":false,"burned_reissuance_tokens":0}}}}]"#
    );
    let (mut host, rec) = host_with(
        Network::Liquid,
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
            ("/assets/registry".to_string(), registry),
        ]),
    );
    host.activate(Screen::Dashboard).expect("assets registry");
    let paths = rec.gets();
    assert!(
        paths.iter().any(|path| path == "/assets/registry"),
        "the assets registry was not requested: {paths:?}"
    );
    for path in &paths {
        assert_path_has_no_query_or_hash(path);
    }
    let paint = painted(&host);
    assert!(
        paint.lines().any(|line| line == "Test Asset"),
        "the issued name was dropped: {paint}"
    );
    assert!(
        paint.lines().any(|line| line == "TEST"),
        "the ticker value was not painted, so the registry row is missing: {paint}"
    );
    assert!(
        paint.lines().any(|line| line == "Ticker"),
        "the Ticker column header was dropped: {paint}"
    );
    assert!(
        paint.lines().any(|line| line == "Issuer domain"),
        "the Issuer domain column header was dropped: {paint}"
    );
    assert!(
        paint.lines().any(|line| line == hex_row),
        "the asset id hex row was dropped: {paint}"
    );
    assert_ne!(
        hex_row, LABEL,
        "the asset id hex row was treated as the column header"
    );
    assert!(
        paint
            .lines()
            .filter(|line| *line == LABEL)
            .all(|line| line != hex_row && line != PARENTHETICAL && !line.contains("(TEST)")),
        "the asset id hex row or the Name line parenthetical was counted as the column header: {paint}"
    );
    assert_eq!(
        paint.lines().filter(|line| *line == LABEL).count(),
        1,
        "the assets registry column header {LABEL} was missing or repeated: {paint}"
    );
    assert_eq!(
        paint.matches(LABEL).count(),
        1,
        "the column header text {LABEL} was not once: {paint}"
    );
    assert_paint_is_not_html(&paint);
    assert!(!paint.contains("&amp;"), "{paint}");
    assert!(!paint.contains("&#"), "{paint}");
}

/// Assets list column header. Angular `i18n="Asset name header"` paints the
/// plaintext line `Name` once when `GET /assets/registry` is shown.
///
/// The line is the four letters Name. It is not the asset-page line
/// `Name Test Asset (TEST)`, and it is not a loading skeleton. A substring
/// check is not this test, because that asset-page line already contains the
/// word Name. Ticker, Issuer domain, and Asset ID stay after Name. The
/// `asset id` hex row stays. The request path has no query string.
#[test]
fn assets_registry_paints_name_column_header() {
    const LABEL: &str = "Name";
    const PARENTHETICAL: &str = "Name Test Asset (TEST)";
    let asset = "12".repeat(32);
    let hex_row = format!("asset id {asset}");
    let registry = format!(
        r#"[{{"asset_id":"{asset}","name":"Test Asset","ticker":"TEST","precision":2,"issuance_txin":{{"txid":"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa","vin":0}},"issuance_prevout":{{"txid":"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa","vout":0}},"reissuance_token":"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa","status":{{"confirmed":true}},"chain_stats":{{"tx_count":1,"issuance_count":1,"issued_amount":1,"burned_amount":0,"has_blinded_issuances":false,"burned_reissuance_tokens":0}},"mempool_stats":{{"tx_count":0,"issuance_count":0,"issued_amount":0,"burned_amount":0,"has_blinded_issuances":false,"burned_reissuance_tokens":0}}}}]"#
    );
    let (mut host, rec) = host_with(
        Network::Liquid,
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
            ("/assets/registry".to_string(), registry),
        ]),
    );
    host.activate(Screen::Dashboard).expect("assets registry");
    let paths = rec.gets();
    assert!(
        paths.iter().any(|path| path == "/assets/registry"),
        "the assets registry was not requested: {paths:?}"
    );
    for path in &paths {
        assert_path_has_no_query_or_hash(path);
    }
    let paint = painted(&host);
    assert!(
        paint.lines().any(|line| line == "Test Asset"),
        "the issued name was dropped: {paint}"
    );
    assert!(
        paint.lines().any(|line| line == "TEST"),
        "the ticker value was not painted, so the registry row is missing: {paint}"
    );
    assert!(
        paint.lines().any(|line| line == "Ticker"),
        "the Ticker column header was dropped: {paint}"
    );
    assert!(
        paint.lines().any(|line| line == "Issuer domain"),
        "the Issuer domain column header was dropped: {paint}"
    );
    assert!(
        paint.lines().any(|line| line == "Asset ID"),
        "the Asset ID column header was dropped: {paint}"
    );
    assert!(
        paint.lines().any(|line| line == hex_row),
        "the asset id hex row was dropped: {paint}"
    );
    assert_ne!(
        hex_row, LABEL,
        "the asset id hex row was treated as the column header"
    );
    assert_ne!(
        PARENTHETICAL, LABEL,
        "the asset-page parenthetical was treated as the column header"
    );
    assert!(
        paint
            .lines()
            .filter(|line| *line == LABEL)
            .all(|line| line != PARENTHETICAL && !line.contains("(TEST)")),
        "the Name line parenthetical was counted as the column header: {paint}"
    );
    assert_eq!(
        paint.lines().filter(|line| *line == LABEL).count(),
        1,
        "the assets registry column header {LABEL} was missing or repeated: {paint}"
    );
    let name_at = paint.lines().position(|line| line == "Name");
    let ticker_at = paint.lines().position(|line| line == "Ticker");
    let issuer_at = paint.lines().position(|line| line == "Issuer domain");
    let asset_id_at = paint.lines().position(|line| line == "Asset ID");
    assert!(
        name_at < ticker_at && ticker_at < issuer_at && issuer_at < asset_id_at,
        "header order was not Name, Ticker, Issuer domain, Asset ID: name {name_at:?} ticker {ticker_at:?} issuer {issuer_at:?} asset id {asset_id_at:?}\n{paint}"
    );
    assert_paint_is_not_html(&paint);
    assert!(!paint.contains("&amp;"), "{paint}");
    assert!(!paint.contains("&#"), "{paint}");
}

/// Recent transactions column header from `GET /mempool/recent`.
///
/// One exact line `vSize` when that route parses to at least one row. The
/// cell stays the raw integer. An empty list, a blank body, and invalid JSON
/// omit the line. Two rows print it once. The blocks list does not gain
/// `vSize`, `Size`, or `Weight`.
#[test]
fn recent_transactions_paint_vsize_header() {
    const LABEL: &str = "vSize";

    fn summary() -> String {
        r#"{"count":1,"vsize":9,"total_fee":1,"fee_histogram":[]}"#.to_string()
    }

    fn paint_recent(body: &str) -> (Result<(), ClientError>, String) {
        let (mut host, rec) = host_with(
            Network::Mainnet,
            BTreeMap::from([
                ("/mempool".to_string(), summary()),
                ("/mempool/recent".to_string(), body.to_string()),
            ]),
        );
        let opened = host.activate(Screen::Mempool);
        let paths = rec.gets();
        assert!(
            paths.iter().any(|path| path == "/mempool/recent"),
            "did not GET /mempool/recent: {paths:?}"
        );
        assert!(
            paths
                .iter()
                .all(|path| !path.contains('?') && !path.contains('#')),
            "{paths:?}"
        );
        (opened, painted(&host))
    }

    fn assert_header_once_before(painted: &str, txid: &str) {
        assert_eq!(
            painted.lines().filter(|line| *line == LABEL).count(),
            1,
            "{LABEL} was not one exact line: {painted}"
        );
        assert_eq!(
            painted.matches(LABEL).count(),
            1,
            "{LABEL} was not once: {painted}"
        );
        let lines: Vec<&str> = painted.lines().collect();
        let header_at = lines.iter().position(|line| *line == LABEL);
        let row_at = lines
            .iter()
            .position(|line| line.contains(&format!("recent txid {txid}")));
        assert!(
            header_at.is_some() && row_at.is_some() && header_at < row_at,
            "{LABEL} was not before recent txid {txid}: {painted}"
        );
    }

    fn assert_raw_cell(painted: &str, txid: &str, vsize: u32) {
        let marker = format!("vsize {vsize}");
        let row = painted
            .lines()
            .find(|line| {
                line.starts_with("recent txid ") && line.contains(txid) && line.contains(&marker)
            })
            .unwrap_or_else(|| panic!("missing raw cell {marker} for {txid}: {painted}"));
        assert!(
            !row.contains("vB") && !row.contains("kvB") && !row.contains(&format!("{vsize}.00")),
            "the cell did not stay the raw integer: {row}"
        );
        assert!(
            !row.contains(&format!("{vsize} vB")) && !row.contains(&format!("1,{vsize}")),
            "the cell gained a unit or a thousands separator: {row}"
        );
    }

    fn assert_header_absent(painted: &str, why: &str) {
        assert!(
            !painted.lines().any(|line| line == LABEL),
            "{why} painted {LABEL}: {painted}"
        );
        assert!(
            !painted.contains(LABEL),
            "{why} contains {LABEL}: {painted}"
        );
    }

    let one = r#"[{"txid":"recent-vsize-header","fee":10,"vsize":141,"value":1}]"#;
    let (opened, text) = paint_recent(one);
    opened.unwrap_or_else(|err| panic!("vsize 141 did not open: {err}"));
    assert_header_once_before(&text, "recent-vsize-header");
    assert_raw_cell(&text, "recent-vsize-header", 141);
    assert!(!text.contains("vB"), "{text}");

    let zero = r#"[{"txid":"recent-zero-vsize","fee":10,"vsize":0,"value":1}]"#;
    let (opened, text) = paint_recent(zero);
    opened.unwrap_or_else(|err| panic!("vsize 0 did not open: {err}"));
    assert_header_once_before(&text, "recent-zero-vsize");
    assert_raw_cell(&text, "recent-zero-vsize", 0);
    assert!(
        text.lines()
            .any(|line| line == "recent txid recent-zero-vsize fee 10 vsize 0 value 1"),
        "vsize 0 did not stay the raw integer: {text}"
    );

    let two = r#"[{"txid":"recent-one","fee":10,"vsize":141,"value":1},{"txid":"recent-two","fee":20,"vsize":1000,"value":2}]"#;
    let (opened, text) = paint_recent(two);
    opened.unwrap_or_else(|err| panic!("two rows did not open: {err}"));
    assert_header_once_before(&text, "recent-one");
    assert_raw_cell(&text, "recent-one", 141);
    assert_raw_cell(&text, "recent-two", 1000);
    assert!(
        !text.contains("1,000") && !text.contains("vB") && !text.contains("kvB"),
        "two rows did not keep the raw integers: {text}"
    );

    let (opened, text) = paint_recent("[]");
    opened.unwrap_or_else(|err| panic!("empty list did not open: {err}"));
    assert_header_absent(&text, "empty list");
    assert!(
        text.contains("recent:"),
        "empty list dropped the recent marker: {text}"
    );

    let (opened, text) = paint_recent("");
    assert!(opened.is_err(), "blank body was treated as rows: {text}");
    assert_header_absent(&text, "blank body");

    let (opened, text) = paint_recent("not-json");
    assert!(opened.is_err(), "invalid JSON was treated as rows: {text}");
    assert_header_absent(&text, "invalid JSON");

    let (opened, text) = paint_recent("[{}]");
    assert!(
        opened.is_err(),
        "a row missing vsize was treated as rows: {text}"
    );
    assert_header_absent(&text, "invalid row");

    let block_id = "blkvs1";
    let blocks_body = format!(
        r#"[{{"id":"{block_id}","height":1,"version":1,"timestamp":1,"tx_count":1,"size":20,"weight":80,"merkle_root":"mm","mediantime":1,"difficulty":1.0}}]"#
    );
    let (mut host, rec) = host_with(
        Network::Mainnet,
        BTreeMap::from([("/blocks".to_string(), blocks_body)]),
    );
    host.activate(Screen::Blocks)
        .unwrap_or_else(|err| panic!("blocks list did not open: {err}"));
    let paths = rec.gets();
    assert!(
        paths.iter().any(|path| path == "/blocks"),
        "list did not GET /blocks: {paths:?}"
    );
    assert!(
        paths
            .iter()
            .all(|path| !path.contains('?') && !path.contains('#')),
        "{paths:?}"
    );
    let blocks = painted(&host);
    assert!(
        !blocks.contains("vSize"),
        "blocks list painted vSize: {blocks}"
    );
    assert!(
        !blocks.contains("Size"),
        "blocks list painted Size: {blocks}"
    );
    assert!(
        !blocks.contains("Weight"),
        "blocks list painted Weight: {blocks}"
    );
    assert!(blocks.contains("TXs"), "TXs was dropped: {blocks}");
    assert!(
        blocks.contains(block_id) && blocks.contains("merkle_root mm"),
        "the block row was dropped: {blocks}"
    );
    assert!(
        blocks.contains("difficulty 1") && !blocks.contains("difficulty 1.0"),
        "difficulty left plain_number: {blocks}"
    );
}

/// Recent transactions amount column header from `GET /mempool/recent`.
///
/// One exact line `Amount` when that route parses to at least one row. The
/// value cell stays the raw integer, including zero. An empty list, a blank
/// body, and invalid JSON omit the line. Two rows print it once. The blocks
/// list does not gain `Amount`, `Size`, or `Weight`.
#[test]
fn recent_transactions_paint_amount_header() {
    const LABEL: &str = "Amount";

    fn summary() -> String {
        r#"{"count":1,"vsize":9,"total_fee":1,"fee_histogram":[]}"#.to_string()
    }

    fn paint_recent(body: &str) -> (Result<(), ClientError>, String) {
        let (mut host, rec) = host_with(
            Network::Mainnet,
            BTreeMap::from([
                ("/mempool".to_string(), summary()),
                ("/mempool/recent".to_string(), body.to_string()),
            ]),
        );
        let opened = host.activate(Screen::Mempool);
        let paths = rec.gets();
        assert!(
            paths.iter().any(|path| path == "/mempool/recent"),
            "did not GET /mempool/recent: {paths:?}"
        );
        assert!(
            paths
                .iter()
                .all(|path| !path.contains('?') && !path.contains('#')),
            "{paths:?}"
        );
        (opened, painted(&host))
    }

    fn assert_header_once_before(painted: &str, txid: &str) {
        assert_eq!(
            painted.lines().filter(|line| *line == LABEL).count(),
            1,
            "{LABEL} was not one exact line: {painted}"
        );
        assert_eq!(
            painted.matches(LABEL).count(),
            1,
            "{LABEL} was not once: {painted}"
        );
        let lines: Vec<&str> = painted.lines().collect();
        let header_at = lines.iter().position(|line| *line == LABEL);
        let vsize_at = lines.iter().position(|line| *line == "vSize");
        let row_at = lines
            .iter()
            .position(|line| line.contains(&format!("recent txid {txid}")));
        assert!(
            header_at.is_some()
                && vsize_at.is_some()
                && row_at.is_some()
                && header_at < vsize_at
                && vsize_at < row_at,
            "{LABEL} was not before vSize and recent txid {txid}: {painted}"
        );
    }

    fn value_token(row: &str, value: u64) -> bool {
        let expected = value.to_string();
        row.split_whitespace()
            .collect::<Vec<_>>()
            .windows(2)
            .any(|pair| pair == ["value", expected.as_str()])
    }

    fn assert_raw_value(painted: &str, txid: &str, value: u64) {
        let row = painted
            .lines()
            .find(|line| line.starts_with("recent txid ") && line.contains(txid))
            .unwrap_or_else(|| panic!("missing recent row for {txid}: {painted}"));
        assert!(
            value_token(row, value),
            "missing raw value {value} for {txid}: {row}"
        );
        assert!(
            !row.contains(',')
                && !row.contains('$')
                && !row.contains('€')
                && !row.contains("BTC")
                && !row.contains("vB")
                && !row.contains(&format!("{value}.00")),
            "the value cell did not stay the raw integer: {row}"
        );
        assert!(
            !painted.contains("Fiat price last updated"),
            "the amount header was treated as the fiat column: {painted}"
        );
    }

    fn assert_header_absent(painted: &str, why: &str) {
        assert!(
            !painted.lines().any(|line| line == LABEL),
            "{why} painted {LABEL}: {painted}"
        );
        assert!(
            !painted.contains(LABEL),
            "{why} contains {LABEL}: {painted}"
        );
    }

    let one = r#"[{"txid":"recent-amount-header","fee":10,"vsize":141,"value":1000}]"#;
    let (opened, text) = paint_recent(one);
    opened.unwrap_or_else(|err| panic!("value 1000 did not open: {err}"));
    assert_header_once_before(&text, "recent-amount-header");
    assert_raw_value(&text, "recent-amount-header", 1000);
    assert!(
        !text.contains("1,000") && !text.contains('$') && !text.contains("BTC"),
        "value 1000 did not stay the raw integer: {text}"
    );

    let zero = r#"[{"txid":"recent-zero-amount","fee":10,"vsize":0,"value":0}]"#;
    let (opened, text) = paint_recent(zero);
    opened.unwrap_or_else(|err| panic!("value 0 did not open: {err}"));
    assert_header_once_before(&text, "recent-zero-amount");
    assert_raw_value(&text, "recent-zero-amount", 0);
    assert!(
        text.lines()
            .any(|line| { line == "recent txid recent-zero-amount fee 10 vsize 0 value 0" }),
        "value 0 did not stay the raw integer: {text}"
    );

    let two = r#"[{"txid":"recent-amt-one","fee":10,"vsize":141,"value":1},{"txid":"recent-amt-two","fee":20,"vsize":1000,"value":2}]"#;
    let (opened, text) = paint_recent(two);
    opened.unwrap_or_else(|err| panic!("two rows did not open: {err}"));
    assert_header_once_before(&text, "recent-amt-one");
    assert_raw_value(&text, "recent-amt-one", 1);
    assert_raw_value(&text, "recent-amt-two", 2);
    assert!(
        !text.contains("1,000") && !text.contains('$') && !text.contains("BTC"),
        "two rows did not keep the raw integers: {text}"
    );

    let (opened, text) = paint_recent("[]");
    opened.unwrap_or_else(|err| panic!("empty list did not open: {err}"));
    assert_header_absent(&text, "empty list");
    assert!(
        text.contains("recent:"),
        "empty list dropped the recent marker: {text}"
    );

    let (opened, text) = paint_recent("");
    assert!(opened.is_err(), "blank body was treated as rows: {text}");
    assert_header_absent(&text, "blank body");

    let (opened, text) = paint_recent("not-json");
    assert!(opened.is_err(), "invalid JSON was treated as rows: {text}");
    assert_header_absent(&text, "invalid JSON");

    let (opened, text) = paint_recent("[{}]");
    assert!(
        opened.is_err(),
        "a row missing value was treated as rows: {text}"
    );
    assert_header_absent(&text, "invalid row");

    let block_id = "blkamt";
    let blocks_body = format!(
        r#"[{{"id":"{block_id}","height":1,"version":1,"timestamp":1,"tx_count":1,"size":20,"weight":80,"merkle_root":"mm","mediantime":1,"difficulty":1.0}}]"#
    );
    let (mut host, rec) = host_with(
        Network::Mainnet,
        BTreeMap::from([("/blocks".to_string(), blocks_body)]),
    );
    host.activate(Screen::Blocks)
        .unwrap_or_else(|err| panic!("blocks list did not open: {err}"));
    let paths = rec.gets();
    assert!(
        paths.iter().any(|path| path == "/blocks"),
        "list did not GET /blocks: {paths:?}"
    );
    assert!(
        paths
            .iter()
            .all(|path| !path.contains('?') && !path.contains('#')),
        "{paths:?}"
    );
    let blocks = painted(&host);
    assert!(
        !blocks.contains("Amount"),
        "blocks list painted Amount: {blocks}"
    );
    assert!(
        !blocks.contains("Size"),
        "blocks list painted Size: {blocks}"
    );
    assert!(
        !blocks.contains("Weight"),
        "blocks list painted Weight: {blocks}"
    );
    assert!(blocks.contains("TXs"), "TXs was dropped: {blocks}");
    assert!(
        blocks.contains(block_id) && blocks.contains("merkle_root mm"),
        "the block row was dropped: {blocks}"
    );
    assert!(
        blocks.contains("difficulty 1") && !blocks.contains("difficulty 1.0"),
        "difficulty left plain_number: {blocks}"
    );
    assert!(
        !blocks.contains("Fiat price last updated"),
        "blocks list painted the fiat column: {blocks}"
    );
}

/// Recent transactions txid column header from `GET /mempool/recent`.
///
/// The Angular header cell is `TXID`. One exact line `TXID` when that route
/// parses to at least one row, before `Amount` and `vSize`. The txid cell stays
/// `recent txid`. An empty list, a blank body, and invalid JSON omit the line.
/// Two rows print it once. The blocks list does not gain `TXID`, `Amount`,
/// `vSize`, `Size`, or `Weight`.
#[test]
fn recent_transactions_paint_txid_header() {
    const LABEL: &str = "TXID";

    fn summary() -> String {
        r#"{"count":1,"vsize":9,"total_fee":1,"fee_histogram":[]}"#.to_string()
    }

    fn paint_recent(body: &str) -> (Result<(), ClientError>, String) {
        let (mut host, rec) = host_with(
            Network::Mainnet,
            BTreeMap::from([
                ("/mempool".to_string(), summary()),
                ("/mempool/recent".to_string(), body.to_string()),
            ]),
        );
        let opened = host.activate(Screen::Mempool);
        let paths = rec.gets();
        assert!(
            paths.iter().any(|path| path == "/mempool/recent"),
            "did not GET /mempool/recent: {paths:?}"
        );
        assert!(
            paths
                .iter()
                .all(|path| !path.contains('?') && !path.contains('#')),
            "{paths:?}"
        );
        (opened, painted(&host))
    }

    fn assert_header_once_before(painted: &str, txid: &str) {
        assert_eq!(
            painted.lines().filter(|line| *line == LABEL).count(),
            1,
            "{LABEL} was not one exact line: {painted}"
        );
        assert_eq!(
            painted.matches(LABEL).count(),
            1,
            "{LABEL} was not once: {painted}"
        );
        let lines: Vec<&str> = painted.lines().collect();
        let header_at = lines.iter().position(|line| *line == LABEL);
        let amount_at = lines.iter().position(|line| *line == "Amount");
        let vsize_at = lines.iter().position(|line| *line == "vSize");
        let row_at = lines
            .iter()
            .position(|line| line.contains(&format!("recent txid {txid}")));
        assert!(
            header_at.is_some()
                && amount_at.is_some()
                && vsize_at.is_some()
                && row_at.is_some()
                && header_at < amount_at
                && amount_at < vsize_at
                && vsize_at < row_at,
            "{LABEL} was not before Amount, vSize, and recent txid {txid}: {painted}"
        );
    }

    fn assert_raw_txid(painted: &str, txid: &str, fee: u64, vsize: u32, value: u64) {
        let row = painted
            .lines()
            .find(|line| line.starts_with("recent txid ") && line.contains(txid))
            .unwrap_or_else(|| panic!("missing recent txid cell for {txid}: {painted}"));
        assert!(
            row.split_whitespace()
                .collect::<Vec<_>>()
                .windows(2)
                .any(|pair| pair == ["txid", txid]),
            "the txid cell was not the raw id: {row}"
        );
        assert!(
            row.contains(&format!("fee {fee}")) && row.contains(&format!("vsize {vsize}")),
            "the existing cell lost fee or vsize: {row}"
        );
        let value_text = value.to_string();
        assert!(
            row.split_whitespace()
                .collect::<Vec<_>>()
                .windows(2)
                .any(|pair| pair == ["value", value_text.as_str()]),
            "the existing cell lost the raw value: {row}"
        );
        assert!(
            !painted.contains('$')
                && !painted.contains('€')
                && !painted.contains("BTC")
                && !painted.contains("Fiat price last updated"),
            "the txid header was treated as a fiat amount: {painted}"
        );
    }

    fn assert_header_absent(painted: &str, why: &str) {
        assert!(
            !painted.lines().any(|line| line == LABEL),
            "{why} painted {LABEL}: {painted}"
        );
        assert!(
            !painted.contains(LABEL),
            "{why} contains {LABEL}: {painted}"
        );
    }

    let one = r#"[{"txid":"recent-txid-header","fee":10,"vsize":141,"value":1000}]"#;
    let (opened, text) = paint_recent(one);
    opened.unwrap_or_else(|err| panic!("txid header row did not open: {err}"));
    assert_header_once_before(&text, "recent-txid-header");
    assert_raw_txid(&text, "recent-txid-header", 10, 141, 1000);

    let two = r#"[{"txid":"recent-txid-one","fee":10,"vsize":141,"value":1},{"txid":"recent-txid-two","fee":20,"vsize":1000,"value":2}]"#;
    let (opened, text) = paint_recent(two);
    opened.unwrap_or_else(|err| panic!("two rows did not open: {err}"));
    assert_header_once_before(&text, "recent-txid-one");
    assert_raw_txid(&text, "recent-txid-one", 10, 141, 1);
    assert_raw_txid(&text, "recent-txid-two", 20, 1000, 2);

    let (opened, text) = paint_recent("[]");
    opened.unwrap_or_else(|err| panic!("empty list did not open: {err}"));
    assert_header_absent(&text, "empty list");
    assert!(
        text.contains("recent:"),
        "empty list dropped the recent marker: {text}"
    );

    let (opened, text) = paint_recent("");
    assert!(opened.is_err(), "blank body was treated as rows: {text}");
    assert_header_absent(&text, "blank body");

    let (opened, text) = paint_recent("not-json");
    assert!(opened.is_err(), "invalid JSON was treated as rows: {text}");
    assert_header_absent(&text, "invalid JSON");

    let (opened, text) = paint_recent("[{}]");
    assert!(
        opened.is_err(),
        "a row missing txid was treated as rows: {text}"
    );
    assert_header_absent(&text, "invalid row");

    let block_id = "blktxid";
    let blocks_body = format!(
        r#"[{{"id":"{block_id}","height":1,"version":1,"timestamp":1,"tx_count":1,"size":20,"weight":80,"merkle_root":"mm","mediantime":1,"difficulty":1.0}}]"#
    );
    let (mut host, rec) = host_with(
        Network::Mainnet,
        BTreeMap::from([("/blocks".to_string(), blocks_body)]),
    );
    host.activate(Screen::Blocks)
        .unwrap_or_else(|err| panic!("blocks list did not open: {err}"));
    let paths = rec.gets();
    assert!(
        paths.iter().any(|path| path == "/blocks"),
        "list did not GET /blocks: {paths:?}"
    );
    assert!(
        paths
            .iter()
            .all(|path| !path.contains('?') && !path.contains('#')),
        "{paths:?}"
    );
    let blocks = painted(&host);
    assert!(
        !blocks.contains("TXID"),
        "blocks list painted TXID: {blocks}"
    );
    assert!(
        !blocks.contains("Amount"),
        "blocks list painted Amount: {blocks}"
    );
    assert!(
        !blocks.contains("vSize"),
        "blocks list painted vSize: {blocks}"
    );
    assert!(
        !blocks.contains("Size"),
        "blocks list painted Size: {blocks}"
    );
    assert!(
        !blocks.contains("Weight"),
        "blocks list painted Weight: {blocks}"
    );
    assert!(blocks.contains("TXs"), "TXs was dropped: {blocks}");
    assert!(
        blocks.contains(block_id) && blocks.contains("merkle_root mm"),
        "the block row was dropped: {blocks}"
    );
    assert!(
        blocks.contains("difficulty 1") && !blocks.contains("difficulty 1.0"),
        "difficulty left plain_number: {blocks}"
    );
    assert!(
        !blocks.contains("Fiat price last updated"),
        "blocks list painted the fiat column: {blocks}"
    );
}

/// Recent transactions fee column header from `GET /mempool/recent`.
///
/// The Angular header cell is `Fee`. One exact line `Fee` when that route
/// parses to at least one row, after `TXID`, `Amount`, and `vSize`. The fee
/// cell stays the raw integer. An empty list, a blank body, and invalid JSON
/// omit the line. Two rows print it once. The blocks list does not gain
/// `Fee`, `TXID`, `Amount`, `vSize`, `Size`, or `Weight`. This is not a
/// loading skeleton.
#[test]
fn recent_transactions_paint_fee_header() {
    const LABEL: &str = "Fee";

    fn summary() -> String {
        r#"{"count":1,"vsize":9,"total_fee":1,"fee_histogram":[]}"#.to_string()
    }

    fn paint_recent(body: &str) -> (Result<(), ClientError>, String) {
        let (mut host, rec) = host_with(
            Network::Mainnet,
            BTreeMap::from([
                ("/mempool".to_string(), summary()),
                ("/mempool/recent".to_string(), body.to_string()),
            ]),
        );
        let opened = host.activate(Screen::Mempool);
        let paths = rec.gets();
        assert!(
            paths.iter().any(|path| path == "/mempool/recent"),
            "did not GET /mempool/recent: {paths:?}"
        );
        assert!(
            paths
                .iter()
                .all(|path| !path.contains('?') && !path.contains('#')),
            "{paths:?}"
        );
        (opened, painted(&host))
    }

    fn assert_header_once_before(painted: &str, txid: &str) {
        assert_eq!(
            painted.lines().filter(|line| *line == LABEL).count(),
            1,
            "{LABEL} was not one exact line: {painted}"
        );
        assert_eq!(
            painted.matches(LABEL).count(),
            1,
            "{LABEL} was not once: {painted}"
        );
        let lines: Vec<&str> = painted.lines().collect();
        let txid_at = lines.iter().position(|line| *line == "TXID");
        let amount_at = lines.iter().position(|line| *line == "Amount");
        let vsize_at = lines.iter().position(|line| *line == "vSize");
        let header_at = lines.iter().position(|line| *line == LABEL);
        let row_at = lines
            .iter()
            .position(|line| line.contains(&format!("recent txid {txid}")));
        assert!(
            txid_at.is_some()
                && amount_at.is_some()
                && vsize_at.is_some()
                && header_at.is_some()
                && row_at.is_some()
                && txid_at < amount_at
                && amount_at < vsize_at
                && vsize_at < header_at
                && header_at < row_at,
            "{LABEL} was not after vSize and before recent txid {txid}: {painted}"
        );
        for kept in ["TXID", "Amount", "vSize"] {
            assert_eq!(
                painted.lines().filter(|line| *line == kept).count(),
                1,
                "{kept} was dropped or repeated: {painted}"
            );
        }
        assert!(
            !painted.contains("skeleton"),
            "the fee header painted a loading skeleton: {painted}"
        );
    }

    fn assert_raw_fee(painted: &str, txid: &str, fee: u64, vsize: u32, value: u64) {
        let row = painted
            .lines()
            .find(|line| line.starts_with("recent txid ") && line.contains(txid))
            .unwrap_or_else(|| panic!("missing recent fee cell for {txid}: {painted}"));
        let fee_text = fee.to_string();
        assert!(
            row.split_whitespace()
                .collect::<Vec<_>>()
                .windows(2)
                .any(|pair| pair == ["fee", fee_text.as_str()]),
            "the fee cell was not the raw integer {fee}: {row}"
        );
        assert!(
            row.contains(&format!("vsize {vsize}")),
            "the existing cell lost vsize: {row}"
        );
        let value_text = value.to_string();
        assert!(
            row.split_whitespace()
                .collect::<Vec<_>>()
                .windows(2)
                .any(|pair| pair == ["value", value_text.as_str()]),
            "the existing cell lost the raw value: {row}"
        );
        assert!(
            !row.contains(',')
                && !row.contains('$')
                && !row.contains('€')
                && !row.contains("BTC")
                && !row.contains("sat/vB")
                && !row.contains("sats")
                && !row.contains(&format!("{fee}.00")),
            "the fee cell did not stay the raw integer: {row}"
        );
    }

    fn assert_header_absent(painted: &str, why: &str) {
        assert!(
            !painted.lines().any(|line| line == LABEL),
            "{why} painted {LABEL}: {painted}"
        );
        assert!(
            !painted.contains(LABEL),
            "{why} contains {LABEL}: {painted}"
        );
        assert!(
            !painted.contains("skeleton"),
            "{why} painted a loading skeleton: {painted}"
        );
    }

    let one = r#"[{"txid":"recent-fee-header","fee":10,"vsize":141,"value":1000}]"#;
    let (opened, text) = paint_recent(one);
    opened.unwrap_or_else(|err| panic!("fee 10 did not open: {err}"));
    assert_header_once_before(&text, "recent-fee-header");
    assert_raw_fee(&text, "recent-fee-header", 10, 141, 1000);

    let zero = r#"[{"txid":"recent-zero-fee","fee":0,"vsize":0,"value":1}]"#;
    let (opened, text) = paint_recent(zero);
    opened.unwrap_or_else(|err| panic!("fee 0 did not open: {err}"));
    assert_header_once_before(&text, "recent-zero-fee");
    assert_raw_fee(&text, "recent-zero-fee", 0, 0, 1);
    assert!(
        text.lines()
            .any(|line| line == "recent txid recent-zero-fee fee 0 vsize 0 value 1"),
        "fee 0 did not stay the raw integer: {text}"
    );

    let two = r#"[{"txid":"recent-fee-one","fee":10,"vsize":141,"value":1},{"txid":"recent-fee-two","fee":1500,"vsize":1000,"value":2}]"#;
    let (opened, text) = paint_recent(two);
    opened.unwrap_or_else(|err| panic!("two rows did not open: {err}"));
    assert_header_once_before(&text, "recent-fee-one");
    assert_raw_fee(&text, "recent-fee-one", 10, 141, 1);
    assert_raw_fee(&text, "recent-fee-two", 1500, 1000, 2);
    assert!(
        !text.contains("1,500") && !text.contains("sat/vB") && !text.contains('$'),
        "two rows did not keep the raw fee integers: {text}"
    );

    let (opened, text) = paint_recent("[]");
    opened.unwrap_or_else(|err| panic!("empty list did not open: {err}"));
    assert_header_absent(&text, "empty list");
    assert!(
        text.contains("recent:"),
        "empty list dropped the recent marker: {text}"
    );

    let (opened, text) = paint_recent("");
    assert!(opened.is_err(), "blank body was treated as rows: {text}");
    assert_header_absent(&text, "blank body");

    let (opened, text) = paint_recent("not-json");
    assert!(opened.is_err(), "invalid JSON was treated as rows: {text}");
    assert_header_absent(&text, "invalid JSON");

    let (opened, text) = paint_recent("[{}]");
    assert!(
        opened.is_err(),
        "a row missing fee was treated as rows: {text}"
    );
    assert_header_absent(&text, "invalid row");

    let block_id = "blkfee";
    let blocks_body = format!(
        r#"[{{"id":"{block_id}","height":1,"version":1,"timestamp":1,"tx_count":1,"size":20,"weight":80,"merkle_root":"mm","mediantime":1,"difficulty":1.0}}]"#
    );
    let (mut host, rec) = host_with(
        Network::Mainnet,
        BTreeMap::from([("/blocks".to_string(), blocks_body)]),
    );
    host.activate(Screen::Blocks)
        .unwrap_or_else(|err| panic!("blocks list did not open: {err}"));
    let paths = rec.gets();
    assert!(
        paths.iter().any(|path| path == "/blocks"),
        "list did not GET /blocks: {paths:?}"
    );
    assert!(
        paths
            .iter()
            .all(|path| !path.contains('?') && !path.contains('#')),
        "{paths:?}"
    );
    let blocks = painted(&host);
    assert!(!blocks.contains("Fee"), "blocks list painted Fee: {blocks}");
    assert!(
        !blocks.contains("TXID"),
        "blocks list painted TXID: {blocks}"
    );
    assert!(
        !blocks.contains("Amount"),
        "blocks list painted Amount: {blocks}"
    );
    assert!(
        !blocks.contains("vSize"),
        "blocks list painted vSize: {blocks}"
    );
    assert!(
        !blocks.contains("Size"),
        "blocks list painted Size: {blocks}"
    );
    assert!(
        !blocks.contains("Weight"),
        "blocks list painted Weight: {blocks}"
    );
    assert!(
        !blocks.contains("skeleton"),
        "blocks list painted a loading skeleton: {blocks}"
    );
    assert!(blocks.contains("TXs"), "TXs was dropped: {blocks}");
    assert!(
        blocks.contains(block_id) && blocks.contains("merkle_root mm"),
        "the block row was dropped: {blocks}"
    );
    assert!(
        blocks.contains("difficulty 1") && !blocks.contains("difficulty 1.0"),
        "difficulty left plain_number: {blocks}"
    );
}

fn loaded_tx(txid: &str) -> String {
    format!(
        r#"{{"txid":"{txid}","version":1,"locktime":0,"vin":[],"vout":[],"size":2,"weight":8,"sigops":0,"fee":4,"status":{{"confirmed":true,"block_height":12}}}}"#
    )
}

fn loaded_txs(txids: &[&str]) -> String {
    let rows = txids
        .iter()
        .map(|txid| loaded_tx(txid))
        .collect::<Vec<_>>()
        .join(",");
    format!("[{rows}]")
}

fn loaded_address_stats(address: &str, chain_count: u64, mempool_count: u64) -> String {
    format!(
        r#"{{"address":"{address}","chain_stats":{{"tx_count":{chain_count},"funded_txo_count":1,"spent_txo_count":0,"funded_txo_sum":10,"spent_txo_sum":0}},"mempool_stats":{{"tx_count":{mempool_count},"funded_txo_count":0,"spent_txo_count":0,"funded_txo_sum":0,"spent_txo_sum":0}}}}"#
    )
}

fn loaded_scripthash_stats(hash: &str, chain_count: u64, mempool_count: u64) -> String {
    format!(
        r#"{{"scripthash":"{hash}","chain_stats":{{"tx_count":{chain_count},"funded_txo_count":1,"spent_txo_count":0,"funded_txo_sum":10,"spent_txo_sum":0}},"mempool_stats":{{"tx_count":{mempool_count},"funded_txo_count":0,"spent_txo_count":0,"funded_txo_sum":0,"spent_txo_sum":0}}}}"#
    )
}

fn assert_first_page_paths(paths: &[String]) {
    for path in paths {
        assert!(!path.contains('?'), "{path}");
        assert!(!path.contains('#'), "{path}");
        assert!(!path.contains("after_txid"), "{path}");
    }
}

fn paint_address_loaded(address: &str, chain_count: u64, mempool_count: u64, txs: &str) -> String {
    let (mut host, rec) = host_with(
        Network::Mainnet,
        BTreeMap::from([
            (
                format!("/address/{address}"),
                loaded_address_stats(address, chain_count, mempool_count),
            ),
            (format!("/address/{address}/txs"), txs.to_string()),
            (format!("/address/{address}/utxo"), "[]".to_string()),
        ]),
    );
    host.open_address(address).expect("address");
    let paths = rec.gets();
    assert!(
        paths
            .iter()
            .any(|path| path == &format!("/address/{address}/txs")),
        "first page was not requested: {paths:?}"
    );
    assert_first_page_paths(&paths);
    painted(&host)
}

fn paint_scripthash_loaded(hash: &str, chain_count: u64, mempool_count: u64, txs: &str) -> String {
    assert_eq!(hash.len(), 64, "{hash}");
    let (mut host, rec) = host_with(
        Network::Mainnet,
        BTreeMap::from([
            (
                format!("/scripthash/{hash}"),
                loaded_scripthash_stats(hash, chain_count, mempool_count),
            ),
            (format!("/scripthash/{hash}/txs"), txs.to_string()),
            (format!("/scripthash/{hash}/utxo"), "[]".to_string()),
        ]),
    );
    host.open_address(hash).expect("scripthash");
    let paths = rec.gets();
    assert!(
        paths
            .iter()
            .any(|path| path == &format!("/scripthash/{hash}/txs")),
        "first page was not requested: {paths:?}"
    );
    assert!(
        paths.iter().all(|path| !path.starts_with("/address/")),
        "scripthash used the address route: {paths:?}"
    );
    assert_first_page_paths(&paths);
    painted(&host)
}

fn assert_exact_line_once(paint: &str, line: &str) {
    assert_eq!(
        paint.lines().filter(|row| *row == line).count(),
        1,
        "expected one exact line {line:?}\n{paint}"
    );
}

fn assert_heading_above_list(paint: &str, heading: &str) {
    let heading_at = paint
        .lines()
        .position(|line| line == heading)
        .unwrap_or_else(|| panic!("heading missing: {heading:?}\n{paint}"));
    let list_at = paint
        .lines()
        .position(|line| line.starts_with("address tx txid "))
        .unwrap_or_else(|| panic!("loaded transaction list missing\n{paint}"));
    assert!(
        heading_at < list_at,
        "heading is not above the loaded list\n{paint}"
    );
}

fn assert_transactions_sum_stays(paint: &str, sum: i128) {
    let label = format!("Transactions {sum}");
    assert!(
        paint
            .lines()
            .any(|line| line.contains(&label) && !line.contains(" of ")),
        "Transactions-plus-sum line was replaced\n{paint}"
    );
}

fn assert_empty_has_no_count_line(paint: &str) {
    assert!(
        paint.lines().any(|line| line == "\u{00A0}"),
        "empty loaded list did not leave a non-breaking space\n{paint}"
    );
    assert!(
        !paint.lines().any(|line| {
            let words: Vec<&str> = line.split_whitespace().collect();
            words.len() == 4
                && words[1] == "of"
                && (words[3] == "transaction" || words[3] == "transactions")
        }),
        "empty loaded list painted a count\n{paint}"
    );
    assert!(
        !paint.contains("0 of "),
        "empty loaded list painted a fake zero count\n{paint}"
    );
}

/// Address h2 above the loaded transaction list.
///
/// One loaded transaction is `{loaded} of {chain tx_count + mempool tx_count} transaction`.
/// More than one uses `transactions`. An empty loaded list is a non-breaking space
/// and does not get that line. Address and scripthash share this screen.
/// `Transactions {sum}` stays. The first page is `GET /address/:addr/txs` or
/// `GET /scripthash/:hash/txs` with no query string and no `after_txid`.
#[test]
fn address_screen_paints_loaded_transactions_of() {
    const CHAIN: u64 = 4;
    const MEMPOOL: u64 = 3;
    const SUM: i128 = 7;

    let one = paint_address_loaded("bc1qloadedone", CHAIN, MEMPOOL, &loaded_txs(&["loadedtx1"]));
    assert_exact_line_once(&one, "1 of 7 transaction");
    assert!(
        !one.lines().any(|line| line == "1 of 7 transactions"),
        "one loaded transaction used the plural\n{one}"
    );
    assert!(
        !one.lines()
            .any(|line| line == "1 of 4 transaction" || line == "1 of 3 transaction"),
        "the heading used one tx_count instead of the sum\n{one}"
    );
    assert_heading_above_list(&one, "1 of 7 transaction");
    assert_transactions_sum_stays(&one, SUM);
    assert!(one.contains("address tx txid loadedtx1"), "{one}");

    let many = paint_address_loaded(
        "bc1qloadedmany",
        CHAIN,
        MEMPOOL,
        &loaded_txs(&["loadedtx2", "loadedtx3"]),
    );
    assert_exact_line_once(&many, "2 of 7 transactions");
    assert!(
        !many.lines().any(|line| line == "2 of 7 transaction"),
        "more than one loaded transaction used the singular\n{many}"
    );
    assert_heading_above_list(&many, "2 of 7 transactions");
    assert_transactions_sum_stays(&many, SUM);

    let empty = paint_address_loaded("bc1qloadednone", CHAIN, MEMPOOL, "[]");
    assert_empty_has_no_count_line(&empty);
    assert_transactions_sum_stays(&empty, SUM);
    let nbsp_at = empty
        .lines()
        .position(|line| line == "\u{00A0}")
        .expect("nbsp");
    let list_at = empty
        .lines()
        .position(|line| line == "The address transaction list is empty.")
        .expect("empty list");
    assert!(
        nbsp_at < list_at,
        "non-breaking space is not above the list\n{empty}"
    );

    let hash_one = "c1".repeat(32);
    let script_one =
        paint_scripthash_loaded(&hash_one, CHAIN, MEMPOOL, &loaded_txs(&["scripthash1"]));
    assert_exact_line_once(&script_one, "1 of 7 transaction");
    assert_heading_above_list(&script_one, "1 of 7 transaction");
    assert_transactions_sum_stays(&script_one, SUM);

    let hash_many = "c2".repeat(32);
    let script_many = paint_scripthash_loaded(
        &hash_many,
        CHAIN,
        MEMPOOL,
        &loaded_txs(&["scripthash2", "scripthash3"]),
    );
    assert_exact_line_once(&script_many, "2 of 7 transactions");
    assert_transactions_sum_stays(&script_many, SUM);

    let hash_empty = "c3".repeat(32);
    let script_empty = paint_scripthash_loaded(&hash_empty, CHAIN, MEMPOOL, "[]");
    assert_empty_has_no_count_line(&script_empty);
    assert_transactions_sum_stays(&script_empty, SUM);
}

/// Transaction page badge from a no-query `GET /tx/:txid`.
///
/// `tx-features.component.html` line 2 is the word `SegWit` when segwit is
/// enabled and the realized weight savings are nonzero and the potential
/// savings are zero. The word stays off when that gain is zero or segwit is
/// off. The tooltip stays off. The RBF badge stays off. The line
/// `confirmations 1` stays. `P2WPKH` is the address badge, not this line.
/// `OP_RETURN` stays. This fixture does not paint `Runestone`.
#[test]
fn transaction_screen_paints_segwit_badge() {
    const WIT: &str = "aabb";
    const OP_RETURN_SCRIPT: &str = "6a046869";

    fn vin(witness: bool) -> String {
        let witness_field = if witness {
            format!(r#","witness":["{WIT}"]"#)
        } else {
            String::new()
        };
        format!(
            r#"{{"txid":"prev","vout":0,"prevout":{{"scriptpubkey":"0014aa","scriptpubkey_asm":"OP_0 OP_PUSHBYTES_20 aa","scriptpubkey_type":"v0_p2wpkh","value":5}},"scriptsig":"","scriptsig_asm":"","is_coinbase":false,"sequence":4294967295{witness_field}}}"#
        )
    }

    fn tx_body(txid: &str, height: u64, witness: bool) -> String {
        format!(
            r#"{{"txid":"{txid}","version":1,"locktime":0,"vin":[{}],"vout":[{{"scriptpubkey":"{OP_RETURN_SCRIPT}","scriptpubkey_asm":"OP_RETURN OP_PUSHBYTES_4","scriptpubkey_type":"op_return","value":0}}],"size":10,"weight":400,"sigops":0,"fee":1,"status":{{"confirmed":true,"block_height":{height}}}}}"#,
            vin(witness)
        )
    }

    fn paint(txid: &str, height: u64, witness: bool) -> String {
        let (mut host, rec) = host_with(
            Network::Mainnet,
            BTreeMap::from([
                (format!("/tx/{txid}"), tx_body(txid, height, witness)),
                ("/blocks/tip/height".to_string(), height.to_string()),
            ]),
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

    fn assert_badge_neighbors(painted: &str) {
        assert!(
            painted.lines().any(|line| line == "confirmations 1"),
            "confirmations 1 left the screen: {painted}"
        );
        assert_eq!(
            painted.lines().filter(|line| *line == "OP_RETURN").count(),
            1,
            "OP_RETURN was dropped or repeated: {painted}"
        );
        assert!(
            !painted.lines().any(|line| line == "Runestone"),
            "this fixture painted Runestone: {painted}"
        );
        assert!(
            painted
                .lines()
                .any(|line| line == "Previous output type V0_P2WPKH"),
            "Previous output type V0_P2WPKH was dropped: {painted}"
        );
        assert!(
            !painted
                .lines()
                .any(|line| line == "P2WPKH" || line == "Type P2WPKH"),
            "the address badge was painted on the transaction page: {painted}"
        );
        assert!(
            !painted.lines().any(|line| line == "RBF"),
            "the RBF badge was painted: {painted}"
        );
        assert!(
            !painted.contains("Replace-By-Fee") && !painted.contains("on fees"),
            "a badge tooltip was painted: {painted}"
        );
        assert!(
            !painted.contains("native SegWit"),
            "the SegWit tooltip was painted: {painted}"
        );
    }

    let on = paint("segwiton", 800_000, true);
    assert!(
        on.lines().any(|line| line == "SegWit"),
        "SegWit is absent: {on}"
    );
    assert_eq!(
        on.lines().filter(|line| *line == "SegWit").count(),
        1,
        "SegWit was not painted once: {on}"
    );
    assert_badge_neighbors(&on);

    let zero = paint("segwitzero", 800_000, false);
    assert!(
        zero.contains("tx txid segwitzero"),
        "a zero gain hid the transaction: {zero}"
    );
    assert!(
        !zero.contains("SegWit"),
        "a zero gain painted SegWit: {zero}"
    );
    assert_badge_neighbors(&zero);

    let off = paint("segwitoff", 477_119, true);
    assert!(
        off.contains("tx txid segwitoff"),
        "segwit off hid the transaction: {off}"
    );
    assert!(!off.contains("SegWit"), "segwit off painted SegWit: {off}");
    assert_badge_neighbors(&off);
}

/// Transaction page badge from a no-query `GET /tx/:txid`.
///
/// `tx-features.component.html` lines 11-25 are the word `Taproot` when
/// taproot is enabled. Confirmed mainnet is on at height 709632. The word
/// stays off below that height. An unconfirmed transaction is on. Liquid
/// hides the features row. The tooltip stays off. The RBF badge stays off.
/// The line `confirmations 1` stays. `SegWit` is a different badge. `P2WPKH`
/// is the address badge, not this line. `Runestone` stays. Script hex, script
/// asm, scriptsig, and witness stay off the page.
#[test]
fn transaction_screen_paints_taproot_badge() {
    const WIT: &str = "taprootwit";
    const PREV_SCRIPT: &str = "0014taprootprev";
    const PREV_ASM: &str = "taprootprevasm";
    const STONE_SCRIPT: &str = "6a5dtaprootstone";
    const STONE_ASM: &str = "taprootstoneasm";
    const SIG: &str = "taprootsig";
    const SIG_ASM: &str = "taprootsigasm";

    fn vin(witness: bool) -> String {
        let witness_field = if witness {
            format!(r#","witness":["{WIT}"]"#)
        } else {
            String::new()
        };
        format!(
            r#"{{"txid":"prev","vout":0,"prevout":{{"scriptpubkey":"{PREV_SCRIPT}","scriptpubkey_asm":"{PREV_ASM}","scriptpubkey_type":"v0_p2wpkh","value":5}},"scriptsig":"{SIG}","scriptsig_asm":"{SIG_ASM}","is_coinbase":false,"sequence":4294967295{witness_field}}}"#
        )
    }

    fn tx_body(txid: &str, status: &str, witness: bool) -> String {
        format!(
            r#"{{"txid":"{txid}","version":1,"locktime":0,"vin":[{}],"vout":[{{"scriptpubkey":"{STONE_SCRIPT}","scriptpubkey_asm":"{STONE_ASM}","scriptpubkey_type":"op_return","value":0}}],"size":10,"weight":400,"sigops":0,"fee":1,"status":{status}}}"#,
            vin(witness)
        )
    }

    fn paint(
        network: Network,
        txid: &str,
        status: &str,
        witness: bool,
        tip: Option<u64>,
    ) -> String {
        let mut bodies = BTreeMap::from([(format!("/tx/{txid}"), tx_body(txid, status, witness))]);
        if let Some(height) = tip {
            bodies.insert("/blocks/tip/height".to_string(), height.to_string());
        }
        let (mut host, rec) = host_with(network, bodies);
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

    fn assert_script_stays_off(text: &str) {
        for needle in [
            WIT,
            PREV_SCRIPT,
            PREV_ASM,
            STONE_SCRIPT,
            STONE_ASM,
            SIG,
            SIG_ASM,
        ] {
            assert!(!text.contains(needle), "{needle} was painted: {text}");
        }
    }

    fn assert_badge_neighbors(painted: &str) {
        assert!(
            painted.lines().any(|line| line == "confirmations 1"),
            "confirmations 1 left the screen: {painted}"
        );
        assert_eq!(
            painted.lines().filter(|line| *line == "OP_RETURN").count(),
            1,
            "OP_RETURN was dropped or repeated: {painted}"
        );
        assert_eq!(
            painted.lines().filter(|line| *line == "Runestone").count(),
            1,
            "Runestone was dropped or repeated: {painted}"
        );
        assert!(
            painted
                .lines()
                .any(|line| line == "Previous output type V0_P2WPKH"),
            "Previous output type V0_P2WPKH was dropped: {painted}"
        );
        assert!(
            !painted
                .lines()
                .any(|line| line == "P2WPKH" || line == "Type P2WPKH"),
            "the address badge was painted on the transaction page: {painted}"
        );
        assert!(
            !painted.lines().any(|line| line == "RBF"),
            "the RBF badge was painted: {painted}"
        );
        assert!(
            !painted.contains("Replace-By-Fee")
                && !painted.contains("on fees")
                && !painted.contains("uses Taproot")
                && !painted.contains("does not use Taproot")
                && !painted.contains("native SegWit"),
            "a badge tooltip was painted: {painted}"
        );
        assert_script_stays_off(painted);
    }

    let on = paint(
        Network::Mainnet,
        "taprooton",
        r#"{"confirmed":true,"block_height":800000}"#,
        true,
        Some(800_000),
    );
    assert!(
        on.lines().any(|line| line == "Taproot"),
        "Taproot is absent: {on}"
    );
    assert_eq!(
        on.lines().filter(|line| *line == "Taproot").count(),
        1,
        "Taproot was not painted once: {on}"
    );
    assert!(
        on.lines().any(|line| line == "SegWit"),
        "SegWit left the screen: {on}"
    );
    assert_badge_neighbors(&on);

    let off = paint(
        Network::Mainnet,
        "taprootoff",
        r#"{"confirmed":true,"block_height":709631}"#,
        true,
        Some(709_631),
    );
    assert!(
        off.contains("tx txid taprootoff"),
        "taproot off hid the transaction: {off}"
    );
    assert!(
        !off.contains("Taproot"),
        "taproot off painted Taproot: {off}"
    );
    assert!(
        off.lines().any(|line| line == "SegWit"),
        "SegWit left the screen when taproot was off: {off}"
    );
    assert_badge_neighbors(&off);

    let pending = paint(
        Network::Mainnet,
        "taprootpending",
        r#"{"confirmed":false}"#,
        true,
        Some(800_000),
    );
    assert!(
        pending.contains("tx txid taprootpending"),
        "an unconfirmed transaction was hidden: {pending}"
    );
    assert_eq!(
        pending.lines().filter(|line| *line == "Taproot").count(),
        1,
        "an unconfirmed transaction did not paint Taproot once: {pending}"
    );
    assert!(
        pending.lines().any(|line| line == "Unconfirmed"),
        "the unconfirmed line left the screen: {pending}"
    );
    assert!(
        !pending.lines().any(|line| line == "RBF"),
        "the RBF badge was painted: {pending}"
    );
    assert!(
        pending.lines().any(|line| line == "Runestone"),
        "Runestone left the unconfirmed screen: {pending}"
    );
    assert_script_stays_off(&pending);
    assert!(
        !pending.contains("on fees") && !pending.contains("uses Taproot"),
        "a badge tooltip was painted: {pending}"
    );

    let liquid = paint(
        Network::Liquid,
        "taprootliquid",
        r#"{"confirmed":false}"#,
        true,
        Some(1),
    );
    assert!(
        liquid.contains("tx txid taprootliquid"),
        "liquid hid the transaction: {liquid}"
    );
    assert!(
        !liquid.contains("Taproot"),
        "liquid painted Taproot: {liquid}"
    );
    assert!(
        liquid.lines().any(|line| line == "Runestone"),
        "Runestone left the liquid screen: {liquid}"
    );
    assert_script_stays_off(&liquid);
}

/// Transaction input warning from a no-query `GET /tx/:txid`.
///
/// A `v0_p2wsh` input paints the line `Lightning Force Close` when
/// `inner_witnessscript_asm` matches the commitment-output pattern in
/// `script.utils.ts` and the witness item under the last item is not the
/// one-byte revocation marker `01`. The delayed to_local case uses an empty
/// stack item and paints the line. The stored `01` case does not paint this
/// line and does not paint `Revoked Lightning Force Close`. A witness script
/// that is not the commitment pattern does not paint it. The asm, the witness,
/// and the script hex stay off the screen. This is not a Lightning route.
/// `Multisig 2 of 3`, `Type P2SH-P2WPKH`, and `Type P2SH-P2WSH` are other
/// labels. The output type line stays `Type MULTISIG`.
const FORCE_CLOSE_TXID: &str = "b10c0000004da5a9d1d9b4ae32e09f0b3e62d21a5cce5428d4ad714fb444eb5d";
const FORCE_CLOSE_PREV: &str = "637db3928a8fb1b22b81f92dc738ee7637e5b172d650363d0b327429578bd001";
const FORCE_CLOSE_ADDRESS: &str = "bc1q49fs59nletdxwtq59mnrdhx3w9uku6003cm658mh7dw93mwh5dts2w2kht";
const FORCE_CLOSE_SCRIPT_HEX: &str =
    "0020a9530a167fcada672c142ee636dcd171796e69ef8e37aa1f77f35c58edd7a357";
const FORCE_CLOSE_SCRIPT_ASM: &str =
    "OP_0 OP_PUSHBYTES_32 a9530a167fcada672c142ee636dcd171796e69ef8e37aa1f77f35c58edd7a357";
const FORCE_CLOSE_SIG: &str = "303d021c32f9454db85cb1a4ca63a9883d4347c5e13f3654e884ae44e9efa3c8021d62f07fe452c06b084bc3e09afd3aac4039136549a465533bc1ca66967902";
const FORCE_CLOSE_WITNESS_SCRIPT: &str = "632102fd6db4de50399b2aa086edb23f8e140bbc823d6651e024a0eb871288068789cd67012ab27521034134a2bb35c3f83dab2489d96160741888b8b5589bb694dea6e7bc24486e9c6f68ac";
const FORCE_CLOSE_COMMITMENT_ASM: &str = "OP_IF OP_PUSHBYTES_33 02fd6db4de50399b2aa086edb23f8e140bbc823d6651e024a0eb871288068789cd OP_ELSE OP_PUSHBYTES_1 2a OP_CSV OP_DROP OP_PUSHBYTES_33 034134a2bb35c3f83dab2489d96160741888b8b5589bb694dea6e7bc24486e9c6f OP_ENDIF OP_CHECKSIG";
const FORCE_CLOSE_OTHER_ASM: &str = "OP_DUP OP_HASH160 OP_PUSHBYTES_20 db865fd920959506111079995f1e4017b489bfe3 OP_EQUAL OP_IF OP_CHECKSIG OP_ELSE OP_PUSHBYTES_33 024d560f7f5d28aae5e1a8aa2b7ba615d7fc48e4ea27e5d27336e6a8f5fa0f5c8c OP_SWAP OP_SIZE OP_PUSHBYTES_1 20 OP_EQUAL OP_NOTIF OP_DROP OP_PUSHNUM_2 OP_SWAP OP_PUSHBYTES_33 03443e8834fa7d79d7b5e95e0e9d0847f6b03ac3ea977979858b4104947fca87ca OP_PUSHNUM_2 OP_CHECKMULTISIG OP_ELSE OP_HASH160 OP_PUSHBYTES_20 46c3747322b220fdb925c9802f0e949c1feab999 OP_EQUALVERIFY OP_CHECKSIG OP_ENDIF OP_ENDIF";
const FORCE_CLOSE_OTHER_WITNESS: &str = r#"["303f021c65aee6696e80be6e14545cfd64b44f17b0514c150eefdb090c0f0bd9021f3fef4aa95c252a225622aba99e4d5af5a6fe40d177acd593e64cf2f8557ccc03","03b55c6f0749e0f3e2caeca05f68e3699f1b3c62a550730f704985a6a9aae437a1","76a914db865fd920959506111079995f1e4017b489bfe38763ac6721024d560f7f5d28aae5e1a8aa2b7ba615d7fc48e4ea27e5d27336e6a8f5fa0f5c8c7c820120876475527c2103443e8834fa7d79d7b5e95e0e9d0847f6b03ac3ea977979858b4104947fca87ca52ae67a91446c3747322b220fdb925c9802f0e949c1feab99988ac6868"]"#;
const FORCE_CLOSE_OUT_HEX: &str =
    "5121028b45a50f795be0413680036665d17a3eca099648ea80637bc3a70a7d2b52ae2851ae";
const FORCE_CLOSE_OUT_ASM: &str = "OP_PUSHNUM_1 OP_PUSHBYTES_33 028b45a50f795be0413680036665d17a3eca099648ea80637bc3a70a7d2b52ae28 OP_PUSHNUM_1 OP_CHECKMULTISIG";
const FORCE_CLOSE_LINE: &str = "Lightning Force Close";
const FORCE_CLOSE_REVOKED: &str = "Revoked Lightning Force Close";

fn force_close_witness(under_last: &str) -> String {
    format!(r#"["{FORCE_CLOSE_SIG}","{under_last}","{FORCE_CLOSE_WITNESS_SCRIPT}"]"#)
}

fn force_close_vin(witness: &str, witness_asm: &str) -> String {
    format!(
        r#"{{"txid":"{FORCE_CLOSE_PREV}","vout":0,"prevout":{{"scriptpubkey":"{FORCE_CLOSE_SCRIPT_HEX}","scriptpubkey_asm":"{FORCE_CLOSE_SCRIPT_ASM}","scriptpubkey_type":"v0_p2wsh","scriptpubkey_address":"{FORCE_CLOSE_ADDRESS}","value":8149}},"scriptsig":"","scriptsig_asm":"","witness":{witness},"is_coinbase":false,"sequence":4190024921,"inner_witnessscript_asm":"{witness_asm}"}}"#
    )
}

fn force_close_tx(vin_json: &str) -> String {
    format!(
        r#"{{"txid":"{FORCE_CLOSE_TXID}","version":1,"locktime":1231006505,"vin":[{vin_json}],"vout":[{{"scriptpubkey":"{FORCE_CLOSE_OUT_HEX}","scriptpubkey_asm":"{FORCE_CLOSE_OUT_ASM}","scriptpubkey_type":"multisig","value":582}}],"size":3500,"weight":8186,"sigops":115,"fee":71294}}"#
    )
}

fn paint_force_close(witness: &str, witness_asm: &str) -> (Vec<String>, String) {
    let (mut host, rec) = host_with(
        Network::Mainnet,
        BTreeMap::from([(
            format!("/tx/{FORCE_CLOSE_TXID}"),
            force_close_tx(&force_close_vin(witness, witness_asm)),
        )]),
    );
    host.open_transaction(FORCE_CLOSE_TXID)
        .unwrap_or_else(|err| panic!("tx {FORCE_CLOSE_TXID} did not open: {err}"));
    (rec.gets(), painted(&host))
}

fn assert_force_close_request(paths: &[String]) {
    assert!(
        paths
            .iter()
            .any(|path| path == &format!("/tx/{FORCE_CLOSE_TXID}")),
        "no-query GET /tx/:txid is missing: {paths:?}"
    );
    assert!(
        paths
            .iter()
            .all(|path| !path.contains('?') && !path.contains('#')),
        "a transaction open used a query or a fragment: {paths:?}"
    );
    assert!(
        paths.iter().all(|path| !path.contains("/lightning")),
        "a transaction open added a Lightning route: {paths:?}"
    );
}

fn assert_force_close_script_off(text: &str) {
    for needle in [
        FORCE_CLOSE_COMMITMENT_ASM,
        FORCE_CLOSE_OTHER_ASM,
        FORCE_CLOSE_SCRIPT_HEX,
        FORCE_CLOSE_SCRIPT_ASM,
        FORCE_CLOSE_SIG,
        FORCE_CLOSE_WITNESS_SCRIPT,
        FORCE_CLOSE_OUT_HEX,
        FORCE_CLOSE_OUT_ASM,
        "303f021c65aee6696e80be6e14545cfd64b44f17b0514c150eefdb090c0f0bd9021f3fef4aa95c252a225622aba99e4d5af5a6fe40d177acd593e64cf2f8557ccc03",
        "03b55c6f0749e0f3e2caeca05f68e3699f1b3c62a550730f704985a6a9aae437a1",
        "76a914db865fd920959506111079995f1e4017b489bfe38763ac6721024d560f7f5d28aae5e1a8aa2b7ba615d7fc48e4ea27e5d27336e6a8f5fa0f5c8c7c820120876475527c2103443e8834fa7d79d7b5e95e0e9d0847f6b03ac3ea977979858b4104947fca87ca52ae67a91446c3747322b220fdb925c9802f0e949c1feab99988ac6868",
        "title=",
        "tooltip",
        "/lightning",
        FORCE_CLOSE_REVOKED,
    ] {
        assert!(
            !text.contains(needle),
            "{needle} leaked into the screen: {text}"
        );
    }
}

fn assert_force_close_neighbors(text: &str) {
    assert!(
        text.lines().any(|line| line == INPUTS_OUTPUTS_HEADING),
        "Inputs & Outputs left the screen: {text}"
    );
    assert_eq!(
        text.lines().filter(|line| *line == "Type MULTISIG").count(),
        1,
        "the output type line Type MULTISIG was replaced: {text}"
    );
    assert!(
        text.lines()
            .any(|line| line == "Previous output type V0_P2WSH"),
        "Previous output type V0_P2WSH was replaced: {text}"
    );
    assert!(
        text.lines()
            .any(|line| line == "input script type v0_p2wsh"),
        "input script type v0_p2wsh was replaced: {text}"
    );
    assert!(
        !text.lines().any(|line| line == "Multisig 2 of 3"),
        "this commitment spend painted Multisig 2 of 3: {text}"
    );
    assert!(
        !text
            .lines()
            .any(|line| line == "Type P2SH-P2WPKH" || line == "Type P2SH-P2WSH"),
        "an address type badge was painted on this transaction: {text}"
    );
}

#[test]
fn transaction_screen_paints_lightning_force_close() {
    assert!(
        FORCE_CLOSE_COMMITMENT_ASM.starts_with("OP_IF OP_PUSHBYTES_33 "),
        "the commitment witness script was replaced"
    );
    assert!(
        FORCE_CLOSE_COMMITMENT_ASM.ends_with(" OP_ENDIF OP_CHECKSIG"),
        "the commitment witness script was replaced"
    );
    assert!(
        !FORCE_CLOSE_OTHER_ASM.starts_with("OP_IF OP_PUSHBYTES_33 "),
        "the non-commitment witness script was replaced"
    );

    let (paths, text) = paint_force_close(&force_close_witness(""), FORCE_CLOSE_COMMITMENT_ASM);
    assert_force_close_request(&paths);
    assert!(
        text.contains(&format!("tx txid {FORCE_CLOSE_TXID}")),
        "the parsed transaction was hidden: {text}"
    );
    assert!(
        text.lines().any(|line| line == FORCE_CLOSE_LINE),
        "the screen did not show Lightning Force Close: {text}"
    );
    assert_eq!(
        text.lines()
            .filter(|line| *line == FORCE_CLOSE_LINE)
            .count(),
        1,
        "Lightning Force Close was not painted once: {text}"
    );
    let rows: Vec<&str> = text.lines().collect();
    let outpoint = rows
        .iter()
        .position(|line| *line == format!("input outpoint {FORCE_CLOSE_PREV}:0"));
    let warning = rows.iter().position(|line| *line == FORCE_CLOSE_LINE);
    let output = rows
        .iter()
        .position(|line| line.starts_with("output address "));
    assert!(
        outpoint.is_some() && warning.is_some() && output.is_some(),
        "the input or the output left the screen: {text}"
    );
    assert!(
        outpoint.unwrap() < warning.unwrap() && warning.unwrap() < output.unwrap(),
        "Lightning Force Close was not on the transaction input: {text}"
    );
    assert_force_close_neighbors(&text);
    assert_force_close_script_off(&text);

    let (revoked_paths, revoked) =
        paint_force_close(&force_close_witness("01"), FORCE_CLOSE_COMMITMENT_ASM);
    assert_force_close_request(&revoked_paths);
    assert!(
        revoked.contains(&format!("tx txid {FORCE_CLOSE_TXID}")),
        "the stored 01 case hid the transaction: {revoked}"
    );
    assert!(
        !revoked.lines().any(|line| line == FORCE_CLOSE_LINE),
        "the stored 01 case painted Lightning Force Close: {revoked}"
    );
    assert!(
        !revoked.contains(FORCE_CLOSE_REVOKED),
        "the stored 01 case painted Revoked Lightning Force Close: {revoked}"
    );
    assert_force_close_neighbors(&revoked);
    assert_force_close_script_off(&revoked);

    let (other_paths, other) = paint_force_close(FORCE_CLOSE_OTHER_WITNESS, FORCE_CLOSE_OTHER_ASM);
    assert_force_close_request(&other_paths);
    assert!(
        other.contains(&format!("tx txid {FORCE_CLOSE_TXID}")),
        "a non-commitment witness script hid the transaction: {other}"
    );
    assert!(
        !other.lines().any(|line| line == FORCE_CLOSE_LINE),
        "a non-commitment witness script painted Lightning Force Close: {other}"
    );
    assert!(
        !other.contains(FORCE_CLOSE_REVOKED),
        "a non-commitment witness script painted Revoked Lightning Force Close: {other}"
    );
    assert_force_close_neighbors(&other);
    assert_force_close_script_off(&other);
}

/// Transaction input warning from a no-query `GET /tx/:txid`.
///
/// A `v0_p2wsh` input paints the line `Lightning HTLC` when
/// `inner_witnessscript_asm` matches the offered or received HTLC pattern
/// in `script.utils.ts` and the witness item under the last item is
/// non-empty and its length is not 66. A 66-character item is the revoked
/// case. An empty item is the expired case. Those sentences stay off this
/// screen, and so does `Revoked Lightning Force Close`. The stored offered
/// script whose middle witness item is 66 hex characters does not paint
/// `Lightning HTLC`. The asm, the witness, and the script hex stay off the
/// screen. This is not a Lightning route.
const HTLC_LINE: &str = "Lightning HTLC";
const HTLC_REVOKED: &str = "Revoked Lightning HTLC";
const HTLC_EXPIRED: &str = "Expired Lightning HTLC";
/// Payment preimage, 32 bytes. Non-empty, and not 66 hex characters.
const HTLC_PREIMAGE: &str = "00112233445566778899aabbccddeeff00112233445566778899aabbccddeeff";
/// Same stored offered spend, the item under the witness script. 33 bytes.
const HTLC_REVOKED_ITEM: &str =
    "03b55c6f0749e0f3e2caeca05f68e3699f1b3c62a550730f704985a6a9aae437a1";
const HTLC_SIG: &str = "303f021c65aee6696e80be6e14545cfd64b44f17b0514c150eefdb090c0f0bd9021f3fef4aa95c252a225622aba99e4d5af5a6fe40d177acd593e64cf2f8557ccc03";
const HTLC_SCRIPT: &str = "76a914db865fd920959506111079995f1e4017b489bfe38763ac6721024d560f7f5d28aae5e1a8aa2b7ba615d7fc48e4ea27e5d27336e6a8f5fa0f5c8c7c820120876475527c2103443e8834fa7d79d7b5e95e0e9d0847f6b03ac3ea977979858b4104947fca87ca52ae67a91446c3747322b220fdb925c9802f0e949c1feab99988ac6868";
/// `detectScriptTemplate` received HTLC, same hashes, plus a 3-byte CLTV push.
const HTLC_RECEIVED_ASM: &str = "OP_DUP OP_HASH160 OP_PUSHBYTES_20 db865fd920959506111079995f1e4017b489bfe3 OP_EQUAL OP_IF OP_CHECKSIG OP_ELSE OP_PUSHBYTES_33 024d560f7f5d28aae5e1a8aa2b7ba615d7fc48e4ea27e5d27336e6a8f5fa0f5c8c OP_SWAP OP_SIZE OP_PUSHBYTES_1 20 OP_EQUAL OP_IF OP_HASH160 OP_PUSHBYTES_20 46c3747322b220fdb925c9802f0e949c1feab999 OP_EQUALVERIFY OP_PUSHNUM_2 OP_SWAP OP_PUSHBYTES_33 03443e8834fa7d79d7b5e95e0e9d0847f6b03ac3ea977979858b4104947fca87ca OP_PUSHNUM_2 OP_CHECKMULTISIG OP_ELSE OP_DROP OP_PUSHBYTES_3 80b12a OP_CLTV OP_DROP OP_CHECKSIG OP_ENDIF OP_ENDIF";

fn htlc_witness(under_last: &str) -> String {
    format!(r#"["{HTLC_SIG}","{under_last}","{HTLC_SCRIPT}"]"#)
}

fn assert_htlc_sentences_off(text: &str) {
    for needle in [HTLC_REVOKED, HTLC_EXPIRED, FORCE_CLOSE_REVOKED] {
        assert!(
            !text.contains(needle),
            "{needle} leaked into the screen: {text}"
        );
    }
}

fn assert_paints_exact_htlc(text: &str) {
    assert!(
        text.lines().any(|line| line == HTLC_LINE),
        "the screen did not show Lightning HTLC: {text}"
    );
    assert_eq!(
        text.lines().filter(|line| *line == HTLC_LINE).count(),
        1,
        "Lightning HTLC was not painted once: {text}"
    );
    assert!(
        text.lines()
            .filter(|line| line.contains(HTLC_LINE))
            .all(|line| line == HTLC_LINE),
        "Lightning HTLC was painted inside a longer sentence: {text}"
    );
    let rows: Vec<&str> = text.lines().collect();
    let outpoint = rows
        .iter()
        .position(|line| *line == format!("input outpoint {FORCE_CLOSE_PREV}:0"));
    let warning = rows.iter().position(|line| *line == HTLC_LINE);
    let output = rows
        .iter()
        .position(|line| line.starts_with("output address "));
    assert!(
        outpoint.is_some() && warning.is_some() && output.is_some(),
        "the input or the output left the screen: {text}"
    );
    assert!(
        outpoint.unwrap() < warning.unwrap() && warning.unwrap() < output.unwrap(),
        "Lightning HTLC was not on the transaction input: {text}"
    );
    assert!(
        !text.lines().any(|line| line == FORCE_CLOSE_LINE),
        "this HTLC spend painted Lightning Force Close: {text}"
    );
    assert_htlc_sentences_off(text);
    assert_force_close_neighbors(text);
    assert_force_close_script_off(text);
    assert!(
        !text.contains(HTLC_PREIMAGE),
        "the preimage leaked into the screen: {text}"
    );
}

fn assert_no_htlc_line(text: &str) {
    assert!(
        !text.lines().any(|line| line == HTLC_LINE),
        "the screen painted Lightning HTLC: {text}"
    );
    assert!(
        text.lines().all(|line| !line.contains(HTLC_LINE)),
        "Lightning HTLC was painted inside a longer sentence: {text}"
    );
    assert_htlc_sentences_off(text);
    assert_force_close_neighbors(text);
    assert_force_close_script_off(text);
}

#[test]
fn transaction_screen_paints_lightning_htlc() {
    assert_eq!(
        HTLC_PREIMAGE.len(),
        64,
        "the preimage must not be 66 characters"
    );
    assert!(!HTLC_PREIMAGE.is_empty(), "the preimage must be non-empty");
    assert_eq!(
        HTLC_REVOKED_ITEM.len(),
        66,
        "the stored revoked item was replaced"
    );
    assert!(
        FORCE_CLOSE_OTHER_ASM.contains(" OP_NOTIF OP_DROP OP_PUSHNUM_2 "),
        "the stored offered HTLC witness script was replaced"
    );
    assert!(
        HTLC_RECEIVED_ASM.contains(" OP_EQUAL OP_IF OP_HASH160 OP_PUSHBYTES_20 "),
        "the received HTLC witness script was replaced"
    );
    assert!(
        !HTLC_RECEIVED_ASM.contains(" OP_NOTIF "),
        "the received HTLC witness script was replaced"
    );
    assert_eq!(
        FORCE_CLOSE_OTHER_WITNESS,
        htlc_witness(HTLC_REVOKED_ITEM),
        "the stored 66-character stack was replaced"
    );

    let (paths, offered) = paint_force_close(&htlc_witness(HTLC_PREIMAGE), FORCE_CLOSE_OTHER_ASM);
    assert_force_close_request(&paths);
    assert!(
        offered.contains(&format!("tx txid {FORCE_CLOSE_TXID}")),
        "the parsed transaction was hidden: {offered}"
    );
    assert_paints_exact_htlc(&offered);

    let (received_paths, received) =
        paint_force_close(&htlc_witness(HTLC_PREIMAGE), HTLC_RECEIVED_ASM);
    assert_force_close_request(&received_paths);
    assert!(
        received.contains(&format!("tx txid {FORCE_CLOSE_TXID}")),
        "the received HTLC hid the transaction: {received}"
    );
    assert_paints_exact_htlc(&received);
    assert!(
        !received.contains(HTLC_RECEIVED_ASM),
        "the received witness script leaked into the screen: {received}"
    );

    let (stored_paths, stored) =
        paint_force_close(FORCE_CLOSE_OTHER_WITNESS, FORCE_CLOSE_OTHER_ASM);
    assert_force_close_request(&stored_paths);
    assert!(
        stored.contains(&format!("tx txid {FORCE_CLOSE_TXID}")),
        "the stored 66-character offered HTLC hid the transaction: {stored}"
    );
    assert_no_htlc_line(&stored);

    let (expired_paths, expired) = paint_force_close(&htlc_witness(""), FORCE_CLOSE_OTHER_ASM);
    assert_force_close_request(&expired_paths);
    assert!(
        expired.contains(&format!("tx txid {FORCE_CLOSE_TXID}")),
        "the empty HTLC item hid the transaction: {expired}"
    );
    assert_no_htlc_line(&expired);

    let (commitment_paths, commitment) =
        paint_force_close(&force_close_witness(""), FORCE_CLOSE_COMMITMENT_ASM);
    assert_force_close_request(&commitment_paths);
    assert!(
        commitment.lines().any(|line| line == FORCE_CLOSE_LINE),
        "Lightning Force Close left the commitment spend: {commitment}"
    );
    assert_no_htlc_line(&commitment);
}

/// Transaction input button from a no-query `GET /tx/:txid`.
///
/// A `v1_p2tr` input paints the exact line `Inscription` when the network is
/// not Liquid and the taproot script-path script includes the bytes
/// `0063036f7264`. That script is the witness item `parseTaproot` names
/// `scriptPath.script`. A key-path spend, a script that lacks those bytes, and
/// a non-`v1_p2tr` input do not paint it. Liquid does not paint it. `Runestone`
/// stays on an `op_return` script that starts with `6a5d`. The script, the
/// witness, and the script hex stay off the line. There is no tooltip.
#[test]
fn transaction_screen_paints_inscription() {
    const LINE: &str = "Inscription";
    const MARK: &str = "0063036f7264";
    const SCRIPT: &str = "aa0063036f7264bb";
    const PLAIN: &str = "51ac";
    const STACK: &str = "cc0063036f7264dd";
    const CONTROL: &str = "c0111111111111111111111111111111111111111111111111111111111111111111";
    const ANNEX: &str = "50annexsentinel";
    const SIG: &str = "inscriptionsigsentinel";
    const SIG_ASM: &str = "inscriptionsigasm";
    const PREV: &str = "5120inscriptionprev";
    const PREV_ASM: &str = "inscriptionprevasm";
    const STONE: &str = "6a5dinscriptionstone";
    const STONE_ASM: &str = "inscriptionstoneasm";

    fn witness(items: &[&str]) -> String {
        let body = items
            .iter()
            .map(|item| format!("\"{item}\""))
            .collect::<Vec<_>>()
            .join(",");
        format!("[{body}]")
    }

    fn vin(kind: &str, witness_json: &str) -> String {
        format!(
            r#"{{"txid":"prev","vout":0,"prevout":{{"scriptpubkey":"{PREV}","scriptpubkey_asm":"{PREV_ASM}","scriptpubkey_type":"{kind}","value":5}},"scriptsig":"{SIG}","scriptsig_asm":"{SIG_ASM}","witness":{witness_json},"is_coinbase":false,"sequence":4294967295}}"#
        )
    }

    fn tx_body(txid: &str, vin_json: &str) -> String {
        format!(
            r#"{{"txid":"{txid}","version":1,"locktime":0,"vin":[{vin_json}],"vout":[{{"scriptpubkey":"{STONE}","scriptpubkey_asm":"{STONE_ASM}","scriptpubkey_type":"op_return","value":0}}],"size":10,"weight":400,"sigops":0,"fee":1,"status":{{"confirmed":true,"block_height":800000}}}}"#
        )
    }

    fn paint(network: Network, txid: &str, kind: &str, witness_json: &str) -> String {
        let (mut host, rec) = host_with(
            network,
            BTreeMap::from([
                (
                    format!("/tx/{txid}"),
                    tx_body(txid, &vin(kind, witness_json)),
                ),
                ("/blocks/tip/height".to_string(), "800000".to_string()),
            ]),
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

    fn assert_script_stays_off(text: &str) {
        for needle in [
            MARK, SCRIPT, PLAIN, STACK, CONTROL, ANNEX, SIG, SIG_ASM, PREV, PREV_ASM, STONE,
            STONE_ASM,
        ] {
            assert!(!text.contains(needle), "{needle} was painted: {text}");
        }
    }

    fn assert_exact_inscription(text: &str) {
        assert!(
            text.lines().any(|line| line == LINE),
            "Inscription is absent: {text}"
        );
        assert_eq!(
            text.lines().filter(|line| *line == LINE).count(),
            1,
            "Inscription was not painted once: {text}"
        );
        assert!(
            text.lines()
                .filter(|line| line.contains(LINE))
                .all(|line| line == LINE),
            "Inscription was painted inside a longer sentence: {text}"
        );
    }

    fn assert_no_inscription(text: &str) {
        assert!(
            !text.lines().any(|line| line == LINE),
            "the screen painted Inscription: {text}"
        );
        assert!(
            text.lines().all(|line| !line.contains(LINE)),
            "Inscription was painted inside a longer sentence: {text}"
        );
    }

    fn assert_input_neighbors(text: &str, previous: &str) {
        let rows: Vec<&str> = text.lines().collect();
        let outpoint = rows
            .iter()
            .position(|line| *line == "input outpoint prev:0");
        let output = rows
            .iter()
            .position(|line| line.starts_with("output address "));
        assert!(
            outpoint.is_some() && output.is_some() && outpoint.unwrap() < output.unwrap(),
            "the input or the output left the screen: {text}"
        );
        assert!(
            text.lines().any(|line| line == previous),
            "{previous} was dropped: {text}"
        );
        assert_eq!(
            text.lines().filter(|line| *line == "Runestone").count(),
            1,
            "Runestone was dropped or repeated: {text}"
        );
        assert_eq!(
            text.lines().filter(|line| *line == "OP_RETURN").count(),
            1,
            "OP_RETURN was dropped or repeated: {text}"
        );
        assert!(
            text.lines().any(|line| line == "confirmations 1"),
            "confirmations 1 left the screen: {text}"
        );
        assert!(
            !text.contains("tooltip") && !text.contains("Inscription data"),
            "a tooltip was painted: {text}"
        );
        assert_script_stays_off(text);
    }

    let marked = witness(&[SIG, SCRIPT, CONTROL]);
    let on = paint(Network::Mainnet, "inscriptionon", "v1_p2tr", &marked);
    assert!(
        on.contains("tx txid inscriptionon"),
        "the parsed transaction was hidden: {on}"
    );
    assert_exact_inscription(&on);
    let rows: Vec<&str> = on.lines().collect();
    let outpoint = rows
        .iter()
        .position(|line| *line == "input outpoint prev:0");
    let button = rows.iter().position(|line| *line == LINE);
    let output = rows
        .iter()
        .position(|line| line.starts_with("output address "));
    assert!(
        outpoint.is_some() && button.is_some() && output.is_some(),
        "the input or the output left the screen: {on}"
    );
    assert!(
        outpoint.unwrap() < button.unwrap() && button.unwrap() < output.unwrap(),
        "Inscription was not on the transaction input: {on}"
    );
    assert_eq!(
        on.lines().filter(|line| *line == "P2TR tapscript").count(),
        1,
        "P2TR tapscript was dropped or repeated: {on}"
    );
    assert!(
        on.lines().any(|line| line == "Taproot"),
        "Taproot left the screen: {on}"
    );
    assert!(
        on.lines().any(|line| line == "SegWit"),
        "SegWit left the screen: {on}"
    );
    assert_input_neighbors(&on, "Previous output type V1_P2TR");

    let annex = witness(&[SIG, SCRIPT, CONTROL, ANNEX]);
    let annexed = paint(Network::Mainnet, "inscriptionannex", "v1_p2tr", &annex);
    assert!(
        annexed.contains("tx txid inscriptionannex"),
        "an annexed script-path spend hid the transaction: {annexed}"
    );
    assert_exact_inscription(&annexed);
    assert_input_neighbors(&annexed, "Previous output type V1_P2TR");

    let plain = witness(&[SIG, PLAIN, CONTROL]);
    let without = paint(Network::Mainnet, "inscriptionplain", "v1_p2tr", &plain);
    assert!(
        without.contains("tx txid inscriptionplain"),
        "a v1_p2tr input without the marker hid the transaction: {without}"
    );
    assert_no_inscription(&without);
    assert_eq!(
        without
            .lines()
            .filter(|line| *line == "P2TR tapscript")
            .count(),
        1,
        "P2TR tapscript was dropped: {without}"
    );
    assert_input_neighbors(&without, "Previous output type V1_P2TR");

    let stacked = witness(&[STACK, PLAIN, CONTROL]);
    let other_item = paint(Network::Mainnet, "inscriptionstack", "v1_p2tr", &stacked);
    assert!(
        other_item.contains("tx txid inscriptionstack"),
        "a marker outside the script-path script hid the transaction: {other_item}"
    );
    assert_no_inscription(&other_item);
    assert_input_neighbors(&other_item, "Previous output type V1_P2TR");

    let key_path = witness(&[SCRIPT]);
    let key = paint(Network::Mainnet, "inscriptionkey", "v1_p2tr", &key_path);
    assert!(
        key.contains("tx txid inscriptionkey"),
        "a key-path spend hid the transaction: {key}"
    );
    assert_no_inscription(&key);
    assert_input_neighbors(&key, "Previous output type V1_P2TR");

    let other = paint(Network::Mainnet, "inscriptionother", "v0_p2wpkh", &marked);
    assert!(
        other.contains("tx txid inscriptionother"),
        "a non-taproot input hid the transaction: {other}"
    );
    assert_no_inscription(&other);
    assert!(
        !other.lines().any(|line| line == "P2TR tapscript"),
        "a v0_p2wpkh input painted P2TR tapscript: {other}"
    );
    assert_input_neighbors(&other, "Previous output type V0_P2WPKH");

    let liquid = paint(Network::Liquid, "inscriptionliquid", "v1_p2tr", &marked);
    assert!(
        liquid.contains("tx txid inscriptionliquid"),
        "liquid hid the transaction: {liquid}"
    );
    assert_no_inscription(&liquid);
    assert!(
        !liquid.lines().any(|line| line == "Taproot"),
        "liquid painted Taproot: {liquid}"
    );
    assert_eq!(
        liquid
            .lines()
            .filter(|line| *line == "P2TR tapscript")
            .count(),
        1,
        "P2TR tapscript was dropped on liquid: {liquid}"
    );
    assert_input_neighbors(&liquid, "Previous output type V1_P2TR");
}

/// Details button on the loaded transaction page from no-query `GET /tx/:txid`.
///
/// `transaction.component.html` paints a button whose text is `Details`.
/// The click toggles script rows on the transaction list. It does not open
/// the flow diagram. `Show diagram` stays off. A line equal to `Details`
/// passes. `Transaction Details` and `Click Details for more` do not.
/// `confirmations 1` stays. Script hex stays off.
#[test]
fn transaction_screen_paints_details_button() {
    const LINE: &str = "Details";
    const TXID: &str = "detailsbtn";
    const SCRIPT: &str = "0014aa";
    const SIG: &str = "deadbeef";
    const WIT: &str = "abcdwitnesshex";

    let tx = format!(
        r#"{{"txid":"{TXID}","version":1,"locktime":0,"vin":[{{"txid":"prev","vout":0,"prevout":{{"scriptpubkey":"{SCRIPT}","scriptpubkey_asm":"OP_0 OP_PUSHBYTES_20 aa","scriptpubkey_type":"v0_p2wpkh","value":5}},"scriptsig":"{SIG}","scriptsig_asm":"OP_CHECKSIG","witness":["{WIT}"],"is_coinbase":false,"sequence":4294967295}}],"vout":[{{"scriptpubkey":"6a046869","scriptpubkey_asm":"OP_RETURN OP_PUSHBYTES_4","scriptpubkey_type":"op_return","value":0}}],"size":10,"weight":400,"sigops":0,"fee":1,"status":{{"confirmed":true,"block_height":800000}}}}"#
    );
    let (mut host, rec) = host_with(
        Network::Mainnet,
        BTreeMap::from([
            (format!("/tx/{TXID}"), tx),
            ("/blocks/tip/height".to_string(), "800000".to_string()),
        ]),
    );
    host.open_transaction(TXID)
        .unwrap_or_else(|err| panic!("tx {TXID} did not open: {err}"));
    let paths = rec.gets();
    assert!(
        paths.iter().any(|path| path == &format!("/tx/{TXID}")),
        "GET /tx/{TXID} was not requested: {paths:?}"
    );
    assert!(
        paths
            .iter()
            .all(|path| !path.contains('?') && !path.contains('#')),
        "a transaction open used a query or a fragment: {paths:?}"
    );
    let text = painted(&host);
    assert!(
        text.contains(&format!("tx txid {TXID}")),
        "the parsed transaction was hidden: {text}"
    );
    assert!(
        text.lines().any(|line| line == LINE),
        "Details is absent: {text}"
    );
    assert_eq!(
        text.lines().filter(|line| *line == LINE).count(),
        1,
        "Details was not painted once: {text}"
    );
    assert!(
        text.lines()
            .filter(|line| line.contains(LINE))
            .all(|line| line == LINE),
        "Details was painted inside a longer sentence: {text}"
    );
    assert!(
        text.lines().any(|line| line == "Inputs & Outputs"),
        "Inputs & Outputs was dropped: {text}"
    );
    assert!(
        text.lines().any(|line| line == "confirmations 1"),
        "confirmations 1 left the screen: {text}"
    );
    assert!(
        !text
            .lines()
            .any(|line| line == "Show diagram" || line == "Hide diagram"),
        "the diagram control was painted: {text}"
    );
    assert!(!text.contains(SCRIPT), "{text}");
    assert!(!text.contains(SIG), "{text}");
    assert!(!text.contains(WIT), "{text}");
    assert!(!text.contains("OP_CHECKSIG"), "{text}");
}

/// Page heading from no-query `GET /tx/:txid`.
///
/// `transaction.component.html` paints an h1 whose text is `Transaction`.
/// A line equal to `Transaction` passes. A longer line that contains that
/// word does not. `Transaction hex` stays off. `Details` and
/// `Inputs & Outputs` stay.
#[test]
fn transaction_screen_paints_transaction_heading() {
    const LINE: &str = "Transaction";
    const TXID: &str = "pageheading";
    let vin = r#"{"txid":"prev","vout":0,"prevout":null,"scriptsig":"","scriptsig_asm":"","is_coinbase":false,"sequence":1}"#;
    let vout = r#"{"scriptpubkey":"","scriptpubkey_asm":"","scriptpubkey_type":"","value":9}"#;
    let (mut host, rec) = host_with(
        Network::Mainnet,
        BTreeMap::from([(format!("/tx/{TXID}"), tx_with_parties(TXID, vin, vout))]),
    );
    host.open_transaction(TXID)
        .unwrap_or_else(|err| panic!("tx {TXID} did not open: {err}"));
    let paths = rec.gets();
    assert!(
        paths.iter().any(|path| path == &format!("/tx/{TXID}")),
        "GET /tx/{TXID} was not requested: {paths:?}"
    );
    assert!(
        paths
            .iter()
            .all(|path| !path.contains('?') && !path.contains('#')),
        "a transaction open used a query or a fragment: {paths:?}"
    );
    let text = painted(&host);
    assert!(
        text.contains(&format!("tx txid {TXID}")),
        "the parsed transaction was hidden: {text}"
    );
    assert!(
        text.lines().any(|line| line == LINE),
        "Transaction is absent: {text}"
    );
    assert_eq!(
        text.lines().filter(|line| *line == LINE).count(),
        1,
        "Transaction was not painted once: {text}"
    );
    assert!(
        text.lines()
            .filter(|line| line.contains(LINE))
            .all(|line| line == LINE),
        "Transaction was painted inside a longer sentence: {text}"
    );
    assert_eq!(
        text.matches(LINE).count(),
        1,
        "Transaction was not once: {text}"
    );
    assert!(
        !text.contains("Transaction hex"),
        "Transaction hex was painted: {text}"
    );
    assert!(
        text.lines().any(|line| line == "Inputs & Outputs"),
        "Inputs & Outputs was dropped: {text}"
    );
    assert!(
        text.lines().any(|line| line == "Details"),
        "Details was dropped: {text}"
    );
}

/// Page heading from no-query `GET /address/:address`.
///
/// `address.component.html` paints an h1 whose text is `Address`.
/// A line equal to `Address` passes. A longer line that contains that
/// word does not. `Error loading address data` does not. A type badge
/// line does not. The loaded address line and `Type P2WPKH` stay.
#[test]
fn screen_paints_address_heading() {
    const LINE: &str = "Address";
    const ADDRESS: &str = "bc1q8yj0herd4r4yxszw3nkfvt53433thk0f5qst4g";
    let (mut host, rec) = host_with(
        Network::Mainnet,
        BTreeMap::from([
            (format!("/address/{ADDRESS}"), address_stats(ADDRESS)),
            (format!("/address/{ADDRESS}/txs"), "[]".to_string()),
            (format!("/address/{ADDRESS}/utxo"), "[]".to_string()),
        ]),
    );
    host.open_address(ADDRESS)
        .unwrap_or_else(|err| panic!("address {ADDRESS} did not open: {err}"));
    let paths = rec.gets();
    assert!(
        paths
            .iter()
            .any(|path| path == &format!("/address/{ADDRESS}")),
        "GET /address/{ADDRESS} was not requested: {paths:?}"
    );
    assert!(
        paths
            .iter()
            .all(|path| !path.contains('?') && !path.contains('#')),
        "an address open used a query or a fragment: {paths:?}"
    );
    let text = painted(&host);
    assert!(
        text.contains(&format!("address {ADDRESS}")),
        "the parsed address was hidden: {text}"
    );
    assert!(
        text.lines().any(|line| line == LINE),
        "Address is absent: {text}"
    );
    assert_eq!(
        text.lines().filter(|line| *line == LINE).count(),
        1,
        "Address was not painted once: {text}"
    );
    assert!(
        text.lines()
            .filter(|line| line.contains(LINE))
            .all(|line| line == LINE),
        "Address was painted inside a longer sentence: {text}"
    );
    assert_eq!(
        text.matches(LINE).count(),
        1,
        "Address was not once: {text}"
    );
    assert!(
        !text.contains("Error loading address data"),
        "Error loading address data was painted: {text}"
    );
    assert!(
        text.lines().any(|line| line == "Type P2WPKH"),
        "the type badge was dropped: {text}"
    );
    assert!(
        text.lines()
            .filter(|line| line.starts_with("Type "))
            .all(|line| line != LINE && !line.contains(LINE)),
        "an address type badge line satisfied the heading: {text}"
    );
}

/// Heading from no-query `GET /mempool/recent` on the dashboard and the recent page.
///
/// `dashboard.component.html` and `recent-transactions-list.component.html`
/// both paint `Recent Transactions`. A line equal to that heading passes.
/// A longer sentence that contains those words does not. The recent row stays
/// `recent txid {txid} fee {fee} vsize {vsize} value {value}`.
#[test]
fn screen_paints_recent_transactions_heading() {
    const HEADING: &str = "Recent Transactions";
    const ROW: &str = "recent txid recent-heading fee 10 vsize 0 value 1";

    fn summary() -> String {
        r#"{"count":1,"vsize":9,"total_fee":1,"fee_histogram":[]}"#.to_string()
    }

    fn recent_body() -> String {
        r#"[{"txid":"recent-heading","fee":10,"vsize":0,"value":1}]"#.to_string()
    }

    fn assert_heading_and_row(painted: &str) {
        assert!(
            painted.lines().any(|line| line == HEADING),
            "Recent Transactions is absent: {painted}"
        );
        assert_eq!(
            painted.lines().filter(|line| *line == HEADING).count(),
            1,
            "Recent Transactions was not painted once: {painted}"
        );
        assert!(
            painted
                .lines()
                .filter(|line| line.contains(HEADING))
                .all(|line| line == HEADING),
            "Recent Transactions was painted inside a longer sentence: {painted}"
        );
        assert_eq!(
            painted.matches(HEADING).count(),
            1,
            "Recent Transactions was not once: {painted}"
        );
        assert!(
            painted.lines().any(|line| line == ROW),
            "the recent row changed: {painted}"
        );
        let row = painted
            .lines()
            .find(|line| *line == ROW)
            .unwrap_or_else(|| panic!("missing recent row: {painted}"));
        assert!(
            !row.contains("sat/vB") && !row.contains("fee_rate"),
            "the recent row gained a fee rate suffix: {row}"
        );
        let lines: Vec<&str> = painted.lines().collect();
        let heading_at = lines.iter().position(|line| *line == HEADING);
        let row_at = lines.iter().position(|line| *line == ROW);
        assert!(
            heading_at.is_some() && row_at.is_some() && heading_at < row_at,
            "Recent Transactions was not before the recent row: {painted}"
        );
    }

    fn assert_no_query(paths: &[String]) {
        assert!(
            paths.iter().any(|path| path == "/mempool/recent"),
            "did not GET /mempool/recent: {paths:?}"
        );
        assert!(
            paths
                .iter()
                .all(|path| !path.contains('?') && !path.contains('#')),
            "{paths:?}"
        );
    }

    let (mut host, rec) = host_with(
        Network::Mainnet,
        BTreeMap::from([
            ("/mempool".to_string(), summary()),
            ("/mempool/recent".to_string(), recent_body()),
        ]),
    );
    let opened = host.activate(Screen::Mempool);
    assert_no_query(&rec.gets());
    opened.unwrap_or_else(|err| panic!("recent page did not open: {err}"));
    assert_heading_and_row(&painted(&host));

    let (mut host, rec) = host_with(
        Network::Mainnet,
        BTreeMap::from([
            ("/blocks".to_string(), "[]".to_string()),
            ("/blocks/tip/height".to_string(), "1".to_string()),
            ("/blocks/tip/hash".to_string(), "abc".to_string()),
            ("/fee-estimates".to_string(), "{}".to_string()),
            ("/mempool".to_string(), summary()),
            ("/mempool/recent".to_string(), recent_body()),
        ]),
    );
    let opened = host.activate(Screen::Dashboard);
    assert_no_query(&rec.gets());
    opened.unwrap_or_else(|err| panic!("dashboard did not open: {err}"));
    assert_heading_and_row(&painted(&host));
}

/// Page heading from no-query `GET /block/:hash`.
///
/// `block.component.html` paints `Block` when the block is not stale and
/// height is null or greater than 0. Height 0 uses `Genesis`. A stale block
/// uses `Stale Block`. A line equal to `Block` passes. A longer line that
/// contains that word does not. `Block Header Hex` stays off. `TXs` stays.
#[test]
fn screen_paints_block_heading() {
    const LINE: &str = "Block";
    const ID: &str = "blockheading";

    fn open_block_paint(id: &str, body: String) -> (String, Vec<String>) {
        let (mut host, rec) = host_with(
            Network::Mainnet,
            BTreeMap::from([
                (format!("/block/{id}"), body),
                (format!("/block/{id}/txs"), "[]".to_string()),
                (format!("/block/{id}/txids"), "[]".to_string()),
            ]),
        );
        host.open_block(id)
            .unwrap_or_else(|err| panic!("block {id} did not open: {err}"));
        (painted(&host), rec.gets())
    }

    fn assert_block_line_once(text: &str) {
        assert!(
            text.lines().any(|line| line == LINE),
            "Block is absent: {text}"
        );
        assert_eq!(
            text.lines().filter(|line| *line == LINE).count(),
            1,
            "Block was not painted once: {text}"
        );
        assert!(
            text.lines()
                .filter(|line| line.contains(LINE))
                .all(|line| line == LINE),
            "Block was painted inside a longer sentence: {text}"
        );
        assert_eq!(text.matches(LINE).count(), 1, "Block was not once: {text}");
    }

    let (text, paths) = open_block_paint(ID, block_json(ID));
    assert!(
        paths.iter().any(|path| path == &format!("/block/{ID}")),
        "GET /block/{ID} was not requested: {paths:?}"
    );
    assert!(
        paths
            .iter()
            .all(|path| !path.contains('?') && !path.contains('#')),
        "a block open used a query or a fragment: {paths:?}"
    );
    assert!(
        text.contains(&format!("block hash {ID}")),
        "the parsed block was hidden: {text}"
    );
    assert_block_line_once(&text);
    assert!(
        !text.contains("Block Header Hex"),
        "Block Header Hex was painted: {text}"
    );
    assert!(
        text.lines().any(|line| line == "TXs"),
        "TXs was dropped: {text}"
    );

    let genesis_id = "genesisheading";
    let genesis_body = block_json(genesis_id).replacen("\"height\":800000", "\"height\":0", 1);
    let (genesis, _) = open_block_paint(genesis_id, genesis_body);
    assert_eq!(
        genesis.lines().filter(|line| *line == "Genesis").count(),
        1,
        "Genesis is absent: {genesis}"
    );
    assert_eq!(
        genesis.lines().filter(|line| *line == LINE).count(),
        0,
        "height 0 painted the whole line Block: {genesis}"
    );

    let stale_id = "staleheading";
    let mut stale_body = block_json(stale_id);
    assert!(stale_body.ends_with('}'));
    stale_body.pop();
    stale_body.push_str(r#","stale":true}"#);
    let (stale, _) = open_block_paint(stale_id, stale_body);
    assert!(
        stale.contains(&format!("block hash {stale_id}")),
        "the stale block was hidden: {stale}"
    );
    assert_eq!(
        stale.lines().filter(|line| *line == "Stale Block").count(),
        1,
        "Stale Block is absent: {stale}"
    );
    assert_eq!(
        stale.lines().filter(|line| *line == LINE).count(),
        0,
        "a stale block painted the whole line Block: {stale}"
    );
}

/// Page heading from no-query `GET /assets/registry` on the liquid dashboard.
///
/// Angular `assets-nav.component.html` paints
/// `<h1 i18n="Assets page header">Assets</h1>`. A line equal to `Assets`
/// passes. A longer line that contains that word does not. A comment is not
/// a line. The column headers Name, Ticker, Issuer domain, and Asset ID stay,
/// and they are not this heading. The issued name and ticker stay. The
/// request path has no query string and no fragment.
#[test]
fn screen_paints_assets_heading() {
    const LINE: &str = "Assets";
    let asset = "12".repeat(32);
    let registry = format!(
        r#"[{{"asset_id":"{asset}","name":"Test Asset","ticker":"TEST","precision":2,"issuance_txin":{{"txid":"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa","vin":0}},"issuance_prevout":{{"txid":"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa","vout":0}},"reissuance_token":"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa","status":{{"confirmed":true}},"chain_stats":{{"tx_count":1,"issuance_count":1,"issued_amount":1,"burned_amount":0,"has_blinded_issuances":false,"burned_reissuance_tokens":0}},"mempool_stats":{{"tx_count":0,"issuance_count":0,"issued_amount":0,"burned_amount":0,"has_blinded_issuances":false,"burned_reissuance_tokens":0}}}}]"#
    );
    let (mut host, rec) = host_with(
        Network::Liquid,
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
            ("/assets/registry".to_string(), registry),
        ]),
    );
    let opened = host.activate(Screen::Dashboard);
    let paths = rec.gets();
    assert!(
        paths.iter().any(|path| path == "/assets/registry"),
        "the assets registry was not requested: {paths:?}"
    );
    assert!(
        paths
            .iter()
            .all(|path| !path.contains('?') && !path.contains('#')),
        "the assets registry used a query or a fragment: {paths:?}"
    );
    opened.unwrap_or_else(|err| panic!("assets registry did not open: {err}"));
    let text = painted(&host);
    assert!(
        text.lines().any(|line| line == "Test Asset"),
        "the issued name was dropped: {text}"
    );
    assert!(
        text.lines().any(|line| line == "TEST"),
        "the ticker value was dropped: {text}"
    );
    assert!(
        text.lines().any(|line| line == "Name"),
        "the Name column header was dropped: {text}"
    );
    assert!(
        text.lines().any(|line| line == "Ticker"),
        "the Ticker column header was dropped: {text}"
    );
    assert!(
        text.lines().any(|line| line == "Issuer domain"),
        "the Issuer domain column header was dropped: {text}"
    );
    assert!(
        text.lines().any(|line| line == "Asset ID"),
        "the Asset ID column header was dropped: {text}"
    );
    assert!(
        text.lines().any(|line| line == LINE),
        "Assets is absent: {text}"
    );
    assert_eq!(
        text.lines().filter(|line| *line == LINE).count(),
        1,
        "Assets was not painted once: {text}"
    );
    assert!(
        text.lines()
            .filter(|line| line.contains(LINE))
            .all(|line| line == LINE),
        "Assets was painted inside a longer sentence: {text}"
    );
    assert_eq!(text.matches(LINE).count(), 1, "Assets was not once: {text}");
    assert_ne!(LINE, "Asset ID");
    assert_ne!(LINE, "Name");
    let lines: Vec<&str> = text.lines().collect();
    let heading_at = lines.iter().position(|line| *line == LINE);
    let name_at = lines.iter().position(|line| *line == "Name");
    assert!(
        heading_at.is_some() && name_at.is_some() && heading_at < name_at,
        "Assets was not before the Name column header: {text}"
    );
    assert_paint_is_not_html(&text);
}

/// Card title from no-query `GET /fee-estimates` on the dashboard.
///
/// `dashboard.component.html` paints `Transaction Fees` when the network
/// is not liquid. A line equal to that title passes. A longer line that
/// contains those words does not. The fee target line stays. Liquid does
/// not paint the title. This is not the details row `Transaction fee`.
#[test]
fn screen_paints_transaction_fees_heading() {
    const LINE: &str = "Transaction Fees";
    const FEE: &str = "fee target 2 rate 1.1";

    fn bodies() -> BTreeMap<String, String> {
        BTreeMap::from([
            ("/blocks".to_string(), "[]".to_string()),
            ("/blocks/tip/height".to_string(), "1".to_string()),
            ("/blocks/tip/hash".to_string(), "abc".to_string()),
            ("/fee-estimates".to_string(), r#"{"2":1.1}"#.to_string()),
            (
                "/mempool".to_string(),
                r#"{"count":1,"vsize":9,"total_fee":1,"fee_histogram":[]}"#.to_string(),
            ),
            ("/mempool/recent".to_string(), "[]".to_string()),
        ])
    }

    fn assert_no_query(paths: &[String]) {
        assert!(
            paths.iter().any(|path| path == "/fee-estimates"),
            "did not GET /fee-estimates: {paths:?}"
        );
        assert!(
            paths
                .iter()
                .all(|path| !path.contains('?') && !path.contains('#')),
            "{paths:?}"
        );
    }

    let (mut host, rec) = host_with(Network::Mainnet, bodies());
    let opened = host.activate(Screen::Dashboard);
    assert_no_query(&rec.gets());
    opened.unwrap_or_else(|err| panic!("dashboard did not open: {err}"));
    let text = painted(&host);
    assert!(
        text.lines().any(|line| line == FEE),
        "the fee estimate was hidden: {text}"
    );
    assert!(
        text.lines().any(|line| line == LINE),
        "Transaction Fees is absent: {text}"
    );
    assert_eq!(
        text.lines().filter(|line| *line == LINE).count(),
        1,
        "Transaction Fees was not painted once: {text}"
    );
    assert!(
        text.lines()
            .filter(|line| line.contains(LINE))
            .all(|line| line == LINE),
        "Transaction Fees was painted inside a longer sentence: {text}"
    );
    assert_eq!(
        text.matches(LINE).count(),
        1,
        "Transaction Fees was not once: {text}"
    );
    assert!(
        !text.lines().any(|line| line == "Transaction fee"),
        "the details row Transaction fee was painted on the dashboard: {text}"
    );
    assert!(
        text.lines()
            .filter(|line| *line == LINE)
            .all(|line| !line.contains("sat/vB")),
        "Transaction Fees gained sat/vB: {text}"
    );
    let lines: Vec<&str> = text.lines().collect();
    let title_at = lines.iter().position(|line| *line == LINE);
    let fee_at = lines.iter().position(|line| *line == FEE);
    assert!(
        title_at.is_some() && fee_at.is_some() && title_at < fee_at,
        "Transaction Fees was not before the fee target: {text}"
    );

    let (mut host, rec) = host_with(Network::Liquid, bodies());
    let opened = host.activate(Screen::Dashboard);
    assert_no_query(&rec.gets());
    opened.unwrap_or_else(|err| panic!("liquid dashboard did not open: {err}"));
    let liquid = painted(&host);
    assert!(
        liquid.lines().any(|line| line == FEE),
        "the liquid fee estimate was hidden: {liquid}"
    );
    assert!(
        !liquid.lines().any(|line| line == LINE),
        "liquid painted Transaction Fees: {liquid}"
    );
}

/// Details label from no-query `GET /block/:hash`.
///
/// `block.component.html` paints `Nonce` beside `decimal2hex` when the
/// network is not liquid and details are open. The whole line is `Nonce`
/// plus that hex. A longer line that contains the word does not. The raw
/// `nonce` line stays. A missing key and JSON null omit the label. Liquid
/// does not paint it.
#[test]
fn screen_paints_nonce() {
    const LABEL: &str = "Nonce";
    const RAW: &str = "nonce 9 0x00000009";
    const LINE: &str = "Nonce 0x00000009";

    fn assert_label_once(text: &str) {
        assert!(
            text.lines().any(|line| line == LINE),
            "Nonce 0x00000009 is absent: {text}"
        );
        assert_eq!(
            text.lines().filter(|line| *line == LINE).count(),
            1,
            "Nonce was not painted once: {text}"
        );
        assert!(
            text.lines()
                .filter(|line| line.contains(LABEL))
                .all(|line| line == LINE),
            "Nonce was painted inside a longer sentence: {text}"
        );
        assert_eq!(text.matches(LABEL).count(), 1, "Nonce was not once: {text}");
        assert!(
            text.lines().any(|line| line == RAW),
            "the raw nonce line was dropped: {text}"
        );
        let lines: Vec<&str> = text.lines().collect();
        let label_at = lines.iter().position(|line| *line == LINE);
        let raw_at = lines.iter().position(|line| *line == RAW);
        assert!(
            label_at.is_some() && raw_at.is_some() && label_at.unwrap() + 1 == raw_at.unwrap(),
            "Nonce was not immediately before the raw nonce line: {text}"
        );
        assert!(
            text.lines().any(|line| line == "Difficulty 1"),
            "Difficulty was dropped: {text}"
        );
        assert!(
            !text
                .lines()
                .any(|line| line == "Nonce 9" || line == "Nonce 9 0x00000009"),
            "Nonce gained the decimal: {text}"
        );
    }

    let id = "noncelabel9";
    assert!(!id.bytes().all(|byte| byte.is_ascii_digit()), "{id}");
    let (mut host, rec) = host_with(
        Network::Mainnet,
        BTreeMap::from([
            (format!("/block/{id}"), block_nonce_json(id, "9")),
            (format!("/block/{id}/txs"), "[]".to_string()),
            (format!("/block/{id}/txids"), "[]".to_string()),
        ]),
    );
    host.block.query = id.to_string();
    host.activate(Screen::Block)
        .unwrap_or_else(|err| panic!("block {id} did not open: {err}"));
    let paths = rec.gets();
    assert!(
        paths.iter().any(|path| path == &format!("/block/{id}")),
        "GET /block/{id} was not requested: {paths:?}"
    );
    assert!(
        paths
            .iter()
            .all(|path| !path.contains('?') && !path.contains('#')),
        "a block open used a query or a fragment: {paths:?}"
    );
    let closed = painted(&host);
    assert!(
        !closed.lines().any(|line| line == LINE),
        "closed details painted {LINE}: {closed}"
    );
    assert!(
        closed.lines().any(|line| line.starts_with("block hash ")),
        "the block summary was hidden: {closed}"
    );
    host.open_row("row block details")
        .unwrap_or_else(|err| panic!("block details did not open: {err}"));
    assert_label_once(&painted(&host));

    let zero = paint_block_bits("noncelabel0", block_nonce_json("noncelabel0", "0"));
    assert!(
        zero.lines().any(|line| line == "Nonce 0x00000000"),
        "nonce 0 did not paint Nonce 0x00000000: {zero}"
    );
    assert!(
        zero.lines().any(|line| line == "nonce 0 0x00000000"),
        "nonce 0 dropped the raw line: {zero}"
    );

    let missing = paint_block_bits("noncelabelmiss", missing_nonce_json("noncelabelmiss"));
    assert!(
        !missing.lines().any(|line| line.starts_with(LABEL)),
        "missing nonce painted Nonce: {missing}"
    );
    assert!(
        !missing.contains(LABEL),
        "missing nonce painted Nonce: {missing}"
    );

    let null = paint_block_bits("noncelabelnull", block_nonce_json("noncelabelnull", "null"));
    assert!(
        !null.lines().any(|line| line.starts_with(LABEL)),
        "null nonce painted Nonce: {null}"
    );

    let liquid_id = "noncelabelliq";
    let (mut host, rec) = host_with(
        Network::Liquid,
        BTreeMap::from([
            (
                format!("/block/{liquid_id}"),
                block_nonce_json(liquid_id, "9"),
            ),
            (format!("/block/{liquid_id}/txs"), "[]".to_string()),
            (format!("/block/{liquid_id}/txids"), "[]".to_string()),
        ]),
    );
    host.block.query = liquid_id.to_string();
    host.activate(Screen::Block)
        .unwrap_or_else(|err| panic!("liquid block {liquid_id} did not open: {err}"));
    let paths = rec.gets();
    assert!(
        paths
            .iter()
            .any(|path| path == &format!("/block/{liquid_id}")),
        "GET /block/{liquid_id} was not requested: {paths:?}"
    );
    assert!(
        paths
            .iter()
            .all(|path| !path.contains('?') && !path.contains('#')),
        "{paths:?}"
    );
    host.open_row("row block details")
        .unwrap_or_else(|err| panic!("liquid block details did not open: {err}"));
    let liquid = painted(&host);
    assert!(
        liquid.lines().any(|line| line == RAW),
        "liquid dropped the raw nonce line: {liquid}"
    );
    assert!(
        !liquid
            .lines()
            .any(|line| line == LINE || line.starts_with(LABEL)),
        "liquid painted Nonce: {liquid}"
    );
}

/// Details label from no-query `GET /block/:hash`.
///
/// `block.component.html` paints `Weight` as its own cell beside the weight
/// units. The whole line is `Weight`. A longer line that contains the word
/// does not. Closed details do not paint it.
#[test]
fn screen_paints_weight() {
    const LINE: &str = "Weight";
    let id = "weightlabel1";
    let body = format!(
        r#"{{"id":"{id}","height":800000,"version":1,"timestamp":1700000000,"tx_count":2,"size":200,"weight":800,"merkle_root":"mm","previousblockhash":"pp","mediantime":1700000000,"nonce":1,"bits":2,"difficulty":3.0}}"#
    );
    let (mut host, rec) = host_with(
        Network::Mainnet,
        BTreeMap::from([
            (format!("/block/{id}"), body),
            (format!("/block/{id}/txs"), "[]".to_string()),
            (format!("/block/{id}/txids"), "[]".to_string()),
        ]),
    );
    host.block.query = id.to_string();
    host.activate(Screen::Block)
        .unwrap_or_else(|err| panic!("block {id} did not open: {err}"));
    let paths = rec.gets();
    assert!(
        paths.iter().any(|path| path == &format!("/block/{id}")),
        "GET /block/{id} was not requested: {paths:?}"
    );
    assert!(
        paths
            .iter()
            .all(|path| !path.contains('?') && !path.contains('#')),
        "a block open used a query or a fragment: {paths:?}"
    );
    let closed = painted(&host);
    assert!(
        !closed.lines().any(|line| line == LINE),
        "closed details painted {LINE}: {closed}"
    );
    host.open_row("row block details")
        .unwrap_or_else(|err| panic!("block details did not open: {err}"));
    let text = painted(&host);
    assert!(
        text.lines().any(|line| line == LINE),
        "Weight is absent: {text}"
    );
    assert_eq!(
        text.lines().filter(|line| *line == LINE).count(),
        1,
        "Weight was not painted once: {text}"
    );
    assert!(
        text.lines()
            .filter(|line| line.contains(LINE))
            .all(|line| line == LINE),
        "Weight was painted inside a longer sentence: {text}"
    );
    assert_eq!(text.matches(LINE).count(), 1, "Weight was not once: {text}");
}

/// Page heading from no-query `GET /blocks`.
///
/// `blocks-list.component.html` paints
/// `<h1 i18n="master-page.blocks">Blocks</h1>` when the list is not a widget.
/// A line equal to `Blocks` passes. A longer line that contains that word
/// does not. This is not the block page heading `Block`. `TXs` stays. The
/// request path has no query string and no fragment.
#[test]
fn screen_paints_blocks_heading() {
    const LINE: &str = "Blocks";
    const ID: &str = "listhead1";
    let body = format!(
        r#"[{{"id":"{ID}","height":2,"version":1,"timestamp":9,"tx_count":17,"size":20,"weight":80,"merkle_root":"mm","mediantime":8,"difficulty":1.0}}]"#
    );
    let (mut host, rec) = host_with(
        Network::Mainnet,
        BTreeMap::from([("/blocks".to_string(), body)]),
    );
    let opened = host.activate(Screen::Blocks);
    let paths = rec.gets();
    assert!(
        paths.iter().any(|path| path == "/blocks"),
        "the blocks list was not requested: {paths:?}"
    );
    assert!(
        paths
            .iter()
            .all(|path| !path.contains('?') && !path.contains('#')),
        "the blocks list used a query or a fragment: {paths:?}"
    );
    opened.unwrap_or_else(|err| panic!("blocks list did not open: {err}"));
    let text = painted(&host);
    assert!(
        text.contains(&format!("block hash {ID}")),
        "the block row was dropped: {text}"
    );
    assert!(
        text.lines().any(|line| line == "TXs"),
        "TXs was dropped: {text}"
    );
    assert!(
        text.lines().any(|line| line == LINE),
        "Blocks is absent: {text}"
    );
    assert_eq!(
        text.lines().filter(|line| *line == LINE).count(),
        1,
        "Blocks was not painted once: {text}"
    );
    assert!(
        text.lines()
            .filter(|line| line.contains(LINE))
            .all(|line| line == LINE),
        "Blocks was painted inside a longer sentence: {text}"
    );
    assert_eq!(text.matches(LINE).count(), 1, "Blocks was not once: {text}");
    assert_ne!(LINE, "Block");
    let lines: Vec<&str> = text.lines().collect();
    let heading_at = lines.iter().position(|line| *line == LINE);
    let txs_at = lines.iter().position(|line| *line == "TXs");
    assert!(
        heading_at.is_some() && txs_at.is_some() && heading_at < txs_at,
        "Blocks was not before TXs: {text}"
    );
    assert_paint_is_not_html(&text);
}

/// Blocks list column header from no-query `GET /blocks`.
///
/// `blocks-list.component.html` paints `Transactions` when the list is not
/// the mempool module. A line equal to `Transactions` passes. A longer line
/// that contains that word does not. This is not the transaction page title
/// `Transaction`. `Blocks` and `TXs` stay. The request path has no query
/// string and no fragment.
#[test]
fn screen_paints_blocks_transactions_header() {
    const LINE: &str = "Transactions";
    const ID: &str = "txcolhdr1";
    let body = format!(
        r#"[{{"id":"{ID}","height":2,"version":1,"timestamp":9,"tx_count":17,"size":20,"weight":80,"merkle_root":"mm","mediantime":8,"difficulty":1.0}}]"#
    );
    let (mut host, rec) = host_with(
        Network::Mainnet,
        BTreeMap::from([("/blocks".to_string(), body)]),
    );
    let opened = host.activate(Screen::Blocks);
    let paths = rec.gets();
    assert!(
        paths.iter().any(|path| path == "/blocks"),
        "the blocks list was not requested: {paths:?}"
    );
    assert!(
        paths
            .iter()
            .all(|path| !path.contains('?') && !path.contains('#')),
        "the blocks list used a query or a fragment: {paths:?}"
    );
    opened.unwrap_or_else(|err| panic!("blocks list did not open: {err}"));
    let text = painted(&host);
    assert!(
        text.contains(&format!("block hash {ID}")),
        "the block row was dropped: {text}"
    );
    assert!(
        text.lines().any(|line| line == "TXs"),
        "TXs was dropped: {text}"
    );
    assert!(
        text.lines().any(|line| line == "Blocks"),
        "Blocks was dropped: {text}"
    );
    assert!(
        text.lines().any(|line| line == LINE),
        "Transactions is absent: {text}"
    );
    assert_eq!(
        text.lines().filter(|line| *line == LINE).count(),
        1,
        "Transactions was not painted once: {text}"
    );
    assert!(
        text.lines()
            .filter(|line| line.contains(LINE))
            .all(|line| line == LINE),
        "Transactions was painted inside a longer sentence: {text}"
    );
    assert_eq!(
        text.matches(LINE).count(),
        1,
        "Transactions was not once: {text}"
    );
    assert_ne!(LINE, "Transaction");
    let lines: Vec<&str> = text.lines().collect();
    let header_at = lines.iter().position(|line| *line == LINE);
    let txs_at = lines.iter().position(|line| *line == "TXs");
    assert!(
        header_at.is_some() && txs_at.is_some() && header_at < txs_at,
        "Transactions was not before TXs: {text}"
    );
    assert_paint_is_not_html(&text);
}

/// Column header from no-query `POST /txs/test`.
///
/// `test-transactions.component.html` paints `Rejection reason` on the
/// result table. A line equal to `Rejection reason` passes. A longer line
/// that contains those words does not. The data token `reject-reason` stays
/// on its own row. `Allowed?` stays. The request path has no query string
/// and no fragment.
#[test]
fn screen_paints_rejection_reason() {
    const LINE: &str = "Rejection reason";
    const DATA: &str = "test txid rejectedtxid888 allowed false reject-reason dust";
    let body = r#"[{"txid":"rejectedtxid888","wtxid":"rejectedwtxid888","allowed":false,"reject-reason":"dust"}]"#.to_string();
    let (mut host, rec) = host_with(
        Network::Mainnet,
        BTreeMap::from([("/txs/test".to_string(), body)]),
    );
    host.activate(Screen::TestTransactions)
        .expect("test transactions");
    host.test_txs.input = "abcd".to_string();
    host.submit().expect("post test transactions");
    let posts = rec.posts();
    assert!(
        posts.iter().any(|(path, _)| path == "/txs/test"),
        "POST /txs/test was not requested: {posts:?}"
    );
    assert!(
        posts
            .iter()
            .all(|(path, _)| !path.contains('?') && !path.contains('#')),
        "the test post used a query or a fragment: {posts:?}"
    );
    let text = painted(&host);
    assert!(
        text.lines().any(|line| line == DATA),
        "the reject-reason row was dropped: {text}"
    );
    assert!(
        text.lines().any(|line| line == "Allowed?"),
        "Allowed? was dropped: {text}"
    );
    assert!(
        text.lines().any(|line| line == LINE),
        "Rejection reason is absent: {text}"
    );
    assert_eq!(
        text.lines().filter(|line| *line == LINE).count(),
        1,
        "Rejection reason was not painted once: {text}"
    );
    assert!(
        text.lines()
            .filter(|line| line.contains(LINE))
            .all(|line| line == LINE),
        "Rejection reason was painted inside a longer sentence: {text}"
    );
    assert_eq!(
        text.matches(LINE).count(),
        1,
        "Rejection reason was not once: {text}"
    );
    assert_ne!(LINE, "reject-reason");
    let lines: Vec<&str> = text.lines().collect();
    let header_at = lines.iter().position(|line| *line == LINE);
    let data_at = lines.iter().position(|line| *line == DATA);
    assert!(
        header_at.is_some() && data_at.is_some() && header_at < data_at,
        "Rejection reason was not before the reject-reason row: {text}"
    );
    assert_paint_is_not_html(&text);
}

/// Column header from no-query `POST /txs/test`.
///
/// The result table paints `Effective fee rate` on its own line. A longer
/// line that contains those words does not. The data token stays
/// `effective_feerate` plus the value from `effective-feerate`. That token
/// is not the header. `Allowed?` and `Rejection reason` stay. The request
/// path has no query string and no fragment.
#[test]
fn screen_paints_effective_fee_rate() {
    const LINE: &str = "Effective fee rate";
    const DATA: &str = "test txid test-tx-1 allowed false reject-reason min relay fee not met fee 0.00001 effective_feerate 1";
    let body = r#"[{"txid":"test-tx-1","wtxid":"test-wtxid-1","allowed":false,"vsize":141,"fees":{"base":0.00001,"effective-feerate":1.0,"effective-includes":["test-tx-1"]},"reject-reason":"min relay fee not met"}]"#.to_string();
    let (mut host, rec) = host_with(
        Network::Mainnet,
        BTreeMap::from([("/txs/test".to_string(), body)]),
    );
    host.activate(Screen::TestTransactions)
        .expect("test transactions");
    host.test_txs.input = "abcd".to_string();
    host.submit().expect("post test transactions");
    let posts = rec.posts();
    assert!(
        posts.iter().any(|(path, _)| path == "/txs/test"),
        "POST /txs/test was not requested: {posts:?}"
    );
    assert!(
        posts
            .iter()
            .all(|(path, _)| !path.contains('?') && !path.contains('#')),
        "the test post used a query or a fragment: {posts:?}"
    );
    let text = painted(&host);
    assert!(
        text.lines().any(|line| line == DATA),
        "the effective_feerate row was dropped: {text}"
    );
    assert!(
        text.lines().any(|line| line == "Allowed?"),
        "Allowed? was dropped: {text}"
    );
    assert!(
        text.lines().any(|line| line == LINE),
        "Effective fee rate is absent: {text}"
    );
    assert_eq!(
        text.lines().filter(|line| *line == LINE).count(),
        1,
        "Effective fee rate was not painted once: {text}"
    );
    assert!(
        text.lines()
            .filter(|line| line.contains(LINE))
            .all(|line| line == LINE),
        "Effective fee rate was painted inside a longer sentence: {text}"
    );
    assert_eq!(
        text.matches(LINE).count(),
        1,
        "Effective fee rate was not once: {text}"
    );
    assert_ne!(LINE, "effective_feerate");
    assert_ne!(LINE, "effective-feerate");
    let lines: Vec<&str> = text.lines().collect();
    let header_at = lines.iter().position(|line| *line == LINE);
    let data_at = lines.iter().position(|line| *line == DATA);
    assert!(
        header_at.is_some() && data_at.is_some() && header_at < data_at,
        "Effective fee rate was not before the effective_feerate row: {text}"
    );
    assert_paint_is_not_html(&text);
}

/// Address-cell label from no-query `GET /tx/:txid`.
///
/// `transactions-list.component.html` paints `Coinbase` when `is_coinbase`
/// is true. A line equal to `Coinbase` passes. A longer line that contains
/// that word does not. The data row stays `input coinbase`.
/// `(Newly Generated Coins)` stays. False does not paint the label.
/// The request path has no query string and no fragment.
#[test]
fn screen_paints_coinbase() {
    const LINE: &str = "Coinbase";
    const DATA: &str = "input coinbase";
    const TX: &str = "coinbasetxlabel";
    let body = format!(
        r#"{{
        "txid":"{TX}",
        "version":2,
        "locktime":1,
        "vin":[{{
            "txid":"prev",
            "vout":1,
            "prevout":null,
            "scriptsig":"deadbeef",
            "scriptsig_asm":"OP_CHECKSIG",
            "is_coinbase":true,
            "sequence":0
        }}],
        "vout":[{{
            "scriptpubkey":"51",
            "scriptpubkey_asm":"OP_TRUE",
            "scriptpubkey_type":"op_return",
            "value":9
        }}],
        "size":10,
        "weight":40,
        "sigops":1,
        "fee":1,
        "status":{{"confirmed":false}}
    }}"#
    );
    let (mut host, rec) = host_with(
        Network::Mainnet,
        BTreeMap::from([(format!("/tx/{TX}"), body)]),
    );
    host.open_transaction(TX).expect("tx");
    let paths = rec.gets();
    assert!(
        paths.iter().any(|path| path == &format!("/tx/{TX}")),
        "GET /tx/{TX} was not requested: {paths:?}"
    );
    assert!(
        paths
            .iter()
            .all(|path| !path.contains('?') && !path.contains('#')),
        "the transaction get used a query or a fragment: {paths:?}"
    );
    let text = painted(&host);
    assert!(
        text.lines().any(|line| line == DATA),
        "the input coinbase row was dropped: {text}"
    );
    assert!(
        text.lines().any(|line| line == "(Newly Generated Coins)"),
        "(Newly Generated Coins) was dropped: {text}"
    );
    assert!(
        text.lines().any(|line| line == LINE),
        "Coinbase is absent: {text}"
    );
    assert_eq!(
        text.lines().filter(|line| *line == LINE).count(),
        1,
        "Coinbase was not painted once: {text}"
    );
    assert!(
        text.lines()
            .filter(|line| line.contains(LINE))
            .all(|line| line == LINE),
        "Coinbase was painted inside a longer sentence: {text}"
    );
    assert_eq!(
        text.matches(LINE).count(),
        1,
        "Coinbase was not once: {text}"
    );
    assert_ne!(LINE, "is_coinbase");
    assert_ne!(LINE, "input coinbase");
    let lines: Vec<&str> = text.lines().collect();
    let header_at = lines.iter().position(|line| *line == LINE);
    let data_at = lines.iter().position(|line| *line == DATA);
    assert!(
        header_at.is_some() && data_at.is_some() && header_at < data_at,
        "Coinbase was not before the input coinbase row: {text}"
    );
    assert_paint_is_not_html(&text);

    let ordinary = paint_tx_input("coinbaseoff", r#","is_coinbase":false,"sequence":7"#);
    assert!(
        !ordinary.lines().any(|line| line == LINE),
        "false coinbase painted Coinbase: {ordinary}"
    );
    assert!(
        !ordinary.contains(LINE),
        "false coinbase painted Coinbase: {ordinary}"
    );
}

/// Address-cell caption from `GET /tx/:txid` when an output has `pegout`.
///
/// The template prints the exact line `Peg-out to` when the output has no
/// address and the type is not `p2pk`. An address, JSON null `pegout`, and a
/// `p2pk` output do not gain that line. `Liquid Peg Out` stays the separate
/// badge. The path has no query and no fragment. Script hex and script asm
/// stay off this line.
#[test]
fn transaction_screen_paints_peg_out_to_label() {
    const LABEL: &str = "Peg-out to";
    const PEGOUT: &str =
        r#"{"genesis_hash":"aa","scriptpubkey":"bb","scriptpubkey_asm":"OP_TRUE"}"#;

    fn vin() -> String {
        r#"{"txid":"prev","vout":0,"prevout":{"scriptpubkey":"0014aa","scriptpubkey_asm":"OP_0","scriptpubkey_type":"v0_p2wpkh","scriptpubkey_address":"bc1qin","value":8},"scriptsig":"","scriptsig_asm":"","is_coinbase":false,"sequence":1}"#.to_string()
    }

    fn vout(extra: &str) -> String {
        format!(
            r#"{{"scriptpubkey":"0014aa","scriptpubkey_asm":"OP_0","scriptpubkey_type":"v0_p2wpkh","value":1{extra}}}"#
        )
    }

    fn paint(txid: &str, vout_json: &str) -> String {
        assert!(
            !txid.is_empty() && !txid.contains('/') && !txid.contains('?') && !txid.contains('#'),
            "{txid}"
        );
        let body = format!(
            r#"{{"txid":"{txid}","version":1,"locktime":0,"vin":[{vin}],"vout":[{vout_json}],"size":10,"weight":40,"sigops":0,"fee":1,"status":{{"confirmed":true,"block_height":4,"block_hash":"tiphash1","block_time":1600000000}}}}"#,
            vin = vin(),
        );
        let (mut host, rec) = host_with(
            Network::Mainnet,
            BTreeMap::from([(format!("/tx/{txid}"), body)]),
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

    let ordinary = r#"{"scriptpubkey":"0014bb","scriptpubkey_asm":"OP_0","scriptpubkey_type":"v0_p2wpkh","scriptpubkey_address":"bc1qordinary","value":9}"#;
    let pegged = paint(
        "pegoutto",
        &format!("{},{ordinary}", vout(&format!(r#","pegout":{PEGOUT}"#))),
    );
    assert_eq!(
        pegged.lines().filter(|line| *line == LABEL).count(),
        1,
        "{LABEL} was not one exact line: {pegged}"
    );
    assert!(
        pegged.lines().any(|line| line == "Liquid Peg Out"),
        "Liquid Peg Out was dropped: {pegged}"
    );
    let rows: Vec<&str> = pegged.lines().collect();
    let address = rows
        .iter()
        .position(|line| *line == "output address  value 1");
    let script_type = rows
        .iter()
        .position(|line| *line == "output script type v0_p2wpkh");
    let type_line = rows.iter().position(|line| *line == "Type V0_P2WPKH");
    let label_at = rows.iter().position(|line| *line == LABEL);
    let ordinary_at = rows
        .iter()
        .position(|line| *line == "output address bc1qordinary value 9");
    assert!(
        address.is_some()
            && script_type.is_some()
            && type_line.is_some()
            && label_at.is_some()
            && ordinary_at.is_some(),
        "the peg-out output lines or the ordinary output were dropped: {pegged}"
    );
    assert!(
        address.unwrap() < script_type.unwrap()
            && script_type.unwrap() < type_line.unwrap()
            && type_line.unwrap() < label_at.unwrap()
            && label_at.unwrap() < ordinary_at.unwrap(),
        "{LABEL} was not on the peg-out output after Type V0_P2WPKH and before the ordinary output: {pegged}"
    );
    assert!(!pegged.contains("0014aa"), "{pegged}");
    assert!(!pegged.contains("OP_TRUE"), "{pegged}");
    assert!(!pegged.contains("OP_0"), "{pegged}");
    assert_paint_is_not_html(&pegged);

    let addressed = paint(
        "pegoutaddr",
        &vout(&format!(
            r#","scriptpubkey_address":"bc1qpeg","pegout":{PEGOUT}"#
        )),
    );
    assert_eq!(
        addressed.lines().filter(|line| *line == LABEL).count(),
        0,
        "an addressed peg-out painted {LABEL}: {addressed}"
    );
    assert!(
        addressed
            .lines()
            .any(|line| line == "output address bc1qpeg value 1"),
        "the addressed peg-out output was dropped: {addressed}"
    );
    assert!(
        addressed.lines().any(|line| line == "Liquid Peg Out"),
        "Liquid Peg Out was dropped on an addressed peg-out: {addressed}"
    );

    let null_peg = paint("pegoutnull", &vout(r#","pegout":null"#));
    assert!(
        !null_peg.lines().any(|line| line == LABEL),
        "JSON null pegout painted {LABEL}: {null_peg}"
    );
    assert!(
        !null_peg.lines().any(|line| line == "Liquid Peg Out"),
        "JSON null pegout painted Liquid Peg Out: {null_peg}"
    );

    let p2pk = paint(
        "pegoutp2pk",
        &format!(
            r#"{{"scriptpubkey":"0014aa","scriptpubkey_asm":"OP_0","scriptpubkey_type":"p2pk","value":1,"pegout":{PEGOUT}}}"#
        ),
    );
    assert_eq!(
        p2pk.lines().filter(|line| *line == LABEL).count(),
        0,
        "a p2pk peg-out painted {LABEL}: {p2pk}"
    );
    assert_eq!(
        p2pk.lines().filter(|line| *line == "P2PK").count(),
        1,
        "P2PK was dropped on a p2pk peg-out: {p2pk}"
    );
    assert!(
        p2pk.lines().any(|line| line == "Liquid Peg Out"),
        "Liquid Peg Out was dropped on a p2pk peg-out: {p2pk}"
    );
}

/// Column header from no-query `GET /blocks` on the dashboard.
///
/// `dashboard.component.html` paints `Size` on the recent blocks table.
/// A line equal to `Size` passes. A longer line that contains that word does
/// not. The block summary keeps lowercase `size` and the bytes text. Liquid
/// does not paint the header. An empty list does not. The blocks list does
/// not. The request path has no query string and no fragment.
#[test]
fn screen_paints_size_column() {
    const LINE: &str = "Size";
    const ID: &str = "sizedash1";
    const SUMMARY: &str = "size 20 B";

    fn bodies(blocks: &str) -> BTreeMap<String, String> {
        BTreeMap::from([
            ("/blocks".to_string(), blocks.to_string()),
            ("/blocks/tip/height".to_string(), "1".to_string()),
            ("/blocks/tip/hash".to_string(), "abc".to_string()),
            ("/fee-estimates".to_string(), "{}".to_string()),
            (
                "/mempool".to_string(),
                r#"{"count":1,"vsize":9,"total_fee":1,"fee_histogram":[]}"#.to_string(),
            ),
            ("/mempool/recent".to_string(), "[]".to_string()),
        ])
    }

    fn assert_no_query(paths: &[String]) {
        assert!(
            paths.iter().any(|path| path == "/blocks"),
            "did not GET /blocks: {paths:?}"
        );
        assert!(
            paths
                .iter()
                .all(|path| !path.contains('?') && !path.contains('#')),
            "{paths:?}"
        );
    }

    let blocks = format!("[{}]", block_json(ID));
    assert!(
        blocks.contains("\"size\":20"),
        "the block fixture dropped size: {blocks}"
    );
    let (mut host, rec) = host_with(Network::Mainnet, bodies(&blocks));
    let opened = host.activate(Screen::Dashboard);
    assert_no_query(&rec.gets());
    opened.unwrap_or_else(|err| panic!("dashboard did not open: {err}"));
    let text = painted(&host);
    let summary = text
        .lines()
        .find(|line| line.starts_with("block hash ") && line.contains(ID))
        .unwrap_or_else(|| panic!("the block summary was dropped: {text}"));
    assert!(
        summary.contains(SUMMARY),
        "the size field was dropped: {summary}"
    );
    assert!(
        !summary.contains("Size "),
        "title-case Size leaked into the summary line: {summary}"
    );
    assert!(
        text.lines().any(|line| line == LINE),
        "Size is absent: {text}"
    );
    assert_eq!(
        text.lines().filter(|line| *line == LINE).count(),
        1,
        "Size was not painted once: {text}"
    );
    assert!(
        text.lines()
            .filter(|line| line.contains(LINE))
            .all(|line| line == LINE),
        "Size was painted inside a longer sentence: {text}"
    );
    assert_eq!(text.matches(LINE).count(), 1, "Size was not once: {text}");
    assert!(
        !text.lines().any(|line| line.starts_with("Size ")),
        "the details row Size was painted on the dashboard: {text}"
    );
    let lines: Vec<&str> = text.lines().collect();
    let header_at = lines.iter().position(|line| *line == LINE);
    let summary_at = lines.iter().position(|line| line.contains(SUMMARY));
    assert!(
        header_at.is_some() && summary_at.is_some() && header_at < summary_at,
        "Size was not before the block summary: {text}"
    );

    let (mut host, rec) = host_with(Network::Liquid, bodies(&blocks));
    let opened = host.activate(Screen::Dashboard);
    assert_no_query(&rec.gets());
    opened.unwrap_or_else(|err| panic!("liquid dashboard did not open: {err}"));
    let liquid = painted(&host);
    assert!(
        liquid.contains(SUMMARY),
        "the liquid size field was hidden: {liquid}"
    );
    assert!(
        !liquid.lines().any(|line| line == LINE),
        "liquid painted Size: {liquid}"
    );

    let (mut host, rec) = host_with(Network::Mainnet, bodies("[]"));
    let opened = host.activate(Screen::Dashboard);
    assert_no_query(&rec.gets());
    opened.unwrap_or_else(|err| panic!("empty dashboard did not open: {err}"));
    let empty = painted(&host);
    assert!(
        empty.contains("blocks:"),
        "empty list dropped the blocks marker: {empty}"
    );
    assert!(
        !empty.lines().any(|line| line == LINE),
        "an empty block list painted Size: {empty}"
    );

    let (mut host, rec) = host_with(
        Network::Mainnet,
        BTreeMap::from([("/blocks".to_string(), blocks)]),
    );
    host.activate(Screen::Blocks)
        .unwrap_or_else(|err| panic!("blocks list did not open: {err}"));
    let paths = rec.gets();
    assert!(
        paths.iter().any(|path| path == "/blocks"),
        "list did not GET /blocks: {paths:?}"
    );
    assert!(
        paths
            .iter()
            .all(|path| !path.contains('?') && !path.contains('#')),
        "{paths:?}"
    );
    let list = painted(&host);
    assert!(!list.contains(LINE), "blocks list painted Size: {list}");
}

/// Features label from a no-query `GET /tx/:txid`.
///
/// `transaction-details.component.html` paints the word Features when the
/// network is not liquid and feature activation is on. Unconfirmed
/// `status.confirmed` is on. A confirmed mainnet height is on at rbf
/// activation, 399701. A confirmed height below that, a missing status,
/// an empty body, invalid JSON, and the block list omit the label. The
/// whole line is Features. Script hex, script asm, witness, and RBF stay
/// off that line.
#[test]
fn transaction_screen_paints_features_label() {
    const LINE: &str = "Features";
    const SCRIPT_HEX: &str = "FEATPUBHEXDEAD";
    const SCRIPT_SIG_HEX: &str = "FEATSIGHEXDEAD";
    const SCRIPT_SIG_ASM: &str = "FEATSIGASMDEAD";
    const SCRIPT_ASM: &str = "FEATASMDEAD";
    const WITNESS: &str = "FEATWITDEAD";

    let tx_body = |txid: &str, status: &str| -> String {
        let status_field = if status.is_empty() {
            String::new()
        } else {
            format!(r#","status":{status}"#)
        };
        format!(
            r#"{{"txid":"{txid}","version":2,"locktime":0,"vin":[{{"txid":"bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb","vout":0,"prevout":null,"scriptsig":"{SCRIPT_SIG_HEX}","scriptsig_asm":"{SCRIPT_SIG_ASM}","witness":["{WITNESS}"],"is_coinbase":false,"sequence":4294967295}}],"vout":[{{"scriptpubkey":"{SCRIPT_HEX}","scriptpubkey_asm":"{SCRIPT_ASM}","scriptpubkey_type":"p2pkh","value":1}}],"size":10,"weight":40,"sigops":0,"fee":1{status_field}}}"#
        )
    };

    let paint = |txid: &str, status: &str| -> String {
        let (mut host, rec) = host_with(
            Network::Mainnet,
            BTreeMap::from([(format!("/tx/{txid}"), tx_body(txid, status))]),
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
    };

    let assert_scripts_stay_off = |text: &str| {
        for needle in [
            SCRIPT_HEX,
            SCRIPT_SIG_HEX,
            SCRIPT_SIG_ASM,
            SCRIPT_ASM,
            WITNESS,
            "RBF",
            "Replace-By-Fee",
            "replace-by-fee",
        ] {
            assert!(!text.contains(needle), "{needle} was painted: {text}");
        }
    };

    let assert_features_once = |text: &str, txid: &str| {
        assert!(
            text.contains(&format!("tx txid {txid}")),
            "the transaction row was dropped: {text}"
        );
        assert!(
            text.lines().any(|line| line == LINE),
            "Features is absent: {text}"
        );
        assert_eq!(
            text.lines().filter(|line| *line == LINE).count(),
            1,
            "Features was not painted once: {text}"
        );
        assert!(
            text.lines()
                .filter(|line| line.contains(LINE))
                .all(|line| line == LINE),
            "Features was painted inside a longer sentence: {text}"
        );
        assert_eq!(
            text.matches(LINE).count(),
            1,
            "Features was not once: {text}"
        );
        assert_scripts_stay_off(text);
        assert_paint_is_not_html(text);
    };

    let assert_features_absent = |text: &str| {
        assert!(
            !text.lines().any(|line| line == LINE),
            "Features was painted: {text}"
        );
        assert!(!text.contains(LINE), "Features leaked: {text}");
        assert_scripts_stay_off(text);
    };

    let pending = paint("featuresunconf", r#"{"confirmed":false}"#);
    assert_features_once(&pending, "featuresunconf");

    let on = paint(
        "features399701",
        r#"{"confirmed":true,"block_height":399701,"block_hash":"dddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddd","block_time":1600000000}"#,
    );
    assert_features_once(&on, "features399701");
    assert!(
        on.lines().any(|line| line == "Timestamp"),
        "the block time line was dropped: {on}"
    );

    let below = paint(
        "features399700",
        r#"{"confirmed":true,"block_height":399700,"block_hash":"dddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddd"}"#,
    );
    assert!(
        below.contains("tx txid features399700"),
        "a lower height hid the transaction: {below}"
    );
    assert_features_absent(&below);

    let missing = paint("featuresnostatus", "");
    assert!(
        missing.contains("tx txid featuresnostatus"),
        "a missing status hid the transaction: {missing}"
    );
    assert_features_absent(&missing);

    for (txid, body) in [
        ("featuresblank", ""),
        ("featuresbad", "not-json"),
        ("featuresarr", "[]"),
    ] {
        let (mut host, _) = host_with(
            Network::Mainnet,
            BTreeMap::from([(format!("/tx/{txid}"), body.to_string())]),
        );
        let _opened = host.open_transaction(txid);
        let painted = painted(&host);
        assert!(
            painted.contains(txid),
            "the unparsed body hid the transaction screen: {painted}"
        );
        assert!(
            !painted.contains(&format!("tx txid {txid}")),
            "an unparsed body painted a transaction row: {painted}"
        );
        assert_features_absent(&painted);
    }

    let block_id = "featblock1";
    let txid = "featuresblocktx";
    let (mut host, rec) = host_with(
        Network::Mainnet,
        BTreeMap::from([
            (format!("/block/{block_id}"), block_json(block_id)),
            (
                format!("/block/{block_id}/txs"),
                format!("[{}]", tx_body(txid, r#"{"confirmed":false}"#)),
            ),
            (format!("/block/{block_id}/txids"), "[]".to_string()),
        ]),
    );
    host.open_block(block_id)
        .unwrap_or_else(|err| panic!("block {block_id} did not open: {err}"));
    let paths = rec.gets();
    assert!(
        paths
            .iter()
            .any(|path| path == &format!("/block/{block_id}/txs")),
        "the block list did not GET txs: {paths:?}"
    );
    assert!(
        paths
            .iter()
            .all(|path| !path.contains('?') && !path.contains('#')),
        "the block list used a query or a fragment: {paths:?}"
    );
    let blocks = painted(&host);
    assert!(
        blocks.contains(&format!("tx txid {txid}")),
        "the block list hid the transaction row: {blocks}"
    );
    assert_features_absent(&blocks);
}

/// Address transaction list column header from `GET /address/:addr/txs`.
///
/// Angular `address-transactions-widget.component.html` paints the word Date
/// as that column header. One exact line `Date` when the list has rows. The
/// `block_time` cell stays the raw integer. An empty list omits the word.
/// This is not a formatted clock and not a relative duration.
#[test]
fn address_transactions_paint_date_header() {
    const LABEL: &str = "Date";
    const TIME: u64 = 1_700_000_000;

    fn one_tx(txid: &str, fee: u64) -> String {
        format!(
            r#"{{"txid":"{txid}","version":1,"locktime":0,"vin":[],"vout":[],"size":10,"weight":40,"sigops":0,"fee":{fee},"status":{{"confirmed":true,"block_height":1,"block_time":{TIME}}}}}"#
        )
    }

    fn txs(rows: &[(&str, u64)]) -> String {
        let body = rows
            .iter()
            .map(|(txid, fee)| one_tx(txid, *fee))
            .collect::<Vec<_>>()
            .join(",");
        format!("[{body}]")
    }

    fn assert_header_once_before(paint: &str, txid: &str) {
        assert_eq!(
            paint.lines().filter(|line| *line == LABEL).count(),
            1,
            "{LABEL} was not one exact line: {paint}"
        );
        assert_eq!(
            paint.matches(LABEL).count(),
            1,
            "{LABEL} was not once: {paint}"
        );
        let header_at = paint
            .lines()
            .position(|line| line == LABEL)
            .unwrap_or_else(|| panic!("{LABEL} missing\n{paint}"));
        let row_at = paint
            .lines()
            .position(|line| line.starts_with(&format!("address tx txid {txid} ")))
            .unwrap_or_else(|| panic!("address tx row missing\n{paint}"));
        assert!(
            header_at < row_at,
            "{LABEL} was not before the address transaction row\n{paint}"
        );
    }

    fn assert_raw_time(paint: &str, txid: &str, fee: u64) {
        let row = paint
            .lines()
            .find(|line| line.starts_with(&format!("address tx txid {txid} ")))
            .unwrap_or_else(|| panic!("missing address tx row {txid}: {paint}"));
        let marker = format!("block_time {TIME}");
        assert!(
            row.contains(&marker) && row.contains(&format!("fee {fee}")),
            "the cell did not stay the raw integer: {row}"
        );
        assert!(
            !row.contains("ago")
                && !row.contains("UTC")
                && !row.contains(&format!("{TIME}.00"))
                && !row.contains("2023-"),
            "the cell gained a clock or a formatted date: {row}"
        );
    }

    fn assert_header_absent(paint: &str, why: &str) {
        assert!(
            !paint.lines().any(|line| line == LABEL),
            "{why} painted {LABEL}: {paint}"
        );
        assert!(!paint.contains(LABEL), "{why} contains {LABEL}: {paint}");
    }

    let one = paint_address_loaded("bc1qhdrone", 1, 0, &txs(&[("hdrtxone", 4)]));
    assert_header_once_before(&one, "hdrtxone");
    assert_raw_time(&one, "hdrtxone", 4);

    let two = paint_address_loaded("bc1qhdrtwo", 2, 0, &txs(&[("hdrtxa", 1), ("hdrtxb", 2)]));
    assert_header_once_before(&two, "hdrtxa");
    assert_raw_time(&two, "hdrtxa", 1);
    assert_raw_time(&two, "hdrtxb", 2);

    let hash = "ab".repeat(32);
    let script = paint_scripthash_loaded(&hash, 1, 0, &txs(&[("hdrscript", 9)]));
    assert_header_once_before(&script, "hdrscript");
    assert_raw_time(&script, "hdrscript", 9);

    let empty = paint_address_loaded("bc1qhdrempty", 1, 0, "[]");
    assert_header_absent(&empty, "empty list");
    assert!(
        empty
            .lines()
            .any(|line| line == "The address transaction list is empty."),
        "empty list dropped the empty sentence: {empty}"
    );

    let hash_empty = "cd".repeat(32);
    let script_empty = paint_scripthash_loaded(&hash_empty, 1, 0, "[]");
    assert_header_absent(&script_empty, "empty scripthash list");
    assert!(
        script_empty
            .lines()
            .any(|line| line == "The address transaction list is empty."),
        "empty scripthash list dropped the empty sentence: {script_empty}"
    );
}

/// Mempool summary from no-query `GET /mempool`.
///
/// The summary paints count, vsize, and total_fee, and the sentence that the
/// virtual-byte rate is not in the payload. It does not paint
/// `Incoming Transactions`. The virtual-byte rate is not in this payload, so
/// `vB/s` and `WU/s` stay off. The blocks list does not paint the label.
/// The request path has no query string and no fragment.
#[test]
fn screen_paints_incoming_transactions() {
    const LINE: &str = "Incoming Transactions";
    const COUNT: &str = "mempool count 4 vsize 500 total_fee 90";
    const RATE: &str = "The incoming virtual-bytes-per-second rate is not in the payload.";

    fn mempool_bodies() -> BTreeMap<String, String> {
        BTreeMap::from([
            (
                "/mempool".to_string(),
                r#"{"count":4,"vsize":500,"total_fee":90,"fee_histogram":[[2,100]]}"#.to_string(),
            ),
            ("/mempool/recent".to_string(), "[]".to_string()),
        ])
    }

    fn assert_no_query(paths: &[String]) {
        assert!(
            paths.iter().any(|path| path == "/mempool"),
            "did not GET /mempool: {paths:?}"
        );
        assert!(
            paths
                .iter()
                .all(|path| !path.contains('?') && !path.contains('#')),
            "{paths:?}"
        );
    }

    fn assert_label_once(text: &str) {
        assert!(
            text.lines().any(|line| line == COUNT),
            "the mempool count line was dropped: {text}"
        );
        assert!(
            text.lines().any(|line| line == RATE),
            "the missing-rate sentence was dropped: {text}"
        );
        assert!(
            !text.contains(LINE),
            "Incoming Transactions is still painted: {text}"
        );
        assert_eq!(
            text.lines().filter(|line| *line == LINE).count(),
            0,
            "Incoming Transactions is still its own line: {text}"
        );
        assert!(
            text.lines()
                .filter(|line| line.contains(LINE))
                .all(|line| line == LINE),
            "Incoming Transactions was painted inside a longer sentence: {text}"
        );
        assert_eq!(
            text.matches(LINE).count(),
            0,
            "Incoming Transactions is still in the summary: {text}"
        );
        assert!(!text.contains("vB/s"), "vB/s was painted: {text}");
        assert!(!text.contains("WU/s"), "WU/s was painted: {text}");
        assert_paint_is_not_html(text);
    }

    let (mut host, rec) = host_with(Network::Mainnet, mempool_bodies());
    let opened = host.activate(Screen::Mempool);
    assert_no_query(&rec.gets());
    opened.unwrap_or_else(|err| panic!("mempool did not open: {err}"));
    assert_label_once(&painted(&host));

    let (mut host, rec) = host_with(
        Network::Mainnet,
        BTreeMap::from([
            ("/blocks".to_string(), "[]".to_string()),
            ("/blocks/tip/height".to_string(), "1".to_string()),
            ("/blocks/tip/hash".to_string(), "abc".to_string()),
            ("/fee-estimates".to_string(), "{}".to_string()),
            (
                "/mempool".to_string(),
                r#"{"count":4,"vsize":500,"total_fee":90,"fee_histogram":[[2,100]]}"#.to_string(),
            ),
            ("/mempool/recent".to_string(), "[]".to_string()),
        ]),
    );
    let opened = host.activate(Screen::Dashboard);
    assert_no_query(&rec.gets());
    opened.unwrap_or_else(|err| panic!("dashboard did not open: {err}"));
    assert_label_once(&painted(&host));

    let (mut host, rec) = host_with(
        Network::Mainnet,
        BTreeMap::from([(
            "/blocks".to_string(),
            format!("[{}]", block_json("noblockfooter")),
        )]),
    );
    host.activate(Screen::Blocks)
        .unwrap_or_else(|err| panic!("blocks list did not open: {err}"));
    let paths = rec.gets();
    assert!(
        paths.iter().any(|path| path == "/blocks"),
        "list did not GET /blocks: {paths:?}"
    );
    assert!(
        paths
            .iter()
            .all(|path| !path.contains('?') && !path.contains('#')),
        "{paths:?}"
    );
    let list = painted(&host);
    assert!(
        !list.lines().any(|line| line == LINE),
        "blocks list painted Incoming Transactions: {list}"
    );
}

/// Search dropdown title from `GET /assets/registry/search`.
///
/// Angular `search-results.component.html` paints the card title
/// `Liquid Asset` when `results.liquidAssets` is non-empty. One exact line,
/// before the asset name. The name and the ticker stay their own lines.
/// An empty hit list omits the title. The path has no query string and no fragment.
#[test]
fn liquid_search_paints_liquid_asset_heading() {
    const LABEL: &str = "Liquid Asset";
    const NAME: &str = "Liquid Bitcoin";
    const TICKER: &str = "L-BTC";

    fn liquid_shell(search_body: &str) -> BTreeMap<String, String> {
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
            (
                "/assets/registry/search".to_string(),
                search_body.to_string(),
            ),
        ])
    }

    let asset_id = "bb".repeat(32);
    let hits = format!(r#"[{{"asset_id":"{asset_id}","name":"{NAME}","ticker":"{TICKER}"}}]"#);
    let (mut host, rec) = host_with(Network::Liquid, liquid_shell(&hits));
    host.activate(Screen::Dashboard).expect("dashboard");
    host.set_query("USDt");
    host.reload().expect("registry search");
    let paths = rec.gets();
    assert!(
        paths.iter().any(|path| path == "/assets/registry/search"),
        "liquid search was not requested: {paths:?}"
    );
    assert!(
        paths
            .iter()
            .all(|path| !path.contains('?') && !path.contains('#')),
        "{paths:?}"
    );
    let paint = painted(&host);
    let title_at = paint.lines().position(|line| line == LABEL);
    let name_at = paint.lines().position(|line| line == NAME);
    assert!(
        title_at.is_some(),
        "the liquid asset heading was not painted\n{paint}"
    );
    assert_eq!(
        paint.lines().filter(|line| *line == LABEL).count(),
        1,
        "{LABEL} was not one exact line: {paint}"
    );
    assert_eq!(
        paint.matches(LABEL).count(),
        1,
        "{LABEL} was painted inside a longer sentence: {paint}"
    );
    assert!(name_at.is_some(), "the asset name was dropped\n{paint}");
    assert!(
        title_at.unwrap() < name_at.unwrap(),
        "{LABEL} was not before {NAME}\n{paint}"
    );
    assert!(
        paint.lines().any(|line| line == TICKER),
        "the ticker was dropped\n{paint}"
    );

    let (mut empty_host, empty_rec) = host_with(Network::Liquid, liquid_shell("[]"));
    empty_host
        .activate(Screen::Dashboard)
        .expect("empty dashboard");
    empty_host.set_query("USDt");
    empty_host.reload().expect("empty registry search");
    assert!(
        empty_rec
            .gets()
            .iter()
            .any(|path| path == "/assets/registry/search"),
        "empty liquid search was not requested: {:?}",
        empty_rec.gets()
    );
    assert!(
        empty_rec
            .gets()
            .iter()
            .all(|path| !path.contains('?') && !path.contains('#')),
        "{:?}",
        empty_rec.gets()
    );
    let empty_paint = painted(&empty_host);
    assert!(
        !empty_paint.lines().any(|line| line == LABEL),
        "an empty hit list painted {LABEL}: {empty_paint}"
    );
}

const SCRIPTSIG_ASM_LABEL: &str = "ScriptSig (ASM)";
const SSA_SIG: &str = "SSAHEXDEAD";
const SSA_ASM: &str = "SSAASMDEAD";

fn scriptsig_asm_vin(sig: &str, asm: &str) -> String {
    format!(
        r#"{{"txid":"prevssa","vout":0,"prevout":{{"scriptpubkey":"SSAPREVHEX","scriptpubkey_asm":"SSAPREVASM","scriptpubkey_type":"v0_p2wpkh","scriptpubkey_address":"bc1qssa","value":5}},"scriptsig":"{sig}","scriptsig_asm":"{asm}","is_coinbase":false,"sequence":1}}"#
    )
}

fn paint_scriptsig_asm(txid: &str, vins: &str) -> String {
    assert!(
        !txid.is_empty() && !txid.contains('/') && !txid.contains('?') && !txid.contains('#'),
        "{txid}"
    );
    let body = format!(
        r#"{{"txid":"{txid}","version":1,"locktime":0,"vin":[{vins}],"vout":[{{"scriptpubkey":"SSAOUTHEX","scriptpubkey_asm":"SSAOUTASM","scriptpubkey_type":"op_return","value":0}}],"size":10,"weight":40,"sigops":0,"fee":1,"status":{{"confirmed":false}}}}"#
    );
    let (mut host, rec) = host_with(
        Network::Mainnet,
        BTreeMap::from([(format!("/tx/{txid}"), body)]),
    );
    host.open_transaction(txid).expect("tx");
    let paths = rec.gets();
    assert!(
        paths.iter().any(|path| path == &format!("/tx/{txid}")),
        "GET /tx/{txid} was not registered: {paths:?}"
    );
    assert!(
        paths
            .iter()
            .all(|path| !path.contains('?') && !path.contains('#')),
        "{paths:?}"
    );
    painted(&host)
}

fn assert_scriptsig_asm_bytes_stay_off(painted: &str) {
    for needle in [
        SSA_SIG,
        SSA_ASM,
        "SSAPREVHEX",
        "SSAPREVASM",
        "SSAOUTHEX",
        "SSAOUTASM",
    ] {
        assert!(
            !painted.contains(needle),
            "{needle} leaked into the screen: {painted}"
        );
    }
}

/// `ScriptSig (ASM)` stays off the transaction screen from `GET /tx/:txid`.
///
/// A non-empty `scriptsig`, an empty `scriptsig`, and a non-empty `scriptsig`
/// with an empty asm all omit the caption. Two such inputs omit it too. The
/// hex and the asm stay off the screen. The path has no query string and no
/// fragment.
#[test]
fn transaction_screen_paints_scriptsig_asm_label() {
    let present = paint_scriptsig_asm("ssapresent", &scriptsig_asm_vin(SSA_SIG, SSA_ASM));
    assert!(
        present.contains("input outpoint"),
        "a scriptsig input dropped the input outpoint: {present}"
    );
    assert!(
        !present.contains("ScriptSig (ASM)"),
        "ScriptSig (ASM) is still painted: {present}"
    );
    assert_eq!(
        present
            .lines()
            .filter(|line| *line == SCRIPTSIG_ASM_LABEL)
            .count(),
        0,
        "ScriptSig (ASM) was painted as a whole line: {present}"
    );
    assert_eq!(
        present.matches(SCRIPTSIG_ASM_LABEL).count(),
        0,
        "ScriptSig (ASM) was painted inside a longer sentence: {present}"
    );
    assert!(
        present.lines().any(|line| line == "Previous output script"),
        "Previous output script was dropped: {present}"
    );
    assert_scriptsig_asm_bytes_stay_off(&present);

    let empty_hex = paint_scriptsig_asm("ssaempty", &scriptsig_asm_vin("", SSA_ASM));
    assert!(
        empty_hex.contains("input outpoint"),
        "an empty scriptsig dropped the input outpoint: {empty_hex}"
    );
    assert!(
        !empty_hex.lines().any(|line| line == SCRIPTSIG_ASM_LABEL),
        "an empty scriptsig painted ScriptSig (ASM): {empty_hex}"
    );
    assert_scriptsig_asm_bytes_stay_off(&empty_hex);

    let empty_asm = paint_scriptsig_asm("ssaasmoff", &scriptsig_asm_vin(SSA_SIG, ""));
    assert!(
        !empty_asm.contains("ScriptSig (ASM)"),
        "a non-empty scriptsig with an empty asm painted ScriptSig (ASM): {empty_asm}"
    );
    assert_scriptsig_asm_bytes_stay_off(&empty_asm);

    let two = paint_scriptsig_asm(
        "ssatwo",
        &format!(
            "{},{}",
            scriptsig_asm_vin(SSA_SIG, SSA_ASM),
            scriptsig_asm_vin(SSA_SIG, SSA_ASM)
        ),
    );
    assert!(
        !two.contains("ScriptSig (ASM)"),
        "two scriptsig inputs painted ScriptSig (ASM): {two}"
    );
    assert_scriptsig_asm_bytes_stay_off(&two);
}

/// Block height caption from a no-query `GET /tx/:txid`.
///
/// `tracker.component.html` line 96 is the exact line `Block height` when the
/// transaction is confirmed and `status.block_height` is present. The line
/// sits next to `confirmations 1`. A missing height, an unconfirmed status,
/// and a height on an unconfirmed status omit the caption. The height number
/// stays on the existing transaction row. It is not a second caption. Sent,
/// Pending, Soon, Next Block, Sign In, and Liquid Asset stay off.
#[test]
fn transaction_screen_paints_block_height_label() {
    const LINE: &str = "Block height";

    fn tx_body(txid: &str, status: &str) -> String {
        format!(
            r#"{{"txid":"{txid}","version":1,"locktime":0,"vin":[{{"txid":"prev","vout":0,"prevout":null,"scriptsig":"","scriptsig_asm":"","is_coinbase":false,"sequence":1}}],"vout":[{{"scriptpubkey":"0014aa","scriptpubkey_asm":"OP_0","scriptpubkey_type":"op_return","value":9}}],"size":10,"weight":40,"sigops":0,"fee":1,"status":{status}}}"#
        )
    }

    fn paint(txid: &str, status: &str, tip: Option<&str>) -> String {
        assert!(
            !txid.is_empty() && !txid.contains('/') && !txid.contains('?') && !txid.contains('#'),
            "{txid}"
        );
        let mut bodies = BTreeMap::from([(format!("/tx/{txid}"), tx_body(txid, status))]);
        if let Some(tip) = tip {
            bodies.insert("/blocks/tip/height".to_string(), tip.to_string());
        }
        let (mut host, rec) = host_with(Network::Mainnet, bodies);
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

    let assert_caption_off = |text: &str| {
        assert!(
            !text.lines().any(|line| line == LINE),
            "{LINE} was painted: {text}"
        );
        assert!(!text.contains(LINE), "{LINE} leaked: {text}");
    };
    let assert_stage_captions_off = |text: &str| {
        for caption in [
            "Sent",
            "Pending",
            "Soon",
            "Next Block",
            "Sign In",
            "Liquid Asset",
        ] {
            assert!(
                !text.lines().any(|line| line == caption),
                "{caption} was painted: {text}"
            );
        }
    };

    let confirmed = paint(
        "blockheighton",
        r#"{"confirmed":true,"block_height":4}"#,
        Some("4"),
    );
    assert!(
        confirmed.lines().any(|line| line == LINE),
        "Block height is absent: {confirmed}"
    );
    assert_eq!(
        confirmed.lines().filter(|line| *line == LINE).count(),
        1,
        "Block height was not painted once: {confirmed}"
    );
    assert!(
        confirmed
            .lines()
            .filter(|line| line.contains(LINE))
            .all(|line| line == LINE),
        "Block height was painted inside a longer sentence: {confirmed}"
    );
    assert_eq!(
        confirmed.matches(LINE).count(),
        1,
        "Block height was not once: {confirmed}"
    );
    let lines: Vec<&str> = confirmed.lines().collect();
    let confirmations_at = lines.iter().position(|line| *line == "confirmations 1");
    let caption_at = lines.iter().position(|line| *line == LINE);
    assert!(
        confirmations_at.is_some() && caption_at == confirmations_at.map(|index| index + 1),
        "Block height was not the line after confirmations 1: {confirmed}"
    );
    assert!(
        !confirmed.lines().any(|line| line == "Confirmations 1"),
        "the confirmation line changed letter case: {confirmed}"
    );
    assert!(
        confirmed.contains("block_height 4"),
        "the existing height value left the transaction row: {confirmed}"
    );
    assert!(
        !confirmed
            .lines()
            .any(|line| line == "Block height 4" || line.starts_with("Block height ")),
        "the height was painted as a second caption: {confirmed}"
    );
    assert_stage_captions_off(&confirmed);
    assert_paint_is_not_html(&confirmed);

    let unconfirmed = paint("blockheightoff", r#"{"confirmed":false}"#, Some("4"));
    assert!(
        unconfirmed.contains("tx txid blockheightoff"),
        "an unconfirmed transaction was hidden: {unconfirmed}"
    );
    assert_caption_off(&unconfirmed);
    assert!(
        unconfirmed.lines().any(|line| line == "Unconfirmed"),
        "Unconfirmed left the screen: {unconfirmed}"
    );

    let no_height = paint("blockheightnone", r#"{"confirmed":true}"#, Some("4"));
    assert!(
        no_height.contains("tx txid blockheightnone"),
        "a confirmed transaction without a height was hidden: {no_height}"
    );
    assert_caption_off(&no_height);

    let height_unconfirmed = paint(
        "blockheightfalse",
        r#"{"confirmed":false,"block_height":4}"#,
        Some("4"),
    );
    assert!(
        height_unconfirmed
            .lines()
            .any(|line| line == "confirmations 1"),
        "a height on an unconfirmed status dropped confirmations 1: {height_unconfirmed}"
    );
    assert_caption_off(&height_unconfirmed);

    let no_tip = paint(
        "blockheightnotip",
        r#"{"confirmed":true,"block_height":4}"#,
        None,
    );
    assert!(
        no_tip.lines().any(|line| line == "Confirmed"),
        "a missing tip dropped Confirmed: {no_tip}"
    );
    assert!(
        no_tip.lines().any(|line| line == LINE),
        "Block height is absent beside Confirmed: {no_tip}"
    );
    let no_tip_lines: Vec<&str> = no_tip.lines().collect();
    let confirmed_word_at = no_tip_lines.iter().position(|line| *line == "Confirmed");
    let no_tip_caption = no_tip_lines.iter().position(|line| *line == LINE);
    assert!(
        confirmed_word_at.is_some() && no_tip_caption == confirmed_word_at.map(|index| index + 1),
        "Block height was not the line after Confirmed: {no_tip}"
    );
    assert_stage_captions_off(&no_tip);
}

/// Status footer label from no-query `GET /mempool`.
///
/// `footer.component.html` paints the exact line `Unconfirmed:` beside the
/// unconfirmed count (`BacklogStats.count`). The count stays on the existing
/// `mempool count` line. A zero count still paints the label. A body with no
/// count does not. `Incoming Transactions`, `vB/s`, and `WU/s` stay
/// off. The confirmation word `Unconfirmed` stays off this screen. The path
/// has no query string and no fragment.
#[test]
fn status_footer_paints_unconfirmed_label() {
    const LINE: &str = "Unconfirmed:";
    const COUNT: &str = "mempool count 4 vsize 500 total_fee 90";
    const ZERO: &str = "mempool count 0 vsize 0 total_fee 0";

    fn summary_bodies(body: &str) -> BTreeMap<String, String> {
        BTreeMap::from([
            ("/mempool".to_string(), body.to_string()),
            ("/mempool/recent".to_string(), "[]".to_string()),
        ])
    }

    fn assert_no_query(paths: &[String]) {
        assert!(
            paths.iter().any(|path| path == "/mempool"),
            "did not GET /mempool: {paths:?}"
        );
        assert!(
            paths
                .iter()
                .all(|path| !path.contains('?') && !path.contains('#')),
            "{paths:?}"
        );
    }

    fn assert_label_once(text: &str, count_line: &str) {
        assert!(
            text.lines().any(|line| line == LINE),
            "{LINE} is absent: {text}"
        );
        assert_eq!(
            text.lines().filter(|line| *line == LINE).count(),
            1,
            "{LINE} was not painted once: {text}"
        );
        assert!(
            text.lines()
                .filter(|line| line.contains(LINE))
                .all(|line| line == LINE),
            "{LINE} was painted inside a longer sentence: {text}"
        );
        assert_eq!(
            text.matches(LINE).count(),
            1,
            "{LINE} was not once in the summary: {text}"
        );
        let lines: Vec<&str> = text.lines().collect();
        let label_at = lines.iter().position(|line| *line == LINE);
        let count_at = lines.iter().position(|line| *line == count_line);
        assert!(
            label_at.is_some() && count_at == label_at.map(|index| index + 1),
            "{LINE} was not the line before {count_line}: {text}"
        );
        assert!(
            !text.lines().any(|line| line == "Unconfirmed"),
            "the confirmation word was painted on the mempool summary: {text}"
        );
        assert!(
            !text.contains("Incoming Transactions"),
            "Incoming Transactions was painted: {text}"
        );
        assert!(!text.contains("vB/s"), "vB/s was painted: {text}");
        assert!(!text.contains("WU/s"), "WU/s was painted: {text}");
        assert_paint_is_not_html(text);
    }

    let counted = r#"{"count":4,"vsize":500,"total_fee":90,"fee_histogram":[[2,100]]}"#;
    let (mut host, rec) = host_with(Network::Mainnet, summary_bodies(counted));
    let opened = host.activate(Screen::Mempool);
    assert_no_query(&rec.gets());
    opened.unwrap_or_else(|err| panic!("mempool did not open: {err}"));
    assert_label_once(&painted(&host), COUNT);

    let (mut host, rec) = host_with(
        Network::Mainnet,
        BTreeMap::from([
            ("/blocks".to_string(), "[]".to_string()),
            ("/blocks/tip/height".to_string(), "1".to_string()),
            ("/blocks/tip/hash".to_string(), "abc".to_string()),
            ("/fee-estimates".to_string(), "{}".to_string()),
            ("/mempool".to_string(), counted.to_string()),
            ("/mempool/recent".to_string(), "[]".to_string()),
        ]),
    );
    let opened = host.activate(Screen::Dashboard);
    assert_no_query(&rec.gets());
    opened.unwrap_or_else(|err| panic!("dashboard did not open: {err}"));
    assert_label_once(&painted(&host), COUNT);

    let zero = r#"{"count":0,"vsize":0,"total_fee":0,"fee_histogram":[]}"#;
    let (mut host, rec) = host_with(Network::Mainnet, summary_bodies(zero));
    host.activate(Screen::Mempool)
        .unwrap_or_else(|err| panic!("zero mempool did not open: {err}"));
    assert_no_query(&rec.gets());
    assert_label_once(&painted(&host), ZERO);

    let (mut host, rec) = host_with(
        Network::Mainnet,
        summary_bodies(r#"{"vsize":1,"total_fee":0,"fee_histogram":[]}"#),
    );
    let err = host
        .activate(Screen::Mempool)
        .expect_err("a mempool body with no count must fail");
    assert!(
        err.to_string().contains("count"),
        "missing count was not a parse error: {err}"
    );
    assert_no_query(&rec.gets());
    let missing = painted(&host);
    assert!(
        !missing.lines().any(|line| line == LINE),
        "a body with no count painted {LINE}: {missing}"
    );

    let (mut host, rec) = host_with(
        Network::Mainnet,
        BTreeMap::from([(
            "/blocks".to_string(),
            format!("[{}]", block_json("nostatusfooter")),
        )]),
    );
    host.activate(Screen::Blocks)
        .unwrap_or_else(|err| panic!("blocks list did not open: {err}"));
    let paths = rec.gets();
    assert!(
        paths.iter().any(|path| path == "/blocks"),
        "list did not GET /blocks: {paths:?}"
    );
    assert!(
        paths
            .iter()
            .all(|path| !path.contains('?') && !path.contains('#')),
        "{paths:?}"
    );
    let list = painted(&host);
    assert!(
        !list.lines().any(|line| line == LINE),
        "blocks list painted {LINE}: {list}"
    );
}

/// Status footer label from no-query `GET /mempool`.
///
/// `footer.component.html` paints `Mempool size:` and the raw virtual-byte
/// count (`BacklogStats.vsize`). The digits are not kB or MB, and they are
/// not a projected block count. A zero vsize still paints the label. A body
/// with no vsize does not. `Unconfirmed:` stays the one count caption. The
/// path has no query string and no fragment.
#[test]
fn status_footer_paints_mempool_size_from_vsize() {
    const LINE: &str = "Mempool size:";
    const COUNT: &str = "mempool count 4 vsize 500 total_fee 90";
    const ZERO_COUNT: &str = "mempool count 0 vsize 0 total_fee 0";
    const WIDE_COUNT: &str = "mempool count 2 vsize 1000000 total_fee 1500";

    fn summary_bodies(body: &str) -> BTreeMap<String, String> {
        BTreeMap::from([
            ("/mempool".to_string(), body.to_string()),
            ("/mempool/recent".to_string(), "[]".to_string()),
        ])
    }

    fn assert_no_query(paths: &[String]) {
        assert!(
            paths.iter().any(|path| path == "/mempool"),
            "did not GET /mempool: {paths:?}"
        );
        assert!(
            paths
                .iter()
                .all(|path| !path.contains('?') && !path.contains('#')),
            "{paths:?}"
        );
    }

    fn assert_size_once(text: &str, digits: &str, count_line: &str) {
        assert!(
            text.lines().any(|line| line == LINE),
            "{LINE} is absent: {text}"
        );
        assert_eq!(
            text.lines().filter(|line| *line == LINE).count(),
            1,
            "{LINE} was not painted once: {text}"
        );
        assert!(
            text.lines()
                .filter(|line| line.contains(LINE))
                .all(|line| line == LINE),
            "{LINE} was painted inside a longer sentence: {text}"
        );
        assert_eq!(
            text.matches(LINE).count(),
            1,
            "{LINE} was not once in the summary: {text}"
        );
        let lines: Vec<&str> = text.lines().collect();
        let label_at = lines.iter().position(|line| *line == LINE);
        let value_at = label_at.map(|index| index + 1);
        assert_eq!(
            value_at.and_then(|index| lines.get(index).copied()),
            Some(digits),
            "{LINE} was not followed by the raw vsize {digits}: {text}"
        );
        assert!(
            digits.chars().all(|ch| ch.is_ascii_digit()),
            "the vsize was not raw digits: {digits}"
        );
        assert!(
            !digits.contains(','),
            "the vsize gained a separator: {digits}"
        );
        assert!(
            text.lines().any(|line| line == count_line),
            "the mempool count line was dropped: {text}"
        );
        assert_eq!(
            text.lines().filter(|line| *line == "Unconfirmed:").count(),
            1,
            "Unconfirmed: was painted again: {text}"
        );
        assert!(
            !text.lines().any(|line| line == "Unconfirmed"),
            "the confirmation word was painted on the mempool summary: {text}"
        );
        assert!(
            !text.contains("Incoming Transactions"),
            "Incoming Transactions was painted: {text}"
        );
        assert!(!text.contains("kB"), "kB was painted: {text}");
        assert!(!text.contains("MB"), "MB was painted: {text}");
        assert!(
            !text.contains("1,000,000"),
            "the vsize gained a thousands separator: {text}"
        );
        assert!(
            !lines.iter().any(|line| line.contains("block")),
            "a projected block count was painted: {text}"
        );
        assert_paint_is_not_html(text);
    }

    let counted = r#"{"count":4,"vsize":500,"total_fee":90,"fee_histogram":[[2,100]]}"#;
    let (mut host, rec) = host_with(Network::Mainnet, summary_bodies(counted));
    let opened = host.activate(Screen::Mempool);
    assert_no_query(&rec.gets());
    opened.unwrap_or_else(|err| panic!("mempool did not open: {err}"));
    assert_size_once(&painted(&host), "500", COUNT);

    let (mut host, rec) = host_with(
        Network::Mainnet,
        BTreeMap::from([
            ("/blocks".to_string(), "[]".to_string()),
            ("/blocks/tip/height".to_string(), "1".to_string()),
            ("/blocks/tip/hash".to_string(), "abc".to_string()),
            ("/fee-estimates".to_string(), "{}".to_string()),
            ("/mempool".to_string(), counted.to_string()),
            ("/mempool/recent".to_string(), "[]".to_string()),
        ]),
    );
    let opened = host.activate(Screen::Dashboard);
    assert_no_query(&rec.gets());
    opened.unwrap_or_else(|err| panic!("dashboard did not open: {err}"));
    let dashboard = painted(&host);
    assert!(
        dashboard.lines().any(|line| line == LINE),
        "{LINE} is absent on the dashboard: {dashboard}"
    );
    assert_eq!(
        dashboard.lines().filter(|line| *line == LINE).count(),
        1,
        "{LINE} was not painted once on the dashboard: {dashboard}"
    );
    let dashboard_lines: Vec<&str> = dashboard.lines().collect();
    let dashboard_label = dashboard_lines.iter().position(|line| *line == LINE);
    assert_eq!(
        dashboard_label.and_then(|index| dashboard_lines.get(index + 1).copied()),
        Some("500"),
        "{LINE} was not followed by the raw vsize on the dashboard: {dashboard}"
    );
    assert!(
        dashboard.lines().any(|line| line == COUNT),
        "the dashboard dropped the mempool count line: {dashboard}"
    );
    assert!(!dashboard.contains("kB"), "kB was painted: {dashboard}");
    assert!(!dashboard.contains("MB"), "MB was painted: {dashboard}");
    assert_paint_is_not_html(&dashboard);

    let zero = r#"{"count":0,"vsize":0,"total_fee":0,"fee_histogram":[]}"#;
    let (mut host, rec) = host_with(Network::Mainnet, summary_bodies(zero));
    host.activate(Screen::Mempool)
        .unwrap_or_else(|err| panic!("zero mempool did not open: {err}"));
    assert_no_query(&rec.gets());
    assert_size_once(&painted(&host), "0", ZERO_COUNT);

    let wide = r#"{"count":2,"vsize":1000000,"total_fee":1500,"fee_histogram":[]}"#;
    let (mut host, rec) = host_with(Network::Mainnet, summary_bodies(wide));
    host.activate(Screen::Mempool)
        .unwrap_or_else(|err| panic!("wide mempool did not open: {err}"));
    assert_no_query(&rec.gets());
    assert_size_once(&painted(&host), "1000000", WIDE_COUNT);

    let (mut host, rec) = host_with(
        Network::Mainnet,
        summary_bodies(r#"{"count":4,"total_fee":90,"fee_histogram":[]}"#),
    );
    let err = host
        .activate(Screen::Mempool)
        .expect_err("a mempool body with no vsize must fail");
    assert!(
        err.to_string().contains("vsize"),
        "missing vsize was not a parse error: {err}"
    );
    assert_no_query(&rec.gets());
    let missing = painted(&host);
    assert!(
        !missing.lines().any(|line| line == LINE),
        "a body with no vsize painted {LINE}: {missing}"
    );

    let (mut host, rec) = host_with(
        Network::Mainnet,
        BTreeMap::from([(
            "/blocks".to_string(),
            format!("[{}]", block_json("nomempoolsize")),
        )]),
    );
    host.activate(Screen::Blocks)
        .unwrap_or_else(|err| panic!("blocks list did not open: {err}"));
    let paths = rec.gets();
    assert!(
        paths.iter().any(|path| path == "/blocks"),
        "list did not GET /blocks: {paths:?}"
    );
    assert!(
        paths
            .iter()
            .all(|path| !path.contains('?') && !path.contains('#')),
        "{paths:?}"
    );
    let list = painted(&host);
    assert!(
        !list.lines().any(|line| line == LINE),
        "blocks list painted {LINE}: {list}"
    );
}

/// Address heading when `GET /address/:addr/utxo` has more than two rows.
///
/// `address.component.html` paints the exact line `Unspent Outputs` above
/// those rows. Two rows, an empty list, or a body that does not parse leaves
/// the rows without that heading. This is not `Unspent TXOs`. The path has
/// no query string and no fragment.
#[test]
fn address_screen_paints_unspent_outputs() {
    const LINE: &str = "Unspent Outputs";
    const ADDRESS: &str = "bc1qunspent";
    const TXID: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";

    fn one_utxo(vout: u32, value: u64) -> String {
        format!(
            r#"{{"txid":"{TXID}","vout":{vout},"status":{{"confirmed":true,"block_height":1}},"value":{value}}}"#
        )
    }

    fn utxos_json(count: usize) -> String {
        let rows: Vec<String> = (0..count)
            .map(|index| one_utxo(index as u32, 1000 + index as u64))
            .collect();
        format!("[{}]", rows.join(","))
    }

    fn open(utxo_body: &str) -> (ScreenHost<Rec>, Rec) {
        let (mut host, rec) = host_with(
            Network::Mainnet,
            BTreeMap::from([
                (format!("/address/{ADDRESS}"), address_stats(ADDRESS)),
                (format!("/address/{ADDRESS}/txs"), "[]".to_string()),
                (format!("/address/{ADDRESS}/utxo"), utxo_body.to_string()),
            ]),
        );
        host.open_address(ADDRESS)
            .unwrap_or_else(|err| panic!("address did not open: {err}"));
        let paths = rec.gets();
        assert!(
            paths
                .iter()
                .any(|path| path == &format!("/address/{ADDRESS}/utxo")),
            "did not GET /address/:addr/utxo: {paths:?}"
        );
        assert!(
            paths
                .iter()
                .all(|path| !path.contains('?') && !path.contains('#')),
            "{paths:?}"
        );
        (host, rec)
    }

    let (host, _) = open(&utxos_json(3));
    let three = painted(&host);
    assert!(
        three.lines().any(|line| line == LINE),
        "{LINE} is absent: {three}"
    );
    assert_eq!(
        three.lines().filter(|line| *line == LINE).count(),
        1,
        "{LINE} was not painted once: {three}"
    );
    let lines: Vec<&str> = three.lines().collect();
    let heading_at = lines
        .iter()
        .position(|line| *line == LINE)
        .expect("heading");
    let first_utxo = lines
        .iter()
        .position(|line| line.contains("value 1000"))
        .expect("the first raw value was dropped");
    assert!(
        heading_at < first_utxo,
        "{LINE} was not above the utxo rows: {three}"
    );
    assert!(
        three.contains("value 1002"),
        "the third raw value was dropped: {three}"
    );

    let (host, _) = open(&utxos_json(2));
    let two = painted(&host);
    assert!(
        !two.lines().any(|line| line == LINE),
        "two utxos painted {LINE}: {two}"
    );
    assert!(
        two.contains("value 1001"),
        "the two-row value was dropped: {two}"
    );

    let (host, _) = open("[]");
    let empty = painted(&host);
    assert!(
        !empty.lines().any(|line| line == LINE),
        "an empty utxo list painted {LINE}: {empty}"
    );
    assert!(
        empty.contains("The utxo list is empty."),
        "the empty utxo list was dropped: {empty}"
    );

    let (mut host, rec) = host_with(
        Network::Mainnet,
        BTreeMap::from([
            (format!("/address/{ADDRESS}"), address_stats(ADDRESS)),
            (format!("/address/{ADDRESS}/txs"), "[]".to_string()),
            (format!("/address/{ADDRESS}/utxo"), "not-json".to_string()),
        ]),
    );
    let err = host
        .open_address(ADDRESS)
        .expect_err("an unparsed utxo body must fail");
    assert!(
        !err.to_string().contains(LINE),
        "the parse error invented {LINE}: {err}"
    );
    let paths = rec.gets();
    assert!(
        paths
            .iter()
            .any(|path| path == &format!("/address/{ADDRESS}/utxo")),
        "did not GET /address/:addr/utxo: {paths:?}"
    );
    assert!(
        paths
            .iter()
            .all(|path| !path.contains('?') && !path.contains('#')),
        "{paths:?}"
    );
    let unparsed = painted(&host);
    assert!(
        !unparsed.lines().any(|line| line == LINE),
        "an unparsed body painted {LINE}: {unparsed}"
    );
}

/// Heading from no-query `GET /blocks` on the dashboard.
///
/// `dashboard.component.html` paints `Recent Blocks` above the latest blocks.
/// A line equal to `Recent Blocks` passes. A longer line that contains those
/// words does not. An empty list does not paint the heading. A body that
/// does not parse does not. The blocks list screen does not. Liquid still
/// paints the heading. The request path has no query string and no fragment.
/// This is not a clock and not the blocks-list heading `Blocks`.
#[test]
fn dashboard_screen_paints_recent_blocks() {
    const LINE: &str = "Recent Blocks";
    const ID: &str = "recentblk1";

    fn bodies(blocks: &str) -> BTreeMap<String, String> {
        BTreeMap::from([
            ("/blocks".to_string(), blocks.to_string()),
            ("/blocks/tip/height".to_string(), "1".to_string()),
            ("/blocks/tip/hash".to_string(), "abc".to_string()),
            ("/fee-estimates".to_string(), "{}".to_string()),
            (
                "/mempool".to_string(),
                r#"{"count":1,"vsize":9,"total_fee":1,"fee_histogram":[]}"#.to_string(),
            ),
            ("/mempool/recent".to_string(), "[]".to_string()),
        ])
    }

    fn assert_no_query(paths: &[String]) {
        assert!(
            paths.iter().any(|path| path == "/blocks"),
            "did not GET /blocks: {paths:?}"
        );
        assert!(
            paths
                .iter()
                .all(|path| !path.contains('?') && !path.contains('#')),
            "{paths:?}"
        );
    }

    let blocks = format!("[{}]", block_json(ID));
    let (mut host, rec) = host_with(Network::Mainnet, bodies(&blocks));
    let opened = host.activate(Screen::Dashboard);
    assert_no_query(&rec.gets());
    opened.unwrap_or_else(|err| panic!("dashboard did not open: {err}"));
    let text = painted(&host);
    let summary = format!("block hash {ID}");
    assert!(
        text.contains(&summary),
        "the block summary was dropped: {text}"
    );
    assert!(
        text.lines().any(|line| line == LINE),
        "{LINE} is absent: {text}"
    );
    assert_eq!(
        text.lines().filter(|line| *line == LINE).count(),
        1,
        "{LINE} was not painted once: {text}"
    );
    assert!(
        text.lines()
            .filter(|line| line.contains(LINE))
            .all(|line| line == LINE),
        "{LINE} was painted inside a longer sentence: {text}"
    );
    assert_eq!(text.matches(LINE).count(), 1, "{LINE} was not once: {text}");
    assert!(
        !text.contains("Recent Blocks 1700000000"),
        "{LINE} became a clock: {text}"
    );
    assert!(!text.contains("ago"), "{LINE} became a clock: {text}");
    assert_ne!(LINE, "Blocks");
    let lines: Vec<&str> = text.lines().collect();
    let heading_at = lines.iter().position(|line| *line == LINE);
    let summary_at = lines.iter().position(|line| line.contains(&summary));
    assert!(
        heading_at.is_some() && summary_at.is_some() && heading_at < summary_at,
        "{LINE} was not above the block rows: {text}"
    );
    assert_paint_is_not_html(&text);

    let (mut host, rec) = host_with(Network::Liquid, bodies(&blocks));
    let opened = host.activate(Screen::Dashboard);
    assert_no_query(&rec.gets());
    opened.unwrap_or_else(|err| panic!("liquid dashboard did not open: {err}"));
    let liquid = painted(&host);
    assert!(
        liquid.contains(&summary),
        "the liquid block summary was hidden: {liquid}"
    );
    assert!(
        liquid.lines().any(|line| line == LINE),
        "liquid omitted {LINE}: {liquid}"
    );
    assert_eq!(
        liquid.lines().filter(|line| *line == LINE).count(),
        1,
        "liquid painted {LINE} more than once: {liquid}"
    );

    let (mut host, rec) = host_with(Network::Mainnet, bodies("[]"));
    let opened = host.activate(Screen::Dashboard);
    assert_no_query(&rec.gets());
    opened.unwrap_or_else(|err| panic!("empty dashboard did not open: {err}"));
    let empty = painted(&host);
    assert!(
        empty.contains("blocks:"),
        "empty list dropped the blocks marker: {empty}"
    );
    assert!(
        !empty.lines().any(|line| line == LINE),
        "an empty block list painted {LINE}: {empty}"
    );

    let (mut host, rec) = host_with(Network::Mainnet, bodies("not-json"));
    let err = host
        .activate(Screen::Dashboard)
        .expect_err("an unparsed block list must fail");
    assert!(
        !err.to_string().contains(LINE),
        "the parse error invented {LINE}: {err}"
    );
    assert_no_query(&rec.gets());
    let unparsed = painted(&host);
    assert!(
        !unparsed.lines().any(|line| line == LINE),
        "an unparsed body painted {LINE}: {unparsed}"
    );

    let (mut host, rec) = host_with(
        Network::Mainnet,
        BTreeMap::from([("/blocks".to_string(), blocks)]),
    );
    host.activate(Screen::Blocks)
        .unwrap_or_else(|err| panic!("blocks list did not open: {err}"));
    let paths = rec.gets();
    assert!(
        paths.iter().any(|path| path == "/blocks"),
        "list did not GET /blocks: {paths:?}"
    );
    assert!(
        paths
            .iter()
            .all(|path| !path.contains('?') && !path.contains('#')),
        "{paths:?}"
    );
    let list = painted(&host);
    assert!(
        !list.lines().any(|line| line == LINE),
        "blocks list painted {LINE}: {list}"
    );
    assert!(
        list.lines().any(|line| line == "Blocks"),
        "blocks list dropped Blocks: {list}"
    );
}

/// Column from no-query `GET /blocks` on the dashboard.
///
/// `dashboard.component.html` paints `Mined` beside `block.timestamp`.
/// A line equal to `Mined` passes. A longer line that contains that word
/// does not. An empty list does not paint it. A body that does not parse
/// does not. The blocks list screen does not. Liquid still paints it.
/// The request path has no query string and no fragment. This is not a
/// clock and not the blocks-list label `Timestamp`. The raw timestamp stays.
#[test]
fn dashboard_screen_paints_mined() {
    const LINE: &str = "Mined";
    const ID: &str = "minedblk1";
    const SECOND: &str = "minedblk2";
    const TIMESTAMP: &str = "1700000000";
    const LATER: &str = "1699990000";

    fn bodies(blocks: &str) -> BTreeMap<String, String> {
        BTreeMap::from([
            ("/blocks".to_string(), blocks.to_string()),
            ("/blocks/tip/height".to_string(), "1".to_string()),
            ("/blocks/tip/hash".to_string(), "abc".to_string()),
            ("/fee-estimates".to_string(), "{}".to_string()),
            (
                "/mempool".to_string(),
                r#"{"count":1,"vsize":9,"total_fee":1,"fee_histogram":[]}"#.to_string(),
            ),
            ("/mempool/recent".to_string(), "[]".to_string()),
        ])
    }

    fn assert_no_query(paths: &[String]) {
        assert!(
            paths.iter().any(|path| path == "/blocks"),
            "did not GET /blocks: {paths:?}"
        );
        assert!(
            paths
                .iter()
                .all(|path| !path.contains('?') && !path.contains('#')),
            "{paths:?}"
        );
    }

    fn assert_mined_beside_timestamp(text: &str, id: &str) {
        assert!(
            text.lines().any(|line| line == LINE),
            "{LINE} is absent: {text}"
        );
        assert_eq!(
            text.lines().filter(|line| *line == LINE).count(),
            1,
            "{LINE} was not painted once: {text}"
        );
        assert!(
            text.lines()
                .filter(|line| line.contains(LINE))
                .all(|line| line == LINE),
            "{LINE} was painted inside a longer sentence: {text}"
        );
        assert_eq!(text.matches(LINE).count(), 1, "{LINE} was not once: {text}");
        assert!(
            !text.contains(&format!("{LINE} {TIMESTAMP}"))
                && !text.contains(&format!("{LINE} {LATER}"))
                && !text.contains("ago"),
            "{LINE} became a clock: {text}"
        );
        assert!(
            text.contains(&format!("timestamp {TIMESTAMP}")),
            "the raw timestamp was dropped: {text}"
        );
        assert!(
            text.contains(&format!("timestamp {LATER}")),
            "the second raw timestamp was dropped: {text}"
        );
        let lines: Vec<&str> = text.lines().collect();
        let mined_at = lines.iter().position(|line| *line == LINE);
        let stamp_at = lines.iter().position(|line| {
            line.contains(&format!("block hash {id}"))
                && line.contains(&format!("timestamp {TIMESTAMP}"))
        });
        assert!(
            mined_at.is_some() && stamp_at.is_some() && mined_at.unwrap() + 1 == stamp_at.unwrap(),
            "{LINE} was not beside the raw timestamp: {text}"
        );
        assert_paint_is_not_html(text);
    }

    let second = block_json(SECOND).replacen(
        &format!("\"timestamp\":{TIMESTAMP}"),
        &format!("\"timestamp\":{LATER}"),
        1,
    );
    let blocks = format!("[{},{}]", block_json(ID), second);
    let (mut host, rec) = host_with(Network::Mainnet, bodies(&blocks));
    let opened = host.activate(Screen::Dashboard);
    assert_no_query(&rec.gets());
    opened.unwrap_or_else(|err| panic!("dashboard did not open: {err}"));
    let text = painted(&host);
    assert!(
        text.lines().any(|line| line == "Recent Blocks"),
        "Recent Blocks was dropped: {text}"
    );
    assert!(
        text.lines().any(|line| line == "Size"),
        "Size was dropped: {text}"
    );
    assert_mined_beside_timestamp(&text, ID);
    let lines: Vec<&str> = text.lines().collect();
    let recent_at = lines.iter().position(|line| *line == "Recent Blocks");
    let size_at = lines.iter().position(|line| *line == "Size");
    let mined_at = lines.iter().position(|line| *line == LINE);
    assert!(
        recent_at.is_some()
            && size_at.is_some()
            && mined_at.is_some()
            && recent_at < size_at
            && size_at < mined_at,
        "Mined was not after Recent Blocks and Size: {text}"
    );
    assert!(
        !text.lines().any(|line| line == "Timestamp"),
        "Mined became the Timestamp label: {text}"
    );

    let (mut host, rec) = host_with(Network::Liquid, bodies(&blocks));
    let opened = host.activate(Screen::Dashboard);
    assert_no_query(&rec.gets());
    opened.unwrap_or_else(|err| panic!("liquid dashboard did not open: {err}"));
    let liquid = painted(&host);
    assert_mined_beside_timestamp(&liquid, ID);
    assert!(
        liquid.lines().any(|line| line == "Recent Blocks"),
        "liquid dropped Recent Blocks: {liquid}"
    );

    let (mut host, rec) = host_with(Network::Mainnet, bodies("[]"));
    let opened = host.activate(Screen::Dashboard);
    assert_no_query(&rec.gets());
    opened.unwrap_or_else(|err| panic!("empty dashboard did not open: {err}"));
    let empty = painted(&host);
    assert!(
        empty.contains("blocks:"),
        "empty list dropped the blocks marker: {empty}"
    );
    assert!(
        !empty.lines().any(|line| line == LINE),
        "an empty block list painted {LINE}: {empty}"
    );

    let (mut host, rec) = host_with(Network::Mainnet, bodies("not-json"));
    let err = host
        .activate(Screen::Dashboard)
        .expect_err("an unparsed block list must fail");
    assert!(
        !err.to_string().contains(LINE),
        "the parse error invented {LINE}: {err}"
    );
    assert_no_query(&rec.gets());
    let unparsed = painted(&host);
    assert!(
        !unparsed.lines().any(|line| line == LINE),
        "an unparsed body painted {LINE}: {unparsed}"
    );

    let (mut host, rec) = host_with(
        Network::Mainnet,
        BTreeMap::from([("/blocks".to_string(), blocks)]),
    );
    host.activate(Screen::Blocks)
        .unwrap_or_else(|err| panic!("blocks list did not open: {err}"));
    let paths = rec.gets();
    assert!(
        paths.iter().any(|path| path == "/blocks"),
        "list did not GET /blocks: {paths:?}"
    );
    assert!(
        paths
            .iter()
            .all(|path| !path.contains('?') && !path.contains('#')),
        "{paths:?}"
    );
    let list = painted(&host);
    assert!(
        list.contains(&format!("block hash {ID}")),
        "blocks list dropped the row: {list}"
    );
    assert!(
        list.contains(&format!("timestamp {TIMESTAMP}")),
        "blocks list dropped the raw timestamp: {list}"
    );
    assert!(
        !list.lines().any(|line| line == LINE),
        "blocks list painted {LINE}: {list}"
    );
    assert!(!list.contains(LINE), "blocks list gained {LINE}: {list}");
}

/// Details word from a no-query `GET /tx/:txid`.
///
/// Angular paints the exact word `data` on an `op_return` output.
/// A non-`op_return` output does not. An input prevout does not.
/// A script that starts with `6a5d` still paints `data` and keeps `Runestone`.
/// Script hex and script asm stay off. This is not `OP_RETURN`.
#[test]
fn transaction_screen_paints_op_return_data() {
    const LINE: &str = "data";
    const SCRIPT: &str = "ZZPAYLOADHEXZZ";
    const ASM: &str = "ZZPAYLOADASMZZ";
    const STONE: &str = "6a5d0614c0a2331441ee";
    const TXID: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
    const PREV: &str = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";

    fn vout(script: &str, asm: &str, kind: &str) -> String {
        format!(
            r#"{{"scriptpubkey":"{script}","scriptpubkey_asm":"{asm}","scriptpubkey_type":"{kind}","value":0}}"#
        )
    }

    fn paint(vin_json: &str, vout_json: &str) -> String {
        let body = format!(
            r#"{{"txid":"{TXID}","version":1,"locktime":0,"vin":[{vin_json}],"vout":[{vout_json}],"size":10,"weight":40,"sigops":0,"fee":1,"status":{{"confirmed":false}}}}"#
        );
        let (mut host, rec) = host_with(
            Network::Mainnet,
            BTreeMap::from([(format!("/tx/{TXID}"), body)]),
        );
        host.open_transaction(TXID)
            .unwrap_or_else(|err| panic!("tx did not open: {err}"));
        let paths = rec.gets();
        assert!(
            paths.iter().any(|path| path == &format!("/tx/{TXID}")),
            "did not GET /tx/:txid: {paths:?}"
        );
        assert!(
            paths
                .iter()
                .all(|path| !path.contains('?') && !path.contains('#')),
            "{paths:?}"
        );
        painted(&host)
    }

    fn empty_vin() -> String {
        format!(
            r#"{{"txid":"{PREV}","vout":0,"prevout":null,"scriptsig":"SIGHEXZZ","scriptsig_asm":"SIGASMZZ","is_coinbase":false,"sequence":1}}"#
        )
    }

    let text = paint(&empty_vin(), &vout(SCRIPT, ASM, "op_return"));
    assert!(
        text.lines().any(|line| line == LINE),
        "{LINE} is absent: {text}"
    );
    assert_eq!(
        text.lines().filter(|line| *line == LINE).count(),
        1,
        "{LINE} was not painted once: {text}"
    );
    let lines: Vec<&str> = text.lines().collect();
    let op_at = lines.iter().position(|line| *line == "OP_RETURN");
    let data_at = lines.iter().position(|line| *line == LINE);
    assert!(
        op_at.is_some() && data_at.is_some() && op_at < data_at,
        "{LINE} was not after OP_RETURN: {text}"
    );
    assert!(!text.contains(SCRIPT), "script hex was painted: {text}");
    assert!(!text.contains(ASM), "script asm was painted: {text}");
    assert!(
        !text.contains("SIGHEXZZ"),
        "scriptsig hex was painted: {text}"
    );
    assert!(
        !text.contains("SIGASMZZ"),
        "scriptsig asm was painted: {text}"
    );

    let stone = paint(&empty_vin(), &vout(STONE, ASM, "op_return"));
    assert_eq!(
        stone.lines().filter(|line| *line == LINE).count(),
        1,
        "a runestone output did not paint {LINE} once: {stone}"
    );
    assert!(
        stone.lines().any(|line| line == "Runestone"),
        "Runestone was dropped: {stone}"
    );
    let stone_lines: Vec<&str> = stone.lines().collect();
    let rune_at = stone_lines.iter().position(|line| *line == "Runestone");
    let stone_data = stone_lines.iter().position(|line| *line == LINE);
    assert!(
        rune_at.is_some() && stone_data.is_some() && rune_at < stone_data,
        "{LINE} was not after Runestone: {stone}"
    );
    assert!(
        !stone.contains(STONE),
        "runestone script hex was painted: {stone}"
    );
    assert!(
        !stone.contains(ASM),
        "runestone script asm was painted: {stone}"
    );

    let other = paint(&empty_vin(), &vout(SCRIPT, ASM, "p2pkh"));
    assert!(
        !other.lines().any(|line| line == LINE),
        "a p2pkh output painted {LINE}: {other}"
    );
    assert!(
        other.lines().any(|line| line == "Type P2PKH"),
        "Type P2PKH was dropped: {other}"
    );
    assert!(
        !other.contains(SCRIPT),
        "p2pkh script hex was painted: {other}"
    );
    assert!(
        !other.contains(ASM),
        "p2pkh script asm was painted: {other}"
    );

    let prevout = vout(SCRIPT, ASM, "op_return");
    let fee_input = format!(
        r#"{{"txid":"{PREV}","vout":0,"prevout":{prevout},"scriptsig":"SIGHEXZZ","scriptsig_asm":"SIGASMZZ","is_coinbase":false,"sequence":1}}"#
    );
    let input_only = paint(&fee_input, &vout("0014aa", "OUTASMZZ", "p2pkh"));
    assert!(
        !input_only.lines().any(|line| line == LINE),
        "an input prevout painted {LINE}: {input_only}"
    );
    assert!(
        input_only
            .lines()
            .any(|line| line == "Previous output type OP_RETURN"),
        "Previous output type OP_RETURN was dropped: {input_only}"
    );
    assert!(
        !input_only.contains(SCRIPT),
        "input script hex was painted: {input_only}"
    );
    assert!(
        !input_only.contains("OUTASMZZ"),
        "output asm was painted: {input_only}"
    );
}

/// Empty address transactions and empty mempool recent paint one exact line.
///
/// The Leptos page paints `There are no transactions.` when `GET /address/:addr/txs`
/// or `GET /mempool/recent` is an empty array, including whitespace inside the
/// brackets. A loaded transaction, a recent row, and an empty block transaction
/// list do not. `The address transaction list is empty.` stays. The block list
/// keeps `The transaction list is empty.` and does not gain this sentence.
#[test]
fn screen_paints_there_are_no_transactions() {
    const SENTENCE: &str = "There are no transactions.";
    const ADDR: &str = "bc1qnotransactions";
    const ONE: &str = r#"{"txid":"addrconfirmed","version":1,"locktime":0,"vin":[],"vout":[],"size":10,"weight":40,"sigops":0,"fee":1,"status":{"confirmed":true,"block_height":3}}"#;
    const RECENT: &str = r#"[{"txid":"recent-one","fee":10,"vsize":141,"value":1}]"#;

    fn assert_once(text: &str) {
        assert!(
            text.lines().any(|line| line == SENTENCE),
            "{SENTENCE} is absent: {text}"
        );
        assert_eq!(
            text.lines().filter(|line| *line == SENTENCE).count(),
            1,
            "{SENTENCE} was not painted once: {text}"
        );
    }

    fn assert_absent(text: &str) {
        assert!(
            !text.lines().any(|line| line == SENTENCE),
            "{SENTENCE} was painted: {text}"
        );
    }

    fn open_address_txs(txs: &str) -> (ScreenHost<Rec>, Rec) {
        let (mut host, rec) = host_with(
            Network::Mainnet,
            BTreeMap::from([
                (format!("/address/{ADDR}"), address_stats(ADDR)),
                (format!("/address/{ADDR}/txs"), txs.to_string()),
                (format!("/address/{ADDR}/utxo"), "[]".to_string()),
            ]),
        );
        host.open_address(ADDR)
            .unwrap_or_else(|err| panic!("address did not open for {txs:?}: {err}"));
        let paths = rec.gets();
        assert!(
            paths
                .iter()
                .any(|path| path == &format!("/address/{ADDR}/txs")),
            "did not GET /address/:addr/txs: {paths:?}"
        );
        assert!(
            paths
                .iter()
                .all(|path| !path.contains('?') && !path.contains('#')),
            "{paths:?}"
        );
        (host, rec)
    }

    let (empty, _) = open_address_txs("[]");
    let empty_text = painted(&empty);
    assert_once(&empty_text);
    assert!(
        empty_text
            .lines()
            .any(|line| line == "The address transaction list is empty."),
        "The address transaction list is empty. was dropped: {empty_text}"
    );

    let (spaced, _) = open_address_txs(" [ ] ");
    assert_once(&painted(&spaced));

    let (loaded, _) = open_address_txs(&format!("[{ONE}]"));
    let loaded_text = painted(&loaded);
    assert_absent(&loaded_text);
    assert!(
        loaded_text.contains("addrconfirmed"),
        "the loaded transaction was dropped: {loaded_text}"
    );

    let (mut broken, _) = host_with(
        Network::Mainnet,
        BTreeMap::from([
            (format!("/address/{ADDR}"), address_stats(ADDR)),
            (format!("/address/{ADDR}/txs"), "not-json".to_string()),
            (format!("/address/{ADDR}/utxo"), "[]".to_string()),
        ]),
    );
    assert!(
        broken.open_address(ADDR).is_err(),
        "unparsed address transactions opened"
    );
    assert_absent(&painted(&broken));

    let (mut recent_empty, recent_rec) = host_with(
        Network::Mainnet,
        BTreeMap::from([
            (
                "/mempool".to_string(),
                r#"{"count":0,"vsize":0,"total_fee":0,"fee_histogram":[]}"#.to_string(),
            ),
            ("/mempool/recent".to_string(), "[]".to_string()),
        ]),
    );
    recent_empty
        .activate(Screen::Mempool)
        .expect("empty recent");
    let recent_paths = recent_rec.gets();
    assert!(
        recent_paths.iter().any(|path| path == "/mempool/recent"),
        "did not GET /mempool/recent: {recent_paths:?}"
    );
    assert!(
        recent_paths
            .iter()
            .all(|path| !path.contains('?') && !path.contains('#')),
        "{recent_paths:?}"
    );
    let recent_text = painted(&recent_empty);
    assert_once(&recent_text);
    assert!(
        recent_text.lines().any(|line| line == "recent:"),
        "recent: was dropped: {recent_text}"
    );

    let (mut recent_spaced, _) = host_with(
        Network::Mainnet,
        BTreeMap::from([
            (
                "/mempool".to_string(),
                r#"{"count":0,"vsize":0,"total_fee":0,"fee_histogram":[]}"#.to_string(),
            ),
            ("/mempool/recent".to_string(), "[\n]".to_string()),
        ]),
    );
    recent_spaced
        .activate(Screen::Mempool)
        .expect("spaced recent");
    assert_once(&painted(&recent_spaced));

    let (mut recent_one, _) = host_with(
        Network::Mainnet,
        BTreeMap::from([
            (
                "/mempool".to_string(),
                r#"{"count":1,"vsize":141,"total_fee":10,"fee_histogram":[]}"#.to_string(),
            ),
            ("/mempool/recent".to_string(), RECENT.to_string()),
        ]),
    );
    recent_one.activate(Screen::Mempool).expect("one recent");
    let one_text = painted(&recent_one);
    assert_absent(&one_text);
    assert!(
        one_text.contains("recent txid recent-one"),
        "the recent row was dropped: {one_text}"
    );

    let hash = "emptyblocktxs";
    let (mut block, _) = host_with(
        Network::Mainnet,
        BTreeMap::from([
            (format!("/block/{hash}"), block_json(hash)),
            (format!("/block/{hash}/txs"), "[]".to_string()),
            (format!("/block/{hash}/txids"), "[]".to_string()),
        ]),
    );
    block.open_block(hash).expect("block");
    let block_text = painted(&block);
    assert_absent(&block_text);
    assert!(
        block_text
            .lines()
            .any(|line| line == "The transaction list is empty."),
        "The transaction list is empty. was dropped: {block_text}"
    );
}

/// Empty address and scripthash utxo lists paint one exact line.
///
/// The Leptos page paints `There are no unspent outputs.` when
/// `GET /address/:addr/utxo` or `GET /scripthash/:hash/utxo` is an empty
/// array, including whitespace inside the brackets. A loaded utxo, a blank
/// body, and invalid JSON do not. `The utxo list is empty.` stays on the
/// next line. `Unspent Outputs` stays off an empty list.
#[test]
fn address_screen_paints_there_are_no_unspent_outputs() {
    const SENTENCE: &str = "There are no unspent outputs.";
    const NEXT: &str = "The utxo list is empty.";
    const ADDR: &str = "bc1qnounspent";
    const HASH: &str = "abababababababababababababababababababababababababababababababab";
    const ONE: &str = r#"[{"txid":"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa","vout":0,"value":1,"status":{"confirmed":true,"block_height":1}}]"#;

    fn assert_once(text: &str) {
        assert!(
            text.lines().any(|line| line == SENTENCE),
            "{SENTENCE} is absent: {text}"
        );
        assert_eq!(
            text.lines().filter(|line| *line == SENTENCE).count(),
            1,
            "{SENTENCE} was not painted once: {text}"
        );
        let lines: Vec<&str> = text.lines().collect();
        let sentence_at = lines
            .iter()
            .position(|line| *line == SENTENCE)
            .expect("sentence");
        let next_at = lines
            .iter()
            .position(|line| *line == NEXT)
            .unwrap_or_else(|| panic!("{NEXT} was dropped: {text}"));
        assert_eq!(
            sentence_at + 1,
            next_at,
            "{SENTENCE} does not come immediately before {NEXT}: {text}"
        );
        assert!(
            !lines.iter().any(|line| *line == "Unspent Outputs"),
            "an empty utxo list painted Unspent Outputs: {text}"
        );
    }

    fn assert_absent(text: &str) {
        assert!(
            !text.lines().any(|line| line == SENTENCE),
            "{SENTENCE} was painted: {text}"
        );
    }

    fn open_utxo(script: &str, stats: String, utxo: &str) -> String {
        let prefix = if script.len() == 64 {
            "scripthash"
        } else {
            "address"
        };
        let (mut host, rec) = host_with(
            Network::Mainnet,
            BTreeMap::from([
                (format!("/{prefix}/{script}"), stats),
                (format!("/{prefix}/{script}/txs"), "[]".to_string()),
                (format!("/{prefix}/{script}/utxo"), utxo.to_string()),
            ]),
        );
        host.open_address(script)
            .unwrap_or_else(|err| panic!("{prefix} did not open for {utxo:?}: {err}"));
        let paths = rec.gets();
        assert!(
            paths
                .iter()
                .any(|path| path == &format!("/{prefix}/{script}/utxo")),
            "did not GET /{prefix}/:script/utxo: {paths:?}"
        );
        assert!(
            paths
                .iter()
                .all(|path| !path.contains('?') && !path.contains('#')),
            "{paths:?}"
        );
        painted(&host)
    }

    assert_once(&open_utxo(ADDR, address_stats(ADDR), "[]"));
    assert_once(&open_utxo(ADDR, address_stats(ADDR), " [ ] "));
    assert_once(&open_utxo(HASH, loaded_scripthash_stats(HASH, 1, 0), "[]"));

    let loaded = open_utxo(ADDR, address_stats(ADDR), ONE);
    assert_absent(&loaded);
    assert!(
        loaded.contains("value 1"),
        "the loaded utxo was dropped: {loaded}"
    );
    assert_absent(&open_utxo(HASH, loaded_scripthash_stats(HASH, 1, 0), ONE));

    let (mut broken, _) = host_with(
        Network::Mainnet,
        BTreeMap::from([
            (format!("/address/{ADDR}"), address_stats(ADDR)),
            (format!("/address/{ADDR}/txs"), "[]".to_string()),
            (format!("/address/{ADDR}/utxo"), "not-json".to_string()),
        ]),
    );
    assert!(broken.open_address(ADDR).is_err(), "unparsed utxos opened");
    assert_absent(&painted(&broken));

    let (mut blank, _) = host_with(
        Network::Mainnet,
        BTreeMap::from([
            (format!("/address/{ADDR}"), address_stats(ADDR)),
            (format!("/address/{ADDR}/txs"), "[]".to_string()),
            (format!("/address/{ADDR}/utxo"), String::new()),
        ]),
    );
    assert!(
        blank.open_address(ADDR).is_err(),
        "a blank utxo body opened"
    );
    assert_absent(&painted(&blank));

    let hash = "emptyblockutxo";
    let (mut block, _) = host_with(
        Network::Mainnet,
        BTreeMap::from([
            (format!("/block/{hash}"), block_json(hash)),
            (format!("/block/{hash}/txs"), "[]".to_string()),
            (format!("/block/{hash}/txids"), "[]".to_string()),
        ]),
    );
    block.open_block(hash).expect("block");
    assert_absent(&painted(&block));
}

/// Empty address-prefix lists paint one exact line.
///
/// The Leptos page paints `There are no addresses.` when
/// `GET /address-prefix/:prefix` is an empty JSON array, including
/// whitespace inside the brackets. A listed address, an array of blank
/// strings, a disabled search, and a body that does not parse do not.
#[test]
fn dashboard_search_paints_there_are_no_addresses() {
    const SENTENCE: &str = "There are no addresses.";
    const PREFIX: &str = "xyz";
    const PATH: &str = "/address-prefix/xyz";
    const MATCH: &str = "bc1qexample";

    fn assert_once(text: &str) {
        assert!(
            text.lines().any(|line| line == SENTENCE),
            "{SENTENCE} is absent: {text}"
        );
        assert_eq!(
            text.lines().filter(|line| *line == SENTENCE).count(),
            1,
            "{SENTENCE} was not painted once: {text}"
        );
    }

    fn assert_absent(text: &str) {
        assert!(
            !text.lines().any(|line| line == SENTENCE),
            "{SENTENCE} was painted: {text}"
        );
    }

    fn search_prefix(body: &str) -> (Result<(), ClientError>, String, Vec<String>) {
        let (mut host, rec) = host_with(
            Network::Mainnet,
            BTreeMap::from([(PATH.to_string(), body.to_string())]),
        );
        host.activate(Screen::Dashboard).expect("dashboard");
        host.set_query(PREFIX);
        let opened = host.reload();
        let paths = rec.gets();
        assert!(
            paths.iter().any(|path| path == PATH),
            "did not GET /address-prefix/:prefix: {paths:?}"
        );
        assert!(
            paths
                .iter()
                .all(|path| !path.contains('?') && !path.contains('#')),
            "{paths:?}"
        );
        (opened, painted(&host), paths)
    }

    let (opened, empty, _) = search_prefix("[]");
    opened.expect("empty address list");
    assert_once(&empty);
    assert!(
        empty.lines().all(|line| line != MATCH),
        "an empty list painted an address: {empty}"
    );

    let (opened, spaced, _) = search_prefix(" [ ] ");
    opened.expect("whitespace inside empty brackets");
    assert_once(&spaced);

    let (opened, listed, _) = search_prefix(&format!(r#"["{MATCH}"]"#));
    opened.expect("one address");
    assert_absent(&listed);
    assert!(
        listed.contains(MATCH),
        "the listed address was dropped: {listed}"
    );

    let (opened, blanks, _) = search_prefix(r#"["", ""]"#);
    opened.expect("blank address strings");
    assert_absent(&blanks);

    let (opened, disabled, _) = search_prefix("address search disabled");
    opened.expect("disabled address search");
    assert_absent(&disabled);
    assert!(
        disabled.contains("address search disabled"),
        "the disabled sentence was dropped: {disabled}"
    );

    let (opened, broken, _) = search_prefix("not-json");
    assert!(opened.is_err(), "unparsed prefix JSON opened");
    assert_absent(&broken);
}

/// Blinded asset supply paints the indexer sentence.
///
/// The Leptos page paints `Asset supply is blinded` when
/// `GET /asset/:asset/supply` is that sentence. A plain integer supply
/// does not. The route has no query string.
#[test]
fn asset_screen_paints_asset_supply_is_blinded() {
    const SENTENCE: &str = "Asset supply is blinded";
    let asset = "ab".repeat(32);
    let document = format!(
        r#"{{"asset_id":"{asset}","name":"Tether USD","ticker":"USDt","precision":8,"status":{{"confirmed":true}}}}"#
    );

    fn open_asset(
        asset: &str,
        document: &str,
        supply: Option<&str>,
    ) -> (Result<(), ClientError>, String, Vec<String>) {
        let mut bodies = BTreeMap::from([
            (format!("/tx/{asset}"), "Transaction not found".to_string()),
            (format!("/block/{asset}"), "Block not found".to_string()),
            (format!("/asset/{asset}"), document.to_string()),
            (format!("/asset/{asset}/txs"), "[]".to_string()),
            (format!("/asset/{asset}/txs/chain"), "[]".to_string()),
            (format!("/asset/{asset}/txs/mempool"), "[]".to_string()),
        ]);
        if let Some(supply) = supply {
            bodies.insert(format!("/asset/{asset}/supply"), supply.to_string());
        }
        let (mut host, rec) = host_with(Network::Liquid, bodies);
        host.activate(Screen::Dashboard).expect("dashboard");
        host.set_query(asset);
        let opened = host.reload();
        let paths = rec.gets();
        assert!(
            paths
                .iter()
                .any(|path| path == &format!("/asset/{asset}/supply")),
            "did not GET /asset/:asset/supply: {paths:?}"
        );
        assert!(
            paths
                .iter()
                .all(|path| !path.contains('?') && !path.contains('#')),
            "{paths:?}"
        );
        (opened, painted(&host), paths)
    }

    let (opened, blinded, paths) = open_asset(&asset, &document, Some(SENTENCE));
    opened.expect("blinded supply");
    assert!(
        paths
            .iter()
            .any(|path| path == &format!("/asset/{asset}/supply")),
        "did not GET /asset/:asset/supply: {paths:?}"
    );
    assert!(
        blinded.lines().any(|line| line == SENTENCE),
        "{SENTENCE} is absent: {blinded}"
    );
    assert_eq!(
        blinded.lines().filter(|line| *line == SENTENCE).count(),
        1,
        "{SENTENCE} was not painted once: {blinded}"
    );

    let (opened, integer, _) = open_asset(&asset, &document, Some("21000000"));
    opened.expect("integer supply");
    assert!(
        integer.lines().any(|line| line == "21000000"),
        "the integer supply was dropped: {integer}"
    );
    assert!(
        !integer.lines().any(|line| line == SENTENCE),
        "{SENTENCE} was painted for an integer supply: {integer}"
    );

    let (opened, missing, _) = open_asset(&asset, &document, None);
    opened.expect("missing supply");
    assert!(
        !missing.lines().any(|line| line == SENTENCE),
        "{SENTENCE} was painted when supply was missing: {missing}"
    );

    let (mut host, rec) = host_with(
        Network::Mainnet,
        BTreeMap::from([(format!("/asset/{asset}/supply"), SENTENCE.to_string())]),
    );
    host.activate(Screen::Dashboard).expect("mainnet dashboard");
    host.set_query(&asset);
    let _ = host.reload();
    let main_paths = rec.gets();
    assert!(
        main_paths.iter().all(|path| !path.contains("/asset/")),
        "mainnet requested the asset supply route: {main_paths:?}"
    );
    assert!(
        !painted(&host).lines().any(|line| line == SENTENCE),
        "{SENTENCE} was painted on mainnet: {}",
        painted(&host)
    );
}

/// Spent outspend paints `spent true` from `GET /tx/:txid/outspends`.
///
/// The Leptos page paints that exact text when `spent` is true. An unspent
/// output stays `spent false` and does not paint this line. The path has no
/// query string.
#[test]
fn transaction_screen_paints_spent_true() {
    const LINE: &str = "spent true";
    const TXID: &str = "spenttrue";
    const SPEND: &str = "eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee";

    fn open_spends(spends: &str) -> (String, Vec<String>) {
        let tx = format!(
            r#"{{"txid":"{TXID}","version":1,"locktime":0,"vin":[],"vout":[{{"scriptpubkey":"0014aa","scriptpubkey_asm":"OP_0","scriptpubkey_type":"v0_p2wpkh","scriptpubkey_address":"bc1qout","value":9}}],"size":10,"weight":40,"sigops":0,"fee":1,"status":{{"confirmed":true}}}}"#
        );
        let (mut host, rec) = host_with(
            Network::Mainnet,
            BTreeMap::from([
                (format!("/tx/{TXID}"), tx),
                (format!("/tx/{TXID}/outspends"), spends.to_string()),
            ]),
        );
        host.open_transaction(TXID).expect("tx");
        let paths = rec.gets();
        assert!(
            paths
                .iter()
                .any(|path| path == &format!("/tx/{TXID}/outspends")),
            "did not GET /tx/:txid/outspends: {paths:?}"
        );
        assert!(
            paths
                .iter()
                .all(|path| !path.contains('?') && !path.contains('#')),
            "{paths:?}"
        );
        (painted(&host), paths)
    }

    let spent =
        format!(r#"[{{"spent":true,"txid":"{SPEND}","vin":1,"status":{{"confirmed":true}}}}]"#);
    let (text, _) = open_spends(&spent);
    assert!(
        text.lines().any(|line| line == LINE),
        "{LINE} is absent: {text}"
    );
    assert_eq!(
        text.lines().filter(|line| *line == LINE).count(),
        1,
        "{LINE} was not painted once: {text}"
    );
    assert!(
        text.lines()
            .any(|line| line == format!("spend txid {SPEND}")),
        "the spending txid was dropped: {text}"
    );

    let (unspent, _) = open_spends(r#"[{"spent":false}]"#);
    assert!(
        unspent.lines().any(|line| line == "spent false"),
        "spent false is absent: {unspent}"
    );
    assert!(
        !unspent.lines().any(|line| line == LINE),
        "{LINE} was painted for an unspent output: {unspent}"
    );
}

/// Address heading when `GET /address/:addr/txs` has more than two rows.
///
/// `address.component.html` paints the exact line `Balance History` above
/// those rows. Two rows, an empty list, or a body that does not parse leaves
/// the rows without that heading. The path has no query string and no fragment.
#[test]
fn address_screen_paints_balance_history() {
    const LINE: &str = "Balance History";
    const ADDRESS: &str = "bc1qbalancehistory";

    fn one_tx(index: u64) -> String {
        let txid = format!("{:064x}", index + 1);
        let fee = index + 1;
        format!(
            r#"{{"txid":"{txid}","version":1,"locktime":0,"vin":[],"vout":[],"size":10,"weight":40,"sigops":0,"fee":{fee},"status":{{"confirmed":true,"block_height":2}}}}"#
        )
    }

    fn txs_json(count: usize) -> String {
        let rows: Vec<String> = (0..count as u64).map(one_tx).collect();
        format!("[{}]", rows.join(","))
    }

    fn open(txs_body: &str) -> (ScreenHost<Rec>, Rec) {
        let (mut host, rec) = host_with(
            Network::Mainnet,
            BTreeMap::from([
                (format!("/address/{ADDRESS}"), address_stats(ADDRESS)),
                (format!("/address/{ADDRESS}/txs"), txs_body.to_string()),
                (format!("/address/{ADDRESS}/utxo"), "[]".to_string()),
            ]),
        );
        host.open_address(ADDRESS)
            .unwrap_or_else(|err| panic!("address did not open: {err}"));
        let paths = rec.gets();
        assert!(
            paths
                .iter()
                .any(|path| path == &format!("/address/{ADDRESS}/txs")),
            "did not GET /address/:addr/txs: {paths:?}"
        );
        assert!(
            paths
                .iter()
                .all(|path| !path.contains('?') && !path.contains('#')),
            "{paths:?}"
        );
        (host, rec)
    }

    let (host, _) = open(&txs_json(3));
    let three = painted(&host);
    assert!(
        three.lines().any(|line| line == LINE),
        "{LINE} is absent: {three}"
    );
    assert_eq!(
        three.lines().filter(|line| *line == LINE).count(),
        1,
        "{LINE} was not painted once: {three}"
    );
    let lines: Vec<&str> = three.lines().collect();
    let heading_at = lines
        .iter()
        .position(|line| *line == LINE)
        .expect("heading");
    let first_fee = lines
        .iter()
        .position(|line| line.contains("fee 1"))
        .expect("the first raw fee was dropped");
    assert!(
        heading_at < first_fee,
        "{LINE} was not above the transaction rows: {three}"
    );
    assert!(
        three.contains("fee 3"),
        "the third raw fee was dropped: {three}"
    );

    let (host, _) = open(&txs_json(2));
    let two = painted(&host);
    assert!(
        !two.lines().any(|line| line == LINE),
        "two transactions painted {LINE}: {two}"
    );
    assert!(two.contains("fee 2"), "the two-row fee was dropped: {two}");

    let (host, _) = open("[]");
    let empty = painted(&host);
    assert!(
        !empty.lines().any(|line| line == LINE),
        "an empty transaction list painted {LINE}: {empty}"
    );

    let (mut host, rec) = host_with(
        Network::Mainnet,
        BTreeMap::from([
            (format!("/address/{ADDRESS}"), address_stats(ADDRESS)),
            (format!("/address/{ADDRESS}/txs"), "not-json".to_string()),
            (format!("/address/{ADDRESS}/utxo"), "[]".to_string()),
        ]),
    );
    let err = host
        .open_address(ADDRESS)
        .expect_err("an unparsed transaction body must fail");
    assert!(
        !err.to_string().contains(LINE),
        "the parse error invented {LINE}: {err}"
    );
    let paths = rec.gets();
    assert!(
        paths
            .iter()
            .any(|path| path == &format!("/address/{ADDRESS}/txs")),
        "did not GET /address/:addr/txs: {paths:?}"
    );
    assert!(
        paths
            .iter()
            .all(|path| !path.contains('?') && !path.contains('#')),
        "{paths:?}"
    );
    let unparsed = painted(&host);
    assert!(
        !unparsed.lines().any(|line| line == LINE),
        "an unparsed body painted {LINE}: {unparsed}"
    );
}

/// Spend input index from a no-query `GET /tx/:txid/outspends`.
///
/// The Leptos page paints `vin 1` when the spend includes `vin`. A spend
/// with no `vin` does not invent `vin 0`. `spent true` and `spend txid` stay.
#[test]
fn transaction_screen_paints_vin() {
    const LINE: &str = "vin 1";
    const TXID: &str = "vinindex";
    const SPEND: &str = "eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee";

    fn open_spends(spends: &str) -> String {
        let tx = format!(
            r#"{{"txid":"{TXID}","version":1,"locktime":0,"vin":[],"vout":[{{"scriptpubkey":"0014aa","scriptpubkey_asm":"OP_0","scriptpubkey_type":"v0_p2wpkh","scriptpubkey_address":"bc1qout","value":9}}],"size":10,"weight":40,"sigops":0,"fee":1,"status":{{"confirmed":true}}}}"#
        );
        let (mut host, rec) = host_with(
            Network::Mainnet,
            BTreeMap::from([
                (format!("/tx/{TXID}"), tx),
                (format!("/tx/{TXID}/outspends"), spends.to_string()),
            ]),
        );
        host.open_transaction(TXID).expect("tx");
        let paths = rec.gets();
        assert!(
            paths
                .iter()
                .any(|path| path == &format!("/tx/{TXID}/outspends")),
            "did not GET /tx/:txid/outspends: {paths:?}"
        );
        assert!(
            paths
                .iter()
                .all(|path| !path.contains('?') && !path.contains('#')),
            "{paths:?}"
        );
        painted(&host)
    }

    let spent =
        format!(r#"[{{"spent":true,"txid":"{SPEND}","vin":1,"status":{{"confirmed":true}}}}]"#);
    let text = open_spends(&spent);
    assert!(
        text.lines().any(|line| line == LINE),
        "{LINE} is absent: {text}"
    );
    assert_eq!(
        text.lines().filter(|line| *line == LINE).count(),
        1,
        "{LINE} was not painted once: {text}"
    );
    assert!(
        text.lines().any(|line| line == "spent true"),
        "spent true was dropped: {text}"
    );
    assert!(
        text.lines()
            .any(|line| line == format!("spend txid {SPEND}")),
        "the spending txid was dropped: {text}"
    );

    let unspent = open_spends(r#"[{"spent":false}]"#);
    assert!(
        unspent.lines().any(|line| line == "spent false"),
        "spent false is absent: {unspent}"
    );
    assert!(
        !unspent.lines().any(|line| line.starts_with("vin ")),
        "a spend with no vin invented a vin line: {unspent}"
    );
}

/// `P2WSH witness script` from a no-query `GET /tx/:txid`.
///
/// The Leptos page paints that exact line when `inner_witnessscript_asm` is
/// nonempty and the prevout is not `v1_p2tr`. A null prevout still paints it.
/// A `v1_p2tr` prevout keeps `P2TR tapscript` and omits this line. A missing
/// inner script omits it. An output whose type is `v0_p2wsh` does not paint
/// it. The witness script asm, script hex, script asm, scriptsig, and witness
/// stay off the line.
#[test]
fn transaction_screen_paints_p2wsh_witness_script() {
    const LINE: &str = "P2WSH witness script";
    const TXID: &str = "p2wshwitness";
    const INNER: &str = "P2WSHWITNESSASMDEAD";
    const PREV: &str = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";

    fn prevout(kind: &str) -> String {
        format!(
            r#"{{"scriptpubkey":"P2WSHPUBHEXDEAD","scriptpubkey_asm":"P2WSHPUBASMDEAD","scriptpubkey_type":"{kind}"}}"#
        )
    }

    fn open(prevout_json: &str, inner_field: &str, out_kind: &str) -> String {
        let body = format!(
            r#"{{"txid":"{TXID}","version":1,"locktime":0,"vin":[{{"txid":"{PREV}","vout":0,"prevout":{prevout_json},"scriptsig":"P2WSHSIGHEXDEAD","scriptsig_asm":"P2WSHSIGASMDEAD","witness":["P2WSHWITDEAD"],"is_coinbase":false,"sequence":1{inner_field}}}],"vout":[{{"scriptpubkey":"P2WSHOUTHEXDEAD","scriptpubkey_asm":"P2WSHOUTASMDEAD","scriptpubkey_type":"{out_kind}","value":1}}],"size":10,"weight":7,"sigops":0,"fee":1}}"#
        );
        let (mut host, rec) = host_with(
            Network::Mainnet,
            BTreeMap::from([(format!("/tx/{TXID}"), body)]),
        );
        host.open_transaction(TXID).expect("tx");
        let paths = rec.gets();
        assert!(
            paths.iter().any(|path| path == &format!("/tx/{TXID}")),
            "did not GET /tx/:txid: {paths:?}"
        );
        assert!(
            paths
                .iter()
                .all(|path| !path.contains('?') && !path.contains('#')),
            "{paths:?}"
        );
        painted(&host)
    }

    fn assert_secrets_stay_off(text: &str) {
        for needle in [
            INNER,
            "P2WSHSIGHEXDEAD",
            "P2WSHSIGASMDEAD",
            "P2WSHWITDEAD",
            "P2WSHPUBHEXDEAD",
            "P2WSHPUBASMDEAD",
            "P2WSHOUTHEXDEAD",
            "P2WSHOUTASMDEAD",
        ] {
            assert!(!text.contains(needle), "{needle} leaked: {text}");
        }
    }

    let present = open(
        &prevout("v0_p2wsh"),
        &format!(r#","inner_witnessscript_asm":"{INNER}""#),
        "p2pkh",
    );
    assert!(
        present.lines().any(|line| line == LINE),
        "{LINE} is absent: {present}"
    );
    assert_eq!(
        present.lines().filter(|line| *line == LINE).count(),
        1,
        "{LINE} was not painted once: {present}"
    );
    assert!(
        !present.lines().any(|line| line == "P2TR tapscript"),
        "a v0_p2wsh input painted P2TR tapscript: {present}"
    );
    assert_secrets_stay_off(&present);

    let null_prevout = open(
        "null",
        &format!(r#","inner_witnessscript_asm":"{INNER}""#),
        "p2pkh",
    );
    assert_eq!(
        null_prevout.lines().filter(|line| *line == LINE).count(),
        1,
        "a null prevout did not paint {LINE} once: {null_prevout}"
    );
    assert_secrets_stay_off(&null_prevout);

    let tapscript = open(
        &prevout("v1_p2tr"),
        &format!(r#","inner_witnessscript_asm":"{INNER}""#),
        "p2pkh",
    );
    assert!(
        tapscript.lines().any(|line| line == "P2TR tapscript"),
        "P2TR tapscript was dropped: {tapscript}"
    );
    assert!(
        !tapscript.lines().any(|line| line == LINE),
        "a v1_p2tr prevout painted {LINE}: {tapscript}"
    );
    assert_secrets_stay_off(&tapscript);

    let missing = open(&prevout("v0_p2wsh"), "", "p2pkh");
    assert!(
        !missing.lines().any(|line| line == LINE),
        "a missing inner script painted {LINE}: {missing}"
    );
    assert_secrets_stay_off(&missing);

    let output_only = open(&prevout("p2pkh"), "", "v0_p2wsh");
    assert!(
        !output_only.lines().any(|line| line == LINE),
        "a v0_p2wsh output painted {LINE}: {output_only}"
    );
    assert_secrets_stay_off(&output_only);
}

/// Fee rate label from no-query `GET /tx/:txid`.
///
/// The transaction row paints `Fee rate` as its own line. A longer line that
/// contains those words does not. Fee 584 and weight 572 are the served body.
#[test]
fn transaction_screen_paints_fee_rate_label() {
    const LINE: &str = "Fee rate";
    let text = paint_tx_fee_rate("feeratelabel", 584, r#""weight":572"#);
    assert!(
        text.lines().any(|line| line == LINE),
        "Fee rate is absent: {text}"
    );
    assert_eq!(
        text.lines().filter(|line| *line == LINE).count(),
        1,
        "Fee rate was not painted once: {text}"
    );
}

/// Locktime label from no-query `GET /tx/:txid`.
///
/// The transaction row paints `Locktime` as its own line. A longer line that
/// contains that word does not. Locktime 0 is the served body. The sentences
/// `Locktime 0` and `Locktime 500000000` stay.
#[test]
fn transaction_screen_paints_locktime_label() {
    const LINE: &str = "Locktime";
    let text = paint_tx_locktime("locktimelabel", 0);
    assert!(
        text.lines().any(|line| line == LINE),
        "Locktime is absent: {text}"
    );
    assert_eq!(
        text.lines().filter(|line| *line == LINE).count(),
        1,
        "Locktime was not painted once: {text}"
    );
}

/// Lowercase coinbase word from no-query `GET /tx/:txid`.
///
/// `is_coinbase` true paints `coinbase` as its own line. `Coinbase` and
/// `input coinbase` stay. A longer line that contains the word does not.
#[test]
fn transaction_screen_paints_coinbase_word() {
    const LINE: &str = "coinbase";
    let text = paint_tx_input("coinbaseword", r#","is_coinbase":true,"sequence":0"#);
    assert!(
        text.lines().any(|line| line == LINE),
        "coinbase is absent: {text}"
    );
    assert_eq!(
        text.lines().filter(|line| *line == LINE).count(),
        1,
        "coinbase was not painted once: {text}"
    );
}

/// Capitalized output total from no-query `GET /tx/:txid`.
///
/// Output values 100 and 50 paint `Output total 150` as its own line. The
/// lowercase `output total` sentence stays. A longer line that contains
/// those words does not.
#[test]
fn transaction_screen_paints_output_total_sentence() {
    const LINE: &str = "Output total 150";
    let text = paint_tx_outputs(
        "outtotalsentence",
        &format!(
            "{},{}",
            tx_output_value(Some(100), None),
            tx_output_value(Some(50), None)
        ),
    );
    assert!(
        text.lines().any(|line| line == LINE),
        "Output total 150 is absent: {text}"
    );
    assert_eq!(
        text.lines().filter(|line| *line == LINE).count(),
        1,
        "Output total 150 was not painted once: {text}"
    );
}

/// Output type label from no-query `GET /tx/:txid`.
///
/// A nonempty `vout.scriptpubkey_type` paints `Type` as its own line.
/// `Type V0_P2WPKH` stays. A longer line that contains the word does not.
#[test]
fn transaction_screen_paints_output_type_label() {
    const LINE: &str = "Type";
    let text = paint_tx_body(
        "otypelabel",
        &tx_with_parties("otypelabel", otype_input_null(), &otype_output("v0_p2wpkh")),
    );
    assert!(
        text.lines().any(|line| line == LINE),
        "Type is absent: {text}"
    );
    assert_eq!(
        text.lines().filter(|line| *line == LINE).count(),
        1,
        "Type was not painted once: {text}"
    );
}

/// Outspend confirmation from a no-query `GET /tx/:txid/outspends`.
///
/// `status.confirmed` true paints `confirmed true` as its own line. A longer
/// line that contains those words does not. A missing status does not paint
/// it. Block height, block hash, and block time stay off. `spent true` and
/// `vin 1` stay.
#[test]
fn transaction_screen_paints_outspend_confirmed() {
    const LINE: &str = "confirmed true";
    const TXID: &str = "outspendconf";
    const SPEND: &str = "eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee";
    const BLOCK: &str = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";

    fn open_spends(spends: &str) -> String {
        let tx = format!(
            r#"{{"txid":"{TXID}","version":1,"locktime":0,"vin":[],"vout":[{{"scriptpubkey":"0014aa","scriptpubkey_asm":"OP_0","scriptpubkey_type":"v0_p2wpkh","scriptpubkey_address":"bc1qout","value":9}}],"size":10,"weight":40,"sigops":0,"fee":1,"status":{{"confirmed":false}}}}"#
        );
        let (mut host, rec) = host_with(
            Network::Mainnet,
            BTreeMap::from([
                (format!("/tx/{TXID}"), tx),
                (format!("/tx/{TXID}/outspends"), spends.to_string()),
            ]),
        );
        host.open_transaction(TXID).expect("tx");
        let paths = rec.gets();
        assert!(
            paths
                .iter()
                .any(|path| path == &format!("/tx/{TXID}/outspends")),
            "did not GET /tx/:txid/outspends: {paths:?}"
        );
        assert!(
            paths
                .iter()
                .all(|path| !path.contains('?') && !path.contains('#')),
            "{paths:?}"
        );
        painted(&host)
    }

    let spent = format!(
        r#"[{{"spent":true,"txid":"{SPEND}","vin":1,"status":{{"confirmed":true,"block_height":842001,"block_hash":"{BLOCK}","block_time":1600000001}}}}]"#
    );
    let text = open_spends(&spent);
    assert!(
        text.lines().any(|line| line == LINE),
        "{LINE} is absent: {text}"
    );
    assert_eq!(
        text.lines().filter(|line| *line == LINE).count(),
        1,
        "{LINE} was not painted once: {text}"
    );
    assert!(!text.contains(BLOCK), "block hash leaked: {text}");
    assert!(!text.contains("842001"), "block height leaked: {text}");
    assert!(!text.contains("1600000001"), "block time leaked: {text}");
    assert!(
        text.lines().any(|line| line == "spent true"),
        "spent true was dropped: {text}"
    );
    assert!(
        text.lines().any(|line| line == "vin 1"),
        "vin 1 was dropped: {text}"
    );

    let bare = open_spends(r#"[{"spent":false}]"#);
    assert!(
        !bare.lines().any(|line| line == LINE),
        "a missing status painted {LINE}: {bare}"
    );
}

/// Poison warning sentence from a no-query `GET /tx/:txid`.
///
/// Similar output addresses paint the Angular sentence as its own line.
/// The shorter warning stays. One output does not paint the sentence.
/// A longer line that contains part of the sentence does not.
#[test]
fn transaction_screen_paints_poison_attack_warning() {
    const LINE: &str = "Warning! This transaction involves deceptively similar addresses. It may be an address poisoning attack.";
    const SHORT: &str = "Warning! This transaction involves deceptively similar addresses.";
    const TXID: &str = "poisonwarn";

    fn vout(address: &str, value: u64) -> String {
        format!(
            r#"{{"scriptpubkey":"0014aa","scriptpubkey_asm":"OP_0","scriptpubkey_type":"v0_p2wpkh","scriptpubkey_address":"{address}","value":{value}}}"#
        )
    }

    fn open(vout_json: &str) -> String {
        let vin = r#"{"txid":"bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb","vout":0,"prevout":{"scriptpubkey":"0014bb","scriptpubkey_asm":"OP_0","scriptpubkey_type":"v0_p2wpkh","scriptpubkey_address":"bc1qunrelated","value":8},"scriptsig":"","scriptsig_asm":"","is_coinbase":false,"sequence":4294967295}"#;
        let body = format!(
            r#"{{"txid":"{TXID}","version":1,"locktime":0,"vin":[{vin}],"vout":[{vout_json}],"size":10,"weight":40,"sigops":0,"fee":1,"status":{{"confirmed":false}}}}"#
        );
        let (mut host, rec) = host_with(
            Network::Mainnet,
            BTreeMap::from([(format!("/tx/{TXID}"), body)]),
        );
        host.open_transaction(TXID).expect("tx");
        let paths = rec.gets();
        assert!(
            paths.iter().any(|path| path == &format!("/tx/{TXID}")),
            "did not GET /tx/:txid: {paths:?}"
        );
        assert!(
            paths
                .iter()
                .all(|path| !path.contains('?') && !path.contains('#')),
            "{paths:?}"
        );
        painted(&host)
    }

    let pair = format!("{},{}", vout("bc1qlookaaa", 3), vout("bc1qlookXY", 4));
    let text = open(&pair);
    assert!(
        text.lines().any(|line| line == LINE),
        "{LINE} is absent: {text}"
    );
    assert_eq!(
        text.lines().filter(|line| *line == LINE).count(),
        1,
        "{LINE} was not painted once: {text}"
    );
    assert!(
        text.lines().any(|line| line == SHORT),
        "the shorter warning was dropped: {text}"
    );

    let alone = open(&vout("bc1qonlyone", 3));
    assert!(
        !alone.lines().any(|line| line == LINE),
        "one output painted {LINE}: {alone}"
    );
}

/// Combined balance from no-query `GET /address/:address`.
///
/// The four sums paint `Balance 4015` as its own line. The same words stay
/// on the longer address stats line. A missing sum does not paint that line.
#[test]
fn address_screen_paints_balance_line() {
    const LINE: &str = "Balance 4015";
    let text = paint_address(
        "bc1qexample",
        r#"{"address":"bc1qexample","chain_stats":{"tx_count":4,"funded_txo_count":4,"spent_txo_count":1,"funded_txo_sum":5000,"spent_txo_sum":1000},"mempool_stats":{"tx_count":1,"funded_txo_count":4,"spent_txo_count":1,"funded_txo_sum":20,"spent_txo_sum":5}}"#,
    );
    assert!(
        text.lines().any(|line| line == LINE),
        "{LINE} is absent: {text}"
    );
    assert_eq!(
        text.lines().filter(|line| *line == LINE).count(),
        1,
        "{LINE} was not painted once: {text}"
    );
    assert!(
        text.contains("Confirmed balance 4000"),
        "confirmed balance was dropped: {text}"
    );
    assert!(
        text.contains("Pending 15"),
        "pending amount was dropped: {text}"
    );

    let omitted = paint_address(
        "bc1qexampleomit",
        r#"{"address":"bc1qexampleomit","chain_stats":{"tx_count":4,"funded_txo_count":4,"spent_txo_count":1,"funded_txo_sum":5000,"spent_txo_sum":1000},"mempool_stats":{"tx_count":1,"funded_txo_count":4,"spent_txo_count":1,"funded_txo_sum":20}}"#,
    );
    assert!(
        !omitted.lines().any(|line| line == LINE),
        "a missing sum painted {LINE}: {omitted}"
    );
    assert!(
        !omitted.contains("Balance "),
        "a missing sum painted a balance: {omitted}"
    );
}

/// Confirmed balance from no-query `GET /scripthash/:hash`.
///
/// Chain funded minus chain spent paints `Confirmed balance 4000` as its own
/// line. The same words stay on the longer stats line. A missing chain sum
/// does not paint that line. Mempool sums are not this difference.
#[test]
fn scripthash_screen_paints_confirmed_balance_line() {
    const LINE: &str = "Confirmed balance 4000";
    const HASH: &str = "dddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddd";
    let text = paint_scripthash(
        HASH,
        &format!(
            r#"{{"scripthash":"{HASH}","chain_stats":{{"tx_count":4,"funded_txo_count":4,"spent_txo_count":1,"funded_txo_sum":5000,"spent_txo_sum":1000}},"mempool_stats":{{"tx_count":2,"funded_txo_count":9,"spent_txo_count":1,"funded_txo_sum":20,"spent_txo_sum":7}}}}"#
        ),
    );
    assert!(
        text.lines().any(|line| line == LINE),
        "{LINE} is absent: {text}"
    );
    assert_eq!(
        text.lines().filter(|line| *line == LINE).count(),
        1,
        "{LINE} was not painted once: {text}"
    );
    assert!(
        text.lines().any(|line| line.contains(LINE) && line != LINE),
        "the longer stats line dropped {LINE}: {text}"
    );

    let omitted = paint_scripthash(
        HASH,
        &format!(
            r#"{{"scripthash":"{HASH}","chain_stats":{{"tx_count":4,"funded_txo_count":4,"spent_txo_count":1,"funded_txo_sum":5000}},"mempool_stats":{{"tx_count":2,"funded_txo_count":9,"spent_txo_count":1,"funded_txo_sum":20,"spent_txo_sum":7}}}}"#
        ),
    );
    assert!(
        !omitted.lines().any(|line| line == LINE),
        "a missing sum painted {LINE}: {omitted}"
    );
    assert!(
        !omitted.contains(LINE),
        "a missing sum painted confirmed balance: {omitted}"
    );
}

/// Sigops label from no-query `GET /tx/:txid`.
///
/// Angular paints the label `Sigops` when sigops is present, including 0.
/// The transaction row paints `Sigops` as its own line. `Sigops 0` stays.
/// A longer line that contains the word does not. A missing key or JSON
/// null does not paint the label.
#[test]
fn transaction_screen_paints_sigops_label() {
    const LINE: &str = "Sigops";
    let text = paint_tx_adjusted("sigopslabel", r#""weight":7"#, r#""sigops":0"#);
    assert!(
        text.lines().any(|line| line == LINE),
        "{LINE} is absent: {text}"
    );
    assert_eq!(
        text.lines().filter(|line| *line == LINE).count(),
        1,
        "{LINE} was not painted once: {text}"
    );
    assert!(
        text.lines().any(|line| line == "Sigops 0"),
        "Sigops 0 was dropped: {text}"
    );
    assert!(
        text.lines().any(|line| line.contains(LINE) && line != LINE),
        "the longer sigops line dropped {LINE}: {text}"
    );

    let absent = paint_tx_adjusted("sigopslabelabs", r#""weight":7"#, "");
    assert!(
        !absent.lines().any(|line| line == LINE),
        "a missing sigops painted {LINE}: {absent}"
    );
    assert!(
        !absent.contains(LINE),
        "a missing sigops painted Sigops: {absent}"
    );

    let null = paint_tx_adjusted("sigopslabelnull", r#""weight":7"#, r#""sigops":null"#);
    assert!(
        !null.lines().any(|line| line == LINE),
        "null sigops painted {LINE}: {null}"
    );
    assert!(!null.contains(LINE), "null sigops painted Sigops: {null}");
}

/// Capitalized virtual size from no-query `GET /tx/:txid`.
///
/// Weight 4 paints `Virtual size 1` as its own line. The lowercase
/// `virtual size 1` stays. Weight 7 does not round up to 2. Weight 0
/// paints `Virtual size 0`. A missing weight or JSON null does not paint
/// the capitalized line.
#[test]
fn transaction_screen_paints_virtual_size_sentence() {
    const LINE: &str = "Virtual size 1";
    let text = paint_tx_weight("vsizeword", r#""weight":4"#);
    assert!(
        text.lines().any(|line| line == LINE),
        "{LINE} is absent: {text}"
    );
    assert_eq!(
        text.lines().filter(|line| *line == LINE).count(),
        1,
        "{LINE} was not painted once: {text}"
    );
    assert!(
        text.lines().any(|line| line == "virtual size 1"),
        "virtual size 1 was dropped: {text}"
    );
    assert!(
        !text.lines().any(|line| line == "Virtual size 2"),
        "weight 4 rounded up: {text}"
    );

    let seven = paint_tx_weight("vsizeword7", r#""weight":7"#);
    assert!(
        seven.lines().any(|line| line == LINE),
        "weight 7 did not paint {LINE}: {seven}"
    );
    assert!(
        !seven.lines().any(|line| line == "Virtual size 2"),
        "weight 7 rounded up: {seven}"
    );
    assert!(
        seven.lines().any(|line| line == "virtual size 1"),
        "weight 7 dropped virtual size 1: {seven}"
    );

    let zero = paint_tx_weight("vsizeword0", r#""weight":0"#);
    assert!(
        zero.lines().any(|line| line == "Virtual size 0"),
        "weight 0 omitted Virtual size 0: {zero}"
    );
    assert!(
        zero.lines().any(|line| line == "virtual size 0"),
        "weight 0 dropped virtual size 0: {zero}"
    );
    assert!(
        !zero.lines().any(|line| line == LINE),
        "weight 0 painted {LINE}: {zero}"
    );

    let absent = paint_tx_weight("vsizewordabs", "");
    assert!(
        !absent.lines().any(|line| line == LINE),
        "a missing weight painted {LINE}: {absent}"
    );
    assert!(
        !absent.contains("Virtual size"),
        "a missing weight painted Virtual size: {absent}"
    );

    let null = paint_tx_weight("vsizewordnull", r#""weight":null"#);
    assert!(
        !null.lines().any(|line| line == LINE),
        "null weight painted {LINE}: {null}"
    );
    assert!(
        !null.contains("Virtual size"),
        "null weight painted Virtual size: {null}"
    );
}

/// Chain funded sum from no-query `GET /address/:script`.
///
/// `funded_txo_sum` 5000 paints `Total received 5000` as its own line.
/// The same words stay on the address stats line. A missing sum does not
/// paint the line. The confirmed balance and the mempool funded sum are
/// not this line.
#[test]
fn address_screen_paints_total_received_5000() {
    const LINE: &str = "Total received 5000";
    let text = paint_address(
        "bc1qexample",
        r#"{"address":"bc1qexample","chain_stats":{"tx_count":4,"funded_txo_count":4,"spent_txo_count":1,"funded_txo_sum":5000,"spent_txo_sum":1000},"mempool_stats":{"tx_count":1,"funded_txo_count":1,"spent_txo_count":0,"funded_txo_sum":20,"spent_txo_sum":0}}"#,
    );
    assert!(
        text.lines().any(|line| line == LINE),
        "{LINE} is absent: {text}"
    );
    assert_eq!(
        text.lines().filter(|line| *line == LINE).count(),
        1,
        "{LINE} was not painted once: {text}"
    );
    assert!(
        text.lines().any(|line| line != LINE && line.contains(LINE)),
        "the stats line dropped {LINE}: {text}"
    );
    assert!(
        !text.lines().any(|line| line == "Total received 4000"),
        "confirmed balance was painted as total received: {text}"
    );
    assert!(
        !text.lines().any(|line| line == "Total received 20"),
        "mempool funded sum was painted as total received: {text}"
    );

    let omitted = paint_address(
        "bc1qnorecv",
        r#"{"address":"bc1qnorecv","chain_stats":{"tx_count":4,"funded_txo_count":4,"spent_txo_count":1,"spent_txo_sum":1000},"mempool_stats":{"tx_count":1,"funded_txo_count":1,"spent_txo_count":0,"funded_txo_sum":20,"spent_txo_sum":0}}"#,
    );
    assert!(
        !omitted.lines().any(|line| line == LINE),
        "a missing funded sum painted {LINE}: {omitted}"
    );
    assert!(
        !omitted.contains("Total received"),
        "a missing funded sum painted Total received: {omitted}"
    );
}

/// Chain spent sum plus mempool spent sum from no-query `GET /address/:script`.
///
/// `spent_txo_sum` 1000 plus mempool `spent_txo_sum` 20 paints `Total sent 1020`
/// as its own line. The same words stay on the address stats line. A missing
/// sum does not paint the line. The chain spent sum alone is not this line.
#[test]
fn address_screen_paints_total_sent_1020() {
    const LINE: &str = "Total sent 1020";
    let text = paint_address(
        "bc1qsent1020",
        r#"{"address":"bc1qsent1020","chain_stats":{"tx_count":4,"funded_txo_count":4,"spent_txo_count":1,"funded_txo_sum":5000,"spent_txo_sum":1000},"mempool_stats":{"tx_count":1,"funded_txo_count":1,"spent_txo_count":0,"funded_txo_sum":20,"spent_txo_sum":20}}"#,
    );
    assert!(
        text.lines().any(|line| line == LINE),
        "{LINE} is absent: {text}"
    );
    assert_eq!(
        text.lines().filter(|line| *line == LINE).count(),
        1,
        "{LINE} was not painted once: {text}"
    );
    assert!(
        text.lines().any(|line| line != LINE && line.contains(LINE)),
        "the stats line dropped {LINE}: {text}"
    );
    assert!(
        !text.lines().any(|line| line == "Total sent 1000"),
        "total sent ignored the mempool spent sum: {text}"
    );
    assert!(
        !text.lines().any(|line| line == "Total sent 20"),
        "the mempool spent sum was painted alone: {text}"
    );

    let omitted = paint_address(
        "bc1qnosent1020",
        r#"{"address":"bc1qnosent1020","chain_stats":{"tx_count":4,"funded_txo_count":4,"spent_txo_count":1,"funded_txo_sum":5000,"spent_txo_sum":1000},"mempool_stats":{"tx_count":1,"funded_txo_count":1,"spent_txo_count":0,"funded_txo_sum":20}}"#,
    );
    assert!(
        !omitted.lines().any(|line| line == LINE),
        "a missing mempool spent sum painted {LINE}: {omitted}"
    );
    assert!(
        !omitted.contains("Total sent"),
        "a missing mempool spent sum painted Total sent: {omitted}"
    );
}

/// Chain tx count plus mempool tx count from no-query `GET /address/:script`.
///
/// `tx_count` 4 plus mempool `tx_count` 1 paints `Transactions 5` as its own
/// line. The same words stay on the address stats line. A missing count does
/// not paint the line. The chain count alone is not this line.
#[test]
fn address_screen_paints_transactions_5() {
    const LINE: &str = "Transactions 5";
    let text = paint_address(
        "bc1qtx5",
        r#"{"address":"bc1qtx5","chain_stats":{"tx_count":4,"funded_txo_count":4,"spent_txo_count":1,"funded_txo_sum":5000,"spent_txo_sum":1000},"mempool_stats":{"tx_count":1,"funded_txo_count":2,"spent_txo_count":1,"funded_txo_sum":20,"spent_txo_sum":7}}"#,
    );
    assert!(
        text.lines().any(|line| line == LINE),
        "{LINE} is absent: {text}"
    );
    assert_eq!(
        text.lines().filter(|line| *line == LINE).count(),
        1,
        "{LINE} was not painted once: {text}"
    );
    assert!(
        text.lines().any(|line| line != LINE && line.contains(LINE)),
        "the stats line dropped {LINE}: {text}"
    );
    assert!(
        !text.lines().any(|line| line == "Transactions 4"),
        "the chain tx count was painted alone: {text}"
    );
    assert!(
        !text.lines().any(|line| line == "Transactions 1"),
        "the mempool tx count was painted alone: {text}"
    );

    let omitted = paint_address(
        "bc1qnotx5",
        r#"{"address":"bc1qnotx5","chain_stats":{"tx_count":4,"funded_txo_count":4,"spent_txo_count":1,"funded_txo_sum":5000,"spent_txo_sum":1000},"mempool_stats":{"funded_txo_count":2,"spent_txo_count":1,"funded_txo_sum":20,"spent_txo_sum":7}}"#,
    );
    assert!(
        !omitted.lines().any(|line| line == LINE),
        "a missing mempool tx count painted {LINE}: {omitted}"
    );
    assert!(
        !omitted.contains("Transactions"),
        "a missing mempool tx count painted Transactions: {omitted}"
    );
}

/// Chain funded count minus chain spent count from no-query `GET /address/:script`.
///
/// `funded_txo_count` 4 minus `spent_txo_count` 1 paints `Unspent TXOs 3` as
/// its own line. Mempool counts are not this line. A missing count does not
/// paint the line. Confirmed UTXOs is not this line.
#[test]
fn address_screen_paints_unspent_txos_3() {
    const LINE: &str = "Unspent TXOs 3";
    let text = paint_address(
        "bc1qunspent3",
        r#"{"address":"bc1qunspent3","chain_stats":{"tx_count":4,"funded_txo_count":4,"spent_txo_count":1,"funded_txo_sum":5000,"spent_txo_sum":1000},"mempool_stats":{"tx_count":1,"funded_txo_count":9,"spent_txo_count":1,"funded_txo_sum":20,"spent_txo_sum":0}}"#,
    );
    assert!(
        text.lines().any(|line| line == LINE),
        "{LINE} is absent: {text}"
    );
    assert_eq!(
        text.lines().filter(|line| *line == LINE).count(),
        1,
        "{LINE} was not painted once: {text}"
    );
    assert!(
        !text.lines().any(|line| line == "Unspent TXOs 8"),
        "the mempool count difference was painted: {text}"
    );
    assert!(
        !text.lines().any(|line| line == "Unspent TXOs 4"),
        "the chain funded count was painted without subtracting chain spent: {text}"
    );
    assert!(
        !text.lines().any(|line| line == "Unspent TXOs 11"),
        "mempool counts were folded into Unspent TXOs: {text}"
    );
    assert!(
        !text.lines().any(|line| line == "Unspent TXOs 5"),
        "mempool funded count was folded into Unspent TXOs: {text}"
    );

    let omitted = paint_address(
        "bc1qnounspent3",
        r#"{"address":"bc1qnounspent3","chain_stats":{"tx_count":4,"funded_txo_count":4,"funded_txo_sum":5000,"spent_txo_sum":1000},"mempool_stats":{"tx_count":1,"funded_txo_count":9,"spent_txo_count":1,"funded_txo_sum":20,"spent_txo_sum":0}}"#,
    );
    assert!(
        !omitted.lines().any(|line| line == LINE),
        "a missing chain spent count painted {LINE}: {omitted}"
    );
    assert!(
        !omitted.contains("Unspent TXOs"),
        "a missing chain spent count painted Unspent TXOs: {omitted}"
    );
}

/// Chain funded count minus chain spent count from no-query `GET /address/:script`.
///
/// `funded_txo_count` 4 minus `spent_txo_count` 1 paints `Confirmed UTXOs 3`
/// as its own line. The same words stay on the address stats line. Mempool
/// counts are not this line. A missing count does not paint the line.
/// Unspent TXOs is not this line.
#[test]
fn address_screen_paints_confirmed_utxos_3() {
    const LINE: &str = "Confirmed UTXOs 3";
    let text = paint_address(
        "bc1qconfirmedutxos3",
        r#"{"address":"bc1qconfirmedutxos3","chain_stats":{"tx_count":4,"funded_txo_count":4,"spent_txo_count":1,"funded_txo_sum":5000,"spent_txo_sum":1000},"mempool_stats":{"tx_count":1,"funded_txo_count":9,"spent_txo_count":1,"funded_txo_sum":20,"spent_txo_sum":0}}"#,
    );
    assert!(
        text.lines().any(|line| line == LINE),
        "{LINE} is absent: {text}"
    );
    assert_eq!(
        text.lines().filter(|line| *line == LINE).count(),
        1,
        "{LINE} was not painted once: {text}"
    );
    assert!(
        text.lines().any(|line| line != LINE && line.contains(LINE)),
        "the stats line dropped {LINE}: {text}"
    );
    assert!(
        !text.lines().any(|line| line == "Confirmed UTXOs 8"),
        "the mempool count difference was painted: {text}"
    );
    assert!(
        !text.lines().any(|line| line == "Confirmed UTXOs 4"),
        "the chain funded count was painted without subtracting chain spent: {text}"
    );
    assert!(
        !text.lines().any(|line| line == "Confirmed UTXOs 11"),
        "mempool counts were folded into Confirmed UTXOs: {text}"
    );
    assert!(
        !text.lines().any(|line| line == "Confirmed UTXOs 5"),
        "mempool funded count was folded into Confirmed UTXOs: {text}"
    );

    let omitted = paint_address(
        "bc1qnocount3",
        r#"{"address":"bc1qnocount3","chain_stats":{"tx_count":4,"funded_txo_count":4,"funded_txo_sum":5000,"spent_txo_sum":1000},"mempool_stats":{"tx_count":1,"funded_txo_count":9,"spent_txo_count":1,"funded_txo_sum":20,"spent_txo_sum":0}}"#,
    );
    assert!(
        !omitted.lines().any(|line| line == LINE),
        "a missing chain spent count painted {LINE}: {omitted}"
    );
    assert!(
        !omitted.contains("Confirmed UTXOs"),
        "a missing chain spent count painted Confirmed UTXOs: {omitted}"
    );
}

/// Mempool funded count minus mempool spent count from no-query `GET /address/:script`.
///
/// `funded_txo_count` 5 minus `spent_txo_count` 2 paints `Pending UTXOs 3`
/// as its own line. The same words stay on the address stats line. Chain
/// counts are not this line. A missing count does not paint the line.
/// Confirmed UTXOs is not this line.
#[test]
fn address_screen_paints_pending_utxos_3() {
    const LINE: &str = "Pending UTXOs 3";
    let text = paint_address(
        "bc1qpendingutxos3",
        r#"{"address":"bc1qpendingutxos3","chain_stats":{"tx_count":4,"funded_txo_count":4,"spent_txo_count":1,"funded_txo_sum":5000,"spent_txo_sum":1000},"mempool_stats":{"tx_count":1,"funded_txo_count":5,"spent_txo_count":2,"funded_txo_sum":20,"spent_txo_sum":250}}"#,
    );
    assert!(
        text.lines().any(|line| line == LINE),
        "{LINE} is absent: {text}"
    );
    assert_eq!(
        text.lines().filter(|line| *line == LINE).count(),
        1,
        "{LINE} was not painted once: {text}"
    );
    assert!(
        text.lines().any(|line| line != LINE && line.contains(LINE)),
        "the stats line dropped {LINE}: {text}"
    );
    assert!(
        !text.lines().any(|line| line == "Pending UTXOs +3"),
        "a plus was painted on {LINE}: {text}"
    );
    assert!(
        !text.lines().any(|line| line == "Pending UTXOs 4"),
        "the chain funded count was painted as Pending UTXOs: {text}"
    );
    assert!(
        !text.lines().any(|line| line == "Pending UTXOs 7"),
        "chain counts were folded into Pending UTXOs: {text}"
    );

    let omitted = paint_address(
        "bc1qnopending3",
        r#"{"address":"bc1qnopending3","chain_stats":{"tx_count":4,"funded_txo_count":4,"spent_txo_count":1,"funded_txo_sum":5000,"spent_txo_sum":1000},"mempool_stats":{"tx_count":1,"spent_txo_count":2,"funded_txo_sum":20,"spent_txo_sum":250}}"#,
    );
    assert!(
        !omitted.lines().any(|line| line == LINE),
        "a missing mempool funded count painted {LINE}: {omitted}"
    );
    assert!(
        !omitted.contains("Pending UTXOs"),
        "a missing mempool funded count painted Pending UTXOs: {omitted}"
    );
}

/// Mempool funded sum minus mempool spent sum from no-query `GET /address/:script`.
///
/// `funded_txo_sum` 20 minus `spent_txo_sum` 5 paints `Pending 15` as its own
/// line. The same words stay on the address stats line. Chain sums are not
/// this line. A missing sum does not paint the line. Pending UTXOs is not
/// this line. A plus is not this line.
#[test]
fn address_screen_paints_pending_15() {
    const LINE: &str = "Pending 15";
    let text = paint_address(
        "bc1qpending15",
        r#"{"address":"bc1qpending15","chain_stats":{"tx_count":4,"funded_txo_count":4,"spent_txo_count":1,"funded_txo_sum":5000,"spent_txo_sum":1000},"mempool_stats":{"tx_count":1,"funded_txo_count":4,"spent_txo_count":1,"funded_txo_sum":20,"spent_txo_sum":5}}"#,
    );
    assert!(
        text.lines().any(|line| line == LINE),
        "{LINE} is absent: {text}"
    );
    assert_eq!(
        text.lines().filter(|line| *line == LINE).count(),
        1,
        "{LINE} was not painted once: {text}"
    );
    assert!(
        text.lines().any(|line| line != LINE && line.contains(LINE)),
        "the stats line dropped {LINE}: {text}"
    );
    assert!(
        !text.lines().any(|line| line == "Pending +15"),
        "a plus was painted on {LINE}: {text}"
    );
    assert!(
        !text.lines().any(|line| line == "Pending 20"),
        "the mempool funded sum was painted alone: {text}"
    );
    assert!(
        !text.lines().any(|line| line == "Pending 5"),
        "the mempool spent sum was painted alone: {text}"
    );
    assert!(
        !text.lines().any(|line| line == "Pending -15"),
        "the mempool sums were swapped: {text}"
    );

    let omitted = paint_address(
        "bc1qnopending15",
        r#"{"address":"bc1qnopending15","chain_stats":{"tx_count":4,"funded_txo_count":4,"spent_txo_count":1,"funded_txo_sum":5000,"spent_txo_sum":1000},"mempool_stats":{"tx_count":1,"funded_txo_count":4,"spent_txo_count":1,"funded_txo_sum":20}}"#,
    );
    assert!(
        !omitted.lines().any(|line| line == LINE),
        "a missing mempool spent sum painted {LINE}: {omitted}"
    );
    assert!(
        !omitted.contains("Pending 15"),
        "a missing mempool spent sum painted Pending 15: {omitted}"
    );
    assert!(
        !omitted.lines().any(|line| line == "Pending 0"),
        "a missing mempool spent sum painted Pending 0: {omitted}"
    );
}
