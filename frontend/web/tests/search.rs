//! The search control trims the query and classifies it as a block height when every character is an ASCII digit, including height 0 and a 64-digit id, as a transaction href, a block href, and the scripthash reads GET /scripthash/{hex}, GET /scripthash/{hex}/txs, GET /scripthash/{hex}/utxo, and GET /scripthash/{hex}/txs/mempool when the query is exactly 64 hex digits that are not all digits, as an address when it starts with bc1, tb1, bcrt1, lq1, ex1, or ert1 in any case, except a bc1p string shorter than 62 characters which is an address prefix folded to ASCII lowercase, or is a base58 string of length 26 through 90 starting with 1, 3, m, n, or 2, and otherwise as an address prefix of 1 to 64 characters with no page href when it has no slash, question mark, hash, space, or control character. A bc1 prefix is ASCII lowercase. Any other prefix keeps the typed case. Empty input and anything else are not a search. Mainnet search does not request /asset/.

use splora_web::{
    IndexerRequest, SearchPlan, address_prefix_markup, plan_search, search_links_markup,
    search_result_markup,
};

fn assert_one_get(plan: &SearchPlan, path: &str) {
    assert_eq!(
        plan.requests,
        vec![IndexerRequest {
            method: "GET",
            path: path.to_string(),
        }]
    );
}

fn assert_scripthash_reads(plan: &SearchPlan, hex: &str) {
    assert!(plan.requests.iter().all(|req| req.method == "GET"));
    let paths: Vec<String> = plan.requests.iter().map(|req| req.path.clone()).collect();
    assert_eq!(
        paths,
        vec![
            format!("/scripthash/{hex}"),
            format!("/scripthash/{hex}/txs"),
            format!("/scripthash/{hex}/utxo"),
            format!("/scripthash/{hex}/txs/mempool"),
        ]
    );
    assert!(
        paths.iter().all(|path| !path.contains("/asset/")),
        "mainnet search does not look up an asset: {paths:?}"
    );
}

fn assert_no_replacements_or_lightning(plan: &SearchPlan) {
    let rendered = format!("{plan:?}");
    assert!(
        !rendered.contains("replacements"),
        "plan contains replacements: {rendered}"
    );
    assert!(
        !rendered.contains("lightning"),
        "plan contains lightning: {rendered}"
    );
    for href in &plan.hrefs {
        assert!(!href.contains("replacements"), "{href}");
        assert!(!href.contains("lightning"), "{href}");
    }
    for request in &plan.requests {
        assert!(!request.path.contains("replacements"), "{}", request.path);
        assert!(!request.path.contains("lightning"), "{}", request.path);
    }
}

#[test]
fn height_800000_opens_the_block_height_page() {
    let plan = plan_search("800000").expect("height");
    assert_eq!(plan.hrefs, vec!["/block-height/800000".to_string()]);
    assert_one_get(&plan, "/block-height/800000");
    assert_eq!(
        search_links_markup("800000"),
        "<a href=\"/block-height/800000\">/block-height/800000</a>"
    );
    let trimmed = plan_search("  800000\n").expect("trimmed height");
    assert_eq!(trimmed.hrefs, vec!["/block-height/800000".to_string()]);
    assert_one_get(&trimmed, "/block-height/800000");
}

#[test]
fn height_zero_is_a_block_height() {
    let plan = plan_search("0").expect("height zero");
    assert_eq!(plan.hrefs, vec!["/block-height/0".to_string()]);
    assert_one_get(&plan, "/block-height/0");
    assert_eq!(
        search_links_markup("0"),
        "<a href=\"/block-height/0\">/block-height/0</a>"
    );
}

#[test]
fn long_digit_string_stays_a_height() {
    let digits = "1".repeat(64);
    let plan = plan_search(&digits).expect("64 digits");
    assert_eq!(plan.hrefs, vec![format!("/block-height/{digits}")]);
    assert_one_get(&plan, &format!("/block-height/{digits}"));
    assert!(plan.hrefs.iter().all(|href| !href.contains("/tx/")));
    assert!(
        plan.hrefs
            .iter()
            .all(|href| !href.contains("/block/") && href.starts_with("/block-height/"))
    );

    let longer = "9".repeat(91);
    let plan = plan_search(&longer).expect("91 digits still a height");
    assert_eq!(plan.hrefs, vec![format!("/block-height/{longer}")]);
    assert_one_get(&plan, &format!("/block-height/{longer}"));
}

#[test]
fn empty_query_is_not_a_search() {
    assert_eq!(plan_search(""), None);
    assert_eq!(plan_search("   "), None);
    assert_eq!(plan_search("\n\t"), None);
    assert_eq!(search_links_markup(""), "");
    assert_eq!(search_links_markup("   "), "");
    assert_eq!(address_prefix_markup(""), "");
    assert_eq!(address_prefix_markup(" \n\t"), "");
}

