//! Cypress it() title: updates the transaction list over time.
//!
//! The spec visits `/txs`, sends one batch of six transactions, and expects
//! `[data-cy="transactions-list"] tr` to have a length greater than 0. It
//! then sends a second batch of six. The joined text of
//! `[data-cy="transactions-list"] tr .table-cell-txid a` must include the
//! first 10 characters of that second batch's first txid. This test calls
//! `paint_updates_the_transaction_list_over_time` and reads the laid-out
//! anchor text and row heights from that paint. It does not search source
//! text and it does not rebuild the list. The two batches match the shared
//! test of the same name. The marker is a slice of the second batch argument.

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
    assert_eq!(first[0].len(), 64);
    assert_eq!(second[0].len(), 64);
    assert_eq!(marker.len(), 10);

    let painted = splora_native::paint_updates_the_transaction_list_over_time(&first, &second);

    assert!(
        painted.first_row_count > 0,
        "cy.get('[data-cy=\"transactions-list\"] tr').should('have.length.greaterThan', 0); got {}",
        painted.first_row_count
    );
    assert_eq!(
        painted.first_row_heights.len(),
        painted.first_row_count,
        "each first-batch row needs a laid-out height"
    );
    assert!(
        painted.first_row_heights.iter().all(|height| *height > 0.0),
        "first-batch rows were not laid out, heights {:?}",
        painted.first_row_heights
    );
    assert!(
        !painted.first_visible_text.contains(marker),
        "the first batch does not already show the second batch marker {marker}; got {}",
        painted.first_visible_text
    );

    assert!(
        painted.later_visible_text.contains(marker),
        "expect(visibleText).to.include(markerTxid) where markerTxid is {marker}; got {}",
        painted.later_visible_text
    );
    assert!(
        painted.later_row_count > 0,
        "the second batch still leaves transaction rows on the list; got {}",
        painted.later_row_count
    );
    assert_eq!(
        painted.later_row_heights.len(),
        painted.later_row_count,
        "each later row needs a laid-out height"
    );
    assert!(
        painted.later_row_heights.iter().all(|height| *height > 0.0),
        "later rows were not laid out, heights {:?}",
        painted.later_row_heights
    );
}
