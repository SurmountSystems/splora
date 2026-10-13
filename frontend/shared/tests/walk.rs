//! Path strings and parsers for the indexer walk both apps call.
//!
//! Shapes follow the handlers in `src/rest.rs`. Liquid asset routes are path
//! strings only.

use splora_frontend_shared::wire::{BlockStatus, MerkleProof, Outspend, PackageResult, TxSummary};
use splora_frontend_shared::{
    IndexerCall, address_prefix_path, address_txs_chain_page_path, address_txs_chain_path,
    address_txs_mempool_path, address_txs_summary_page_path, address_txs_summary_path, asset_path,
    asset_registry_path, asset_txs_path, block_status_path, indexer_request,
    mempool_txids_page_path, mempool_txids_path, package_body, package_path, parse_address_prefix,
    parse_address_txs_chain, parse_address_txs_mempool, parse_address_txs_summary,
    parse_block_status, parse_mempool_txids, parse_merkle_proof, parse_outspend, parse_outspends,
    parse_package, parse_scripthash, parse_scripthash_txs, parse_scripthash_txs_chain,
    parse_scripthash_txs_mempool, parse_scripthash_utxos, parse_tx_hex, path_list, scripthash_path,
    scripthash_txs_chain_page_path, scripthash_txs_chain_path, scripthash_txs_mempool_path,
    scripthash_txs_path, scripthash_utxo_path, tx_hex_path, tx_merkle_proof_path, tx_outspend_path,
    tx_outspends_path,
};

fn fixture(name: &str) -> String {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(name);
    std::fs::read_to_string(path).unwrap()
}

#[test]
fn walk_paths_match_the_indexer_suffixes() {
    assert_eq!(block_status_path("abc"), "/block/abc/status");
    assert_eq!(tx_outspends_path("abc"), "/tx/abc/outspends");
    assert_eq!(tx_outspend_path("abc", 2), "/tx/abc/outspend/2");
    assert_eq!(tx_merkle_proof_path("abc"), "/tx/abc/merkle-proof");
    assert_eq!(tx_hex_path("abc"), "/tx/abc/hex");
    assert_eq!(address_txs_chain_path("xyz"), "/address/xyz/txs/chain");
    assert_eq!(
        address_txs_chain_page_path("xyz", "txid-9"),
        "/address/xyz/txs/chain/txid-9"
    );
    assert_eq!(address_txs_mempool_path("xyz"), "/address/xyz/txs/mempool");
    assert_eq!(address_txs_summary_path("xyz"), "/address/xyz/txs/summary");
    assert_eq!(
        address_txs_summary_page_path("xyz", "txid-9"),
        "/address/xyz/txs/summary/txid-9"
    );
    assert_eq!(mempool_txids_path(), "/mempool/txids");
    assert_eq!(
        mempool_txids_page_path("txid-9"),
        "/mempool/txids/page/txid-9"
    );
    assert_eq!(address_prefix_path("bc1q"), "/address-prefix/bc1q");
    assert_eq!(scripthash_path("aa"), "/scripthash/aa");
    assert_eq!(scripthash_txs_path("aa"), "/scripthash/aa/txs");
    assert_eq!(scripthash_txs_chain_path("aa"), "/scripthash/aa/txs/chain");
    assert_eq!(
        scripthash_txs_chain_page_path("aa", "txid-9"),
        "/scripthash/aa/txs/chain/txid-9"
    );
    assert_eq!(
        scripthash_txs_mempool_path("aa"),
        "/scripthash/aa/txs/mempool"
    );
    assert_eq!(scripthash_utxo_path("aa"), "/scripthash/aa/utxo");
    assert_eq!(package_path(), "/txs/package");
    assert_eq!(package_body(&["aa", "bb"]), "[\"aa\",\"bb\"]");
    assert_eq!(asset_path("liq"), "/asset/liq");
    assert_eq!(asset_txs_path("liq"), "/asset/liq/txs");
    assert_eq!(asset_registry_path("liq"), "/assets/registry/liq");
}

