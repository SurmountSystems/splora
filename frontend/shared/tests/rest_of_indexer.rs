//! Parsers and path strings for public indexer routes neither app requests.
//!
//! Shapes follow `handle_request` in `src/rest.rs`. Liquid routes stay path
//! strings. They are not rows in [`splora_frontend_shared::path_list`].

use splora_frontend_shared::wire::{Outspend, TxStatus, TxSummary};
use splora_frontend_shared::{
    IndexerCall, IssuedRegistryAsset, NativeRegistryAsset, RegistryAsset, TXS_OUTSPENDS_MAX,
    addresses_txs_body, addresses_txs_path, addresses_txs_summary_page_path,
    addresses_txs_summary_path, asset_supply_decimal_path, asset_supply_path,
    asset_txs_chain_page_path, asset_txs_chain_path, asset_txs_mempool_path,
    assets_registry_page_path, assets_registry_path, assets_registry_search_path,
    block_header_path, block_raw_path, block_txid_path, indexer_request, parse_addresses_txs,
    parse_addresses_txs_summary, parse_asset_registry, parse_asset_supply,
    parse_asset_supply_decimal, parse_asset_txs_chain, parse_asset_txs_mempool,
    parse_assets_registry, parse_assets_registry_search, parse_assets_registry_total,
    parse_block_header, parse_block_header_bytes, parse_block_raw, parse_block_raw_hex,
    parse_block_txid, parse_merkleblock_proof, parse_merkleblock_proof_bytes,
    parse_scripthash_txs_summary, parse_scripthashes_txs, parse_scripthashes_txs_summary,
    parse_tx_raw, parse_tx_raw_hex, parse_tx_status, parse_txs_outspends, path_list,
    scripthash_txs_summary_page_path, scripthash_txs_summary_path, scripthashes_txs_body,
    scripthashes_txs_path, scripthashes_txs_summary_page_path, scripthashes_txs_summary_path,
    tx_merkleblock_proof_path, tx_raw_path, tx_status_path, txs_outspends_path,
};

fn fixture(name: &str) -> String {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(name);
    std::fs::read_to_string(path).unwrap()
}

#[test]
fn summary_and_batch_paths_match_the_indexer() {
    assert_eq!(
        scripthash_txs_summary_path("aa"),
        "/scripthash/aa/txs/summary"
    );
    assert_eq!(
        scripthash_txs_summary_page_path("aa", "txid-9"),
        "/scripthash/aa/txs/summary/txid-9"
    );
    assert_eq!(addresses_txs_path(), "/addresses/txs");
    assert_eq!(scripthashes_txs_path(), "/scripthashes/txs");
    assert_eq!(addresses_txs_summary_path(), "/addresses/txs/summary");
    assert_eq!(
        addresses_txs_summary_page_path("txid-9"),
        "/addresses/txs/summary/txid-9"
    );
    assert_eq!(scripthashes_txs_summary_path(), "/scripthashes/txs/summary");
    assert_eq!(
        scripthashes_txs_summary_page_path("txid-9"),
        "/scripthashes/txs/summary/txid-9"
    );
    assert_eq!(
        addresses_txs_body(&["addr1", "addr2"]),
        "[\"addr1\",\"addr2\"]"
    );
    assert_eq!(scripthashes_txs_body(&["aa", "bb"]), "[\"aa\",\"bb\"]");
}

#[test]
fn status_raw_and_outspend_paths_match_the_indexer() {
    assert_eq!(TXS_OUTSPENDS_MAX, 50);
    assert_eq!(
        txs_outspends_path(&["aa", "bb"]).unwrap(),
        "/txs/outspends?txids=aa,bb"
    );
    let fifty: Vec<String> = (0..50).map(|index| format!("t{index}")).collect();
    let fifty_refs: Vec<&str> = fifty.iter().map(String::as_str).collect();
    assert!(txs_outspends_path(&fifty_refs).is_ok());
    let mut too_many = fifty_refs.clone();
    too_many.push("extra");
    let err = txs_outspends_path(&too_many).unwrap_err();
    assert!(err.contains("50"), "detail was {err}");
    assert!(txs_outspends_path(&[]).is_err());
    assert!(txs_outspends_path(&["aa,bb"]).is_err());

    assert_eq!(tx_status_path("abc"), "/tx/abc/status");
    assert_eq!(block_txid_path("abc", 4), "/block/abc/txid/4");
    assert_eq!(block_header_path("abc"), "/block/abc/header");
    assert_eq!(block_raw_path("abc"), "/block/abc/raw");
    assert_eq!(tx_raw_path("abc"), "/tx/abc/raw");
    assert_eq!(
        tx_merkleblock_proof_path("abc"),
        "/tx/abc/merkleblock-proof"
    );
}

