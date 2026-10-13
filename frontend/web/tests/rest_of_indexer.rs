//! Compact history, batch spends, block and transaction side bodies, and liquid registry.
//!
//! A 64-digit id stays a height. A failed broadcast is not an `/tx/` link.
//! Whitespace inside `[]` is an empty list. A failed hex body is a notice.

use splora_web::{
    BatchPost, address_requests, address_summary_page_request, address_summary_request,
    asset_confirmed_page_request, asset_confirmed_txs_request, asset_history_markup,
    asset_mempool_txs_request, asset_supply_decimal_request, asset_supply_request,
    batch_outspends_markup, block_header_request, block_load_plan, block_raw_request,
    block_txid_index_markup, block_txid_index_request, broadcast_fields_text, indexed_bytes_markup,
    liquid_asset_requests, multi_address_summary_post, multi_address_txs_post, one_outspend_markup,
    plan_search, plan_search_on, registry_entry_markup, registry_entry_request,
    registry_list_markup, registry_list_request, registry_search_markup, registry_search_request,
    search_loaded_markup, submission_links, summary_rows_markup, summary_rows_with_status,
    supply_markup, transaction_row_markup, tx_merkleblock_proof_request, tx_outspend_index_request,
    tx_raw_request, tx_status_markup, tx_status_request, txs_outspends_request,
};

const TX: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
const SPEND: &str = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
const HEX: &str = "abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789";

fn tx_link(txid: &str) -> String {
    format!("<a href=\"/tx/{txid}\">{txid}</a>")
}

