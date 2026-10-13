//! Address poisoning screen parts from `GET /tx/:txid`.
//!
//! The Cypress rows are `address poisoning` in
//! `ref/mempool/frontend/cypress/e2e/mainnet/mainnet.spec.ts`.
//! Prefix, infix, and postfix are the address-text spans, not the HTML
//! leading space on `.postfix`. Virtual size is not involved.

use splora_frontend_shared::wire::Transaction;
use splora_frontend_shared::{address_parts, poison_alerts, tx_path};

#[test]
fn transaction_route_is_get_tx_without_a_public_prefix() {
    let txid = "152a5dea805f95d6f83e50a9fd082630f542a52a076ebabdb295723eaf53fa30";
    assert_eq!(tx_path(txid), format!("/tx/{txid}"));
}

#[test]
fn donate_outputs_split_into_prefix_and_infix_with_two_alerts() {
    let outputs = [
        output("p2pkh", "1DonatePLeaseSenderAddressXVXCmAY"),
        output("p2pkh", "1FeexV6bAHb8ybZjqQMjJrcCrHGW9sb6uF"),
        output("p2pkh", "1DonatePLease5btcToSenderXXWBoKhB"),
        output(
            "v0_p2wsh",
            "bc1qjasf9z3h7w3jspkhtgatgpyvvzgpa2wwd2lr0eh5tx44reyn2k7sfc27a4",
        ),
        output("v0_p2wpkh", "bc1q8yj0herd4r4yxszw3nkfvt53433thk0f5qst4g"),
        output("p2pkh", "1Ay8vMC7R1UbyCCZRVULMV7iQpHSAbguJP"),
        output(
            "v0_p2wsh",
            "bc1qd4ysezhmypwty5dnw7c8nqy5h5nxg0xqsvaefd0qn5kq32vwnwqqgv4rzr",
        ),
        output(
            "v1_p2tr",
            "bc1pahzjr2vnfqnqwl0h8jps2aa4zqe29wqxn2nagvd5a2z982k7ttdq8ry370",
        ),
    ];
    let vin = input(&output(
        "v1_p2tr",
        "bc1pahzjr2vnfqnqwl0h8jps2aa4zqe29wqxn2nagvd5a2z982k7ttdq8ry370",
    ));
    let tx = transaction(
        "152a5dea805f95d6f83e50a9fd082630f542a52a076ebabdb295723eaf53fa30",
        &vin,
        &outputs.join(","),
    );
    let alerts = poison_alerts(&tx, "mainnet");
    assert_eq!(alerts.len(), 2);
    assert_eq!(alerts[0].address, "1DonatePLeaseSenderAddressXVXCmAY");
    assert_eq!(alerts[0].script_type, "p2pkh");
    assert_eq!(alerts[0].parts.prefix, "1DonatePLease");
    assert_eq!(alerts[0].parts.infix, "SenderAddressXVXCmAY");
    assert_eq!(alerts[0].parts.postfix, "");
    assert_eq!(alerts[0].group, 0);
    assert_eq!(alerts[1].address, "1DonatePLease5btcToSenderXXWBoKhB");
    assert_eq!(alerts[1].script_type, "p2pkh");
    assert_eq!(alerts[1].parts.prefix, "1DonatePLease");
    assert_eq!(alerts[1].parts.infix, "5btcToSenderXXWBoKhB");
    assert_eq!(alerts[1].parts.postfix, "");
    assert_eq!(alerts[1].group, 0);
    assert!(
        alerts
            .iter()
            .all(|row| row.address != "1FeexV6bAHb8ybZjqQMjJrcCrHGW9sb6uF")
    );
    assert!(
        alerts
            .iter()
            .all(|row| row.address != "1Ay8vMC7R1UbyCCZRVULMV7iQpHSAbguJP")
    );
}

