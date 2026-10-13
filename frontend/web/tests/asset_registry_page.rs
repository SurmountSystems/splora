//! Later page of `GET /assets/registry` on the assets screen.
//!
//! Angular `getLiquidAssetsRegistry$` sends `start_index`, `limit`,
//! `sort_field=name`, and `sort_dir=asc`. The next indexer page is
//! start index 25 and limit 25. Featured cards stay the unpaged list.
//! Mainnet does not request the page.

use splora_frontend_shared::{assets_registry_page_path, assets_registry_search_path};
use splora_web::{registry_list_markup, registry_list_request, registry_page_request};

const LATER_ID: &str = "e1e1e1e1e1e1e1e1e1e1e1e1e1e1e1e1e1e1e1e1e1e1e1e1e1e1e1e1e1e1e1e1";
const ISSUANCE_TX: &str = "d2d2d2d2d2d2d2d2d2d2d2d2d2d2d2d2d2d2d2d2d2d2d2d2d2d2d2d2d2d2d2d2";

fn views_source() -> &'static str {
    include_str!("../src/views.rs")
}

/// Body of one view function, not the import list and not the next function.
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
        .min();
    let end = after_sig + end_in_tail.unwrap_or(tail.len());
    &rest[..end]
}

fn later_issued_row() -> String {
    format!(
        r#"[{{"asset_id":"{LATER_ID}","name":"Later Asset","ticker":"LATE","precision":8,"issuance_txin":{{"txid":"{ISSUANCE_TX}","vin":0}},"issuance_prevout":{{"txid":"{ISSUANCE_TX}","vout":0}},"reissuance_token":"{ISSUANCE_TX}","status":{{"confirmed":true}},"chain_stats":{{"tx_count":1,"issuance_count":1,"issued_amount":1,"burned_amount":0,"has_blinded_issuances":false,"burned_reissuance_tokens":0}},"mempool_stats":{{"tx_count":0,"issuance_count":0,"issued_amount":0,"burned_amount":0,"has_blinded_issuances":false,"burned_reissuance_tokens":0}}}}]"#
    )
}

#[test]
fn later_registry_page_uses_the_indexer_query() {
    let page = registry_page_request(true, Some("25"), Some("25"), Some("name"), Some("asc"))
        .expect("liquid later page");
    assert_eq!(page.method, "GET");
    assert_eq!(
        page.path,
        assets_registry_page_path(Some("25"), Some("25"), Some("name"), Some("asc"))
    );

    let empty_sort = registry_page_request(true, Some("25"), Some("25"), Some(""), Some(""))
        .expect("empty sort keys stay");
    assert_eq!(
        empty_sort.path,
        assets_registry_page_path(Some("25"), Some("25"), Some(""), Some(""))
    );

    assert!(
        registry_page_request(false, Some("25"), Some("25"), Some("name"), Some("asc")).is_none()
    );

    let list = registry_list_request(true).expect("unpaged registry");
    assert_eq!(list.path, "/assets/registry");
    assert!(!list.path.contains('?'), "{}", list.path);

    assert_eq!(
        assets_registry_search_path("a b"),
        "/assets/registry/search?q=a%20b"
    );

    let body = later_issued_row();
    let painted = registry_list_markup(&body, true);
    assert!(painted.contains("Later Asset"), "{painted}");
    assert!(painted.contains("LATE"), "{painted}");
    assert!(painted.contains(&format!("/asset/{LATER_ID}")), "{painted}");
    assert!(!registry_list_markup(&body, false).contains("Later Asset"));

    let screen = view_fn_body("AssetsPage");
    assert!(
        screen.starts_with("fn AssetsPage("),
        "slice did not start at AssetsPage"
    );
    assert!(
        !screen.contains("fn RegistryPage("),
        "AssetsPage slice included RegistryPage"
    );
    assert!(screen.contains("registry_page_request("), "{screen}");
    assert!(screen.contains("registry_list_markup("), "{screen}");
    assert!(screen.contains("asset_cards_markup("), "{screen}");
    assert!(screen.contains("registry_list_request("), "{screen}");
    assert!(!screen.contains("/assets/featured"), "{screen}");
}
