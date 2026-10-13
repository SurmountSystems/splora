//! One HTML row per block, transaction, or utxo.
//!
//! Field sentences stay available for the screen text helpers. These functions
//! are what the views put on the page.

use splora_frontend_shared::wire::{
    AddressStats, Block, Issuance, RecentTransaction, Transaction, TxInput, TxOutput, Utxo,
    recent_value_text,
};
use splora_frontend_shared::{
    BlockRow, IssuedRegistryAsset, NativeRegistryAsset, RegistryAsset, TxRow,
    parse_assets_registry, sat_per_vbyte_from_vsize,
};

use crate::models::{
    parse_address_stats_json, parse_block_json, parse_blocks_json, parse_blocks_tip_height,
    parse_recent_transactions_json, parse_transaction_json, parse_transactions_json,
    parse_utxos_json,
};
use crate::screens::{
    UtxoAmount, closed_notice, confirmed_balance_words, confirmed_utxo_words, plain_number,
    utxo_amount,
};

fn html_escape(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for ch in text.chars() {
        match ch {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&#39;"),
            _ => out.push(ch),
        }
    }
    out
}

fn td(class: &str, text: &str) -> String {
    format!("<td class=\"{class}\">{text}</td>")
}

fn txid_anchor(txid: &str) -> String {
    let escaped = html_escape(txid);
    format!("<a href=\"/tx/{escaped}\">{escaped}</a>")
}

fn block_anchor(hash: &str) -> String {
    let escaped = html_escape(hash);
    format!("<a href=\"/block/{escaped}\">{escaped}</a>")
}

fn height_on_block(height: impl std::fmt::Display, hash: &str) -> String {
    let escaped = html_escape(hash);
    format!("<a href=\"/block/{escaped}\">{height}</a>")
}

fn height_only_anchor(height: impl std::fmt::Display) -> String {
    format!("<a href=\"/block-height/{height}\">{height}</a>")
}

fn address_anchor(address: &str) -> String {
    let escaped = html_escape(address);
    format!("<a class=\"address\" href=\"/address/{escaped}\">{escaped}</a>")
}

fn finite_rate(rate: Option<f64>) -> Option<String> {
    rate.filter(|rate| rate.is_finite())
        .map(|rate| format!("{}", rate))
}

fn paragraph(class: &str, text: &str) -> String {
    format!("<p class=\"{class}\">{text}</p>")
}

fn push_sum(out: &mut String, class: &str, sum: Option<u64>) {
    if let Some(sum) = sum {
        out.push_str(&paragraph(class, &sum.to_string()));
    }
}

/// Chain funded sum. `None` omits the line. `Some(0)` is `Total received 0`.
fn total_received_words(funded: Option<u64>) -> Option<String> {
    funded.map(|sum| format!("Total received {sum}"))
}

/// Chain spent plus mempool spent. Either missing sum omits the line.
/// `Some(0)` plus `Some(0)` is `Total sent 0`. The add is `u128` so it does not wrap.
fn total_sent_words(chain_spent: Option<u64>, mempool_spent: Option<u64>) -> Option<String> {
    match (chain_spent, mempool_spent) {
        (Some(chain_spent), Some(mempool_spent)) => {
            let sum = chain_spent as u128 + mempool_spent as u128;
            Some(format!("Total sent {sum}"))
        }
        _ => None,
    }
}

/// Mempool funded count minus mempool spent count.
/// Both counts are present on a successful parse. Signed, not saturating.
/// `0 - 0` is `Pending UTXOs 0`. A missing count never reaches this function.
fn pending_utxo_words(funded: u64, spent: u64) -> String {
    let count = funded as i128 - spent as i128;
    format!("Pending UTXOs {count}")
}

/// Mempool funded sum minus mempool spent sum.
/// Either missing sum omits the line. A missing sum is not zero.
/// `Some(0)` minus `Some(0)` is `Pending 0`. The difference is signed, not saturating.
fn pending_amount_words(funded: Option<u64>, spent: Option<u64>) -> Option<String> {
    match (funded, spent) {
        (Some(funded), Some(spent)) => {
            let amount = funded as i128 - spent as i128;
            Some(format!("Pending {amount}"))
        }
        _ => None,
    }
}

/// Chain funded plus mempool funded, minus chain spent plus mempool spent.
/// Any missing sum omits the line. A missing sum is not zero. Do not treat
/// a missing sum as zero. All four `Some(0)` is `Balance 0`. Add and subtract
/// as `i128`, so the difference is signed and does not wrap.
fn balance_words(
    chain_funded: Option<u64>,
    chain_spent: Option<u64>,
    mempool_funded: Option<u64>,
    mempool_spent: Option<u64>,
) -> Option<String> {
    match (chain_funded, chain_spent, mempool_funded, mempool_spent) {
        (Some(chain_funded), Some(chain_spent), Some(mempool_funded), Some(mempool_spent)) => {
            let balance = (chain_funded as i128 + mempool_funded as i128)
                - (chain_spent as i128 + mempool_spent as i128);
            Some(format!("Balance {balance}"))
        }
        _ => None,
    }
}

/// Chain tx count plus mempool tx count.
/// Both counts are present on a successful parse. The add is `u128` so it does not wrap.
/// `0 + 0` is `Transactions 0`. A missing count never reaches this function.
fn transactions_words(chain_count: u64, mempool_count: u64) -> String {
    let sum = chain_count as u128 + mempool_count as u128;
    format!("Transactions {sum}")
}

fn table(class: &str, rows: Vec<String>) -> String {
    if rows.is_empty() {
        String::new()
    } else {
        format!(
            "<table class=\"{class}\"><tbody>{}</tbody></table>",
            rows.concat()
        )
    }
}

fn yes_no(value: bool) -> &'static str {
    if value { "true" } else { "false" }
}

fn skip(body: &str) -> bool {
    let trimmed = body.trim();
    trimmed.is_empty() || closed_notice(trimmed).is_some()
}

fn is_notice(line: &str) -> bool {
    line == "unreadable indexer response" || closed_notice(line).is_some()
}

/// Keep indexer failure lines. Drop the flat field sentences.
pub(crate) fn rows_with_notices(fields: &str, rows: &str) -> String {
    let failure = fields
        .lines()
        .any(|line| line.starts_with("indexer error ") || line == crate::FAIL_CLOSED);
    let mut out = String::new();
    for line in fields.lines() {
        if line.trim().is_empty() {
            continue;
        }
        if failure || is_notice(line) {
            out.push_str(&paragraph("notice", &html_escape(line)));
        }
    }
    out.push_str(rows);
    out
}

fn blocks_from(body: &str) -> Vec<Block> {
    if skip(body) {
        return Vec::new();
    }
    parse_blocks_json(body).unwrap_or_default()
}

fn one_block(body: &str) -> Option<Block> {
    if skip(body) {
        return None;
    }
    parse_block_json(body).ok()
}

fn recent_from(body: &str) -> Vec<RecentTransaction> {
    if skip(body) {
        return Vec::new();
    }
    parse_recent_transactions_json(body).unwrap_or_default()
}

fn txs_from(body: &str) -> Vec<Transaction> {
    if skip(body) {
        return Vec::new();
    }
    parse_transactions_json(body).unwrap_or_default()
}

fn one_tx(body: &str) -> Option<Transaction> {
    if skip(body) {
        return None;
    }
    parse_transaction_json(body).ok()
}

fn one_stats(body: &str) -> Option<AddressStats> {
    if skip(body) {
        return None;
    }
    parse_address_stats_json(body).ok()
}

fn utxos_from(body: &str) -> Vec<Utxo> {
    if skip(body) {
        return Vec::new();
    }
    parse_utxos_json(body).unwrap_or_default()
}

fn nonempty(value: Option<&str>) -> Option<&str> {
    value.map(str::trim).filter(|text| !text.is_empty())
}

