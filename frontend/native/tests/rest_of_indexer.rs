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
