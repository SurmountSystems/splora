//! Electrs paths each v1 page requests, and the field lines that page shows.

use splora_frontend_shared::wire::{
    AddressStats, Block, RecentTransaction, TestTxResult, Transaction, Utxo,
};
use splora_frontend_shared::{
    address_path, address_txs_path, address_utxo_path, block_height_path, block_path,
    block_txids_path, block_txs_path, block_txs_start_index_path, blocks_path,
    blocks_start_height_path, broadcast_body, mempool_recent_path, parse_block_txids,
    test_txs_body, tx_path,
};

use crate::FAIL_CLOSED;
use crate::models::{
    parse_address_stats_json, parse_block_json, parse_blocks_json, parse_broadcast_result_json,
    parse_recent_transactions_json, parse_test_tx_results_json, parse_transaction_json,
    parse_transactions_json, parse_utxos_json,
};
use crate::paths::IndexerRequest;

fn get(path: String) -> IndexerRequest {
    IndexerRequest {
        method: "GET",
        path,
    }
}

fn push_unreadable(lines: &mut Vec<String>) {
    let notice = "unreadable indexer response";
    if !lines.iter().any(|line| line == notice) {
        lines.push(notice.to_string());
    }
}

/// Signer refusal stays the fail-closed sentence. A failed indexer call names
/// the request instead of that sentence.
pub(crate) fn closed_notice(body: &str) -> Option<&str> {
    let trimmed = body.trim();
    if trimmed == FAIL_CLOSED {
        return Some(FAIL_CLOSED);
    }
    if let Some(rest) = trimmed.strip_prefix("indexer error ") {
        if !rest.trim().is_empty() {
            return Some(trimmed);
        }
    }
    None
}

fn closed_or_empty(body: &str, lines: &mut Vec<String>) -> bool {
    if body.trim().is_empty() {
        return true;
    }
    if let Some(line) = closed_notice(body) {
        if !lines.iter().any(|existing| existing == line) {
            lines.push(line.to_string());
        }
        return true;
    }
    false
}

fn yes_no(value: bool) -> &'static str {
    if value { "true" } else { "false" }
}

fn block_line(block: &Block) -> String {
    let mut line = format!(
        "block {id} height {height} tx_count {tx_count} timestamp {timestamp} size {size} weight {weight}",
        id = block.id,
        height = block.height,
        tx_count = block.tx_count,
        timestamp = block.timestamp,
        size = block.size,
        weight = block.weight,
    );
    if let Some(previous) = block.previous_block_hash.as_deref() {
        if !previous.is_empty() {
            line.push_str(" previous ");
            line.push_str(previous);
        }
    }
    line.push_str(" hash ");
    line.push_str(&block.id);
    line.push_str(" median_time ");
    line.push_str(&block.median_time.to_string());
    line
}

fn recent_line(tx: &RecentTransaction) -> String {
    format!(
        "recent {txid} fee {fee} vsize {vsize} value {value} txid {txid}",
        txid = tx.txid,
        fee = tx.fee,
        vsize = tx.vsize,
        value = tx.value,
    )
}

fn tx_line(tx: &Transaction) -> String {
    let confirmed = tx
        .status
        .as_ref()
        .map(|status| status.confirmed)
        .unwrap_or(false);
    let mut line = format!(
        "txid {txid} confirmed {confirmed} fee {fee} size {size} weight {weight}",
        txid = tx.txid,
        confirmed = yes_no(confirmed),
        fee = tx.fee,
        size = tx.size,
        weight = tx.weight,
    );
    if let Some(status) = tx.status.as_ref() {
        if let Some(height) = status.block_height {
            line.push_str(" block_height ");
            line.push_str(&height.to_string());
        }
        if let Some(hash) = status.block_hash.as_deref().filter(|hash| !hash.is_empty()) {
            line.push_str(" block_hash ");
            line.push_str(hash);
        }
        if let Some(time) = status.block_time {
            line.push_str(" block_time ");
            line.push_str(&time.to_string());
        }
    }
    append_tx_addresses(&mut line, tx);
    line
}

fn append_address_value(line: &mut String, address: Option<&str>, value: u64) {
    let Some(address) = address.map(str::trim).filter(|address| !address.is_empty()) else {
        return;
    };
    line.push_str(" address ");
    line.push_str(address);
    line.push_str(" value ");
    line.push_str(&value.to_string());
}

