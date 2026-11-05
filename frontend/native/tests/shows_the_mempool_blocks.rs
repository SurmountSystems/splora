//! Cypress `it('shows the mempool blocks')` display strings.
//!
//! Spec: `ref/mempool/frontend/cypress/e2e/mainnet/mining.spec.ts`.
//! Expected lines come from `splora_frontend_shared::shows_the_mempool_blocks`.
//! This calls `paint_shows_the_mempool_blocks` before that paint exists.
//! It does not fetch a URL and it does not hard-code a block's display text
//! as the source of the five lines.

#[test]
fn shows_the_mempool_blocks() {
    let wide_fields =
        splora_frontend_shared::shows_the_mempool_blocks(12, 1, 40, 250_000_000, 1500, 8);
    let narrow_fields = splora_frontend_shared::shows_the_mempool_blocks(4, 2, 7, 10_000_000, 3, 2);
    let one_fields = splora_frontend_shared::shows_the_mempool_blocks(4, 2, 7, 10_000_000, 1, 1);

    let wide = splora_native::paint_shows_the_mempool_blocks(12, 1, 40, 250_000_000, 1500, 8);
    let narrow = splora_native::paint_shows_the_mempool_blocks(4, 2, 7, 10_000_000, 3, 2);
    let one = splora_native::paint_shows_the_mempool_blocks(4, 2, 7, 10_000_000, 1, 1);

    assert_eq!(wide, wide_fields);
    assert_eq!(narrow, narrow_fields);
    assert_eq!(one, one_fields);

    for (left, right) in wide.iter().zip(narrow.iter()) {
        assert_ne!(
            left, right,
            "the five strings must differ across the two field sets"
        );
    }
    assert_cypress_plural(&wide);
    assert_cypress_plural(&narrow);

    assert_eq!(one[3], "1 transaction");
    assert_eq!(one[4], "In ~1 minute");
    assert!(
        !matches_transactions_plural(&one[3]),
        "singular transaction text is outside /(.*) transactions/"
    );
    assert!(
        !matches_minutes_plural(&one[4]),
        "singular minute text is outside /In ~(.*) minutes/"
    );
}

fn assert_cypress_plural(lines: &[String; 5]) {
    assert!(
        matches_fees(&lines[0]),
        "fees must match /~(.*) sat\\/vB/, got {}",
        lines[0]
    );
    assert!(
        matches_fee_span(&lines[1]),
        "fee span must match /(.*) - (.*) sat\\/vB/, got {}",
        lines[1]
    );
    assert!(
        matches_btc(&lines[2]),
        "total fees must match /(.*) BTC/, got {}",
        lines[2]
    );
    assert!(
        matches_transactions_plural(&lines[3]),
        "transaction count must match /(.*) transactions/, got {}",
        lines[3]
    );
    assert!(
        matches_minutes_plural(&lines[4]),
        "time must match /In ~(.*) minutes/, got {}",
        lines[4]
    );
}

/// `/~(.*) sat\/vB/` searched per line. `.` does not cross a newline.
fn matches_fees(text: &str) -> bool {
    text.split('\n').any(|line| match line.find('~') {
        Some(index) => line[index + 1..].contains(" sat/vB"),
        None => false,
    })
}

/// `/(.*) - (.*) sat\/vB/` searched per line.
fn matches_fee_span(text: &str) -> bool {
    text.split('\n').any(|line| {
        line.find(" - ")
            .is_some_and(|index| line[index + 3..].contains(" sat/vB"))
    })
}

/// `/(.*) BTC/` searched per line.
fn matches_btc(text: &str) -> bool {
    text.split('\n').any(|line| line.contains(" BTC"))
}

/// `/(.*) transactions/` searched per line. Singular `transaction` does not match.
fn matches_transactions_plural(text: &str) -> bool {
    text.split('\n').any(|line| line.contains(" transactions"))
}

/// `/In ~(.*) minutes/` searched per line. Singular `minute` does not match.
fn matches_minutes_plural(text: &str) -> bool {
    text.split('\n').any(|line| {
        line.find("In ~")
            .is_some_and(|index| line[index + 4..].contains(" minutes"))
    })
}
