//! Block, transaction, mempool, utxo, and address fields the page has to show.
//!
//! Row markup is the page. Field sentences have to carry the same facts.

use splora_web::{
    address_fields_text, address_rows_markup, block_fields_text, block_rows_markup,
    blocks_rows_markup, broadcast_fields_text, dashboard_fields_text, dashboard_rows_markup,
    dashboard_tip_fees_text, transaction_fields_text, transaction_row_markup,
};

const BLOCK_HASH: &str = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
const TXID_A: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
const TXID_B: &str = "dddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddd";

fn views_source() -> &'static str {
    include_str!("../src/views.rs")
}

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
        .min()
        .unwrap_or(tail.len());
    &rest[..after_sig + end_in_tail]
}

fn row_bodies(markup: &str, class: &str) -> Vec<String> {
    let needle = format!("<tr class=\"{class}\">");
    let mut rows = Vec::new();
    let mut rest = markup;
    while let Some(start) = rest.find(&needle) {
        let after = &rest[start + needle.len()..];
        let end = after
            .find("</tr>")
            .unwrap_or_else(|| panic!("{class} row is not closed: {markup}"));
        rows.push(after[..end].to_string());
        rest = &after[end + "</tr>".len()..];
    }
    rows
}

fn cell(row: &str, class: &str) -> String {
    let open = format!("<td class=\"{class}\">");
    let start = row
        .find(&open)
        .unwrap_or_else(|| panic!("row missing {class}: {row}"));
    assert_eq!(
        row.matches(&open).count(),
        1,
        "row has more than one {class} cell: {row}"
    );
    let after = &row[start + open.len()..];
    let end = after
        .find("</td>")
        .unwrap_or_else(|| panic!("{class} cell is not closed: {row}"));
    after[..end].to_string()
}

fn element_text(markup: &str, class: &str) -> String {
    let open = format!("<p class=\"{class}\">");
    let start = markup
        .find(&open)
        .unwrap_or_else(|| panic!("missing {class} in {markup}"));
    assert_eq!(
        markup.matches(&open).count(),
        1,
        "more than one {class} in {markup}"
    );
    let after = &markup[start + open.len()..];
    let end = after
        .find("</p>")
        .unwrap_or_else(|| panic!("{class} is not closed: {markup}"));
    after[..end].to_string()
}

fn txid_link(id: &str) -> String {
    format!("<a href=\"/tx/{id}\">{id}</a>")
}

fn rows_style_block_json() -> String {
    format!(
        r#"{{"id":"{BLOCK_HASH}","height":842001,"version":1,"timestamp":1700000000,"tx_count":17,"size":1234,"weight":5678,"merkle_root":"merkle","previousblockhash":"cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc","mediantime":1699999000,"nonce":1,"bits":1,"difficulty":1}}"#
    )
}

fn parties_tx_json() -> String {
    format!(
        r#"{{"txid":"{TXID_A}","version":2,"locktime":0,"vin":[{{"txid":"prev","vout":0,"prevout":{{"scriptpubkey":"0014aa","scriptpubkey_asm":"OP_0","scriptpubkey_type":"v0_p2wpkh","scriptpubkey_address":"bc1qinput","value":610677}},"scriptsig":"","scriptsig_asm":"","is_coinbase":false,"sequence":4294967295}}],"vout":[{{"scriptpubkey":"76a914bb88ac","scriptpubkey_asm":"OP_DUP","scriptpubkey_type":"p2pkh","scriptpubkey_address":"bc1qperson","value":344697}}],"size":224,"weight":572,"sigops":1,"fee":584,"status":{{"confirmed":true,"block_height":800000,"block_hash":"{BLOCK_HASH}","block_time":1600000000}}}}"#
    )
}

fn address_stats_json() -> String {
    r#"{"address":"bc1qexample","chain_stats":{"tx_count":4,"funded_txo_count":4,"spent_txo_count":1,"funded_txo_sum":5000,"spent_txo_sum":1000},"mempool_stats":{"tx_count":1,"funded_txo_count":1,"spent_txo_count":0,"funded_txo_sum":20,"spent_txo_sum":0}}"#
        .to_string()
}

fn assert_script_hex_stays_off(body: &str) {
    assert!(!body.contains("0014aa"), "script hex leaked: {body}");
    assert!(!body.contains("76a914bb88ac"), "script hex leaked: {body}");
    assert!(!body.contains("OP_0"), "script asm leaked: {body}");
    assert!(!body.contains("witness"), "witness leaked: {body}");
}

