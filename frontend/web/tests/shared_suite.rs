//! Thin driver for `splora_frontend_shared::suite`.
//!
//! Each case loads a fixture this app already parses, renders the shipped
//! Leptos markup function, and checks that markup against the shared case.
//! This file does not keep a second case list and it does not build the
//! expected page in the test.

use splora_frontend_shared::{
    address_tx_list, address_view_path, adjacent_blocks, block_details_open, block_open,
    block_open_from_height, block_pages_differ, block_tx_page, home_view, indexer_path_is_plain,
    liquid_rows_painted, next_block_tx_path, prefix_hits, prefix_lookup, recent_capped,
    recent_unconfirmed, recent_while_paused, recent_with_arrival, shared_cases,
    status_from_mempool, transaction_view,
};
use splora_web::{
    address_highlight_markup, address_prefix_markup, address_rows_markup, block_details_markup,
    block_page_heading_markup, block_rows_markup, block_transactions_markup, confirmation_markup,
    dashboard_loaded_markup, dashboard_recent_blocks_markup, dashboard_rows_markup,
    mempool_size_markup, mempool_summary_text, paint_block_prev_next_links,
    paint_caps_the_list_when_changing_the_limit_to_10,
    paint_loads_first_blockchain_blocks_visible_and_click_on_the_arrow_right,
    paint_loads_genesis_block_and_click_on_the_arrow_left,
    paint_pauses_updates_when_clicking_the_pause_icon, plan_search, recent_rows_markup,
    search_result_markup, shows_empty_blocks_properly_markup, transaction_row_markup,
};

const BLOCK: &str = include_str!("../../shared/tests/fixtures/block.json");
const BLOCKS: &str = include_str!("../../shared/tests/fixtures/blocks.json");
const BLOCK_TXS: &str = include_str!("../../shared/tests/fixtures/block_txs.json");
const TX: &str = include_str!("../../shared/tests/fixtures/tx.json");
const ADDRESS_TXS: &str = include_str!("../../shared/tests/fixtures/address_txs.json");
const RECENT: &str = include_str!("../../shared/tests/fixtures/mempool_recent.json");

/// `GET /mempool` body already used by the dashboard and status tests.
const MEMPOOL: &str = r#"{"count":4,"vsize":500,"total_fee":90,"fee_histogram":[[2,100]]}"#;

const PAGE: &str = "1wiz32gbHZwMzJCRHMGehJuBgsMTPdaCa";
const OTHER: &str = "bc1qother000000000000000000000000000000";

fn observe(case: &str, field: &str, value: &str) {
    println!("observable case={case} field={field} value={value}");
}

fn miss(errors: &mut Vec<String>, case: &str, detail: String) {
    println!("observable case={case} field=miss value={detail}");
    errors.push(format!("{case}: {detail}"));
}

fn require_contains(errors: &mut Vec<String>, case: &str, markup: &str, needle: &str) {
    if markup.contains(needle) {
        observe(case, "markup_has", needle);
    } else {
        miss(errors, case, format!("markup missing {needle}"));
    }
}

fn positions_in_order(markup: &str, ids: &[String]) -> bool {
    let mut from = 0usize;
    for id in ids {
        match markup[from..].find(id) {
            Some(at) => from += at + id.len(),
            None => return false,
        }
    }
    true
}

fn highlight_count(section: &str) -> usize {
    section.matches("<span class=\"highlight\">").count()
}

fn table_section(markup: &str, class: &str) -> String {
    let needle = format!("<table class=\"{class}\">");
    let Some(start) = markup.find(&needle) else {
        return String::new();
    };
    let rest = &markup[start..];
    let end = rest.find("</table>").unwrap_or(rest.len());
    rest[..end].to_string()
}

fn vin(address: &str) -> String {
    format!(
        r#"{{"txid":"prev","vout":0,"prevout":{{"scriptpubkey":"0014aa","scriptpubkey_asm":"OP_0","scriptpubkey_type":"v0_p2wpkh","scriptpubkey_address":"{address}","value":1}},"scriptsig":"","scriptsig_asm":"","is_coinbase":false,"sequence":1}}"#
    )
}

fn vout(address: &str) -> String {
    format!(
        r#"{{"scriptpubkey":"76a914bb88ac","scriptpubkey_asm":"OP_DUP","scriptpubkey_type":"p2pkh","scriptpubkey_address":"{address}","value":2}}"#
    )
}