#[test]
fn liquid_paths_stay_out_of_the_route_table() {
    assert_eq!(assets_registry_path(), "/assets/registry");
    assert_eq!(
        assets_registry_search_path("USDt"),
        "/assets/registry/search?q=USDt"
    );
    assert_eq!(
        assets_registry_search_path("a b"),
        "/assets/registry/search?q=a%20b"
    );
    assert_eq!(
        assets_registry_search_path(""),
        "/assets/registry/search?q="
    );
    assert_eq!(asset_txs_chain_path("liq"), "/asset/liq/txs/chain");
    assert_eq!(
        asset_txs_chain_page_path("liq", "txid-9"),
        "/asset/liq/txs/chain/txid-9"
    );
    assert_eq!(asset_txs_mempool_path("liq"), "/asset/liq/txs/mempool");
    assert_eq!(asset_supply_path("liq"), "/asset/liq/supply");
    assert_eq!(
        asset_supply_decimal_path("liq"),
        "/asset/liq/supply/decimal"
    );

    let routes = path_list();
    for pattern in [
        "/scripthash/:hash/txs/summary",
        "/scripthash/:hash/txs/summary/:last_seen_txid",
        "/addresses/txs",
        "/scripthashes/txs",
        "/addresses/txs/summary",
        "/addresses/txs/summary/:last_seen_txid",
        "/scripthashes/txs/summary",
        "/scripthashes/txs/summary/:last_seen_txid",
        "/txs/outspends",
        "/tx/:txid/status",
        "/block/:hash/txid/:index",
        "/block/:hash/header",
        "/block/:hash/raw",
        "/tx/:txid/raw",
        "/tx/:txid/merkleblock-proof",
    ] {
        assert!(
            routes.iter().any(|route| route.pattern == pattern),
            "missing {pattern}"
        );
    }
    for pattern in [
        "/assets/registry",
        "/assets/registry/:asset",
        "/assets/registry/search",
        "/asset/:asset/txs/chain",
        "/asset/:asset/txs/chain/:last_seen_txid",
        "/asset/:asset/txs/mempool",
        "/asset/:asset/supply",
        "/asset/:asset/supply/decimal",
        "/internal/txs",
        "/electrum",
        "/api/v1/ws",
        "/block-template",
    ] {
        assert!(
            routes.iter().all(|route| route.pattern != pattern),
            "{pattern} must not be registered"
        );
    }
}

/// `GET /assets/registry` reads `start_index`, `limit`, `sort_field`, and
/// `sort_dir`. A missing sort key is the default. An empty sort key is
/// rejected, so `Some("")` stays `key=`.
#[test]
fn assets_registry_page_appends_the_indexer_query() {
    assert_eq!(assets_registry_path(), "/assets/registry");
    assert_eq!(
        assets_registry_page_path(None, None, None, None),
        "/assets/registry"
    );
    assert_eq!(
        assets_registry_page_path(Some("25"), Some("10"), Some("name"), Some("desc")),
        "/assets/registry?start_index=25&limit=10&sort_field=name&sort_dir=desc"
    );
    assert_eq!(
        assets_registry_page_path(Some("25"), None, Some("a b"), Some("a&b=c")),
        "/assets/registry?start_index=25&sort_field=a%20b&sort_dir=a%26b%3Dc"
    );
    assert_eq!(
        assets_registry_page_path(None, None, Some(""), Some("")),
        "/assets/registry?sort_field=&sort_dir="
    );
    assert_eq!(
        assets_registry_page_path(Some(""), Some(""), None, None),
        "/assets/registry?start_index=&limit="
    );
    assert_eq!(
        assets_registry_search_path("a b"),
        "/assets/registry/search?q=a%20b"
    );
}

