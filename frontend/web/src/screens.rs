//! Electrs paths each v1 page requests, and the field lines that page shows.

use splora_frontend_shared::{
    AddressStats, Block, RecentTx, TestTxResult, Tx, Utxo, address_path, address_txs_path,
    address_utxo_path, block_height_path, block_path, block_txids_path, block_txs_path,
    block_txs_start_index_path, blocks_path, blocks_start_height_path, broadcast_body,
    mempool_recent_path, parse_address, parse_address_txs, parse_address_utxos, parse_block,
    parse_block_txids, parse_block_txs, parse_blocks, parse_broadcast, parse_mempool_recent,
    parse_test_txs, parse_tx, test_txs_body, tx_path,
};

use crate::FAIL_CLOSED;
use crate::paths::IndexerRequest;

fn get(path: String) -> IndexerRequest {
    IndexerRequest {
        method: "GET",
        path,
    }
}

fn push_closed(lines: &mut Vec<String>) {
    if !lines.iter().any(|line| line == FAIL_CLOSED) {
        lines.push(FAIL_CLOSED.to_string());
    }
}

fn push_unreadable(lines: &mut Vec<String>) {
    let notice = "unreadable indexer response";
    if !lines.iter().any(|line| line == notice) {
        lines.push(notice.to_string());
    }
}

fn closed_or_empty(body: &str, lines: &mut Vec<String>) -> bool {
    if body.trim().is_empty() {
        return true;
    }
    if body == FAIL_CLOSED {
        push_closed(lines);
        return true;
    }
    false
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
    if !block.previous_hash.is_empty() {
        line.push_str(" previous ");
        line.push_str(&block.previous_hash);
    }
    line
}

fn recent_line(tx: &RecentTx) -> String {
    format!(
        "recent {txid} fee {fee} vsize {vsize} value {value}",
        txid = tx.txid,
        fee = tx.fee,
        vsize = tx.vsize,
        value = tx.value,
    )
}

fn tx_line(tx: &Tx) -> String {
    let mut line = format!(
        "txid {txid} confirmed {confirmed} fee {fee} size {size} weight {weight}",
        txid = tx.txid,
        confirmed = tx.confirmed,
        fee = tx.fee,
        size = tx.size,
        weight = tx.weight,
    );
    if !tx.block_height.is_empty() {
        line.push_str(" block_height ");
        line.push_str(&tx.block_height);
    }
    line
}

fn address_line(stats: &AddressStats) -> String {
    format!(
        "address {address} chain_tx_count {chain} funded_txo_sum {funded} mempool_tx_count {mempool}",
        address = stats.address,
        chain = stats.chain_tx_count,
        funded = stats.funded_sum,
        mempool = stats.mempool_tx_count,
    )
}

fn utxo_line(utxo: &Utxo) -> String {
    format!(
        "utxo {txid} vout {vout} value {value} confirmed {confirmed}",
        txid = utxo.txid,
        vout = utxo.vout,
        value = utxo.value,
        confirmed = utxo.confirmed,
    )
}

fn block_lines(body: &str, lines: &mut Vec<String>) {
    if closed_or_empty(body, lines) {
        return;
    }
    match parse_blocks(body) {
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
        match parse_mempool_recent(recent_json) {
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

fn append_txs(lines: &mut Vec<String>, txs: Vec<Tx>) {
    for tx in txs {
        lines.push(tx_line(&tx));
    }
}

pub fn block_fields_text(block_json: &str, txs_json: &str) -> String {
    let mut lines = Vec::new();
    if !closed_or_empty(block_json, &mut lines) {
        match parse_block(block_json) {
            Ok(block) => lines.push(block_line(&block)),
            Err(_) => push_unreadable(&mut lines),
        }
    }
    if !closed_or_empty(txs_json, &mut lines) {
        match parse_block_txs(txs_json) {
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
        match parse_tx(body) {
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
        match parse_address(stats_json) {
            Ok(stats) => lines.push(address_line(&stats)),
            Err(_) => push_unreadable(&mut lines),
        }
    }
    if !closed_or_empty(txs_json, &mut lines) {
        match parse_address_txs(txs_json) {
            Ok(txs) => append_txs(&mut lines, txs),
            Err(_) => push_unreadable(&mut lines),
        }
    }
    if !closed_or_empty(utxo_json, &mut lines) {
        match parse_address_utxos(utxo_json) {
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
        .map(|script| get(address_path(script)))
        .collect()
}

pub fn multi_address_fields_text(rows: &[(String, String)]) -> String {
    let mut lines = Vec::new();
    for (_path, body) in rows {
        if closed_or_empty(body, &mut lines) {
            continue;
        }
        match parse_address(body) {
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
    if body == FAIL_CLOSED {
        return body.to_string();
    }
    match parse_broadcast(body) {
        Ok(txid) => format!("txid {txid}"),
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

fn test_line(result: &TestTxResult) -> String {
    let mut line = format!(
        "txid {txid} allowed {allowed}",
        txid = result.txid,
        allowed = result.allowed,
    );
    if !result.reject_reason.is_empty() {
        line.push_str(" reject-reason ");
        line.push_str(&result.reject_reason);
    }
    line
}

pub fn test_transactions_fields_text(body: &str) -> String {
    let mut lines = Vec::new();
    if closed_or_empty(body, &mut lines) {
        return lines.join("\n");
    }
    match parse_test_txs(body) {
        Ok(rows) => {
            for result in rows {
                lines.push(test_line(&result));
            }
        }
        Err(_) => push_unreadable(&mut lines),
    }
    lines.join("\n")
}