fn tx_body(txid: &str, vins: &str, vouts: &str, confirmed: bool) -> String {
    format!(
        r#"{{"txid":"{txid}","version":1,"locktime":0,"vin":[{vins}],"vout":[{vouts}],"size":10,"weight":40,"sigops":0,"fee":1,"status":{{"confirmed":{confirmed}}}}}"#
    )
}

fn txs_array(items: &[String]) -> String {
    format!("[{}]", items.join(","))
}

fn with_tx_count(block_json: &str, tx_count: u32) -> String {
    block_json.replace("\"tx_count\": 12", &format!("\"tx_count\": {tx_count}"))
}

fn with_height(block_json: &str, height: u32) -> String {
    block_json.replace("\"height\": 100", &format!("\"height\": {height}"))
}

fn sole_object(array_json: &str) -> String {
    array_json
        .trim()
        .trim_start_matches('[')
        .trim_end_matches(']')
        .trim()
        .to_string()
}

fn newest_first_address_txs() -> String {
    let older = sole_object(ADDRESS_TXS);
    let newer = older.replace("addr-tx-1", "newer-tx");
    format!("[{newer},{older}]")
}

fn address_page(address: &str, txs_json: &str) -> String {
    let highlighted = address_highlight_markup(address, txs_json);
    let rows = address_rows_markup("", txs_json, "[]");
    format!("{rows}{highlighted}")
}

fn home_projected(errors: &mut Vec<String>) {
    let case = "home_projected_mempool_block";
    let home = match home_view(MEMPOOL, BLOCKS) {
        Ok(home) => home,
        Err(error) => {
            miss(errors, case, format!("home_view: {error}"));
            return;
        }
    };
    let markup = dashboard_loaded_markup(MEMPOOL);
    observe(case, "projected_link_id", &home.projected_link_id);
    observe(case, "projected_open_path", &home.projected_open_path);
    observe(
        case,
        "projected_navigable",
        if home.projected_navigable {
            "true"
        } else {
            "false"
        },
    );
    require_contains(errors, case, &markup, &home.projected_link_id);
    require_contains(errors, case, &markup, "blockLink");
    require_contains(errors, case, &markup, &home.projected_open_path);
    if !home.projected_navigable {
        miss(errors, case, "projected block is not navigable".to_string());
    }
    if !indexer_path_is_plain(&home.projected_open_path) {
        miss(
            errors,
            case,
            format!("path is not plain: {}", home.projected_open_path),
        );
    }
}

fn home_confirmed(errors: &mut Vec<String>) {
    let case = "home_recent_confirmed_blocks";
    let home = match home_view(MEMPOOL, BLOCKS) {
        Ok(home) => home,
        Err(error) => {
            miss(errors, case, format!("home_view: {error}"));
            return;
        }
    };
    let rows = dashboard_rows_markup(BLOCKS, "[]");
    let markup = dashboard_recent_blocks_markup(&rows, BLOCKS, true);
    if home.confirmed.is_empty() {
        miss(errors, case, "no confirmed block links".to_string());
    }
    for link in &home.confirmed {
        observe(case, "confirmed_id", &link.id);
        observe(case, "confirmed_height", &link.height.to_string());
        observe(case, "confirmed_open_path", &link.open_path);
        require_contains(errors, case, &markup, &link.id);
        require_contains(errors, case, &markup, &link.height.to_string());
        require_contains(errors, case, &markup, &link.open_path);
        if !indexer_path_is_plain(&link.open_path) {
            miss(
                errors,
                case,
                format!("path is not plain: {}", link.open_path),
            );
        }
    }
}

fn search_legacy(errors: &mut Vec<String>) {
    let case = "search_partial_legacy_address";
    let typed = "1wiz";
    let lookup = match prefix_lookup(typed) {
        Ok(lookup) => lookup,
        Err(error) => {
            miss(errors, case, format!("prefix_lookup: {error}"));
            return;
        }
    };
    let Some(plan) = plan_search(typed) else {
        miss(
            errors,
            case,
            format!("plan_search returned none for {typed}"),
        );
        return;
    };
    let path = plan
        .requests
        .first()
        .map(|request| request.path.as_str())
        .unwrap_or("");
    observe(case, "lookup_path", &lookup.path);
    observe(case, "screen_path", path);
    observe(case, "folded", if lookup.folded { "true" } else { "false" });
    if path != lookup.path {
        miss(
            errors,
            case,
            format!("screen path {path} != shared {}", lookup.path),
        );
    }
    if lookup.folded {
        miss(errors, case, "legacy prefix was folded".to_string());
    }
    let body = format!("[\"{PAGE}\"]");
    let hits = match prefix_hits(typed, &body) {
        Ok(hits) => hits,
        Err(error) => {
            miss(errors, case, format!("prefix_hits: {error}"));
            return;
        }
    };
    let markup = format!(
        "{}{}",
        search_result_markup(typed, &body),
        address_prefix_markup(&body)
    );
    if hits.is_empty() {
        miss(errors, case, "no resolved address".to_string());
    }
    for hit in &hits {
        let open = address_view_path(hit);
        observe(case, "resolved_address", hit);
        observe(case, "address_view_path", &open);
        require_contains(errors, case, &markup, &open);
    }
}

