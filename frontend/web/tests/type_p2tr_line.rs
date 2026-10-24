//! Whole line `Type P2TR` on the address page from `GET /address/:addr`.
//!
//! GPUI `address_screen_paints_p2tr_badge` requires one line whose entire
//! text equals `Type P2TR`. The address JSON has no type field. The badge
//! is the address string. The address-type paragraph stays. That paragraph
//! is a longer line. It is not this sentence. A `bc1q` pay-to-witness-pubkey-hash
//! stays `Type P2WPKH` and does not paint `P2TR`. A longer `bc1q` stays
//! `Type P2WSH` and does not paint `P2TR`. `P2TR tapscript` and `V1_P2TR`
//! stay off the address badge. An address that fails `mainnet_v1_p2tr`
//! omits the line.

use splora_web::address_rows_markup;

const LINE: &str = "Type P2TR";
const ADDR: &str = "bc1pahzjr2vnfqnqwl0h8jps2aa4zqe29wqxn2nagvd5a2z982k7ttdq8ry370";
const WPKH: &str = "bc1q8yj0herd4r4yxszw3nkfvt53433thk0f5qst4g";
const WSH: &str = "bc1qjasf9z3h7w3jspkhtgatgpyvvzgpa2wwd2lr0eh5tx44reyn2k7sfc27a4";
const PARAGRAPH: &str = "<p class=\"address-type\"><span class=\"address-type-label\">Type</span><span class=\"badge bg-primary\"><span>P2TR</span></span></p>";

fn stats(address: &str) -> String {
    format!(
        r#"{{"address":"{address}","chain_stats":{{"tx_count":1,"funded_txo_count":1,"spent_txo_count":0,"funded_txo_sum":10,"spent_txo_sum":0}},"mempool_stats":{{"tx_count":0,"funded_txo_count":0,"spent_txo_count":0,"funded_txo_sum":0,"spent_txo_sum":0}}}}"#
    )
}

fn paint(body: &str) -> String {
    address_rows_markup(body, "", "")
}

#[test]
fn address_page_paints_type_p2tr_as_its_own_line() {
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
        !markup.contains("P2TR tapscript"),
        "the address badge was the tapscript label: {markup}"
    );
    assert!(
        !markup.contains("V1_P2TR"),
        "address badge was uppercased into the output type: {markup}"
    );

    let short = paint(&stats(WPKH));
    assert!(
        short.lines().any(|line| line == "Type P2WPKH"),
        "the shorter bc1q address dropped Type P2WPKH: {short}"
    );
    assert!(
        !short.lines().any(|line| line == LINE),
        "the shorter bc1q address painted {LINE}: {short}"
    );
    assert!(
        !short.contains("P2TR"),
        "the shorter bc1q address painted P2TR: {short}"
    );

    let script = paint(&stats(WSH));
    assert!(
        script.lines().any(|line| line == "Type P2WSH"),
        "the longer bc1q address dropped Type P2WSH: {script}"
    );
    assert!(
        !script.lines().any(|line| line == LINE),
        "the longer bc1q address painted {LINE}: {script}"
    );
    assert!(
        !script.contains("P2TR"),
        "the longer bc1q address painted P2TR: {script}"
    );

    let empty = paint("");
    assert!(
        !empty.lines().any(|line| line == LINE),
        "an empty body painted {LINE}: {empty}"
    );
    assert!(
        !empty.contains("P2TR"),
        "an empty body painted P2TR: {empty}"
    );

    let missing_address = paint(
        r#"{"chain_stats":{"tx_count":1,"funded_txo_count":1,"spent_txo_count":0,"funded_txo_sum":10,"spent_txo_sum":0},"mempool_stats":{"tx_count":0,"funded_txo_count":0,"spent_txo_count":0,"funded_txo_sum":0,"spent_txo_sum":0}}"#,
    );
    assert!(
        !missing_address.lines().any(|line| line == LINE),
        "a missing address painted {LINE}: {missing_address}"
    );
    assert!(
        !missing_address.contains("P2TR"),
        "a missing address painted P2TR: {missing_address}"
    );
}
