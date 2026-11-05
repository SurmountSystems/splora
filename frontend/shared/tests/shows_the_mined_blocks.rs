//! Cypress `it('shows the mined blocks')` display strings.
//!
//! Spec: `ref/mempool/frontend/cypress/e2e/mainnet/mining.spec.ts`.
//! The seven checks are regexes on visible text. Attribute names such as
//! `bitcoin-block-0-fees` include neither the template offset nor the index,
//! so this test does not treat those strings as the rendered text.
//! This calls `shows_the_mined_blocks` before that function exists.
//! It does not fetch a URL.

#[test]
fn shows_the_mined_blocks() {
    // Shared fee range. Index 3 is 12. A median of 12 uses index 1 (5) as
    // the low end. A median of 4 does not equal index 3, so the low end is
    // index 0 (1). The high end is the last entry (40) either way.
    let fee_range = [1_u64, 5, 8, 12, 40];
    let clock = 1_700_000_000_u64;
    let eight_minutes_ago = clock - 8 * 60;
    let just_now = clock - 30;

    let special = splora_frontend_shared::shows_the_mined_blocks(
        840_000,
        12,
        &fee_range,
        250_000_000,
        1500,
        eight_minutes_ago,
        clock,
        "Foundry",
    );
    let ordinary = splora_frontend_shared::shows_the_mined_blocks(
        840_001, 4, &fee_range, 10_000_000, 3, just_now, clock, "AntPool",
    );
    let one = splora_frontend_shared::shows_the_mined_blocks(
        840_002, 4, &fee_range, 10_000_000, 1, just_now, clock, "ViaBTC",
    );
    let elapsed_zero = splora_frontend_shared::shows_the_mined_blocks(
        840_003, 4, &fee_range, 10_000_000, 3, clock, clock, "ViaBTC",
    );
    let elapsed_59 = splora_frontend_shared::shows_the_mined_blocks(
        840_004,
        4,
        &fee_range,
        10_000_000,
        3,
        clock - 59,
        clock,
        "ViaBTC",
    );
    let elapsed_60 = splora_frontend_shared::shows_the_mined_blocks(
        840_005,
        4,
        &fee_range,
        10_000_000,
        3,
        clock - 60,
        clock,
        "ViaBTC",
    );

    assert_eq!(fee_range[3], 12);
    assert_eq!(
        special,
        [
            "840000".to_string(),
            "~12 sat/vB".to_string(),
            "5.00 - 40.0 sat/vB".to_string(),
            "2.50 BTC".to_string(),
            "1,500 transactions".to_string(),
            "8 minutes ago".to_string(),
            "Foundry".to_string(),
        ]
    );
    assert_eq!(
        ordinary,
        [
            "840001".to_string(),
            "~4 sat/vB".to_string(),
            "1.00 - 40.0 sat/vB".to_string(),
            "0.10 BTC".to_string(),
            "3 transactions".to_string(),
            "Just now".to_string(),
            "AntPool".to_string(),
        ]
    );
    for (left, right) in special.iter().zip(ordinary.iter()) {
        assert_ne!(left, right, "each of the seven strings must differ");
    }
    assert_ne!(
        special[2], ordinary[2],
        "feeRange[1] low and feeRange[0] low must differ"
    );
    assert!(
        special[2].starts_with("5.00"),
        "median equal to feeRange[3] uses feeRange[1], got {}",
        special[2]
    );
    assert!(
        ordinary[2].starts_with("1.00"),
        "median not equal to feeRange[3] uses feeRange[0], got {}",
        ordinary[2]
    );
    assert!(
        !special[3].contains("2.500"),
        "2.5 BTC uses digits 1.2-3, got {}",
        special[3]
    );

    assert_cypress_plural(&special);
    assert_cypress_plural(&ordinary);
    assert_visible_text(&special);
    assert_visible_text(&ordinary);

    assert_eq!(one[4], "1 transaction");
    assert!(
        !matches_transactions_plural(&one[4]),
        "singular transaction text is outside /(.*) transactions/"
    );

    assert_eq!(special[5], "8 minutes ago");
    assert!(special[5].ends_with("ago"));
    assert_eq!(ordinary[5], "Just now");
    assert_eq!(elapsed_zero[5], "Just now");
    assert_eq!(elapsed_59[5], "Just now");
    assert_eq!(elapsed_60[5], "1 minute ago");
    assert!(elapsed_60[5].ends_with("ago"));

    assert_ne!(special[6], ordinary[6]);
    assert_eq!(special[6], "Foundry");
    assert_eq!(ordinary[6], "AntPool");
}

fn assert_cypress_plural(lines: &[String; 7]) {
    assert!(
        matches_height(&lines[0]),
        "height must match /(\\d)/, got {}",
        lines[0]
    );
    assert!(
        matches_fees(&lines[1]),
        "fees must match /~(.*) sat\\/vB/, got {}",
        lines[1]
    );
    assert!(
        matches_fee_span(&lines[2]),
        "fee span must match /(.*) - (.*) sat\\/vB/, got {}",
        lines[2]
    );
    assert!(
        matches_btc(&lines[3]),
        "total fees must match /(.*) BTC/, got {}",
        lines[3]
    );
    assert!(
        matches_transactions_plural(&lines[4]),
        "transaction count must match /(.*) transactions/, got {}",
        lines[4]
    );
    assert!(
        matches_since(&lines[5]),
        "time must match /((.*) ago|Just now)/, got {}",
        lines[5]
    );
    assert!(
        matches_pool(&lines[6]),
        "pool must match /(\\w)/, got {}",
        lines[6]
    );
}

/// Visible strings are not the Cypress attribute names. Those names omit
/// the offset and the index that the template writes into `data-cy`.
fn assert_visible_text(lines: &[String; 7]) {
    for line in lines {
        assert_ne!(line, "bitcoin-block-0-fees");
        assert_ne!(line, "bitcoin-block-0-fee-span");
        assert_ne!(line, "bitcoin-block-0-total-fees");
        assert_ne!(line, "bitcoin-block-0-time");
        assert_ne!(line, "bitcoin-block-0-pool");
    }
}

/// `/(\d)/` searched per line.
fn matches_height(text: &str) -> bool {
    text.split('\n')
        .any(|line| line.chars().any(|ch| ch.is_ascii_digit()))
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

/// `/((.*) ago|Just now)/` searched per line.
fn matches_since(text: &str) -> bool {
    text.split('\n')
        .any(|line| line.contains("Just now") || line.contains(" ago"))
}

/// `/(\w)/` searched per line.
fn matches_pool(text: &str) -> bool {
    text.split('\n').any(|line| {
        line.chars()
            .any(|ch| ch.is_ascii_alphanumeric() || ch == '_')
    })
}