fn search_narrows(errors: &mut Vec<String>) {
    let case = "search_narrows";
    let short = "1wiz";
    let long = "1wiz32";
    let short_body = format!("[\"{PAGE}\",\"1wizOTHERADDRESS111111111111111\"]");
    let long_body = format!("[\"{PAGE}\"]");
    let short_lookup = match prefix_lookup(short) {
        Ok(lookup) => lookup,
        Err(error) => {
            miss(errors, case, format!("short lookup: {error}"));
            return;
        }
    };
    let long_lookup = match prefix_lookup(long) {
        Ok(lookup) => lookup,
        Err(error) => {
            miss(errors, case, format!("long lookup: {error}"));
            return;
        }
    };
    let short_hits = prefix_hits(short, &short_body).unwrap_or_default();
    let long_hits = prefix_hits(long, &long_body).unwrap_or_default();
    observe(case, "short_path", &short_lookup.path);
    observe(case, "long_path", &long_lookup.path);
    observe(case, "short_hit_count", &short_hits.len().to_string());
    observe(case, "long_hit_count", &long_hits.len().to_string());
    if long_hits.len() >= short_hits.len() {
        miss(
            errors,
            case,
            "longer prefix did not narrow the hits".to_string(),
        );
    }
    let Some(short_plan) = plan_search(short) else {
        miss(errors, case, "short plan missing".to_string());
        return;
    };
    let Some(long_plan) = plan_search(long) else {
        miss(errors, case, "long plan missing".to_string());
        return;
    };
    let short_path = short_plan.requests[0].path.as_str();
    let long_path = long_plan.requests[0].path.as_str();
    if short_path != short_lookup.path || long_path != long_lookup.path {
        miss(
            errors,
            case,
            format!("paths {short_path} {long_path} != shared"),
        );
    }
    let short_markup = search_result_markup(short, &short_body);
    let long_markup = search_result_markup(long, &long_body);
    for hit in &short_hits {
        require_contains(errors, case, &short_markup, &address_view_path(hit));
    }
    for hit in &long_hits {
        require_contains(errors, case, &long_markup, &address_view_path(hit));
    }
    if long_markup.contains("1wizOTHERADDRESS111111111111111") {
        miss(
            errors,
            case,
            "longer query still showed the dropped address".to_string(),
        );
    }
}

fn search_folded(errors: &mut Vec<String>, case: &str, typed: &str, body: &str) {
    let lookup = match prefix_lookup(typed) {
        Ok(lookup) => lookup,
        Err(error) => {
            miss(errors, case, format!("prefix_lookup: {error}"));
            return;
        }
    };
    let Some(plan) = plan_search(typed) else {
        miss(
            errors,
            case,
            format!("plan_search returned none for {typed}"),
        );
        return;
    };
    let path = plan.requests[0].path.as_str();
    observe(case, "typed", typed);
    observe(case, "lookup_path", &lookup.path);
    observe(case, "screen_path", path);
    observe(case, "folded", if lookup.folded { "true" } else { "false" });
    if !lookup.folded {
        miss(errors, case, "shared lookup did not fold".to_string());
    }
    if path != lookup.path {
        miss(
            errors,
            case,
            format!("screen path {path} != shared {}", lookup.path),
        );
    }
    let hits = match prefix_hits(typed, body) {
        Ok(hits) => hits,
        Err(error) => {
            miss(errors, case, format!("prefix_hits: {error}"));
            return;
        }
    };
    let markup = search_result_markup(typed, body);
    if hits.is_empty() {
        miss(errors, case, "no folded hit".to_string());
    }
    for hit in &hits {
        let open = address_view_path(hit);
        observe(case, "resolved_address", hit);
        require_contains(errors, case, &markup, &open);
    }
}

