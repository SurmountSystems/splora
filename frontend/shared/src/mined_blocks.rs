//! Seven visible strings for one mined block on the mining dashboard.
//!
//! Order is the Cypress `shows the mined blocks` checks: height, median
//! fee, fee-range span, total fees, transaction count, time since the
//! block, and pool name. Arguments are caller-supplied. This does not
//! fetch a URL.
//!
//! Median fee, the span ends, total fees, and the transaction count use
//! `shows_the_mempool_blocks`, which already matches `rounding="1.0-0"`,
//! the default fee pipe (`dp` 3), amount digits `1.2-3`, and the `number`
//! pipe. The until-minute phrase from that call is discarded. Mined blocks
//! use `app-time` kind `since` instead.
//!
//! The span low is `feeRange[1]` when `medianFee` equals `feeRange[3]`.
//! Otherwise it is `feeRange[0]`. The high end is the last `feeRange`
//! entry. Height is the raw block height. The pool string is the
//! centralised `app-mining-pool` name.

/// Seven visible strings for one mined block, from caller-supplied fields.
///
/// `total_fee_sats` is the amount component's satoshi input. `block_timestamp`
/// and `clock_unix` are both unix seconds, so the phrase does not read the
/// wall clock. `pool_name` is the visible pool name, not a slug looked up
/// from a pool list.
pub fn shows_the_mined_blocks(
    height: u64,
    median_fee: u64,
    fee_range: &[u64],
    total_fee_sats: u64,
    transaction_count: u64,
    block_timestamp: u64,
    clock_unix: u64,
    pool_name: &str,
) -> [String; 7] {
    let low = min_block_fee(median_fee, fee_range);
    let high = max_block_fee(fee_range);
    // Minutes are unused. Mined time is kind `since`, not `until`.
    let [median_text, span_text, total_text, count_text, _] =
        crate::mempool_blocks::shows_the_mempool_blocks(
            median_fee,
            low,
            high,
            total_fee_sats,
            transaction_count,
            0,
        );
    [
        height.to_string(),
        median_text,
        span_text,
        total_text,
        count_text,
        format_since(block_timestamp, clock_unix),
        pool_name.to_string(),
    ]
}

/// `getMinBlockFee`: index 1 when the median equals index 3, else index 0.
fn min_block_fee(median_fee: u64, fee_range: &[u64]) -> u64 {
    if fee_range.is_empty() {
        return 0;
    }
    if fee_range.len() > 3 && median_fee == fee_range[3] {
        fee_range[1]
    } else {
        fee_range[0]
    }
}

/// `getMaxBlockFee`: the last `feeRange` entry, or 0 when the range is empty.
fn max_block_fee(fee_range: &[u64]) -> u64 {
    fee_range.last().copied().unwrap_or(0)
}

/// `TimeService.calculate` for kind `since`, precision 1, minUnit minute.
///
/// Elapsed under 60 seconds, including a clock earlier than the block, is
/// `Just now`. A longer elapsed time is one unit phrase ending in `ago`.
fn format_since(block_timestamp: u64, clock_unix: u64) -> String {
    let Some(seconds) = clock_unix.checked_sub(block_timestamp) else {
        return "Just now".to_string();
    };
    if seconds < 60 {
        return "Just now".to_string();
    }
    for (index, unit) in SINCE_UNITS.iter().copied().enumerate() {
        let precision_index = (index + 1).min(SINCE_UNITS.len() - 1);
        let mut precision_unit = SINCE_UNITS[precision_index];
        let mut counter = seconds / interval_seconds(unit);
        let precision_counter = round_div(seconds, interval_seconds(precision_unit));
        if precision_counter > precision_threshold(precision_unit) {
            precision_unit = unit;
        }
        if precision_unit == SinceUnit::Minute {
            counter = counter.max(1);
        }
        if counter > 0 {
            let rounded = round_div(seconds, interval_seconds(precision_unit));
            return since_phrase(precision_unit, rounded);
        }
    }
    String::new()
}

/// `Math.round(seconds / interval)` for a positive elapsed time.
fn round_div(seconds: u64, interval: u64) -> u64 {
    (seconds + interval / 2) / interval
}

/// `dates.ts` word plus the `time-since` suffix `ago`.
fn since_phrase(unit: SinceUnit, count: u64) -> String {
    let word = match (unit, count == 1) {
        (SinceUnit::Year, true) => "year",
        (SinceUnit::Year, false) => "years",
        (SinceUnit::Month, true) => "month",
        (SinceUnit::Month, false) => "months",
        (SinceUnit::Week, true) => "week",
        (SinceUnit::Week, false) => "weeks",
        (SinceUnit::Day, true) => "day",
        (SinceUnit::Day, false) => "days",
        (SinceUnit::Hour, true) => "hour",
        (SinceUnit::Hour, false) => "hours",
        (SinceUnit::Minute, true) => "minute",
        (SinceUnit::Minute, false) => "minutes",
        (SinceUnit::Second, true) => "second",
        (SinceUnit::Second, false) => "seconds",
    };
    format!("{count} {word} ago")
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum SinceUnit {
    Year,
    Month,
    Week,
    Day,
    Hour,
    Minute,
    Second,
}

const SINCE_UNITS: [SinceUnit; 7] = [
    SinceUnit::Year,
    SinceUnit::Month,
    SinceUnit::Week,
    SinceUnit::Day,
    SinceUnit::Hour,
    SinceUnit::Minute,
    SinceUnit::Second,
];

fn interval_seconds(unit: SinceUnit) -> u64 {
    match unit {
        SinceUnit::Year => 31_536_000,
        SinceUnit::Month => 2_592_000,
        SinceUnit::Week => 604_800,
        SinceUnit::Day => 86_400,
        SinceUnit::Hour => 3_600,
        SinceUnit::Minute => 60,
        SinceUnit::Second => 1,
    }
}

fn precision_threshold(unit: SinceUnit) -> u64 {
    match unit {
        SinceUnit::Year => 100,
        SinceUnit::Month => 18,
        SinceUnit::Week => 12,
        SinceUnit::Day => 31,
        SinceUnit::Hour => 48,
        SinceUnit::Minute => 90,
        SinceUnit::Second => 90,
    }
}
