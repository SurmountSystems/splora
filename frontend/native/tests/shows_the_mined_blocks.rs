//! Cypress `it('shows the mined blocks')` display strings.
//!
//! Spec: `ref/mempool/frontend/cypress/e2e/mainnet/mining.spec.ts`.
//! Expected lines come from `splora_frontend_shared::shows_the_mined_blocks`.
//! This calls `paint_shows_the_mined_blocks` before that paint exists.
//! It does not fetch a URL and it does not hard-code the seven display strings.

#[test]
fn shows_the_mined_blocks() {
    let fee_range = [1_u64, 5, 8, 12, 40];
    let clock = 1_700_000_000_u64;
    let cases = [
        (
            840_000_u64,
            12_u64,
            250_000_000_u64,
            1500_u64,
            clock - 8 * 60,
            "Foundry",
        ),
        (840_001, 4, 10_000_000, 3, clock - 30, "AntPool"),
        (840_002, 4, 10_000_000, 1, clock - 30, "ViaBTC"),
        (840_003, 4, 10_000_000, 3, clock, "ViaBTC"),
        (840_004, 4, 10_000_000, 3, clock - 59, "ViaBTC"),
        (840_005, 4, 10_000_000, 3, clock - 60, "ViaBTC"),
    ];

    let mut painted_cases = Vec::with_capacity(cases.len());
    for (height, median_fee, total_fee_sats, transaction_count, block_timestamp, pool_name) in cases
    {
        let shared = splora_frontend_shared::shows_the_mined_blocks(
            height,
            median_fee,
            &fee_range,
            total_fee_sats,
            transaction_count,
            block_timestamp,
            clock,
            pool_name,
        );
        let painted = splora_native::paint_shows_the_mined_blocks(
            height,
            median_fee,
            &fee_range,
            total_fee_sats,
            transaction_count,
            block_timestamp,
            clock,
            pool_name,
        );
        assert_eq!(painted, shared);
        painted_cases.push(painted);
    }

    for (left, right) in painted_cases[0].iter().zip(painted_cases[1].iter()) {
        assert_ne!(
            left, right,
            "the seven strings must differ across the two field sets"
        );
    }
    assert_ne!(painted_cases[2][4], painted_cases[1][4]);
    assert_ne!(painted_cases[5][5], painted_cases[3][5]);
}
