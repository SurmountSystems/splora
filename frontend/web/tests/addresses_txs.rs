//! The multi-address page posts `POST /addresses/txs` for an address list and
//! `POST /scripthashes/txs` for a scripthash list, and paints both with the
//! address transaction rows. The summary posts stay.

use splora_web::{address_rows_markup, multi_address_txs_post};

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

fn code_has_call(body: &str, call: &str) -> bool {
    body.lines().any(|line| {
        let code = line.split("//").next().unwrap_or(line);
        code.contains(call)
    })
}

#[test]
fn multi_address_page_posts_addresses_txs_and_paints_the_rows() {
    let values = vec!["bc1qone".to_string(), "bc1qtwo".to_string()];
    let posts = multi_address_txs_post(&values);
    assert_eq!(posts.len(), 1, "{posts:?}");
    assert_eq!(posts[0].method, "POST");
    assert_eq!(posts[0].path, "/addresses/txs");
    assert_eq!(posts[0].body, r#"["bc1qone","bc1qtwo"]"#);

    let screen = view_fn_body("MultiAddressPage");
    assert!(
        screen.starts_with("fn MultiAddressPage("),
        "slice did not start at MultiAddressPage"
    );
    assert!(
        !screen.contains("fn BroadcastPage("),
        "slice included BroadcastPage"
    );
    assert!(
        code_has_call(screen, "multi_address_txs_post("),
        "MultiAddressPage does not post the address transaction list: {screen}"
    );
    assert!(
        code_has_call(screen, "addresses_txs_path("),
        "MultiAddressPage does not request POST /addresses/txs: {screen}"
    );
    assert!(
        code_has_call(screen, "scripthashes_txs_path("),
        "a scripthash list posts POST /scripthashes/txs: {screen}"
    );
    assert!(
        code_has_call(screen, "address_rows_markup(\"\", &txs_body.get(), \"\")"),
        "MultiAddressPage does not paint the POST /addresses/txs rows: {screen}"
    );

    let txid = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
    let body = format!(
        r#"[{{"txid":"{txid}","version":1,"locktime":0,"vin":[],"vout":[],"size":50,"weight":200,"sigops":0,"fee":7,"status":{{"confirmed":true,"block_height":842001}}}}]"#
    );
    let markup = address_rows_markup("", &body, "");
    assert!(markup.contains(&format!("href=\"/tx/{txid}\"")), "{markup}");
    assert!(markup.contains("tx-rows"), "{markup}");
    assert!(!markup.contains('¥'), "{markup}");
}

#[test]
fn scripthash_list_posts_scripthashes_txs_and_paints_the_rows() {
    let hash = "abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789";
    let posts = multi_address_txs_post(&[hash.to_string()]);
    assert_eq!(posts.len(), 1, "{posts:?}");
    assert_eq!(posts[0].method, "POST");
    assert_eq!(posts[0].path, "/scripthashes/txs");
    assert_eq!(posts[0].body, format!("[\"{hash}\"]"));
    assert!(
        posts[0].body.chars().all(|ch| !ch.is_ascii_uppercase()),
        "body is the lowercase JSON array: {}",
        posts[0].body
    );

    let addresses = vec!["bc1qone".to_string(), "bc1qtwo".to_string()];
    let address_posts = multi_address_txs_post(&addresses);
    assert_eq!(address_posts.len(), 1, "{address_posts:?}");
    assert_eq!(address_posts[0].method, "POST");
    assert_eq!(address_posts[0].path, "/addresses/txs");
    assert_eq!(address_posts[0].body, r#"["bc1qone","bc1qtwo"]"#);
    assert!(
        address_posts
            .iter()
            .all(|post| post.path != "/scripthashes/txs"),
        "{address_posts:?}"
    );

    let screen = view_fn_body("MultiAddressPage");
    assert!(
        screen.starts_with("fn MultiAddressPage("),
        "slice did not start at MultiAddressPage"
    );
    assert!(
        !screen.contains("fn BroadcastPage("),
        "slice included BroadcastPage"
    );
    assert!(
        code_has_call(screen, "multi_address_txs_post("),
        "MultiAddressPage does not post the transaction lists: {screen}"
    );
    assert!(
        code_has_call(screen, "addresses_txs_path("),
        "an address list stays on POST /addresses/txs: {screen}"
    );
    assert!(
        code_has_call(screen, "scripthashes_txs_path("),
        "a scripthash list does not post POST /scripthashes/txs: {screen}"
    );
    assert!(
        code_has_call(screen, "address_rows_markup(\"\", &script_txs.get(), \"\")"),
        "MultiAddressPage does not paint the scripthash transaction rows: {screen}"
    );
    assert!(
        code_has_call(screen, "address_rows_markup(\"\", &txs_body.get(), \"\")"),
        "the address list still uses address_rows_markup: {screen}"
    );
    assert!(
        !screen.contains("/scripthash/txs"),
        "singular batch path is not a route: {screen}"
    );
    assert!(
        !screen.contains("GET /broadcast"),
        "broadcast is not this page: {screen}"
    );

    let txid = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
    let body = format!(
        r#"[{{"txid":"{txid}","version":1,"locktime":0,"vin":[],"vout":[],"size":50,"weight":200,"sigops":0,"fee":7,"status":{{"confirmed":true,"block_height":842001}}}}]"#
    );
    let markup = address_rows_markup("", &body, "");
    assert!(markup.contains(&format!("href=\"/tx/{txid}\"")), "{markup}");
    assert!(markup.contains("tx-rows"), "{markup}");
}