fn block_row(block: &Block) -> String {
    let row = BlockRow::from(block);
    let mut cells = vec![
        td("height", &height_on_block(row.height, &row.id)),
        td("hash", &block_anchor(&row.id)),
        td("tx-count", &row.tx_count.to_string()),
        td("time", &row.timestamp.to_string()),
        td("size", &block.size.to_string()),
        td("weight", &block.weight.to_string()),
        td("median-time", &block.median_time.to_string()),
    ];
    if let Some(previous) = nonempty(block.previous_block_hash.as_deref()) {
        cells.push(td("previous", &block_anchor(previous)));
    }
    cells.push(td("version", &row.version.to_string()));
    cells.push(td("merkle-root", &html_escape(&row.merkle_root)));
    // A missing key is None. Some(0) still prints. Do not substitute zero.
    if let Some(nonce) = row.nonce {
        cells.push(td("nonce", &nonce.to_string()));
    }
    if let Some(bits) = row.bits {
        cells.push(td("bits", &bits.to_string()));
    }
    if let Some(difficulty) = row.difficulty {
        cells.push(td("difficulty", &plain_number(difficulty)));
    }
    format!("<tr class=\"block-row\">{}</tr>", cells.concat())
}

fn address_span(address: &str) -> String {
    address_anchor(address)
}

fn value_span(value: u64) -> String {
    format!("<span class=\"value\">{value}</span>")
}

fn script_type_span(text: &str) -> String {
    format!(
        "<span class=\"scriptpubkey-type\">{}</span>",
        html_escape(text)
    )
}

fn issuance_span(phrase: &str) -> String {
    format!("<span>{}</span>", html_escape(phrase))
}

/// Input issuance, not a prevout field. A missing amount stays blank.
/// A non-empty commitment is the word Confidential, never the hex.
fn issuance_markup(issuance: &Issuance) -> String {
    let mut out = String::new();
    if !issuance.asset_id.is_empty() {
        out.push_str(&issuance_span(&format!(
            "issuance asset {}",
            issuance.asset_id
        )));
    }
    out.push_str(&issuance_span(&format!(
        "issuance reissuance {}",
        yes_no(issuance.is_reissuance)
    )));
    if let Some(hash) = issuance
        .contract_hash
        .as_deref()
        .filter(|hash| !hash.is_empty())
    {
        out.push_str(&issuance_span(&format!("issuance contract {hash}")));
    }
    if !issuance.asset_entropy.is_empty() {
        out.push_str(&issuance_span(&format!(
            "issuance entropy {}",
            issuance.asset_entropy
        )));
    }
    if let Some(nonce) = issuance
        .asset_blinding_nonce
        .as_deref()
        .filter(|nonce| !nonce.is_empty())
    {
        out.push_str(&issuance_span(&format!("issuance blinding nonce {nonce}")));
    }
    if let Some(phrase) = issuance_amount_phrase(
        "asset",
        issuance.assetamount,
        issuance.assetamountcommitment.as_deref(),
    ) {
        out.push_str(&issuance_span(&phrase));
    }
    if let Some(phrase) = issuance_amount_phrase(
        "token",
        issuance.tokenamount,
        issuance.tokenamountcommitment.as_deref(),
    ) {
        out.push_str(&issuance_span(&phrase));
    }
    out
}

fn issuance_amount_phrase(
    kind: &str,
    amount: Option<u64>,
    commitment: Option<&str>,
) -> Option<String> {
    if let Some(amount) = amount {
        return Some(format!("issuance {kind} amount {amount}"));
    }
    if commitment.is_some_and(|text| !text.is_empty()) {
        return Some(format!("issuance {kind} amount Confidential"));
    }
    None
}

fn input_party(input: &TxInput) -> String {
    let mut inner = txid_anchor(&input.txid);
    inner.push_str(&format!("<span class=\"vout\">{}</span>", input.vout));
    if let Some(prevout) = &input.prevout {
        if let Some(address) = nonempty(prevout.script_pubkey_address.as_deref()) {
            inner.push_str(&address_span(address));
        }
        if let Some(value) = prevout.value {
            inner.push_str(&value_span(value));
        }
        if !prevout.script_pubkey_type.is_empty() {
            inner.push_str(&script_type_span(&format!(
                "input script type {}",
                prevout.script_pubkey_type
            )));
        }
    }
    if let Some(issuance) = &input.issuance {
        inner.push_str(&issuance_markup(issuance));
    }
    // A parsed input always has sequence, including zero. A missing key never
    // reaches here. Coinbase paints only when the bool is true.
    if input.is_coinbase {
        inner.push_str("<span class=\"coinbase\">coinbase</span>");
    }
    inner.push_str(&format!(
        "<span class=\"sequence\">sequence {}</span>",
        input.sequence
    ));
    format!("<span class=\"party\">{inner}</span>")
}

fn output_party(output: &TxOutput) -> String {
    let mut inner = String::new();
    if let Some(address) = nonempty(output.script_pubkey_address.as_deref()) {
        inner.push_str(&address_span(address));
    }
    if output.script_pubkey_type == "op_return" {
        inner.push_str("<span class=\"scriptpubkey-type\">op_return</span>");
        inner.push_str("<span class=\"op-return\">OP_RETURN</span>");
    } else if !output.script_pubkey_type.is_empty() {
        inner.push_str(&script_type_span(&output.script_pubkey_type));
    }
    if let Some(value) = output.value {
        inner.push_str(&value_span(value));
    }
    format!("<span class=\"party\">{inner}</span>")
}

fn tx_parties(tx: &Transaction) -> String {
    let mut parties = String::new();
    for input in &tx.vin {
        parties.push_str(&input_party(input));
    }
    for output in &tx.vout {
        parties.push_str(&output_party(output));
    }
    parties
}

/// Mempool `Math.max(weight / 4, sigops * 5)`, then `Math.round(n * 100) / 100`.
/// Floating division, not the integer virtual-size quotient.
fn adjusted_vsize_text(weight: u32, sigops: u32) -> String {
    let value = (f64::from(weight) / 4.0).max(f64::from(sigops) * 5.0);
    let rounded = (value * 100.0).round() / 100.0;
    plain_number(rounded)
}

/// `tx.size | bytes: 2`: decimal 2, from `B`, powers of 1000. `u32` stays under TB.
/// Divide in f64, then `(value * 100).round() / 100`. Integer half-up paints 1005 as 1.01 kB.
fn size_number_and_unit(size: u32) -> (String, &'static str) {
    let (scale, unit): (f64, &str) = if size < 1_000 {
        (1.0, "B")
    } else if size < 1_000_000 {
        (1_000.0, "kB")
    } else if size < 1_000_000_000 {
        (1_000_000.0, "MB")
    } else {
        (1_000_000_000.0, "GB")
    };
    let value = f64::from(size) / scale;
    let rounded = (value * 100.0).round() / 100.0;
    (plain_number(rounded), unit)
}

/// `tx.weight | wuBytes: 2`, plaintext. Scale 1000, not 1024.
/// The unit comes from the raw weight before rounding. Half away from zero
/// on the hundredths, then the JavaScript number string. `u32` stays under TWU.
fn weight_number_and_unit(weight: u32) -> (String, &'static str) {
    let weight = u64::from(weight);
    let (scale, unit): (f64, &str) = if weight < 1_000 {
        (1.0, "WU")
    } else if weight < 1_000_000 {
        (1_000.0, "kWU")
    } else if weight < 1_000_000_000 {
        (1_000_000.0, "MWU")
    } else if weight < 1_000_000_000_000 {
        (1_000_000_000.0, "GWU")
    } else {
        (1_000_000_000_000.0, "TWU")
    };
    let value = weight as f64 / scale;
    let rounded = (value * 100.0).round() / 100.0;
    (plain_number(rounded), unit)
}

/// Angular `{{ tx.fee | number }}`: DecimalPipe `1.0-3`, en-US grouping.
/// Fee is an integer, so there is no fraction. `0` stays `0`.
fn grouped_integer(value: u64) -> String {
    let digits = value.to_string();
    let mut grouped = String::with_capacity(digits.len() + digits.len() / 3);
    for (index, digit) in digits.chars().rev().enumerate() {
        if index > 0 && index % 3 == 0 {
            grouped.push(',');
        }
        grouped.push(digit);
    }
    grouped.chars().rev().collect()
}