#[test]
fn block_row_shows_version_merkle_root_nonce_bits_and_whole_difficulty() {
    let markup = block_rows_markup(&rows_style_block_json(), "[]");
    let rows = row_bodies(&markup, "block-row");
    assert_eq!(rows.len(), 1, "block page: {markup}");
    assert_eq!(cell(&rows[0], "version"), "1");
    assert_eq!(cell(&rows[0], "merkle-root"), "merkle");
    assert_eq!(cell(&rows[0], "nonce"), "1");
    assert_eq!(cell(&rows[0], "bits"), "1");
    assert_eq!(cell(&rows[0], "difficulty"), "1");
    assert!(
        !rows[0].contains("1.0"),
        "whole difficulty printed as 1.0: {}",
        rows[0]
    );

    let list = blocks_rows_markup(&format!("[{}]", rows_style_block_json()));
    assert!(
        row_bodies(&list, "tx-row").is_empty(),
        "blocks list invented tx rows: {list}"
    );
    let listed = row_bodies(&list, "block-row");
    assert_eq!(listed.len(), 1, "blocks list: {list}");
    assert_eq!(cell(&listed[0], "difficulty"), "1");
}

#[test]
fn block_fixture_sentence_appends_version_merkle_nonce_bits_and_difficulty() {
    let block = include_str!("../../shared/tests/fixtures/block.json");
    let text = block_fields_text(block, "[]");
    assert!(text.contains("version 536870912"), "{text}");
    assert!(text.contains("merkle_root merkle-root-1"), "{text}");
    assert!(text.contains("nonce 7"), "{text}");
    assert!(text.contains("bits 386604799"), "{text}");
    assert!(text.contains("difficulty 1"), "{text}");
    assert!(
        !text.contains("difficulty 1.0"),
        "difficulty was not a whole number: {text}"
    );
    assert!(text.contains("height 100"), "{text}");
    assert!(text.contains("tx_count 12"), "{text}");
}

#[test]
fn transaction_fee_rate_is_fee_over_weight_divided_by_four() {
    let body = parties_tx_json();
    let markup = transaction_row_markup(&body);
    let rows = row_bodies(&markup, "tx-row");
    assert_eq!(rows.len(), 1, "parties tx: {markup}");
    let rate = format!("{}", 584.0f64 / 143.0);
    assert_eq!(cell(&rows[0], "fee-rate"), rate, "row {}", rows[0]);
    assert_eq!(cell(&rows[0], "version"), "2");
    assert_eq!(cell(&rows[0], "locktime"), "0");
    assert_eq!(cell(&rows[0], "sigops"), "1");
    assert!(
        markup.contains("href=\"/tx/prev\""),
        "previous outpoint is not a tx link: {markup}"
    );
    assert!(
        markup.contains("class=\"vout\">0<"),
        "input vout 0 missing: {markup}"
    );
    assert!(markup.contains("bc1qinput"), "{markup}");
    assert!(markup.contains("610677"), "{markup}");
    assert!(markup.contains("bc1qperson"), "{markup}");
    assert!(markup.contains("344697"), "{markup}");
    assert_script_hex_stays_off(&markup);

    let text = transaction_fields_text(&body);
    assert!(
        text.contains(&format!("fee_rate {rate}")),
        "field sentence dropped the fee rate: {text}"
    );
    assert!(text.contains("version 2"), "{text}");
    assert!(text.contains("locktime 0"), "{text}");
    assert!(text.contains("sigops 1"), "{text}");
    assert!(text.contains("outpoint prev"), "{text}");
    assert!(text.contains("vout 0"), "{text}");
    assert!(text.contains("address bc1qinput"), "{text}");
    assert!(text.contains("value 610677"), "{text}");
    assert!(text.contains("address bc1qperson"), "{text}");
    assert!(text.contains("value 344697"), "{text}");
    assert_script_hex_stays_off(&text);
}

