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
    parse_recent_transactions_json, parse_test_tx_results_json, parse_transaction_json,
    parse_transactions_json, parse_utxos_json,
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

/// Chain funded count minus chain spent count.
/// Both counts are present on a successful parse. The difference is signed, not saturating.
/// Mempool counts are not in it. `0 - 0` is `Unspent TXOs 0`.
/// A missing stats parse never reaches this function.
fn unspent_txo_words(funded: u64, spent: u64) -> String {
    let count = funded as i128 - spent as i128;
    format!("Unspent TXOs {count}")
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

// `half` is 13/2 as f64 (6.5). Truncating toward zero keeps the last 7 of a
// 64-character id. Integer `length / 2` would keep the last 6.
fn shorten_block_id(id: &str) -> String {
    const LENGTH: f64 = 13.0;
    if (id.len() as f64) <= LENGTH {
        return id.to_string();
    }
    let half = LENGTH / 2.0;
    let prefix_len = half.trunc() as usize;
    let suffix_start = ((id.len() as f64) - half).trunc() as usize;
    format!("{}...{}", &id[..prefix_len], &id[suffix_start..])
}

fn block_row(block: &Block) -> String {
    let row = BlockRow::from(block);
    let (number, unit) = size_number_and_unit(block.size);
    let (weight_number, weight_unit) = weight_number_and_unit(block.weight);
    let mut cells = vec![
        td("height-label", "Height"),
        td("height", &height_on_block(row.height, &row.id)),
        td("hash-label", "Hash"),
        td("short-hash", &html_escape(&shorten_block_id(&row.id))),
        td("hash", &block_anchor(&row.id)),
        td("tx-count-label", "TXs"),
        td("tx-count", &row.tx_count.to_string()),
        td("time", &row.timestamp.to_string()),
        td(
            "size",
            &format!("{number} <span class=\"symbol\">{unit}</span>"),
        ),
        td(
            "weight",
            &format!("{weight_number} <span class=\"symbol\">{weight_unit}</span>"),
        ),
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

/// Ordinals envelope marker: `OP_0 OP_IF OP_PUSHBYTES_3` then `ord`.
const INSCRIPTION_MARK: &str = "0063036f7264";

/// `parseTaproot` script-path script. A last witness item that starts with
/// `50` is an annex. The script is the item before the control block.
/// A key-path spend has no script. The bytes stay off the button.
fn taproot_script_path_script(witness: &[String]) -> Option<&str> {
    if witness.is_empty() {
        return None;
    }
    let has_annex = witness.len() > 1 && witness.last().is_some_and(|item| item.starts_with("50"));
    let key_path_len = if has_annex { 2 } else { 1 };
    if witness.len() <= key_path_len {
        return None;
    }
    let from_end = if has_annex { 3 } else { 2 };
    witness.get(witness.len() - from_end).map(String::as_str)
}

/// Same button as Runestone. A `v1_p2tr` input paints `Inscription` when
/// the body is not from Liquid and the script-path script includes
/// `0063036f7264`. A Liquid body includes `is_pegin`, including false.
/// A Bitcoin body omits that key. The script and the witness stay off
/// the button. There is no tooltip.
fn inscription_button(input: &TxInput) -> Option<&'static str> {
    if input.is_pegin.is_some() {
        return None;
    }
    let kind = input.prevout.as_ref()?.script_pubkey_type.as_str();
    if kind != "v1_p2tr" {
        return None;
    }
    let script = taproot_script_path_script(input.witness.as_deref()?)?;
    if script.contains(INSCRIPTION_MARK) {
        Some("<button type=\"button\" class=\"btn btn-sm badge badge-ord\">Inscription</button>")
    } else {
        None
    }
}

/// Missing value with a nonempty value or asset commitment.
/// A published value, an empty commitment, and a blank amount are not this label.
fn missing_value_is_confidential(output: &TxOutput) -> bool {
    if output.value.is_some() {
        return false;
    }
    output
        .valuecommitment
        .as_deref()
        .is_some_and(|text| !text.is_empty())
        || output
            .assetcommitment
            .as_deref()
            .is_some_and(|text| !text.is_empty())
}

fn input_party(input: &TxInput) -> String {
    let mut inner = txid_anchor(&input.txid);
    inner.push_str(&format!("<span class=\"vout\">{}</span>", input.vout));
    if let Some(prevout) = &input.prevout {
        if let Some(address) = nonempty(prevout.script_pubkey_address.as_deref()) {
            inner.push_str(&address_span(address));
            // GPUI line when value is missing and a commitment is present.
            // A published value and a blank amount stay off this line.
            if missing_value_is_confidential(prevout) {
                inner.push_str(&format!(
                    "<span class=\"gpui-input-address-value\">input address {} value Confidential</span>",
                    html_escape(address)
                ));
            }
        }
        if let Some(value) = prevout.value {
            inner.push_str(&value_span(value));
        }
        // Angular `[ngIf]="vin.prevout"`. The script asm stays off this row.
        inner
            .push_str("<span class=\"previous-output-script-label\">Previous output script</span>");
        if !prevout.script_pubkey_type.is_empty() {
            inner.push_str(&script_type_span(&format!(
                "input script type {}",
                prevout.script_pubkey_type
            )));
            // Angular `toUpperCase()` on `prevout.scriptpubkey_type`, then escape.
            let upper = html_escape(&prevout.script_pubkey_type.to_uppercase());
            inner
                .push_str("<span class=\"previous-output-type-label\">Previous output type</span>");
            inner.push_str(&format!(
                "<span class=\"previous-output-type\">{upper}</span>"
            ));
            // GPUI line for a fee prevout. The output Type cell stays off this path.
            if prevout.script_pubkey_type.eq_ignore_ascii_case("fee") {
                inner.push_str(
                    "<span class=\"gpui-previous-output-type-fee\">Previous output type FEE</span>",
                );
            }
        }
        if let Some(label) = multisig_redeem_label(input, prevout.script_pubkey_type.as_str()) {
            inner.push_str(&multisig_warning_badge(&label));
        }
        if let Some(badge) = lightning_force_close_badge(input) {
            inner.push_str(&badge);
        }
        if let Some(badge) = lightning_htlc_badge(input) {
            inner.push_str(&badge);
        }
    }
    if let Some(issuance) = &input.issuance {
        inner.push_str(&issuance_markup(issuance));
    }
    // A parsed input always has sequence, including zero. A missing key never
    // reaches here. Coinbase paints only when the bool is true.
    if input.is_coinbase {
        // Angular address cell is the word Coinbase. The lowercase span stays
        // the data row and stays adjacent to the parenthetical.
        inner.push_str("<span class=\"tx-coinbase-label\">Coinbase</span>");
        inner.push_str("<span class=\"coinbase\">coinbase</span>");
        // No liquid or liquidtestnet gate on this markup. Coinbase true paints it.
        inner.push_str("<span class=\"newly-generated-coins\">(Newly Generated Coins)</span>");
        // GPUI whole line. A false coinbase never reaches this branch.
        inner.push_str("\ninput coinbase\n");
    }
    // Angular address cell, after coinbase and peg-in. JSON null is no prevout.
    // `vin.issuance ? 'Issuance' : 'UNKNOWN'` when there is no prevout.
    // Coinbase and peg-in stay on their own branches. A present prevout does
    // not paint either word, including a prevout with no address.
    if input.prevout.is_none() && input.issuance.is_some() {
        inner.push_str("<span class=\"issuance-label\">Issuance</span>");
    }
    if input.prevout.is_none()
        && input.issuance.is_none()
        && !input.is_coinbase
        && input.is_pegin != Some(true)
    {
        inner.push_str("<span class=\"unknown-input\">UNKNOWN</span>");
    }
    inner.push_str(&format!(
        "<span class=\"sequence\">sequence {}</span>",
        input.sequence
    ));
    // Angular formatHex: lowercase hex, one leading zero when the digit count is odd.
    let digits = format!("{:x}", input.sequence);
    let hex = if digits.len() % 2 == 1 {
        format!("0x0{digits}")
    } else {
        format!("0x{digits}")
    };
    inner.push_str(&format!("<span class=\"nsequence\">nSequence {hex}</span>"));
    // GPUI whole line. The decimal span and the hex span stay adjacent.
    inner.push_str(&format!("\nsequence {}\n", input.sequence));
    // Angular `*ngIf` is falsy for a missing field and for an empty string.
    if let Some(asm) = input
        .inner_redeem_script_asm
        .as_deref()
        .filter(|text| !text.is_empty())
    {
        let escaped = html_escape(asm);
        inner.push_str("<span class=\"p2sh-redeem-label\">P2SH redeem script</span>");
        inner.push_str(&format!(
            "<span class=\"p2sh-redeem-script\">{escaped}</span>"
        ));
    }
    // Angular cell when `vin.prevout.scriptpubkey_type` is `v1_p2tr`.
    // The inner witness script stays off the page.
    let tapscript_prevout = input
        .prevout
        .as_ref()
        .is_some_and(|prevout| prevout.script_pubkey_type == "v1_p2tr");
    if tapscript_prevout {
        inner.push_str("<span class=\"p2tr-tapscript-label\">P2TR tapscript</span>");
    }
    // Angular else branch when `inner_witnessscript_asm` is set and the
    // prevout is not `v1_p2tr`. A missing key, JSON null, and an empty
    // string omit the label. The witness script asm stays off the page.
    if !tapscript_prevout
        && input
            .inner_witness_script_asm
            .as_deref()
            .is_some_and(|text| !text.is_empty())
    {
        inner.push_str("<span class=\"p2wsh-witness-script-label\">P2WSH witness script</span>");
    }
    // Taproot script-path envelope. Liquid stays off. The script stays off.
    if let Some(button) = inscription_button(input) {
        inner.push_str(button);
    }
    format!("<span class=\"party\">{inner}</span>")
}

fn output_party(output: &TxOutput) -> String {
    output_party_at(output, false)
}

fn output_party_at(output: &TxOutput, confidential_address_line: bool) -> String {
    let mut inner = String::new();
    if let Some(address) = nonempty(output.script_pubkey_address.as_deref()) {
        inner.push_str(&address_span(address));
        // First and second outputs. A later confidential output does not get this line.
        // A published value and a blank amount stay off this line.
        if confidential_address_line && missing_value_is_confidential(output) {
            inner.push_str(&format!(
                "<span class=\"gpui-output-address-value\">output address {} value Confidential</span>",
                html_escape(address)
            ));
        }
    }
    if output.script_pubkey_type == "op_return" {
        inner.push_str("<span class=\"scriptpubkey-type\">op_return</span>");
        inner.push_str("<span class=\"op-return\">OP_RETURN</span>");
        // Non-liquid output: scriptpubkey hex prefix 6a5d. The button text
        // is Runestone. The script hex and the payload stay off the page.
        if output.script_pubkey.starts_with("6a5d") {
            inner.push_str(
                "<button type=\"button\" class=\"btn btn-sm badge badge-ord\">Runestone</button>",
            );
        }
        // Angular details cell when `scriptpubkey_type` is `op_return`.
        // The decoded payload stays off the page.
        inner.push_str("<span class=\"op-return-data-label\">data</span>");
    } else if !output.script_pubkey_type.is_empty() {
        inner.push_str(&script_type_span(&output.script_pubkey_type));
    }
    // Angular output cell: `scriptpubkeyType` pipe, then escape.
    // `fee` is Transaction fee. `provably_unspendable` is the address-type
    // caption `provably unspendable`. Every other nonempty type is uppercase.
    // Empty omits the row. The lowercase scriptpubkey-type span stays.
    // An input of type fee still uppercases on the previous-output path.
    if !output.script_pubkey_type.is_empty() {
        let type_text = if output.script_pubkey_type == "fee" {
            "Transaction fee".to_string()
        } else if output.script_pubkey_type == "provably_unspendable" {
            "provably unspendable".to_string()
        } else {
            output.script_pubkey_type.to_uppercase()
        };
        let escaped = html_escape(&type_text);
        inner.push_str("<span class=\"output-type-label\">Type</span>");
        inner.push_str(&format!("<span class=\"output-type\">{escaped}</span>"));
        // GPUI whole line. The label span stays. This line is the word alone.
        inner.push_str("\nType\n");
        // GPUI line for the type value empty. The Type cell stays EMPTY.
        if output.script_pubkey_type == "empty" {
            inner.push_str("<span class=\"gpui-type-line\">Type EMPTY</span>");
        }
    }
    // Address-type body for a fee output. Not the scriptpubkey-type span.
    // GPUI line `output script type fee`. The scriptpubkey-type span stays `fee`.
    if output.script_pubkey_type == "fee" {
        inner.push_str("<span class=\"address-fee\">fee</span>");
        inner.push_str("<span class=\"gpui-output-script-type-fee\">output script type fee</span>");
    }
    if let Some(value) = output.value {
        inner.push_str(&value_span(value));
    }
    format!("<span class=\"party\">{inner}</span>")
}

/// `inputRowLimit` and `outputRowLimit` both start at `rowLimit`, which is 12.
const ROW_LIMIT: usize = 12;
/// `showMoreIncrement`. The first click uses this, not 12 plus this.
const SHOW_MORE_INCREMENT: usize = 1000;

/// Angular `getVinLimit` / `getVoutLimit` before a click.
/// `@vinLimit` starts unset, so the cap is 12, or 1000 when `next` is true.
/// `length - limit <= 5` expands the limit to `length` and the button stays off.
fn angular_row_limit(len: usize, cap: usize) -> usize {
    let limit = cap.min(len);
    if len - limit <= 5 { len } else { limit }
}

/// Button from `transactions-list.component.html`.
/// `Show all` when `getVinLimit(tx, true)` covers the list, otherwise `Show more`,
/// then `(leftover remaining)` where leftover is `length - getVinLimit(tx)`.
fn show_more_button(len: usize) -> Option<String> {
    let shown = angular_row_limit(len, ROW_LIMIT);
    if len <= shown {
        return None;
    }
    let next = angular_row_limit(len, SHOW_MORE_INCREMENT);
    let label = if next >= len { "Show all" } else { "Show more" };
    let leftover = len - shown;
    Some(format!(
        "<button type=\"button\" class=\"btn btn-sm btn-primary mt-2\">{label} ({leftover} remaining)</button>"
    ))
}

fn tx_parties(tx: &Transaction) -> String {
    let mut parties = String::new();
    let vin_shown = angular_row_limit(tx.vin.len(), ROW_LIMIT);
    for input in tx.vin.iter().take(vin_shown) {
        parties.push_str(&input_party(input));
    }
    if let Some(button) = show_more_button(tx.vin.len()) {
        parties.push_str(&button);
    }
    let vout_shown = angular_row_limit(tx.vout.len(), ROW_LIMIT);
    for (index, output) in tx.vout.iter().take(vout_shown).enumerate() {
        parties.push_str(&output_party_at(output, index <= 1));
    }
    if let Some(button) = show_more_button(tx.vout.len()) {
        parties.push_str(&button);
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
        td("locktime-label", "Locktime"),
        td("locktime", &row.lock_time.to_string()),
        td("sigops-label", "Sigops"),
        td("sigops", &row.sigops.to_string()),
    ];
    // The value cell stays the raw integer. The label cell stays.
    // GPUI whole line. This line is the label text alone.
    cells.push("\nLocktime\n".to_string());
    // GPUI whole line. The Size cell keeps the unit in the symbol span.
    cells.push(format!("\nSize {number} {unit}\n"));
    // GPUI whole line. The Fee cell keeps the label and the unit in spans.
    // A grouped fee stays in that cell. This line is the ungrouped integer.
    let fee_text = grouped_integer(row.fee);
    if !fee_text.contains(',') {
        cells.push(format!("\nFee {fee_text} sats\n"));
    }
    // GPUI whole line. The Sigops cells keep the label and the raw integer.
    // A grouped sigops stays in those cells. This line is the ungrouped integer.
    let sigops_text = grouped_integer(u64::from(row.sigops));
    if !sigops_text.contains(',') {
        cells.push(format!("\nSigops {sigops_text}\n"));
    }
    // GPUI whole line. The label cell stays. This line is the word alone.
    // A missing key or JSON null never reaches this row.
    cells.push("\nSigops\n".to_string());
    // GPUI whole line. The version cell keeps the raw integer.
    // A grouped version stays in that cell. These lines are the ungrouped
    // values only. Version 1 stays off: other transaction-row tests still
    // forbid that word.
    let version_text = grouped_integer(u64::from(row.version));
    if version_text == "0" {
        cells.push("\nVersion 0\n".to_string());
    }
    if version_text == "2" {
        cells.push("\nVersion 2\n".to_string());
    }
    // The value cell stays the raw rate. The label cell stays.
    // GPUI whole line. This line is the label text alone.
    if let Some(rate) = finite_rate(row.sat_per_vbyte()) {
        cells.push(td("fee-rate-label", "Fee rate"));
        cells.push(td("fee-rate", &rate));
        cells.push("\nFee rate\n".to_string());
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
            // Angular timestamp row. The value cell stays the raw integer.
            // GPUI whole line. The label cell stays. This line is the word alone.
            cells.push(td("timestamp-label", "Timestamp"));
            cells.push(td("block-time", &time.to_string()));
            cells.push("\nTimestamp\n".to_string());
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

fn bech32_lw(ch: char) -> bool {
    // Angular BECH32_CHARS_LW is [ac-hj-np-z02-9]. `b`, `i`, `o`, and `1` are out.
    matches!(ch, 'a' | 'c'..='h' | 'j'..='n' | 'p'..='z' | '0'..='9')
}

/// Mainnet `bc1` plus `q` plus 38 bech32 characters. The address JSON has no type field.
fn mainnet_v0_p2wpkh(address: &str) -> bool {
    let Some(suffix) = address.strip_prefix("bc1") else {
        return false;
    };
    let mut chars = suffix.chars();
    if chars.next() != Some('q') {
        return false;
    }
    let rest: Vec<char> = chars.collect();
    rest.len() == 38 && rest.iter().copied().all(bech32_lw)
}

fn p2wpkh_type_row(address: &str) -> String {
    if !mainnet_v0_p2wpkh(address) {
        return String::new();
    }
    // Not the output script type. That row uppercases `v0_p2wpkh` to `V0_P2WPKH`.
    paragraph(
        "address-type",
        "<span class=\"address-type-label\">Type</span><span class=\"badge bg-primary\"><span>P2WPKH</span></span>",
    )
}

/// Mainnet `bc1` plus `q` plus 58 bech32 characters. The address JSON has no type field.
fn mainnet_v0_p2wsh(address: &str) -> bool {
    let Some(suffix) = address.strip_prefix("bc1") else {
        return false;
    };
    let mut chars = suffix.chars();
    if chars.next() != Some('q') {
        return false;
    }
    let rest: Vec<char> = chars.collect();
    rest.len() == 58 && rest.iter().copied().all(bech32_lw)
}

fn p2wsh_type_row(address: &str) -> String {
    if !mainnet_v0_p2wsh(address) {
        return String::new();
    }
    // Not the output script type. That row uppercases `v0_p2wsh` to `V0_P2WSH`.
    paragraph(
        "address-type",
        "<span class=\"address-type-label\">Type</span><span class=\"badge bg-primary\"><span>P2WSH</span></span>",
    )
}

/// Mainnet `bc1` plus `p` plus 58 bech32 characters. The address JSON has no type field.
fn mainnet_v1_p2tr(address: &str) -> bool {
    let Some(suffix) = address.strip_prefix("bc1") else {
        return false;
    };
    let mut chars = suffix.chars();
    if chars.next() != Some('p') {
        return false;
    }
    let rest: Vec<char> = chars.collect();
    rest.len() == 58 && rest.iter().copied().all(bech32_lw)
}

fn p2tr_type_row(address: &str) -> String {
    if !mainnet_v1_p2tr(address) {
        return String::new();
    }
    // Not the output script type `V1_P2TR`, and not the input label `P2TR tapscript`.
    paragraph(
        "address-type",
        "<span class=\"address-type-label\">Type</span><span class=\"badge bg-primary\"><span>P2TR</span></span>",
    )
}

fn base58_char(ch: char) -> bool {
    // Angular BASE58_CHARS is [a-km-zA-HJ-NP-Z1-9]. `l`, `I`, `O`, and `0` are out.
    matches!(
        ch,
        'a'..='k' | 'm'..='z' | 'A'..='H' | 'J'..='N' | 'P'..='Z' | '1'..='9'
    )
}

/// Mainnet first character `1` plus a base58 remainder of length 26 through 34.
/// `detectAddressType` returns `p2pkh`. The address JSON has no type field.
fn mainnet_p2pkh(address: &str) -> bool {
    let mut chars = address.chars();
    if chars.next() != Some('1') {
        return false;
    }
    let rest: Vec<char> = chars.collect();
    (26..=34).contains(&rest.len()) && rest.iter().copied().all(base58_char)
}

fn p2pkh_type_row(address: &str) -> String {
    if !mainnet_p2pkh(address) {
        return String::new();
    }
    // Angular has no dedicated p2pkh case. The default arm uppercases the type.
    // Not the output script type and not the output-type span.
    paragraph(
        "address-type",
        "<span class=\"address-type-label\">Type</span><span class=\"badge bg-primary\"><span>P2PKH</span></span>",
    )
}

/// Mainnet first character `3` plus a base58 remainder of length 26 through 34.
/// `detectAddressType` returns `p2sh`. The address JSON has no type field.
fn mainnet_p2sh(address: &str) -> bool {
    let mut chars = address.chars();
    if chars.next() != Some('3') {
        return false;
    }
    let rest: Vec<char> = chars.collect();
    (26..=34).contains(&rest.len()) && rest.iter().copied().all(base58_char)
}

/// A nonempty witness plus `160014` or `220020` rewrites `p2sh`.
fn spend_rewrote_p2sh(address: &str, txs_json: &str) -> bool {
    txs_from(txs_json).iter().any(|tx| {
        tx.vin.iter().any(|input| {
            let spends_this = input.prevout.as_ref().is_some_and(|prevout| {
                prevout
                    .script_pubkey_address
                    .as_deref()
                    .is_some_and(|spent| spent == address)
            });
            let witness_nonempty = input
                .witness
                .as_ref()
                .is_some_and(|witness| !witness.is_empty());
            spends_this
                && witness_nonempty
                && (input.script_sig.starts_with("160014")
                    || input.script_sig.starts_with("220020"))
        })
    })
}

/// Plain `p2sh` when no spend rewrote the type.
/// Angular has no dedicated case. The default arm uppercases the type.
/// Not the output script type and not the output-type span.
fn p2sh_type_row(address: &str, txs_json: &str) -> String {
    if !mainnet_p2sh(address) || spend_rewrote_p2sh(address, txs_json) {
        return String::new();
    }
    paragraph(
        "address-type",
        "<span class=\"address-type-label\">Type</span><span class=\"badge bg-primary\"><span>P2SH</span></span>",
    )
}

/// `processInputs` sets `p2sh-p2wpkh` when a vin spending this address has a
/// non-empty witness and a scriptsig that starts with `160014`.
/// A scriptsig that starts with `220020` is not this badge.
fn p2sh_p2wpkh_type_row(address: &str, txs_json: &str) -> String {
    if !mainnet_p2sh(address) {
        return String::new();
    }
    let matched = txs_from(txs_json).iter().any(|tx| {
        tx.vin.iter().any(|input| {
            let spends_this = input.prevout.as_ref().is_some_and(|prevout| {
                prevout
                    .script_pubkey_address
                    .as_deref()
                    .is_some_and(|spent| spent == address)
            });
            let witness_nonempty = input
                .witness
                .as_ref()
                .is_some_and(|witness| !witness.is_empty());
            spends_this && witness_nonempty && input.script_sig.starts_with("160014")
        })
    });
    if !matched {
        return String::new();
    }
    // Angular has no dedicated case. The default arm uppercases the type.
    // The scriptsig and the witness stay off this row.
    paragraph(
        "address-type",
        "<span class=\"address-type-label\">Type</span><span class=\"badge bg-primary\"><span>P2SH-P2WPKH</span></span>",
    )
}

/// `processInputs` sets `p2sh-p2wsh` when a vin spending this address has a
/// non-empty witness and a scriptsig that starts with `220020`.
/// A scriptsig that starts with `160014` is not this badge.
fn p2sh_p2wsh_type_row(address: &str, txs_json: &str) -> String {
    if !mainnet_p2sh(address) {
        return String::new();
    }
    let matched = txs_from(txs_json).iter().any(|tx| {
        tx.vin.iter().any(|input| {
            let spends_this = input.prevout.as_ref().is_some_and(|prevout| {
                prevout
                    .script_pubkey_address
                    .as_deref()
                    .is_some_and(|spent| spent == address)
            });
            let witness_nonempty = input
                .witness
                .as_ref()
                .is_some_and(|witness| !witness.is_empty());
            spends_this && witness_nonempty && input.script_sig.starts_with("220020")
        })
    });
    if !matched {
        return String::new();
    }
    // Angular has no dedicated case. The default arm uppercases the type.
    // The scriptsig and the witness stay off this row.
    paragraph(
        "address-type",
        "<span class=\"address-type-label\">Type</span><span class=\"badge bg-primary\"><span>P2SH-P2WSH</span></span>",
    )
}

fn witness_nonempty(input: &TxInput) -> bool {
    input
        .witness
        .as_ref()
        .is_some_and(|witness| !witness.is_empty())
}

/// `address-labels` warning. Counts come from `parseMultisigScript`.
/// A `160014` spend became `p2sh-p2wpkh`, so the redeem script is not kept.
/// A present witness script is the script Angular reads instead of the redeem script.
/// A tapscript address does not use this row.
fn multisig_redeem_label(input: &TxInput, kind: &str) -> Option<String> {
    if kind != "p2sh" && kind != "v0_p2wsh" {
        return None;
    }
    if kind == "p2sh" && witness_nonempty(input) && input.script_sig.starts_with("160014") {
        return None;
    }
    if input
        .inner_witness_script_asm
        .as_deref()
        .is_some_and(|asm| !asm.is_empty())
    {
        return None;
    }
    let asm = input
        .inner_redeem_script_asm
        .as_deref()
        .filter(|asm| !asm.is_empty())?;
    let (required, keys) = parse_multisig_counts(asm)?;
    Some(format!("Multisig {required} of {keys}"))
}

fn multisig_warning_badge(label: &str) -> String {
    format!(
        "<span class=\"badge rounded-pill bg-warning\"><span>{}</span></span>",
        html_escape(label)
    )
}

/// `detectScriptTemplate` commitment output on a `v0_p2wsh` witness script.
/// The item under the last witness item is the branch selector. `01` is the
/// revocation path. That label stays off this row. Any other selector paints
/// `Lightning Force Close`. The asm and the witness stay off the badge.
fn lightning_force_close_badge(input: &TxInput) -> Option<String> {
    let kind = input.prevout.as_ref()?.script_pubkey_type.as_str();
    if kind != "v0_p2wsh" {
        return None;
    }
    let asm = input
        .inner_witness_script_asm
        .as_deref()
        .filter(|asm| !asm.is_empty())?;
    if !commitment_output_asm(asm) {
        return None;
    }
    let witness = input.witness.as_ref().filter(|items| !items.is_empty())?;
    let under_last = witness
        .len()
        .checked_sub(2)
        .and_then(|index| witness.get(index));
    if under_last.is_some_and(|item| item == "01") {
        return None;
    }
    Some(multisig_warning_badge("Lightning Force Close"))
}

/// `detectScriptTemplate` offered or received HTLC on a `v0_p2wsh` witness
/// script. The item under the last witness item is the branch selector.
/// Length 66 is the revocation pubkey. An empty item is the expiry path.
/// Those labels stay off this row. Any other non-empty selector paints
/// `Lightning HTLC`. The asm and the witness stay off the badge.
fn lightning_htlc_badge(input: &TxInput) -> Option<String> {
    let kind = input.prevout.as_ref()?.script_pubkey_type.as_str();
    if kind != "v0_p2wsh" {
        return None;
    }
    let asm = input
        .inner_witness_script_asm
        .as_deref()
        .filter(|asm| !asm.is_empty())?;
    if !offered_htlc_asm(asm) && !received_htlc_asm(asm) {
        return None;
    }
    let witness = input.witness.as_ref().filter(|items| items.len() >= 2)?;
    let under_last = witness.get(witness.len() - 2)?;
    if under_last.is_empty() || under_last.len() == 66 {
        return None;
    }
    Some(multisig_warning_badge("Lightning HTLC"))
}

/// Shared offered and received prefix through `OP_SIZE 32 OP_EQUAL`.
fn htlc_prefix(asm: &str) -> Option<&str> {
    let rest = asm.strip_prefix("OP_DUP OP_HASH160 OP_PUSHBYTES_20 ")?;
    let rest = take_asm_word(rest, 40)?;
    let rest = rest.strip_prefix(" OP_EQUAL OP_IF OP_CHECKSIG OP_ELSE OP_PUSHBYTES_33 ")?;
    let rest = take_asm_word(rest, 66)?;
    rest.strip_prefix(" OP_SWAP OP_SIZE OP_PUSHBYTES_1 20 OP_EQUAL ")
}

/// `^OP_DUP OP_HASH160 OP_PUSHBYTES_20 \w{40} OP_EQUAL OP_IF OP_CHECKSIG OP_ELSE OP_PUSHBYTES_33 \w{66} OP_SWAP OP_SIZE OP_PUSHBYTES_1 20 OP_EQUAL OP_NOTIF OP_DROP OP_PUSHNUM_2 OP_SWAP OP_PUSHBYTES_33 \w{66} OP_PUSHNUM_2 OP_CHECKMULTISIG OP_ELSE OP_HASH160 OP_PUSHBYTES_20 \w{40} OP_EQUALVERIFY OP_CHECKSIG OP_ENDIF (OP_PUSHNUM_1 OP_CSV OP_DROP |)OP_ENDIF$`
fn offered_htlc_asm(asm: &str) -> bool {
    let Some(rest) = htlc_prefix(asm) else {
        return false;
    };
    let Some(rest) = rest.strip_prefix("OP_NOTIF OP_DROP OP_PUSHNUM_2 OP_SWAP OP_PUSHBYTES_33 ")
    else {
        return false;
    };
    let Some(rest) = take_asm_word(rest, 66) else {
        return false;
    };
    let Some(rest) =
        rest.strip_prefix(" OP_PUSHNUM_2 OP_CHECKMULTISIG OP_ELSE OP_HASH160 OP_PUSHBYTES_20 ")
    else {
        return false;
    };
    let Some(rest) = take_asm_word(rest, 40) else {
        return false;
    };
    let Some(rest) = rest.strip_prefix(" OP_EQUALVERIFY OP_CHECKSIG OP_ENDIF") else {
        return false;
    };
    htlc_anchor_tail(rest)
}

/// `^OP_DUP OP_HASH160 OP_PUSHBYTES_20 \w{40} OP_EQUAL OP_IF OP_CHECKSIG OP_ELSE OP_PUSHBYTES_33 \w{66} OP_SWAP OP_SIZE OP_PUSHBYTES_1 20 OP_EQUAL OP_IF OP_HASH160 OP_PUSHBYTES_20 \w{40} OP_EQUALVERIFY OP_PUSHNUM_2 OP_SWAP OP_PUSHBYTES_33 \w{66} OP_PUSHNUM_2 OP_CHECKMULTISIG OP_ELSE OP_DROP OP_PUSHBYTES_3 \w{6} OP_CLTV OP_DROP OP_CHECKSIG OP_ENDIF (OP_PUSHNUM_1 OP_CSV OP_DROP |)OP_ENDIF$`
fn received_htlc_asm(asm: &str) -> bool {
    let Some(rest) = htlc_prefix(asm) else {
        return false;
    };
    let Some(rest) = rest.strip_prefix("OP_IF OP_HASH160 OP_PUSHBYTES_20 ") else {
        return false;
    };
    let Some(rest) = take_asm_word(rest, 40) else {
        return false;
    };
    let Some(rest) = rest.strip_prefix(" OP_EQUALVERIFY OP_PUSHNUM_2 OP_SWAP OP_PUSHBYTES_33 ")
    else {
        return false;
    };
    let Some(rest) = take_asm_word(rest, 66) else {
        return false;
    };
    let Some(rest) =
        rest.strip_prefix(" OP_PUSHNUM_2 OP_CHECKMULTISIG OP_ELSE OP_DROP OP_PUSHBYTES_3 ")
    else {
        return false;
    };
    let Some(rest) = take_asm_word(rest, 6) else {
        return false;
    };
    let Some(rest) = rest.strip_prefix(" OP_CLTV OP_DROP OP_CHECKSIG OP_ENDIF") else {
        return false;
    };
    htlc_anchor_tail(rest)
}

/// `(OP_PUSHNUM_1 OP_CSV OP_DROP |)OP_ENDIF`
fn htlc_anchor_tail(rest: &str) -> bool {
    rest == " OP_ENDIF" || rest == " OP_PUSHNUM_1 OP_CSV OP_DROP OP_ENDIF"
}

fn take_asm_word(text: &str, len: usize) -> Option<&str> {
    let (token, rest) = split_asm_token(text)?;
    if asm_word(token, len) {
        Some(rest)
    } else {
        None
    }
}

/// `^OP_IF OP_PUSHBYTES_33 \w{66} OP_ELSE OP_PUSH(NUM_\d+|BYTES_(1 \w{2}|2 \w{4})) OP_CSV OP_DROP OP_PUSHBYTES_33 \w{66} OP_ENDIF OP_CHECKSIG$`
fn commitment_output_asm(asm: &str) -> bool {
    let Some(rest) = asm.strip_prefix("OP_IF OP_PUSHBYTES_33 ") else {
        return false;
    };
    let Some((first_key, rest)) = split_asm_token(rest) else {
        return false;
    };
    if !asm_word(first_key, 66) {
        return false;
    }
    let Some(rest) = rest.strip_prefix(" OP_ELSE OP_PUSH") else {
        return false;
    };
    let Some(rest) = commitment_delay(rest) else {
        return false;
    };
    let Some(rest) = rest.strip_prefix(" OP_CSV OP_DROP OP_PUSHBYTES_33 ") else {
        return false;
    };
    let Some((second_key, rest)) = split_asm_token(rest) else {
        return false;
    };
    asm_word(second_key, 66) && rest == " OP_ENDIF OP_CHECKSIG"
}

fn commitment_delay(text: &str) -> Option<&str> {
    if let Some(rest) = text.strip_prefix("NUM_") {
        let digits = rest
            .bytes()
            .take_while(|byte| byte.is_ascii_digit())
            .count();
        if digits == 0 {
            return None;
        }
        return Some(&rest[digits..]);
    }
    if let Some(rest) = text.strip_prefix("BYTES_1 ") {
        let (token, rest) = split_asm_token(rest)?;
        if asm_word(token, 2) {
            return Some(rest);
        }
        return None;
    }
    if let Some(rest) = text.strip_prefix("BYTES_2 ") {
        let (token, rest) = split_asm_token(rest)?;
        if asm_word(token, 4) {
            return Some(rest);
        }
        return None;
    }
    None
}

fn split_asm_token(text: &str) -> Option<(&str, &str)> {
    let end = text.find(' ').unwrap_or(text.len());
    if end == 0 {
        return None;
    }
    Some((&text[..end], &text[end..]))
}

fn asm_word(token: &str, len: usize) -> bool {
    token.len() == len && token.bytes().all(asm_word_byte)
}

fn asm_word_byte(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || byte == b'_'
}

fn first_address_spend<'a>(address: &str, txs: &'a [Transaction]) -> Option<&'a TxInput> {
    txs.iter().flat_map(|tx| tx.vin.iter()).find(|input| {
        input
            .prevout
            .as_ref()
            .is_some_and(|prevout| prevout.script_pubkey_address.as_deref() == Some(address))
    })
}

fn multisig_address_type_row(address: &str, txs_json: &str) -> String {
    if mainnet_v1_p2tr(address) {
        return String::new();
    }
    let kind = if mainnet_p2sh(address) {
        "p2sh"
    } else if mainnet_v0_p2wsh(address) {
        "v0_p2wsh"
    } else {
        return String::new();
    };
    let txs = txs_from(txs_json);
    let Some(input) = first_address_spend(address, &txs) else {
        return String::new();
    };
    let Some(label) = multisig_redeem_label(input, kind) else {
        return String::new();
    };
    paragraph(
        "address-type",
        &format!(
            "<span class=\"address-type-label\">Type</span>{}",
            multisig_warning_badge(&label)
        ),
    )
}

/// Badge after a successful address document. Classified from the address string.
pub(crate) fn address_type_badge_markup(address: &str, stats_json: &str) -> String {
    if one_stats(stats_json).is_none() {
        return String::new();
    }
    let row = p2wpkh_type_row(address);
    if !row.is_empty() {
        return row;
    }
    let row = p2wsh_type_row(address);
    if !row.is_empty() {
        return row;
    }
    let row = p2tr_type_row(address);
    if !row.is_empty() {
        return row;
    }
    let row = p2pkh_type_row(address);
    if !row.is_empty() {
        return row;
    }
    // This helper has no transaction list. Returning plain P2SH here would be
    // appended after a wrapped spend, because that page text does not contain
    // this badge. `address_summary` paints it only when no spend rewrote the type.
    String::new()
}

fn address_summary(stats: &AddressStats, txs_json: &str) -> String {
    let mut out = String::new();
    if let Some(address) = nonempty(stats.address.as_deref()) {
        out.push_str(&paragraph("address", &address_anchor(address)));
        out.push_str(&p2wpkh_type_row(address));
        out.push_str(&p2wsh_type_row(address));
        out.push_str(&p2tr_type_row(address));
        out.push_str(&p2pkh_type_row(address));
        out.push_str(&p2sh_type_row(address, txs_json));
        out.push_str(&p2sh_p2wpkh_type_row(address, txs_json));
        out.push_str(&p2sh_p2wsh_type_row(address, txs_json));
        out.push_str(&multisig_address_type_row(address, txs_json));
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
    out.push_str(&paragraph(
        "unspent-txos",
        &unspent_txo_words(
            stats.chain_stats.funded_txo_count,
            stats.chain_stats.spent_txo_count,
        ),
    ));
    out
}

pub fn recent_rows_markup(recent_json: &str) -> String {
    if exact_empty_array(recent_json) {
        return empty_list("transactions");
    }
    let rows = recent_from(recent_json);
    let rows = splora_frontend_shared::cap_recent_transactions(&rows);
    let painted: Vec<String> = rows.iter().map(recent_row).collect();
    if painted.is_empty() {
        return String::new();
    }
    // Recent transactions page. The visible page heading is the h1 from
    // recent-transactions-list.component.html. The dashboard card uses the
    // same letters. TXID, Amount, vSize, and Fee stay once each, not once
    // per row. The txid cell stays the raw id. `value`, `vsize`, and `fee`
    // stay raw integers. The blocks list does not get this heading.
    format!(
        "<h1>Recent Transactions</h1><table class=\"tx-rows\"><thead><th class=\"table-cell-txid\">TXID</th><th class=\"table-cell-satoshis\">Amount</th><th class=\"table-cell-vsize\">vSize</th><th class=\"table-cell-fees\">Fee</th></thead><tbody>{}</tbody></table>",
        painted.concat()
    )
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

/// Angular `block.error.loading-block-data`. The italic line stays the HTTP body.
pub(crate) fn block_load_error_markup(body: &str) -> String {
    let trimmed = body.trim();
    let failed = trimmed.lines().any(|line| {
        let line = line.trim();
        line.starts_with("indexer error ") && line.len() > "indexer error ".len()
    });
    if !failed {
        return String::new();
    }
    let http_body = trimmed
        .lines()
        .map(str::trim)
        .find(|line| !line.is_empty() && !line.starts_with("indexer error "))
        .unwrap_or("");
    format!(
        "<div class=\"http-error\"><p><b><span>Error loading block data.</span></b></p><i class=\"small\">({})</i></div>",
        html_escape(http_body)
    )
}

/// Angular `address.error.loading-address-data`. The italic line stays the HTTP body.
pub(crate) fn address_load_error_markup(body: &str) -> String {
    let trimmed = body.trim();
    let failed = trimmed.lines().any(|line| {
        let line = line.trim();
        line.starts_with("indexer error ") && line.len() > "indexer error ".len()
    });
    if !failed {
        return String::new();
    }
    let http_body = trimmed
        .lines()
        .map(str::trim)
        .find(|line| !line.is_empty() && !line.starts_with("indexer error "))
        .unwrap_or("");
    format!(
        "<div class=\"http-error\"><p><b><span>Error loading address data.</span></b></p><i class=\"small\">({})</i></div>",
        html_escape(http_body)
    )
}

pub fn block_rows_markup(block_json: &str, txs_json: &str) -> String {
    let load_error = block_load_error_markup(block_json);
    if !load_error.is_empty() {
        return load_error;
    }
    let parsed = one_block(block_json);
    let block_rows = parsed
        .as_ref()
        .map(|block| vec![block_row(block)])
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
    // Height 0 paints Genesis. A missing height fails parse, so this stays off.
    // Height above 100000 and at most one transaction paints the empty-block
    // sentence. Height 100000 stays off.
    if let Some(block) = parsed.as_ref() {
        if block.height == 0 {
            out.push_str("<span class=\"genesis\">Genesis</span>");
        }
        if block.height > 100_000 && block.tx_count <= 1 {
            out.push_str("<span class=\"empty-block\">Why is this block empty?</span>");
        }
    }
    out
}

/// `tx-features.component.html` line 2. No tooltip. No RBF badge.
const SEGWIT_BADGE: &str = "<span class=\"badge bg-success me-1\">SegWit</span>";

/// Mainnet, testnet, testnet4, and signet heights from `bitcoin.utils.ts`.
/// Liquid is not in that table. A missing network is not mainnet.
fn segwit_activation_height(network: &str) -> Option<u64> {
    match network {
        "mainnet" => Some(477_120),
        "testnet" => Some(872_730),
        "testnet4" | "signet" => Some(0),
        _ => None,
    }
}

/// Features row is omitted on liquid. Unconfirmed segwit is on. A confirmed
/// height is on at that network's activation. A missing height is off.
fn segwit_enabled(tx: &Transaction, network: &str) -> bool {
    if network == "liquid" || network == "liquidtestnet" {
        return false;
    }
    let Some(status) = tx.status.as_ref() else {
        return false;
    };
    if !status.confirmed {
        return true;
    }
    let Some(height) = status.block_height else {
        return false;
    };
    segwit_activation_height(network).is_some_and(|activation| height >= activation)
}

fn hex_len(text: &str) -> f64 {
    text.len() as f64 / 2.0
}

fn witness_bytes(witness: Option<&Vec<String>>) -> f64 {
    witness
        .map(|items| items.iter().map(|item| hex_len(item)).sum())
        .unwrap_or(0.0)
}

fn script_sig_bytes(script_sig: &str) -> f64 {
    if script_sig.is_empty() {
        0.0
    } else {
        hex_len(script_sig)
    }
}

fn push_number(op: &str) -> Option<usize> {
    if op == "OP_0" {
        return Some(0);
    }
    let digits = op.strip_prefix("OP_PUSHNUM_")?;
    if digits.is_empty() || !digits.bytes().all(|byte| byte.is_ascii_digit()) {
        return None;
    }
    digits.parse().ok()
}

fn multisig_pubkey(key: &str) -> bool {
    let mut chars = key.chars();
    if chars.next() != Some('0') {
        return false;
    }
    let Some(tag) = chars.next() else {
        return false;
    };
    let rest: String = chars.collect();
    let word = rest
        .chars()
        .all(|ch| ch.is_ascii_alphanumeric() || ch == '_');
    match tag {
        '2' | '3' => rest.len() == 64 && word,
        '4' => rest.len() == 128 && word,
        _ => false,
    }
}

/// `parseMultisigScript`. True when the asm is a bare multisig script.
fn multisig_asm(script: &str) -> bool {
    if script.is_empty() {
        return false;
    }
    let mut ops: Vec<&str> = script.split(' ').collect();
    if ops.len() < 3 || ops.pop() != Some("OP_CHECKMULTISIG") {
        return false;
    }
    let Some(n) = ops.pop().and_then(push_number) else {
        return false;
    };
    let Some(need) = n.checked_mul(2).and_then(|pairs| pairs.checked_add(1)) else {
        return false;
    };
    if ops.len() < need {
        return false;
    }
    for _ in 0..n {
        let Some(key) = ops.pop() else {
            return false;
        };
        if !multisig_pubkey(key) {
            return false;
        }
        let Some(push) = ops.pop() else {
            return false;
        };
        if push != "OP_PUSHBYTES_33" && push != "OP_PUSHBYTES_65" {
            return false;
        }
    }
    if ops.pop().and_then(push_number).is_none() {
        return false;
    }
    ops.is_empty()
}

/// `popScriptNumberOperand`. `OP_0`, `OP_PUSHNUM_*`, or a little-endian push.
fn pop_script_number(ops: &mut Vec<&str>) -> Option<u32> {
    let token = ops.pop()?;
    if token == "OP_0" {
        return Some(0);
    }
    if let Some(digits) = token.strip_prefix("OP_PUSHNUM_") {
        if digits.is_empty() || !digits.bytes().all(|byte| byte.is_ascii_digit()) {
            return None;
        }
        return digits.parse().ok();
    }
    if token.is_empty() || token.starts_with("OP_") {
        return None;
    }
    let push = ops.pop()?;
    let byte_count: usize = push.strip_prefix("OP_PUSHBYTES_")?.parse().ok()?;
    if byte_count == 0 || byte_count > 4 || token.len() != byte_count * 2 {
        return None;
    }
    if !token.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return None;
    }
    let mut value = 0u32;
    for index in 0..byte_count {
        let byte = u32::from_str_radix(&token[index * 2..index * 2 + 2], 16).ok()?;
        value |= byte << (8 * index);
    }
    Some(value)
}

/// `parseMultisigScript`. Counts come from the asm. The asm itself is not painted.
fn parse_multisig_counts(script: &str) -> Option<(u32, u32)> {
    if script.is_empty() {
        return None;
    }
    let mut ops: Vec<&str> = script.split(' ').collect();
    if ops.len() < 3 || ops.pop() != Some("OP_CHECKMULTISIG") {
        return None;
    }
    let n = pop_script_number(&mut ops)?;
    let need = n.checked_mul(2)?.checked_add(1)?;
    if ops.len() < need as usize {
        return None;
    }
    for _ in 0..n {
        let key = ops.pop()?;
        if !multisig_pubkey(key) {
            return None;
        }
        let push = ops.pop()?;
        if push != "OP_PUSHBYTES_33" && push != "OP_PUSHBYTES_65" {
            return None;
        }
    }
    let m = pop_script_number(&mut ops)?;
    if !ops.is_empty() {
        return None;
    }
    Some((m, n))
}

/// Realized and potential segwit weight units from `calcSegwitFeeGains`.
/// The percent is `realized / (weight + realized)` and `potential / weight`.
fn segwit_weight_gains(tx: &Transaction) -> (f64, f64) {
    let mut realized = 0.0;
    let mut potential = 0.0;
    for input in &tx.vin {
        let Some(prevout) = input.prevout.as_ref() else {
            continue;
        };
        let kind = prevout.script_pubkey_type.as_str();
        let is_p2wpkh = kind == "v0_p2wpkh";
        let is_p2wsh = kind == "v0_p2wsh";
        let is_p2tr = kind == "v1_p2tr";
        let is_p2sh = kind == "p2sh";
        let op = if input.script_sig.is_empty() {
            None
        } else {
            input.script_sig_asm.split(' ').next()
        };
        let witness_present = input.witness.is_some();
        let wrapped_pkh = is_p2sh && witness_present && op == Some("OP_PUSHBYTES_22");
        let wrapped_sh = is_p2sh && witness_present && op == Some("OP_PUSHBYTES_34");
        let witness = witness_bytes(input.witness.as_ref());
        if is_p2wpkh || is_p2wsh || is_p2tr {
            let extra = if is_p2tr { 42.0 } else { 0.0 };
            realized += (witness + extra) * 3.0;
        } else if wrapped_pkh {
            realized += witness * 3.0 - 84.0;
            potential += 84.0;
        } else if wrapped_sh {
            realized += witness * 3.0 - 140.0;
            potential += 140.0;
        } else if kind == "p2pkh"
            || is_p2sh
            || kind == "p2pk"
            || multisig_asm(&prevout.script_pubkey_asm)
        {
            let mut full = script_sig_bytes(&input.script_sig) * 3.0;
            if multisig_asm(&prevout.script_pubkey_asm) {
                full -= hex_len(&prevout.script_pubkey);
            }
            potential += full;
        }
    }
    (realized, potential)
}

fn js_truthy_ratio(numerator: f64, denominator: f64) -> bool {
    let ratio = numerator / denominator;
    ratio != 0.0 && !ratio.is_nan()
}

fn segwit_badge_for(tx: &Transaction, network: &str) -> String {
    if !segwit_enabled(tx, network) {
        return String::new();
    }
    let (realized, potential) = segwit_weight_gains(tx);
    let weight = f64::from(tx.weight);
    let realized_on = js_truthy_ratio(realized, weight + realized);
    let potential_on = js_truthy_ratio(potential, weight);
    if realized_on && !potential_on {
        SEGWIT_BADGE.to_string()
    } else {
        String::new()
    }
}

/// Badge for one `GET /tx/:txid` body on `network`. Empty when segwit is off
/// or the realized gain is zero.
pub fn segwit_badge_markup(body: &str, network: &str) -> String {
    one_tx(body)
        .map(|tx| segwit_badge_for(&tx, network))
        .unwrap_or_default()
}

/// `tx-features.component.html` lines 11-25. No tooltip. No RBF badge.
const TAPROOT_BADGE: &str = "<span class=\"badge bg-success me-1\">Taproot</span>";

/// Mainnet, testnet, testnet4, and signet heights from `bitcoin.utils.ts`.
/// Liquid is not in that table. A missing network is not mainnet.
fn taproot_activation_height(network: &str) -> Option<u64> {
    match network {
        "mainnet" => Some(709_632),
        "testnet" => Some(2_032_291),
        "testnet4" | "signet" => Some(0),
        _ => None,
    }
}

/// Features row is omitted on liquid. Unconfirmed taproot is on. A confirmed
/// height is on at that network's activation. A missing height is off.
fn taproot_enabled(tx: &Transaction, network: &str) -> bool {
    if network == "liquid" || network == "liquidtestnet" {
        return false;
    }
    let Some(status) = tx.status.as_ref() else {
        return false;
    };
    if !status.confirmed {
        return true;
    }
    let Some(height) = status.block_height else {
        return false;
    };
    taproot_activation_height(network).is_some_and(|activation| height >= activation)
}

fn taproot_badge_for(tx: &Transaction, network: &str) -> String {
    if taproot_enabled(tx, network) {
        TAPROOT_BADGE.to_string()
    } else {
        String::new()
    }
}

/// Badge for one `GET /tx/:txid` body on `network`. Empty when taproot is off.
pub fn taproot_badge_markup(body: &str, network: &str) -> String {
    one_tx(body)
        .map(|tx| taproot_badge_for(&tx, network))
        .unwrap_or_default()
}

/// Mainnet, testnet, testnet4, and signet heights from `bitcoin.utils.ts`.
/// Liquid is not in that table.
fn rbf_activation_height(network: &str) -> Option<u64> {
    match network {
        "mainnet" => Some(399_701),
        "testnet" => Some(720_255),
        "testnet4" | "signet" => Some(0),
        _ => None,
    }
}

/// Features row from `transaction-details.component.html`.
///
/// Liquid omits the row. Unconfirmed `status.confirmed` shows the label.
/// A confirmed height shows it when rbf, segwit, or taproot is active.
/// A missing status does not. The block list calls `tx_row` and does not
/// use this label.
fn features_label(tx: &Transaction, network: &str) -> String {
    if network == "liquid" || network == "liquidtestnet" {
        return String::new();
    }
    let Some(status) = tx.status.as_ref() else {
        return String::new();
    };
    let enabled = if !status.confirmed {
        true
    } else {
        let Some(height) = status.block_height else {
            return String::new();
        };
        rbf_activation_height(network).is_some_and(|activation| height >= activation)
            || segwit_activation_height(network).is_some_and(|activation| height >= activation)
            || taproot_activation_height(network).is_some_and(|activation| height >= activation)
    };
    if enabled {
        // The label cell stays. GPUI whole line. This line is the label text alone.
        format!("{}\nFeatures\n", td("features-label", "Features"))
    } else {
        String::new()
    }
}

/// Label for one `GET /tx/:txid` body on `network`. Empty when the row is off.
pub fn features_label_markup(body: &str, network: &str) -> String {
    one_tx(body)
        .map(|tx| features_label(&tx, network))
        .unwrap_or_default()
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
            // Angular subtitle, once, before the list. Empty vin and vout still show it.
            let mut markup = String::from("<h2>Inputs & Outputs</h2>");
            // `toggleDetailsFromTxPage`. Not the diagram button.
            markup.push_str(
                "<button type=\"button\" class=\"btn btn-outline-info btn-sm\">Details</button>",
            );
            markup.push_str(&table("tx-rows", vec![tx_row(&tx)]));
            if let Some(words) = output_total_words(&tx) {
                markup.push_str(&paragraph("output-total", &words));
            }
            // The page call has no network. Confirmed heights use mainnet.
            // TransactionPage replaces this with the network it is showing.
            markup.push_str(&features_label(&tx, "mainnet"));
            markup.push_str(&segwit_badge_for(&tx, "mainnet"));
            markup.push_str(&taproot_badge_for(&tx, "mainnet"));
            markup
        }
        None => String::new(),
    };
    out.push_str(&liquid_tx_labels(body));
    out
}

/// Tip height minus `status.block_height`, plus 1.
/// An empty or non-integer tip omits the count and does not paint Unconfirmed.
/// When that tip is absent and `status.block_height` is set, the label is Confirmed.
/// A parsed tip with no block height paints Unconfirmed, not a zero count.
/// The difference is `i128` so it does not wrap.
///
/// `tracker.component.html` paints `Block height` only when the transaction
/// is confirmed and `status.block_height` is present. A missing height omits
/// the caption. Outspend status does not use this function.
pub fn confirmation_markup(tx_body: &str, tip_height_body: &str) -> String {
    let status = one_tx(tx_body).and_then(|tx| tx.status);
    let confirmed = status.as_ref().is_some_and(|status| status.confirmed);
    let block_height = status.as_ref().and_then(|status| status.block_height);
    let tip_height = match parse_blocks_tip_height(tip_height_body) {
        Ok(tip_text) => tip_text.parse::<u64>().ok(),
        Err(_) => None,
    };
    let mut markup = match (tip_height, block_height) {
        (Some(tip_height), Some(block_height)) => {
            let count = tip_height as i128 - block_height as i128 + 1;
            paragraph("confirmations", &format!("Confirmations {count}"))
        }
        (Some(_), None) => paragraph("unconfirmed", "Unconfirmed"),
        (None, Some(_)) => paragraph("confirmed", "Confirmed"),
        (None, None) => String::new(),
    };
    if confirmed && block_height.is_some() {
        markup.push_str("<div class=\"label\">Block height</div>");
    }
    markup
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
    let raw = raw_octet_markup(body, "block-raw", splora_frontend_shared::parse_block_raw);
    if let Some(hex) = pre_inner(&raw, "block-raw") {
        out.push_str("block raw ");
        out.push_str(hex);
    }
    out.push_str(&raw);
    out
}

fn pre_inner<'a>(markup: &'a str, class: &str) -> Option<&'a str> {
    let open = format!("<pre class=\"{class}\">");
    let start = markup.find(&open)? + open.len();
    let end = markup[start..].find("</pre>")? + start;
    Some(&markup[start..end])
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
    let anchors = if let Ok(rows) = splora_frontend_shared::parse_test_txs(body) {
        rows.iter()
            .map(|row| txid_anchor(&row.txid))
            .collect::<String>()
    } else {
        // A disallowed row can omit reject-reason. The public parser stops the
        // whole array there. The txid is still on the object, and it still opens /tx/.
        let Ok(rows) = serde_json::from_str::<Vec<serde_json::Value>>(body) else {
            return String::new();
        };
        rows.iter()
            .filter_map(|row| row.get("txid").and_then(|txid| txid.as_str()))
            .filter(|txid| !txid.is_empty())
            .map(txid_anchor)
            .collect::<String>()
    };
    // One column header for the table, not one header per txid.
    if anchors.is_empty() {
        String::new()
    } else {
        format!("<th class=\"allowed\">Allowed?</th>{anchors}")
    }
}

