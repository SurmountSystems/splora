//! Cypress it() title: updates the transaction list over time.
//!
//! The spec visits `/txs`, sends one batch of six transactions, and expects
//! `[data-cy="transactions-list"] tr` to have a length greater than 0. It
//! then sends a second batch of six. `markerTxid` is the first 10 characters
//! of that second batch's first txid. The joined text of
//! `[data-cy="transactions-list"] tr .table-cell-txid a` must include that
//! marker. `app-truncate` puts the whole txid in the anchor text, so the
//! first 10 characters are in that text. The spec builds those txids in
//! `generateMockTransactions`. It does not compare the list to a saved
//! indexer body or to a websocket frame file. The init chart and mempool
//! frames are not what `expect(visibleText).to.include(markerTxid)` checks.

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

fn joined_anchor_text(txids: &[&str]) -> String {
    txids
        .iter()
        .map(|txid| txid.trim())
        .collect::<Vec<_>>()
        .join(" ")
}

#[test]
fn updates_the_transaction_list_over_time() {
    let first_owned = batch(0x00);
    let second_owned = batch(0x01);
    let first = refs(&first_owned);
    let second = refs(&second_owned);
    let marker = &second[0][..10];

    let (after_first, after_second) =
        splora_frontend_shared::updates_the_transaction_list_over_time(&first, &second);

    assert!(
        after_first.len() > 0,
        "cy.get('[data-cy=\"transactions-list\"] tr').should('have.length.greaterThan', 0)"
    );
    let before = joined_anchor_text(&after_first);
    assert!(
        !before.contains(marker),
        "the first batch does not already show the second batch marker {marker}"
    );

    let visible_text = joined_anchor_text(&after_second);
    assert!(
        visible_text.contains(marker),
        "expect(visibleText).to.include(markerTxid) where markerTxid is {marker}"
    );
    assert!(
        after_second.len() > 0,
        "the second batch still leaves transaction rows on the list"
    );
}