#[test]
fn bc1_address_opens_the_address_page() {
    let address = "bc1qexample";
    let plan = plan_search(address).expect("bc1 address");
    assert_eq!(plan.hrefs, vec![format!("/address/{address}")]);
    assert_one_get(&plan, &format!("/address/{address}"));
    assert!(!plan.requests[0].path.contains('%'));
    assert_eq!(
        search_links_markup(address),
        format!("<a href=\"/address/{address}\">/address/{address}</a>")
    );

    let encoded_looking = "bc1q+example";
    let plus = plan_search(encoded_looking).expect("plus stays in the address");
    assert_eq!(plus.hrefs, vec!["/address/bc1q+example".to_string()]);
    assert_one_get(&plus, "/address/bc1q+example");
    assert!(!plus.requests[0].path.contains('%'));
}

#[test]
fn bech32_family_prefixes_are_addresses_in_any_case() {
    for prefix in [
        "bc1", "tb1", "bcrt1", "lq1", "ex1", "ert1", "BC1", "Tb1", "BCRT1",
    ] {
        let plan = plan_search(prefix).expect(prefix);
        assert_eq!(plan.hrefs, vec![format!("/address/{prefix}")], "{prefix}");
        assert_one_get(&plan, &format!("/address/{prefix}"));
        assert!(
            !plan.requests[0].path.contains("address-prefix"),
            "{prefix} is an address, not a prefix lookup"
        );
    }
}

#[test]
fn short_bc1q_is_an_address() {
    let plan = plan_search("bc1q").expect("bc1q");
    assert_eq!(plan.hrefs, vec!["/address/bc1q".to_string()]);
    assert_one_get(&plan, "/address/bc1q");
    assert!(!plan.requests[0].path.contains("address-prefix"));
}

#[test]
fn base58_address_opens_the_address_page() {
    let address = "1A1zP1eP5QGefi2DMPTfTL5SLmv7DivfNa";
    let plan = plan_search(address).expect("genesis address");
    assert_eq!(plan.hrefs, vec![format!("/address/{address}")]);
    assert_one_get(&plan, &format!("/address/{address}"));
    assert_eq!(
        search_links_markup(address),
        format!("<a href=\"/address/{address}\">/address/{address}</a>")
    );
}

#[test]
fn base58_length_boundaries_choose_address_or_prefix() {
    let address = format!("1{}", "A".repeat(25));
    let plan = plan_search(&address).expect("26 base58 chars");
    assert_eq!(plan.hrefs, vec![format!("/address/{address}")]);
    assert_one_get(&plan, &format!("/address/{address}"));

    let longest = format!("3{}", "A".repeat(89));
    let plan = plan_search(&longest).expect("90 chars");
    assert_eq!(plan.hrefs, vec![format!("/address/{longest}")]);
    assert_one_get(&plan, &format!("/address/{longest}"));

    let sixty_four_m = "m".repeat(64);
    let plan = plan_search(&sixty_four_m).expect("64 m is base58, not hex");
    assert_eq!(plan.hrefs, vec![format!("/address/{sixty_four_m}")]);
    assert_one_get(&plan, &format!("/address/{sixty_four_m}"));

    let short = "1A1zP";
    let plan = plan_search(short).expect("short base58");
    assert_eq!(plan.hrefs, Vec::<String>::new());
    assert_one_get(&plan, "/address-prefix/1A1zP");
    assert_eq!(search_links_markup(short), "");

    let not_base58 = format!("1O{}", "A".repeat(24));
    let plan = plan_search(&not_base58).expect("O is not base58");
    assert_eq!(plan.hrefs, Vec::<String>::new());
    assert_one_get(&plan, &format!("/address-prefix/{not_base58}"));
}

#[test]
fn hex64_opens_tx_block_and_scripthash() {
    let hex = "ab".repeat(32);
    assert_eq!(hex.len(), 64);
    let plan = plan_search(&hex).expect("64 hex");
    assert_eq!(
        plan.hrefs,
        vec![format!("/tx/{hex}"), format!("/block/{hex}")]
    );
    assert_scripthash_reads(&plan, &hex);
    assert_eq!(
        search_links_markup(&hex),
        format!("<a href=\"/tx/{hex}\">/tx/{hex}</a><a href=\"/block/{hex}\">/block/{hex}</a>")
    );
    assert!(!search_links_markup(&hex).contains("scripthash"));
}

#[test]
fn hex64_plan_is_not_replacements_or_lightning() {
    let hex = "ab".repeat(32);
    let plan = plan_search(&hex).expect("64 hex");
    assert_no_replacements_or_lightning(&plan);
    let links = search_links_markup(&hex);
    assert!(!links.contains("replacements"), "{links}");
    assert!(!links.contains("lightning"), "{links}");
}