#[test]
fn indexer_request_builds_the_new_calls() {
    let summary = indexer_request(&IndexerCall::ScriptHashTxsSummary {
        hash: "aa".to_string(),
    });
    assert_eq!(summary.method, "GET");
    assert_eq!(summary.path, scripthash_txs_summary_path("aa"));
    assert_eq!(summary.body, None);

    let batch = indexer_request(&IndexerCall::AddressesTxs {
        scripts: vec!["addr1".to_string(), "addr2".to_string()],
    });
    assert_eq!(batch.method, "POST");
    assert_eq!(batch.path, addresses_txs_path());
    assert_eq!(batch.body.as_deref(), Some("[\"addr1\",\"addr2\"]"));

    let page = indexer_request(&IndexerCall::AddressesTxsSummaryPage {
        scripts: vec!["addr1".to_string()],
        last_seen_txid: "txid-9".to_string(),
    });
    assert_eq!(page.method, "POST");
    assert_eq!(page.path, addresses_txs_summary_page_path("txid-9"));
    assert_eq!(page.body.as_deref(), Some("[\"addr1\"]"));

    let hashes = indexer_request(&IndexerCall::ScriptHashesTxsSummary {
        hashes: vec!["aa".to_string()],
    });
    assert_eq!(hashes.method, "POST");
    assert_eq!(hashes.path, scripthashes_txs_summary_path());
    assert_eq!(hashes.body.as_deref(), Some("[\"aa\"]"));

    let status = indexer_request(&IndexerCall::TxStatus {
        txid: "abc".to_string(),
    });
    assert_eq!(status.path, tx_status_path("abc"));
    assert_eq!(status.method, "GET");

    let proof = indexer_request(&IndexerCall::TxMerkleBlockProof {
        txid: "abc".to_string(),
    });
    assert_eq!(proof.path, tx_merkleblock_proof_path("abc"));

    let search = indexer_request(&IndexerCall::AssetsRegistrySearch {
        query: "a b".to_string(),
    });
    assert_eq!(search.method, "GET");
    assert_eq!(search.path, assets_registry_search_path("a b"));
    assert_eq!(search.body, None);

    let supply = indexer_request(&IndexerCall::AssetSupplyDecimal {
        asset: "liq".to_string(),
    });
    assert_eq!(supply.path, asset_supply_decimal_path("liq"));
}

#[test]
fn parses_compact_history_with_tx_summary() {
    let body = r#"[{"txid":"sum-1","height":100,"value":5000,"time":1600000000,"tx_position":2},{"txid":"sum-2","height":99,"value":-100,"time":1599999000,"tx_position":0}]"#;
    let rows: Vec<TxSummary> = parse_scripthash_txs_summary(body).unwrap();
    assert_eq!(rows[0].txid, "sum-1");
    assert_eq!(rows[0].value, 5000);
    assert_eq!(rows[1].value, -100);

    let batched: Vec<TxSummary> = parse_addresses_txs_summary(body).unwrap();
    assert_eq!(batched, rows);
    let hashed: Vec<TxSummary> = parse_scripthashes_txs_summary(body).unwrap();
    assert_eq!(hashed[1].tx_position, 0);

    let txs = parse_addresses_txs(&fixture("address_txs.json")).unwrap();
    assert_eq!(txs[0].txid, "addr-tx-1");
    assert_eq!(txs[0].confirmed, "false");
    let same = parse_scripthashes_txs(&fixture("address_txs.json")).unwrap();
    assert_eq!(same[0].fee, "111");
}

