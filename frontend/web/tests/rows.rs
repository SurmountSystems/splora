//! Row markup for the dashboard, blocks, block, transaction, and address views.
//!
//! One `<tr>` per block or transaction. A flat sentence in a `<pre>` is not a row.

use splora_web::{
    address_fields_text, address_rows_markup, block_rows_markup, blocks_rows_markup,
    dashboard_rows_markup, transaction_row_markup,
};

const BLOCK_HASH: &str = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
const BLOCK_PREV: &str = "cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc";
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

fn assert_cell(row: &str, class: &str, value: &str) {
    assert_eq!(cell(row, class), value, "cell {class} in {row}");
}

fn txid_link(id: &str) -> String {
    format!("<a href=\"/tx/{id}\">{id}</a>")
}

fn block_link(hash: &str) -> String {
    format!("<a href=\"/block/{hash}\">{hash}</a>")
}

fn height_on_block(height: &str, hash: &str) -> String {
    format!("<a href=\"/block/{hash}\">{height}</a>")
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

fn assert_not_flat_sentence(markup: &str) {
    assert!(
        !markup.contains(" height "),
        "markup is still one flat sentence: {markup}"
    );
    assert!(
        !markup.contains("tx_count "),
        "markup is still one flat sentence: {markup}"
    );
    assert!(
        !markup.contains(" fee "),
        "markup is still one flat sentence: {markup}"
    );
    assert!(
        !markup.contains("<pre>"),
        "markup is still one flat sentence: {markup}"
    );
}

fn blocks_json() -> String {
    format!(
        r#"[{{"id":"{BLOCK_HASH}","height":842001,"version":1,"timestamp":1700000000,"tx_count":17,"size":1234,"weight":5678,"merkle_root":"merkle","previousblockhash":"{BLOCK_PREV}","mediantime":1699999000,"nonce":1,"bits":1,"difficulty":1}},{{"id":"{BLOCK_PREV}","height":842000,"version":1,"timestamp":1699990000,"tx_count":3,"size":100,"weight":400,"merkle_root":"merkle","previousblockhash":"{TXID_A}","mediantime":1699980000,"nonce":1,"bits":1,"difficulty":1}}]"#
    )
}

fn recent_json() -> String {
    format!(
        r#"[{{"txid":"{TXID_A}","fee":12345,"vsize":222,"value":99999}},{{"txid":"{TXID_B}","fee":50,"vsize":110,"value":10}}]"#
    )
}

fn block_json() -> String {
    format!(
        r#"{{"id":"{BLOCK_HASH}","height":842001,"version":1,"timestamp":1700000000,"tx_count":17,"size":1234,"weight":5678,"merkle_root":"merkle","previousblockhash":"{BLOCK_PREV}","mediantime":1699999000,"nonce":1,"bits":1,"difficulty":1}}"#
    )
}

fn txs_json() -> String {
    format!(
        r#"[{{"txid":"{TXID_B}","version":1,"locktime":0,"vin":[],"vout":[],"size":200,"weight":800,"sigops":0,"fee":9,"status":{{"confirmed":true,"block_height":842001,"block_hash":"{BLOCK_HASH}","block_time":1700000000}}}},{{"txid":"{TXID_A}","version":1,"locktime":0,"vin":[],"vout":[],"size":80,"weight":320,"sigops":0,"fee":4,"status":{{"confirmed":false}}}}]"#
    )
}

fn one_tx_json() -> String {
    format!(
        r#"{{"txid":"{TXID_A}","version":2,"locktime":0,"vin":[],"vout":[],"size":111,"weight":444,"sigops":0,"fee":9,"status":{{"confirmed":false}}}}"#
    )
}

fn tx_with_parties_json() -> String {
    format!(
        r#"{{"txid":"{TXID_A}","version":2,"locktime":0,"vin":[{{"txid":"prev","vout":0,"prevout":{{"scriptpubkey":"0014aa","scriptpubkey_asm":"OP_0","scriptpubkey_type":"v0_p2wpkh","scriptpubkey_address":"bc1qinput","value":610677}},"scriptsig":"","scriptsig_asm":"","is_coinbase":false,"sequence":4294967295}}],"vout":[{{"scriptpubkey":"76a914bb88ac","scriptpubkey_asm":"OP_DUP","scriptpubkey_type":"p2pkh","scriptpubkey_address":"bc1qperson","value":344697}}],"size":224,"weight":572,"sigops":1,"fee":584,"status":{{"confirmed":true,"block_height":800000,"block_hash":"{BLOCK_HASH}","block_time":1600000000}}}}"#
    )
}

fn address_stats_json() -> String {
    r#"{"address":"bc1qexample","chain_stats":{"tx_count":4,"funded_txo_count":4,"spent_txo_count":1,"funded_txo_sum":5000,"spent_txo_sum":1000},"mempool_stats":{"tx_count":1,"funded_txo_count":1,"spent_txo_count":0,"funded_txo_sum":20,"spent_txo_sum":0}}"#
        .to_string()
}

fn address_utxos_json() -> String {
    format!(
        r#"[{{"txid":"{TXID_B}","vout":1,"value":42000,"status":{{"confirmed":true,"block_height":842001}}}},{{"txid":"{TXID_A}","vout":0,"value":7,"status":{{"confirmed":false}}}}]"#
    )
}

fn assert_view_uses_rows(name: &str, markup_fn: &str, fields_fn: &str) {
    let body = view_fn_body(name);
    assert!(
        body.contains(&format!("{markup_fn}(")),
        "{name} does not render {markup_fn}"
    );
    assert!(
        body.contains("inner_html"),
        "{name} does not place row markup in the view"
    );
    assert!(
        body.contains(&format!("{fields_fn}(")),
        "{name} dropped {fields_fn}"
    );
    assert!(
        body.contains("rows_with_notices("),
        "{name} still places the flat field sentence in the view"
    );
    let flat = format!("<pre>{{move || {fields_fn}(");
    assert!(
        !body.contains(&flat),
        "{name} still renders one flat sentence"
    );
}

#[test]
fn explorer_views_render_one_row_per_block_or_transaction() {
    let dashboard = dashboard_rows_markup(&blocks_json(), &recent_json());
    assert_not_flat_sentence(&dashboard);
    let block_rows = row_bodies(&dashboard, "block-row");
    assert_eq!(block_rows.len(), 2, "dashboard blocks: {dashboard}");
    assert_cell(
        &block_rows[0],
        "height",
        &height_on_block("842001", BLOCK_HASH),
    );
    assert_cell(&block_rows[0], "hash", &block_link(BLOCK_HASH));
    assert_cell(&block_rows[0], "tx-count", "17");
    assert_cell(&block_rows[0], "time", "1700000000");
    assert_cell(
        &block_rows[1],
        "height",
        &height_on_block("842000", BLOCK_PREV),
    );
    assert_cell(&block_rows[1], "hash", &block_link(BLOCK_PREV));
    assert_cell(&block_rows[1], "tx-count", "3");
    assert_cell(&block_rows[1], "time", "1699990000");
    let tx_rows = row_bodies(&dashboard, "tx-row");
    assert_eq!(tx_rows.len(), 2, "dashboard txs: {dashboard}");
    assert_cell(&tx_rows[0], "txid", &txid_link(TXID_A));
    assert_cell(&tx_rows[0], "fee", "12345");
    assert_cell(&tx_rows[1], "txid", &txid_link(TXID_B));
    assert_cell(&tx_rows[1], "fee", "50");

    let blocks = blocks_rows_markup(&blocks_json());
    assert_not_flat_sentence(&blocks);
    let block_rows = row_bodies(&blocks, "block-row");
    assert_eq!(block_rows.len(), 2, "blocks page: {blocks}");
    assert!(
        row_bodies(&blocks, "tx-row").is_empty(),
        "blocks page invented tx rows: {blocks}"
    );
    assert_cell(
        &block_rows[0],
        "height",
        &height_on_block("842001", BLOCK_HASH),
    );
    assert_cell(&block_rows[0], "hash", &block_link(BLOCK_HASH));
    assert_cell(&block_rows[0], "tx-count", "17");
    assert_cell(&block_rows[0], "time", "1700000000");
    assert_cell(
        &block_rows[1],
        "height",
        &height_on_block("842000", BLOCK_PREV),
    );
    assert_cell(&block_rows[1], "hash", &block_link(BLOCK_PREV));
    assert_cell(&block_rows[1], "tx-count", "3");
    assert_cell(&block_rows[1], "time", "1699990000");

    let block = block_rows_markup(&block_json(), &txs_json());
    assert_not_flat_sentence(&block);
    let block_rows = row_bodies(&block, "block-row");
    assert_eq!(block_rows.len(), 1, "block page: {block}");
    assert_cell(
        &block_rows[0],
        "height",
        &height_on_block("842001", BLOCK_HASH),
    );
    assert_cell(&block_rows[0], "hash", &block_link(BLOCK_HASH));
    assert_cell(&block_rows[0], "tx-count", "17");
    assert_cell(&block_rows[0], "time", "1700000000");
    let tx_rows = row_bodies(&block, "tx-row");
    assert_eq!(tx_rows.len(), 2, "block txs: {block}");
    assert_cell(&tx_rows[0], "txid", &txid_link(TXID_B));
    assert_cell(&tx_rows[0], "fee", "9");
    assert_cell(&tx_rows[1], "txid", &txid_link(TXID_A));
    assert_cell(&tx_rows[1], "fee", "4");

    let transaction = transaction_row_markup(&one_tx_json());
    assert_not_flat_sentence(&transaction);
    let tx_rows = row_bodies(&transaction, "tx-row");
    assert_eq!(tx_rows.len(), 1, "transaction page: {transaction}");
    assert_cell(&tx_rows[0], "txid", &txid_link(TXID_A));
    assert_cell(&tx_rows[0], "fee", "9");
    assert!(
        row_bodies(&transaction, "block-row").is_empty(),
        "transaction page invented a block row: {transaction}"
    );

    let with_parties = transaction_row_markup(&tx_with_parties_json());
    let party_rows = row_bodies(&with_parties, "tx-row");
    assert_eq!(party_rows.len(), 1, "parties must stay on one tx row");
    assert_cell(&party_rows[0], "txid", &txid_link(TXID_A));
    assert_cell(&party_rows[0], "fee", "584");
    assert!(
        party_rows[0].contains("bc1qinput"),
        "tx row dropped the input address: {}",
        party_rows[0]
    );
    assert!(
        party_rows[0].contains("610677"),
        "tx row dropped the input value: {}",
        party_rows[0]
    );
    assert!(
        party_rows[0].contains("bc1qperson"),
        "tx row dropped the output address: {}",
        party_rows[0]
    );
    assert!(
        party_rows[0].contains("344697"),
        "tx row dropped the output value: {}",
        party_rows[0]
    );

    let address = address_rows_markup(&address_stats_json(), &txs_json(), &address_utxos_json());
    assert_not_flat_sentence(&address);
    assert_eq!(element_text(&address, "funded-sum"), "5000");
    let utxo_rows = row_bodies(&address, "utxo-row");
    assert_eq!(utxo_rows.len(), 2, "address utxos: {address}");
    assert_cell(&utxo_rows[0], "value", "42000");
    assert_cell(&utxo_rows[1], "value", "7");
    let address_txs = row_bodies(&address, "tx-row");
    assert_eq!(address_txs.len(), 2, "address txs: {address}");
    assert_cell(&address_txs[0], "txid", &txid_link(TXID_B));
    assert_cell(&address_txs[0], "fee", "9");
    assert_cell(&address_txs[1], "txid", &txid_link(TXID_A));
    assert_cell(&address_txs[1], "fee", "4");

    assert!(
        dashboard_rows_markup("not-json", "not-json").is_empty()
            || !dashboard_rows_markup("not-json", "not-json").contains("block-row"),
        "unreadable indexer JSON invented a block row"
    );
    assert!(!dashboard_rows_markup("not-json", "not-json").contains("<tr"));
    assert!(!blocks_rows_markup("not-json").contains("<tr"));
    assert!(!block_rows_markup("not-json", "not-json").contains("<tr"));
    assert!(!transaction_row_markup("not-json").contains("<tr"));
    assert!(!address_rows_markup("not-json", "not-json", "not-json").contains("<tr"));
    assert!(!address_rows_markup("not-json", "", "").contains("funded-sum"));

    assert_view_uses_rows(
        "Dashboard",
        "dashboard_rows_markup",
        "dashboard_fields_text",
    );
    assert_view_uses_rows(
        "BlocksPage",
        "blocks_rows_markup",
        "blocks_list_fields_text",
    );
    assert_view_uses_rows("BlockPage", "block_rows_markup", "block_fields_text");
    assert_view_uses_rows(
        "TransactionPage",
        "transaction_row_markup",
        "transaction_fields_text",
    );
    assert_view_uses_rows("AddressPage", "address_rows_markup", "address_fields_text");
}

#[test]
fn utxo_omitted_value_paints_confidential_blank_or_zero() {
    let commitment = "a".repeat(66);
    let utxos = format!(
        r#"[
            {{"txid":"blind","vout":1,"status":{{"confirmed":false}},"valuecommitment":"{commitment}"}},
            {{"txid":"plain","vout":1,"status":{{"confirmed":false}}}},
            {{"txid":"emptyc","vout":1,"status":{{"confirmed":false}},"valuecommitment":"","assetcommitment":""}},
            {{"txid":"zeroed","vout":1,"status":{{"confirmed":true}},"value":0}}
        ]"#
    );
    let markup = address_rows_markup("", "", &utxos);
    let rows = row_bodies(&markup, "utxo-row");
    assert_eq!(rows.len(), 4, "{markup}");
    assert_eq!(
        cell(&rows[0], "value"),
        "<span class=\"amount\">Confidential</span>"
    );
    assert!(rows[0].contains("Confidential"), "{}", rows[0]);
    assert!(!rows[0].contains(&commitment), "{}", rows[0]);
    assert!(!rows[0].contains('0'), "{}", rows[0]);
    assert!(!rows[0].contains("Some("), "{}", rows[0]);
    assert!(!rows[0].contains("0.001"), "{}", rows[0]);

    assert_eq!(cell(&rows[1], "value"), "");
    assert!(!rows[1].contains("Confidential"), "{}", rows[1]);
    assert!(!rows[1].contains('0'), "{}", rows[1]);

    assert_eq!(cell(&rows[2], "value"), "");
    assert!(!rows[2].contains("Confidential"), "{}", rows[2]);
    assert!(!rows[2].contains('0'), "{}", rows[2]);

    assert_eq!(cell(&rows[3], "value"), "0");
    assert!(!rows[3].contains("Confidential"), "{}", rows[3]);

    let text = address_fields_text("", "", &utxos);
    assert!(
        text.contains("utxo blind vout 1 value Confidential confirmed false"),
        "{text}"
    );
    assert!(
        text.contains("utxo plain vout 1 value  confirmed false"),
        "{text}"
    );
    assert!(
        text.contains("utxo emptyc vout 1 value  confirmed false"),
        "{text}"
    );
    assert!(
        text.contains("utxo zeroed vout 1 value 0 confirmed true"),
        "{text}"
    );
    assert!(!text.contains(&commitment), "{text}");
    assert!(!text.contains("Some("), "{text}");
    assert!(!text.contains("0.001"), "{text}");
}