fn tx_row(tx: &Transaction) -> String {
    let row = TxRow::from(tx);
    let confirmed = tx
        .status
        .as_ref()
        .map(|status| status.confirmed)
        .unwrap_or(false);
    let n = adjusted_vsize_text(tx.weight, tx.sigops);
    let (number, unit) = size_number_and_unit(tx.size);
    let (weight_number, weight_unit) = weight_number_and_unit(tx.weight);
    let mut cells = vec![
        td("txid", &txid_anchor(&row.txid)),
        td("fee", &row.fee.to_string()),
        td(
            "preview-fee",
            &format!(
                "<span class=\"label\">Fee</span> {} <span class=\"symbol\">sats</span>",
                grouped_integer(row.fee)
            ),
        ),
        td("confirmed", yes_no(confirmed)),
        td(
            "size",
            &format!("Size {number} <span class=\"symbol\">{unit}</span>"),
        ),
        td("weight", &tx.weight.to_string()),
        td(
            "weight-units",
            &format!("Weight {weight_number} {weight_unit}"),
        ),
        td("virtual-size", &format!("Virtual size {}", tx.weight / 4)),
        td("adjusted-vsize", &format!("Adjusted vsize {n}")),
        td("version", &row.version.to_string()),
        td("locktime", &row.lock_time.to_string()),
        td("sigops-label", "Sigops"),
        td("sigops", &row.sigops.to_string()),
    ];
    if let Some(rate) = finite_rate(row.sat_per_vbyte()) {
        cells.push(td("fee-rate", &rate));
    }
    let confirming_hash = tx
        .status
        .as_ref()
        .and_then(|status| nonempty(status.block_hash.as_deref()));
    if let Some(height) = row.block_height {
        let text = match confirming_hash {
            Some(hash) => height_on_block(height, hash),
            None => height_only_anchor(height),
        };
        cells.push(td("block-height", &text));
    }
    if let Some(status) = tx.status.as_ref() {
        if let Some(hash) = confirming_hash {
            cells.push(td("block-hash", &block_anchor(hash)));
        }
        if let Some(time) = status.block_time {
            cells.push(td("block-time", &time.to_string()));
        }
    }
    let parties = tx_parties(tx);
    if !parties.is_empty() {
        cells.push(format!("<td class=\"parties\">{parties}</td>"));
    }
    format!("<tr class=\"tx-row\">{}</tr>", cells.concat())
}

fn recent_row(tx: &RecentTransaction) -> String {
    let mut cells = vec![
        td("txid", &txid_anchor(&tx.txid)),
        td("fee", &tx.fee.to_string()),
        td("vsize", &tx.vsize.to_string()),
        td("value", &recent_value_text(tx.value, "")),
    ];
    if let Some(rate) = finite_rate(sat_per_vbyte_from_vsize(tx.fee, tx.vsize)) {
        cells.push(td("fee-rate", &rate));
    }
    format!("<tr class=\"tx-row\">{}</tr>", cells.concat())
}

fn utxo_value_cell(utxo: &Utxo) -> String {
    match utxo_amount(utxo) {
        UtxoAmount::Number(amount) => amount.to_string(),
        UtxoAmount::Confidential => "<span class=\"amount\">Confidential</span>".to_string(),
        UtxoAmount::Blank => String::new(),
    }
}

fn utxo_row(utxo: &Utxo) -> String {
    let mut cells = vec![
        td("txid", &txid_anchor(&utxo.txid)),
        td("vout", &utxo.vout.to_string()),
        td("value", &utxo_value_cell(utxo)),
        td("confirmed", yes_no(utxo.status.confirmed)),
    ];
    let confirming_hash = nonempty(utxo.status.block_hash.as_deref());
    if let Some(height) = utxo.status.block_height {
        let text = match confirming_hash {
            Some(hash) => height_on_block(height, hash),
            None => height_only_anchor(height),
        };
        cells.push(td("block-height", &text));
    }
    if let Some(hash) = confirming_hash {
        cells.push(td("block-hash", &block_anchor(hash)));
    }
    if let Some(time) = utxo.status.block_time {
        cells.push(td("block-time", &time.to_string()));
    }
    format!("<tr class=\"utxo-row\">{}</tr>", cells.concat())
}

fn address_summary(stats: &AddressStats) -> String {
    let mut out = String::new();
    if let Some(address) = nonempty(stats.address.as_deref()) {
        out.push_str(&paragraph("address", &address_anchor(address)));
    }
    if let Some(scripthash) = nonempty(stats.scripthash.as_deref()) {
        out.push_str(&paragraph("scripthash", &html_escape(scripthash)));
    }
    out.push_str(&paragraph(
        "chain-tx-count",
        &stats.chain_stats.tx_count.to_string(),
    ));
    push_sum(&mut out, "funded-sum", stats.chain_stats.funded_txo_sum);
    out.push_str(&paragraph(
        "mempool-tx-count",
        &stats.mempool_stats.tx_count.to_string(),
    ));
    push_sum(
        &mut out,
        "mempool-funded-sum",
        stats.mempool_stats.funded_txo_sum,
    );
    out.push_str(&paragraph(
        "chain-funded-txo-count",
        &stats.chain_stats.funded_txo_count.to_string(),
    ));
    out.push_str(&paragraph(
        "chain-spent-txo-count",
        &stats.chain_stats.spent_txo_count.to_string(),
    ));
    push_sum(&mut out, "chain-spent-sum", stats.chain_stats.spent_txo_sum);
    out.push_str(&paragraph(
        "mempool-funded-txo-count",
        &stats.mempool_stats.funded_txo_count.to_string(),
    ));
    out.push_str(&paragraph(
        "mempool-spent-txo-count",
        &stats.mempool_stats.spent_txo_count.to_string(),
    ));
    push_sum(
        &mut out,
        "mempool-spent-sum",
        stats.mempool_stats.spent_txo_sum,
    );
    if let Some(words) = confirmed_balance_words(
        stats.chain_stats.funded_txo_sum,
        stats.chain_stats.spent_txo_sum,
    ) {
        out.push_str(&paragraph("confirmed-balance", &words));
    }
    out.push_str(&paragraph(
        "confirmed-utxos",
        &confirmed_utxo_words(
            stats.chain_stats.funded_txo_count,
            stats.chain_stats.spent_txo_count,
        ),
    ));
    if let Some(words) = total_received_words(stats.chain_stats.funded_txo_sum) {
        out.push_str(&paragraph("total-received", &words));
    }
    if let Some(words) = total_sent_words(
        stats.chain_stats.spent_txo_sum,
        stats.mempool_stats.spent_txo_sum,
    ) {
        out.push_str(&paragraph("total-sent", &words));
    }
    out.push_str(&paragraph(
        "pending-utxos",
        &pending_utxo_words(
            stats.mempool_stats.funded_txo_count,
            stats.mempool_stats.spent_txo_count,
        ),
    ));
    if let Some(words) = pending_amount_words(
        stats.mempool_stats.funded_txo_sum,
        stats.mempool_stats.spent_txo_sum,
    ) {
        out.push_str(&paragraph("pending-amount", &words));
    }
    if let Some(words) = balance_words(
        stats.chain_stats.funded_txo_sum,
        stats.chain_stats.spent_txo_sum,
        stats.mempool_stats.funded_txo_sum,
        stats.mempool_stats.spent_txo_sum,
    ) {
        out.push_str(&paragraph("balance", &words));
    }
    out.push_str(&paragraph(
        "transactions",
        &transactions_words(stats.chain_stats.tx_count, stats.mempool_stats.tx_count),
    ));
    out
}

pub fn recent_rows_markup(recent_json: &str) -> String {
    if exact_empty_array(recent_json) {
        return empty_list("transactions");
    }
    let rows = recent_from(recent_json);
    let rows = splora_frontend_shared::cap_recent_transactions(&rows);
    table("tx-rows", rows.iter().map(recent_row).collect())
}

pub fn block_txid_links_markup(body: &str) -> String {
    if skip(body) {
        return String::new();
    }
    match splora_frontend_shared::parse_block_txids(body) {
        Ok(txids) => txids.iter().map(|txid| txid_anchor(txid)).collect(),
        Err(_) => String::new(),
    }
}

