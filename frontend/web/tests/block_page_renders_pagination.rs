//! The block page paints its transaction pager for a caller-supplied width.
//!
//! At least 125 transactions: viewport 760 has 9 `li` children in the
//! first `ul.pagination`, and viewport 669 has 7. Those counts come from
//! the page function. This test does not search source text and does not
//! call `block_transactions_markup_for_viewport` itself.

const HASH: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";

fn local_block_json(tx_count: u32) -> String {
    format!(
        r#"{{"id":"{HASH}","height":2,"version":536870912,"timestamp":1700000000,"tx_count":{tx_count},"size":100,"weight":400,"merkle_root":"merkle-root","previousblockhash":null,"mediantime":1699990000,"nonce":1,"bits":386547056,"difficulty":1}}"#
    )
}

fn first_pagination_li_children(markup: &str) -> usize {
    let start = markup
        .find("<ul class=\"pagination\">")
        .unwrap_or_else(|| panic!("missing pager in {markup}"));
    let rest = &markup[start..];
    let end = rest
        .find("</ul>")
        .unwrap_or_else(|| panic!("pager is not closed in {markup}"));
    rest[..end].matches("<li").count()
}

#[test]
fn block_page_renders_pagination_nine_li_at_760_and_seven_at_669() {
    let block = local_block_json(125);
    let txs = "[]";
    let desktop = splora_web::block_page_transactions_markup(&block, txs, 1, 760);
    assert_eq!(
        first_pagination_li_children(&desktop),
        9,
        "viewport 760 should paint 9 pagination children: {desktop}"
    );
    let phone = splora_web::block_page_transactions_markup(&block, txs, 1, 669);
    assert_eq!(
        first_pagination_li_children(&phone),
        7,
        "viewport 669 should paint 7 pagination children: {phone}"
    );
}