#[test]
fn path_list_includes_the_walk_the_indexer_already_serves() {
    let routes = path_list();
    let expected = [
        ("GET", "/block/:hash/status"),
        ("GET", "/tx/:txid/outspends"),
        ("GET", "/tx/:txid/outspend/:index"),
        ("GET", "/tx/:txid/merkle-proof"),
        ("GET", "/tx/:txid/hex"),
        ("GET", "/address/:script/txs/chain"),
        ("GET", "/address/:script/txs/chain/:last_seen_txid"),
        ("GET", "/address/:script/txs/mempool"),
        ("GET", "/address/:script/txs/summary"),
        ("GET", "/address/:script/txs/summary/:last_seen_txid"),
        ("GET", "/mempool/txids"),
        ("GET", "/mempool/txids/page/:last_seen_txid"),
        ("GET", "/address-prefix/:prefix"),
        ("GET", "/scripthash/:hash"),
        ("GET", "/scripthash/:hash/txs"),
        ("GET", "/scripthash/:hash/txs/chain"),
        ("GET", "/scripthash/:hash/txs/chain/:last_seen_txid"),
        ("GET", "/scripthash/:hash/txs/mempool"),
        ("GET", "/scripthash/:hash/utxo"),
        ("POST", "/txs/package"),
    ];
    for (method, pattern) in expected {
        assert!(
            routes
                .iter()
                .any(|route| route.method == method && route.pattern == pattern),
            "missing {method} {pattern}"
        );
    }
    assert!(
        routes.iter().all(|route| {
            route.pattern != "/asset/:asset"
                && route.pattern != "/asset/:asset/txs"
                && route.pattern != "/assets/registry/:asset"
        }),
        "liquid asset routes stay path strings"
    );
}

#[test]
fn indexer_request_builds_status_package_and_asset_calls() {
    let status = indexer_request(&IndexerCall::BlockStatus {
        hash: "abc".to_string(),
    });
    assert_eq!(status.method, "GET");
    assert_eq!(status.path, block_status_path("abc"));
    assert_eq!(status.body, None);

    let package = indexer_request(&IndexerCall::Package {
        hexes: vec!["aa".to_string(), "bb".to_string()],
    });
    assert_eq!(package.method, "POST");
    assert_eq!(package.path, package_path());
    assert_eq!(package.body.as_deref(), Some("[\"aa\",\"bb\"]"));

    let asset = indexer_request(&IndexerCall::Asset {
        asset: "liq".to_string(),
    });
    assert_eq!(asset.method, "GET");
    assert_eq!(asset.path, asset_path("liq"));
    assert_eq!(asset.body, None);
}