fn empty_list(kind: &str) -> String {
    format!("<p class=\"empty\">There are no {kind}.</p>")
}

fn exact_empty_array(body: &str) -> bool {
    let trimmed = body.trim();
    let Some(inner) = trimmed
        .strip_prefix('[')
        .and_then(|rest| rest.strip_suffix(']'))
    else {
        return false;
    };
    inner.trim().is_empty()
}

fn failure_markup(body: &str) -> Option<String> {
    let trimmed = body.trim();
    if trimmed.is_empty() || trimmed.starts_with('{') || trimmed.starts_with('[') {
        return None;
    }
    if !trimmed
        .lines()
        .any(|line| line.starts_with("indexer error "))
        && trimmed != crate::FAIL_CLOSED
        && !trimmed.contains(' ')
    {
        return None;
    }
    let html = trimmed
        .lines()
        .filter(|line| !line.trim().is_empty())
        .map(|line| paragraph("notice", &html_escape(line)))
        .collect();
    Some(html)
}

pub fn dashboard_rows_markup(blocks_json: &str, recent_json: &str) -> String {
    let mut out = if exact_empty_array(blocks_json) {
        empty_list("blocks")
    } else {
        table(
            "block-rows",
            blocks_from(blocks_json).iter().map(block_row).collect(),
        )
    };
    if exact_empty_array(recent_json) {
        out.push_str(&empty_list("transactions"));
    } else {
        out.push_str(&recent_rows_markup(recent_json));
    }
    out
}

pub fn blocks_rows_markup(blocks_json: &str) -> String {
    if exact_empty_array(blocks_json) {
        return empty_list("blocks");
    }
    table(
        "block-rows",
        blocks_from(blocks_json).iter().map(block_row).collect(),
    )
}

pub fn block_rows_markup(block_json: &str, txs_json: &str) -> String {
    let block_rows = one_block(block_json)
        .map(|block| vec![block_row(&block)])
        .unwrap_or_default();
    let mut out = table("block-rows", block_rows);
    if exact_empty_array(txs_json) {
        out.push_str(&empty_list("transactions"));
    } else {
        out.push_str(&table(
            "tx-rows",
            txs_from(txs_json).iter().map(tx_row).collect(),
        ));
    }
    out.push_str(&liquid_tx_labels(txs_json));
    out
}

/// Sum of output values on one transaction. Every present value is added,
/// including zero. An empty output list is 0. Any missing value omits the
/// total. A missing value is not zero. The add is `u128` so it does not wrap.
/// `valuecommitment` is not an amount.
fn output_total_words(tx: &Transaction) -> Option<String> {
    let mut sum: u128 = 0;
    for output in &tx.vout {
        let Some(value) = output.value else {
            return None;
        };
        sum += value as u128;
    }
    Some(format!("Output total {sum}"))
}

pub fn transaction_row_markup(body: &str) -> String {
    let mut out = match one_tx(body) {
        Some(tx) => {
            let mut markup = table("tx-rows", vec![tx_row(&tx)]);
            if let Some(words) = output_total_words(&tx) {
                markup.push_str(&paragraph("output-total", &words));
            }
            markup
        }
        None => String::new(),
    };
    out.push_str(&liquid_tx_labels(body));
    out
}

/// Tip height minus `status.block_height`, plus 1.
/// A missing block height, a missing status, or a tip body that is empty or
/// not an integer omits the paragraph. The difference is `i128` so it does not wrap.
pub fn confirmation_markup(tx_body: &str, tip_height_body: &str) -> String {
    let Some(block_height) = one_tx(tx_body)
        .and_then(|tx| tx.status)
        .and_then(|status| status.block_height)
    else {
        return String::new();
    };
    let Ok(tip_text) = parse_blocks_tip_height(tip_height_body) else {
        return String::new();
    };
    let Ok(tip_height) = tip_text.parse::<u64>() else {
        return String::new();
    };
    let count = tip_height as i128 - block_height as i128 + 1;
    paragraph("confirmations", &format!("Confirmations {count}"))
}

pub fn summary_rows_markup(body: &str) -> String {
    summary_rows_with_status(body, &[])
}

/// Compact history rows. A matching status body adds confirmation beside the summary fields.
pub fn summary_rows_with_status(body: &str, statuses: &[(String, String)]) -> String {
    if exact_empty_array(body) {
        return empty_list("transactions");
    }
    let Ok(rows) = splora_frontend_shared::parse_address_txs_summary(body) else {
        return String::new();
    };
    let rendered = rows
        .iter()
        .map(|row| {
            let mut cells = vec![
                td("txid", &txid_anchor(&row.txid)),
                td("height", &height_only_anchor(row.height)),
                td("value", &row.value.to_string()),
                td("time", &row.time.to_string()),
                td("tx-position", &row.tx_position.to_string()),
            ];
            if let Some((_, status)) = statuses.iter().find(|(txid, _)| txid == &row.txid) {
                cells.push(td("confirmation", &tx_status_markup(status)));
            }
            format!("<tr class=\"summary-row\">{}</tr>", cells.concat())
        })
        .collect();
    table("summary-rows", rendered)
}

pub fn tx_status_markup(body: &str) -> String {
    if let Some(notice) = byte_notice(body) {
        return notice;
    }
    let Ok(status) = splora_frontend_shared::parse_tx_status(body) else {
        return byte_notice(body).unwrap_or_default();
    };
    let word = if status.confirmed { "true" } else { "false" };
    let mut out = format!("confirmed {word}");
    if let Some(height) = status.block_height {
        out.push(' ');
        match status.block_hash.as_deref().filter(|hash| !hash.is_empty()) {
            Some(hash) => out.push_str(&height_on_block(height, hash)),
            None => out.push_str(&height_only_anchor(height)),
        }
    }
    if let Some(hash) = status.block_hash.as_deref().filter(|hash| !hash.is_empty()) {
        out.push(' ');
        out.push_str(&block_anchor(hash));
    }
    out
}

pub fn mempool_txids_markup(body: &str) -> String {
    if exact_empty_array(body) {
        return empty_list("transactions");
    }
    let Ok(ids) = splora_frontend_shared::parse_mempool_txids(body) else {
        return String::new();
    };
    ids.iter().map(|txid| txid_anchor(txid)).collect()
}

fn paint_outspend(spend: &splora_frontend_shared::wire::Outspend) -> String {
    let mut out = String::from("<p>");
    if spend.spent {
        out.push_str("spent true");
        if let Some(txid) = spend.txid.as_deref().filter(|txid| !txid.is_empty()) {
            out.push(' ');
            out.push_str(&txid_anchor(txid));
        }
    } else {
        out.push_str("spent false");
    }
    if let Some(vin) = spend.vin {
        out.push_str(&format!(" vin {vin}"));
    }
    // A missing status or JSON null is None. false still prints. Do not substitute false.
    if let Some(status) = spend.status.as_ref() {
        out.push(' ');
        out.push_str(&format!(
            "<span class=\"spend-confirmed\">confirmed {}</span>",
            yes_no(status.confirmed)
        ));
    }
    out.push_str("</p>");
    out
}

pub fn outspends_markup(body: &str) -> String {
    if let Some(notice) = failure_markup(body) {
        return notice;
    }
    let Ok(rows) = splora_frontend_shared::parse_outspends(body) else {
        return String::new();
    };
    rows.iter().map(paint_outspend).collect()
}

pub fn batch_outspends_markup(body: &str) -> String {
    if let Some(notice) = byte_notice(body) {
        return notice;
    }
    let Ok(groups) = splora_frontend_shared::parse_txs_outspends(body) else {
        return outspends_markup(body);
    };
    groups
        .iter()
        .flat_map(|group| group.iter().map(paint_outspend))
        .collect()
}

pub fn one_outspend_markup(body: &str) -> String {
    if let Some(notice) = byte_notice(body) {
        return notice;
    }
    match splora_frontend_shared::parse_outspend(body) {
        Ok(spend) => paint_outspend(&spend),
        Err(_) => byte_notice(body).unwrap_or_default(),
    }
}