#[test]
fn parses_tx_status_one_outspend_and_batched_outspends() {
    let status = parse_tx_status(
        r#"{"confirmed":true,"block_height":100,"block_hash":"block-id-1","block_time":1600000000}"#,
    )
    .unwrap();
    assert_eq!(
        status,
        TxStatus {
            confirmed: true,
            block_height: Some(100),
            block_hash: Some("block-id-1".to_string()),
            block_time: Some(1_600_000_000),
        }
    );
    let unconfirmed = parse_tx_status(r#"{"confirmed":false}"#).unwrap();
    assert!(!unconfirmed.confirmed);
    assert_eq!(unconfirmed.block_height, None);
    assert!(parse_tx_status(r#"{"in_best_chain":true,"height":1,"next_best":null}"#).is_err());

    let groups = parse_txs_outspends(
        r#"[[{"spent":false}],[{"spent":true,"txid":"spend-tx","vin":1,"status":{"confirmed":false}}]]"#,
    )
    .unwrap();
    assert_eq!(groups.len(), 2);
    assert_eq!(
        groups[0][0],
        Outspend {
            spent: false,
            txid: None,
            vin: None,
            status: None,
        }
    );
    assert_eq!(groups[1][0].txid.as_deref(), Some("spend-tx"));
    assert_eq!(groups[1][0].vin, Some(1));
}

#[test]
fn parses_plain_txid_and_hex_or_raw_bytes() {
    assert_eq!(parse_block_txid("  abcdef\n").unwrap(), "abcdef");
    assert!(parse_block_txid(" \n").is_err());

    assert_eq!(parse_block_header("  00ff\n").unwrap(), "00ff");
    assert_eq!(parse_block_header_bytes("00ff").unwrap(), vec![0x00, 0xff]);
    assert!(parse_block_header("zz").is_err());
    assert!(parse_block_header("abc").is_err());
    assert!(parse_block_header("").is_err());

    assert_eq!(parse_block_raw(&[1, 2, 3]).unwrap(), vec![1, 2, 3]);
    assert!(parse_block_raw(&[]).is_err());
    assert_eq!(parse_block_raw_hex("010203").unwrap(), "010203");
    assert!(parse_block_raw_hex("01020").is_err());

    assert_eq!(parse_tx_raw(&[9]).unwrap(), vec![9]);
    assert!(parse_tx_raw(&[]).is_err());
    assert_eq!(parse_tx_raw_hex("  ab\n").unwrap(), "ab");

    assert_eq!(parse_merkleblock_proof("ab").unwrap(), "ab");
    assert_eq!(parse_merkleblock_proof_bytes("ab").unwrap(), vec![0xab]);
    assert!(parse_merkleblock_proof(r#"{"block_height":1}"#).is_err());
}

#[test]
fn parses_liquid_registry_search_entry_and_page() {
    let hits = parse_assets_registry_search(
        r#"[{"asset_id":"aa","name":"Tether USD","ticker":"USDt","domain":"tether.to"},{"asset_id":"bb","name":"Bare"}]"#,
    )
    .unwrap();
    assert_eq!(hits[0].asset_id, "aa");
    assert_eq!(hits[0].name, "Tether USD");
    assert_eq!(hits[0].ticker.as_deref(), Some("USDt"));
    assert_eq!(hits[0].domain.as_deref(), Some("tether.to"));
    assert_eq!(hits[1].ticker, None);
    assert_eq!(hits[1].domain, None);

    let entry = parse_asset_registry(
        r#"{"contract":{"entity":{"domain":"example.com"}},"entity":{"domain":"example.com"},"precision":8,"name":"Test","ticker":"TST"}"#,
    )
    .unwrap();
    assert_eq!(entry.precision, 8);
    assert_eq!(entry.name, "Test");
    assert_eq!(entry.ticker.as_deref(), Some("TST"));
    assert_eq!(entry.entity["domain"].as_str(), Some("example.com"));
    assert!(entry.contract.is_object());

    let bare = parse_asset_registry(r#"{"precision":0,"name":"Bare"}"#).unwrap();
    assert_eq!(bare.precision, 0);
    assert_eq!(bare.name, "Bare");
    assert_eq!(bare.ticker, None);
    assert!(bare.contract.is_null());
    assert!(bare.entity.is_null());

    let page = parse_assets_registry(
        r#"[
            {
                "asset_id":"issued-1",
                "issuance_txin":{"txid":"issue-tx","vin":0},
                "issuance_prevout":{"txid":"prev-tx","vout":1},
                "reissuance_token":"token-1",
                "contract_hash":"hash-1",
                "status":{"confirmed":true,"block_height":100,"block_hash":"block-id-1","block_time":1600000000},
                "chain_stats":{"tx_count":2,"issuance_count":1,"issued_amount":1000,"burned_amount":5,"has_blinded_issuances":false,"reissuance_tokens":1,"burned_reissuance_tokens":0},
                "mempool_stats":{"tx_count":0,"issuance_count":0,"issued_amount":0,"burned_amount":0,"has_blinded_issuances":false,"reissuance_tokens":null,"burned_reissuance_tokens":0},
                "contract":{"name":"c"},
                "entity":{"domain":"example.com"},
                "precision":8,
                "name":"Test",
                "ticker":"TST"
            },
            {
                "asset_id":"native-1",
                "chain_stats":{"tx_count":3,"peg_in_count":1,"peg_in_amount":50,"peg_out_count":0,"peg_out_amount":0,"burn_count":0,"burned_amount":0},
                "mempool_stats":{"tx_count":0,"peg_in_count":0,"peg_in_amount":0,"peg_out_count":0,"peg_out_amount":0,"burn_count":0,"burned_amount":0}
            }
        ]"#,
    )
    .unwrap();
    assert_eq!(page.len(), 2);
    match &page[0] {
        RegistryAsset::Issued(asset) => {
            assert_issued(asset);
        }
        RegistryAsset::Native(_) => panic!("issued asset parsed as native"),
    }
    match &page[1] {
        RegistryAsset::Native(asset) => {
            assert_eq!(asset.asset_id, "native-1");
            assert_eq!(asset.chain_stats.peg_in_amount, 50);
            assert_eq!(asset.chain_stats.tx_count, 3);
            assert_eq!(asset.mempool_stats.peg_out_amount, 0);
        }
        RegistryAsset::Issued(_) => panic!("native asset parsed as issued"),
    }
    assert_eq!(parse_assets_registry_total("  12\n").unwrap(), 12);
    assert!(parse_assets_registry_total("nope").is_err());
}

fn assert_issued(asset: &IssuedRegistryAsset) {
    assert_eq!(asset.asset_id, "issued-1");
    assert_eq!(asset.issuance_txin.txid, "issue-tx");
    assert_eq!(asset.issuance_txin.vin, 0);
    assert_eq!(asset.issuance_prevout.txid, "prev-tx");
    assert_eq!(asset.issuance_prevout.vout, 1);
    assert_eq!(asset.reissuance_token, "token-1");
    assert_eq!(asset.contract_hash.as_deref(), Some("hash-1"));
    assert!(asset.status.confirmed);
    assert_eq!(asset.status.block_height, Some(100));
    assert_eq!(asset.chain_stats.issued_amount, 1000);
    assert_eq!(asset.chain_stats.burned_amount, 5);
    assert!(!asset.chain_stats.has_blinded_issuances);
    assert_eq!(asset.chain_stats.reissuance_tokens, Some(1));
    assert_eq!(asset.mempool_stats.reissuance_tokens, None);
    assert_eq!(asset.precision, Some(8));
    assert_eq!(asset.name.as_deref(), Some("Test"));
    assert_eq!(asset.ticker.as_deref(), Some("TST"));
    assert_eq!(asset.entity["domain"].as_str(), Some("example.com"));
}

#[test]
fn parses_asset_history_and_supply_text() {
    let chain = parse_asset_txs_chain(&fixture("address_txs.json")).unwrap();
    assert_eq!(chain[0].txid, "addr-tx-1");
    let mempool = parse_asset_txs_mempool(&fixture("address_txs.json")).unwrap();
    assert_eq!(mempool[0].weight, "360");

    assert_eq!(parse_asset_supply("  21000000\n").unwrap(), "21000000");
    assert!(parse_asset_supply("1.5").is_err());
    assert!(parse_asset_supply("").is_err());
    assert_eq!(parse_asset_supply_decimal("1.5").unwrap(), "1.5");
    assert_eq!(
        parse_asset_supply_decimal("  21000000\n").unwrap(),
        "21000000"
    );
    assert!(parse_asset_supply_decimal("not-a-number").is_err());
    assert!(parse_asset_supply_decimal("").is_err());

    let _native: NativeRegistryAsset = NativeRegistryAsset {
        asset_id: "native-1".to_string(),
        chain_stats: splora_frontend_shared::PeggedAssetStats {
            tx_count: 3,
            peg_in_count: 1,
            peg_in_amount: 50,
            peg_out_count: 0,
            peg_out_amount: 0,
            burn_count: 0,
            burned_amount: 0,
        },
        mempool_stats: splora_frontend_shared::PeggedAssetStats {
            tx_count: 0,
            peg_in_count: 0,
            peg_in_amount: 0,
            peg_out_count: 0,
            peg_out_amount: 0,
            burn_count: 0,
            burned_amount: 0,
        },
    };
}
