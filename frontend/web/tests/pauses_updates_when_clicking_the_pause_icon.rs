//! Cypress `it('pauses updates when clicking the pause icon')`.
//! File: `ref/mempool/frontend/cypress/e2e/mainnet/recent-transactions.spec.ts`,
//! lines 77-102.
//!
//! The it() stubs the browser socket, visits `/txs`, sends six transactions,
//! clicks `[data-cy="btn-pause"]`, then sends six more. The first ten
//! characters of the first new txid must not appear in the txid anchors, and
//! those anchors must stay equal to the txids captured at the click.
//! `paint_pauses_updates_when_clicking_the_pause_icon` is not in this crate.
//! The call is the missing paint.

/// Six txids already on screen when the pause icon is clicked.
const PAUSED: &[&str] = &[
    "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
    "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
    "cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc",
    "dddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddd",
    "eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee",
    "ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff",
];

/// Six txids that arrive after the click. They must not replace `PAUSED`.
const INCOMING: &[&str] = &[
    "1111111111111111111111111111111111111111111111111111111111111111",
    "2222222222222222222222222222222222222222222222222222222222222222",
    "3333333333333333333333333333333333333333333333333333333333333333",
    "4444444444444444444444444444444444444444444444444444444444444444",
    "5555555555555555555555555555555555555555555555555555555555555555",
    "6666666666666666666666666666666666666666666666666666666666666666",
];

#[test]
fn pauses_updates_when_clicking_the_pause_icon() {
    let markup: String =
        splora_web::paint_pauses_updates_when_clicking_the_pause_icon(PAUSED, INCOMING);
    let marker = &INCOMING[0][..10];
    let visible = visible_txid_texts(&markup);
    let visible_text = visible.join(" ");
    assert!(
        markup.contains("data-cy=\"transactions-list\""),
        "the pause click did not leave the transactions list Cypress queries: {markup}"
    );
    assert!(
        !visible_text.contains(marker),
        "paused list included the new txid marker {marker}: {markup}"
    );
    assert_eq!(
        visible.as_slice(),
        PAUSED,
        "visible txids changed after the pause click: {markup}"
    );
}

/// Text of each `.table-cell-txid a`, which is what Cypress reads.
fn visible_txid_texts(markup: &str) -> Vec<&str> {
    const CLASS_NAME: &str = "class=\"table-cell-txid\"";
    let mut texts = Vec::new();
    let mut search_from = 0;
    while let Some(relative) = markup[search_from..].find(CLASS_NAME) {
        let cell_start = search_from + relative + CLASS_NAME.len();
        let cell_end = markup[cell_start..]
            .find("</t")
            .map(|end| cell_start + end)
            .unwrap_or(markup.len());
        if let Some(text) = anchor_text(&markup[cell_start..cell_end]) {
            if !text.is_empty() {
                texts.push(text);
            }
        }
        search_from = cell_end;
        if search_from >= markup.len() {
            break;
        }
    }
    texts
}

fn anchor_text(cell: &str) -> Option<&str> {
    let anchor_at = cell.find("<a")?;
    let after_anchor = &cell[anchor_at..];
    let content_at = after_anchor.find('>')? + 1;
    let end = after_anchor[content_at..].find("</a>")?;
    Some(after_anchor[content_at..content_at + end].trim())
}
