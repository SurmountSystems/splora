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
        vec!["nonce 0 0x00000000"],
        "0x00000000 was not only the nonce line: {zero}"
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
