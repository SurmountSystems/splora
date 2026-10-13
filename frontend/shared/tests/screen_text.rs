//! Screen text the address, block, and recent-transaction pages already show.
//!
//! Highlight counts follow the desktop `.highlight` rule on
//! `GET /address/:address/txs`. The block heading is the English plural of
//! `Block.tx_count`. The recent list is the first 10 rows from
//! `GET /mempool/recent`. No virtual-byte rate and no projected block.

use splora_frontend_shared::wire::{RecentTransaction, Transaction};
use splora_frontend_shared::{
    address_txs_path, cap_recent_transactions, highlight_counts, mempool_recent_path,
    transaction_count_heading,
};

#[test]
fn address_and_recent_routes_stay_on_the_indexer() {
    assert_eq!(
        address_txs_path("1wiz1rtKFBA58qjb582WF5KAFg9mWCuZV"),
        "/address/1wiz1rtKFBA58qjb582WF5KAFg9mWCuZV/txs"
    );
    assert_eq!(mempool_recent_path(), "/mempool/recent");
}

#[test]
fn highlight_counts_follow_each_transaction() {
    let address = "1wiz1rtKFBA58qjb582WF5KAFg9mWCuZV";
    let other = "1otherAddress111111111111111111111";
    let single_input = transaction(
        "tx-0",
        &input(&output("p2pkh", Some(address), "76a91400")),
        &output("p2pkh", Some(other), "76a91411"),
    );
    let both_sides = transaction(
        "tx-1",
        &input(&output(
            "v0_p2wpkh",
            Some("bc1q03u63r6hm7a3v6em58zdqtp446w2pw30nm63mv"),
            "0014aa",
        )),
        &output(
            "v0_p2wpkh",
            Some("bc1q03u63r6hm7a3v6em58zdqtp446w2pw30nm63mv"),
            "0014bb",
        ),
    );
    let two_outputs = transaction(
        "tx-1-outs",
        &input(&output("p2pkh", Some(other), "76a91422")),
        &format!(
            "{},{}",
            output(
                "p2pkh",
                Some("1F3Q3sQmiGsWSqK5K6T9tYnX8yqzYRgQbe"),
                "76a91433"
            ),
            output(
                "p2pkh",
                Some("1F3Q3sQmiGsWSqK5K6T9tYnX8yqzYRgQbe"),
                "76a91444"
            ),
        ),
    );
    let two_inputs = transaction(
        "tx-2",
        &format!(
            "{},{},{}",
            input(&output("p2pkh", Some(address), "76a91455")),
            input(&output("p2pkh", Some(address), "76a91466")),
            input(&output("p2pkh", Some(other), "76a91477")),
        ),
        &output("p2pkh", Some(other), "76a91488"),
    );
    let counts = highlight_counts(address, &[single_input, two_inputs]);
    assert_eq!(counts.len(), 2);
    assert_eq!(counts[0].inputs, 1);
    assert_eq!(counts[0].outputs, 0);
    assert_eq!(counts[1].inputs, 2);
    assert_eq!(counts[1].outputs, 0);

    let output_address = "1F3Q3sQmiGsWSqK5K6T9tYnX8yqzYRgQbe";
    let output_counts = highlight_counts(output_address, &[two_outputs]);
    assert_eq!(output_counts.len(), 1);
    assert_eq!(output_counts[0].inputs, 0);
    assert_eq!(output_counts[0].outputs, 2);

    let both_address = "bc1q03u63r6hm7a3v6em58zdqtp446w2pw30nm63mv";
    let both_counts = highlight_counts(both_address, &[both_sides]);
    assert_eq!(both_counts[0].inputs, 1);
    assert_eq!(both_counts[0].outputs, 1);
}

