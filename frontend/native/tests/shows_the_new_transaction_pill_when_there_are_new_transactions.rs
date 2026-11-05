//! Cypress it() title: shows the new transaction pill when there are new transactions.
//!
//! The it() visits `/txs`, sends six transactions, scrolls until
//! `window.scrollY` is greater than 0, then sends six more.
//! `cy.get('[data-cy="new-tx-pill"]').should('be.visible')`.
//! The list is not the widget and the manual pause control is off.
//! The visible label for six buffered ids is `6 new transactions`.
//! Scroll top 0, a repeated batch, manual pause, and the widget leave
//! that pill off the screen. One new id paints `1 new transaction`.
//! This it() does not read a GET `/mempool/recent` body.
//! `paint_shows_the_new_transaction_pill_when_there_are_new_transactions`
//! is not defined on `splora_native`. This test does not open a window.

const CYPRESS_IT: &str = "shows the new transaction pill when there are new transactions";

fn paint(
    widget: bool,
    manual_paused: bool,
    scroll_top: i64,
    before: &[&str],
    after: &[&str],
) -> String {
    splora_native::paint_shows_the_new_transaction_pill_when_there_are_new_transactions(
        widget,
        manual_paused,
        scroll_top,
        before,
        after,
    )
}

fn has_pill(painted: &str) -> bool {
    painted
        .lines()
        .any(|line| line == "data-cy=\"new-tx-pill\"")
}

fn has_label(painted: &str, label: &str) -> bool {
    painted.lines().any(|line| line == label)
}

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

    let painted = paint(false, false, 1, &before, &after);
    assert!(
        has_pill(&painted) && has_label(&painted, "6 new transactions"),
        "{CYPRESS_IT}: cy.get('[data-cy=\"new-tx-pill\"]').should('be.visible') with label 6 new transactions after scrollY > 0 and six more ids; got {painted:?}"
    );

    let at_top = paint(false, false, 0, &before, &after);
    assert!(
        !has_pill(&at_top) && !has_label(&at_top, "6 new transactions"),
        "{CYPRESS_IT}: scroll_top 0 leaves the pill hidden; got {at_top:?}"
    );

    let no_new = paint(false, false, 1, &before, &before);
    assert!(
        !has_pill(&no_new),
        "{CYPRESS_IT}: the same ids arriving again leave the pill hidden; got {no_new:?}"
    );

    let one_new = paint(false, false, 1, &before, &["later-only"]);
    assert!(
        has_pill(&one_new) && has_label(&one_new, "1 new transaction"),
        "{CYPRESS_IT}: one new id shows the pill labeled 1 new transaction; got {one_new:?}"
    );
    assert!(
        !has_label(&one_new, "6 new transactions") && !one_new.contains("new transactions"),
        "{CYPRESS_IT}: one new id must not use the plural label; got {one_new:?}"
    );

    let manual = paint(false, true, 1, &before, &after);
    assert!(
        !has_pill(&manual) && !has_label(&manual, "6 new transactions"),
        "{CYPRESS_IT}: manual pause hides the pill; got {manual:?}"
    );

    let widget = paint(true, false, 1, &before, &after);
    assert!(
        !has_pill(&widget) && !has_label(&widget, "6 new transactions"),
        "{CYPRESS_IT}: the widget hides the pill and does not auto-pause from scroll; got {widget:?}"
    );
}