fn address_order(errors: &mut Vec<String>) {
    let case = "address_txs_newest_first";
    let txs = newest_first_address_txs();
    let list = match address_tx_list(PAGE, &txs) {
        Ok(list) => list,
        Err(error) => {
            miss(errors, case, format!("address_tx_list: {error}"));
            return;
        }
    };
    let markup = address_page(PAGE, &txs);
    let shown = list.txids.join(",");
    observe(case, "txids", &shown);
    if list.txids.first().map(String::as_str) != Some("newer-tx") {
        miss(errors, case, format!("newest txid was not first: {shown}"));
    }
    if !positions_in_order(&markup, &list.txids) {
        miss(errors, case, format!("markup order != {shown}"));
    } else {
        observe(case, "markup_order", &shown);
    }
}

fn address_marks(errors: &mut Vec<String>, case: &str, vins: &str, vouts: &str) {
    let body = txs_array(&[tx_body("marked-tx", vins, vouts, true)]);
    let list = match address_tx_list(PAGE, &body) {
        Ok(list) => list,
        Err(error) => {
            miss(errors, case, format!("address_tx_list: {error}"));
            return;
        }
    };
    let markup = address_highlight_markup(PAGE, &body);
    let vin_table = table_section(&markup, "table-tx-vin");
    let vout_table = table_section(&markup, "table-tx-vout");
    let mark = list
        .marks
        .first()
        .copied()
        .unwrap_or(splora_frontend_shared::HighlightCount {
            inputs: 0,
            outputs: 0,
        });
    observe(case, "highlight_inputs", &mark.inputs.to_string());
    observe(case, "highlight_outputs", &mark.outputs.to_string());
    let painted_inputs = highlight_count(&vin_table);
    let painted_outputs = highlight_count(&vout_table);
    observe(case, "painted_inputs", &painted_inputs.to_string());
    observe(case, "painted_outputs", &painted_outputs.to_string());
    if painted_inputs != mark.inputs || painted_outputs != mark.outputs {
        miss(
            errors,
            case,
            format!(
                "painted {painted_inputs}/{painted_outputs} != shared {}/{}",
                mark.inputs, mark.outputs
            ),
        );
    }
}

fn transaction_io(errors: &mut Vec<String>) {
    let case = "transaction_inputs_outputs";
    let body = tx_body(
        "io-tx",
        &format!("{},{}", vin(PAGE), vin(OTHER)),
        &format!("{},{}", vout(PAGE), vout(OTHER)),
        true,
    );
    let view = match transaction_view(&body) {
        Ok(view) => view,
        Err(error) => {
            miss(errors, case, format!("transaction_view: {error}"));
            return;
        }
    };
    let markup = transaction_row_markup(&body);
    observe(case, "txid", &view.txid);
    observe(case, "input_count", &view.input_count.to_string());
    observe(case, "output_count", &view.output_count.to_string());
    require_contains(errors, case, &markup, &view.txid);
    require_contains(errors, case, &markup, "Inputs & Outputs");
    let painted_inputs = markup.matches("<span class=\"vout\">").count();
    if painted_inputs != view.input_count {
        miss(
            errors,
            case,
            format!("painted inputs {painted_inputs} != {}", view.input_count),
        );
    } else {
        observe(case, "painted_inputs", &painted_inputs.to_string());
    }
    let painted_outputs = markup.matches(&format!("href=\"/address/{PAGE}\"")).count()
        + markup
            .matches(&format!("href=\"/address/{OTHER}\""))
            .count();
    // Each input with a prevout address also paints that address.
    let input_addresses = 2usize;
    if painted_outputs < view.output_count + input_addresses {
        miss(
            errors,
            case,
            format!("painted address links {painted_outputs} under inputs plus outputs"),
        );
    } else {
        observe(case, "painted_address_links", &painted_outputs.to_string());
    }
}

fn transaction_confirmed(errors: &mut Vec<String>, case: &str, body: &str, tip: &str, want: bool) {
    let view = match transaction_view(body) {
        Ok(view) => view,
        Err(error) => {
            miss(errors, case, format!("transaction_view: {error}"));
            return;
        }
    };
    let markup = format!(
        "{}{}",
        transaction_row_markup(body),
        confirmation_markup(body, tip)
    );
    observe(case, "txid", &view.txid);
    observe(
        case,
        "confirmed",
        if view.confirmed { "true" } else { "false" },
    );
    if let Some(hash) = &view.block_hash {
        observe(case, "block_hash", hash);
        require_contains(errors, case, &markup, hash);
    }
    if view.confirmed != want {
        miss(
            errors,
            case,
            format!("shared confirmed {} != {want}", view.confirmed),
        );
    }
    require_contains(errors, case, &markup, &view.txid);
    let word = if want { "Confirmed" } else { "Unconfirmed" };
    let flag = if want { "true" } else { "false" };
    require_contains(errors, case, &markup, word);
    require_contains(
        errors,
        case,
        &markup,
        &format!("class=\"confirmed\">{flag}"),
    );
}

