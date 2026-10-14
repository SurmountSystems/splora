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
            paths
                .iter()
                .all(|path| !path.contains("/block-height/")),
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
            paths
                .iter()
                .all(|path| !path.contains("/block-height/")),
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

    let zero = paint_block("blk0tx0", block_json("blk0tx0", Some(0), Some(0)))
        .unwrap_or_else(|(err, painted)| panic!("height 0 tx_count 0 did not open: {err}\n{painted}"));
    assert_genesis_once("height 0 tx_count 0", &zero);

    let one = paint_block("blk0tx1", block_json("blk0tx1", Some(0), Some(1)))
        .unwrap_or_else(|(err, painted)| panic!("height 0 tx_count 1 did not open: {err}\n{painted}"));
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
