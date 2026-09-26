//! One HTML row per block, transaction, or utxo.
//!
//! Field sentences stay available for the screen text helpers. These functions
//! are what the views put on the page.

use splora_frontend_shared::wire::{AddressStats, Block, RecentTransaction, Transaction, Utxo};
use splora_frontend_shared::{BlockRow, TxRow};

use crate::models::{
    parse_address_stats_json, parse_block_json, parse_blocks_json, parse_recent_transactions_json,
    parse_transaction_json, parse_transactions_json, parse_utxos_json,
};
use crate::screens::closed_notice;

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

fn paragraph(class: &str, text: &str) -> String {
    format!("<p class=\"{class}\">{text}</p>")
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
    let mut out = String::new();
    for line in fields.lines() {
        if is_notice(line) {
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
        td("height", &row.height.to_string()),
        td("hash", &html_escape(&row.id)),
        td("tx-count", &row.tx_count.to_string()),
        td("time", &row.timestamp.to_string()),
        td("size", &block.size.to_string()),
        td("weight", &block.weight.to_string()),
        td("median-time", &block.median_time.to_string()),
    ];
    if let Some(previous) = nonempty(block.previous_block_hash.as_deref()) {
        cells.push(td("previous", &html_escape(previous)));
    }
    format!("<tr class=\"block-row\">{}</tr>", cells.concat())
}

fn party(address: Option<&str>, value: u64) -> Option<String> {
    let address = nonempty(address)?;
    Some(format!(
        "<span class=\"party\"><span class=\"address\">{}</span><span class=\"value\">{value}</span></span>",
        html_escape(address),
    ))
}

fn tx_parties(tx: &Transaction) -> String {
    let mut parties = Vec::new();
    for input in &tx.vin {
        if let Some(prevout) = &input.prevout {
            if let Some(markup) = party(prevout.script_pubkey_address.as_deref(), prevout.value) {
                parties.push(markup);
            }
        }
    }
    for output in &tx.vout {
        if let Some(markup) = party(output.script_pubkey_address.as_deref(), output.value) {
            parties.push(markup);
        }
    }
    parties.concat()
}

fn tx_row(tx: &Transaction) -> String {
    let row = TxRow::from(tx);
    let confirmed = tx
        .status
        .as_ref()
        .map(|status| status.confirmed)
        .unwrap_or(false);
    let mut cells = vec![
        td("txid", &html_escape(&row.txid)),
        td("fee", &row.fee.to_string()),
        td("confirmed", yes_no(confirmed)),
        td("size", &tx.size.to_string()),
        td("weight", &tx.weight.to_string()),
    ];
    if let Some(height) = row.block_height {
        cells.push(td("block-height", &height.to_string()));
    }
    if let Some(status) = tx.status.as_ref() {
        if let Some(hash) = nonempty(status.block_hash.as_deref()) {
            cells.push(td("block-hash", &html_escape(hash)));
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
    format!(
        "<tr class=\"tx-row\">{}{}{}{}</tr>",
        td("txid", &html_escape(&tx.txid)),
        td("fee", &tx.fee.to_string()),
        td("vsize", &tx.vsize.to_string()),
        td("value", &tx.value.to_string()),
    )
}

fn utxo_row(utxo: &Utxo) -> String {
    let mut cells = vec![
        td("txid", &html_escape(&utxo.txid)),
        td("vout", &utxo.vout.to_string()),
        td("value", &utxo.value.to_string()),
        td("confirmed", yes_no(utxo.status.confirmed)),
    ];
    if let Some(height) = utxo.status.block_height {
        cells.push(td("block-height", &height.to_string()));
    }
    if let Some(hash) = nonempty(utxo.status.block_hash.as_deref()) {
        cells.push(td("block-hash", &html_escape(hash)));
    }
    if let Some(time) = utxo.status.block_time {
        cells.push(td("block-time", &time.to_string()));
    }
    format!("<tr class=\"utxo-row\">{}</tr>", cells.concat())
}

fn address_summary(stats: &AddressStats) -> String {
    let mut out = String::new();
    if let Some(address) = nonempty(stats.address.as_deref()) {
        out.push_str(&paragraph("address", &html_escape(address)));
    }
    if let Some(scripthash) = nonempty(stats.scripthash.as_deref()) {
        out.push_str(&paragraph("scripthash", &html_escape(scripthash)));
    }
    out.push_str(&paragraph(
        "chain-tx-count",
        &stats.chain_stats.tx_count.to_string(),
    ));
    out.push_str(&paragraph(
        "funded-sum",
        &stats.chain_stats.funded_txo_sum.to_string(),
    ));
    out.push_str(&paragraph(
        "mempool-tx-count",
        &stats.mempool_stats.tx_count.to_string(),
    ));
    out.push_str(&paragraph(
        "mempool-funded-sum",
        &stats.mempool_stats.funded_txo_sum.to_string(),
    ));
    out
}

pub fn dashboard_rows_markup(blocks_json: &str, recent_json: &str) -> String {
    let mut out = table(
        "block-rows",
        blocks_from(blocks_json).iter().map(block_row).collect(),
    );
    out.push_str(&table(
        "tx-rows",
        recent_from(recent_json).iter().map(recent_row).collect(),
    ));
    out
}

pub fn blocks_rows_markup(blocks_json: &str) -> String {
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
    out.push_str(&table(
        "tx-rows",
        txs_from(txs_json).iter().map(tx_row).collect(),
    ));
    out
}

pub fn transaction_row_markup(body: &str) -> String {
    match one_tx(body) {
        Some(tx) => table("tx-rows", vec![tx_row(&tx)]),
        None => String::new(),
    }
}

pub fn address_rows_markup(stats_json: &str, txs_json: &str, utxo_json: &str) -> String {
    let mut out = one_stats(stats_json)
        .map(|stats| address_summary(&stats))
        .unwrap_or_default();
    out.push_str(&table(
        "tx-rows",
        txs_from(txs_json).iter().map(tx_row).collect(),
    ));
    out.push_str(&table(
        "utxo-rows",
        utxos_from(utxo_json).iter().map(utxo_row).collect(),
    ));
    out
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
