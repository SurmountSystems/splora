use std::path::Path;

use splora_frontend_shared::{
    Route, address_path, address_txs_path, address_utxo_path, block_height_path, block_path,
    block_txs_path, blocks_path, broadcast_body, broadcast_path, mempool_recent_path,
    parse_address, parse_address_txs, parse_address_utxos, parse_block, parse_block_height,
    parse_block_txs, parse_blocks, parse_broadcast, parse_mempool_recent, parse_test_txs, parse_tx,
    path_list, test_txs_body, test_txs_path, tx_path,
};

fn fixture(name: &str) -> String {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(name);
    std::fs::read_to_string(path).unwrap()
}

#[test]
fn path_list_matches_indexer_routes_after_the_public_prefix() {
    assert_eq!(
        path_list(),
        vec![
            Route {
                method: "GET",
                pattern: "/blocks",
            },
            Route {
                method: "GET",
                pattern: "/mempool/recent",
            },
            Route {
                method: "GET",
                pattern: "/block/:hash",
            },
            Route {
                method: "GET",
                pattern: "/block/:hash/txs",
            },
            Route {
                method: "GET",
                pattern: "/block-height/:height",
            },
            Route {
                method: "GET",
                pattern: "/tx/:txid",
            },
            Route {
                method: "GET",
                pattern: "/address/:script",
            },
            Route {
                method: "GET",
                pattern: "/address/:script/txs",
            },
            Route {
                method: "GET",
                pattern: "/address/:script/utxo",
            },
            Route {
                method: "POST",
                pattern: "/tx",
            },
            Route {
                method: "POST",
                pattern: "/txs/test",
            },
            Route {
                method: "GET",
                pattern: "/blocks/tip/hash",
            },
            Route {
                method: "GET",
                pattern: "/blocks/tip/height",
            },
            Route {
                method: "GET",
                pattern: "/blocks/:start_height",
            },
            Route {
                method: "GET",
                pattern: "/block/:hash/txids",
            },
            Route {
                method: "GET",
                pattern: "/block/:hash/txs/:start_index",
            },
            Route {
                method: "GET",
                pattern: "/mempool",
            },
            Route {
                method: "GET",
                pattern: "/fee-estimates",
            },
        ]
    );
}

#[test]
fn concrete_paths_have_no_public_prefix() {
    assert_eq!(blocks_path(), "/blocks");
    assert_eq!(mempool_recent_path(), "/mempool/recent");
    assert_eq!(block_path("abc"), "/block/abc");
    assert_eq!(block_txs_path("abc"), "/block/abc/txs");
    assert_eq!(block_height_path(10), "/block-height/10");
    assert_eq!(tx_path("abc"), "/tx/abc");
    assert_eq!(address_path("xyz"), "/address/xyz");
    assert_eq!(address_txs_path("xyz"), "/address/xyz/txs");
    assert_eq!(address_utxo_path("xyz"), "/address/xyz/utxo");
    assert_eq!(broadcast_path(), "/tx");
    assert_eq!(test_txs_path(), "/txs/test");
}

#[test]
fn broadcast_body_trims_whitespace_around_hex() {
    assert_eq!(broadcast_body("  deadbeef\n"), "deadbeef");
}

#[test]
fn test_txs_body_is_a_json_array_of_hex_strings() {
    assert_eq!(test_txs_body(&["aa", "bb"]), "[\"aa\",\"bb\"]");
}

#[test]
fn parses_blocks_screen() {
    let blocks = parse_blocks(&fixture("blocks.json")).unwrap();
    assert_eq!(blocks.len(), 1);
    let block = &blocks[0];
    assert_eq!(block.id, "block-id-1");
    assert_eq!(block.height, "100");
    assert_eq!(block.tx_count, "12");
    assert_eq!(block.timestamp, "1600000000");
    assert_eq!(block.size, "1500");
    assert_eq!(block.weight, "4000");
    assert_eq!(block.previous_hash, "prev-block-1");
}

#[test]
fn parses_block_screen() {
    let block = parse_block(&fixture("block.json")).unwrap();
    assert_eq!(block.id, "block-id-1");
    assert_eq!(block.height, "100");
    assert_eq!(block.tx_count, "12");
    assert_eq!(block.timestamp, "1600000000");
    assert_eq!(block.size, "1500");
    assert_eq!(block.weight, "4000");
    assert_eq!(block.previous_hash, "prev-block-1");
}

#[test]
fn parses_block_txs_screen() {
    let txs = parse_block_txs(&fixture("block_txs.json")).unwrap();
    assert_eq!(txs.len(), 1);
    let tx = &txs[0];
    assert_eq!(tx.txid, "block-tx-1");
    assert_eq!(tx.confirmed, "true");
    assert_eq!(tx.fee, "3000");
    assert_eq!(tx.size, "222");
    assert_eq!(tx.weight, "888");
}

#[test]
fn parses_tx_screen() {
    let tx = parse_tx(&fixture("tx.json")).unwrap();
    assert_eq!(tx.txid, "tx-1");
    assert_eq!(tx.confirmed, "true");
    assert_eq!(tx.fee, "4500");
    assert_eq!(tx.size, "180");
    assert_eq!(tx.weight, "720");
}

#[test]
fn parses_address_screen() {
    let stats = parse_address(&fixture("address.json")).unwrap();
    assert_eq!(stats.address, "xyz");
    assert_eq!(stats.chain_tx_count, "9");
    assert_eq!(stats.funded_sum, "500000");
    assert_eq!(stats.mempool_tx_count, "3");
}

#[test]
fn parses_address_txs_screen() {
    let txs = parse_address_txs(&fixture("address_txs.json")).unwrap();
    assert_eq!(txs.len(), 1);
    let tx = &txs[0];
    assert_eq!(tx.txid, "addr-tx-1");
    assert_eq!(tx.confirmed, "false");
    assert_eq!(tx.fee, "111");
    assert_eq!(tx.size, "90");
    assert_eq!(tx.weight, "360");
}

#[test]
fn parses_address_utxos_screen() {
    let utxos = parse_address_utxos(&fixture("address_utxos.json")).unwrap();
    assert_eq!(utxos.len(), 1);
    let utxo = &utxos[0];
    assert_eq!(utxo.txid, "utxo-tx-1");
    assert_eq!(utxo.vout, "2");
    assert_eq!(utxo.value, "42000");
}

#[test]
fn parses_mempool_recent_screen() {
    let recent = parse_mempool_recent(&fixture("mempool_recent.json")).unwrap();
    assert_eq!(recent.len(), 1);
    let tx = &recent[0];
    assert_eq!(tx.txid, "recent-tx-1");
    assert_eq!(tx.fee, "800");
    assert_eq!(tx.vsize, "141");
    assert_eq!(tx.value, "99000");
}

#[test]
fn parses_broadcast_result() {
    assert_eq!(
        parse_broadcast(&fixture("broadcast.txt")).unwrap(),
        "broadcast-txid-1"
    );
}

#[test]
fn parses_test_tx_result() {
    let results = parse_test_txs(&fixture("test_tx.json")).unwrap();
    assert_eq!(results.len(), 1);
    let result = &results[0];
    assert_eq!(result.txid, "test-tx-1");
    assert_eq!(result.allowed, "false");
    assert_eq!(result.reject_reason, "min relay fee not met");
}

#[test]
fn parses_block_height_body() {
    assert_eq!(
        parse_block_height(&fixture("block_height.txt")).unwrap(),
        "height-block-hash-1"
    );
}