#[test]
fn mixed_case_hex_is_lowercased() {
    let upper = "AB".repeat(32);
    let hex = "ab".repeat(32);
    let plan = plan_search(&upper).expect("upper hex");
    assert_eq!(
        plan.hrefs,
        vec![format!("/tx/{hex}"), format!("/block/{hex}")]
    );
    assert_scripthash_reads(&plan, &hex);
    assert_eq!(
        search_links_markup(&upper),
        format!("<a href=\"/tx/{hex}\">/tx/{hex}</a><a href=\"/block/{hex}\">/block/{hex}</a>")
    );

    let sixty_three = "a".repeat(63);
    let plan = plan_search(&sixty_three).expect("63 hex is a prefix");
    assert_eq!(plan.hrefs, Vec::<String>::new());
    assert_one_get(&plan, &format!("/address-prefix/{sixty_three}"));

    let sixty_four_a = "a".repeat(64);
    let plan = plan_search(&sixty_four_a).expect("64 a is hex");
    assert_eq!(
        plan.hrefs,
        vec![
            format!("/tx/{sixty_four_a}"),
            format!("/block/{sixty_four_a}")
        ]
    );
    assert_scripthash_reads(&plan, &sixty_four_a);
}

#[test]
fn prefix_xyz_requests_address_prefix_without_a_page() {
    let plan = plan_search("xyz").expect("prefix");
    assert_eq!(plan.hrefs, Vec::<String>::new());
    assert_one_get(&plan, "/address-prefix/xyz");
    assert_eq!(search_links_markup("xyz"), "");

    let plus = plan_search("a+b").expect("plus prefix");
    assert_eq!(plus.hrefs, Vec::<String>::new());
    assert_one_get(&plus, "/address-prefix/a+b");
    assert!(!plus.requests[0].path.contains('%'));
}

#[test]
fn spaced_slashed_and_control_queries_are_not_a_search() {
    for query in ["has space", "a/b", "a?b", "a#b", "a\tb", "a\nb", "a\u{7f}b"] {
        assert_eq!(plan_search(query), None, "{query:?}");
        assert_eq!(search_links_markup(query), "", "{query:?}");
    }
}

#[test]
fn too_long_query_is_not_a_search() {
    assert_eq!(plan_search(&"z".repeat(65)), None);
    assert_eq!(plan_search(&"m".repeat(91)), None);
    let sixty_four = "z".repeat(64);
    let plan = plan_search(&sixty_four).expect("64 z is a prefix");
    assert_eq!(plan.hrefs, Vec::<String>::new());
    assert_one_get(&plan, &format!("/address-prefix/{sixty_four}"));
}

#[test]
fn disabled_address_search_is_a_notice() {
    let markup = address_prefix_markup("address search disabled");
    assert!(markup.contains("address search disabled"));
    assert_eq!(markup, "<p class=\"notice\">address search disabled</p>");
    assert_eq!(
        address_prefix_markup("  address search disabled\n"),
        "<p class=\"notice\">address search disabled</p>"
    );
}

