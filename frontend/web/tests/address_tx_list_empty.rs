//! Empty address transaction list from no-query `GET /address/:addr/txs`.
//!
//! The GPUI line is `The address transaction list is empty.` when that
//! array is empty. `There are no transactions.` stays. A loaded row does
//! not gain this sentence. An empty utxo list does not gain it. A blank
//! body and invalid JSON do not gain it. Scripthash uses the same sentence
//! on `GET /scripthash/:hash/txs`.

use splora_web::{address_requests, address_rows_markup};

const ADDR: &str = "bc1qexample";
const HASH: &str = "abababababababababababababababababababababababababababababababab";
const SENTENCE: &str = "The address transaction list is empty.";
const MARK: &str = "<p class=\"address-tx-empty\">The address transaction list is empty.</p>";
const OLD: &str = "<p class=\"empty\">There are no transactions.</p>";

fn stats(key: &str, value: &str) -> String {
    format!(
        r#"{{"{key}":"{value}","chain_stats":{{"tx_count":4,"funded_txo_count":1,"spent_txo_count":0,"funded_txo_sum":10,"spent_txo_sum":0}},"mempool_stats":{{"tx_count":3,"funded_txo_count":0,"spent_txo_count":0,"funded_txo_sum":0,"spent_txo_sum":0}}}}"#
    )
}

fn one_tx() -> String {
    r#"[{"txid":"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa","version":1,"locktime":0,"vin":[],"vout":[],"size":10,"weight":40,"sigops":0,"fee":4,"status":{"confirmed":true,"block_height":1,"block_time":1700000000}}]"#
        .to_string()
}

fn assert_once(markup: &str) {
    assert_eq!(markup.matches(SENTENCE).count(), 1, "{markup}");
    assert_eq!(markup.matches(MARK).count(), 1, "{markup}");
    assert!(markup.contains(OLD), "{markup}");
    let old_at = markup.find(OLD).expect("old empty paragraph");
    let mark_at = markup.find(MARK).expect("empty sentence");
    assert!(
        old_at + OLD.len() == mark_at,
        "the sentence does not follow the older empty paragraph: {markup}"
    );
}

fn assert_absent(markup: &str) {
    assert!(
        !markup.contains(SENTENCE),
        "the empty sentence leaked: {markup}"
    );
    assert!(!markup.contains("address-tx-empty"), "{markup}");
}

#[test]
fn empty_address_transaction_list_paints_the_sentence() {
    let requests = address_requests(ADDR);
    assert!(
        requests
            .iter()
            .any(|req| req.method == "GET" && req.path == format!("/address/{ADDR}/txs")),
        "{requests:?}"
    );
    assert!(
        requests
            .iter()
            .all(|req| !req.path.contains('?') && !req.path.contains('#')),
        "{requests:?}"
    );
    let hash_requests = address_requests(HASH);
    assert!(
        hash_requests
            .iter()
            .any(|req| { req.method == "GET" && req.path == format!("/scripthash/{HASH}/txs") }),
        "{hash_requests:?}"
    );

    assert_once(&address_rows_markup(&stats("address", ADDR), "[]", ""));
    assert_once(&address_rows_markup(&stats("address", ADDR), "[ ]", "[]"));
    assert_once(&address_rows_markup("", "[]", ""));
    assert_once(&address_rows_markup(&stats("scripthash", HASH), "[]", ""));

    assert_absent(&address_rows_markup(
        &stats("address", ADDR),
        &one_tx(),
        "[]",
    ));
    assert_absent(&address_rows_markup(&stats("address", ADDR), "", "[]"));
    assert_absent(&address_rows_markup(
        &stats("address", ADDR),
        "not-json",
        "",
    ));
    assert_absent(&address_rows_markup("", "", "[]"));
}
