//! A 64-digit id is a block height, not a block hash.
//! A failed broadcast stays the indexer sentence and does not open `/tx/`.

use splora_frontend_shared::broadcast_path;
use splora_web::{
    block_load_plan, block_status_request, block_txs_older_href, broadcast_fields_text,
    broadcast_txid_markup, indexer_failure_text, looks_like_block_hash, package_txid_markup,
    resolved_block_hash, submission_links,
};

const HASH: &str = "abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789";
const TXID: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";

fn digits() -> String {
    "1".repeat(64)
}

fn block_json(id: &str) -> String {
    format!(
        r#"{{"id":"{id}","height":100,"version":1,"timestamp":1,"tx_count":1,"size":1,"weight":4,"merkle_root":"m","mediantime":1,"nonce":1,"bits":1,"difficulty":1}}"#
    )
}

fn older_markup(route: &str, height_body: &str, block_json: &str) -> String {
    resolved_block_hash(route, height_body, block_json)
        .and_then(|hash| block_txs_older_href(&hash, 0))
        .map(|href| format!("<a href=\"{href}\">Older transactions</a>"))
        .unwrap_or_default()
}

fn status_path(id: &str) -> Option<String> {
    looks_like_block_hash(id).then(|| block_status_request(id).path)
}

#[test]
fn sixty_four_digits_are_a_height_not_a_block_hash() {
    let digits = digits();
    assert!(
        !looks_like_block_hash(&digits),
        "an all-digit id is not a block hash"
    );
    assert!(!looks_like_block_hash("100"));
    assert!(!looks_like_block_hash("800000"));
    assert!(looks_like_block_hash(HASH));
    assert!(looks_like_block_hash(&HASH.to_ascii_uppercase()));

    assert_eq!(
        status_path(&digits),
        None,
        "no GET /block/{{digits}}/status"
    );
    assert_eq!(status_path("100"), None);
    assert_eq!(status_path("800000"), None);
    assert_eq!(
        status_path(HASH).as_deref(),
        Some(format!("/block/{HASH}/status").as_str())
    );

    let missing = older_markup(&digits, "", "");
    assert_eq!(missing, "");
    assert!(
        !missing.contains("/block/"),
        "a missing hash emits no /block/ link: {missing}"
    );
    assert_eq!(resolved_block_hash(&digits, "", ""), None);
    assert_eq!(resolved_block_hash("100", "", ""), None);
    assert_eq!(resolved_block_hash("800000", "", ""), None);

    let from_height = older_markup(&digits, &format!(" {HASH}\n"), "");
    assert!(
        from_height.contains(&format!("/block/{HASH}?start_index=25")),
        "{from_height}"
    );
    assert!(
        !from_height.contains(&format!("/block/{digits}")),
        "{from_height}"
    );
    assert_eq!(
        resolved_block_hash(&digits, &format!(" {HASH}\n"), "").as_deref(),
        Some(HASH)
    );

    let from_block = resolved_block_hash(&digits, "Block not found", &block_json(HASH));
    assert_eq!(from_block.as_deref(), Some(HASH));

    assert_eq!(
        resolved_block_hash("100", &format!(" {HASH}\n"), "").as_deref(),
        Some(HASH)
    );
    assert_eq!(
        resolved_block_hash("800000", HASH, "").as_deref(),
        Some(HASH)
    );
    assert_eq!(resolved_block_hash(HASH, "", "").as_deref(), Some(HASH));
    let upper = HASH.to_ascii_uppercase();
    assert_eq!(
        resolved_block_hash(&upper, "", "").as_deref(),
        Some(upper.as_str())
    );

    let height_gets = block_load_plan(&digits, None);
    assert_eq!(
        height_gets
            .iter()
            .map(|call| call.path.as_str())
            .collect::<Vec<_>>(),
        vec![format!("/block-height/{digits}")]
    );
    assert!(height_gets.iter().all(|call| call.method == "GET"));
    assert!(
        height_gets
            .iter()
            .all(|call| !call.path.starts_with(&format!("/block/{digits}")))
    );
    let loaded = block_load_plan(&digits, Some(&format!(" {HASH}\n")));
    assert!(
        loaded
            .iter()
            .any(|call| call.path == format!("/block/{HASH}")),
        "{loaded:?}"
    );
    assert!(
        loaded
            .iter()
            .all(|call| !call.path.contains(digits.as_str())),
        "{loaded:?}"
    );
    assert_eq!(
        block_load_plan("100", None)
            .iter()
            .map(|call| call.path.as_str())
            .collect::<Vec<_>>(),
        vec!["/block-height/100"]
    );
    assert_eq!(
        block_load_plan("800000", None)
            .iter()
            .map(|call| call.path.as_str())
            .collect::<Vec<_>>(),
        vec!["/block-height/800000"]
    );

    let short = older_markup("100", "", "");
    assert_eq!(short, "");
    assert!(!short.contains("/block/"), "{short}");
    let short_known = older_markup("800000", HASH, "");
    assert!(
        short_known.contains(&format!("/block/{HASH}?start_index=25")),
        "{short_known}"
    );
    assert!(!short_known.contains("/block/800000"), "{short_known}");
}

#[test]
fn failed_broadcast_stays_the_sentence() {
    let post_tx = indexer_failure_text("POST", "/tx", "sendrawtransaction RPC error");
    assert!(
        post_tx.contains("sendrawtransaction RPC error"),
        "{post_tx}"
    );
    assert!(post_tx.contains("indexer error POST /tx"), "{post_tx}");
    let shown = broadcast_fields_text(&post_tx);
    assert!(shown.contains("sendrawtransaction RPC error"), "{shown}");
    assert!(shown.contains("indexer error POST /tx"), "{shown}");
    let links = submission_links(broadcast_path(), &post_tx);
    assert_eq!(links, "");
    assert!(
        !links.contains("href=\"/tx/"),
        "a failed POST /tx is not a transaction link: {links}"
    );

    let post_package = indexer_failure_text("POST", "/txs/package", "package rejected");
    assert!(post_package.contains("package rejected"), "{post_package}");
    assert!(
        post_package.contains("indexer error POST /txs/package"),
        "{post_package}"
    );
    let package_links = submission_links(broadcast_path(), &post_package);
    assert_eq!(package_links, "");
    assert!(
        !package_links.contains("href=\"/tx/"),
        "a failed POST /txs/package is not a transaction link: {package_links}"
    );

    let bare = "indexer error POST /tx";
    assert_eq!(submission_links(broadcast_path(), bare), "");

    let plain = submission_links(broadcast_path(), TXID);
    assert_eq!(plain, broadcast_txid_markup(TXID));
    assert!(plain.contains(&format!("href=\"/tx/{TXID}\"")), "{plain}");

    let package =
        format!(r#"{{"package_msg":"success","tx-results":{{"{TXID}":{{"txid":"{TXID}"}}}}}}"#);
    let expected = package_txid_markup(&package);
    assert_eq!(submission_links(broadcast_path(), &package), expected);
    assert!(expected.contains("success"), "{expected}");
    assert!(
        expected.contains(&format!("href=\"/tx/{TXID}\"")),
        "{expected}"
    );
}