#[test]
fn address_prefix_json_links_the_address() {
    let markup = address_prefix_markup("[\"bc1qexample\"]");
    assert!(markup.contains("href=\"/address/bc1qexample\""));
    assert_eq!(
        markup,
        "<a class=\"address\" href=\"/address/bc1qexample\">bc1qexample</a>"
    );
    assert_eq!(
        address_prefix_markup("[\n  \"bc1qexample\"\n]"),
        "<a class=\"address\" href=\"/address/bc1qexample\">bc1qexample</a>"
    );
    assert_eq!(
        address_prefix_markup(r#"["bc1qone","", "bc1qtwo"]"#),
        "<a class=\"address\" href=\"/address/bc1qone\">bc1qone</a><a class=\"address\" href=\"/address/bc1qtwo\">bc1qtwo</a>"
    );
}

#[test]
fn empty_address_list_says_there_are_no_addresses() {
    let markup = address_prefix_markup("[]");
    assert!(markup.contains("There are no addresses."));
    assert_eq!(markup, "<p class=\"empty\">There are no addresses.</p>");
    assert_eq!(
        address_prefix_markup(" [ ] "),
        "<p class=\"empty\">There are no addresses.</p>"
    );
    assert_eq!(address_prefix_markup("[\"\"]"), "");
    assert_eq!(address_prefix_markup(r#"["", ""]"#), "");
}

#[test]
fn indexer_sentence_is_a_notice() {
    assert_eq!(
        address_prefix_markup("Invalid prefix"),
        "<p class=\"notice\">Invalid prefix</p>"
    );
    assert_eq!(
        address_prefix_markup("a < b & c"),
        "<p class=\"notice\">a &lt; b &amp; c</p>"
    );
}

#[test]
fn non_address_json_is_not_shown_as_a_sentence() {
    assert_eq!(address_prefix_markup("[1]"), "");
    assert_eq!(address_prefix_markup("{\"error\":\"x\"}"), "");
    assert_eq!(address_prefix_markup("one\ntwo"), "");
}

#[test]
fn search_pane_keeps_block_not_found() {
    let body = "Block not found\nindexer error GET /block-height/100";
    let markup = search_result_markup("100", body);
    assert!(markup.contains("Block not found"), "{markup}");
    assert!(
        markup.contains("indexer error GET /block-height/100"),
        "{markup}"
    );
}

#[test]
fn wrapped_disabled_address_search_keeps_the_sentence() {
    let body = "address search disabled\nindexer error GET /address-prefix/pref";
    let markup = address_prefix_markup(body);
    assert!(markup.contains("address search disabled"), "{markup}");
    assert!(
        markup.contains("indexer error GET /address-prefix/pref"),
        "{markup}"
    );
}

#[test]
fn address_markup_escapes_html() {
    let body = r#"["a&b<c>\"d'e"]"#;
    assert_eq!(
        address_prefix_markup(body),
        "<a class=\"address\" href=\"/address/a&amp;b&lt;c&gt;&quot;d&#39;e\">a&amp;b&lt;c&gt;&quot;d&#39;e</a>"
    );
}

#[test]
fn search_links_escape_special_characters() {
    assert_eq!(
        search_links_markup("bc1<a'>"),
        "<a href=\"/address/bc1&lt;a&#39;&gt;\">/address/bc1&lt;a&#39;&gt;</a>"
    );
    let spaced = plan_search("bc1 foo").expect("bc1 still wins over the space");
    assert_eq!(spaced.hrefs, vec!["/address/bc1 foo".to_string()]);
    assert_one_get(&spaced, "/address/bc1 foo");
    assert!(!spaced.requests[0].path.contains('%'));
    assert_eq!(
        search_links_markup("bc1 foo"),
        "<a href=\"/address/bc1 foo\">/address/bc1 foo</a>"
    );
}

/// A `bc1` prefix uses ASCII lowercase. A base58 prefix keeps the typed case.
/// A full address still uses the address plan, not the prefix plan.
#[test]
fn bc1_prefix_folds_ascii_lowercase_base58_prefix_keeps_case() {
    for query in ["BC1PQYQS", "bc1PqYqS"] {
        let plan = plan_search(query).expect(query);
        assert_one_get(&plan, "/address-prefix/bc1pqyqs");
        assert_eq!(plan.hrefs, Vec::<String>::new(), "{query}");
        let path = &plan.requests[0].path;
        assert!(!path.contains('?'), "{path}");
        assert!(!path.contains('#'), "{path}");
    }

    let wiz = plan_search("1wiz").expect("1wiz");
    assert_eq!(wiz.hrefs, Vec::<String>::new());
    assert_one_get(&wiz, "/address-prefix/1wiz");
    assert!(!wiz.requests[0].path.contains('?'));
    assert!(!wiz.requests[0].path.contains('#'));

    let mixed = plan_search("1Wiz").expect("1Wiz");
    assert_eq!(mixed.hrefs, Vec::<String>::new());
    assert_one_get(&mixed, "/address-prefix/1Wiz");
    assert!(!mixed.requests[0].path.contains('?'));
    assert!(!mixed.requests[0].path.contains('#'));

    // Genesis is a full base58 address that `is_address` already accepts.
    let genesis = "1A1zP1eP5QGefi2DMPTfTL5SLmv7DivfNa";
    let plan = plan_search(genesis).expect("genesis address");
    assert_eq!(plan.hrefs, vec![format!("/address/{genesis}")]);
    assert_one_get(&plan, &format!("/address/{genesis}"));
    assert!(
        plan.requests
            .iter()
            .all(|req| !req.path.contains("address-prefix")),
        "a full address is not a prefix lookup: {:?}",
        plan.requests
    );

    // Full bech32m (62 characters). `is_address` accepts it because it starts with bc1.
    let bech32m = "bc1pahzjr2vnfqnqwl0h8jps2aa4zqe29wqxn2nagvd5a2z982k7ttdq8ry370";
    let plan = plan_search(bech32m).expect("full bech32m address");
    assert_eq!(plan.hrefs, vec![format!("/address/{bech32m}")]);
    assert_one_get(&plan, &format!("/address/{bech32m}"));
    assert!(
        !plan.requests[0].path.contains("address-prefix"),
        "a full bech32 address is not a prefix lookup: {}",
        plan.requests[0].path
    );
}
