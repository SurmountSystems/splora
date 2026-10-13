//! Electrs paths each v1 page requests, and the field lines that page shows.

use splora_frontend_shared::wire::{
    AddressStats, Block, RecentTransaction, TestTxResult, Transaction, Utxo, recent_value_text,
};
use splora_frontend_shared::{
    TXS_OUTSPENDS_MAX, address_path, address_txs_chain_page_path, address_txs_mempool_path,
    address_txs_path, address_txs_summary_page_path, address_txs_summary_path, address_utxo_path,
    addresses_txs_body, addresses_txs_path, addresses_txs_summary_path, asset_path, asset_txs_path,
    block_header_path, block_height_path, block_path, block_raw_path, block_status_path,
    block_txid_path, block_txids_path, block_txs_path, block_txs_start_index_path, blocks_path,
    blocks_start_height_path, broadcast_body, broadcast_path, mempool_recent_path,
    mempool_txids_page_path, mempool_txids_path, package_body, package_path, parse_block_txids,
    sat_per_vbyte_from_vsize, sat_per_vbyte_from_weight, scripthash_path,
    scripthash_txs_mempool_path, scripthash_txs_path, scripthash_txs_summary_page_path,
    scripthash_txs_summary_path, scripthash_utxo_path, scripthashes_txs_body,
    scripthashes_txs_path, scripthashes_txs_summary_path, test_txs_body, tx_hex_path,
    tx_merkle_proof_path, tx_merkleblock_proof_path, tx_outspend_path, tx_outspends_path, tx_path,
    tx_raw_path, tx_status_path, txs_outspends_path,
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
    let indexer_error = trimmed.lines().any(|line| {
        let line = line.trim();
        line.starts_with("indexer error ") && line.len() > "indexer error ".len()
    });
    if indexer_error {
        return Some(trimmed);
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
    line.push_str(&format!(
        " version {version} merkle_root {merkle_root} nonce {nonce} bits {bits} difficulty {difficulty}",
        version = block.version,
        merkle_root = block.merkle_root,
        nonce = block.nonce,
        bits = block.bits,
        difficulty = plain_number(block.difficulty),
    ));
    line
}

fn append_fee_rate(line: &mut String, rate: Option<f64>) {
    let Some(rate) = rate.filter(|rate| rate.is_finite()) else {
        return;
    };
    line.push_str(" fee_rate ");
    line.push_str(&format!("{}", rate));
}

fn recent_line(tx: &RecentTransaction) -> String {
    let mut line = format!(
        "recent {txid} fee {fee} vsize {vsize} value {value} txid {txid}",
        txid = tx.txid,
        fee = tx.fee,
        vsize = tx.vsize,
        value = recent_value_text(tx.value, ""),
    );
    append_fee_rate(&mut line, sat_per_vbyte_from_vsize(tx.fee, tx.vsize));
    line
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
    append_tx_facts(&mut line, tx);
    line
}

fn has_address(address: Option<&str>) -> bool {
    address.map(str::trim).is_some_and(|text| !text.is_empty())
}

fn append_tx_facts(line: &mut String, tx: &Transaction) {
    line.push_str(" version ");
    line.push_str(&tx.version.to_string());
    line.push_str(" locktime ");
    line.push_str(&tx.lock_time.to_string());
    line.push_str(" sigops ");
    line.push_str(&tx.sigops.to_string());
    append_fee_rate(line, sat_per_vbyte_from_weight(tx.fee, tx.weight));
    for input in &tx.vin {
        line.push_str(" outpoint ");
        line.push_str(&input.txid);
        line.push_str(" vout ");
        line.push_str(&input.vout.to_string());
        if let Some(prevout) = &input.prevout {
            if !has_address(prevout.script_pubkey_address.as_deref()) {
                if let Some(value) = prevout.value {
                    line.push_str(" value ");
                    line.push_str(&value.to_string());
                }
            }
        }
    }
    for output in &tx.vout {
        if !has_address(output.script_pubkey_address.as_deref()) {
            if let Some(value) = output.value {
                line.push_str(" value ");
                line.push_str(&value.to_string());
            }
        }
    }
}

fn append_address_value(line: &mut String, address: Option<&str>, value: Option<u64>) {
    let Some(address) = address.map(str::trim).filter(|address| !address.is_empty()) else {
        return;
    };
    line.push_str(" address ");
    line.push_str(address);
    if let Some(value) = value {
        line.push_str(" value ");
        line.push_str(&value.to_string());
    }
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
    line.push_str(&format!(
        " chain_funded_txo_count {funded_count} chain_spent_txo_count {spent_count} chain_spent_txo_sum {spent_sum} mempool_funded_txo_count {mempool_funded_count} mempool_spent_txo_count {mempool_spent_count} mempool_spent_txo_sum {mempool_spent_sum}",
        funded_count = stats.chain_stats.funded_txo_count,
        spent_count = stats.chain_stats.spent_txo_count,
        spent_sum = stats.chain_stats.spent_txo_sum,
        mempool_funded_count = stats.mempool_stats.funded_txo_count,
        mempool_spent_count = stats.mempool_stats.spent_txo_count,
        mempool_spent_sum = stats.mempool_stats.spent_txo_sum,
    ));
    line
}

/// `Some(n)` prints that integer, including zero. `None` with a non-empty
/// value or asset commitment is the word Confidential. Otherwise the amount
/// is blank. Empty string commitments do not count.
pub(crate) enum UtxoAmount {
    Number(u64),
    Confidential,
    Blank,
}

pub(crate) fn utxo_amount(utxo: &Utxo) -> UtxoAmount {
    if let Some(amount) = utxo.value {
        return UtxoAmount::Number(amount);
    }
    let commitment = utxo
        .valuecommitment
        .as_deref()
        .is_some_and(|text| !text.is_empty())
        || utxo
            .assetcommitment
            .as_deref()
            .is_some_and(|text| !text.is_empty());
    if commitment {
        UtxoAmount::Confidential
    } else {
        UtxoAmount::Blank
    }
}

fn utxo_amount_text(utxo: &Utxo) -> String {
    match utxo_amount(utxo) {
        UtxoAmount::Number(amount) => amount.to_string(),
        UtxoAmount::Confidential => "Confidential".to_string(),
        UtxoAmount::Blank => String::new(),
    }
}

fn utxo_line(utxo: &Utxo) -> String {
    let mut line = format!(
        "utxo {txid} vout {vout} value {value} confirmed {confirmed}",
        txid = utxo.txid,
        vout = utxo.vout,
        value = utxo_amount_text(utxo),
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

/// 64 hex characters that include a letter. An all-digit id is a height.
pub(crate) fn is_block_hash(body: &str) -> bool {
    let hash = body.trim();
    hash.len() == 64
        && hash.bytes().all(|byte| byte.is_ascii_hexdigit())
        && hash.bytes().any(|byte| byte.is_ascii_alphabetic())
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
            let path = match id.parse::<u64>() {
                Ok(height) => block_height_path(height),
                // Digits that do not fit in u64 are still a height.
                Err(_) => format!("/block-height/{id}"),
            };
            return vec![get(path)];
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

/// A 64-hex id, including 64 digits, is a scripthash. Search still treats all digits as a height.
fn is_scripthash_id(id: &str) -> bool {
    let id = id.trim();
    id.len() == 64 && id.bytes().all(|byte| byte.is_ascii_hexdigit())
}

pub fn address_requests(script: &str) -> Vec<IndexerRequest> {
    let script = script.trim();
    if script.is_empty() {
        return Vec::new();
    }
    if is_scripthash_id(script) {
        let hash = script.to_ascii_lowercase();
        return vec![
            get(scripthash_path(&hash)),
            get(scripthash_txs_path(&hash)),
            get(scripthash_utxo_path(&hash)),
            get(scripthash_txs_mempool_path(&hash)),
        ];
    }
    vec![
        get(address_path(script)),
        get(address_txs_path(script)),
        get(address_utxo_path(script)),
    ]
}

pub fn address_summary_request(script: &str) -> IndexerRequest {
    let script = script.trim();
    if is_scripthash_id(script) {
        get(scripthash_txs_summary_path(&script.to_ascii_lowercase()))
    } else {
        get(address_txs_summary_path(script))
    }
}

pub fn address_summary_page_request(script: &str, last_seen_txid: &str) -> IndexerRequest {
    let script = script.trim();
    let last_seen_txid = last_seen_txid.trim();
    if is_scripthash_id(script) {
        get(scripthash_txs_summary_page_path(
            &script.to_ascii_lowercase(),
            last_seen_txid,
        ))
    } else {
        get(address_txs_summary_page_path(script, last_seen_txid))
    }
}

/// One POST body for several addresses, and another when the list is scripthashes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BatchPost {
    pub method: &'static str,
    pub path: &'static str,
    pub body: String,
}

fn batch_posts(
    values: &[String],
    address_post: &'static str,
    scripthash_post: &'static str,
) -> Vec<BatchPost> {
    let mut addresses = Vec::new();
    let mut hashes = Vec::new();
    for value in values {
        let value = value.trim();
        if value.is_empty() {
            continue;
        }
        if is_scripthash_id(value) {
            hashes.push(value.to_ascii_lowercase());
        } else {
            addresses.push(value.to_string());
        }
    }
    let mut posts = Vec::new();
    if !addresses.is_empty() {
        let refs: Vec<&str> = addresses.iter().map(String::as_str).collect();
        posts.push(BatchPost {
            method: "POST",
            path: address_post,
            body: addresses_txs_body(&refs),
        });
    }
    if !hashes.is_empty() {
        let refs: Vec<&str> = hashes.iter().map(String::as_str).collect();
        posts.push(BatchPost {
            method: "POST",
            path: scripthash_post,
            body: scripthashes_txs_body(&refs),
        });
    }
    posts
}

pub fn multi_address_summary_post(values: &[String]) -> Vec<BatchPost> {
    batch_posts(
        values,
        addresses_txs_summary_path(),
        scripthashes_txs_summary_path(),
    )
}

pub fn multi_address_txs_post(values: &[String]) -> Vec<BatchPost> {
    batch_posts(values, addresses_txs_path(), scripthashes_txs_path())
}

/// At most 50 transaction ids. An empty list is not a request.
pub fn txs_outspends_request(txids: &[String]) -> Option<IndexerRequest> {
    let kept: Vec<&str> = txids
        .iter()
        .map(|txid| txid.trim())
        .filter(|txid| !txid.is_empty())
        .take(TXS_OUTSPENDS_MAX)
        .collect();
    txs_outspends_path(&kept).ok().map(get)
}

pub fn tx_outspend_index_request(txid: &str, index: u64) -> IndexerRequest {
    get(tx_outspend_path(txid.trim(), index))
}

pub fn tx_status_request(txid: &str) -> IndexerRequest {
    get(tx_status_path(txid.trim()))
}

pub fn block_header_request(hash: &str) -> IndexerRequest {
    get(block_header_path(hash.trim()))
}

pub fn block_raw_request(hash: &str) -> IndexerRequest {
    get(block_raw_path(hash.trim()))
}

pub fn block_txid_index_request(hash: &str, index: u64) -> IndexerRequest {
    get(block_txid_path(hash.trim(), index))
}

pub fn tx_raw_request(txid: &str) -> IndexerRequest {
    get(tx_raw_path(txid.trim()))
}

pub fn tx_merkleblock_proof_request(txid: &str) -> IndexerRequest {
    get(tx_merkleblock_proof_path(txid.trim()))
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

pub(crate) fn plain_number(value: f64) -> String {
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

/// Older blocks start one height below the lowest block on this page.
pub fn blocks_older_href(blocks_json: &str) -> Option<String> {
    let blocks = parse_blocks_json(blocks_json).ok()?;
    let lowest = blocks.iter().map(|block| block.height).min()?;
    if lowest == 0 {
        return None;
    }
    Some(format!("/blocks?start_height={}", u64::from(lowest) - 1))
}

/// A height route becomes a block hash once `/block-height/` or the block body has one.
pub fn resolved_block_hash(route_id: &str, height_body: &str, block_json: &str) -> Option<String> {
    let route_id = route_id.trim();
    if is_block_hash(route_id) {
        return Some(route_id.to_string());
    }
    if is_block_hash(height_body) {
        return Some(height_body.trim().to_string());
    }
    let block = parse_block_json(block_json).ok()?;
    let id = block.id.trim();
    if is_block_hash(id) {
        Some(id.to_string())
    } else {
        None
    }
}

/// The next block transaction page is 25 past this start. Other starts are not a page.
pub fn block_txs_older_href(hash: &str, start_index: u32) -> Option<String> {
    if start_index % 25 != 0 {
        return None;
    }
    let next = start_index.checked_add(25)?;
    Some(format!("/block/{hash}?start_index={next}"))
}

pub fn address_chain_request(script: &str, last_seen_txid: &str) -> IndexerRequest {
    get(address_txs_chain_page_path(script, last_seen_txid))
}

pub fn address_mempool_txs_request(script: &str) -> IndexerRequest {
    get(address_txs_mempool_path(script))
}

pub fn mempool_txids_request() -> IndexerRequest {
    get(mempool_txids_path().to_string())
}

pub fn mempool_txid_page_request(last_seen_txid: &str) -> IndexerRequest {
    get(mempool_txids_page_path(last_seen_txid))
}

pub fn transaction_detail_requests(txid: &str) -> Vec<IndexerRequest> {
    let txid = txid.trim();
    if txid.is_empty() {
        return Vec::new();
    }
    vec![
        get(tx_outspends_path(txid)),
        get(tx_merkle_proof_path(txid)),
        get(tx_hex_path(txid)),
    ]
}

pub fn block_status_request(hash: &str) -> IndexerRequest {
    get(block_status_path(hash))
}

/// The indexer's sentence stays. The method and path stay on the next line.
pub fn indexer_failure_text(method: &str, path: &str, body: &str) -> String {
    let fallback = format!("indexer error {method} {path}");
    let trimmed = body.trim();
    if trimmed.is_empty()
        || trimmed.starts_with("indexer error ")
        || trimmed.starts_with('{')
        || trimmed.starts_with('[')
        || trimmed.contains('\n')
    {
        return fallback;
    }
    format!("{trimmed}\n{fallback}")
}

pub struct BroadcastSubmission {
    pub method: &'static str,
    pub path: &'static str,
    pub body: String,
}

/// One hex is `POST /tx`. More than one hex is `POST /txs/package`.
pub fn broadcast_submission(raw: &str) -> BroadcastSubmission {
    let parts: Vec<&str> = raw
        .split(|ch: char| ch.is_whitespace() || ch == ',')
        .map(str::trim)
        .filter(|part| !part.is_empty())
        .collect();
    if parts.len() > 1 {
        BroadcastSubmission {
            method: "POST",
            path: package_path(),
            body: package_body(&parts),
        }
    } else {
        let body = parts
            .first()
            .copied()
            .map(broadcast_body)
            .unwrap_or_default();
        BroadcastSubmission {
            method: "POST",
            path: broadcast_path(),
            body,
        }
    }
}

/// Liquid asks for the asset and its transactions. Other networks do not.
pub fn asset_requests(liquid: bool, asset_id: &str) -> Vec<IndexerRequest> {
    let asset_id = asset_id.trim();
    if !liquid || asset_id.is_empty() {
        return Vec::new();
    }
    vec![get(asset_path(asset_id)), get(asset_txs_path(asset_id))]
}