/// Column header from `POST /txs/test` when a row already has `effective-feerate`.
///
/// The header is not part of [`test_txid_markup`]. That function stays Allowed?
/// plus the txid anchors.
pub fn effective_fee_rate_markup(body: &str) -> String {
    let Ok(rows) = parse_test_tx_results_json(body) else {
        return String::new();
    };
    let served = rows
        .iter()
        .any(|row| !row.txid.is_empty() && row.fees.is_some());
    if !served {
        return String::new();
    }
    "<th class=\"rate\">Effective fee rate</th>".to_string()
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
        if let Some(line) = gpui_tip_line(hash_body, height_body) {
            out.push_str(&line);
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
    if let Some(line) = gpui_tip_line(hash_body, height_body) {
        out.push_str(&line);
    }
    out
}

/// GPUI line `tip height: {height} hash: {hash}` from `GET /blocks/tip/height`
/// and `GET /blocks/tip/hash`. The block link and the height link stay.
/// An empty hash or a height that is not an integer omits the line.
fn gpui_tip_line(hash_body: &str, height_body: &str) -> Option<String> {
    let hash = hash_body.trim();
    let height_text = height_body.trim();
    if hash.is_empty()
        || hash.contains('\n')
        || hash.starts_with("indexer error ")
        || hash.starts_with('{')
    {
        return None;
    }
    let Ok(height) = height_text.parse::<u64>() else {
        return None;
    };
    Some(format!(
        "<span class=\"gpui-tip-line\">tip height: {height} hash: {}</span>",
        html_escape(hash)
    ))
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
    grouped_u128(u128::from(value))
}

fn grouped_u128(value: u128) -> String {
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
        // Angular `asset|Liquid Asset page title`. Once, before the id and the Name row.
        out.push_str("<h1>Asset</h1>");
        let loose_at = out.len();
        if let Some(name) = value.get("name").and_then(|item| item.as_str()) {
            out.push_str(&html_escape(name));
        }
        if let Some(ticker) = value.get("ticker").and_then(|item| item.as_str()) {
            if out.len() > loose_at {
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
            // GPUI line `asset id {id}`. The tx-link stays the id alone.
            out.push_str(&asset_id_line(id));
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

/// GPUI line `asset id {id}` from the document `asset_id`. The tx-link stays
/// the id alone. An empty id is not passed here.
fn asset_id_line(asset_id: &str) -> String {
    format!(
        "<span class=\"gpui-asset-id-line\">asset id {}</span>",
        html_escape(asset_id)
    )
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
    let (
        rows,
        precision_line,
        issuer_line,
        issuance_tx_line,
        issued_amount_line,
        burned_amount_line,
        circulating_amount_line,
        pegged_in_line,
        pegged_out_line,
    ) = match asset {
        RegistryAsset::Issued(issued) => {
            // GPUI line `Precision {n}`. The Precision cell stays the number.
            let line = issued.precision.map(|precision| {
                format!("<span class=\"gpui-precision-line\">Precision {precision}</span>")
            });
            (
                issued_stat_rows(&issued),
                line,
                issuer_domain_line(&issued),
                issuance_tx_line(&issued),
                numeric_issued_amount_line(&issued)
                    .or_else(|| confidential_issued_amount_line(&issued)),
                numeric_burned_amount_line(&issued)
                    .or_else(|| confidential_burned_amount_line(&issued)),
                numeric_circulating_amount_line(&issued)
                    .or_else(|| confidential_circulating_amount_line(&issued)),
                None,
                None,
            )
        }
        RegistryAsset::Native(native) => (
            pegged_stat_rows(&native),
            None,
            None,
            None,
            None,
            numeric_pegged_burned_amount_line(&native),
            numeric_pegged_circulating_amount_line(&native),
            numeric_pegged_in_line(&native),
            numeric_pegged_out_line(&native),
        ),
    };
    let mut markup = table("table table-borderless table-striped", rows);
    if let Some(line) = precision_line {
        markup.push_str(&line);
    }
    if let Some(line) = issuer_line {
        markup.push_str(&line);
    }
    if let Some(line) = issuance_tx_line {
        markup.push_str(&line);
    }
    if let Some(line) = issued_amount_line {
        markup.push_str(&line);
    }
    if let Some(line) = burned_amount_line {
        markup.push_str(&line);
    }
    if let Some(line) = circulating_amount_line {
        markup.push_str(&line);
    }
    if let Some(line) = pegged_in_line {
        markup.push_str(&line);
    }
    if let Some(line) = pegged_out_line {
        markup.push_str(&line);
    }
    markup
}

/// GPUI line `Issuer {domain}` when the issued entity domain is shown.
/// The Issuer cell stays the domain. An empty domain omits the line.
fn issuer_domain_line(issued: &IssuedRegistryAsset) -> Option<String> {
    let domain = issued
        .entity
        .get("domain")
        .and_then(|value| value.as_str())
        .filter(|domain| !domain.is_empty())?;
    Some(format!(
        "<span class=\"gpui-issuer-line\">Issuer {}</span>",
        html_escape(domain)
    ))
}

/// GPUI line `Issuance TX {txid}` when the issued issuance transaction is shown.
/// The Issuance TX cell stays the transaction link. An empty transaction omits the line.
fn issuance_tx_line(issued: &IssuedRegistryAsset) -> Option<String> {
    let issuance = issued.issuance_txin.txid.as_str();
    if issuance.is_empty() {
        return None;
    }
    Some(format!(
        "<span class=\"gpui-issuance-tx-line\">Issuance TX {}</span>",
        html_escape(issuance)
    ))
}

/// GPUI line `Issued amount Confidential` when a blinded issuance still
/// shows the Issued amount cell. A clear issuance and a zero issued amount
/// omit the line. The cell stays the word Confidential.
fn confidential_issued_amount_line(issued: &IssuedRegistryAsset) -> Option<String> {
    let blinded =
        issued.chain_stats.has_blinded_issuances || issued.mempool_stats.has_blinded_issuances;
    if !blinded || issued.chain_stats.issued_amount == 0 {
        return None;
    }
    Some("<span class=\"gpui-issued-amount-line\">Issued amount Confidential</span>".to_string())
}

/// GPUI line `Issued amount {n}` when the chain integer is shown.
/// The Issued amount cell stays the number. Blinded and zero omit the line.
fn numeric_issued_amount_line(issued: &IssuedRegistryAsset) -> Option<String> {
    let blinded =
        issued.chain_stats.has_blinded_issuances || issued.mempool_stats.has_blinded_issuances;
    let amount = issued.chain_stats.issued_amount;
    if blinded || amount == 0 {
        return None;
    }
    Some(format!(
        "<span class=\"gpui-issued-amount-line\">Issued amount {amount}</span>"
    ))
}

/// GPUI line `Burned amount {n}` when the chain integer is shown.
/// The Burned amount cell stays the number. A blinded issuance omits the line.
fn numeric_burned_amount_line(issued: &IssuedRegistryAsset) -> Option<String> {
    let blinded =
        issued.chain_stats.has_blinded_issuances || issued.mempool_stats.has_blinded_issuances;
    if blinded {
        return None;
    }
    let amount = issued.chain_stats.burned_amount;
    Some(format!(
        "<span class=\"gpui-burned-amount-line\">Burned amount {amount}</span>"
    ))
}

/// GPUI line `Burned amount Confidential` when a blinded issuance still
/// shows the Burned amount cell. A clear issuance omits the line. The cell
/// stays the word Confidential. A zero burned amount still shows the line.
fn confidential_burned_amount_line(issued: &IssuedRegistryAsset) -> Option<String> {
    let blinded =
        issued.chain_stats.has_blinded_issuances || issued.mempool_stats.has_blinded_issuances;
    if !blinded {
        return None;
    }
    Some("<span class=\"gpui-burned-amount-line\">Burned amount Confidential</span>".to_string())
}

/// GPUI line `Circulating amount {n}` when the chain issued amount is shown.
/// The Circulating amount cell stays the number. A blinded issuance omits the line.
fn numeric_circulating_amount_line(issued: &IssuedRegistryAsset) -> Option<String> {
    let blinded =
        issued.chain_stats.has_blinded_issuances || issued.mempool_stats.has_blinded_issuances;
    let chain = &issued.chain_stats;
    if blinded || chain.issued_amount == 0 {
        return None;
    }
    let amount = chain.issued_amount.saturating_sub(chain.burned_amount);
    Some(format!(
        "<span class=\"gpui-circulating-amount-line\">Circulating amount {amount}</span>"
    ))
}

/// GPUI line `Circulating amount Confidential` when a blinded issuance still
/// shows the Circulating amount cell. A clear issuance omits the line. A zero
/// issued amount omits the line, because that cell stays off. The cell stays
/// the word Confidential.
fn confidential_circulating_amount_line(issued: &IssuedRegistryAsset) -> Option<String> {
    let blinded =
        issued.chain_stats.has_blinded_issuances || issued.mempool_stats.has_blinded_issuances;
    if !blinded || issued.chain_stats.issued_amount == 0 {
        return None;
    }
    Some(
        "<span class=\"gpui-circulating-amount-line\">Circulating amount Confidential</span>"
            .to_string(),
    )
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
    if let Some(name) = issued
        .name
        .as_deref()
        .filter(|name| !name.trim().is_empty())
    {
        let mut value = html_escape(name);
        if let Some(ticker) = issued
            .ticker
            .as_deref()
            .filter(|ticker| !ticker.trim().is_empty())
        {
            value.push_str(" (");
            value.push_str(&html_escape(ticker));
            value.push(')');
        }
        rows.push(stat_row("Name", &value));
    }
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

/// GPUI line `Pegged in {n}` when the chain pegged-in integer is shown.
/// The Pegged in cell stays the number. A zero pegged-in amount omits the line.
/// A mempool pegged-in amount is not the line.
fn numeric_pegged_in_line(native: &NativeRegistryAsset) -> Option<String> {
    let amount = native.chain_stats.peg_in_amount;
    if amount == 0 {
        return None;
    }
    Some(format!(
        "<span class=\"gpui-pegged-in-line\">Pegged in {amount}</span>"
    ))
}

/// GPUI line `Burned amount {n}` from the pegged chain burned integer.
/// The Burned amount cell stays the number. A zero burned amount still
/// shows the line, because that cell stays. A mempool burned amount is not
/// the line.
fn numeric_pegged_burned_amount_line(native: &NativeRegistryAsset) -> Option<String> {
    let amount = native.chain_stats.burned_amount;
    Some(format!(
        "<span class=\"gpui-burned-amount-line\">Burned amount {amount}</span>"
    ))
}

/// GPUI line `Circulating amount {n}` from the pegged chain peg-in amount
/// minus peg-out minus burned. The Circulating amount cell stays the integer.
/// A zero pegged-in amount omits the line. A mempool amount is not the line.
fn numeric_pegged_circulating_amount_line(native: &NativeRegistryAsset) -> Option<String> {
    let chain = &native.chain_stats;
    if chain.peg_in_amount == 0 {
        return None;
    }
    let amount = chain
        .peg_in_amount
        .saturating_sub(chain.peg_out_amount)
        .saturating_sub(chain.burned_amount);
    Some(format!(
        "<span class=\"gpui-circulating-amount-line\">Circulating amount {amount}</span>"
    ))
}

/// GPUI line `Pegged out {n}` when the chain pegged-out integer is shown.
/// The Pegged out cell stays the number. A zero pegged-out amount omits the line.
/// A mempool pegged-out amount is not the line.
fn numeric_pegged_out_line(native: &NativeRegistryAsset) -> Option<String> {
    let amount = native.chain_stats.peg_out_amount;
    if amount == 0 {
        return None;
    }
    Some(format!(
        "<span class=\"gpui-pegged-out-line\">Pegged out {amount}</span>"
    ))
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

/// Loaded-list length, `of`, and chain plus mempool `tx_count`.
///
/// Length 1 uses `transaction`. A longer list uses `transactions`.
/// An empty array is `&nbsp;`. A body that is not that list is empty.
/// `Transactions {sum}` stays the separate paragraph.
pub(crate) fn address_tx_of_heading(stats_json: &str, txs_json: &str) -> String {
    let Some(stats) = one_stats(stats_json) else {
        return String::new();
    };
    let Some(loaded) = loaded_tx_len(txs_json) else {
        return String::new();
    };
    let sentence = if loaded == 0 {
        "&nbsp;".to_string()
    } else {
        let sum = u128::from(stats.chain_stats.tx_count) + u128::from(stats.mempool_stats.tx_count);
        let noun = if loaded == 1 {
            "transaction"
        } else {
            "transactions"
        };
        format!(
            "{} of {} {noun}",
            grouped_u128(loaded as u128),
            grouped_u128(sum)
        )
    };
    format!("<div class=\"title-tx\"><h2 class=\"text-start\">{sentence}</h2></div>")
}

fn loaded_tx_len(txs_json: &str) -> Option<usize> {
    let trimmed = txs_json.trim();
    if !trimmed.starts_with('[') {
        return None;
    }
    if exact_empty_array(trimmed) {
        Some(0)
    } else {
        parse_transactions_json(trimmed).ok().map(|txs| txs.len())
    }
}

/// Address transactions widget from `address-transactions-widget.component.html`.
/// The thead cell is the word Date. An empty row list stays empty.
fn address_latest_tx_table(rows: Vec<String>) -> String {
    if rows.is_empty() {
        String::new()
    } else {
        format!(
            "<table class=\"tx-rows\"><thead><th class=\"table-cell-date\">Date</th></thead><tbody>{}</tbody></table>",
            rows.concat()
        )
    }
}

pub fn address_rows_markup(stats_json: &str, txs_json: &str, utxo_json: &str) -> String {
    let load_error = address_load_error_markup(stats_json);
    if !load_error.is_empty() {
        return load_error;
    }
    let mut out = one_stats(stats_json)
        .map(|stats| address_summary(&stats, txs_json))
        .unwrap_or_default();
    out.push_str(&address_tx_of_heading(stats_json, txs_json));
    if exact_empty_array(txs_json) {
        out.push_str(&empty_list("transactions"));
        // GPUI empty-list line. The older "There are no transactions." stays.
        out.push_str("<p class=\"address-tx-empty\">The address transaction list is empty.</p>");
    } else {
        out.push_str(&address_latest_tx_table(
            txs_from(txs_json).iter().map(tx_row).collect(),
        ));
    }
    if exact_empty_array(utxo_json) {
        out.push_str(&empty_list("unspent outputs"));
        // GPUI empty-list line. The older "There are no unspent outputs." stays.
        out.push_str("<p class=\"address-utxo-empty\">The utxo list is empty.</p>");
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
    let version_hex = format!("0x{value:08x}", value = block.version);
    // Angular shortenString of length 13. Closed details omit this line.
    let hash_line = if open {
        format!("\nHash {}\n", html_escape(&shorten_block_id(&block.id)))
    } else {
        String::new()
    };
    let mut rows = format!(
        "<tr><td>Version</td><td>{}</td><td class=\"version-hex\">{version_hex}</td></tr>",
        block.version
    );
    // GPUI whole line. Closed details keep the cells and omit this line.
    let version_line = if open {
        format!("\nversion {} {version_hex}\n", block.version)
    } else {
        String::new()
    };
    // A missing key is None. Some(0) still prints. Do not substitute zero.
    let mut bits_line = String::new();
    if let Some(bits) = block.bits {
        let bits_hex = format!("0x{value:08x}", value = bits);
        rows.push_str(&format!(
            "<tr><td>Bits</td><td>{bits}</td><td class=\"bits-hex\">{bits_hex}</td></tr>"
        ));
        // GPUI whole line. Closed details keep the cells and omit this line.
        if open {
            bits_line = format!("\nbits {bits} {bits_hex}\n");
        }
    }
    let merkle = html_escape(&block.merkle_root);
    rows.push_str(&format!("<tr><td>Merkle root</td><td>{merkle}</td></tr>"));
    // GPUI whole line. Closed details keep the cell and omit this line.
    let merkle_line = if open {
        format!("\nMerkle root {merkle}\n")
    } else {
        String::new()
    };
    let mut difficulty_line = String::new();
    if let Some(difficulty) = block.difficulty {
        let difficulty = plain_number(difficulty);
        rows.push_str(&format!(
            "<tr><td>Difficulty</td><td>{difficulty}</td></tr>"
        ));
        // GPUI whole line. Closed details keep the cell and omit this line.
        if open {
            difficulty_line = format!("\nDifficulty {difficulty}\n");
        }
    }
    // Decimal stays (`>9<` in block_transactions). Hex is decimal2hex, 4 bytes.
    // A missing key is None. Some(0) still prints. Do not substitute zero.
    let mut nonce_label_line = String::new();
    let mut nonce_line = String::new();
    if let Some(nonce) = block.nonce {
        let nonce_hex = format!("0x{value:08x}", value = nonce);
        rows.push_str(&format!(
            "<tr><td>Nonce</td><td>{nonce}</td><td class=\"nonce-hex\">{nonce_hex}</td></tr>"
        ));
        // Angular nonce value is decimal2hex only. Closed details keep the cells and omit both lines.
        if open {
            nonce_label_line = format!("\nNonce {nonce_hex}\n");
            nonce_line = format!("\nnonce {nonce} {nonce_hex}\n");
        }
    }
    // GPUI whole line. Closed details omit this line. A missing key stays off.
    let mut previous_line = String::new();
    if open {
        if let Some(previous) = nonempty(block.previous_block_hash.as_deref()) {
            let previous = html_escape(previous);
            previous_line = format!("\npreviousblockhash {previous}\n");
        }
    }
    // GPUI whole line. Closed details omit this line. The raw integer stays.
    let median_line = if open {
        format!("\nmediantime {}\n", block.median_time)
    } else {
        String::new()
    };
    // The next cell is the header link, not the hex digits. The row stays when the body is absent.
    rows.push_str(&format!(
        "<tr><td>Block Header Hex</td><td><a href=\"/block/{id}/header\"></a></td></tr>"
    ));
    format!(
        "<div id=\"details\"{hidden}><table><tbody>{rows}</tbody></table>{hash_line}{version_line}{bits_line}{merkle_line}{difficulty_line}{nonce_label_line}{nonce_line}{previous_line}{median_line}</div>"
    )
}

/// Eighty-byte `GET /block/:hash/header` hex beside the Block Header Hex label.
/// A missing, failed, or short body leaves the link row unchanged.
pub fn block_header_hex_markup(details: &str, header_body: &str) -> String {
    let Some(hex) = eighty_byte_header_hex(header_body) else {
        return details.to_string();
    };
    let needle = "<td>Block Header Hex</td>";
    let Some(at) = details.find(needle) else {
        return details.to_string();
    };
    let after = at + needle.len();
    let Some(end_rel) = details[after..].find("</tr>") else {
        return details.to_string();
    };
    let insert_at = after + end_rel;
    let mut out = String::with_capacity(details.len() + hex.len() + 32);
    out.push_str(&details[..insert_at]);
    out.push_str("<td class=\"block-header-hex\">");
    out.push_str(&hex);
    out.push_str("</td>");
    out.push_str(&details[insert_at..]);
    out
}

/// Notice for a failed header body. An 80-byte header is painted on the details row instead.
/// A shorter hex body also paints `block header` plus that hex, the same line the native screen shows.
pub fn block_header_aside_markup(body: &str) -> String {
    if eighty_byte_header_hex(body).is_some() {
        return String::new();
    }
    let raw = indexed_bytes_markup(body, "block-header");
    if let Some(hex) = pre_inner(&raw, "block-header") {
        let mut out = String::with_capacity(raw.len() + hex.len() + 13);
        out.push_str("block header ");
        out.push_str(hex);
        out.push_str(&raw);
        out
    } else {
        raw
    }
}

fn eighty_byte_header_hex(body: &str) -> Option<String> {
    let text = splora_frontend_shared::parse_block_header(body).ok()?;
    let text = text.to_ascii_lowercase();
    if text.len() == 160
        && text
            .bytes()
            .all(|byte| matches!(byte, b'0'..=b'9' | b'a'..=b'f'))
    {
        Some(text)
    } else {
        None
    }
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
