//! A person can walk the indexer from the Leptos page.
//!
//! Block ids open `/block/`. A height with no hash opens `/block-height/`.
//! Addresses open `/address/`. Broadcast and test txids open `/tx/`.
//! Older pages, spends, proofs, hex, status, mempool txids, package
//! broadcast, and a liquid asset use the indexer routes. An empty list is a
//! sentence. A failed call keeps the indexer's own sentence.

use splora_api::Network;
use splora_web::{
    ClientError, Nip98Signer, UnsignedNip98, address_chain_request, address_mempool_txs_request,
    address_rows_markup, asset_markup, asset_requests, block_rows_markup, block_status_markup,
    block_status_request, block_txs_older_href, blocks_older_href, blocks_rows_markup,
    broadcast_submission, broadcast_txid_markup, chain_tip_markup, dashboard_rows_markup,
    indexer_failure_text, mempool_txid_page_request, mempool_txids_markup, mempool_txids_request,
    merkle_proof_markup, multi_address_links_markup, outspends_markup, package_txid_markup,
    post_indexer, recent_rows_markup, resolved_block_hash, summary_rows_markup, test_txid_markup,
    tip_links_markup, transaction_detail_requests, transaction_hex_markup, transaction_requests,
    transaction_row_markup,
};

const BLOCK: &str = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
const PREV: &str = "cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc";
const TIP: &str = "dddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddd";
const TX: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
const SPEND: &str = "eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee";

fn block_link(hash: &str) -> String {
    format!("<a href=\"/block/{hash}\">{hash}</a>")
}

fn height_link(height: u64) -> String {
    format!("<a href=\"/block-height/{height}\">{height}</a>")
}

fn address_link(address: &str) -> String {
    format!("<a class=\"address\" href=\"/address/{address}\">{address}</a>")
}

fn tx_link(txid: &str) -> String {
    format!("<a href=\"/tx/{txid}\">{txid}</a>")
}

struct OkSigner;

impl Nip98Signer for OkSigner {
    fn public_key_hex(&self) -> Result<String, ClientError> {
        Ok("ab".repeat(32))
    }