#[test]
fn bech32_input_and_output_split_prefix_infix_and_postfix() {
    let incoming = "bc1qge86gqjnk8aqs3nvv7ejrvcd4zq6qur36p6e3r";
    let outgoing = "bc1qge8xyxjex6zzzx5g8hh65vsel4e548p26p6e3r";
    let tx = transaction(
        "44544516084ea916ff1eb69c675c693e252addbbaf77102ffff86e3979ac6132",
        &input(&output("v0_p2wpkh", incoming)),
        &output("v0_p2wpkh", outgoing),
    );
    let alerts = poison_alerts(&tx, "mainnet");
    assert_eq!(alerts.len(), 2);
    assert_eq!(alerts[0].address, incoming);
    assert_eq!(alerts[0].script_type, "v0_p2wpkh");
    assert_eq!(alerts[0].parts.prefix, "bc1qge8");
    assert_eq!(alerts[0].parts.infix, "6gqjnk8aqs3nvv7ejrvcd4zq6qur3");
    assert_eq!(alerts[0].parts.postfix, "6p6e3r");
    assert_eq!(alerts[0].group, 0);
    assert_eq!(alerts[1].address, outgoing);
    assert_eq!(alerts[1].script_type, "v0_p2wpkh");
    assert_eq!(alerts[1].parts.prefix, "bc1qge8");
    assert_eq!(alerts[1].parts.infix, "xyxjex6zzzx5g8hh65vsel4e548p2");
    assert_eq!(alerts[1].parts.postfix, "6p6e3r");
    assert_eq!(alerts[1].group, 0);
}

#[test]
fn base58_similarity_threshold_is_one_million_to_the_alphabet() {
    assert!(address_parts("1abcAAAA", "1abcBBBB", "p2pkh", "mainnet", 1).is_none());
    let parts = address_parts("1abcdAAAA", "1abcdBBBB", "p2pkh", "mainnet", 1).unwrap();
    assert_eq!(parts.prefix, "1abcd");
    assert_eq!(parts.infix, "AAAA");
    assert_eq!(parts.postfix, "");
    let wrapped = address_parts("3abcdefZZZZ", "3abcdefYYYY", "p2sh", "mainnet", 1).unwrap();
    assert_eq!(wrapped.prefix, "3abcdef");
    assert_eq!(wrapped.infix, "ZZZZ");
    assert_eq!(wrapped.postfix, "");
}

#[test]
fn segwit_similarity_uses_a_32_character_alphabet_and_the_hrp_discount() {
    assert!(address_parts("bc1qabcdZZZZ", "bc1qabcdYYYY", "v0_p2wpkh", "mainnet", 2).is_none());
    let parts = address_parts("bc1qabcdZZZZ", "bc1qabcdYYYY", "v0_p2wpkh", "mainnet", 1).unwrap();
    assert_eq!(parts.prefix, "bc1qabcd");
    assert_eq!(parts.infix, "ZZZZ");
    assert_eq!(parts.postfix, "");
    let longer =
        address_parts("bc1qabcdeZZZZ", "bc1qabcdeYYYY", "v0_p2wpkh", "mainnet", 2).unwrap();
    assert_eq!(longer.prefix, "bc1qabcde");
    assert_eq!(longer.infix, "ZZZZ");
    assert!(address_parts("bcrt1qabZZZZ", "bcrt1qabYYYY", "v0_p2wpkh", "regtest", 1).is_none());
    let regtest = address_parts(
        "bcrt1qabcdZZZZ",
        "bcrt1qabcdYYYY",
        "v0_p2wpkh",
        "regtest",
        1,
    )
    .unwrap();
    assert_eq!(regtest.prefix, "bcrt1qabcd");
    assert_eq!(regtest.infix, "ZZZZ");
    assert_eq!(regtest.postfix, "");
    assert!(address_parts("3abcdefZZZZ", "3abcdefYYYY", "p2sh-p2wpkh", "mainnet", 1).is_none());
}

#[test]
fn a_single_differing_character_stays_inside_the_matched_span() {
    let parts = address_parts("1abcXdef", "1abcYdef", "p2pkh", "mainnet", 1).unwrap();
    assert_eq!(parts.prefix, "1abcXdef");
    assert_eq!(parts.infix, "");
    assert_eq!(parts.postfix, "1abcXdef");
}

#[test]
fn displayed_prefix_and_postfix_stop_at_sixteen_characters() {
    let parts = address_parts(
        "1ABCDEFGHIJKLMNOPQRSTZZZZ",
        "1ABCDEFGHIJKLMNOPQRSTYYYY",
        "p2pkh",
        "mainnet",
        1,
    )
    .unwrap();
    assert_eq!(parts.prefix, "1ABCDEFGHIJKLMNO");
    assert_eq!(parts.infix, "PQRSTZZZZ");
    assert_eq!(parts.postfix, "");
}

