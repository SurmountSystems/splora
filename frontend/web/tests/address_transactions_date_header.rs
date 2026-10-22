//! Address transactions column header from the address transactions widget.
//!
//! Angular `address-transactions-widget.component.html` paints
//! `<th class="table-cell-date">Date</th>` on that table. The word is the
//! column header. It is not a formatted clock and not a relative duration.
//! The address route has no query string. An empty list, a blank body, and
//! invalid JSON do not paint the header. The blocks list and the recent
//! transactions table do not gain it.

use splora_web::{address_rows_markup, block_rows_markup, recent_rows_markup};

const HEADER: &str = "<th class=\"table-cell-date\">Date</th>";
const TXID: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
const BLOCK_ID: &str = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";

fn one_tx(fee: u64) -> String {
    format!(
        r#"{{"txid":"{TXID}","version":1,"locktime":0,"vin":[],"vout":[],"size":10,"weight":40,"sigops":0,"fee":{fee},"status":{{"confirmed":true,"block_height":1,"block_time":1700000000}}}}"#
    )
}

fn txs(fees: &[u64]) -> String {
    let rows: Vec<String> = fees.iter().copied().map(one_tx).collect();
    format!("[{}]", rows.join(","))
}

fn block_json() -> String {
    format!(
        r#"{{"id":"{BLOCK_ID}","height":1,"version":1,"timestamp":1,"tx_count":1,"size":1234,"weight":5678,"merkle_root":"merkle","mediantime":1}}"#
    )
}

fn recent_json() -> String {
    r#"[{"txid":"abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789","fee":10,"vsize":141,"value":1}]"#
        .to_string()
}

fn utxo_json() -> String {
    format!(
        r#"[{{"txid":"{TXID}","vout":0,"value":1,"status":{{"confirmed":true,"block_height":1}}}}]"#
    )
}

fn assert_one(markup: &str) {
    assert_eq!(
        markup.matches(HEADER).count(),
        1,
        "expected one {HEADER} in {markup}"
    );
    assert_eq!(markup.matches("Date").count(), 1, "{markup}");
}

fn assert_absent(markup: &str) {
    assert!(
        !markup.contains(HEADER),
        "{HEADER} leaked into the markup: {markup}"
    );
    assert!(
        !markup.contains("Date"),
        "Date leaked into the markup: {markup}"
    );
}

#[test]
fn one_address_transaction_paints_the_date_header_once() {
    let markup = address_rows_markup("", &txs(&[4]), "");
    assert_one(&markup);
    assert!(
        markup.find(HEADER).unwrap() < markup.find("<tr class=\"tx-row\">").unwrap(),
        "the header belongs before the row: {markup}"
    );
    assert!(
        markup.contains("<td class=\"fee\">4</td>"),
        "the raw fee was dropped: {markup}"
    );
    assert!(
        markup.contains("<td class=\"block-time\">1700000000</td>"),
        "the raw block time was dropped: {markup}"
    );
    assert_absent_fiat(&markup);
}

#[test]
fn two_address_transactions_paint_the_date_header_once() {
    let markup = address_rows_markup("", &txs(&[1, 2]), "");
    assert_one(&markup);
    assert!(markup.contains("<td class=\"fee\">1</td>"), "{markup}");
    assert!(markup.contains("<td class=\"fee\">2</td>"), "{markup}");
    assert_absent_fiat(&markup);
}

#[test]
fn empty_or_unparsed_address_transactions_omit_the_date_header() {
    assert_absent(&address_rows_markup("", "[]", ""));
    assert_absent(&address_rows_markup("", "", ""));
    assert_absent(&address_rows_markup("", "not-json", ""));
    assert_absent(&address_rows_markup("", "[{}]", ""));
    assert_absent(&address_rows_markup("", "", &utxo_json()));
}

#[test]
fn blocks_and_recent_transactions_do_not_gain_the_date_header() {
    let block = block_rows_markup(&block_json(), &txs(&[4]));
    assert!(
        block.contains("<tr class=\"tx-row\">"),
        "the block transaction row was dropped: {block}"
    );
    assert_absent(&block);
    let recent = recent_rows_markup(&recent_json());
    assert!(
        recent.contains("<th class=\"table-cell-txid\">TXID</th>"),
        "the recent header was dropped: {recent}"
    );
    assert_absent(&recent);
}

fn assert_absent_fiat(markup: &str) {
    assert!(!markup.contains('$'), "{markup}");
    assert!(!markup.contains("BTC"), "{markup}");
    assert!(!markup.contains('+'), "{markup}");
}