fn append_tx_addresses(line: &mut String, tx: &Transaction) {
    for input in &tx.vin {
        if let Some(prevout) = &input.prevout {
            append_address_value(
                line,
                prevout.script_pubkey_address.as_deref(),
                prevout.value,
            );
        }
    }
    for output in &tx.vout {
        append_address_value(line, output.script_pubkey_address.as_deref(), output.value);
    }
}

fn address_line(stats: &AddressStats) -> String {
    let mut line = format!(
        "address {address} chain_tx_count {chain} funded_txo_sum {funded} mempool_tx_count {mempool} mempool_funded_txo_sum {mempool_funded}",
        address = stats.address.as_deref().unwrap_or(""),
        chain = stats.chain_stats.tx_count,
        funded = stats.chain_stats.funded_txo_sum,
        mempool = stats.mempool_stats.tx_count,
        mempool_funded = stats.mempool_stats.funded_txo_sum,
    );
    if let Some(scripthash) = stats
        .scripthash
        .as_deref()
        .map(str::trim)
        .filter(|scripthash| !scripthash.is_empty())
    {
        line.push_str(" scripthash ");
        line.push_str(scripthash);
    }
    line
}

fn utxo_line(utxo: &Utxo) -> String {
    let mut line = format!(
        "utxo {txid} vout {vout} value {value} confirmed {confirmed}",
        txid = utxo.txid,
        vout = utxo.vout,
        value = utxo.value,
        confirmed = yes_no(utxo.status.confirmed),
    );
    if let Some(height) = utxo.status.block_height {
        line.push_str(" block_height ");
        line.push_str(&height.to_string());
    }
    if let Some(hash) = utxo
        .status
        .block_hash
        .as_deref()
        .filter(|hash| !hash.is_empty())
    {
        line.push_str(" block_hash ");
        line.push_str(hash);
    }
    if let Some(time) = utxo.status.block_time {
        line.push_str(" block_time ");
        line.push_str(&time.to_string());
    }
    line
}

fn block_lines(body: &str, lines: &mut Vec<String>) {
    if closed_or_empty(body, lines) {
        return;
    }
    match parse_blocks_json(body) {
        Ok(blocks) => {
            for block in blocks {
                lines.push(block_line(&block));
            }
        }
        Err(_) => push_unreadable(lines),
    }
}

fn is_numeric_id(id: &str) -> bool {
    !id.is_empty() && id.bytes().all(|byte| byte.is_ascii_digit())
}

fn is_block_hash(body: &str) -> bool {
    let hash = body.trim();
    hash.len() == 64 && hash.bytes().all(|byte| byte.is_ascii_hexdigit())
}

fn block_and_txs(hash: &str, with_txids: bool, page_start: Option<u32>) -> Vec<IndexerRequest> {
    let mut calls = vec![get(block_path(hash)), get(block_txs_path(hash))];
    if with_txids {
        calls.push(get(block_txids_path(hash)));
    }
    if let Some(start) = page_start {
        if start % 25 == 0 {
            calls.push(get(block_txs_start_index_path(hash, u64::from(start))));
        }
    }
    calls
}

pub fn dashboard_screen_requests() -> Vec<IndexerRequest> {
    vec![
        get(blocks_path().to_string()),
        get(mempool_recent_path().to_string()),
    ]
}

pub fn dashboard_fields_text(blocks_json: &str, recent_json: &str) -> String {
    let mut lines = Vec::new();
    block_lines(blocks_json, &mut lines);
    if !closed_or_empty(recent_json, &mut lines) {
        match parse_recent_transactions_json(recent_json) {
            Ok(recent) => {
                for tx in recent {
                    lines.push(recent_line(&tx));
                }
            }
            Err(_) => push_unreadable(&mut lines),
        }
    }
    lines.join("\n")
}

pub fn blocks_list_requests() -> Vec<IndexerRequest> {
    blocks_list_requests_at(None)
}

pub fn blocks_list_requests_at(start_height: Option<u64>) -> Vec<IndexerRequest> {
    let path = match start_height {
        Some(height) => blocks_start_height_path(height),
        None => blocks_path().to_string(),
    };
    vec![get(path)]
}