fn block_hash_case(errors: &mut Vec<String>) {
    let case = "block_by_hash";
    let opened = match block_open(BLOCK) {
        Ok(opened) => opened,
        Err(error) => {
            miss(errors, case, format!("block_open: {error}"));
            return;
        }
    };
    let markup = format!(
        "{}{}{}",
        block_page_heading_markup(BLOCK),
        block_rows_markup(BLOCK, BLOCK_TXS),
        block_transactions_markup(BLOCK, BLOCK_TXS, 1)
    );
    observe(case, "hash", &opened.hash);
    observe(case, "height", &opened.height.to_string());
    observe(case, "tx_count", &opened.tx_count.to_string());
    observe(case, "heading", &opened.heading);
    observe(case, "open_path", &opened.open_path);
    require_contains(errors, case, &markup, &opened.hash);
    require_contains(errors, case, &markup, &opened.height.to_string());
    require_contains(errors, case, &markup, &opened.heading);
    require_contains(errors, case, &markup, &opened.open_path);
}

fn block_height_case(errors: &mut Vec<String>) {
    let case = "block_by_height";
    let opened = match block_open(BLOCK) {
        Ok(opened) => opened,
        Err(error) => {
            miss(errors, case, format!("block_open: {error}"));
            return;
        }
    };
    let from_height = match block_open_from_height(opened.height, &opened.hash, BLOCK) {
        Ok(opened) => opened,
        Err(error) => {
            miss(errors, case, format!("block_open_from_height: {error}"));
            return;
        }
    };
    let markup = block_page_heading_markup(BLOCK);
    observe(case, "height", &from_height.height.to_string());
    observe(case, "hash", &from_height.hash);
    observe(case, "open_path", &from_height.open_path);
    require_contains(errors, case, &markup, &from_height.height.to_string());
    require_contains(errors, case, &markup, &from_height.open_path);
}

fn block_height_zero(errors: &mut Vec<String>) {
    let case = "block_height_zero";
    let body = with_height(BLOCK, 0);
    let opened = match block_open(&body) {
        Ok(opened) => opened,
        Err(error) => {
            miss(errors, case, format!("block_open: {error}"));
            return;
        }
    };
    if opened.height != 0 {
        miss(errors, case, format!("height {}", opened.height));
    }
    let from_height = match block_open_from_height(0, &opened.hash, &body) {
        Ok(opened) => opened,
        Err(error) => {
            miss(errors, case, format!("block_open_from_height: {error}"));
            return;
        }
    };
    let markup = block_page_heading_markup(&body);
    observe(case, "height", &from_height.height.to_string());
    observe(case, "heading_word", "Genesis");
    require_contains(errors, case, &markup, "Genesis");
    require_contains(errors, case, &markup, &from_height.open_path);
}

fn block_coinbase(errors: &mut Vec<String>) {
    let case = "block_coinbase_only";
    let body = with_tx_count(BLOCK, 1);
    let opened = match block_open(&body) {
        Ok(opened) => opened,
        Err(error) => {
            miss(errors, case, format!("block_open: {error}"));
            return;
        }
    };
    let page = match block_tx_page(BLOCK_TXS) {
        Ok(page) => page,
        Err(error) => {
            miss(errors, case, format!("block_tx_page: {error}"));
            return;
        }
    };
    let markup = format!(
        "{}{}",
        shows_empty_blocks_properly_markup(&body),
        block_transactions_markup(&body, BLOCK_TXS, 1)
    );
    observe(case, "tx_count", &opened.tx_count.to_string());
    observe(case, "heading", &opened.heading);
    observe(case, "page_len", &page.txids.len().to_string());
    if opened.tx_count != 1 || page.txids.len() != 1 {
        miss(
            errors,
            case,
            "coinbase-only block did not have one transaction".to_string(),
        );
    }
    require_contains(errors, case, &markup, &opened.heading);
    for txid in &page.txids {
        observe(case, "txid", txid);
        require_contains(errors, case, &markup, txid);
    }
}