#[test]
fn zero_virtual_size_omits_fee_rate() {
    let body = r#"{"txid":"zero-weight","version":1,"locktime":9,"vin":[],"vout":[],"size":0,"weight":0,"sigops":0,"fee":5,"status":{"confirmed":false}}"#;
    let markup = transaction_row_markup(body);
    assert!(
        !markup.contains("fee-rate"),
        "weight 0 invented a fee rate: {markup}"
    );
    assert!(!markup.contains("NaN"), "{markup}");
    assert!(!markup.contains("inf"), "{markup}");
    let text = transaction_fields_text(body);
    assert!(
        !text.contains("fee_rate"),
        "weight 0 field sentence invented a fee rate: {text}"
    );
    assert!(text.contains("version 1"), "{text}");
    assert!(text.contains("locktime 9"), "{text}");
    assert_eq!(cell(&row_bodies(&markup, "tx-row")[0], "version"), "1");
    assert_eq!(cell(&row_bodies(&markup, "tx-row")[0], "locktime"), "9");
    assert_eq!(cell(&row_bodies(&markup, "tx-row")[0], "sigops"), "0");
}

#[test]
fn input_without_prevout_still_links_the_outpoint_and_hides_script() {
    let body = r#"{"txid":"coin","version":1,"locktime":0,"vin":[{"txid":"orphan","vout":3,"prevout":null,"scriptsig":"deadbeef","scriptsig_asm":"OP_CHECKSIG","witness":["wwit"],"is_coinbase":false,"sequence":1}],"vout":[{"scriptpubkey":"51","scriptpubkey_asm":"OP_TRUE","scriptpubkey_type":"op_return","value":9}],"size":10,"weight":8,"sigops":0,"fee":1,"status":{"confirmed":false}}"#;
    let markup = transaction_row_markup(body);
    assert!(
        markup.contains("href=\"/tx/orphan\""),
        "missing prevout dropped the outpoint link: {markup}"
    );
    assert!(
        markup.contains("class=\"vout\">3<"),
        "missing prevout dropped vout: {markup}"
    );
    assert!(
        markup.contains("class=\"value\">9<"),
        "output value missing: {markup}"
    );
    assert!(!markup.contains("deadbeef"), "{markup}");
    assert!(!markup.contains("OP_CHECKSIG"), "{markup}");
    assert!(!markup.contains("OP_TRUE"), "{markup}");
    assert!(!markup.contains(">51<"), "{markup}");
    assert!(!markup.contains("wwit"), "{markup}");
    assert!(!markup.contains("witness"), "{markup}");
    let text = transaction_fields_text(body);
    assert!(text.contains("outpoint orphan"), "{text}");
    assert!(text.contains("vout 3"), "{text}");
    assert!(text.contains("value 9"), "{text}");
    assert!(!text.contains("deadbeef"), "{text}");
    assert!(!text.contains("witness"), "{text}");
}

