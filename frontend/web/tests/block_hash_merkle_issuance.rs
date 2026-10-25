//! Whole lines the native block and issuance screens already require.
//!
//! `Hash abcdef...hijklmn` is `shortenString` of length 13 on a 14-character id.
//! `merkle_root mm` sits beside the existing `Merkle root mm` line.
//! `issuance asset issuanceasset` sits beside the existing span.
//! A line match is the whole line, not a substring inside a longer line.

use splora_web::{
    block_details_markup, block_rows_markup, parse_transaction_json, transaction_row_markup,
};

const FOURTEEN: &str = "abcdefghijklmn";
const HASH_LINE: &str = "Hash abcdef...hijklmn";
const MERKLE_LINE: &str = "Merkle root mm";
const RAW_MERKLE_LINE: &str = "merkle_root mm";
const ISSUANCE_LINE: &str = "issuance asset issuanceasset";
const ISSUANCE_SPAN: &str = "<span>issuance asset issuanceasset</span>";

fn block_json(id: &str, merkle_root: &str) -> String {
    format!(
        r#"{{"id":"{id}","height":2,"version":1,"timestamp":1700000000,"tx_count":1,"size":100,"weight":400,"merkle_root":"{merkle_root}","previousblockhash":null,"mediantime":1699990000,"nonce":9,"bits":2,"difficulty":1}}"#
    )
}

fn issuance_tx() -> String {
    r#"{"txid":"issueprevtx","version":1,"locktime":0,"vin":[{"txid":"issueprev","vout":0,"prevout":null,"issuance":{"asset_id":"issuanceasset","is_reissuance":false,"asset_entropy":"entropyvalue"},"scriptsig":"deadbeef","scriptsig_asm":"OP_CHECKSIG","witness":["abcdwitnesshex"],"is_coinbase":false,"sequence":1}],"vout":[{"scriptpubkey":"0014aa","scriptpubkey_asm":"OP_0","scriptpubkey_type":"op_return","value":0}],"size":10,"weight":40,"sigops":0,"fee":1,"status":{"confirmed":true}}"#
        .to_string()
}

fn assert_one_line(markup: &str, line: &str) {
    assert!(
        markup.lines().any(|row| row == line),
        "{line} is absent: {markup}"
    );
    assert_eq!(
        markup.lines().filter(|row| *row == line).count(),
        1,
        "{line} was not one line: {markup}"
    );
}

/// Fourteen-character id paints `Hash abcdef...hijklmn` when details are open.
///
/// The hash cell on the block row stays the full id. Closed details omit the line.
#[test]
fn hash_merkle_issuance_paints_hash_abcdef_line() {
    assert_eq!(FOURTEEN.len(), 14);
    let body = block_json(FOURTEEN, "mm");
    let open = block_details_markup(&body, true);
    assert_one_line(&open, HASH_LINE);
    assert!(
        open.contains("<tr><td>Merkle root</td><td>mm</td></tr>"),
        "merkle cell was dropped: {open}"
    );

    let closed = block_details_markup(&body, false);
    assert!(
        !closed.lines().any(|line| line == HASH_LINE),
        "closed details painted {HASH_LINE}: {closed}"
    );

    let rows = block_rows_markup(&body, "[]");
    let hash_cell = format!("<td class=\"hash\"><a href=\"/block/{FOURTEEN}\">{FOURTEEN}</a></td>");
    assert_eq!(
        rows.matches(&hash_cell).count(),
        1,
        "hash cell was dropped: {rows}"
    );
    assert_eq!(
        rows.matches("<td class=\"short-hash\">abcdef...hijklmn</td>")
            .count(),
        1,
        "short-hash cell was dropped: {rows}"
    );
}

/// Thirteen-character id paints the whole line `Hash abcdefghijklm` when details are open.
///
/// Length 13 is not shortened. This line is not `Hash abcdef...hijklmn` and not `Hash abcdefghijkl`.
/// Closed details omit the line. The short-hash cell stays the whole id.
#[test]
fn open_block_details_paint_hash_abcdefghijklm() {
    const ID: &str = "abcdefghijklm";
    const LINE: &str = "Hash abcdefghijklm";
    assert_eq!(ID.len(), 13);
    let body = block_json(ID, "mm");
    let open = block_details_markup(&body, true);
    assert_one_line(&open, LINE);
    assert!(
        !open.lines().any(|line| line == "Hash abcdefghijkl"),
        "painted the 12-character hash line: {open}"
    );
    assert!(
        !open.lines().any(|line| line == "Hash abcdef...hijklmn"),
        "painted the 14-character shortened hash line: {open}"
    );

    let closed = block_details_markup(&body, false);
    assert!(
        !closed.lines().any(|line| line == LINE),
        "closed details painted {LINE}: {closed}"
    );

    let rows = block_rows_markup(&body, "[]");
    assert_eq!(
        rows.matches("<td class=\"short-hash\">abcdefghijklm</td>")
            .count(),
        1,
        "short-hash cell was dropped: {rows}"
    );
}

/// Open details paint `merkle_root mm` and keep `Merkle root mm`.
///
/// Closed details keep the cell and omit both whole lines.
#[test]
fn hash_merkle_issuance_paints_merkle_root_mm() {
    let id = "merklemm";
    let body = block_json(id, "mm");
    let open = block_details_markup(&body, true);
    assert_one_line(&open, RAW_MERKLE_LINE);
    assert_one_line(&open, MERKLE_LINE);
    assert_eq!(
        open.matches("<tr><td>Merkle root</td><td>mm</td></tr>")
            .count(),
        1,
        "Merkle root cell was dropped: {open}"
    );

    let closed = block_details_markup(&body, false);
    assert!(
        !closed.lines().any(|line| line == RAW_MERKLE_LINE),
        "closed details painted {RAW_MERKLE_LINE}: {closed}"
    );
    assert!(
        !closed.lines().any(|line| line == MERKLE_LINE),
        "closed details painted {MERKLE_LINE}: {closed}"
    );
    assert!(
        closed.contains("<tr><td>Merkle root</td><td>mm</td></tr>"),
        "closed details dropped the Merkle root cell: {closed}"
    );
}

/// `transaction_row_markup` reaches `issuance_markup` and paints the whole line.
///
/// The span stays. The phrase occurs twice: once inside the span, once as the line.
#[test]
fn hash_merkle_issuance_paints_issuance_asset_line() {
    let body = issuance_tx();
    let tx = parse_transaction_json(&body).expect("typed tx");
    let issued = tx.vin[0].issuance.as_ref().expect("issuance");
    assert_eq!(issued.asset_id, "issuanceasset");
    let markup = transaction_row_markup(&body);
    assert_one_line(&markup, ISSUANCE_LINE);
    assert_eq!(
        markup.matches(ISSUANCE_SPAN).count(),
        1,
        "issuance span was dropped or repeated: {markup}"
    );
    assert_eq!(
        markup.matches(ISSUANCE_LINE).count(),
        2,
        "span plus whole line should be two phrase matches: {markup}"
    );
}
