//! Whole line `(Newly Generated Coins)` on a coinbase input from `GET /tx/:txid`.
//!
//! GPUI `transaction_screen_paints_newly_generated_coins` requires one line
//! whose entire text is `(Newly Generated Coins)` when `is_coinbase` is true.
//! The lowercase coinbase span stays glued to the parenthetical span. A false
//! coinbase omits the line. An empty body omits the line. A span that contains
//! the words is not that line.

use splora_web::{transaction_requests, transaction_row_markup};

const TXID: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
const PREV: &str = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
const LINE: &str = "(Newly Generated Coins)";
const NEWLY_SPAN: &str = "<span class=\"newly-generated-coins\">(Newly Generated Coins)</span>";
const ADJACENT: &str = "<span class=\"coinbase\">coinbase</span><span class=\"newly-generated-coins\">(Newly Generated Coins)</span>";

fn tx(is_coinbase: bool) -> String {
    format!(
        r#"{{"txid":"{TXID}","version":1,"locktime":0,"vin":[{{"txid":"{PREV}","vout":0,"prevout":null,"scriptsig":"","scriptsig_asm":"","is_coinbase":{is_coinbase},"sequence":0}}],"vout":[{{"scriptpubkey":"51","scriptpubkey_asm":"OP_TRUE","scriptpubkey_type":"p2pkh","value":1}}],"size":10,"weight":40,"sigops":0,"fee":1}}"#
    )
}

fn paint(body: &str) -> String {
    let requests = transaction_requests(TXID);
    assert_eq!(requests.len(), 1, "{requests:?}");
    assert_eq!(requests[0].method, "GET");
    assert_eq!(requests[0].path, format!("/tx/{TXID}"));
    assert!(!requests[0].path.contains('?'));
    assert!(!requests[0].path.contains('#'));
    transaction_row_markup(body)
}

#[test]
fn coinbase_input_paints_newly_generated_coins_as_its_own_line() {
    let markup = paint(&tx(true));
    assert!(
        markup.lines().any(|line| line == LINE),
        "no line equals (Newly Generated Coins): {markup}"
    );
    assert_eq!(
        markup.lines().filter(|line| *line == LINE).count(),
        1,
        "is_coinbase true did not paint the exact line (Newly Generated Coins) once: {markup}"
    );
    assert!(
        markup
            .lines()
            .any(|line| line == LINE && !line.contains('<')),
        "(Newly Generated Coins) was inside a span: {markup}"
    );
    assert_eq!(markup.matches(NEWLY_SPAN).count(), 1, "{markup}");
    assert_eq!(markup.matches(ADJACENT).count(), 1, "{markup}");
    assert!(
        markup.lines().any(|line| line == "coinbase"),
        "the lowercase coinbase line was dropped: {markup}"
    );
    assert!(
        markup.lines().any(|line| line == "input coinbase"),
        "input coinbase was dropped: {markup}"
    );

    let ordinary = paint(&tx(false));
    assert!(
        !ordinary.lines().any(|line| line == LINE),
        "a false coinbase painted {LINE}: {ordinary}"
    );
    assert!(
        !ordinary.contains(LINE),
        "a false coinbase contains {LINE}: {ordinary}"
    );

    let empty = transaction_row_markup("");
    assert!(
        !empty.lines().any(|line| line == LINE),
        "an empty body painted {LINE}: {empty}"
    );
}
