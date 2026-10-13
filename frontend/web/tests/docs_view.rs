#[test]
fn docs_view_text_contains_blocks_and_tx() {
    let text = splora_web::docs_view_text();
    assert!(text.contains("/blocks"), "docs view text must list /blocks");
    assert!(text.contains("/tx/"), "docs view text must list /tx/");
    assert!(
        text.contains("/testnet4"),
        "docs view text must list the /testnet4 prefix"
    );
    assert!(
        text.contains("/testnet"),
        "docs view text must list the /testnet prefix"
    );
    assert!(
        text.contains("/mutinynet"),
        "docs view text must list the /mutinynet prefix"
    );
    assert!(
        text.contains("/liquid"),
        "docs view text must list the /liquid prefix"
    );
    assert!(
        text.contains("/signet"),
        "docs view text must mention /signet"
    );
    assert!(
        text.contains("307"),
        "docs view text must say /signet is HTTP 307"
    );
    assert!(
        text.contains("HTTP 307"),
        "docs view text must say HTTP 307 to /mutinynet"
    );
    let source = include_str!("../src/views.rs");
    assert!(
        text.contains("/blocks")
            && text.contains("/tx/")
            && text.contains("/signet is HTTP 307")
            && source.contains(r#"href="/docs""#),
        "docs body must contain /blocks, /tx/, and the signet 307 together with href=\"/docs\""
    );
}

#[test]
fn view_source_contains_docs_href() {
    let source = include_str!("../src/views.rs");
    assert!(
        source.contains(r#"href="/docs""#),
        "view source must contain href=\"/docs\""
    );
}
