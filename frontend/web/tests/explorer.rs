use std::cell::RefCell;

use splora_api::{Network, signet_is_backend};
use splora_frontend_shared::{FeeEstimate, parse_block, parse_tx};
use splora_web::{
    ACCENT, BACKGROUND, ClientError, Nip98Signer, TEXT, UnsignedNip98, address_fields_text,
    address_path, address_requests, address_txs_path, block_fields_text, block_load_plan,
    block_path, block_txids_path, block_txs_path, blocks_list_fields_text, blocks_list_requests,
    broadcast_fields_text, broadcast_payload, broadcast_request, broadcast_screen_request,
    browser_fetch_url, dashboard_fields_text, dashboard_paths, dashboard_screen_requests,
    docs_api_type_page, entered_addresses, fetch_indexer, mempool_paths, multi_address_fields_text,
    multi_address_requests, multi_address_requests_from_values, multi_address_screen_requests,
    multi_address_screen_route, nip98_u_url, parse_blocks_tip_hash, parse_blocks_tip_height,
    parse_fee_estimates, post_indexer, screen_routes, static_page_requests, static_page_routes,
    switcher_labels, test_transactions_fields_text, test_transactions_payload,
    test_transactions_request, test_transactions_screen_request, transaction_fields_text,
    transaction_requests, tx_path, unsigned_get, v1_client_requests,
};

const ORIGIN: &str = "https://splora.surmount.systems";

#[test]
fn nip98_unsigned_event_is_kind_27235() {
    let event = unsigned_get("ab", 1_700_000_000, ORIGIN, "/tx/abc");
    assert_eq!(event.kind, 27235);
    assert_eq!(event.content, "");
    assert!(
        event
            .tags
            .iter()
            .any(|tag| tag.len() == 2 && tag[0] == "method" && tag[1] == "GET")
    );
    assert!(event.tags.iter().any(|tag| {
        tag.len() == 2 && tag[0] == "u" && tag[1] == "https://splora.surmount.systems/tx/abc"
    }));
    // The type name UnsignedNip98 itself contains the letters "sig".
    assert!(
        !format!("{event:?}")
            .replace("UnsignedNip98", "")
            .contains("sig")
    );
}

#[test]
fn stripped_u_tag_drops_network_prefix() {
    for network in Network::ALL {
        let fetch = browser_fetch_url(ORIGIN, network, "/tx/abc");
        let u = nip98_u_url(ORIGIN, "/tx/abc");
        assert!(fetch.contains(&format!("{}{}", network.api_prefix(), "/tx/abc")));
        assert_eq!(u, "https://splora.surmount.systems/tx/abc");
        assert!(!u.contains(network.api_prefix()));
    }
}

#[test]
fn signet_is_not_a_network() {
    let labels = switcher_labels();
    assert_eq!(labels.len(), 5);
    assert!(labels.iter().all(|label| !label.contains("signet")));
    assert!(!signet_is_backend());
}

#[test]
fn v1_routes_match_indexer() {
    assert_eq!(
        dashboard_paths(),
        [
            "/blocks/tip/hash",
            "/blocks/tip/height",
            "/blocks",
            "/fee-estimates",
            "/mempool",
        ]
    );
    assert_eq!(mempool_paths(), ["/mempool", "/mempool/recent"]);
    assert_eq!(
        screen_routes(),
        [
            "/",
            "/blocks",
            "/block/:hash",
            "/widget/wallet",
            "/tx/push",
            "/pushtx",
            "/tx/test",
            "/tx/:txid",
            "/address/:addr",
            "/mempool",
            "/terms-of-service",
            "/privacy-policy",
            "/trademark-policy",
            "/docs",
            "/docs/faq",
            "/docs/api/rest",
            "/docs/api/:type",
        ]
    );
    assert_eq!(block_path("aa"), "/block/aa");
    assert_eq!(block_txids_path("aa"), "/block/aa/txids");
    assert_eq!(block_txs_path("aa", 0).as_deref(), Ok("/block/aa/txs/0"));
    assert_eq!(block_txs_path("aa", 25).as_deref(), Ok("/block/aa/txs/25"));
    assert!(block_txs_path("aa", 26).is_err());
    assert_eq!(tx_path("bb"), "/tx/bb");
    assert_eq!(address_path("addr"), "/address/addr");
    assert_eq!(address_txs_path("addr"), "/address/addr/txs");
}

#[test]
fn theme_is_black_and_white() {
    assert_eq!(BACKGROUND, "#000000");
    assert_eq!(TEXT, "#FFFFFF");
    assert_eq!(ACCENT, "#FFD100");
}

struct RecordingSigner {
    seen: RefCell<Option<UnsignedNip98>>,
}

impl Nip98Signer for RecordingSigner {
    fn public_key_hex(&self) -> Result<String, ClientError> {
        Ok("ab".repeat(32))
    }

