//! Peg-in, Liquid Peg Out, and a confidential label from the transaction JSON.
//!
//! The shared parser keeps `is_pegin`, `pegout`, `valuecommitment`, and `asset`.
//! The page reads those fields. It does not unblind a hash.

use splora_web::{address_rows_markup, block_rows_markup, transaction_row_markup};

const PEG_ADDRESS: &str = "1BxoGcMg14oaH3CwHD2hF4gU9VcfgX5yoR";
const ASSET_ID: &str = "6f0279e9ed041c3d710a9f57d0c02928416460c4b722ae3457a11eec381c526d";

fn tx(txid: &str, vin: &str, vout: &str) -> String {
    format!(
        r#"{{"txid":"{txid}","version":2,"locktime":0,"vin":[{vin}],"vout":[{vout}],"size":10,"weight":40,"sigops":0,"fee":1}}"#
    )
}

fn input(txid: &str, prevout: &str, extra: &str) -> String {
    let extra = if extra.is_empty() {
        String::new()
    } else {
        format!(",{extra}")
    };
    format!(
        r#"{{"txid":"{txid}","vout":0,"prevout":{prevout},"scriptsig":"","scriptsig_asm":"","is_coinbase":false,"sequence":1{extra}}}"#
    )
}

fn output(address: &str, value: &str, extra: &str) -> String {
    let address_field = if address.is_empty() {
        String::new()
    } else {
        format!(r#","scriptpubkey_address":"{address}""#)
    };
    let extra = if extra.is_empty() {
        String::new()
    } else {
        format!(",{extra}")
    };
    format!(
        r#"{{"scriptpubkey":"76a914bb88ac","scriptpubkey_asm":"OP_DUP","scriptpubkey_type":"p2pkh"{address_field},"value":{value}{extra}}}"#
    )
}

fn pegout_object() -> String {
    format!(
        r#""pegout":{{"genesis_hash":"000000000019d6689c085ae165831e934ff763ae46a2a6c172b3f1b60a8ce26f","scriptpubkey":"76a91489abcdefabbaabbaabbaabbaabbaabbaabbaabba88ac","scriptpubkey_asm":"OP_DUP OP_HASH160 OP_EQUALVERIFY OP_CHECKSIG","scriptpubkey_address":"{PEG_ADDRESS}"}}"#
    )
}

fn block_json() -> String {
    r#"{"id":"abc","height":1,"version":1,"timestamp":1,"tx_count":1,"size":1,"weight":4,"merkle_root":"mm","previousblockhash":"00","mediantime":1,"nonce":1,"bits":1,"difficulty":1.0}"#.to_string()
}

fn address_stats() -> String {
    format!(
        r#"{{"address":"{PEG_ADDRESS}","chain_stats":{{"tx_count":1,"funded_txo_count":1,"spent_txo_count":0,"funded_txo_sum":9,"spent_txo_sum":0}},"mempool_stats":{{"tx_count":0,"funded_txo_count":0,"spent_txo_count":0,"funded_txo_sum":0,"spent_txo_sum":0}}}}"#
    )
}

fn page_body(name: &str) -> String {
    let src = include_str!("../src/views.rs");
    let marker = format!("fn {name}(");
    let start = src
        .find(&marker)
        .unwrap_or_else(|| panic!("{name} is missing"));
    let after = &src[start..];
    let brace = after
        .find('{')
        .unwrap_or_else(|| panic!("{name} has no body"));
    let body_start = start + brace + 1;
    let mut depth = 1i32;
    for (offset, ch) in src[body_start..].char_indices() {
        match ch {
            '{' => depth += 1,
            '}' => {
                depth -= 1;
                if depth == 0 {
                    return src[body_start..body_start + offset].to_string();
                }
            }
            _ => {}
        }
    }
    panic!("{name} body is not closed");
}

fn assert_no_invented_unblinding(markup: &str) {
    assert!(!markup.contains("Fiat price last updated"), "{markup}");
    assert!(!markup.contains("0.00100000"), "{markup}");
    assert!(!markup.contains("0.02364760"), "{markup}");
    assert!(!markup.contains("Error: Invalid blinding data"), "{markup}");
}

#[test]
fn pegin_input_shows_peg_in_on_the_transaction_and_the_block() {
    let body = tx(
        "pegin-tx",
        &input("pegin-prev", "null", r#""is_pegin":true"#),
        &output("bc1qchange", "9", ""),
    );
    let markup = transaction_row_markup(&body);
    assert!(
        markup.contains("<table class=\"table-tx-vin\">"),
        "{markup}"
    );
    assert!(markup.contains("Peg-in"), "{markup}");
    assert!(markup.contains("href=\"/tx/pegin-prev\""), "{markup}");
    assert!(!markup.contains("Liquid Peg Out"), "{markup}");
    assert_no_invented_unblinding(&markup);

    let block = block_rows_markup(&block_json(), &format!("[{body}]"));
    assert!(block.contains("Peg-in"), "{block}");
    assert!(!block.contains("Liquid Peg Out"), "{block}");
}

#[test]
fn pegout_output_shows_the_badge_and_the_address_link() {
    let body = tx(
        "pegout-tx",
        &input("pegout-prev", &output("bc1qsource", "10", ""), ""),
        &output("", "0", &pegout_object()),
    );
    let markup = transaction_row_markup(&body);
    assert!(
        markup.contains("<table class=\"table-tx-vout\">"),
        "{markup}"
    );
    assert!(markup.contains("Peg-out to"), "{markup}");
    assert!(
        markup.contains(&format!("href=\"/address/{PEG_ADDRESS}\"")),
        "{markup}"
    );
    assert!(
        markup.contains("<span class=\"badge\">Liquid Peg Out</span>"),
        "{markup}"
    );
    assert!(!markup.contains(">Peg-in<"), "{markup}");
    assert_no_invented_unblinding(&markup);
}

/// Whole line beside the peg-out badge.
///
/// A line equal to Liquid Peg Out passes. The badge stays. A null peg-out
/// and a peg-in do not gain the words.
#[test]
fn pegout_output_paints_liquid_peg_out_as_its_own_line() {
    const LINE: &str = "Liquid Peg Out";
    let body = tx(
        "pegout-tx",
        &input("pegout-prev", &output("bc1qsource", "10", ""), ""),
        &output("", "0", &pegout_object()),
    );
    let markup = transaction_row_markup(&body);
    assert!(
        markup.lines().any(|line| line == LINE),
        "no line equals {LINE}: {markup}"
    );
    assert_eq!(
        markup.lines().filter(|line| *line == LINE).count(),
        1,
        "{LINE} was not painted once: {markup}"
    );
    assert!(
        markup.contains("<span class=\"badge\">Liquid Peg Out</span>"),
        "the badge was dropped: {markup}"
    );
    let quiet = tx(
        "plain-tx",
        &input(
            "prev",
            &output("bc1qinput", "10", ""),
            r#""is_pegin":false"#,
        ),
        &output("bc1qoutput", "9", r#""pegout":null"#),
    );
    let omitted = transaction_row_markup(&quiet);
    assert!(
        !omitted.contains(LINE),
        "a null peg-out painted {LINE}: {omitted}"
    );
    let pegin = tx(
        "pegin-tx",
        &input("pegin-prev", "null", r#""is_pegin":true"#),
        &output("bc1qchange", "9", ""),
    );
    let pegin_markup = transaction_row_markup(&pegin);
    assert!(
        !pegin_markup.contains(LINE),
        "a peg-in painted {LINE}: {pegin_markup}"
    );
}

#[test]
fn false_pegin_and_null_pegout_do_not_invent_a_label() {
    let body = tx(
        "plain-tx",
        &input(
            "prev",
            &output("bc1qinput", "10", ""),
            r#""is_pegin":false"#,
        ),
        &output("bc1qoutput", "9", r#""pegout":null"#),
    );
    let markup = transaction_row_markup(&body);
    assert!(!markup.contains("Peg-in"), "{markup}");
    assert!(!markup.contains("Liquid Peg Out"), "{markup}");
    assert!(!markup.contains("Confidential"), "{markup}");
    assert!(markup.contains("bc1qoutput"), "{markup}");
    assert_no_invented_unblinding(&markup);
}

#[test]
fn address_list_shows_the_peg_out_badge_only_when_the_json_has_it() {
    let pegout = tx(
        "pegout-tx",
        &input("pegout-prev", &output("bc1qsource", "10", ""), ""),
        &output("", "0", &pegout_object()),
    );
    let painted = address_rows_markup(&address_stats(), &format!("[{pegout}]"), "[]");
    assert!(
        painted.contains("<span class=\"badge\">Liquid Peg Out</span>"),
        "{painted}"
    );
    assert!(
        painted.contains(&format!("href=\"/address/{PEG_ADDRESS}\"")),
        "{painted}"
    );

    let plain = tx(
        "plain-tx",
        &input("prev", &output("bc1qinput", "10", ""), ""),
        &output(PEG_ADDRESS, "9", ""),
    );
    let quiet = address_rows_markup(&address_stats(), &format!("[{plain}]"), "[]");
    assert!(!quiet.contains("Peg-in"), "{quiet}");
    assert!(!quiet.contains("Liquid Peg Out"), "{quiet}");
    assert!(quiet.contains(PEG_ADDRESS), "{quiet}");
    assert_no_invented_unblinding(&quiet);
}

#[test]
fn confidential_commitment_is_a_label_and_not_an_unblinded_amount() {
    let blinded = output(
        "lq1qconfidential",
        "null",
        r#""valuecommitment":"09aa","assetcommitment":"0abb""#,
    );
    // `output` always writes a value key. Replace the null so the key is absent,
    // which is how a confidential output is serialized.
    let blinded = blinded.replacen("\"value\":null,", "", 1);
    let body = tx("blind-tx", &input("prev", "null", ""), &blinded);
    assert!(
        !body.contains("\"value\""),
        "the fixture must omit value, not invent one: {body}"
    );
    let markup = transaction_row_markup(&body);
    assert!(
        markup.contains("<span class=\"amount\">Confidential</span>"),
        "{markup}"
    );
    assert!(!markup.contains("09aa"), "{markup}");
    assert!(!markup.contains("0abb"), "{markup}");
    assert_no_invented_unblinding(&markup);
}

#[test]
fn explicit_asset_id_in_the_json_is_linked() {
    let extra = format!(r#""asset":"{ASSET_ID}""#);
    let body = tx(
        "issued-tx",
        &input("prev", &output("lq1qsource", "100000", &extra), ""),
        &output("lq1qdest", "100000", &extra),
    );
    let markup = transaction_row_markup(&body);
    let link = format!("href=\"/asset/{ASSET_ID}\"");
    assert!(markup.contains(&link), "{markup}");
    assert!(markup.contains(">100000<"), "{markup}");
    assert!(!markup.contains("Confidential"), "{markup}");
    assert_no_invented_unblinding(&markup);
}

#[test]
fn transaction_and_address_pages_paint_the_row_markup() {
    let transaction = page_body("TransactionPage");
    let address = page_body("AddressPage");
    let block = page_body("BlockPage");
    assert!(
        transaction.contains("transaction_row_markup("),
        "{transaction}"
    );
    assert!(address.contains("address_rows_markup("), "{address}");
    assert!(block.contains("block_rows_markup("), "{block}");
    assert!(!transaction.contains("Fiat price last updated"));
    assert!(!address.contains("Fiat price last updated"));
    assert!(!block.contains("Fiat price last updated"));
}