fn raw_failure_notice(text: &str) -> Option<String> {
    let trimmed = text.trim();
    if trimmed == "not-served" || trimmed.contains("endpoint does not exist") {
        return Some(paragraph("notice", &html_escape(trimmed)));
    }
    if trimmed.contains("indexer error ") || trimmed == crate::FAIL_CLOSED {
        return failure_markup(text);
    }
    None
}

fn encode_hex(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut out = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        out.push(HEX[(byte >> 4) as usize] as char);
        out.push(HEX[(byte & 0x0f) as usize] as char);
    }
    out
}

fn raw_octet_markup(
    body: &[u8],
    class: &str,
    parse: fn(&[u8]) -> Result<Vec<u8>, splora_frontend_shared::ParseError>,
) -> String {
    if body.is_empty() {
        return String::new();
    }
    if let Ok(text) = std::str::from_utf8(body) {
        if let Some(notice) = raw_failure_notice(text) {
            return notice;
        }
    }
    match parse(body) {
        Ok(bytes) if !bytes.is_empty() => {
            format!("<pre class=\"{class}\">{}</pre>", encode_hex(&bytes))
        }
        _ => String::new(),
    }
}

/// Block rows, then the octet-stream from `GET /block/:hash/raw`.
pub fn block_raw_beside(rows: &str, body: &[u8]) -> String {
    let mut out = rows.to_string();
    out.push_str(&raw_octet_markup(
        body,
        "block-raw",
        splora_frontend_shared::parse_block_raw,
    ));
    out
}

/// Transaction rows, then the octet-stream from `GET /tx/:txid/raw`.
pub fn tx_raw_beside(rows: &str, body: &[u8]) -> String {
    let mut out = rows.to_string();
    out.push_str(&raw_octet_markup(
        body,
        "tx-raw",
        splora_frontend_shared::parse_tx_raw,
    ));
    out
}

/// A failed `GET /block-height/:height` body is a notice. A block hash is not preformatted bytes.
pub fn height_body_markup(body: &str) -> String {
    byte_notice(body).unwrap_or_default()
}

/// A failed body is a notice. Hex from the shared parsers stays in a pre of `class`.
pub fn indexed_bytes_markup(body: &str, class: &str) -> String {
    if let Some(notice) = byte_notice(body) {
        return notice;
    }
    let trimmed = body.trim();
    let hex = splora_frontend_shared::parse_block_header(trimmed)
        .or_else(|_| splora_frontend_shared::parse_block_raw_hex(trimmed))
        .or_else(|_| splora_frontend_shared::parse_tx_raw_hex(trimmed))
        .or_else(|_| splora_frontend_shared::parse_merkleblock_proof(trimmed));
    match hex {
        Ok(text) if !text.is_empty() => {
            format!("<pre class=\"{class}\">{}</pre>", html_escape(&text))
        }
        _ => String::new(),
    }
}

pub fn block_txid_index_markup(body: &str) -> String {
    if let Some(notice) = byte_notice(body) {
        return notice;
    }
    let Ok(txid) = splora_frontend_shared::parse_block_txid(body) else {
        return byte_notice(body).unwrap_or_default();
    };
    let hex = txid.len() == 64 && txid.bytes().all(|byte| byte.is_ascii_hexdigit());
    if hex {
        txid_anchor(&txid)
    } else {
        byte_notice(&txid).unwrap_or_default()
    }
}

fn byte_notice(body: &str) -> Option<String> {
    let trimmed = body.trim();
    if trimmed == "not-served" || trimmed.contains("endpoint does not exist") {
        return Some(paragraph("notice", &html_escape(trimmed)));
    }
    failure_markup(body)
}

pub fn merkle_proof_markup(body: &str) -> String {
    let Ok(proof) = splora_frontend_shared::parse_merkle_proof(body) else {
        return failure_markup(body).unwrap_or_default();
    };
    let mut out = format!(
        "<p>{} pos {}</p>",
        height_only_anchor(proof.block_height),
        proof.pos
    );
    for txid in &proof.merkle {
        out.push_str(&txid_anchor(txid));
    }
    out
}

pub fn transaction_hex_markup(body: &str) -> String {
    if let Some(notice) = failure_markup(body) {
        return notice;
    }
    let trimmed = body.trim();
    if trimmed.is_empty() {
        return String::new();
    }
    format!("<pre class=\"tx-hex\">{}</pre>", html_escape(trimmed))
}

pub fn block_status_markup(body: &str) -> String {
    let Ok(status) = splora_frontend_shared::parse_block_status(body) else {
        return failure_markup(body).unwrap_or_default();
    };
    let mut out = if status.in_best_chain {
        "true".to_string()
    } else {
        "false".to_string()
    };
    if let Some(height) = status.height {
        out.push(' ');
        out.push_str(&height.to_string());
    }
    if let Some(next) = status.next_best.as_deref().filter(|hash| !hash.is_empty()) {
        out.push_str(&block_anchor(next));
    }
    out
}

pub fn multi_address_links_markup(rows: &[(String, String)]) -> String {
    let mut out = String::new();
    for (path, _) in rows {
        let Some(address) = path.strip_prefix("/address/") else {
            continue;
        };
        if address.is_empty() || address.contains('/') {
            continue;
        }
        out.push_str(&address_anchor(address));
    }
    out
}

pub fn broadcast_txid_markup(txid: &str) -> String {
    let txid = txid.trim();
    if txid.is_empty() {
        return String::new();
    }
    txid_anchor(txid)
}

pub fn test_txid_markup(body: &str) -> String {
    if let Ok(rows) = splora_frontend_shared::parse_test_txs(body) {
        return rows.iter().map(|row| txid_anchor(&row.txid)).collect();
    }
    // A disallowed row can omit reject-reason. The public parser stops the
    // whole array there. The txid is still on the object, and it still opens /tx/.
    let Ok(rows) = serde_json::from_str::<Vec<serde_json::Value>>(body) else {
        return String::new();
    };
    rows.iter()
        .filter_map(|row| row.get("txid").and_then(|txid| txid.as_str()))
        .filter(|txid| !txid.is_empty())
        .map(txid_anchor)
        .collect()
}

pub fn package_txid_markup(body: &str) -> String {
    let Ok(result) = splora_frontend_shared::parse_package(body) else {
        return String::new();
    };
    let mut out = html_escape(&result.package_msg);
    for tx in result.tx_results.values() {
        out.push_str(&txid_anchor(&tx.txid));
        // An empty other-wtxid is not a witness id. A missing one is not the txid.
        if let Some(wtxid) = tx.other_wtxid.as_deref().filter(|wtxid| !wtxid.is_empty()) {
            out.push_str(&html_escape(wtxid));
        }
        // A missing key, JSON null, and an empty string omit the label. The text 0 still prints.
        if let Some(error) = tx.error.as_deref().filter(|error| !error.is_empty()) {
            out.push_str(&format!(
                "<span class=\"package-error\">error {}</span>",
                html_escape(error)
            ));
        }
    }
    out
}

/// When the tip hash is already known, the height opens that block.
pub fn chain_tip_markup(hash_body: &str, height_body: &str) -> String {
    let hash = hash_body.trim();
    let known = hash.len() == 64
        && !hash.contains('\n')
        && !hash.starts_with("indexer error ")
        && hash.bytes().all(|byte| byte.is_ascii_hexdigit());
    if known {
        let mut out = block_anchor(hash);
        if let Ok(height) = height_body.trim().parse::<u64>() {
            out.push_str(&height_on_block(height, hash));
        }
        return out;
    }
    tip_links_markup(hash_body, height_body)
}

pub fn tip_links_markup(hash_body: &str, height_body: &str) -> String {
    let mut out = String::new();
    let hash = hash_body.trim();
    if !hash.is_empty()
        && !hash.contains('\n')
        && !hash.starts_with("indexer error ")
        && !hash.starts_with('{')
    {
        out.push_str(&block_anchor(hash));
    }
    if let Ok(height) = height_body.trim().parse::<u64>() {
        out.push_str(&height_only_anchor(height));
    }
    out
}

/// Liquid Bitcoin on the liquid network. Liquid testnet uses the issuance heading.
const NATIVE_LIQUID_ASSET: &str =
    "6f0279e9ed041c3d710a9f57d0c02928416460c4b722ae3457a11eec381c526d";