fn summary_json() -> String {
    format!(r#"[{{"txid":"{TX}","height":90,"value":-15,"time":1700000000,"tx_position":3}}]"#)
}

#[test]
fn address_summary_links_txid_height_value_time_and_position() {
    let script = "bc1qexample";
    let call = address_summary_request(script);
    assert_eq!(call.method, "GET");
    assert_eq!(call.path, "/address/bc1qexample/txs/summary");
    let page = address_summary_page_request(script, TX);
    assert_eq!(page.method, "GET");
    assert_eq!(page.path, format!("/address/bc1qexample/txs/summary/{TX}"));

    let rows = summary_rows_markup(&summary_json());
    assert!(rows.contains(&tx_link(TX)), "{rows}");
    assert!(rows.contains(">90<"), "{rows}");
    assert!(rows.contains("-15"), "{rows}");
    assert!(rows.contains("1700000000"), "{rows}");
    assert!(rows.contains(">3<"), "{rows}");
    assert!(rows.contains("summary-row"), "{rows}");

    let confirmed = format!(
        r#"{{"confirmed":true,"block_height":90,"block_hash":"{HEX}","block_time":1700000000}}"#
    );
    let with_status = summary_rows_with_status(&summary_json(), &[(TX.to_string(), confirmed)]);
    assert!(with_status.contains("confirmed true"), "{with_status}");
    assert!(with_status.contains(&tx_link(TX)), "{with_status}");
    assert!(
        with_status.contains(&format!("/block/{HEX}")),
        "{with_status}"
    );

    let failed = "Transaction not found\nindexer error GET /tx/abc/status";
    let noticed =
        summary_rows_with_status(&summary_json(), &[(TX.to_string(), failed.to_string())]);
    assert!(noticed.contains("Transaction not found"), "{noticed}");
    assert!(!noticed.contains("confirmed true"), "{noticed}");
    assert!(tx_status_request(TX).path == format!("/tx/{TX}/status"));
}

#[test]
fn scripthash_summary_uses_the_scripthash_paths() {
    let call = address_summary_request(HEX);
    assert_eq!(call.path, format!("/scripthash/{HEX}/txs/summary"));
    let page = address_summary_page_request(HEX, TX);
    assert_eq!(page.path, format!("/scripthash/{HEX}/txs/summary/{TX}"));
    let rows = summary_rows_markup(&summary_json());
    assert!(rows.contains(&tx_link(TX)), "{rows}");
}

#[test]
fn opened_scripthash_loads_txs_utxos_and_mempool() {
    let calls = address_requests(HEX);
    let paths: Vec<&str> = calls.iter().map(|call| call.path.as_str()).collect();
    assert_eq!(
        paths,
        vec![
            format!("/scripthash/{HEX}"),
            format!("/scripthash/{HEX}/txs"),
            format!("/scripthash/{HEX}/utxo"),
            format!("/scripthash/{HEX}/txs/mempool"),
        ]
    );
    assert!(calls.iter().all(|call| call.method == "GET"));
    assert!(calls.iter().all(|call| !call.path.starts_with("/address/")));

    let address = address_requests("bc1qexample");
    assert_eq!(
        address
            .iter()
            .map(|call| call.path.as_str())
            .collect::<Vec<_>>(),
        vec![
            "/address/bc1qexample",
            "/address/bc1qexample/txs",
            "/address/bc1qexample/utxo",
        ]
    );
}

#[test]
fn multi_address_posts_one_summary_or_one_tx_list() {
    let values = vec!["bc1qone".to_string(), "bc1qtwo".to_string()];
    let summary = multi_address_summary_post(&values);
    assert_eq!(summary.len(), 1, "{summary:?}");
    assert_eq!(
        summary[0],
        BatchPost {
            method: "POST",
            path: "/addresses/txs/summary",
            body: r#"["bc1qone","bc1qtwo"]"#.to_string(),
        }
    );
    let txs = multi_address_txs_post(&values);
    assert_eq!(txs.len(), 1);
    assert_eq!(txs[0].method, "POST");
    assert_eq!(txs[0].path, "/addresses/txs");
    assert_eq!(txs[0].body, r#"["bc1qone","bc1qtwo"]"#);

    let scripts = vec![HEX.to_string(), TX.to_string()];
    let script_summary = multi_address_summary_post(&scripts);
    assert_eq!(script_summary.len(), 1);
    assert_eq!(script_summary[0].path, "/scripthashes/txs/summary");
    assert!(script_summary[0].body.contains(HEX), "{script_summary:?}");
    let script_txs = multi_address_txs_post(&scripts);
    assert_eq!(script_txs[0].path, "/scripthashes/txs");

    let rows = summary_rows_markup(&summary_json());
    assert!(rows.contains(&tx_link(TX)), "{rows}");
    assert!(multi_address_summary_post(&[]).is_empty());
}

#[test]
fn batch_and_one_output_spends_link_the_spending_txid() {
    let mut ids: Vec<String> = (0..51).map(|index| format!("{TX}{index:02}")).collect();
    ids[0] = TX.to_string();
    let call = txs_outspends_request(&ids).expect("batch");
    assert_eq!(call.method, "GET");
    assert!(call.path.starts_with("/txs/outspends?txids="), "{call:?}");
    assert!(call.path.contains(TX), "{call:?}");
    assert!(
        !call.path.contains(&ids[50]),
        "the 51st id is past the cap of 50"
    );
    assert_eq!(call.path.matches(',').count(), 49);
    assert!(txs_outspends_request(&[]).is_none());

    let body = format!(r#"[[{{"spent":true,"txid":"{SPEND}","vin":1}}],[{{"spent":false}}]]"#);
    let html = batch_outspends_markup(&body);
    assert!(html.contains(&tx_link(SPEND)), "{html}");
    assert!(html.contains("spent false"), "{html}");

    let one = tx_outspend_index_request(TX, 2);
    assert_eq!(one.method, "GET");
    assert_eq!(one.path, format!("/tx/{TX}/outspend/2"));
    let one_body = format!(r#"{{"spent":true,"txid":"{SPEND}","vin":0}}"#);
    let one_html = one_outspend_markup(&one_body);
    assert!(one_html.contains(&tx_link(SPEND)), "{one_html}");

    let failed = "No txids specified";
    let notice = batch_outspends_markup(failed);
    assert!(notice.contains("No txids specified"), "{notice}");
    assert!(!notice.contains("<pre"), "{notice}");
}

#[test]
fn block_txid_index_opens_the_transaction() {
    let call = block_txid_index_request(HEX, 4);
    assert_eq!(call.method, "GET");
    assert_eq!(call.path, format!("/block/{HEX}/txid/4"));
    let html = block_txid_index_markup(&format!("  {TX}\n"));
    assert_eq!(html, tx_link(TX));
    let missing = block_txid_index_markup("tx index out of range");
    assert!(missing.contains("tx index out of range"), "{missing}");
    assert!(!missing.contains("/tx/"), "{missing}");
    assert!(!missing.contains("<pre"), "{missing}");
}

#[test]
fn side_bodies_sit_beside_fields_and_a_failed_body_is_a_notice() {
    assert_eq!(
        block_header_request(HEX).path,
        format!("/block/{HEX}/header")
    );
    assert_eq!(block_raw_request(HEX).path, format!("/block/{HEX}/raw"));
    assert_eq!(tx_raw_request(TX).path, format!("/tx/{TX}/raw"));
    assert_eq!(
        tx_merkleblock_proof_request(TX).path,
        format!("/tx/{TX}/merkleblock-proof")
    );
    assert_eq!(
        tx_status_markup(r#"{"confirmed":false}"#).contains("confirmed false"),
        true
    );

    let header = indexed_bytes_markup("  aa55\n", "block-header");
    assert_eq!(header, "<pre class=\"block-header\">aa55</pre>");
    let failed = "Block not found\nindexer error GET /block/abc/header";
    let notice = indexed_bytes_markup(failed, "block-header");
    assert!(notice.contains("Block not found"), "{notice}");
    assert!(notice.contains("<p class=\"notice\">"), "{notice}");
    assert!(!notice.contains("<pre"), "{notice}");
    assert!(!notice.contains("block-header"), "{notice}");

    let raw_fail = indexed_bytes_markup("endpoint does not exist /block/abc/raw", "block-raw");
    assert!(raw_fail.contains("endpoint does not exist"), "{raw_fail}");
    assert!(!raw_fail.contains("<pre"), "{raw_fail}");
    let proof_fail = indexed_bytes_markup("not-served", "merkleblock-proof");
    assert!(proof_fail.contains("not-served"), "{proof_fail}");
    assert!(!proof_fail.contains("<pre"), "{proof_fail}");
}

#[test]
fn script_hex_and_witness_stay_off_the_transaction_row() {
    let body = format!(
        r#"{{"txid":"{TX}","version":1,"locktime":0,"vin":[{{"txid":"{TX}","vout":0,"prevout":null,"scriptsig":"SCRIPTHEXDEAD","scriptsig_asm":"OP_0","witness":["WITNESSHEXDEAD"],"is_coinbase":false,"sequence":1}}],"vout":[{{"scriptpubkey":"PUBKEYHEXDEAD","scriptpubkey_asm":"OP_DUP","scriptpubkey_type":"p2pkh","value":1}}],"size":1,"weight":4,"sigops":0,"fee":1,"status":{{"confirmed":false}}}}"#
    );
    let row = transaction_row_markup(&body);
    assert!(row.contains(&tx_link(TX)), "{row}");
    assert!(!row.contains("SCRIPTHEXDEAD"), "{row}");
    assert!(!row.contains("WITNESSHEXDEAD"), "{row}");
    assert!(!row.contains("PUBKEYHEXDEAD"), "{row}");
}

#[test]
fn sixty_four_digits_stay_a_height_and_failed_broadcast_is_not_a_tx_link() {
    let digits = "1".repeat(64);
    let plan = plan_search(&digits).expect("height");
    assert_eq!(plan.requests[0].path, format!("/block-height/{digits}"));
    assert!(plan.requests.iter().all(|req| !req.path.contains("/tx/")));
    let liquid = plan_search_on(&digits, true).expect("height on liquid");
    assert_eq!(liquid.requests[0].path, format!("/block-height/{digits}"));
    assert!(
        liquid
            .requests
            .iter()
            .all(|req| !req.path.contains("/asset/"))
    );
    assert!(
        liquid
            .requests
            .iter()
            .all(|req| !req.path.contains("/scripthash/"))
    );

    let height = block_load_plan(&digits, None);
    assert_eq!(height.len(), 1);
    assert!(height[0].path.starts_with("/block-height/"), "{height:?}");

    let failed = "min relay fee not met\nindexer error POST /tx";
    assert!(
        broadcast_fields_text(failed).contains("min relay fee not met")
            || broadcast_fields_text(failed).contains("indexer error")
    );
    let links = submission_links("/tx", failed);
    assert!(!links.contains(&format!("/tx/{failed}")), "{links}");
    assert!(!links.contains("href=\"/tx/min"), "{links}");

    assert_eq!(
        summary_rows_markup("[ ]"),
        "<p class=\"empty\">There are no transactions.</p>"
    );
    let hex_fail = indexed_bytes_markup(
        "Transaction not found\nindexer error GET /tx/abc/hex",
        "tx-raw",
    );
    assert!(hex_fail.contains("Transaction not found"), "{hex_fail}");
    assert!(!hex_fail.contains("<pre"), "{hex_fail}");
}

#[test]
fn liquid_search_looks_up_the_asset_and_mainnet_does_not() {
    let mainnet = plan_search_on(HEX, false).expect("mainnet hex");
    assert!(
        mainnet
            .requests
            .iter()
            .all(|req| !req.path.contains("/asset/"))
    );
    assert!(
        mainnet
            .requests
            .iter()
            .any(|req| req.path == format!("/scripthash/{HEX}/txs"))
    );
    assert!(
        mainnet
            .requests
            .iter()
            .any(|req| req.path == format!("/scripthash/{HEX}/utxo"))
    );
    assert!(
        mainnet
            .requests
            .iter()
            .any(|req| req.path == format!("/scripthash/{HEX}/txs/mempool"))
    );

    let liquid = plan_search_on(HEX, true).expect("liquid hex");
    assert!(
        liquid
            .requests
            .iter()
            .any(|req| req.path == format!("/asset/{HEX}"))
    );
    assert!(
        liquid
            .requests
            .iter()
            .any(|req| req.path == format!("/asset/{HEX}/txs"))
    );
    let asset = r#"{"asset_id":"abcdef","name":"Tether USD","ticker":"USDt"}"#;
    let txs = format!(
        r#"[{{"txid":"{TX}","version":1,"locktime":0,"vin":[],"vout":[],"size":1,"weight":4,"sigops":0,"fee":1,"status":{{"confirmed":false}}}}]"#
    );
    let painted = search_loaded_markup(
        HEX,
        true,
        &[
            (format!("/asset/{HEX}"), asset.to_string()),
            (format!("/asset/{HEX}/txs"), txs),
        ],
    );
    assert!(painted.contains("Tether USD"), "{painted}");
    assert!(painted.contains("USDt"), "{painted}");
    assert!(painted.contains(&tx_link(TX)), "{painted}");
    let hidden = search_loaded_markup(HEX, false, &[(format!("/asset/{HEX}"), asset.to_string())]);
    assert!(!hidden.contains("Tether USD"), "{hidden}");
    assert!(!hidden.contains("USDt"), "{hidden}");
}

#[test]
fn liquid_registry_history_and_supply_stay_off_mainnet() {
    let id = HEX;
    assert!(registry_list_request(false).is_none());
    assert!(registry_entry_request(false, id).is_none());
    assert!(registry_search_request(false, "usd").is_none());
    assert!(liquid_asset_requests(false, id).is_empty());

    let list = registry_list_request(true).expect("list");
    assert_eq!(list.method, "GET");
    assert_eq!(list.path, "/assets/registry");
    let entry = registry_entry_request(true, id).expect("entry");
    assert_eq!(entry.path, format!("/assets/registry/{id}"));
    let search = registry_search_request(true, "usd t").expect("search");
    assert_eq!(search.path, "/assets/registry/search?q=usd%20t");
    assert_eq!(
        asset_confirmed_txs_request(true, id).expect("chain").path,
        format!("/asset/{id}/txs/chain")
    );
    assert_eq!(
        asset_confirmed_page_request(true, id, TX)
            .expect("page")
            .path,
        format!("/asset/{id}/txs/chain/{TX}")
    );
    assert_eq!(
        asset_mempool_txs_request(true, id).expect("mempool").path,
        format!("/asset/{id}/txs/mempool")
    );
    assert_eq!(
        asset_supply_request(true, id).expect("supply").path,
        format!("/asset/{id}/supply")
    );
    assert_eq!(
        asset_supply_decimal_request(true, id)
            .expect("decimal")
            .path,
        format!("/asset/{id}/supply/decimal")
    );

    let registry = format!(
        r#"[{{"asset_id":"{id}","name":"Tether USD","ticker":"USDt","precision":8,"issuance_txin":{{"txid":"{TX}","vin":0}},"issuance_prevout":{{"txid":"{TX}","vout":0}},"reissuance_token":"{TX}","status":{{"confirmed":true}},"chain_stats":{{"tx_count":1,"issuance_count":1,"issued_amount":1,"burned_amount":0,"has_blinded_issuances":false,"burned_reissuance_tokens":0}},"mempool_stats":{{"tx_count":0,"issuance_count":0,"issued_amount":0,"burned_amount":0,"has_blinded_issuances":false,"burned_reissuance_tokens":0}}}}]"#
    );
    let listed = registry_list_markup(&registry, true);
    assert!(listed.contains("Tether USD"), "{listed}");
    assert!(listed.contains("USDt"), "{listed}");
    assert!(listed.contains(&format!("/asset/{id}")), "{listed}");
    assert!(!registry_list_markup(&registry, false).contains("Tether USD"));

    let one = format!(
        r#"{{"name":"Tether USD","ticker":"USDt","precision":8,"contract":{{"entity":"tether"}},"entity":{{"domain":"tether.to"}}}}"#
    );
    let entry_html = registry_entry_markup(&one, true);
    assert!(entry_html.contains("Tether USD"), "{entry_html}");
    assert!(entry_html.contains("8"), "{entry_html}");
    assert!(entry_html.contains("tether.to"), "{entry_html}");
    assert!(!registry_entry_markup(&one, false).contains("Tether USD"));

    let hits = format!(
        r#"[{{"asset_id":"{id}","name":"Tether USD","ticker":"USDt","domain":"tether.to"}}]"#
    );
    let found = registry_search_markup(&hits, true);
    assert!(found.contains("Tether USD"), "{found}");
    assert!(found.contains("tether.to"), "{found}");
    assert!(found.contains(&format!("/asset/{id}")), "{found}");

    let txs = format!(
        r#"[{{"txid":"{TX}","fee":1,"size":1,"weight":4,"status":{{"confirmed":false}}}}]"#
    );
    let history = asset_history_markup(&txs, true);
    assert!(history.contains(&tx_link(TX)), "{history}");
    assert!(!asset_history_markup(&txs, false).contains(TX));
    assert_eq!(
        asset_history_markup("[ ]", true),
        "<p class=\"empty\">There are no transactions.</p>"
    );

    assert_eq!(
        supply_markup("21000000", true),
        "<p class=\"supply\">21000000</p>"
    );
    assert_eq!(
        supply_markup("210000.5", true),
        "<p class=\"supply\">210000.5</p>"
    );
    let blinded = supply_markup("Asset supply is blinded", true);
    assert!(blinded.contains("Asset supply is blinded"), "{blinded}");
    assert!(!blinded.contains("class=\"supply\""), "{blinded}");
    let missing = supply_markup("endpoint does not exist /asset/x/supply", true);
    assert!(missing.contains("endpoint does not exist"), "{missing}");
    assert!(!missing.contains("class=\"supply\""), "{missing}");
    let absent = registry_list_markup("not-served", true);
    assert!(absent.contains("not-served"), "{absent}");
    assert!(absent.contains("class=\"notice\""), "{absent}");
    assert_eq!(supply_markup("21000000", false), "");

    for call in liquid_asset_requests(true, id) {
        assert!(!call.path.contains("/internal"), "{}", call.path);
        assert!(!call.path.contains("electrum"), "{}", call.path);
        assert!(!call.path.contains("block-template"), "{}", call.path);
        assert!(!call.path.contains("/api/v1/ws"), "{}", call.path);
    }
}

#[test]
fn screens_request_the_public_routes_and_not_the_closed_ones() {
    let src = include_str!("../src/views.rs");
    for name in [
        "address_summary_request",
        "address_summary_page_request",
        "multi_address_summary_post",
        "txs_outspends_request",
        "tx_outspend_index_request",
        "block_txid_index_request",
        "block_header_request",
        "block_raw_request",
        "tx_raw_request",
        "tx_merkleblock_proof_request",
        "tx_status_request",
        "plan_search_on",
        "registry_list_request",
        "liquid_asset_requests",
        "Network::Liquid",
    ] {
        assert!(src.contains(name), "views.rs does not use {name}");
    }
    assert!(!src.contains("Fiat price last updated"));
    assert!(!src.contains("/internal/"));
    assert!(!src.contains("POST /electrum") && !src.contains("\"/electrum\""));
    assert!(!src.contains("block-template"));
    assert!(!src.contains("GET /api/v1/ws"));
    assert!(src.contains("does not open that socket"));
}
