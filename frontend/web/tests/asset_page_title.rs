//! Asset page title on the Liquid asset page.
//!
//! Angular `asset.component.html` paints
//! `<h1 i18n="asset|Liquid Asset page title">Asset</h1>` once when the asset
//! document renders. The title is that heading. It is not the registry column
//! `Asset ID` and not the Name row. A pegged asset paints the title and does
//! not paint a Name row. `GET /asset/:id` has no query string.

use splora_web::{asset_markup, asset_page_markup, asset_requests};

const TITLE: &str = "<h1>Asset</h1>";
const NATIVE_ASSET: &str = "6f0279e9ed041c3d710a9f57d0c02928416460c4b722ae3457a11eec381c526d";
const ISSUED: &str = "ac3e0ff248c5051ffd61e00155b7122e5ebc04fd397a0ecbdd4f4e4a56232926";
const ISSUANCE_TX: &str = "cdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcd";
const PREV_TX: &str = "efefefefefefefefefefefefefefefefefefefefefefefefefefefefefefefef";
const REISSUE: &str = "1111111111111111111111111111111111111111111111111111111111111111";

fn issued_document() -> String {
    format!(
        r#"{{"asset_id":"{ISSUED}","name":"Test Asset","ticker":"TEST","precision":2,"entity":{{"domain":"registry.example"}},"contract":null,"issuance_txin":{{"txid":"{ISSUANCE_TX}","vin":0}},"issuance_prevout":{{"txid":"{PREV_TX}","vout":1}},"reissuance_token":"{REISSUE}","status":{{"confirmed":true}},"chain_stats":{{"tx_count":3,"issuance_count":1,"issued_amount":5000,"burned_amount":200,"has_blinded_issuances":false,"burned_reissuance_tokens":0}},"mempool_stats":{{"tx_count":9,"issuance_count":4,"issued_amount":999999,"burned_amount":888888,"has_blinded_issuances":false,"burned_reissuance_tokens":0}}}}"#
    )
}

fn pegged_document() -> String {
    format!(
        r#"{{"asset_id":"{NATIVE_ASSET}","chain_stats":{{"tx_count":4,"peg_in_count":2,"peg_in_amount":1000,"peg_out_count":1,"peg_out_amount":200,"burn_count":1,"burned_amount":50}},"mempool_stats":{{"tx_count":8,"peg_in_count":3,"peg_in_amount":777777,"peg_out_count":2,"peg_out_amount":666666,"burn_count":1,"burned_amount":555555}}}}"#
    )
}

fn page(id: &str, document: &str) -> String {
    let path = format!("/asset/{id}");
    assert!(!path.contains('?') && !path.contains('#'), "{path}");
    asset_page_markup(&path, document, "[]")
        .unwrap_or_else(|| panic!("asset page did not open for {id}"))
}

fn assert_title_once_before(markup: &str, later: &str) {
    assert_eq!(markup.matches(TITLE).count(), 1, "{markup}");
    assert_eq!(markup.matches("<h1>").count(), 1, "{markup}");
    let title_at = markup.find(TITLE).expect("title");
    let later_at = markup
        .find(later)
        .unwrap_or_else(|| panic!("{later} is missing from {markup}"));
    assert!(
        title_at < later_at,
        "title at {title_at} is not before {later} at {later_at}: {markup}"
    );
    assert!(!markup.contains("Asset ID"), "{markup}");
    assert!(!markup.contains("<th>"), "{markup}");
}

fn assert_plain_asset_get(id: &str) {
    let reads = asset_requests(true, id);
    let path = format!("/asset/{id}");
    assert!(
        reads
            .iter()
            .any(|req| req.method == "GET" && req.path == path),
        "GET /asset/:id was not requested: {reads:?}"
    );
    assert!(
        reads
            .iter()
            .all(|req| !req.path.contains('?') && !req.path.contains('#')),
        "asset reads must not carry a query string: {reads:?}"
    );
}

fn asset_page_source() -> &'static str {
    let src = include_str!("../src/views.rs");
    let start = src.find("fn AssetPage()").expect("AssetPage");
    let rest = &src[start..];
    let end = rest.find("fn AssetsPage()").expect("AssetsPage");
    &rest[..end]
}

/// Issued document paints `Asset` once, before the asset id and before the Name row.
#[test]
fn issued_asset_paints_the_title_once_before_the_id_and_name() {
    let document = issued_document();
    let markup = page(ISSUED, &document);
    let direct = asset_markup(&document, "[]", true);
    assert_eq!(direct, markup);
    let name_row = "<tr><td>Name</td><td>Test Asset (TEST)</td></tr>";
    assert_title_once_before(&markup, "<div class=\"tx-link\">");
    assert_title_once_before(&markup, name_row);
    assert!(
        markup.contains(&format!("<div class=\"tx-link\">{ISSUED}</div>")),
        "{markup}"
    );
    for label in [
        "Precision",
        "Issuer",
        "Issuance TX",
        "Issued amount",
        "Burned amount",
        "Circulating amount",
    ] {
        assert!(
            markup.contains(&format!("<td>{label}</td>")),
            "{label} was dropped: {markup}"
        );
    }
    assert!(markup.contains("registry.example"), "{markup}");
    assert!(markup.contains("5000"), "{markup}");
    assert!(markup.contains("200"), "{markup}");
    assert!(markup.contains("4800"), "{markup}");
    assert_plain_asset_get(ISSUED);

    let cypress = format!("/assets/asset/{ISSUED}");
    let opened = asset_page_markup(&cypress, &document, "[]").expect("cypress asset page");
    assert_eq!(opened.matches(TITLE).count(), 1, "{opened}");

    assert!(
        asset_markup(&document, "[]", false).is_empty(),
        "a non-liquid call painted the asset page"
    );
    assert!(!asset_markup(&document, "[]", false).contains(TITLE));
    assert!(!asset_markup("not-served", "[]", true).contains(TITLE));
    assert!(!asset_markup("", "[]", true).contains(TITLE));

    let view = asset_page_source();
    assert!(
        !view.contains("<h2>\"Asset\"</h2>") && !view.contains("<h1>\"Asset\"</h1>"),
        "AssetPage paints a second Asset title outside the document: {view}"
    );
}

/// A pegged asset paints the title and does not paint a Name row.
#[test]
fn pegged_asset_paints_the_title_without_a_name_row() {
    let document = pegged_document();
    let markup = page(NATIVE_ASSET, &document);
    assert_title_once_before(
        &markup,
        &format!("<div class=\"tx-link\">{NATIVE_ASSET}</div>"),
    );
    assert!(
        !markup.contains("<td>Name</td>"),
        "a pegged asset painted a Name row: {markup}"
    );
    for label in [
        "Pegged in",
        "Pegged out",
        "Burned amount",
        "Circulating amount",
    ] {
        assert!(
            markup.contains(&format!("<td>{label}</td>")),
            "{label} was dropped: {markup}"
        );
    }
    assert!(markup.contains(">1000<"), "{markup}");
    assert!(markup.contains(">200<"), "{markup}");
    assert!(markup.contains(">50<"), "{markup}");
    assert!(markup.contains(">750<"), "{markup}");
    assert_plain_asset_get(NATIVE_ASSET);
}