pub fn blocks_list_fields_text(blocks_json: &str) -> String {
    let mut lines = Vec::new();
    block_lines(blocks_json, &mut lines);
    lines.join("\n")
}

/// A numeric id is a height. That route returns a block hash, and only then
/// does the page request the block and its transactions.
pub fn block_load_plan(id: &str, height_body: Option<&str>) -> Vec<IndexerRequest> {
    assemble_block_plan(id, height_body, false, None)
}

/// Block, unpaged transactions, transaction ids, and an optional page.
///
/// `page_start` must be a multiple of 25. Other values omit the page path.
pub fn block_load_plan_with_txids(
    id: &str,
    height_body: Option<&str>,
    page_start: Option<u32>,
) -> Vec<IndexerRequest> {
    assemble_block_plan(id, height_body, true, page_start)
}

fn assemble_block_plan(
    id: &str,
    height_body: Option<&str>,
    with_txids: bool,
    page_start: Option<u32>,
) -> Vec<IndexerRequest> {
    let id = id.trim();
    if id.is_empty() {
        return Vec::new();
    }
    if is_numeric_id(id) {
        let Some(body) = height_body else {
            let Ok(height) = id.parse::<u64>() else {
                return Vec::new();
            };
            return vec![get(block_height_path(height))];
        };
        if is_block_hash(body) {
            return block_and_txs(body.trim(), with_txids, page_start);
        }
        return Vec::new();
    }
    block_and_txs(id, with_txids, page_start)
}

fn append_txs(lines: &mut Vec<String>, txs: Vec<Transaction>) {
    for tx in txs {
        lines.push(tx_line(&tx));
    }
}

pub fn block_fields_text(block_json: &str, txs_json: &str) -> String {
    let mut lines = Vec::new();
    if !closed_or_empty(block_json, &mut lines) {
        match parse_block_json(block_json) {
            Ok(block) => lines.push(block_line(&block)),
            Err(_) => push_unreadable(&mut lines),
        }
    }
    if !closed_or_empty(txs_json, &mut lines) {
        match parse_transactions_json(txs_json) {
            Ok(txs) => append_txs(&mut lines, txs),
            Err(_) => push_unreadable(&mut lines),
        }
    }
    lines.join("\n")
}

pub fn block_txids_fields_text(body: &str) -> String {
    let mut lines = Vec::new();
    if closed_or_empty(body, &mut lines) {
        return lines.join("\n");
    }
    match parse_block_txids(body) {
        Ok(txids) => {
            for txid in txids {
                lines.push(format!("txid {txid}"));
            }
        }
        Err(_) => push_unreadable(&mut lines),
    }
    lines.join("\n")
}

pub fn transaction_requests(txid: &str) -> Vec<IndexerRequest> {
    let txid = txid.trim();
    if txid.is_empty() {
        return Vec::new();
    }
    vec![get(tx_path(txid))]
}

pub fn transaction_fields_text(body: &str) -> String {
    let mut lines = Vec::new();
    if !closed_or_empty(body, &mut lines) {
        match parse_transaction_json(body) {
            Ok(tx) => append_txs(&mut lines, vec![tx]),
            Err(_) => push_unreadable(&mut lines),
        }
    }
    lines.join("\n")
}

pub fn address_requests(script: &str) -> Vec<IndexerRequest> {
    let script = script.trim();
    if script.is_empty() {
        return Vec::new();
    }
    vec![
        get(address_path(script)),
        get(address_txs_path(script)),
        get(address_utxo_path(script)),
    ]
}

pub fn address_fields_text(stats_json: &str, txs_json: &str, utxo_json: &str) -> String {
    let mut lines = Vec::new();
    if !closed_or_empty(stats_json, &mut lines) {
        match parse_address_stats_json(stats_json) {
            Ok(stats) => lines.push(address_line(&stats)),
            Err(_) => push_unreadable(&mut lines),
        }
    }
    if !closed_or_empty(txs_json, &mut lines) {
        match parse_transactions_json(txs_json) {
            Ok(txs) => append_txs(&mut lines, txs),
            Err(_) => push_unreadable(&mut lines),
        }
    }
    if !closed_or_empty(utxo_json, &mut lines) {
        match parse_utxos_json(utxo_json) {
            Ok(utxos) => {
                for utxo in utxos {
                    lines.push(utxo_line(&utxo));
                }
            }
            Err(_) => push_unreadable(&mut lines),
        }
    }
    lines.join("\n")
}