#[test]
fn dashboard_recent_row_links_txid_and_rates_fee_over_vsize() {
    let recent = format!(r#"[{{"txid":"{TXID_A}","fee":12345,"vsize":222,"value":99999}}]"#);
    let markup = dashboard_rows_markup("[]", &recent);
    let rows = row_bodies(&markup, "tx-row");
    assert_eq!(rows.len(), 1, "recent: {markup}");
    assert_eq!(cell(&rows[0], "txid"), txid_link(TXID_A));
    assert_eq!(cell(&rows[0], "fee"), "12345");
    assert_eq!(cell(&rows[0], "vsize"), "222");
    assert_eq!(cell(&rows[0], "value"), "99999");
    let rate = format!("{}", 12345.0f64 / 222.0);
    assert_eq!(cell(&rows[0], "fee-rate"), rate, "row {}", rows[0]);
    let text = dashboard_fields_text("[]", &recent);
    assert!(
        text.contains(&format!("recent {TXID_A} fee 12345 vsize 222 value 99999")),
        "{text}"
    );
    assert!(text.contains(&format!("fee_rate {rate}")), "{text}");
}

#[test]
fn recent_vsize_zero_omits_fee_rate() {
    let recent = r#"[{"txid":"no-rate","fee":5,"vsize":0,"value":1}]"#;
    let markup = dashboard_rows_markup("[]", recent);
    assert!(
        !markup.contains("fee-rate"),
        "vsize 0 invented a fee rate: {markup}"
    );
    assert_eq!(
        cell(&row_bodies(&markup, "tx-row")[0], "txid"),
        txid_link("no-rate")
    );
    let text = dashboard_fields_text("[]", recent);
    assert!(!text.contains("fee_rate"), "{text}");
    assert!(
        text.contains("recent no-rate fee 5 vsize 0 value 1"),
        "{text}"
    );
}

#[test]
fn address_stats_show_every_chain_and_mempool_count_and_sum() {
    let stats = address_stats_json();
    let utxo = format!(
        r#"[{{"txid":"{TXID_B}","vout":1,"value":42000,"status":{{"confirmed":true,"block_height":842001}}}}]"#
    );
    let markup = address_rows_markup(&stats, "[]", &utxo);
    assert_eq!(element_text(&markup, "funded-sum"), "5000");
    assert_eq!(markup.matches("<p class=\"funded-sum\">").count(), 1);
    assert_eq!(element_text(&markup, "chain-tx-count"), "4");
    assert_eq!(element_text(&markup, "chain-funded-txo-count"), "4");
    assert_eq!(element_text(&markup, "chain-spent-txo-count"), "1");
    assert_eq!(element_text(&markup, "chain-spent-sum"), "1000");
    assert_eq!(element_text(&markup, "mempool-tx-count"), "1");
    assert_eq!(element_text(&markup, "mempool-funded-sum"), "20");
    assert_eq!(element_text(&markup, "mempool-funded-txo-count"), "1");
    assert_eq!(element_text(&markup, "mempool-spent-txo-count"), "0");
    assert_eq!(element_text(&markup, "mempool-spent-sum"), "0");
    let utxo_rows = row_bodies(&markup, "utxo-row");
    assert_eq!(utxo_rows.len(), 1, "utxo: {markup}");
    assert_eq!(cell(&utxo_rows[0], "txid"), txid_link(TXID_B));
    assert_eq!(cell(&utxo_rows[0], "vout"), "1");
    assert_eq!(cell(&utxo_rows[0], "value"), "42000");
    assert_eq!(cell(&utxo_rows[0], "confirmed"), "true");

    let text = address_fields_text(&stats, "", "");
    assert!(text.contains("chain_tx_count 4"), "{text}");
    assert!(text.contains("funded_txo_sum 5000"), "{text}");
    assert!(text.contains("mempool_tx_count 1"), "{text}");
    assert!(text.contains("mempool_funded_txo_sum 20"), "{text}");
    assert!(text.contains("chain_funded_txo_count 4"), "{text}");
    assert!(text.contains("chain_spent_txo_count 1"), "{text}");
    assert!(text.contains("chain_spent_txo_sum 1000"), "{text}");
    assert!(text.contains("mempool_funded_txo_count 1"), "{text}");
    assert!(text.contains("mempool_spent_txo_count 0"), "{text}");
    assert!(text.contains("mempool_spent_txo_sum 0"), "{text}");
}

#[test]
fn views_keep_fee_targets_and_the_missing_incoming_rate_sentence() {
    let src = views_source();
    assert!(
        src.contains("The incoming virtual-bytes-per-second rate is not in the payload."),
        "views.rs dropped the incoming virtual-bytes-per-second sentence"
    );
    assert!(
        src.contains("fee target"),
        "views.rs dropped the fee target line"
    );
    // Same body as GPUI `screen_paints_transaction_fees_heading`: `{"2":1.1}`.
    // The whole line is the painter output, not a substring of `Fee rate`.
    let painted = dashboard_tip_fees_text("abc", "1", r#"{"2":1.1}"#);
    assert!(
        painted.lines().any(|line| line == "fee target 2 rate 1.1"),
        "no line equals fee target 2 rate 1.1: {painted}"
    );
    let block_page = view_fn_body("BlockPage");
    assert!(
        !block_page.contains("<pre>{move || block_txids_fields_text"),
        "block txids are still escaped plain text inside <pre>"
    );
    assert!(
        block_page.contains("block_txid_links_markup("),
        "BlockPage does not render each block txid as a link"
    );
    assert!(
        block_page.contains("block_rows_markup("),
        "BlockPage dropped the transaction rows"
    );
    let mempool = view_fn_body("MempoolPage");
    assert!(
        mempool.contains("mempool_summary_text("),
        "MempoolPage dropped mempool_summary_text"
    );
    assert!(
        mempool.contains("recent_rows_markup("),
        "MempoolPage does not render recent rows"
    );
}

#[test]
fn broadcast_sentence_stays_txid_without_a_link() {
    let text = broadcast_fields_text("abc");
    assert_eq!(text, "txid abc");
    assert!(!text.contains("/tx"), "{text}");
}