fn block_next_page(errors: &mut Vec<String>) {
    let case = "block_next_page";
    let body = with_tx_count(BLOCK, 30);
    let opened = match block_open(&body) {
        Ok(opened) => opened,
        Err(error) => {
            miss(errors, case, format!("block_open: {error}"));
            return;
        }
    };
    let next_txs = BLOCK_TXS.replace("block-tx-1", "block-tx-2");
    let differ = match block_pages_differ(BLOCK_TXS, &next_txs) {
        Ok(differ) => differ,
        Err(error) => {
            miss(errors, case, format!("block_pages_differ: {error}"));
            return;
        }
    };
    let first = block_transactions_markup(&body, BLOCK_TXS, 1);
    let second = block_transactions_markup(&body, &next_txs, 2);
    let next_path = next_block_tx_path(&opened.hash);
    observe(case, "pages_differ", if differ { "true" } else { "false" });
    observe(case, "next_path", &next_path);
    if !differ {
        miss(
            errors,
            case,
            "next page txids matched the first page".to_string(),
        );
    }
    require_contains(errors, case, &first, "block-tx-1");
    if first.contains("block-tx-2") {
        miss(
            errors,
            case,
            "first page showed the next page txid".to_string(),
        );
    }
    require_contains(errors, case, &second, "block-tx-2");
    if second.contains("block-tx-1") {
        miss(
            errors,
            case,
            "next page showed the first page txid".to_string(),
        );
    }
    require_contains(errors, case, &second, &next_path);
    if !indexer_path_is_plain(&next_path) {
        miss(errors, case, format!("path is not plain: {next_path}"));
    }
}

fn block_edges(errors: &mut Vec<String>, case: &str, height: u32, tip: u32) {
    let edges = adjacent_blocks(height, tip);
    let (previous, next) = paint_block_prev_next_links(Some(height), tip);
    observe(case, "height", &height.to_string());
    observe(case, "tip", &tip.to_string());
    observe(
        case,
        "previous_shown",
        if edges.previous_shown {
            "true"
        } else {
            "false"
        },
    );
    observe(
        case,
        "next_shown",
        if edges.next_shown { "true" } else { "false" },
    );
    if let Some(previous_height) = edges.previous_height {
        observe(case, "previous_height", &previous_height.to_string());
    }
    if let Some(next_height) = edges.next_height {
        observe(case, "next_height", &next_height.to_string());
    }
    if previous != edges.previous_shown || next != edges.next_shown {
        miss(
            errors,
            case,
            format!("screen flags {previous}/{next} != shared"),
        );
    }
}

fn block_genesis(errors: &mut Vec<String>) {
    let case = "block_genesis_no_previous";
    block_edges(errors, case, 0, 2);
    let (on_genesis, _) = paint_loads_genesis_block_and_click_on_the_arrow_left();
    let edges = adjacent_blocks(0, 2);
    if edges.previous_shown {
        miss(errors, case, "genesis showed a previous block".to_string());
    }
    if on_genesis.contains("Previous Block") {
        miss(
            errors,
            case,
            "genesis markup included Previous Block".to_string(),
        );
    } else {
        observe(case, "previous_label", "absent");
    }
    if edges.next_shown && !on_genesis.contains("Next Block") {
        miss(
            errors,
            case,
            "genesis markup omitted Next Block".to_string(),
        );
    }
}

fn block_tip(errors: &mut Vec<String>) {
    let case = "block_tip_no_next";
    block_edges(errors, case, 2, 2);
    let (on_tip, _) = paint_loads_first_blockchain_blocks_visible_and_click_on_the_arrow_right();
    let edges = adjacent_blocks(2, 2);
    if edges.next_shown {
        miss(errors, case, "tip showed a next block".to_string());
    }
    if on_tip.contains("Next Block") {
        miss(errors, case, "tip markup included Next Block".to_string());
    } else {
        observe(case, "next_label", "absent");
    }
    if edges.previous_shown && !on_tip.contains("Previous Block") {
        miss(
            errors,
            case,
            "tip markup omitted Previous Block".to_string(),
        );
    }
}

