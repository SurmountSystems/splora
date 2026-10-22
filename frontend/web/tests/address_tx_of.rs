//! Heading above the loaded address and scripthash transaction list.
//!
//! Angular `address.component.html` paints an `h2` of class `text-start`
//! inside `div.title-tx`. One loaded transaction is
//! `{length} of {mempool tx_count + chain tx_count} transaction`.
//! More than one is the same sentence with `transactions`. The `| number`
//! pipe is en-US grouping, so 1234 is `1,234`. An empty loaded list is
//! `&nbsp;` and is not a count. `Transactions {sum}` stays the separate
//! paragraph. The first page is `GET /address/:addr/txs` or
//! `GET /scripthash/:hash/txs` with no `after_txid`, no question mark,
//! and no hash fragment.

use splora_web::address_rows_markup;

fn stats(key: &str, value: &str, chain_tx: u64, mempool_tx: u64) -> String {
    format!(
        r#"{{"{key}":"{value}","chain_stats":{{"tx_count":{chain_tx},"funded_txo_count":4,"spent_txo_count":1,"funded_txo_sum":5000,"spent_txo_sum":1000}},"mempool_stats":{{"tx_count":{mempool_tx},"funded_txo_count":2,"spent_txo_count":1,"funded_txo_sum":20,"spent_txo_sum":7}}}}"#
    )
}

fn tx(index: u64) -> String {
    let txid = format!("{:064x}", index + 1);
    format!(
        r#"{{"txid":"{txid}","version":1,"locktime":0,"vin":[],"vout":[],"size":10,"weight":40,"sigops":0,"fee":1,"status":{{"confirmed":true,"block_height":2}}}}"#
    )
}

fn txs(count: u64) -> String {
    let body = (0..count).map(tx).collect::<Vec<_>>().join(",");
    format!("[{body}]")
}

fn heading(sentence: &str) -> String {
    format!("<div class=\"title-tx\"><h2 class=\"text-start\">{sentence}</h2></div>")
}

fn assert_heading_above_list(markup: &str, sentence: &str, list_open: &str) {
    let block = heading(sentence);
    assert_eq!(markup.matches(&block).count(), 1, "{markup}");
    assert_eq!(markup.matches("<h2").count(), 1, "{markup}");
    assert!(!sentence.contains('?'), "{sentence}");
    assert!(!sentence.contains('#'), "{sentence}");
    assert!(!block.contains("after_txid"), "{block}");
    let at = markup
        .find(&block)
        .unwrap_or_else(|| panic!("missing {block} in {markup}"));
    let rest = &markup[at + block.len()..];
    assert!(
        rest.starts_with(list_open),
        "heading is not immediately above the loaded list: {markup}"
    );
}

fn assert_transactions_sentence(markup: &str, sum: u128) {
    let paragraph = format!("<p class=\"transactions\">Transactions {sum}</p>");
    assert_eq!(markup.matches(&paragraph).count(), 1, "{markup}");
}

fn assert_both(txs_json: &str, sentence: &str, list_open: &str, chain: u64, mempool: u64) {
    let sum = chain as u128 + mempool as u128;
    let address = address_rows_markup(
        &stats("address", "bc1qexample", chain, mempool),
        txs_json,
        "",
    );
    let scripthash = address_rows_markup(
        &stats("scripthash", "deadbeef", chain, mempool),
        txs_json,
        "",
    );
    assert!(address.contains("bc1qexample"), "{address}");
    assert!(
        scripthash.contains("<p class=\"scripthash\">deadbeef</p>"),
        "{scripthash}"
    );
    assert!(!scripthash.contains("bc1qexample"), "{scripthash}");
    for markup in [&address, &scripthash] {
        assert_heading_above_list(markup, sentence, list_open);
        assert_transactions_sentence(markup, sum);
    }
}

#[test]
fn one_loaded_transaction_paints_the_singular_of_sentence() {
    let chain = 4u64;
    let mempool = 1u64;
    let sum = chain + mempool;
    assert_both(
        &txs(1),
        &format!("1 of {sum} transaction"),
        "<table class=\"tx-rows\">",
        chain,
        mempool,
    );
}

#[test]
fn several_loaded_transactions_paint_the_plural_of_sentence() {
    assert_both(
        &txs(2),
        "2 of 1,234 transactions",
        "<table class=\"tx-rows\">",
        1_000,
        234,
    );
}

#[test]
fn empty_loaded_list_is_a_non_breaking_space() {
    let chain = 4u64;
    let mempool = 1u64;
    let address = address_rows_markup(&stats("address", "bc1qexample", chain, mempool), "[]", "");
    let scripthash =
        address_rows_markup(&stats("scripthash", "deadbeef", chain, mempool), "[]", "");
    for markup in [&address, &scripthash] {
        assert_heading_above_list(markup, "&nbsp;", "<p class=\"empty\">");
        assert!(!markup.contains(" of "), "{markup}");
        assert!(!markup.contains("0 of"), "{markup}");
        assert!(!markup.contains("0 transaction"), "{markup}");
        assert_transactions_sentence(markup, chain as u128 + mempool as u128);
        assert!(
            markup.contains("<p class=\"empty\">There are no transactions.</p>"),
            "{markup}"
        );
    }
}

#[test]
fn omitted_transaction_list_does_not_invent_a_count() {
    let address = address_rows_markup(&stats("address", "bc1qexample", 4, 1), "", "");
    let scripthash = address_rows_markup(&stats("scripthash", "deadbeef", 4, 1), "", "");
    for markup in [&address, &scripthash] {
        assert!(!markup.contains(" of "), "{markup}");
        assert!(!markup.contains("&nbsp;"), "{markup}");
        assert!(!markup.contains("<h2"), "{markup}");
        assert_transactions_sentence(markup, 5);
    }
}