#[test]
fn p2pk_matches_the_script_middle_and_coinbase_matches_nothing() {
    let pubkey = "02abcdef";
    let script = format!("21{pubkey}ac");
    assert_eq!(&script[2..script.len() - 2], pubkey);
    let p2pk = transaction(
        "p2pk-out",
        &input(&output(
            "p2pkh",
            Some("1otherAddress111111111111111111111"),
            "76a91499",
        )),
        &output_without_address("p2pk", &script),
    );
    let counts = highlight_counts(pubkey, &[p2pk]);
    assert_eq!(counts[0].inputs, 0);
    assert_eq!(counts[0].outputs, 1);

    let coinbase = transaction(
        "coinbase",
        &coinbase_input(),
        &output(
            "p2pkh",
            Some("1wiz32gbHZwMzJCRHMGehJuBgsMTPdaCa"),
            "76a914aa",
        ),
    );
    let ignored = highlight_counts(
        "1wiz32gbHZwMzJCRHMGehJuBgsMTPdaCa",
        std::slice::from_ref(&coinbase),
    );
    assert_eq!(ignored[0].inputs, 0);
    assert_eq!(ignored[0].outputs, 1);
    let absent = highlight_counts("1notInThisTransaction000000000000000", &[coinbase]);
    assert_eq!(absent[0].inputs, 0);
    assert_eq!(absent[0].outputs, 0);
}

#[test]
fn transaction_heading_is_singular_only_for_one() {
    assert_eq!(transaction_count_heading(1), "1 transaction");
    assert_eq!(transaction_count_heading(19), "19 transactions");
    assert_eq!(transaction_count_heading(13), "13 transactions");
    assert_eq!(transaction_count_heading(18), "18 transactions");
    assert_eq!(transaction_count_heading(0), "0 transactions");
}

#[test]
fn recent_list_keeps_the_first_ten() {
    let rows: Vec<RecentTransaction> = (0..50)
        .map(|index| RecentTransaction {
            txid: format!("recent-{index}"),
            fee: 800,
            vsize: 141,
            value: Some(99_000),
        })
        .collect();
    let capped = cap_recent_transactions(&rows);
    assert_eq!(capped.len(), 10);
    assert_eq!(capped[0].txid, "recent-0");
    assert_eq!(capped[9].txid, "recent-9");
    assert_eq!(capped[0].fee, 800);
    assert_eq!(capped[0].vsize, 141);
    assert_eq!(capped[0].value, Some(99_000));
    assert!(capped.iter().all(|row| row.txid != "recent-10"));
    assert_eq!(cap_recent_transactions(&rows[..3]).len(), 3);
    assert_eq!(cap_recent_transactions(&rows[..3])[2].txid, "recent-2");
    assert!(cap_recent_transactions(&[]).is_empty());
}

fn transaction(txid: &str, vin: &str, vout: &str) -> Transaction {
    let body = format!(
        r#"{{"txid":"{txid}","version":2,"locktime":0,"vin":[{vin}],"vout":[{vout}],"size":180,"weight":720,"sigops":1,"fee":4500}}"#
    );
    serde_json::from_str(&body).unwrap()
}

fn output(kind: &str, address: Option<&str>, script: &str) -> String {
    match address {
        Some(address) => format!(
            r#"{{"scriptpubkey":"{script}","scriptpubkey_asm":"OP_0","scriptpubkey_type":"{kind}","scriptpubkey_address":"{address}","value":1}}"#
        ),
        None => output_without_address(kind, script),
    }
}

fn output_without_address(kind: &str, script: &str) -> String {
    format!(
        r#"{{"scriptpubkey":"{script}","scriptpubkey_asm":"OP_0","scriptpubkey_type":"{kind}","value":1}}"#
    )
}

fn input(prevout: &str) -> String {
    format!(
        r#"{{"txid":"prev","vout":0,"prevout":{prevout},"scriptsig":"","scriptsig_asm":"","is_coinbase":false,"sequence":4294967295}}"#
    )
}

fn coinbase_input() -> String {
    r#"{"txid":"0000000000000000000000000000000000000000000000000000000000000000","vout":0,"prevout":null,"scriptsig":"","scriptsig_asm":"","is_coinbase":true,"sequence":4294967295}"#.to_string()
}