#[test]
fn identical_addresses_and_incomparable_types_have_no_parts() {
    assert!(address_parts("1abc", "1abc", "p2pkh", "mainnet", 1).is_none());
    assert!(address_parts("1abcdAAAA", "1abcdBBBB", "op_return", "mainnet", 1).is_none());
    assert!(address_parts("1abcdAAAA", "1abcdBBBB", "p2pk", "mainnet", 1).is_none());
}

#[test]
fn the_higher_score_match_supplies_the_screen_parts() {
    let first = output("p2pkh", "1abcdefgXXXXZZZZ");
    let second = output("p2pkh", "1abcdefgYYYYYYYY");
    let third = output("p2pkh", "1abcdefgXXXXYYYY");
    let tx = transaction(
        "best-score",
        &input(&output("p2pkh", "1otherAAAA")),
        &format!("{first},{second},{third}"),
    );
    let alerts = poison_alerts(&tx, "mainnet");
    assert_eq!(alerts.len(), 3);
    assert_eq!(alerts[0].address, "1abcdefgXXXXZZZZ");
    assert_eq!(alerts[0].parts.prefix, "1abcdefgXXXX");
    assert_eq!(alerts[0].parts.infix, "ZZZZ");
    assert_eq!(alerts[0].parts.postfix, "");
    assert_eq!(alerts[1].address, "1abcdefgYYYYYYYY");
    assert_eq!(alerts[1].parts.prefix, "1abcdefg");
    assert_eq!(alerts[1].parts.infix, "YYYYYYYY");
    assert_eq!(alerts[2].address, "1abcdefgXXXXYYYY");
    assert_eq!(alerts[2].parts.prefix, "1abcdefgXXXX");
    assert_eq!(alerts[2].parts.infix, "YYYY");
}

#[test]
fn wrapped_segwit_types_and_repeated_fragments_are_not_poison_alerts() {
    let wrapped = transaction(
        "wrapped-type",
        &input(&output("p2pkh", "1unrelatedAAAA")),
        &format!(
            "{},{}",
            output("p2sh-p2wpkh", "3ABCDEFGHIJKLMNOPQRSTZZZZ"),
            output("p2sh-p2wpkh", "3ABCDEFGHIJKLMNOPQRSTYYYY"),
        ),
    );
    assert!(poison_alerts(&wrapped, "mainnet").is_empty());
    let fake = transaction(
        "fake-repeat",
        &coinbase(),
        &format!(
            "{},{}",
            output("p2pkh", "1qqqqqqqqqqqqAAAA"),
            output("p2pkh", "1qqqqqqqqqqqqBBBB"),
        ),
    );
    assert!(poison_alerts(&fake, "mainnet").is_empty());
    let mixed = transaction(
        "mixed-types",
        &coinbase(),
        &format!(
            "{},{}",
            output("p2pkh", "1DonatePLeaseSenderAddressXVXCmAY"),
            output("p2sh", "1DonatePLease5btcToSenderXXWBoKhB"),
        ),
    );
    assert!(poison_alerts(&mixed, "mainnet").is_empty());
}

fn transaction(txid: &str, vin: &str, vout: &str) -> Transaction {
    let body = format!(
        r#"{{"txid":"{txid}","version":2,"locktime":0,"vin":[{vin}],"vout":[{vout}],"size":180,"weight":720,"sigops":1,"fee":4500}}"#
    );
    serde_json::from_str(&body).unwrap()
}

fn output(kind: &str, address: &str) -> String {
    format!(
        r#"{{"scriptpubkey":"00","scriptpubkey_asm":"OP_0","scriptpubkey_type":"{kind}","scriptpubkey_address":"{address}","value":1}}"#
    )
}

fn input(prevout: &str) -> String {
    format!(
        r#"{{"txid":"prev","vout":0,"prevout":{prevout},"scriptsig":"","scriptsig_asm":"","is_coinbase":false,"sequence":4294967295}}"#
    )
}

fn coinbase() -> String {
    r#"{"txid":"0000000000000000000000000000000000000000000000000000000000000000","vout":0,"prevout":null,"scriptsig":"","scriptsig_asm":"","is_coinbase":true,"sequence":4294967295}"#.to_string()
}