#[test]
fn parses_block_status_in_the_best_chain_and_orphaned() {
    let status =
        parse_block_status(r#"{"in_best_chain":true,"height":100,"next_best":"next-hash"}"#)
            .unwrap();
    assert_eq!(
        status,
        BlockStatus {
            in_best_chain: true,
            height: Some(100),
            next_best: Some("next-hash".to_string()),
        }
    );

    let orphan =
        parse_block_status(r#"{"in_best_chain":false,"height":null,"next_best":null}"#).unwrap();
    assert!(!orphan.in_best_chain);
    assert_eq!(orphan.height, None);
    assert_eq!(orphan.next_best, None);
}

#[test]
fn parses_outspends_spent_and_unspent() {
    let rows = parse_outspends(
        r#"[{"spent":false},{"spent":true,"txid":"spend-tx","vin":1,"status":{"confirmed":false}},{"spent":true,"txid":"spend-conf","vin":0,"status":{"confirmed":true,"block_height":100,"block_hash":"block-id-1","block_time":1600000000}}]"#,
    )
    .unwrap();
    assert_eq!(rows.len(), 3);
    assert_eq!(
        rows[0],
        Outspend {
            spent: false,
            txid: None,
            vin: None,
            status: None,
        }
    );
    assert!(rows[1].spent);
    assert_eq!(rows[1].txid.as_deref(), Some("spend-tx"));
    assert_eq!(rows[1].vin, Some(1));
    let unconfirmed = rows[1].status.as_ref().unwrap();
    assert!(!unconfirmed.confirmed);
    assert_eq!(unconfirmed.block_height, None);

    let one = parse_outspend(
        r#"{"spent":true,"txid":"spend-conf","vin":0,"status":{"confirmed":true,"block_height":100,"block_hash":"block-id-1","block_time":1600000000}}"#,
    )
    .unwrap();
    assert_eq!(one.txid.as_deref(), Some("spend-conf"));
    assert_eq!(one.vin, Some(0));
    let confirmed = one.status.unwrap();
    assert!(confirmed.confirmed);
    assert_eq!(confirmed.block_height, Some(100));
    assert_eq!(confirmed.block_hash.as_deref(), Some("block-id-1"));
    assert_eq!(confirmed.block_time, Some(1_600_000_000));
}

#[test]
fn parses_merkle_proof() {
    let proof = parse_merkle_proof(r#"{"block_height":100,"merkle":["aa","bb"],"pos":3}"#).unwrap();
    assert_eq!(
        proof,
        MerkleProof {
            block_height: 100,
            merkle: vec!["aa".to_string(), "bb".to_string()],
            pos: 3,
        }
    );
}

#[test]
fn parses_tx_hex_as_plain_text() {
    assert_eq!(parse_tx_hex("  deadbeef\n").unwrap(), "deadbeef");
    let err = parse_tx_hex(" \n").unwrap_err();
    assert!(err.detail.contains("empty"), "detail was {}", err.detail);
}

#[test]
fn parses_address_chain_mempool_and_summary() {
    let chain = parse_address_txs_chain(&fixture("address_txs.json")).unwrap();
    assert_eq!(chain[0].txid, "addr-tx-1");
    assert_eq!(chain[0].confirmed, "false");

    let mempool = parse_address_txs_mempool(&fixture("address_txs.json")).unwrap();
    assert_eq!(mempool[0].txid, "addr-tx-1");

    let rows = parse_address_txs_summary(
        r#"[{"txid":"sum-1","height":100,"value":5000,"time":1600000000,"tx_position":2},{"txid":"sum-2","height":99,"value":-100,"time":1599999000,"tx_position":0}]"#,
    )
    .unwrap();
    assert_eq!(
        rows[0],
        TxSummary {
            txid: "sum-1".to_string(),
            height: 100,
            value: 5000,
            time: 1_600_000_000,
            tx_position: 2,
        }
    );
    assert_eq!(rows[1].value, -100);
    assert_eq!(rows[1].tx_position, 0);
}

#[test]
fn parses_mempool_txids_and_address_prefix() {
    let ids = parse_mempool_txids(r#"["aa","bb"]"#).unwrap();
    assert_eq!(ids, vec!["aa".to_string(), "bb".to_string()]);
    let matches = parse_address_prefix(r#"["bc1qabc","bc1qabd"]"#).unwrap();
    assert_eq!(matches, vec!["bc1qabc".to_string(), "bc1qabd".to_string()]);
}

#[test]
fn parses_scripthash_twins_of_the_address_routes() {
    let stats = parse_scripthash(
        r#"{"scripthash":"aa","chain_stats":{"tx_count":4,"funded_txo_sum":9},"mempool_stats":{"tx_count":1}}"#,
    )
    .unwrap();
    assert_eq!(stats.scripthash, "aa");
    assert_eq!(stats.chain_tx_count, "4");
    assert_eq!(stats.funded_sum.as_deref(), Some("9"));
    assert_eq!(stats.mempool_tx_count, "1");

    let missing = parse_scripthash(
        r#"{"scripthash":"aa","chain_stats":{"tx_count":4},"mempool_stats":{"tx_count":0}}"#,
    )
    .expect("a missing funded sum is not a parse error");
    assert_eq!(missing.scripthash, "aa");
    assert_eq!(missing.chain_tx_count, "4");
    assert_eq!(missing.funded_sum, None);
    assert_ne!(missing.funded_sum.as_deref(), Some("0"));
    assert_eq!(missing.mempool_tx_count, "0");

    let txs = parse_scripthash_txs(&fixture("address_txs.json")).unwrap();
    assert_eq!(txs[0].txid, "addr-tx-1");
    let chain = parse_scripthash_txs_chain(&fixture("address_txs.json")).unwrap();
    assert_eq!(chain[0].fee, "111");
    let mempool = parse_scripthash_txs_mempool(&fixture("address_txs.json")).unwrap();
    assert_eq!(mempool[0].weight, "360");
    let utxos = parse_scripthash_utxos(&fixture("address_utxos.json")).unwrap();
    assert_eq!(utxos[0].txid, "utxo-tx-1");
    assert_eq!(utxos[0].confirmed, "true");
}

#[test]
fn parses_submit_package_result() {
    let body = r#"{
        "package_msg":"success",
        "tx-results":{
            "wtxid-1":{
                "txid":"txid-1",
                "vsize":141,
                "fees":{"base":1.5,"effective-feerate":2.0,"effective-includes":["txid-1"]}
            }
        },
        "replaced-transactions":["old-tx"]
    }"#;
    let parsed = parse_package(body).unwrap();
    assert_eq!(parsed.package_msg, "success");
    assert_eq!(parsed.replaced_transactions, vec!["old-tx".to_string()]);
    let tx = parsed.tx_results.get("wtxid-1").unwrap();
    assert_eq!(tx.txid, "txid-1");
    assert_eq!(tx.vsize, Some(141));
    assert_eq!(tx.error, None);
    let fees = tx.fees.as_ref().unwrap();
    assert_eq!(fees.base, 1.5);
    assert_eq!(fees.effective_feerate, Some(2.0));
    assert_eq!(
        fees.effective_includes.as_ref().unwrap(),
        &vec!["txid-1".to_string()]
    );

    let failed = parse_package(
        r#"{"package_msg":"transaction failed","tx-results":{"wtxid-2":{"txid":"txid-2","error":"txn-already-in-mempool"}}}"#,
    )
    .unwrap();
    assert_eq!(failed.package_msg, "transaction failed");
    assert!(failed.replaced_transactions.is_empty());
    let err = failed.tx_results.get("wtxid-2").unwrap();
    assert_eq!(err.error.as_deref(), Some("txn-already-in-mempool"));
    assert_eq!(err.fees, None);
    let _typed: PackageResult = failed;
}