fn block_details(errors: &mut Vec<String>) {
    let case = "block_details_toggle";
    let opened_flag = block_details_open(false);
    let closed_flag = block_details_open(opened_flag);
    let closed = block_details_markup(BLOCK, false);
    let opened = block_details_markup(BLOCK, opened_flag);
    let closed_again = block_details_markup(BLOCK, closed_flag);
    observe(
        case,
        "after_first_click_open",
        if opened_flag { "true" } else { "false" },
    );
    observe(
        case,
        "after_second_click_open",
        if closed_flag { "true" } else { "false" },
    );
    if !opened_flag || closed_flag {
        miss(
            errors,
            case,
            "shared toggle did not expand then collapse".to_string(),
        );
    }
    if !closed.contains("id=\"details\" hidden") {
        miss(errors, case, "start state did not hide details".to_string());
    } else {
        observe(case, "start", "collapsed");
    }
    if opened.contains("id=\"details\" hidden") || !opened.contains("id=\"details\"") {
        miss(
            errors,
            case,
            "expanded markup still hid details".to_string(),
        );
    } else {
        observe(case, "expanded", "visible");
    }
    if !closed_again.contains("id=\"details\" hidden") {
        miss(
            errors,
            case,
            "second click did not hide details".to_string(),
        );
    } else {
        observe(case, "collapsed", "hidden");
    }
}

