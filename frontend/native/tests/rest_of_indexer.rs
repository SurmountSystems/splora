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
    assert!(!text.contains("OP_RETURN"), "{text}");
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
    assert!(!sum.contains("OP_RETURN"), "{sum}");
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
    assert!(!five.contains("OP_RETURN"), "{five}");
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

fn assert_script_hex_stays_off(painted: &str) {
    assert!(!painted.contains("0014aa"), "{painted}");
    assert!(!painted.contains("deadbeef"), "{painted}");
    assert!(!painted.contains("OP_RETURN"), "{painted}");
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
