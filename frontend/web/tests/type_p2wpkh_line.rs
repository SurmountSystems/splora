//! Whole line `Type P2WPKH` on the address page from `GET /address/:addr`.
//!
//! GPUI `address_screen_paints_p2wpkh_badge` requires one line whose entire
//! text equals `Type P2WPKH`. The address JSON has no type field. The badge
//! is the address string. The address-type paragraph stays. That paragraph
//! is a longer line. It is not this sentence. A different address shape
//! omits the line. An empty body and a body with no address omit the line.

use splora_web::address_rows_markup;

const LINE: &str = "Type P2WPKH";
const ADDR: &str = "bc1q8yj0herd4r4yxszw3nkfvt53433thk0f5qst4g";
const PARAGRAPH: &str = "<p class=\"address-type\"><span class=\"address-type-label\">Type</span><span class=\"badge bg-primary\"><span>P2WPKH</span></span></p>";

fn stats(address: &str) -> String {
    format!(
        r#"{{"address":"{address}","chain_stats":{{"tx_count":1,"funded_txo_count":1,"spent_txo_count":0,"funded_txo_sum":10,"spent_txo_sum":0}},"mempool_stats":{{"tx_count":0,"funded_txo_count":0,"spent_txo_count":0,"funded_txo_sum":0,"spent_txo_sum":0}}}}"#
    )
}

fn paint(body: &str) -> String {
    address_rows_markup(body, "", "")
}

#[test]
fn address_page_paints_type_p2wpkh_as_its_own_line() {
    let body = stats(ADDR);
    assert!(
        !body.contains("\"type\""),
        "address JSON must not carry a type field: {body}"
    );
    let markup = paint(&body);
    assert!(
        markup.contains(PARAGRAPH),
        "the address-type paragraph was dropped: {markup}"
    );
    assert!(
        markup
            .lines()
            .filter(|line| line.contains(PARAGRAPH))
            .all(|line| line != LINE),
        "the address-type paragraph line is the whole sentence: {markup}"
    );
    assert!(
        markup.lines().any(|line| line == LINE),
        "no line equals {LINE}: {markup}"
    );
    assert_eq!(
        markup.lines().filter(|line| *line == LINE).count(),
        1,
        "{LINE} was not painted once: {markup}"
    );
    assert!(
        !markup.contains("V0_P2WPKH"),
        "address badge was uppercased into the output type: {markup}"
    );
    for absent in ["P2PKH", "P2SH", "P2TR", "P2WSH"] {
        assert!(
            !markup.contains(absent),
            "{absent} was painted on the p2wpkh address: {markup}"
        );
    }

    let other = paint(&stats("bc1qnotp2wpkhshape"));
    assert!(
        !other.lines().any(|line| line == LINE),
        "a different address shape painted {LINE}: {other}"
    );
    assert!(
        !other.contains("P2WPKH"),
        "a different address shape painted P2WPKH: {other}"
    );
    assert!(
        !other.lines().any(|line| line.starts_with("Type ")),
        "a different address shape painted a type badge: {other}"
    );

    let empty = paint("");
    assert!(
        !empty.lines().any(|line| line == LINE),
        "an empty body painted {LINE}: {empty}"
    );
    assert!(
        !empty.contains("P2WPKH"),
        "an empty body painted P2WPKH: {empty}"
    );

    let missing_address = paint(
        r#"{"chain_stats":{"tx_count":1,"funded_txo_count":1,"spent_txo_count":0,"funded_txo_sum":10,"spent_txo_sum":0},"mempool_stats":{"tx_count":0,"funded_txo_count":0,"spent_txo_count":0,"funded_txo_sum":0,"spent_txo_sum":0}}"#,
    );
    assert!(
        !missing_address.lines().any(|line| line == LINE),
        "a missing address painted {LINE}: {missing_address}"
    );
    assert!(
        !missing_address.contains("P2WPKH"),
        "a missing address painted P2WPKH: {missing_address}"
    );
}