fn asset_transaction_heading(asset_id: Option<&str>) -> &'static str {
    if asset_id == Some(NATIVE_LIQUID_ASSET) {
        "Peg In/Out and Burn Transactions"
    } else {
        "Issuance and Burn Transactions"
    }
}

/// en-US `DecimalPipe` grouping. `1,234` not `1234`. No fraction on an integer.
fn grouped_count(value: u64) -> String {
    let digits = value.to_string();
    let mut reversed = String::with_capacity(digits.len() + digits.len() / 3);
    for (index, ch) in digits.chars().rev().enumerate() {
        if index > 0 && index.is_multiple_of(3) {
            reversed.push(',');
        }
        reversed.push(ch);
    }
    reversed.chars().rev().collect()
}

fn stats_tx_count(asset: &serde_json::Value, key: &str) -> u64 {
    asset
        .get(key)
        .and_then(|stats| stats.get("tx_count"))
        .and_then(|count| count.as_u64())
        .unwrap_or(0)
}

/// Input and output rows for `GET /asset/:id/txs`. Not an unblinded asset box.
fn asset_transaction_tables(txs_json: &str) -> String {
    let mut out = String::new();
    for tx in txs_from(txs_json) {
        if tx.vin.is_empty() && tx.vout.is_empty() {
            continue;
        }
        out.push_str("<table class=\"table-tx-vin\"><tbody>");
        for input in &tx.vin {
            out.push_str("<tr><td>");
            out.push_str(&input_party(input));
            out.push_str("</td></tr>");
        }
        out.push_str("</tbody></table><table class=\"table-tx-vout\"><tbody>");
        for output in &tx.vout {
            out.push_str("<tr><td>");
            out.push_str(&output_party(output));
            out.push_str("</td></tr>");
        }
        out.push_str("</tbody></table>");
    }
    out
}

pub fn asset_markup(asset_json: &str, txs_json: &str, liquid: bool) -> String {
    if !liquid {
        return String::new();
    }
    let trimmed = asset_json.trim();
    if trimmed == "not-served" || trimmed.contains("endpoint does not exist") {
        return paragraph("notice", &html_escape(trimmed));
    }
    let mut out = String::new();
    if let Ok(value) = serde_json::from_str::<serde_json::Value>(trimmed) {
        if let Some(name) = value.get("name").and_then(|item| item.as_str()) {
            out.push_str(&html_escape(name));
        }
        if let Some(ticker) = value.get("ticker").and_then(|item| item.as_str()) {
            if !out.is_empty() {
                out.push(' ');
            }
            out.push_str(&html_escape(ticker));
        }
        let asset_id = value.get("asset_id").and_then(|item| item.as_str());
        // `asset.component.html` shows this id in `.tx-link`. Not a price or an amount.
        if let Some(id) = asset_id.filter(|id| !id.is_empty()) {
            out.push_str("<div class=\"tx-link\">");
            out.push_str(&html_escape(id));
            out.push_str("</div>");
        }
        out.push_str(&asset_chain_stats_table(trimmed));
        let loaded = txs_from(txs_json);
        out.push_str("<h2>");
        if !loaded.is_empty() {
            let total = stats_tx_count(&value, "chain_stats")
                .saturating_add(stats_tx_count(&value, "mempool_stats"));
            out.push_str(&grouped_count(loaded.len() as u64));
            out.push_str(" of ");
            out.push_str(&grouped_count(total));
            out.push_str("&nbsp;");
        }
        out.push_str(asset_transaction_heading(asset_id));
        out.push_str("</h2>");
    } else if !trimmed.is_empty() {
        out.push_str(&paragraph("notice", &html_escape(trimmed)));
    }
    if let Ok(txs) = parse_transactions_json(txs_json) {
        for tx in txs {
            out.push_str(&txid_anchor(&tx.txid));
        }
    }
    out.push_str(&asset_transaction_tables(txs_json));
    out
}

/// Chain stats from one `GET /asset/:id` document.
///
/// Integers stay the chain values. Mempool issued, burned, pegged-in, and
/// pegged-out amounts are not added. A blinded issuance is the word
/// Confidential. A zero issued or pegged figure stays off the row the asset
/// page hides. Precision does not scale the amount, and a ticker is not appended.
fn asset_chain_stats_table(asset_json: &str) -> String {
    let wrapped = format!("[{}]", asset_json.trim());
    let Ok(rows) = parse_assets_registry(&wrapped) else {
        return String::new();
    };
    let Some(asset) = rows.into_iter().next() else {
        return String::new();
    };
    let rows = match asset {
        RegistryAsset::Issued(issued) => issued_stat_rows(&issued),
        RegistryAsset::Native(native) => pegged_stat_rows(&native),
    };
    table("table table-borderless table-striped", rows)
}

fn stat_row(label: &str, value_html: &str) -> String {
    format!("<tr><td>{label}</td><td>{value_html}</td></tr>")
}

fn stats_amount(blinded: bool, amount: u64) -> String {
    if blinded {
        "Confidential".to_string()
    } else {
        amount.to_string()
    }
}

fn issued_stat_rows(issued: &IssuedRegistryAsset) -> Vec<String> {
    let blinded =
        issued.chain_stats.has_blinded_issuances || issued.mempool_stats.has_blinded_issuances;
    let mut rows = Vec::new();
    if let Some(precision) = issued.precision {
        rows.push(stat_row("Precision", &precision.to_string()));
    }
    if let Some(domain) = issued
        .entity
        .get("domain")
        .and_then(|value| value.as_str())
        .filter(|domain| !domain.is_empty())
    {
        rows.push(stat_row("Issuer", &html_escape(domain)));
    }
    let issuance = &issued.issuance_txin.txid;
    if !issuance.is_empty() {
        rows.push(stat_row("Issuance TX", &txid_anchor(issuance)));
    }
    let chain = &issued.chain_stats;
    if chain.issued_amount != 0 {
        rows.push(stat_row(
            "Issued amount",
            &stats_amount(blinded, chain.issued_amount),
        ));
    }
    rows.push(stat_row(
        "Burned amount",
        &stats_amount(blinded, chain.burned_amount),
    ));
    if chain.issued_amount != 0 {
        let circulating = chain.issued_amount.saturating_sub(chain.burned_amount);
        rows.push(stat_row(
            "Circulating amount",
            &stats_amount(blinded, circulating),
        ));
    }
    rows
}

fn pegged_stat_rows(native: &NativeRegistryAsset) -> Vec<String> {
    let chain = &native.chain_stats;
    let mut rows = Vec::new();
    if chain.peg_in_amount != 0 {
        rows.push(stat_row("Pegged in", &chain.peg_in_amount.to_string()));
    }
    if chain.peg_out_amount != 0 {
        rows.push(stat_row("Pegged out", &chain.peg_out_amount.to_string()));
    }
    rows.push(stat_row("Burned amount", &chain.burned_amount.to_string()));
    if chain.peg_in_amount != 0 {
        let circulating = chain
            .peg_in_amount
            .saturating_sub(chain.peg_out_amount)
            .saturating_sub(chain.burned_amount);
        rows.push(stat_row("Circulating amount", &circulating.to_string()));
    }
    rows
}