fn recent_arrival(errors: &mut Vec<String>) {
    let case = "recent_arrival";
    let visible = match recent_unconfirmed(RECENT) {
        Ok(ids) => ids,
        Err(error) => {
            miss(errors, case, format!("recent_unconfirmed: {error}"));
            return;
        }
    };
    let arrived = "just-arrived";
    let next = recent_with_arrival(&visible, arrived);
    let inner = sole_object(RECENT);
    let arrived_body = format!(r#"[{{"txid":"{arrived}","fee":1,"vsize":2,"value":3}},{inner}]"#);
    let markup = recent_rows_markup(&arrived_body);
    let shown = next.join(",");
    observe(case, "txids", &shown);
    if next.first().map(String::as_str) != Some(arrived) {
        miss(errors, case, format!("arrival was not first: {shown}"));
    }
    if !positions_in_order(&markup, &next) {
        miss(errors, case, format!("markup order != {shown}"));
    } else {
        observe(case, "markup_order", &shown);
    }
}

fn recent_pause(errors: &mut Vec<String>) {
    let case = "recent_pause";
    let visible = vec!["recent-tx-1".to_string(), "recent-tx-2".to_string()];
    let later = vec!["later-tx".to_string()];
    let kept = recent_while_paused(&visible, &later);
    let visible_ref: Vec<&str> = visible.iter().map(String::as_str).collect();
    let later_ref: Vec<&str> = later.iter().map(String::as_str).collect();
    let markup = paint_pauses_updates_when_clicking_the_pause_icon(&visible_ref, &later_ref);
    let shown = kept.join(",");
    observe(case, "paused_txids", &shown);
    if kept != visible {
        miss(errors, case, format!("pause changed the list: {shown}"));
    }
    if markup.contains("later-tx") {
        miss(
            errors,
            case,
            "paused markup included a later txid".to_string(),
        );
    }
    if !positions_in_order(&markup, &kept) {
        miss(errors, case, format!("markup order != {shown}"));
    } else {
        observe(case, "markup_order", &shown);
    }
}

fn recent_cap(errors: &mut Vec<String>) {
    let case = "recent_cap";
    let mut ids = Vec::new();
    let mut objects = Vec::new();
    for index in 0..12 {
        let id = format!("recent-{index}");
        objects.push(format!(r#"{{"txid":"{id}","fee":1,"vsize":2,"value":3}}"#));
        ids.push(id);
    }
    let capped = recent_capped(&ids, 10);
    let markup = recent_rows_markup(&format!("[{}]", objects.join(",")));
    let shown = capped.join(",");
    observe(case, "cap", "10");
    observe(case, "capped_txids", &shown);
    if capped.len() != 10 {
        miss(errors, case, format!("cap length {}", capped.len()));
    }
    if !positions_in_order(&markup, &capped) {
        miss(errors, case, format!("markup order != {shown}"));
    } else {
        observe(case, "markup_order", &shown);
    }
    if markup.contains("recent-10") || markup.contains("recent-11") {
        miss(
            errors,
            case,
            "markup showed a txid past the cap".to_string(),
        );
    }
    let batches = [["recent-0"; 6]; 9];
    let cap_markup = paint_caps_the_list_when_changing_the_limit_to_10(&batches);
    require_contains(errors, case, &cap_markup, "data-cy=\"limit-10\"");
}

fn status_count(errors: &mut Vec<String>) {
    let case = "status_unconfirmed_count";
    let status = match status_from_mempool(MEMPOOL) {
        Ok(status) => status,
        Err(error) => {
            miss(errors, case, format!("status_from_mempool: {error}"));
            return;
        }
    };
    let markup = mempool_summary_text(MEMPOOL);
    observe(
        case,
        "unconfirmed_count",
        &status.unconfirmed_count.to_string(),
    );
    require_contains(errors, case, &markup, "Unconfirmed:");
    require_contains(
        errors,
        case,
        &markup,
        &format!("mempool count {}", status.unconfirmed_count),
    );
}

fn status_vsize(errors: &mut Vec<String>) {
    let case = "status_mempool_vsize";
    let status = match status_from_mempool(MEMPOOL) {
        Ok(status) => status,
        Err(error) => {
            miss(errors, case, format!("status_from_mempool: {error}"));
            return;
        }
    };
    let summary = mempool_summary_text(MEMPOOL);
    let size = mempool_size_markup(MEMPOOL);
    let markup = format!("{summary}{size}");
    observe(case, "vsize", &status.vsize.to_string());
    observe(case, "byte_size", "absent");
    require_contains(errors, case, &markup, "Mempool size");
    require_contains(errors, case, &markup, &format!("vsize {}", status.vsize));
    require_contains(errors, case, &size, &status.vsize.to_string());
}

#[test]
fn shared_suite_drives_shipped_leptos_screens() {
    assert!(
        !liquid_rows_painted(),
        "liquid rows are not painted in this crate"
    );
    observe("suite", "liquid_rows_painted", "false");
    let mut errors = Vec::new();
    for case in shared_cases() {
        println!("shared_case {}", case.name);
        match case.name {
            "home_projected_mempool_block" => home_projected(&mut errors),
            "home_recent_confirmed_blocks" => home_confirmed(&mut errors),
            "search_partial_legacy_address" => search_legacy(&mut errors),
            "search_narrows" => search_narrows(&mut errors),
            "search_bech32_case_insensitive" => {
                // Same partials as `search_folds_partial_bech32_and_bech32m`.
                // Exact `bc1q` stays in the web search unit test.
                let body = "[\"bc1q0003match000000000000000000000000\"]";
                for typed in ["BC1Q0003", "bC1q0003"] {
                    search_folded(&mut errors, case.name, typed, body);
                }
            }
            "search_bech32m_case_insensitive" => search_folded(
                &mut errors,
                case.name,
                "BC1PQYQS",
                "[\"bc1pqyqsexample\",\"bc1qnotbech32m\"]",
            ),
            "address_txs_newest_first" => address_order(&mut errors),
            "address_highlight_inputs" => {
                address_marks(&mut errors, case.name, &vin(PAGE), &vout(OTHER))
            }
            "address_highlight_outputs" => {
                address_marks(&mut errors, case.name, &vin(OTHER), &vout(PAGE))
            }
            "address_highlight_both" => {
                address_marks(&mut errors, case.name, &vin(PAGE), &vout(PAGE))
            }
            "address_highlight_several_inputs" => address_marks(
                &mut errors,
                case.name,
                &format!("{},{}", vin(PAGE), vin(PAGE)),
                &vout(OTHER),
            ),
            "address_highlight_several_outputs" => address_marks(
                &mut errors,
                case.name,
                &vin(OTHER),
                &format!("{},{}", vout(PAGE), vout(PAGE)),
            ),
            "transaction_inputs_outputs" => transaction_io(&mut errors),
            "transaction_confirmed" => transaction_confirmed(&mut errors, case.name, TX, "", true),
            "transaction_unconfirmed" => transaction_confirmed(
                &mut errors,
                case.name,
                &sole_object(ADDRESS_TXS),
                "100",
                false,
            ),
            "block_by_hash" => block_hash_case(&mut errors),
            "block_by_height" => block_height_case(&mut errors),
            "block_height_zero" => block_height_zero(&mut errors),
            "block_coinbase_only" => block_coinbase(&mut errors),
            "block_next_page" => block_next_page(&mut errors),
            "block_adjacent" => block_edges(&mut errors, case.name, 1, 2),
            "block_genesis_no_previous" => block_genesis(&mut errors),
            "block_tip_no_next" => block_tip(&mut errors),
            "block_details_toggle" => block_details(&mut errors),
            "recent_arrival" => recent_arrival(&mut errors),
            "recent_pause" => recent_pause(&mut errors),
            "recent_cap" => recent_cap(&mut errors),
            "status_unconfirmed_count" => status_count(&mut errors),
            "status_mempool_vsize" => status_vsize(&mut errors),
            other => miss(
                &mut errors,
                other,
                "no shipped screen for this case".to_string(),
            ),
        }
    }
    assert!(
        errors.is_empty(),
        "shared suite misses:\n{}",
        errors.join("\n")
    );
}
