//! First mempool block row from a received backlog, or from no response.
//!
//! Cypress checks `#mempool-block-0 > .blockLink` after the skeleton is gone.
//! The loaded template ids that row from the ngFor index (`mempool-block-{{ i }}`).
//! That index is not `MempoolBlock.index`. A received backlog, including an
//! all-zero summary, still renders index 0 and stores that id on the view.
//! No response leaves both flags false and leaves the id empty.

/// Loaded versus still-loading first mempool block.
///
/// `link_id` is owned because the loaded id is formatted from the render index.
/// `String` is not `Copy`, so this view is not `Copy`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FirstMempoolBlock {
    /// `.skeleton-loader` is absent. That is the loaded template branch.
    pub skeleton_gone: bool,
    /// `#mempool-block-0 > .blockLink` is in the document.
    pub block_link_exists: bool,
    /// ngFor render-index DOM id when a backlog was received. Empty otherwise.
    pub link_id: String,
}

/// Received `GET /mempool` summary (`BacklogStats` in `src/new_index/mempool.rs`).
///
/// `count`, `vsize`, and `total_fee` use that struct's integer widths.
/// `fee_histogram_len` is the length of `fee_histogram`. Zeros are a received
/// summary, not a missing response.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MempoolBacklog {
    pub count: u32,
    pub vsize: u32,
    pub total_fee: u64,
    pub fee_histogram_len: usize,
}

/// Project ngFor index 0 from a received backlog.
///
/// The argument is the receipt. An all-zero summary still means the response
/// arrived, so the loaded template renders index 0, including the empty
/// placeholder. The DOM id is `mempool-block-{{ i }}` for that index, stored
/// on `link_id`. Neither flag depends on a positive count, vsize, or histogram
/// length.
pub fn first_mempool_block_from_backlog(stats: MempoolBacklog) -> FirstMempoolBlock {
    // Template `i`, not `MempoolBlock.index`. Not a named link-id constant.
    let render_index = 0;
    let link_id = Some(stats).map(|_| format!("mempool-block-{render_index}"));
    FirstMempoolBlock {
        skeleton_gone: link_id.is_some(),
        block_link_exists: link_id.is_some(),
        link_id: link_id.unwrap_or_default(),
    }
}

/// No backlog response has arrived, so the skeleton is still up.
///
/// This does not call `first_mempool_block_from_backlog`. The id stays empty.
pub fn first_mempool_block_after_skeleton() -> FirstMempoolBlock {
    FirstMempoolBlock {
        skeleton_gone: false,
        block_link_exists: false,
        link_id: String::new(),
    }
}

/// Dashboard click of `#mempool-block-0 > .blockLink`, then the skeleton is gone.
///
/// The dashboard `app-mempool-blocks` does not pass `getHref`, so the anchor
/// uses the component default `/mempool-block/${index}`. Liquid `relativeUrl`
/// leaves that path unprefixed when `BASE_MODULE` is liquid. Index 0 is the
/// row the Cypress selector clicks. The settled loader count is the
/// `waitForSkeletonGone` check. This does not read an indexer body.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoadedFirstMempoolBlock {
    /// Parent id of the clicked row (`mempool-block-0`).
    pub link_id: String,
    /// Child class of `#mempool-block-0 > .blockLink`.
    pub block_link_class: &'static str,
    /// `routerLink` the click follows.
    pub route: String,
    /// `Cypress.$('.skeleton-loader').length` after the click.
    pub skeleton_loader_count: u32,
}

/// The first mempool block link has been clicked and the skeleton is gone.
///
/// `link_id` and `route` are both formatted from render index 0. The class is
/// the template's `blockLink`, not a second index. The loader count is the
/// settled check, not a caller-supplied flag.
pub fn load_first_mempool_block_after_skeleton_loads() -> LoadedFirstMempoolBlock {
    let render_index = 0;
    LoadedFirstMempoolBlock {
        link_id: format!("mempool-block-{render_index}"),
        block_link_class: "blockLink",
        route: format!("/mempool-block/{render_index}"),
        skeleton_loader_count: 0,
    }
}