pub fn scripthash_markup(body: &str) -> String {
    match splora_frontend_shared::parse_scripthash(body) {
        Ok(stats) => {
            let mut out = paragraph("scripthash", &html_escape(&stats.scripthash));
            out.push_str(&paragraph(
                "chain-tx-count",
                &html_escape(&stats.chain_tx_count),
            ));
            if let Some(funded) = stats.funded_sum.as_deref() {
                out.push_str(&paragraph("funded-sum", &html_escape(funded)));
            }
            out.push_str(&paragraph(
                "mempool-tx-count",
                &html_escape(&stats.mempool_tx_count),
            ));
            if let Ok(wire) = parse_address_stats_json(body) {
                if let Some(words) = confirmed_balance_words(
                    wire.chain_stats.funded_txo_sum,
                    wire.chain_stats.spent_txo_sum,
                ) {
                    out.push_str(&paragraph("confirmed-balance", &words));
                }
                out.push_str(&paragraph(
                    "confirmed-utxos",
                    &confirmed_utxo_words(
                        wire.chain_stats.funded_txo_count,
                        wire.chain_stats.spent_txo_count,
                    ),
                ));
                if let Some(words) = total_received_words(wire.chain_stats.funded_txo_sum) {
                    out.push_str(&paragraph("total-received", &words));
                }
                if let Some(words) = total_sent_words(
                    wire.chain_stats.spent_txo_sum,
                    wire.mempool_stats.spent_txo_sum,
                ) {
                    out.push_str(&paragraph("total-sent", &words));
                }
                out.push_str(&paragraph(
                    "pending-utxos",
                    &pending_utxo_words(
                        wire.mempool_stats.funded_txo_count,
                        wire.mempool_stats.spent_txo_count,
                    ),
                ));
                if let Some(words) = pending_amount_words(
                    wire.mempool_stats.funded_txo_sum,
                    wire.mempool_stats.spent_txo_sum,
                ) {
                    out.push_str(&paragraph("pending-amount", &words));
                }
                if let Some(words) = balance_words(
                    wire.chain_stats.funded_txo_sum,
                    wire.chain_stats.spent_txo_sum,
                    wire.mempool_stats.funded_txo_sum,
                    wire.mempool_stats.spent_txo_sum,
                ) {
                    out.push_str(&paragraph("balance", &words));
                }
                out.push_str(&paragraph(
                    "transactions",
                    &transactions_words(wire.chain_stats.tx_count, wire.mempool_stats.tx_count),
                ));
            }
            out
        }
        Err(_) => failure_markup(body).unwrap_or_default(),
    }
}

pub fn address_rows_markup(stats_json: &str, txs_json: &str, utxo_json: &str) -> String {
    let mut out = one_stats(stats_json)
        .map(|stats| address_summary(&stats))
        .unwrap_or_default();
    if exact_empty_array(txs_json) {
        out.push_str(&empty_list("transactions"));
    } else {
        out.push_str(&table(
            "tx-rows",
            txs_from(txs_json).iter().map(tx_row).collect(),
        ));
    }
    if exact_empty_array(utxo_json) {
        out.push_str(&empty_list("unspent outputs"));
    } else {
        out.push_str(&table(
            "utxo-rows",
            utxos_from(utxo_json).iter().map(utxo_row).collect(),
        ));
    }
    out.push_str(&liquid_tx_labels(txs_json));
    out
}

/// Peg-in, peg-out, asset id, and the confidential label from the parsed transaction.
///
/// `TxInput.is_pegin` and `TxOutput.pegout` are on the shared transaction.
/// A missing `is_pegin` is None and does not paint a peg-in.
/// `Some(false)` does not paint a peg-in. `Some(true)` paints Peg-in.
/// A missing `pegout` is none. `TxOutput.asset`,
/// `value`, `valuecommitment`, and `assetcommitment` are on the shared output.
/// A missing value is none, not zero. A commitment is not an amount.
fn liquid_tx_labels(body: &str) -> String {
    let mut out = parsed_peg_labels(body);
    if let Some(tx) = one_tx(body) {
        out.push_str(&asset_labels_from_tx(&tx));
        return out;
    }
    for tx in txs_from(body) {
        out.push_str(&asset_labels_from_tx(&tx));
    }
    out
}

fn parsed_peg_labels(body: &str) -> String {
    if let Some(tx) = one_tx(body) {
        return peg_labels_from_tx(&tx);
    }
    txs_from(body).iter().map(peg_labels_from_tx).collect()
}

fn peg_labels_from_tx(tx: &Transaction) -> String {
    let mut vin_rows = String::new();
    for input in &tx.vin {
        if input.is_pegin != Some(true) {
            continue;
        }
        let link = if input.txid.is_empty() {
            String::new()
        } else {
            txid_anchor(&input.txid)
        };
        vin_rows.push_str(&format!("<tr><td><span>Peg-in</span>{link}</td></tr>"));
    }
    let mut vout_rows = String::new();
    for output in &tx.vout {
        let Some(pegout) = output.pegout.as_ref() else {
            continue;
        };
        let link = pegout
            .script_pubkey_address
            .as_deref()
            .filter(|address| !address.is_empty())
            .map(address_anchor)
            .unwrap_or_default();
        vout_rows.push_str(&format!(
            "<tr><td>Peg-out to {link}<span class=\"badge\">Liquid Peg Out</span></td></tr>"
        ));
    }
    vin_vout_tables(&vin_rows, &vout_rows)
}

fn vin_vout_tables(vin_rows: &str, vout_rows: &str) -> String {
    let mut out = String::new();
    if !vin_rows.is_empty() {
        out.push_str("<table class=\"table-tx-vin\"><tbody>");
        out.push_str(vin_rows);
        out.push_str("</tbody></table>");
    }
    if !vout_rows.is_empty() {
        out.push_str("<table class=\"table-tx-vout\"><tbody>");
        out.push_str(vout_rows);
        out.push_str("</tbody></table>");
    }
    out
}

fn asset_labels_from_tx(tx: &Transaction) -> String {
    let mut vin_rows = String::new();
    for input in &tx.vin {
        if let Some(prevout) = input.prevout.as_ref() {
            if let Some(cell) = output_asset_cell(prevout) {
                vin_rows.push_str(&format!("<tr><td>{cell}</td></tr>"));
            }
        }
    }
    let mut vout_rows = String::new();
    for output in &tx.vout {
        if let Some(cell) = output_asset_cell(output) {
            vout_rows.push_str(&format!("<tr><td>{cell}</td></tr>"));
        }
    }
    vin_vout_tables(&vin_rows, &vout_rows)
}

fn output_asset_cell(output: &TxOutput) -> Option<String> {
    let commitment = output
        .valuecommitment
        .as_deref()
        .is_some_and(|text| !text.is_empty())
        || output
            .assetcommitment
            .as_deref()
            .is_some_and(|text| !text.is_empty());
    let mut cell = String::new();
    // A public asset id can sit on a blinded amount. Keep both.
    if output.value.is_none() && commitment {
        cell.push_str("<span class=\"amount\">Confidential</span>");
    }
    if let Some(asset) = output.asset.as_deref().filter(|text| !text.is_empty()) {
        let escaped = html_escape(asset);
        cell.push_str(&format!(
            "<span class=\"amount\"><a href=\"/asset/{escaped}\">{escaped}</a></span>"
        ));
    }
    if cell.is_empty() { None } else { Some(cell) }
}

/// Indexer chain page size. Cypress `ITEMS_PER_PAGE` is the same 25.
pub const BLOCK_TXS_PER_PAGE: u32 = 25;

/// Wide pager window. A phone window of three needs a viewport and is not used here.
pub const BLOCK_PAGER_MAX_PAGES: u32 = 5;

/// `GET /block/:hash/txs` on page 1, then `GET /block/:hash/txs/:start` in steps of 25.
pub fn block_tx_page_path(hash: &str, page: u32) -> String {
    let start = page
        .max(1)
        .saturating_sub(1)
        .saturating_mul(BLOCK_TXS_PER_PAGE);
    if start == 0 {
        format!("/block/{hash}/txs")
    } else {
        format!("/block/{hash}/txs/{start}")
    }
}

/// One click shows the details panel. The next click hides it.
pub fn details_open_after_click(open: bool) -> bool {
    !open
}

fn block_page_count(tx_count: u32) -> u32 {
    if tx_count == 0 {
        1
    } else {
        tx_count.div_ceil(BLOCK_TXS_PER_PAGE)
    }
}

fn page_window(page: u32, page_count: u32, max_pages: u32) -> (u32, u32) {
    if page_count <= max_pages {
        return (1, page_count.max(1));
    }
    let half = max_pages / 2;
    let mut start = page.saturating_sub(half).max(1);
    let mut end = start + max_pages - 1;
    if end > page_count {
        end = page_count;
        start = end - max_pages + 1;
    }
    (start, end)
}