    fn sign_event_json(&self, unsigned: &UnsignedNip98) -> Result<String, ClientError> {
        *self.seen.borrow_mut() = Some(unsigned.clone());
        Ok(r#"{"kind":27235}"#.to_string())
    }
}

#[test]
fn present_signer_fetches_with_nostr_header() {
    let signer = RecordingSigner {
        seen: RefCell::new(None),
    };
    let mut seen_url = String::new();
    let mut seen_header = String::new();
    let body = fetch_indexer(
        Some(&signer),
        ORIGIN,
        Network::Mutinynet,
        "/tx/abc",
        1_700_000_000,
        &mut |url, authorization| {
            seen_url = url.to_string();
            seen_header = authorization.to_string();
            Ok(b"{}".to_vec())
        },
    )
    .expect("signer present");
    assert_eq!(body, b"{}");
    assert_eq!(
        seen_url,
        "https://splora.surmount.systems/mutinynet/api/tx/abc"
    );
    assert!(seen_header.starts_with("Nostr "));
    let unsigned = signer.seen.borrow().clone().expect("unsigned");
    assert!(unsigned.tags.iter().any(|tag| {
        tag.len() == 2 && tag[0] == "u" && tag[1] == "https://splora.surmount.systems/tx/abc"
    }));
    assert!(!seen_header.contains("nsec"));
}

#[test]
fn parses_block_tx_and_plain_tip() {
    let block = parse_block(
        r#"{"id":"aa","height":3,"timestamp":9,"tx_count":1,"size":2,"weight":8,"previousblockhash":"zz","extra":true}"#,
    )
    .expect("block");
    assert_eq!(block.height.parse::<u32>().expect("height"), 3);
    assert_eq!(block.previous_hash, "zz");
    let tx = parse_tx(
        r#"{"txid":"aa","fee":1,"size":1,"weight":4,"status":{"confirmed":false,"extra":true}}"#,
    )
    .expect("tx");
    assert_eq!(tx.confirmed, "false");
    assert_eq!(parse_blocks_tip_hash("  abc\n").expect("tip hash"), "abc");
    let height = parse_blocks_tip_height(" 10 ").expect("tip height");
    assert_eq!(height, "10");
    assert_eq!(height.parse::<u64>().expect("height number"), 10);
    let fees = parse_fee_estimates(r#"{"2":4.5,"1":8}"#).expect("fees");
    assert_eq!(
        fees,
        vec![
            FeeEstimate {
                target: "1".to_string(),
                rate: "8".to_string(),
            },
            FeeEstimate {
                target: "2".to_string(),
                rate: "4.5".to_string(),
            },
        ]
    );
    assert_eq!(fees[0].target.parse::<u32>().expect("target"), 1);
    assert_eq!(fees[0].rate.parse::<f64>().expect("rate"), 8.0);
    assert_eq!(fees[1].target.parse::<u32>().expect("target"), 2);
    assert_eq!(fees[1].rate.parse::<f64>().expect("rate"), 4.5);
}

#[test]
fn failed_sign_shows_fail_closed_line() {
    let text = splora_web::text_after_signed_get(Err(splora_web::ClientError::MissingSigner));
    assert_eq!(text, splora_web::FAIL_CLOSED);
    assert!(!text.is_empty());
    assert_eq!(
        splora_web::text_after_signed_get(Ok("{\"ok\":true}".into())),
        "{\"ok\":true}"
    );
}

#[test]
fn recent_rows_link_by_txid() {
    let ids = splora_web::recent_txids(r#"[{"txid":"aa","fee":1,"vsize":2,"value":3}]"#);
    assert_eq!(ids, vec!["aa".to_string()]);
    assert_eq!(splora_web::tx_path(&ids[0]), "/tx/aa");
    assert!(splora_web::recent_txids("not-json").is_empty());
}

#[test]
fn broadcast_client_url_is_post_tx() {
    let call = broadcast_request();
    assert_eq!(call.method, "POST");
    assert_eq!(call.path, "/tx");
    let signer = RecordingSigner {
        seen: RefCell::new(None),
    };
    let mut seen_method = String::new();
    let mut seen_url = String::new();
    let body = post_indexer(
        Some(&signer),
        ORIGIN,
        Network::Mainnet,
        &call.path,
        1_700_000_000,
        "abcd",
        &mut |method, url, authorization, payload| {
            seen_method = method.to_string();
            seen_url = url.to_string();
            assert!(authorization.starts_with("Nostr "));
            assert!(!authorization.contains("nsec"));
            assert_eq!(payload, "abcd");
            Ok(b"ok".to_vec())
        },
    )
    .expect("broadcast");
    assert_eq!(body, b"ok");
    assert_eq!(seen_method, "POST");
    assert_eq!(seen_url, "https://splora.surmount.systems/api/tx");
    assert!(!seen_url.contains("/txs/package"));
    assert!(!seen_url.contains("/v1/"));
    let unsigned = signer.seen.borrow().clone().expect("unsigned");
    assert!(
        unsigned
            .tags
            .iter()
            .any(|tag| { tag.len() == 2 && tag[0] == "method" && tag[1] == "POST" })
    );

    let mut called = false;
    let missing = post_indexer(
        None,
        ORIGIN,
        Network::Mainnet,
        "/tx",
        1,
        "abcd",
        &mut |_, _, _, _| {
            called = true;
            Ok(Vec::new())
        },
    );
    assert!(matches!(missing, Err(ClientError::MissingSigner)));
    assert!(!called);

    called = false;
    let package = post_indexer(
        Some(&signer),
        ORIGIN,
        Network::Mainnet,
        "/txs/package",
        1,
        "abcd",
        &mut |_, _, _, _| {
            called = true;
            Ok(Vec::new())
        },
    );
    assert!(matches!(package, Err(ClientError::RejectedEvent)));
    assert!(!called);

    called = false;
    let packaged = post_indexer(
        Some(&signer),
        ORIGIN,
        Network::Mainnet,
        "/api/v1/txs/package",
        1,
        "abcd",
        &mut |_, _, _, _| {
            called = true;
            Ok(Vec::new())
        },
    );
    assert!(matches!(packaged, Err(ClientError::RejectedEvent)));
    assert!(!called);
}

#[test]
fn client_urls_omit_mining_price_lightning_and_accelerator() {
    let forbidden = [
        "/v1/mining",
        "/historical-price",
        "/lightning",
        "/accelerator",
    ];
    for network in Network::ALL {
        for call in v1_client_requests() {
            let url = browser_fetch_url(ORIGIN, network, &call.path);
            assert!(!url.contains("/v1/block"), "{url}");
            assert!(!call.path.contains("/v1/block"));
            for needle in forbidden {
                assert!(!url.contains(needle), "{url} contains {needle}");
                assert!(
                    !call.path.contains(needle),
                    "{} contains {needle}",
                    call.path
                );
            }
        }
    }
}

#[test]
fn multi_address_builds_one_get_per_addresses_query() {
    assert_eq!(multi_address_screen_route(), "/widget/wallet");
    assert!(!multi_address_screen_route().starts_with("/wallet/"));
    let calls = multi_address_requests("addresses=bc1qone,bc1qtwo&unused=1");
    assert_eq!(calls.len(), 2);
    assert_eq!(calls[0].method, "GET");
    assert_eq!(calls[0].path, "/address/bc1qone");
    assert_eq!(calls[1].method, "GET");
    assert_eq!(calls[1].path, "/address/bc1qtwo");
    let repeated = multi_address_requests("?addresses=one&addresses=two");
    assert_eq!(repeated.len(), 2);
    assert_eq!(repeated[0].path, "/address/one");
    assert_eq!(repeated[1].path, "/address/two");
    assert!(multi_address_requests("wallet=named").is_empty());
    let from_values = multi_address_requests_from_values(&[
        "bc1qone,bc1qtwo".to_string(),
        "bc1qthree".to_string(),
    ]);
    assert_eq!(
        from_values
            .iter()
            .map(|call| call.path.as_str())
            .collect::<Vec<_>>(),
        vec!["/address/bc1qone", "/address/bc1qtwo", "/address/bc1qthree"]
    );
    assert!(from_values.iter().all(|call| call.method == "GET"));
    let prefixed = browser_fetch_url(ORIGIN, Network::Testnet3, &calls[0].path);
    assert!(prefixed.ends_with("/testnet/api/address/bc1qone"));
}

#[test]
fn test_transactions_use_post_txs_test() {
    let call = test_transactions_request();
    assert_eq!(call.method, "POST");
    assert_eq!(call.path, "/txs/test");
    let signer = RecordingSigner {
        seen: RefCell::new(None),
    };
    let mut seen_method = String::new();
    let mut seen_url = String::new();
    post_indexer(
        Some(&signer),
        ORIGIN,
        Network::Testnet4,
        &call.path,
        1_700_000_000,
        "ffff",
        &mut |method, url, _authorization, payload| {
            seen_method = method.to_string();
            seen_url = url.to_string();
            assert_eq!(payload, "ffff");
            Ok(Vec::new())
        },
    )
    .expect("test transactions");
    assert_eq!(seen_method, "POST");
    assert_eq!(
        seen_url,
        "https://splora.surmount.systems/testnet4/api/txs/test"
    );
    let unsigned = signer.seen.borrow().clone().expect("unsigned");
    assert!(
        unsigned
            .tags
            .iter()
            .any(|tag| { tag.len() == 2 && tag[0] == "method" && tag[1] == "POST" })
    );
}

#[test]
fn static_pages_do_not_fetch_the_indexer() {
    assert_eq!(
        static_page_routes(),
        [
            "/terms-of-service",
            "/privacy-policy",
            "/trademark-policy",
            "/docs",
            "/docs/faq",
            "/docs/api/rest",
            "/docs/api/:type",
        ]
    );
    for route in static_page_routes() {
        assert!(
            static_page_requests(route).is_empty(),
            "{route} fetched the indexer"
        );
    }
}

#[test]
fn faq_rest_and_api_type_do_not_call_the_indexer() {
    let routes = ["/docs/faq", "/docs/api/rest", "/docs/api/:type"];
    for route in routes {
        assert!(
            static_page_routes().contains(&route),
            "{route} is not a static page"
        );
        assert!(
            screen_routes().contains(&route),
            "{route} is not registered"
        );
        assert!(
            static_page_requests(route).is_empty(),
            "{route} fetched the indexer"
        );
    }

    let rest = docs_api_type_page("rest").expect("rest");
    assert_eq!(rest, "/docs/api/rest");
    assert!(
        static_page_requests(&rest).is_empty(),
        "api/rest fetched the indexer"
    );

    let websocket = docs_api_type_page("websocket").expect("websocket");
    assert_eq!(websocket, "/docs/api/websocket");
    assert!(
        static_page_requests(&websocket).is_empty(),
        "api/:type fetched the indexer for websocket"
    );
    assert!(static_page_requests("/docs/api/:type").is_empty());

    let tx_doc = docs_api_type_page("tx").expect("tx segment");
    assert_eq!(tx_doc, "/docs/api/tx");
    assert!(
        static_page_requests(&tx_doc).is_empty(),
        "api/:type proxied the tx segment to the indexer"
    );
    assert_ne!(tx_doc, "/tx");
    assert_ne!(tx_doc, "/api/tx");

    assert!(!screen_routes().contains(&"/api/:type"));
    assert!(!static_page_routes().contains(&"/api/:type"));
    assert!(screen_routes().contains(&"/block/:hash"));
    assert!(screen_routes().contains(&"/tx/:txid"));
    assert!(screen_routes().contains(&"/address/:addr"));
    assert_eq!(block_path("aa"), "/block/aa");
    assert_eq!(tx_path("bb"), "/tx/bb");
    assert_eq!(address_path("addr"), "/address/addr");

    assert!(docs_api_type_page("").is_none());
    assert!(docs_api_type_page("a/b").is_none());
    assert!(docs_api_type_page(":type").is_none());
    assert!(
        v1_client_requests()
            .iter()
            .all(|call| !call.path.starts_with("/docs/"))
    );
}

const BLOCK_HASH: &str = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
const BLOCK_PREV: &str = "cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc";
const TXID_A: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
const TXID_B: &str = "dddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddd";

fn electrs_paths(calls: &[splora_web::IndexerRequest]) -> Vec<&str> {
    calls.iter().map(|call| call.path.as_str()).collect()
}

fn assert_electrs_gets(calls: &[splora_web::IndexerRequest]) {
    assert!(calls.iter().all(|call| call.method == "GET"));
    for call in calls {
        assert!(!call.path.contains("/api/v1"), "{}", call.path);
        assert!(!call.path.contains("/v1/"), "{}", call.path);
        assert!(!call.path.contains("signet"), "{}", call.path);
        assert!(!call.path.contains("ws"), "{}", call.path);
        let url = browser_fetch_url(ORIGIN, Network::Mainnet, &call.path);
        assert!(url.starts_with("https://splora.surmount.systems/api"));
        assert!(url.ends_with(&call.path));
        assert!(!url.contains("/api/v1"));
    }
}

fn blocks_json() -> String {
    format!(
        r#"[{{"id":"{BLOCK_HASH}","height":842001,"timestamp":1700000000,"tx_count":17,"size":1234,"weight":5678,"previousblockhash":"{BLOCK_PREV}"}},{{"id":"{BLOCK_PREV}","height":842000,"timestamp":1699990000,"tx_count":3,"size":100,"weight":400,"previousblockhash":"{TXID_A}"}}]"#
    )
}

fn recent_json() -> String {
    format!(r#"[{{"txid":"{TXID_A}","fee":12345,"vsize":222,"value":99999}}]"#)
}

fn block_json() -> String {
    format!(
        r#"{{"id":"{BLOCK_HASH}","height":842001,"timestamp":1700000000,"tx_count":17,"size":1234,"weight":5678,"previousblockhash":"{BLOCK_PREV}"}}"#
    )
}

fn txs_json() -> String {
    format!(
        r#"[{{"txid":"{TXID_B}","fee":9,"size":200,"weight":800,"status":{{"confirmed":true,"block_height":842001,"block_hash":"{BLOCK_HASH}"}}}}]"#
    )
}

#[test]
fn dashboard_loads_blocks_and_recent_and_renders_fields() {
    let calls = dashboard_screen_requests();
    assert_eq!(electrs_paths(&calls), vec!["/blocks", "/mempool/recent"]);
    assert_electrs_gets(&calls);
    let recent_url = browser_fetch_url(ORIGIN, Network::Testnet4, "/mempool/recent");
    assert_eq!(
        recent_url,
        "https://splora.surmount.systems/testnet4/api/mempool/recent"
    );
    let text = dashboard_fields_text(&blocks_json(), &recent_json());
    assert!(text.contains("height 842001"), "{text}");
    assert!(text.contains("height 842000"), "{text}");
    assert!(text.contains("tx_count 17"), "{text}");
    assert!(text.contains(BLOCK_HASH), "{text}");
    assert!(text.contains(BLOCK_PREV), "{text}");
    assert!(
        text.contains(&format!("recent {TXID_A} fee 12345 vsize 222 value 99999")),
        "{text}"
    );
    assert!(!text.contains("\"height\""), "{text}");
    assert!(!text.contains("/api/v1"), "{text}");
    assert!(!text.contains("websocket"), "{text}");
    let src = include_str!("../src/views.rs");
    assert!(src.contains("dashboard_screen_requests"));
    assert!(src.contains("dashboard_fields_text"));
}

#[test]
fn blocks_list_loads_blocks_and_renders_fields() {
    let calls = blocks_list_requests();
    assert_eq!(electrs_paths(&calls), vec!["/blocks"]);
    assert_electrs_gets(&calls);
    assert!(screen_routes().contains(&"/blocks"));
    let text = blocks_list_fields_text(&blocks_json());
    assert!(text.contains("height 842001"), "{text}");
    assert!(text.contains("height 842000"), "{text}");
    assert!(text.contains("tx_count 17"), "{text}");
    assert!(text.contains("tx_count 3"), "{text}");
    assert!(text.contains(BLOCK_HASH), "{text}");
    assert!(!text.contains("/api/v1/blocks"), "{text}");
    assert!(!text.contains("\"tx_count\""), "{text}");
    let src = include_str!("../src/views.rs");
    assert!(src.contains("blocks_list_requests"));
    assert!(src.contains("blocks_list_fields_text"));
    assert!(src.contains("path!(\"blocks\")"));
}

#[test]
fn block_page_loads_block_and_txs_and_renders_fields() {
    let hash_plan = block_load_plan(BLOCK_HASH, None);
    assert_eq!(
        electrs_paths(&hash_plan),
        vec![
            format!("/block/{BLOCK_HASH}"),
            format!("/block/{BLOCK_HASH}/txs"),
        ]
    );
    assert_electrs_gets(&hash_plan);
    assert!(hash_plan.iter().all(|call| !call.path.contains("/txids")));
    assert!(hash_plan.iter().all(|call| !call.path.contains("/txs/")));

    let height_plan = block_load_plan("842001", None);
    assert_eq!(electrs_paths(&height_plan), vec!["/block-height/842001"]);
    assert_electrs_gets(&height_plan);
    assert!(block_load_plan("842001", Some("  nope  ")).is_empty());
    let follow = block_load_plan("842001", Some(&format!("  {BLOCK_HASH}\n")));
    assert_eq!(
        electrs_paths(&follow),
        vec![
            format!("/block/{BLOCK_HASH}"),
            format!("/block/{BLOCK_HASH}/txs"),
        ]
    );
    assert_electrs_gets(&follow);

    let text = block_fields_text(&block_json(), &txs_json());
    assert!(text.contains("height 842001"), "{text}");
    assert!(text.contains("tx_count 17"), "{text}");
    assert!(text.contains(&format!("previous {BLOCK_PREV}")), "{text}");
    assert!(
        text.contains(&format!("txid {TXID_B} confirmed true")),
        "{text}"
    );
    assert!(text.contains("fee 9"), "{text}");
    assert!(text.contains("block_height 842001"), "{text}");
    assert!(!text.contains("\"txid\""), "{text}");
    assert!(block_fields_text(splora_web::FAIL_CLOSED, "").contains("was not called"));
    let src = include_str!("../src/views.rs");
    assert!(src.contains("block_load_plan"));
    assert!(src.contains("block_fields_text"));
}

#[test]
fn transaction_page_loads_tx_and_renders_fields() {
    let calls = transaction_requests(TXID_A);
    assert_eq!(electrs_paths(&calls), vec![format!("/tx/{TXID_A}")]);
    assert_electrs_gets(&calls);
    let body = format!(
        r#"{{"txid":"{TXID_A}","version":2,"locktime":0,"size":111,"weight":444,"fee":9,"status":{{"confirmed":false}}}}"#
    );
    let text = transaction_fields_text(&body);
    assert!(
        text.contains(&format!("txid {TXID_A} confirmed false")),
        "{text}"
    );
    assert!(text.contains("fee 9"), "{text}");
    assert!(text.contains("size 111"), "{text}");
    assert!(text.contains("weight 444"), "{text}");
    assert!(!text.contains("status.confirmed"), "{text}");
    assert!(!text.contains("\"confirmed\""), "{text}");
    assert!(transaction_fields_text(splora_web::FAIL_CLOSED).contains("was not called"));
    let src = include_str!("../src/views.rs");
    assert!(src.contains("transaction_requests"));
    assert!(src.contains("transaction_fields_text"));
}

#[test]
fn address_page_loads_stats_txs_utxo_and_renders_fields() {
    let script = "bc1qexample";
    let calls = address_requests(script);
    assert_eq!(
        electrs_paths(&calls),
        vec![
            "/address/bc1qexample",
            "/address/bc1qexample/txs",
            "/address/bc1qexample/utxo",
        ]
    );
    assert_electrs_gets(&calls);
    let stats = r#"{"address":"bc1qexample","chain_stats":{"tx_count":4,"funded_txo_count":4,"spent_txo_count":1,"funded_txo_sum":5000,"spent_txo_sum":1000},"mempool_stats":{"tx_count":1,"funded_txo_count":1,"spent_txo_count":0,"funded_txo_sum":20,"spent_txo_sum":0}}"#;
    let txs = format!(
        r#"[{{"txid":"{TXID_A}","fee":7,"size":50,"weight":200,"status":{{"confirmed":false}}}}]"#
    );
    let utxo = format!(
        r#"[{{"txid":"{TXID_B}","vout":1,"value":42000,"status":{{"confirmed":true,"block_height":842001}}}}]"#
    );
    let text = address_fields_text(stats, &txs, &utxo);
    assert!(text.contains("address bc1qexample"), "{text}");
    assert!(text.contains("chain_tx_count 4"), "{text}");
    assert!(text.contains("funded_txo_sum 5000"), "{text}");
    assert!(text.contains("mempool_tx_count 1"), "{text}");
    assert!(
        text.contains(&format!("txid {TXID_A} confirmed false")),
        "{text}"
    );
    assert!(
        text.contains(&format!("utxo {TXID_B} vout 1 value 42000")),
        "{text}"
    );
    assert!(!text.contains("\"funded_txo_sum\""), "{text}");
    assert!(address_fields_text(splora_web::FAIL_CLOSED, "", "").contains("was not called"));
    let src = include_str!("../src/views.rs");
    assert!(src.contains("address_requests"));
    assert!(src.contains("address_fields_text"));
}

#[test]
fn multi_address_loads_each_address_and_renders_fields() {
    assert_eq!(
        entered_addresses(" bc1qone, bc1qtwo \n"),
        vec!["bc1qone".to_string(), "bc1qtwo".to_string()]
    );
    let calls = multi_address_screen_requests(&["bc1qone".to_string(), "bc1qtwo".to_string()]);
    assert_eq!(
        electrs_paths(&calls),
        vec!["/address/bc1qone", "/address/bc1qtwo"]
    );
    assert_electrs_gets(&calls);
    assert!(calls.iter().all(|call| !call.path.ends_with("/txs")));
    assert!(calls.iter().all(|call| !call.path.ends_with("/utxo")));
    assert!(calls.iter().all(|call| !call.path.contains("/wallet/")));
    let one = r#"{"address":"bc1qone","chain_stats":{"tx_count":2,"funded_txo_sum":5000},"mempool_stats":{"tx_count":0}}"#;
    let two = r#"{"address":"bc1qtwo","chain_stats":{"tx_count":1,"funded_txo_sum":77},"mempool_stats":{"tx_count":0}}"#;
    let text = multi_address_fields_text(&[
        ("/address/bc1qone".to_string(), one.to_string()),
        ("/address/bc1qtwo".to_string(), two.to_string()),
    ]);
    assert!(text.contains("address bc1qone"), "{text}");
    assert!(text.contains("address bc1qtwo"), "{text}");
    assert!(text.contains("funded_txo_sum 5000"), "{text}");
    assert!(text.contains("funded_txo_sum 77"), "{text}");
    assert!(!text.contains("\"chain_stats\""), "{text}");
    let src = include_str!("../src/views.rs");
    assert!(src.contains("multi_address_screen_requests"));
    assert!(src.contains("multi_address_fields_text"));
    assert!(src.contains("entered_addresses"));
}

#[test]
fn broadcast_posts_tx_and_renders_txid() {
    let call = broadcast_screen_request();
    assert_eq!(call.method, "POST");
    assert_eq!(call.path, "/tx");
    assert!(!call.path.contains("package"));
    assert!(!call.path.contains("/v1/"));
    assert!(!call.path.contains("signet"));
    let url = browser_fetch_url(ORIGIN, Network::Mutinynet, &call.path);
    assert_eq!(url, "https://splora.surmount.systems/mutinynet/api/tx");
    assert_eq!(broadcast_payload("  deadbeef\n"), "deadbeef");
    assert!(!broadcast_payload("deadbeef").starts_with('['));
    let text = broadcast_fields_text(TXID_A);
    assert_eq!(text, format!("txid {TXID_A}"));
    assert!(!text.contains("/tx"));
    assert!(!text.contains("package"));
    assert!(broadcast_fields_text(splora_web::FAIL_CLOSED).contains("was not called"));
    let src = include_str!("../src/views.rs");
    assert!(src.contains("broadcast_fields_text"));
    assert!(src.contains("broadcast_payload"));
    assert!(src.contains("broadcast_screen_request"));
}

#[test]
fn test_transactions_post_txs_test_and_render_results() {
    let call = test_transactions_screen_request();
    assert_eq!(call.method, "POST");
    assert_eq!(call.path, "/txs/test");
    assert!(!call.path.contains("package"));
    assert!(!call.path.contains("/v1/"));
    let url = browser_fetch_url(ORIGIN, Network::Liquid, &call.path);
    assert_eq!(url, "https://splora.surmount.systems/liquid/api/txs/test");
    assert_eq!(
        test_transactions_payload("abcd\neeff"),
        r#"["abcd","eeff"]"#
    );
    assert_eq!(
        test_transactions_payload(r#"["abcd","eeff"]"#),
        r#"["abcd","eeff"]"#
    );
    let body = format!(
        r#"[{{"txid":"{TXID_A}","wtxid":"{TXID_B}","allowed":true,"vsize":100}},{{"txid":"{TXID_B}","wtxid":"{TXID_A}","allowed":false,"reject-reason":"txn-already-in-mempool"}}]"#
    );
    let text = test_transactions_fields_text(&body);
    assert!(
        text.contains(&format!("txid {TXID_A} allowed true")),
        "{text}"
    );
    assert!(
        text.contains(&format!("txid {TXID_B} allowed false")),
        "{text}"
    );
    assert!(
        text.contains("reject-reason txn-already-in-mempool"),
        "{text}"
    );
    assert!(!text.contains("\"allowed\""), "{text}");
    assert!(!text.contains("/txs/package"), "{text}");
    assert!(test_transactions_fields_text(splora_web::FAIL_CLOSED).contains("was not called"));
    let src = include_str!("../src/views.rs");
    assert!(src.contains("test_transactions_screen_request"));
    assert!(src.contains("test_transactions_payload"));
    assert!(src.contains("test_transactions_fields_text"));
}

fn views_source() -> &'static str {
    include_str!("../src/views.rs")
}

/// Body of one view function, not the import list and not the next function.
fn view_fn_body(name: &str) -> &'static str {
    let src = views_source();
    let marker = format!("fn {name}(");
    let start = src
        .find(&marker)
        .unwrap_or_else(|| panic!("{name} is missing from views.rs"));
    let rest = &src[start..];
    let after_sig = rest.find('\n').unwrap_or(0) + 1;
    let tail = &rest[after_sig..];
    let end_in_tail = ["\nfn ", "\n#[component]", "\nconst "]
        .iter()
        .filter_map(|needle| tail.find(needle))
        .min();
    let end = after_sig + end_in_tail.unwrap_or(tail.len());
    &rest[..end]
}

fn code_has_call(body: &str, call: &str) -> bool {
    body.lines().any(|line| {
        let code = line.split("//").next().unwrap_or(line);
        code.contains(call)
    })
}

fn assert_view_slice(name: &str, next: &str) {
    let body = view_fn_body(name);
    assert!(
        body.starts_with(&format!("fn {name}(")),
        "{name} slice did not start at the view function"
    );
    assert!(
        !body.contains(&format!("fn {next}(")),
        "{name} slice included {next}"
    );
}

fn assert_view_calls(name: &str, calls: &[&str]) {
    let body = view_fn_body(name);
    let missing: Vec<&str> = calls
        .iter()
        .copied()
        .filter(|call| !code_has_call(body, call))
        .collect();
    assert!(missing.is_empty(), "{name} dropped {}", missing.join(", "));
}

fn assert_rendered(text: &str, needle: &str) {
    assert!(
        text.contains(needle),
        "rendered text dropped {needle}: {text}"
    );
}

fn const_literal(name: &str) -> &'static str {
    let src = views_source();
    let marker = format!("const {name}: &str = \"");
    let start = src
        .find(&marker)
        .unwrap_or_else(|| panic!("{name} is missing from views.rs"));
    let rest = &src[start + marker.len()..];
    let end = rest
        .find("\";")
        .unwrap_or_else(|| panic!("{name} is unclosed"));
    &rest[..end]
}

#[test]
fn view_guard_dashboard_keeps_blocks_and_recent_fields() {
    assert_view_slice("Dashboard", "BlocksPage");
    assert_view_calls(
        "Dashboard",
        &[
            "blocks_path(",
            "mempool_recent_path(",
            "dashboard_fields_text(",
        ],
    );
    let text = dashboard_fields_text(&blocks_json(), &recent_json());
    assert_rendered(&text, &format!("block {BLOCK_HASH}"));
    assert_rendered(&text, "height 842001");
    assert_rendered(&text, "tx_count 17");
    assert_rendered(&text, "timestamp 1700000000");
    assert_rendered(&text, "size 1234");
    assert_rendered(&text, "weight 5678");
    assert_rendered(&text, &format!("previous {BLOCK_PREV}"));
    assert_rendered(
        &text,
        &format!("recent {TXID_A} fee 12345 vsize 222 value 99999"),
    );
}

#[test]
fn view_guard_blocks_keeps_list_and_start_height_fields() {
    assert_view_slice("BlocksPage", "BlockPage");
    assert_view_calls(
        "BlocksPage",
        &[
            "blocks_path(",
            "blocks_start_height_path(",
            "blocks_list_fields_text(",
        ],
    );
    let text = blocks_list_fields_text(&blocks_json());
    assert_rendered(&text, &format!("block {BLOCK_HASH}"));
    assert_rendered(&text, "height 842000");
    assert_rendered(&text, "tx_count 3");
    assert_rendered(&text, "timestamp 1699990000");
    assert_rendered(&text, "size 100");
    assert_rendered(&text, "weight 400");
    assert_rendered(&text, &format!("previous {TXID_A}"));
}

#[test]
fn view_guard_block_keeps_block_txs_and_height_fields() {
    assert_view_slice("BlockPage", "TransactionPage");
    assert_view_calls(
        "BlockPage",
        &[
            "block_path(",
            "block_txs_path(",
            "block_height_path(",
            "block_fields_text(",
        ],
    );
    let text = block_fields_text(&block_json(), &txs_json());
    assert_rendered(&text, &format!("block {BLOCK_HASH}"));
    assert_rendered(&text, "height 842001");
    assert_rendered(&text, "tx_count 17");
    assert_rendered(&text, "timestamp 1700000000");
    assert_rendered(&text, "size 1234");
    assert_rendered(&text, "weight 5678");
    assert_rendered(&text, &format!("previous {BLOCK_PREV}"));
    assert_rendered(&text, &format!("txid {TXID_B} confirmed true"));
    assert_rendered(&text, "fee 9");
    assert_rendered(&text, "size 200");
    assert_rendered(&text, "weight 800");
    assert_rendered(&text, "block_height 842001");
}

#[test]
fn view_guard_transaction_keeps_tx_path_and_fields() {
    assert_view_slice("TransactionPage", "AddressPage");
    assert_view_calls("TransactionPage", &["tx_path(", "transaction_fields_text("]);
    let body = format!(
        r#"{{"txid":"{TXID_A}","version":2,"locktime":0,"size":111,"weight":444,"fee":9,"status":{{"confirmed":true,"block_height":842001}}}}"#
    );
    let text = transaction_fields_text(&body);
    assert_rendered(&text, &format!("txid {TXID_A} confirmed true"));
    assert_rendered(&text, "fee 9");
    assert_rendered(&text, "size 111");
    assert_rendered(&text, "weight 444");
    assert_rendered(&text, "block_height 842001");
}

#[test]
fn view_guard_address_keeps_stats_txs_utxo_fields() {
    assert_view_slice("AddressPage", "post_screen");
    assert_view_calls(
        "AddressPage",
        &[
            "address_path(",
            "address_txs_path(",
            "address_utxo_path(",
            "address_fields_text(",
        ],
    );
    let stats = r#"{"address":"bc1qexample","chain_stats":{"tx_count":4,"funded_txo_count":4,"spent_txo_count":1,"funded_txo_sum":5000,"spent_txo_sum":1000},"mempool_stats":{"tx_count":1,"funded_txo_count":1,"spent_txo_count":0,"funded_txo_sum":20,"spent_txo_sum":0}}"#;
    let txs = format!(
        r#"[{{"txid":"{TXID_A}","fee":7,"size":50,"weight":200,"status":{{"confirmed":true,"block_height":842001}}}}]"#
    );
    let utxo = format!(
        r#"[{{"txid":"{TXID_B}","vout":1,"value":42000,"status":{{"confirmed":true,"block_height":842001}}}}]"#
    );
    let text = address_fields_text(stats, &txs, &utxo);
    assert_rendered(&text, "address bc1qexample");
    assert_rendered(&text, "chain_tx_count 4");
    assert_rendered(&text, "funded_txo_sum 5000");
    assert_rendered(&text, "mempool_tx_count 1");
    assert_rendered(&text, &format!("txid {TXID_A} confirmed true"));
    assert_rendered(&text, "fee 7");
    assert_rendered(&text, "size 50");
    assert_rendered(&text, "weight 200");
    assert_rendered(&text, "block_height 842001");
    assert_rendered(
        &text,
        &format!("utxo {TXID_B} vout 1 value 42000 confirmed true"),
    );
}

#[test]
fn view_guard_multi_address_keeps_address_path_and_fields() {
    assert_view_slice("MultiAddressPage", "BroadcastPage");
    assert_view_calls(
        "MultiAddressPage",
        &[
            "address_path(",
            "multi_address_screen_requests(",
            "multi_address_fields_text(",
            "entered_addresses(",
        ],
    );
    let one = r#"{"address":"bc1qone","chain_stats":{"tx_count":2,"funded_txo_sum":5000},"mempool_stats":{"tx_count":0}}"#;
    let two = r#"{"address":"bc1qtwo","chain_stats":{"tx_count":1,"funded_txo_sum":77},"mempool_stats":{"tx_count":0}}"#;
    let text = multi_address_fields_text(&[
        ("/address/bc1qone".to_string(), one.to_string()),
        ("/address/bc1qtwo".to_string(), two.to_string()),
    ]);
    assert_rendered(&text, "address bc1qone");
    assert_rendered(&text, "chain_tx_count 2");
    assert_rendered(&text, "funded_txo_sum 5000");
    assert_rendered(&text, "mempool_tx_count 0");
    assert_rendered(&text, "address bc1qtwo");
    assert_rendered(&text, "funded_txo_sum 77");
}

#[test]
fn view_guard_broadcast_keeps_broadcast_path_and_txid() {
    assert_view_slice("BroadcastPage", "TestTransactionsPage");
    assert_view_calls(
        "BroadcastPage",
        &[
            "broadcast_path(",
            "broadcast_screen_request(",
            "broadcast_payload",
            "broadcast_fields_text",
        ],
    );
    assert_view_calls("post_screen", &["render("]);
    let text = broadcast_fields_text(TXID_A);
    assert_eq!(text, format!("txid {TXID_A}"));
}

#[test]
fn view_guard_test_transactions_keeps_test_txs_path_and_fields() {
    assert_view_slice("TestTransactionsPage", "TermsPage");
    assert_view_calls(
        "TestTransactionsPage",
        &[
            "test_txs_path(",
            "test_transactions_screen_request(",
            "test_transactions_payload",
            "test_transactions_fields_text",
        ],
    );
    assert_view_calls("post_screen", &["render("]);
    let body = format!(
        r#"[{{"txid":"{TXID_A}","wtxid":"{TXID_B}","allowed":true,"vsize":100}},{{"txid":"{TXID_B}","wtxid":"{TXID_A}","allowed":false,"reject-reason":"txn-already-in-mempool"}}]"#
    );
    let text = test_transactions_fields_text(&body);
    assert_rendered(&text, &format!("txid {TXID_A} allowed true"));
    assert_rendered(&text, &format!("txid {TXID_B} allowed false"));
    assert_rendered(&text, "reject-reason txn-already-in-mempool");
}

#[test]
fn view_guard_terms_keeps_compiled_copy() {
    assert_view_slice("TermsPage", "PrivacyPage");
    let body = view_fn_body("TermsPage");
    assert!(
        code_has_call(body, "This page is not financial, legal, or tax advice."),
        "TermsPage dropped the terms sentence"
    );
    assert!(!code_has_call(body, "signed_get("));
}

#[test]
fn view_guard_privacy_keeps_compiled_copy() {
    assert_view_slice("PrivacyPage", "TrademarkPage");
    let body = view_fn_body("PrivacyPage");
    assert!(
        code_has_call(body, "Splora does not ask for a secret key."),
        "PrivacyPage dropped the privacy sentence"
    );
    assert!(!code_has_call(body, "signed_get("));
}

#[test]
fn view_guard_trademark_keeps_compiled_copy() {
    assert_view_slice("TrademarkPage", "DocsPage");
    let body = view_fn_body("TrademarkPage");
    assert!(
        code_has_call(
            body,
            "Do not use those names to claim that Splora is endorsed by them."
        ),
        "TrademarkPage dropped the trademark sentence"
    );
    assert!(!code_has_call(body, "signed_get("));
}

#[test]
fn view_guard_docs_keeps_compiled_copy() {
    assert_view_slice("DocsPage", "DocsLayout");
    assert_view_slice("FaqPage", "RestApiPage");
    assert_view_slice("RestApiPage", "ApiTypePage");
    assert_view_slice("ApiTypePage", "MempoolPage");
    let docs = view_fn_body("DocsPage");
    assert!(
        code_has_call(docs, "They do not fetch the indexer for the page body."),
        "DocsPage dropped the docs sentence"
    );
    assert!(code_has_call(view_fn_body("FaqPage"), "FAQ_COPY"));
    assert!(
        const_literal("FAQ_COPY")
            .contains("This FAQ is compiled into the page. It does not call the indexer.")
    );
    assert!(code_has_call(view_fn_body("RestApiPage"), "REST_COPY"));
    assert!(
        const_literal("REST_COPY")
            .contains("This page is the reference. It does not call the indexer.")
    );
    assert!(code_has_call(
        view_fn_body("ApiTypePage"),
        "api_section_copy("
    ));
    assert!(code_has_call(
        view_fn_body("api_section_copy"),
        "WEBSOCKET_COPY"
    ));
    assert!(code_has_call(
        view_fn_body("api_section_copy"),
        "ELECTRS_COPY"
    ));
    assert!(const_literal("WEBSOCKET_COPY").contains("This page does not open that socket."));
    assert!(const_literal("ELECTRS_COPY").contains(
        "This page does not open an Electrum TCP connection, and it does not call the indexer."
    ));
    for name in [
        "DocsPage",
        "FaqPage",
        "RestApiPage",
        "ApiTypePage",
        "DocsLayout",
    ] {
        let body = view_fn_body(name);
        assert!(
            !code_has_call(body, "signed_get("),
            "{name} fetches the indexer"
        );
        assert!(
            !code_has_call(body, "schedule("),
            "{name} fetches the indexer"
        );
        assert!(
            !code_has_call(body, "WebSocket"),
            "{name} opens a websocket"
        );
        assert!(!code_has_call(body, ".want("), "{name} opens a websocket");
    }
}