/// Five strings for one projected mempool block on the mining dashboard.
///
/// Order is the Cypress `shows the mempool blocks` checks: median fee,
/// fee-range span, total fees, transaction count, and the until-minute
/// phrase. Arguments are whole sat/vB fees, total fee in satoshis, the
/// transaction count, and the minute count. This does not fetch a URL.
///
/// Median fee uses `app-fee-rate` `rounding="1.0-0"`. The span ends use
/// `FeeRoundingPipe` with no rounding argument, so `dp` is 3: at least
/// 100 is `1.0-0`, under 10 is `1.2-2`, and the rest is `1.1-1`. Total
/// fees use `app-amount` digits `1.2-3` (at least two fraction digits,
/// at most three). Transaction count uses the `number` pipe, which groups
/// thousands in `en-US`. One transaction and one minute stay singular.
pub fn shows_the_mempool_blocks(
    median_fee: u64,
    fee_range_start: u64,
    fee_range_end: u64,
    total_fee_sats: u64,
    transaction_count: u64,
    minutes: u64,
) -> [String; 5] {
    [
        format!("~{} sat/vB", format_digits_1_0_0(median_fee)),
        format!(
            "{} - {} sat/vB",
            format_fee_span_end(fee_range_start),
            format_fee_span_end(fee_range_end)
        ),
        format!("{} BTC", format_btc_digits_1_2_3(total_fee_sats)),
        format_transaction_count(transaction_count),
        format_until_minutes(minutes),
    ]
}

/// `formatNumber` digits `1.0-0` in `en-US`: a whole number, grouped.
fn format_digits_1_0_0(value: u64) -> String {
    group_en_us(&value.to_string())
}

/// Default `FeeRoundingPipe` when `rounding` is null and `dp` is null.
///
/// `(dp || 3)` is 3. `fee >= 10^(3-1)` uses `1.0-0`. `fee < 10^(3-2)`
/// uses `1.2-2`. Otherwise `1.1-1`.
fn format_fee_span_end(fee: u64) -> String {
    if fee >= 100 {
        format_digits_1_0_0(fee)
    } else if fee < 10 {
        format!("{fee}.00")
    } else {
        format!("{fee}.0")
    }
}

/// `satoshis / 100000000` with digits `1.2-3`.
///
/// Three fraction digits is one step of 100000 satoshis. A remainder of
/// at least half that step rounds away from zero. A trailing third zero
/// is dropped because the minimum fraction length is 2, so 2.5 BTC is
/// `2.50` and not `2.500`.
fn format_btc_digits_1_2_3(sats: u64) -> String {
    const SATS_PER_THOUSANDTH_BTC: u64 = 100_000;
    let mut thousandths = sats / SATS_PER_THOUSANDTH_BTC;
    let remainder = sats % SATS_PER_THOUSANDTH_BTC;
    if remainder >= SATS_PER_THOUSANDTH_BTC / 2 {
        thousandths += 1;
    }
    let whole = thousandths / 1_000;
    let frac = (thousandths % 1_000) as u16;
    let whole_text = group_en_us(&whole.to_string());
    if frac % 10 == 0 {
        format!("{whole_text}.{:02}", frac / 10)
    } else {
        format!("{whole_text}.{frac:03}")
    }
}

/// `nTx | number`, then the singular or plural transaction template.
fn format_transaction_count(count: u64) -> String {
    let grouped = group_en_us(&count.to_string());
    if count == 1 {
        format!("{grouped} transaction")
    } else {
        format!("{grouped} transactions")
    }
}

/// `time-until` around `date-base.minute` or `date-base.minutes`.
fn format_until_minutes(minutes: u64) -> String {
    if minutes == 1 {
        format!("In ~{minutes} minute")
    } else {
        format!("In ~{minutes} minutes")
    }
}

/// `en-US` thousands separators. The integer digits are already non-empty.
fn group_en_us(digits: &str) -> String {
    let mut grouped = String::new();
    for (index, ch) in digits.chars().rev().enumerate() {
        if index > 0 && index % 3 == 0 {
            grouped.push(',');
        }
        grouped.push(ch);
    }
    grouped.chars().rev().collect()
}