fn pager_item(hash: &str, page: u32, active: bool, label: &str) -> String {
    let start = page.saturating_sub(1).saturating_mul(BLOCK_TXS_PER_PAGE);
    let href = html_escape(&format!("/block/{hash}?start_index={start}"));
    let class = if active {
        "page-item active"
    } else {
        "page-item"
    };
    format!(
        "<li class=\"{class}\"><a class=\"page-link\" href=\"{href}\">{}</a></li>",
        html_escape(label)
    )
}

fn pager_items(hash: &str, page: u32, page_count: u32) -> String {
    let mut out = String::new();
    out.push_str(&pager_item(hash, 1, false, "«"));
    out.push_str(&pager_item(hash, page.saturating_sub(1).max(1), false, "‹"));
    let (start, end) = page_window(page, page_count, BLOCK_PAGER_MAX_PAGES);
    for number in start..=end {
        out.push_str(&pager_item(
            hash,
            number,
            number == page,
            &number.to_string(),
        ));
    }
    let next = (page + 1).min(page_count);
    out.push_str(&pager_item(hash, next, false, "›"));
    out.push_str(&pager_item(hash, page_count, false, "»"));
    out
}

/// Transaction heading and pager from `block.tx_count`, plus the header link for this page of txs.
pub fn block_transactions_markup(block_json: &str, txs_json: &str, page: u32) -> String {
    let Some(block) = one_block(block_json) else {
        return paragraph("notice", "unreadable indexer response");
    };
    let page_count = block_page_count(block.tx_count);
    let page = page.max(1).min(page_count);
    let path = block_tx_page_path(&block.id, page);
    let heading = splora_frontend_shared::transaction_count_heading(block.tx_count);
    let mut out = format!(
        "<div id=\"block-tx-title\" class=\"block-tx-title\" data-path=\"{}\"><h2 class=\"text-start\">{}</h2><nav class=\"pagination-container\"><ul class=\"pagination\">{}</ul></nav></div>",
        html_escape(&path),
        html_escape(&heading),
        pager_items(&block.id, page, page_count)
    );
    for tx in txs_from(txs_json) {
        if tx.txid.is_empty() {
            continue;
        }
        let id = html_escape(&tx.txid);
        out.push_str(&format!(
            "<div class=\"header-bg box\"><a class=\"tx-link\" href=\"/tx/{id}\">{id}</a></div>"
        ));
    }
    out
}

fn highlighted_address(address: &str, highlight: bool) -> String {
    let anchor = address_anchor(address);
    if highlight {
        format!("<span class=\"highlight\">{anchor}</span>")
    } else {
        anchor
    }
}

fn probe_transaction(vin: Vec<TxInput>, vout: Vec<TxOutput>) -> Transaction {
    Transaction {
        txid: String::new(),
        version: 0,
        lock_time: 0,
        vin,
        vout,
        size: 0,
        weight: 0,
        sigops: 0,
        fee: 0,
        status: None,
    }
}

fn output_matches_page(address: &str, output: &TxOutput) -> bool {
    let probe = probe_transaction(Vec::new(), vec![output.clone()]);
    splora_frontend_shared::highlight_counts(address, std::slice::from_ref(&probe))
        .first()
        .is_some_and(|count| count.outputs > 0)
}

fn painted_output(output: &TxOutput, page: &str, highlight: bool) -> String {
    if let Some(address) = nonempty(output.script_pubkey_address.as_deref()) {
        return highlighted_address(address, highlight);
    }
    if highlight {
        return highlighted_address(page, true);
    }
    String::new()
}

/// Inputs and outputs for the address page. `highlight_counts` decides `class="highlight"`.
pub fn address_highlight_markup(address: &str, txs_json: &str) -> String {
    let txs = txs_from(txs_json);
    let counts = splora_frontend_shared::highlight_counts(address, &txs);
    let mut out = String::new();
    for (index, tx) in txs.iter().enumerate() {
        let inputs_allowed = counts.get(index).map(|count| count.inputs).unwrap_or(0);
        let outputs_allowed = counts.get(index).map(|count| count.outputs).unwrap_or(0);
        out.push_str(&format!("<div data-cy=\"tx-{index}\">"));
        out.push_str("<table class=\"table-tx-vin\"><tbody>");
        let mut inputs_left = inputs_allowed;
        for input in &tx.vin {
            out.push_str("<tr><td>");
            if let Some(prevout) = &input.prevout {
                let highlight = inputs_left > 0 && output_matches_page(address, prevout);
                if highlight {
                    inputs_left -= 1;
                }
                out.push_str(&painted_output(prevout, address, highlight));
            }
            out.push_str("</td></tr>");
        }
        out.push_str("</tbody></table>");
        out.push_str("<table class=\"table-tx-vout\"><tbody>");
        let mut outputs_left = outputs_allowed;
        for output in &tx.vout {
            out.push_str("<tr><td>");
            let highlight = outputs_left > 0 && output_matches_page(address, output);
            if highlight {
                outputs_left -= 1;
            }
            out.push_str(&painted_output(output, address, highlight));
            out.push_str("</td></tr>");
        }
        out.push_str("</tbody></table></div>");
    }
    out
}

/// Poisoning banner from `splora_frontend_shared::poison_alerts`. Empty when that comparison finds none.
pub fn poison_alert_markup(tx_json: &str, network: &str) -> String {
    let Ok(tx) = parse_transaction_json(tx_json) else {
        return String::new();
    };
    let alerts = splora_frontend_shared::poison_alerts(&tx, network);
    if alerts.is_empty() {
        return String::new();
    }
    let mut out = String::from(
        "<div class=\"alert alert-mempool\" role=\"alert\"><span>Warning! This transaction involves deceptively similar addresses. It may be an address poisoning attack.</span></div>",
    );
    for alert in alerts {
        let prefix = html_escape(&alert.parts.prefix);
        let infix = html_escape(&alert.parts.infix);
        let postfix = html_escape(&alert.parts.postfix);
        out.push_str(&format!(
            "<span class=\"prefix\">{prefix}</span><span class=\"infix\">{infix}</span><span class=\"postfix\">{postfix}</span><span class=\"poison-alert\"></span>"
        ));
    }
    out
}

/// Version, bits, merkle root, difficulty, and nonce. Closed markup keeps `hidden` on `#details`.
pub fn block_details_markup(block_json: &str, open: bool) -> String {
    let hidden = if open { "" } else { " hidden" };
    let Some(block) = one_block(block_json) else {
        return format!(
            "<div id=\"details\"{hidden}><p class=\"notice\">unreadable indexer response</p></div>"
        );
    };
    let id = html_escape(&block.id);
    let mut rows = format!("<tr><td>Version</td><td>{}</td></tr>", block.version);
    // A missing key is None. Some(0) still prints. Do not substitute zero.
    if let Some(bits) = block.bits {
        rows.push_str(&format!("<tr><td>Bits</td><td>{bits}</td></tr>"));
    }
    rows.push_str(&format!(
        "<tr><td>Merkle root</td><td>{}</td></tr>",
        html_escape(&block.merkle_root)
    ));
    if let Some(difficulty) = block.difficulty {
        let difficulty = plain_number(difficulty);
        rows.push_str(&format!(
            "<tr><td>Difficulty</td><td>{difficulty}</td></tr>"
        ));
    }
    if let Some(nonce) = block.nonce {
        rows.push_str(&format!("<tr><td>Nonce</td><td>{nonce}</td></tr>"));
    }
    format!(
        "<div id=\"details\"{hidden}><table><tbody>{rows}</tbody></table><a href=\"/block/{id}/header\">Block Header Hex</a></div>"
    )
}

#[cfg(test)]
mod tests {
    use super::rows_with_notices;

    #[test]
    fn notices_do_not_keep_the_flat_sentence() {
        let fields = "block abc height 1 tx_count 2\nunreadable indexer response";
        let rows = "<tr class=\"block-row\"><td class=\"height\">1</td></tr>";
        let combined = rows_with_notices(fields, rows);
        assert!(combined.contains("unreadable indexer response"));
        assert!(combined.contains("block-row"));
        assert!(!combined.contains("tx_count "));
        assert!(!combined.contains(" height "));
        assert!(combined.contains("<p class=\"notice\">"));
    }
}
