//! Cypress it() title: updates the transaction list over time.
//!
//! File: `ref/mempool/frontend/cypress/e2e/mainnet/recent-transactions.spec.ts`,
//! lines 60-75.
//!
//! The it() visits `/txs`, sends six transactions, and expects
//! `[data-cy="transactions-list"] tr` to have a length greater than 0.
//! It then sends six more. The joined text of
//! `[data-cy="transactions-list"] tr .table-cell-txid a` must include the
//! first 10 characters of that second batch's first txid. Those batches
//! match the shared test. The prefix is taken from the second-batch
//! argument. This test does not search product source text.
//! `paint_updates_the_transaction_list_over_time` is not in this crate.
//! The call is the missing paint.

/// Six 64-character ids. `tag` is the first byte so the two batches do not share a prefix.
fn batch(tag: u8) -> [String; 6] {
    std::array::from_fn(|index| {
        let mut id = format!("{tag:02x}{index:02x}");
        id.push_str(&"cd".repeat(30));
        id
    })
}

fn refs(owned: &[String; 6]) -> [&str; 6] {
    std::array::from_fn(|index| owned[index].as_str())
}

#[test]
fn updates_the_transaction_list_over_time() {
    let first_owned = batch(0x00);
    let second_owned = batch(0x01);
    let first = refs(&first_owned);
    let second = refs(&second_owned);
    let marker = &second[0][..10];

    let markup = splora_web::updates_the_transaction_list_over_time::paint_updates_the_transaction_list_over_time(
        &first, &second,
    );
    let rows = transaction_rows(&markup);
    assert!(
        rows > 0,
        "cy.get('[data-cy=\"transactions-list\"] tr').should('have.length.greaterThan', 0), markup was {markup}"
    );
    let visible_text = visible_txid_texts(&markup).join(" ");
    assert!(
        visible_text.contains(marker),
        "expect(visibleText).to.include(markerTxid) where markerTxid is {marker}, visible was {visible_text}"
    );
}

/// `[data-cy="transactions-list"] tr` inside the painted list.
fn transaction_rows(markup: &str) -> usize {
    let list = transactions_list(markup);
    list.matches("<tr").count()
}

fn transactions_list(markup: &str) -> &str {
    const OPEN: &str = "data-cy=\"transactions-list\"";
    let Some(start) = markup.find(OPEN) else {
        return "";
    };
    &markup[start..]
}

/// Text of each `.table-cell-txid a`, which is what Cypress reads.
fn visible_txid_texts(markup: &str) -> Vec<&str> {
    const CLASS_NAME: &str = "class=\"table-cell-txid\"";
    let list = transactions_list(markup);
    let mut texts = Vec::new();
    let mut search_from = 0;
    while let Some(relative) = list[search_from..].find(CLASS_NAME) {
        let cell_start = search_from + relative + CLASS_NAME.len();
        let cell_end = list[cell_start..]
            .find("</t")
            .map(|end| cell_start + end)
            .unwrap_or(list.len());
        if let Some(text) = anchor_text(&list[cell_start..cell_end]) {
            if !text.is_empty() {
                texts.push(text);
            }
        }
        search_from = cell_end;
        if search_from >= list.len() {
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
