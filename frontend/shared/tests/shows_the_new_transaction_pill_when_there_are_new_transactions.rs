//! Cypress it() title: shows the new transaction pill when there are new transactions.
//!
//! The spec visits `/txs`, sends six transactions, scrolls until `scrollY`
//! is greater than 0, then sends six more. `[data-cy="new-tx-pill"]` is
//! visible. The button is not the widget, manual pause is off, and
//! `bufferedCount` is the deduped ids that arrived after scroll froze the
//! list. This it() does not read a GET `/mempool/recent` body.

use splora_frontend_shared::shows_the_new_transaction_pill_when_there_are_new_transactions as pill_after_new_txs;

#[test]
fn shows_the_new_transaction_pill_when_there_are_new_transactions() {
    let before = [
        "already-0",
        "already-1",
        "already-2",
        "already-3",
        "already-4",
        "already-5",
    ];
    let after = [
        "later-0", "later-1", "later-2", "later-3", "later-4", "later-5",
    ];
    let pill = pill_after_new_txs(false, false, 1, &before, &after);
    assert!(
        pill.visible,
        "cy.get('[data-cy=\"new-tx-pill\"]').should('be.visible') after scrollY > 0 and six more transactions"
    );
    assert_eq!(pill.buffered_count, 6);
    assert_eq!(pill.label, "6 new transactions");

    let at_top = pill_after_new_txs(false, false, 0, &before, &after);
    assert!(!at_top.visible);
    assert_eq!(at_top.buffered_count, 0);
    assert_eq!(at_top.label, "");

    let no_new = pill_after_new_txs(false, false, 1, &before, &before);
    assert!(!no_new.visible);
    assert_eq!(no_new.buffered_count, 0);

    let one_new = pill_after_new_txs(false, false, 1, &before, &["later-only"]);
    assert!(one_new.visible);
    assert_eq!(one_new.buffered_count, 1);
    assert_eq!(one_new.label, "1 new transaction");

    let manual = pill_after_new_txs(false, true, 1, &before, &after);
    assert!(!manual.visible);
    assert_eq!(manual.buffered_count, 6);

    let widget = pill_after_new_txs(true, false, 1, &before, &after);
    assert!(!widget.visible);
    assert_eq!(widget.buffered_count, 0);
}
