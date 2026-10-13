//! The address list, the block heading, and the recent list call the shared helpers.
//!
//! `highlight_counts` decides which address cells get `class="highlight"`.
//! `transaction_count_heading` is the only "1 transaction" plural.
//! `cap_recent_transactions` keeps the first 10 mempool rows.

use splora_web::{address_highlight_markup, block_transactions_markup, recent_rows_markup};

const HASH: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";

fn rows_fn(name: &str) -> &'static str {
    let src = include_str!("../src/rows.rs");
    let marker = format!("fn {name}(");
    let start = src
        .find(&marker)
        .unwrap_or_else(|| panic!("{name} is missing from rows.rs"));
    let after_name = &src[start..];
    let brace_rel = after_name.find('{').expect("function has no body");
    let body_start = start + brace_rel + 1;
    let mut depth = 1i32;
    for (offset, ch) in src[body_start..].char_indices() {
        match ch {
            '{' => depth += 1,
            '}' => {
                depth -= 1;
                if depth == 0 {
                    return &src[body_start..body_start + offset];
                }
            }
            _ => {}
        }
    }
    panic!("{name} body is not closed");
}

fn recent_row(index: u32) -> String {
    format!(r#"{{"txid":"recent-{index:02}","fee":800,"vsize":141,"value":99000}}"#)
}

fn block_json(tx_count: u32) -> String {
    format!(
        r#"{{"id":"{HASH}","height":2,"version":1,"timestamp":1700000000,"tx_count":{tx_count},"size":100,"weight":400,"merkle_root":"merkle","previousblockhash":null,"mediantime":1699990000,"nonce":1,"bits":1,"difficulty":1}}"#
    )
}

fn h2_text(markup: &str) -> String {
    let start = markup.find("<h2").expect("missing h2");
    let rest = &markup[start..];
    let open = rest.find('>').expect("h2 is not opened");
    let close = rest.find("</h2>").expect("h2 is not closed");
    rest[open + 1..close].to_string()
}

#[test]
fn recent_list_keeps_the_first_ten() {
    let body = rows_fn("recent_rows_markup");
    assert!(
        body.contains("cap_recent_transactions("),
        "recent_rows_markup does not call cap_recent_transactions"
    );
    let rows: Vec<String> = (0..12).map(recent_row).collect();
    let markup = recent_rows_markup(&format!("[{}]", rows.join(",")));
    for index in 0..10 {
        let txid = format!("recent-{index:02}");
        assert!(markup.contains(&txid), "missing {txid} in {markup}");
    }
    assert!(!markup.contains("recent-10"), "{markup}");
    assert!(!markup.contains("recent-11"), "{markup}");
    let short: Vec<String> = (0..3).map(recent_row).collect();
    let short_markup = recent_rows_markup(&format!("[{}]", short.join(",")));
    assert!(short_markup.contains("recent-02"), "{short_markup}");
    assert!(!markup.contains("Fiat price last updated"), "{markup}");
}

#[test]
fn block_heading_calls_transaction_count_heading() {
    let body = rows_fn("block_transactions_markup");
    assert!(
        body.contains("transaction_count_heading("),
        "block_transactions_markup keeps a private plural: {body}"
    );
    assert!(
        !body.contains("\"1 transaction\""),
        "private singular remains: {body}"
    );
    let one = block_transactions_markup(&block_json(1), "[]", 1);
    let many = block_transactions_markup(&block_json(41), "[]", 1);
    assert_eq!(h2_text(&one), "1 transaction");
    assert_eq!(h2_text(&many), "41 transactions");
    assert!(!one.contains("Fiat price last updated"));
}

#[test]
fn address_list_highlight_calls_highlight_counts() {
    let body = rows_fn("address_highlight_markup");
    assert!(
        body.contains("highlight_counts("),
        "address_highlight_markup does not call highlight_counts"
    );
    let page = "1wiz32gbHZwMzJCRHMGehJuBgsMTPdaCa";
    let other = "bc1qother000000000000000000000000000000";
    let txs = format!(
        r#"[{{"txid":"aa","version":1,"locktime":0,"vin":[{{"txid":"prev","vout":0,"prevout":{{"scriptpubkey":"0014aa","scriptpubkey_asm":"OP_0","scriptpubkey_type":"v0_p2wpkh","scriptpubkey_address":"{page}","value":1}},"scriptsig":"","scriptsig_asm":"","is_coinbase":false,"sequence":1}}],"vout":[{{"scriptpubkey":"76a914bb88ac","scriptpubkey_asm":"OP_DUP","scriptpubkey_type":"p2pkh","scriptpubkey_address":"{other}","value":2}}],"size":10,"weight":40,"sigops":0,"fee":1}}]"#
    );
    let markup = address_highlight_markup(page, &txs);
    assert!(
        markup.contains("<span class=\"highlight\">"),
        "highlight class is missing: {markup}"
    );
    assert!(!markup.contains("Fiat price last updated"), "{markup}");
}
