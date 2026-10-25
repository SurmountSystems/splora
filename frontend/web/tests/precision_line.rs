//! Precision line on an issued asset from `GET /asset/:id`.
//!
//! The GPUI screen paints the exact line `Precision 2` when precision is 2.
//! The Angular Precision cell stays. A missing precision and precision 8 omit
//! that line. The path has no query string.

use splora_web::{asset_markup, asset_requests};

const ISSUED: &str = "ac3e0ff248c5051ffd61e00155b7122e5ebc04fd397a0ecbdd4f4e4a56232926";
const ISSUANCE_TX: &str = "cdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcd";
const PREV_TX: &str = "efefefefefefefefefefefefefefefefefefefefefefefefefefefefefefefef";
const REISSUE: &str = "1111111111111111111111111111111111111111111111111111111111111111";
const LINE: &str = "Precision 2";

fn document(precision: Option<u8>) -> String {
    let precision_field = match precision {
        Some(precision) => format!(r#""precision":{precision},"#),
        None => String::new(),
    };
    format!(
        r#"{{"asset_id":"{ISSUED}","name":"Test Asset","ticker":"TEST",{precision_field}"entity":{{"domain":"example.com"}},"contract":null,"issuance_txin":{{"txid":"{ISSUANCE_TX}","vin":0}},"issuance_prevout":{{"txid":"{PREV_TX}","vout":1}},"reissuance_token":"{REISSUE}","status":{{"confirmed":true}},"chain_stats":{{"tx_count":3,"issuance_count":1,"issued_amount":5000,"burned_amount":200,"has_blinded_issuances":false,"burned_reissuance_tokens":0}},"mempool_stats":{{"tx_count":0,"issuance_count":0,"issued_amount":0,"burned_amount":0,"has_blinded_issuances":false,"burned_reissuance_tokens":0}}}}"#
    )
}

fn paint(precision: Option<u8>) -> String {
    let reads = asset_requests(true, ISSUED);
    assert!(
        reads
            .iter()
            .any(|req| req.method == "GET" && req.path == format!("/asset/{ISSUED}")),
        "{reads:?}"
    );
    assert!(
        reads
            .iter()
            .all(|req| !req.path.contains('?') && !req.path.contains('#')),
        "{reads:?}"
    );
    asset_markup(&document(precision), "[]", true)
}

#[test]
fn issued_asset_paints_precision_2() {
    let markup = paint(Some(2));
    assert_eq!(markup.matches(LINE).count(), 1, "{markup}");
    assert!(
        markup.lines().any(|line| line == LINE),
        "no line equals {LINE}: {markup}"
    );
    assert!(
        markup.contains("<tr><td>Precision</td><td>2</td></tr>"),
        "{markup}"
    );

    let eight = paint(Some(8));
    assert!(!eight.contains(LINE), "{eight}");

    let missing = paint(None);
    assert!(!missing.contains(LINE), "{missing}");
    assert!(!missing.contains("<td>Precision</td>"), "{missing}");
}
