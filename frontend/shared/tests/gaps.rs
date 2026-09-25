use std::path::Path;

use splora_frontend_shared::{
    block_txids_path, block_txs_start_index_path, blocks_start_height_path, blocks_tip_hash_path,
    blocks_tip_height_path, fee_estimates_path, mempool_path, parse_address_txs,
    parse_address_utxos, parse_block_txids, parse_block_txs, parse_blocks_tip_hash,
    parse_blocks_tip_height, parse_fee_estimates, parse_mempool, parse_test_txs, parse_tx,
    path_list,
};

fn fixture(name: &str) -> String {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(name);
    std::fs::read_to_string(path).unwrap()
}

#[test]
fn path_list_includes_the_routes_rest_already_matches() {
    let routes = path_list();
    let expected = [
        ("GET", "/blocks/tip/hash"),
        ("GET", "/blocks/tip/height"),
        ("GET", "/blocks/:start_height"),
        ("GET", "/block/:hash/txids"),
        ("GET", "/block/:hash/txs/:start_index"),
        ("GET", "/mempool"),
        ("GET", "/fee-estimates"),
    ];
    for (method, pattern) in expected {
        assert!(
            routes
                .iter()
                .any(|route| route.method == method && route.pattern == pattern),
            "missing {method} {pattern}"
        );
    }
}

#[test]
fn allowed_test_tx_may_omit_reject_reason() {
    let body = r#"[{"txid":"ok-tx","wtxid":"ok-wtxid","allowed":true,"vsize":100}]"#;
    let rows = parse_test_txs(body).expect("allowed row without reject-reason");
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].txid, "ok-tx");
    assert_eq!(rows[0].allowed, "true");
    assert_eq!(rows[0].reject_reason, "");
}

#[test]
fn rejected_test_tx_still_requires_reject_reason() {
    let body = r#"[{"txid":"bad-tx","wtxid":"bad-wtxid","allowed":false}]"#;
    let err = parse_test_txs(body).expect_err("rejected row without reject-reason");
    assert!(
        err.detail.contains("reject-reason"),
        "detail was {}",
        err.detail
    );
}

#[test]
fn concrete_added_paths_use_the_rest_suffixes() {
    assert_eq!(blocks_tip_hash_path(), "/blocks/tip/hash");
    assert_eq!(blocks_tip_height_path(), "/blocks/tip/height");
    assert_eq!(blocks_start_height_path(800000), "/blocks/800000");
    assert_eq!(fee_estimates_path(), "/fee-estimates");
    assert_eq!(mempool_path(), "/mempool");
    assert_eq!(block_txids_path("abc"), "/block/abc/txids");
    assert_eq!(block_txs_start_index_path("abc", 0), "/block/abc/txs/0");
    assert_eq!(block_txs_start_index_path("abc", 25), "/block/abc/txs/25");
}

#[test]
fn parses_tip_hash_and_tip_height_as_plain_text() {
    assert_eq!(
        parse_blocks_tip_hash("  tip-hash-1\n").unwrap(),
        "tip-hash-1"
    );
    assert!(parse_blocks_tip_hash(" \n").is_err());
    assert_eq!(parse_blocks_tip_height(" 800000\n").unwrap(), "800000");
    assert!(parse_blocks_tip_height("not-a-height").is_err());
}

#[test]
fn parses_fee_estimates_object() {
    let rows = parse_fee_estimates(r#"{"6":5,"1":12.5}"#).unwrap();
    assert_eq!(rows.len(), 2);
    assert_eq!(rows[0].target, "1");
    assert_eq!(rows[0].rate, "12.5");
    assert_eq!(rows[1].target, "6");
    assert_eq!(rows[1].rate, "5");
}

#[test]
fn parses_mempool_backlog() {
    let body = r#"{"count":2,"vsize":300,"total_fee":1500,"fee_histogram":[[10.5,200],[1,100]]}"#;
    let stats = parse_mempool(body).unwrap();
    assert_eq!(stats.count, "2");
    assert_eq!(stats.vsize, "300");
    assert_eq!(stats.total_fee, "1500");
    assert_eq!(
        stats.fee_histogram,
        vec![
            ("10.5".to_string(), "200".to_string()),
            ("1".to_string(), "100".to_string())
        ]
    );
}

#[test]
fn parses_block_txids() {
    let ids = parse_block_txids(r#"["aa","bb"]"#).unwrap();
    assert_eq!(ids, vec!["aa".to_string(), "bb".to_string()]);
}

#[test]
fn transaction_record_keeps_status_block_height() {
    let tx = parse_tx(&fixture("tx.json")).unwrap();
    assert_eq!(tx.block_height, "100");
    let block_txs = parse_block_txs(&fixture("block_txs.json")).unwrap();
    assert_eq!(block_txs[0].block_height, "100");
    let address_txs = parse_address_txs(&fixture("address_txs.json")).unwrap();
    assert_eq!(address_txs[0].block_height, "");
}

#[test]
fn utxo_record_keeps_status_confirmed() {
    let utxos = parse_address_utxos(&fixture("address_utxos.json")).unwrap();
    assert_eq!(utxos[0].confirmed, "true");
    let unconfirmed =
        parse_address_utxos(r#"[{"txid":"u","vout":0,"value":1,"status":{"confirmed":false}}]"#)
            .unwrap();
    assert_eq!(unconfirmed[0].confirmed, "false");
}