pub fn entered_addresses(raw: &str) -> Vec<String> {
    raw.split(|ch: char| ch == ',' || ch.is_whitespace())
        .map(str::trim)
        .filter(|part| !part.is_empty())
        .map(str::to_string)
        .collect()
}

pub fn multi_address_screen_requests(values: &[String]) -> Vec<IndexerRequest> {
    values
        .iter()
        .map(|value| value.trim())
        .filter(|script| !script.is_empty())
        .flat_map(|script| [get(address_path(script)), get(address_utxo_path(script))])
        .collect()
}

pub fn multi_address_fields_text(rows: &[(String, String)]) -> String {
    let mut lines = Vec::new();
    for (path, body) in rows {
        if closed_or_empty(body, &mut lines) {
            continue;
        }
        if path.ends_with("/utxo") {
            match parse_utxos_json(body) {
                Ok(utxos) => {
                    for utxo in utxos {
                        lines.push(utxo_line(&utxo));
                    }
                }
                Err(_) => push_unreadable(&mut lines),
            }
            continue;
        }
        match parse_address_stats_json(body) {
            Ok(stats) => lines.push(address_line(&stats)),
            Err(_) => push_unreadable(&mut lines),
        }
    }
    lines.join("\n")
}

pub fn broadcast_screen_request() -> IndexerRequest {
    crate::paths::broadcast_request()
}

pub fn broadcast_payload(raw: &str) -> String {
    broadcast_body(raw)
}

pub fn broadcast_fields_text(body: &str) -> String {
    if let Some(line) = closed_notice(body) {
        return line.to_string();
    }
    match parse_broadcast_result_json(body) {
        Ok(result) => format!("txid {}", result.txid),
        Err(_) => String::new(),
    }
}

pub fn test_transactions_screen_request() -> IndexerRequest {
    crate::paths::test_transactions_request()
}

pub fn test_transactions_payload(raw: &str) -> String {
    let trimmed = raw.trim();
    if let Ok(rows) = serde_json::from_str::<Vec<String>>(trimmed) {
        let refs: Vec<&str> = rows.iter().map(String::as_str).collect();
        return test_txs_body(&refs);
    }
    let rows: Vec<String> = trimmed
        .split(|ch: char| ch.is_whitespace() || ch == ',')
        .map(str::trim)
        .filter(|part| !part.is_empty())
        .map(str::to_string)
        .collect();
    let refs: Vec<&str> = rows.iter().map(String::as_str).collect();
    test_txs_body(&refs)
}

fn plain_number(value: f64) -> String {
    if value.is_finite() && value.fract() == 0.0 && value.abs() < (i64::MAX as f64) {
        format!("{}", value as i64)
    } else {
        format!("{value}")
    }
}

fn test_line(result: &TestTxResult) -> String {
    let allowed = result.allowed.map(yes_no).unwrap_or("");
    let mut line = format!(
        "txid {txid} allowed {allowed}",
        txid = result.txid,
        allowed = allowed,
    );
    if let Some(fees) = &result.fees {
        line.push_str(" fee base ");
        line.push_str(&plain_number(fees.base));
        line.push_str(" effective-feerate ");
        line.push_str(&plain_number(fees.effective_feerate));
        line.push_str(" effective-includes ");
        line.push_str(&fees.effective_includes.join(","));
    }
    if let Some(reason) = result.reject_reason.as_deref() {
        if !reason.is_empty() {
            line.push_str(" reject-reason ");
            line.push_str(reason);
        }
    }
    line
}

pub fn test_transactions_fields_text(body: &str) -> String {
    let mut lines = Vec::new();
    if closed_or_empty(body, &mut lines) {
        return lines.join("\n");
    }
    match parse_test_tx_results_json(body) {
        Ok(rows) => {
            for result in rows {
                lines.push(test_line(&result));
            }
        }
        Err(_) => push_unreadable(&mut lines),
    }
    lines.join("\n")
}