    fn sign_event_json(&self, _unsigned: &UnsignedNip98) -> Result<String, ClientError> {
        Ok(r#"{"sig":"abcd"}"#.to_string())
    }
}

#[test]
fn block_id_height_and_previous_open_a_block() {
    let body = format!(
        r#"[{{"id":"{BLOCK}","height":842001,"version":1,"timestamp":1,"tx_count":2,"size":3,"weight":4,"merkle_root":"merkle","previousblockhash":"{PREV}","mediantime":1,"nonce":1,"bits":1,"difficulty":1}},{{"id":"{PREV}","height":10,"version":1,"timestamp":1,"tx_count":1,"size":1,"weight":1,"merkle_root":"merkle","previousblockhash":null,"mediantime":1,"nonce":1,"bits":1,"difficulty":1}}]"#
    );
    let markup = blocks_rows_markup(&body);
    assert!(
        markup.contains(&block_link(BLOCK)),
        "block id is not a block link: {markup}"
    );
    assert!(
        markup.contains(&block_link(PREV)),
        "previous hash is not a block link: {markup}"
    );
    assert!(
        markup.contains(&format!("<a href=\"/block/{BLOCK}\">842001</a>")),
        "height with a known hash must open that block: {markup}"
    );
    assert!(
        markup.contains(&format!("<a href=\"/block/{PREV}\">10</a>")),
        "second height must open its block: {markup}"
    );
    assert!(
        !markup.contains("/block-height/842001"),
        "a known block hash must not be demoted to a height-only route: {markup}"
    );
}

#[test]
fn confirming_hash_opens_the_block_and_a_lone_height_opens_block_height() {
    let tx = format!(
        r#"{{"txid":"{TX}","version":1,"locktime":0,"vin":[],"vout":[],"size":1,"weight":4,"sigops":0,"fee":1,"status":{{"confirmed":true,"block_height":842001,"block_hash":"{BLOCK}","block_time":9}}}}"#
    );
    let markup = transaction_row_markup(&tx);
    assert!(
        markup.contains(&block_link(BLOCK)),
        "confirming hash is not a block link: {markup}"
    );
    assert!(
        markup.contains(&format!("<a href=\"/block/{BLOCK}\">842001</a>")),
        "confirming height must use the known hash: {markup}"
    );
    assert!(
        !markup.contains("0014"),
        "script hex leaked onto the row: {markup}"
    );

    let lone = format!(
        r#"[{{"txid":"{TX}","vout":1,"value":5,"status":{{"confirmed":true,"block_height":42}}}}]"#
    );
    let address = address_rows_markup(
        r#"{"address":"bc1qexample","chain_stats":{"tx_count":0,"funded_txo_count":0,"spent_txo_count":0,"funded_txo_sum":0,"spent_txo_sum":0},"mempool_stats":{"tx_count":0,"funded_txo_count":0,"spent_txo_count":0,"funded_txo_sum":0,"spent_txo_sum":0}}"#,
        "[]",
        &lone,
    );
    assert!(
        address.contains(&height_link(42)),
        "a utxo height with no hash must open /block-height/: {address}"
    );
    assert!(
        !address.contains(&format!("/block/42")),
        "a lone height must not pretend to be a block hash: {address}"
    );
}

#[test]
fn input_output_and_multi_address_open_the_address() {
    let tx = format!(
        r#"{{"txid":"{TX}","version":1,"locktime":0,"vin":[{{"txid":"prev","vout":0,"prevout":{{"scriptpubkey":"0014aa","scriptpubkey_asm":"OP_0","scriptpubkey_type":"v0_p2wpkh","scriptpubkey_address":"bc1qinput","value":3}},"scriptsig":"","scriptsig_asm":"","is_coinbase":false,"sequence":1}}],"vout":[{{"scriptpubkey":"76a914bb88ac","scriptpubkey_asm":"OP_DUP","scriptpubkey_type":"p2pkh","scriptpubkey_address":"bc1qoutput","value":2}}],"size":10,"weight":40,"sigops":0,"fee":1,"status":{{"confirmed":false}}}}"#
    );
    let markup = transaction_row_markup(&tx);
    assert!(markup.contains(&address_link("bc1qinput")), "{markup}");
    assert!(markup.contains(&address_link("bc1qoutput")), "{markup}");
    assert!(!markup.contains("0014aa"), "{markup}");
    assert!(!markup.contains("76a914bb88ac"), "{markup}");
    assert!(!markup.contains("witness"), "{markup}");

    let links = multi_address_links_markup(&[
        (
            "/address/bc1qone".to_string(),
            r#"{"address":"bc1qone","chain_stats":{"tx_count":1,"funded_txo_count":0,"spent_txo_count":0,"funded_txo_sum":1,"spent_txo_sum":0},"mempool_stats":{"tx_count":0,"funded_txo_count":0,"spent_txo_count":0,"funded_txo_sum":0,"spent_txo_sum":0}}"#.to_string(),
        ),
        (
            "/address/bc1qtwo/utxo".to_string(),
            "[]".to_string(),
        ),
    ]);
    assert!(links.contains(&address_link("bc1qone")), "{links}");
    assert!(
        !links.contains("bc1qtwo"),
        "a utxo path is not an address result: {links}"
    );
}

#[test]
fn broadcast_and_test_txids_open_the_transaction() {
    let one = broadcast_txid_markup(TX);
    assert!(one.contains(&tx_link(TX)), "{one}");
    let tests = test_txid_markup(&format!(
        r#"[{{"txid":"{TX}","allowed":true}},{{"txid":"{SPEND}","allowed":false}}]"#
    ));
    assert!(tests.contains(&tx_link(TX)), "{tests}");
    assert!(tests.contains(&tx_link(SPEND)), "{tests}");
}

#[test]
fn older_blocks_and_block_transactions_use_the_paged_gets() {
    let blocks = format!(
        r#"[{{"id":"{BLOCK}","height":100,"version":1,"timestamp":1,"tx_count":1,"size":1,"weight":4,"merkle_root":"m","previousblockhash":"{PREV}","mediantime":1,"nonce":1,"bits":1,"difficulty":1}},{{"id":"{PREV}","height":90,"version":1,"timestamp":1,"tx_count":1,"size":1,"weight":4,"merkle_root":"m","mediantime":1,"nonce":1,"bits":1,"difficulty":1}}]"#
    );
    assert_eq!(
        blocks_older_href(&blocks).as_deref(),
        Some("/blocks?start_height=89")
    );
    assert_eq!(blocks_older_href("[]"), None);
    assert_eq!(
        block_txs_older_href(BLOCK, 0).as_deref(),
        Some(format!("/block/{BLOCK}?start_index=25").as_str())
    );
    assert_eq!(
        block_txs_older_href(BLOCK, 25).as_deref(),
        Some(format!("/block/{BLOCK}?start_index=50").as_str())
    );
    assert_eq!(block_txs_older_href(BLOCK, 10), None);
    let src = include_str!("../src/views.rs");
    assert!(
        src.contains("blocks_older_href"),
        "the blocks page has no control that sets start_height"
    );
    assert!(
        src.contains("block_txs_older_href"),
        "the block page has no control for the next transaction page"
    );
}

#[test]
fn address_history_and_mempool_use_chain_and_mempool_gets() {
    let chain = address_chain_request("bc1qexample", TX);
    assert_eq!(chain.method, "GET");
    assert_eq!(chain.path, format!("/address/bc1qexample/txs/chain/{TX}"));
    let mempool = address_mempool_txs_request("bc1qexample");
    assert_eq!(mempool.method, "GET");
    assert_eq!(mempool.path, "/address/bc1qexample/txs/mempool");
    let summary =
        format!(r#"[{{"txid":"{TX}","height":90,"value":15,"time":1700000000,"tx_position":3}}]"#);
    let rows = summary_rows_markup(&summary);
    assert!(rows.contains(&tx_link(TX)), "{rows}");
    assert!(rows.contains("90"), "{rows}");
    assert!(rows.contains("15"), "{rows}");
    assert!(rows.contains("1700000000"), "{rows}");
    assert!(rows.contains("3"), "{rows}");
    assert_eq!(
        summary_rows_markup("[]"),
        "<p class=\"empty\">There are no transactions.</p>"
    );
    let src = include_str!("../src/views.rs");
    assert!(src.contains("address_chain_request"));
    assert!(src.contains("address_mempool_txs_request"));
}

#[test]
fn mempool_txids_past_the_recent_queue_link_to_the_transaction() {
    let first = mempool_txids_request();
    assert_eq!(first.method, "GET");
    assert_eq!(first.path, "/mempool/txids");
    let page = mempool_txid_page_request(TX);
    assert_eq!(page.method, "GET");
    assert_eq!(page.path, format!("/mempool/txids/page/{TX}"));
    let markup = mempool_txids_markup(&format!(r#"["{TX}","{SPEND}"]"#));
    assert!(markup.contains(&tx_link(TX)), "{markup}");
    assert!(markup.contains(&tx_link(SPEND)), "{markup}");
    assert_eq!(
        mempool_txids_markup("[]"),
        "<p class=\"empty\">There are no transactions.</p>"
    );
    let src = include_str!("../src/views.rs");
    assert!(src.contains("mempool_txids_request"));
    assert!(src.contains("mempool_txid_page_request"));
}

#[test]
fn transaction_screen_shows_spends_proof_and_hex_off_the_row() {
    let calls = transaction_requests(TX);
    assert_eq!(
        calls.len(),
        1,
        "the original transaction get stays one call"
    );
    assert_eq!(calls[0].path, format!("/tx/{TX}"));
    let detail = transaction_detail_requests(TX);
    let paths: Vec<&str> = detail.iter().map(|call| call.path.as_str()).collect();
    assert!(paths.contains(&format!("/tx/{TX}/outspends").as_str()));
    assert!(paths.contains(&format!("/tx/{TX}/merkle-proof").as_str()));
    assert!(paths.contains(&format!("/tx/{TX}/hex").as_str()));
    assert!(detail.iter().all(|call| call.method == "GET"));

    let spends = format!(
        r#"[{{"spent":false}},{{"spent":true,"txid":"{SPEND}","vin":1,"status":{{"confirmed":true,"block_height":8,"block_hash":"{BLOCK}"}}}}]"#
    );
    let spend_html = outspends_markup(&spends);
    assert!(
        spend_html.contains("spent false") || spend_html.contains("not spent"),
        "{spend_html}"
    );
    assert!(spend_html.contains(&tx_link(SPEND)), "{spend_html}");
    assert!(!spend_html.contains("scriptpubkey"), "{spend_html}");

    let proof = format!(r#"{{"block_height":8,"merkle":["{TX}"],"pos":2}}"#);
    let proof_html = merkle_proof_markup(&proof);
    assert!(proof_html.contains(&height_link(8)), "{proof_html}");
    assert!(proof_html.contains(TX), "{proof_html}");
    assert!(proof_html.contains("2"), "{proof_html}");

    let hex_body = "deadbeef";
    let hex_html = transaction_hex_markup(hex_body);
    assert!(hex_html.contains(hex_body), "{hex_html}");
    assert!(hex_html.contains("tx-hex"), "{hex_html}");
    let row = transaction_row_markup(&format!(
        r#"{{"txid":"{TX}","version":1,"locktime":0,"vin":[],"vout":[],"size":1,"weight":4,"sigops":0,"fee":1,"status":{{"confirmed":false}}}}"#
    ));
    assert!(
        !row.contains("tx-hex"),
        "transaction hex must stay off the row: {row}"
    );
    assert!(!row.contains(hex_body), "{row}");
    let src = include_str!("../src/views.rs");
    assert!(src.contains("transaction_detail_requests"));
    assert!(src.contains("outspends_markup"));
    assert!(src.contains("merkle_proof_markup"));
    assert!(src.contains("transaction_hex_markup"));
}

#[test]
fn block_status_links_next_best_when_present() {
    let call = block_status_request(BLOCK);
    assert_eq!(call.method, "GET");
    assert_eq!(call.path, format!("/block/{BLOCK}/status"));
    let body = format!(r#"{{"in_best_chain":true,"height":100,"next_best":"{TIP}"}}"#);
    let markup = block_status_markup(&body);
    assert!(markup.contains("true"), "{markup}");
    assert!(markup.contains(&block_link(TIP)), "{markup}");
    let orphan = r#"{"in_best_chain":false,"height":null,"next_best":null}"#;
    let orphan_html = block_status_markup(orphan);
    assert!(orphan_html.contains("false"), "{orphan_html}");
    assert!(!orphan_html.contains("href=\"/block/\""), "{orphan_html}");
    let src = include_str!("../src/views.rs");
    assert!(src.contains("block_status_request"));
}

#[test]
fn empty_lists_are_sentences_and_failures_keep_the_indexer_sentence() {
    assert_eq!(
        blocks_rows_markup("[]"),
        "<p class=\"empty\">There are no blocks.</p>"
    );
    assert_eq!(
        dashboard_rows_markup("[]", "[]"),
        "<p class=\"empty\">There are no blocks.</p><p class=\"empty\">There are no transactions.</p>"
    );
    assert!(!blocks_rows_markup("not-json").contains("<tr"));
    let text = indexer_failure_text("GET", "/block/missing", "Block not found");
    assert!(text.contains("Block not found"), "{text}");
    assert!(
        text.contains("indexer error GET /block/missing"),
        "the method and path may stay, but not as the only sentence: {text}"
    );
    let tx = indexer_failure_text("GET", &format!("/tx/{TX}"), "Transaction not found");
    assert!(tx.contains("Transaction not found"), "{tx}");
    assert!(tx.contains("indexer error GET /tx/"), "{tx}");
    let src = include_str!("../src/views.rs");
    assert!(
        src.contains("indexer_failure_text"),
        "the page still drops the indexer sentence"
    );
}

#[test]
fn more_than_one_raw_transaction_uses_package_broadcast() {
    let one = broadcast_submission("  deadbeef\n");
    assert_eq!(one.method, "POST");
    assert_eq!(one.path, "/tx");
    assert_eq!(one.body, "deadbeef");
    let many = broadcast_submission("aaaa\nbbbb");
    assert_eq!(many.method, "POST");
    assert_eq!(many.path, "/txs/package");
    assert_eq!(many.body, r#"["aaaa","bbbb"]"#);
    assert!(!many.path.contains("replacement"));
    let package = package_txid_markup(&format!(
        r#"{{"package_msg":"success","tx-results":{{"{TX}":{{"txid":"{TX}"}}}}}}"#
    ));
    assert!(package.contains(&tx_link(TX)), "{package}");
    assert!(package.contains("success"), "{package}");
    let signer = OkSigner;
    let mut called = false;
    let rejected = post_indexer(
        Some(&signer),
        "https://splora.surmount.systems",
        Network::Mainnet,
        "/txs/package",
        1,
        "abcd",
        &mut |_, _, _, _| {
            called = true;
            Ok(Vec::new())
        },
    );
    assert!(matches!(rejected, Err(ClientError::RejectedEvent)));
    assert!(!called, "one hex is not a package");
    let mut seen = String::new();
    let allowed = post_indexer(
        Some(&signer),
        "https://splora.surmount.systems",
        Network::Mainnet,
        "/txs/package",
        1,
        &many.body,
        &mut |_method, url, _authorization, payload| {
            seen = format!("{url} {payload}");
            Ok(b"ok".to_vec())
        },
    );
    assert!(allowed.is_ok(), "{allowed:?}");
    assert!(seen.contains("/api/txs/package"), "{seen}");
    assert!(seen.contains(&many.body), "{seen}");
    let src = include_str!("../src/views.rs");
    assert!(src.contains("broadcast_submission"));
    assert!(!src.contains("/api/v1/replacements"));
}

#[test]
fn liquid_asset_shows_name_ticker_and_transactions_and_mainnet_does_not() {
    let id = "aa".repeat(32);
    assert!(asset_requests(false, &id).is_empty());
    let calls = asset_requests(true, &id);
    assert_eq!(calls[0].path, format!("/asset/{id}"));
    assert_eq!(calls[1].path, format!("/asset/{id}/txs"));
    assert!(calls.iter().all(|call| call.method == "GET"));
    let asset = r#"{"asset_id":"aa","name":"Tether USD","ticker":"USDt"}"#;
    let txs = format!(
        r#"[{{"txid":"{TX}","version":1,"locktime":0,"vin":[],"vout":[],"size":1,"weight":4,"sigops":0,"fee":1,"status":{{"confirmed":false}}}}]"#
    );
    let painted = asset_markup(asset, &txs, true);
    assert!(painted.contains("Tether USD"), "{painted}");
    assert!(painted.contains("USDt"), "{painted}");
    assert!(painted.contains(&tx_link(TX)), "{painted}");
    let hidden = asset_markup(asset, &txs, false);
    assert!(!hidden.contains("Tether USD"), "{hidden}");
    assert!(!hidden.contains("USDt"), "{hidden}");
    assert!(!hidden.contains(TX), "{hidden}");
    let missing = asset_markup("endpoint does not exist /asset/aa", "", true);
    assert!(
        missing.contains("not-served") || missing.contains("endpoint does not exist"),
        "{missing}"
    );
    let src = include_str!("../src/views.rs");
    assert!(src.contains("asset_requests"));
    assert!(src.contains("Network::Liquid"));
}

#[test]
fn chain_tip_opens_the_block_or_the_height() {
    let markup = tip_links_markup(&format!(" {TIP}\n"), " 800000 ");
    assert!(markup.contains(&block_link(TIP)), "{markup}");
    assert!(markup.contains(&height_link(800_000)), "{markup}");
    let src = include_str!("../src/views.rs");
    assert!(src.contains("tip_links_markup"));
}

/// GPUI paints the exact line `tip height: 1 hash: liqtip` from
/// `GET /blocks/tip/height` and `GET /blocks/tip/hash`. Neither route has a
/// query string. The block link and the height link stay.
#[test]
fn dashboard_paints_tip_height_and_hash_line() {
    const LINE: &str = "tip height: 1 hash: liqtip";
    let markup = tip_links_markup("liqtip", "1");
    assert_eq!(markup.matches(LINE).count(), 1, "{markup}");
}

#[test]
fn known_tip_hash_opens_the_height_on_that_block() {
    let markup = chain_tip_markup(&format!(" {TIP}\n"), " 800000 ");
    assert!(markup.contains(&block_link(TIP)), "{markup}");
    assert!(
        markup.contains(&format!("<a href=\"/block/{TIP}\">800000</a>")),
        "{markup}"
    );
    assert!(!markup.contains("/block-height/800000"), "{markup}");
    let only_height = chain_tip_markup("", "800000");
    assert!(only_height.contains(&height_link(800_000)), "{only_height}");
    assert!(!only_height.contains("/block/"), "{only_height}");
    let src = include_str!("../src/views.rs");
    assert!(src.contains("chain_tip_markup"));
    assert!(src.contains("tip_links_markup"));
}

#[test]
fn height_route_older_transactions_use_the_block_hash() {
    let hash = resolved_block_hash("100", &format!(" {BLOCK}\n"), "").expect("height body");
    assert_eq!(
        block_txs_older_href(&hash, 0).as_deref(),
        Some(format!("/block/{BLOCK}?start_index=25").as_str())
    );
    assert_eq!(resolved_block_hash("100", "", ""), None);
    let block = format!(
        r#"{{"id":"{BLOCK}","height":100,"version":1,"timestamp":1,"tx_count":1,"size":1,"weight":4,"merkle_root":"m","mediantime":1,"nonce":1,"bits":1,"difficulty":1}}"#
    );
    let from_block = resolved_block_hash("100", "Block not found", &block).expect("block id");
    assert_eq!(from_block, BLOCK);
    assert_eq!(resolved_block_hash(BLOCK, "", "").as_deref(), Some(BLOCK));
    let src = include_str!("../src/views.rs");
    assert!(src.contains("resolved_block_hash"));
    assert!(src.contains("block_txs_older_href"));
}

#[test]
fn empty_address_lists_and_recent_mempool_are_sentences() {
    assert_eq!(
        address_rows_markup("", "[]", ""),
        "<p class=\"empty\">There are no transactions.</p><p class=\"address-tx-empty\">The address transaction list is empty.</p>"
    );
    assert_eq!(
        address_rows_markup("", "", "[]"),
        "<p class=\"empty\">There are no unspent outputs.</p><p class=\"address-utxo-empty\">The utxo list is empty.</p>"
    );
    assert_eq!(
        address_rows_markup("", "[]", "[]"),
        "<p class=\"empty\">There are no transactions.</p><p class=\"address-tx-empty\">The address transaction list is empty.</p><p class=\"empty\">There are no unspent outputs.</p><p class=\"address-utxo-empty\">The utxo list is empty.</p>"
    );
    assert_eq!(
        recent_rows_markup("[]"),
        "<p class=\"empty\">There are no transactions.</p>"
    );
    assert_eq!(address_rows_markup("", "", ""), "");
    assert_eq!(recent_rows_markup(""), "");
    assert!(!address_rows_markup("not-json", "not-json", "not-json").contains("There are no"));
    assert!(!recent_rows_markup("not-json").contains("There are no"));
}

#[test]
fn block_rows_still_do_not_put_transaction_hex_on_the_row() {
    let block = format!(
        r#"{{"id":"{BLOCK}","height":1,"version":1,"timestamp":1,"tx_count":0,"size":1,"weight":4,"merkle_root":"m","mediantime":1,"nonce":1,"bits":1,"difficulty":1}}"#
    );
    let markup = block_rows_markup(&block, "[]");
    assert!(markup.contains(&block_link(BLOCK)), "{markup}");
    assert!(
        markup.contains("<p class=\"empty\">There are no transactions.</p>"),
        "{markup}"
    );
    assert!(!markup.contains("tx-hex"), "{markup}");
}
