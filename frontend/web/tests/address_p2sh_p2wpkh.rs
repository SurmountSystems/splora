//! Address-page badge for a mainnet pay-to-script-hash wrapped pay-to-witness-pubkey-hash address.
//!
//! After a successful no-query `GET /address/:script/txs`, a mainnet address
//! whose first character is `3` paints `P2SH-P2WPKH` when a vin spending that
//! address has a non-empty witness and a scriptsig that starts with `160014`.
//! `detectAddressType` starts that address as `p2sh`. `processInputs` changes
//! it to `p2sh-p2wpkh`. Angular has no dedicated case. The default arm
//! uppercases the type, so the span text is `P2SH-P2WPKH`. The Type row
//! appears only after the address load succeeds. The example spend is
//! `transactions-random.json`: address `375PJxsKRtAq4WoS6u82jvgZW94R8Wx3iH`,
//! scriptsig `1600149b27f072e4b972927c445d1946162a550b0914d8`. A scriptsig
//! that starts with `220020` is the other wrapped kind. That spend paints
//! `P2SH-P2WSH` once. It does not paint `P2SH` or `P2SH-P2WPKH`. The
//! scriptsig and the witness stay off the page.

use splora_web::{address_requests, address_rows_markup, indexer_failure_text};

/// `transactions-random.json` prevout for the `160014` scriptsig.
const P2SH_ADDR: &str = "375PJxsKRtAq4WoS6u82jvgZW94R8Wx3iH";
/// Same file, a different mainnet address whose first character is `3`.
const OTHER_P2SH: &str = "3L4YUynB4X44rJBY9CmiLMN8Wjti49JCYB";
/// Same file, the `220020` wrapped spend.
const P2SH_P2WSH_ADDR: &str = "3GcrZrbUuvE4UtUdSbKTXcRnTqmfMdyMAC";
const SCRIPTSIG_P2WPKH: &str = "1600149b27f072e4b972927c445d1946162a550b0914d8";
const SCRIPTSIG_P2WSH: &str =
    "220020a18160de7291554f349c7d5cbee4ab97fb542e94cf302ce8d7e9747e4188ca75";
const WITNESS_0: &str = "3040021c23902a01d4c5cff2c33c8bdb778a5aadea78a9a0d6d4db60aaa0fba1022069237d9dbf2db8cff9c260ba71250493682d01a746f4a45c5c7ea386e56d2bc902";
const WITNESS_1: &str = "0240187acd3e2fd3d8e1acffefa85907b6550730c24f78dfd3301c829fc4daf3cc";
const P2PKH_ADDR: &str = "1stAprEZapjCYGUACUcXohQqSHF9MU5Kj";
const P2WPKH_ADDR: &str = "bc1q8yj0herd4r4yxszw3nkfvt53433thk0f5qst4g";
const P2WSH_ADDR: &str = "bc1qrp33g0q5c5txsp9arysrx4k6zdkfs4nce4xj0gdcccefvpysxf3qccfmv3";
const P2TR_ADDR: &str = "bc1pahzjr2vnfqnqwl0h8jps2aa4zqe29wqxn2nagvd5a2z982k7ttdq8ry370";
const BADGE: &str = "<span class=\"address-type-label\">Type</span><span class=\"badge bg-primary\"><span>P2SH-P2WPKH</span></span>";
const PLAIN_P2SH_BADGE: &str = "<span class=\"badge bg-primary\"><span>P2SH</span></span>";
const P2SH_P2WSH_BADGE: &str = "<span class=\"badge bg-primary\"><span>P2SH-P2WSH</span></span>";
const LOAD_ERROR: &str = "Error loading address data.";
const INDEXER_BODY: &str = "Invalid Bitcoin address";

fn address_body(address: &str) -> String {
    format!(
        r#"{{"address":"{address}","chain_stats":{{"tx_count":1,"funded_txo_count":1,"spent_txo_count":0,"funded_txo_sum":1000,"spent_txo_sum":0}},"mempool_stats":{{"tx_count":0,"funded_txo_count":0,"spent_txo_count":0,"funded_txo_sum":0,"spent_txo_sum":0}}}}"#
    )
}

fn base58_char(ch: char) -> bool {
    matches!(
        ch,
        'a'..='k' | 'm'..='z' | 'A'..='H' | 'J'..='N' | 'P'..='Z' | '1'..='9'
    )
}

/// `detectAddressType` on mainnet: first character `3`, then `base58Regex`.
fn mainnet_p2sh_shape(address: &str) -> bool {
    let mut chars = address.chars();
    if chars.next() != Some('3') {
        return false;
    }
    let rest: Vec<char> = chars.collect();
    (26..=34).contains(&rest.len()) && rest.iter().copied().all(base58_char)
}

fn bech32_lw(ch: char) -> bool {
    matches!(ch, 'a' | 'c'..='h' | 'j'..='n' | 'p'..='z' | '0'..='9')
}

fn mainnet_p2pkh_shape(address: &str) -> bool {
    let mut chars = address.chars();
    if chars.next() != Some('1') {
        return false;
    }
    let rest: Vec<char> = chars.collect();
    (26..=34).contains(&rest.len()) && rest.iter().copied().all(base58_char)
}

fn mainnet_bc1_shape(address: &str, second: char, data_len: usize) -> bool {
    let Some(suffix) = address.strip_prefix("bc1") else {
        return false;
    };
    let mut chars = suffix.chars();
    if chars.next() != Some(second) {
        return false;
    }
    let rest: Vec<char> = chars.collect();
    rest.len() == data_len && rest.iter().copied().all(bech32_lw)
}

fn witness_pair() -> String {
    format!(r#"["{WITNESS_0}","{WITNESS_1}"]"#)
}

fn vin_json(address: &str, script_type: &str, scriptsig: &str, witness: Option<&str>) -> String {
    let asm = if scriptsig.starts_with("160014") {
        "OP_PUSHBYTES_22 00149b27f072e4b972927c445d1946162a550b0914d8"
    } else if scriptsig.starts_with("220020") {
        "OP_PUSHBYTES_34 0020a18160de7291554f349c7d5cbee4ab97fb542e94cf302ce8d7e9747e4188ca75"
    } else {
        ""
    };
    let witness_field = match witness {
        Some(items) => format!(r#","witness":{items}"#),
        None => String::new(),
    };
    format!(
        r#"{{"txid":"45e1cb33599acb071810ccc801b71bd7610865f5b899492946ab1bfbcb61cad6","vout":2,"prevout":{{"scriptpubkey":"a9143b13a1f71c20c799d86bb624b3898c826d6c82da87","scriptpubkey_asm":"OP_HASH160 OP_PUSHBYTES_20 3b13a1f71c20c799d86bb624b3898c826d6c82da OP_EQUAL","scriptpubkey_type":"{script_type}","scriptpubkey_address":"{address}","value":5139}},"scriptsig":"{scriptsig}","scriptsig_asm":"{asm}","is_coinbase":false,"sequence":141{witness_field}}}"#
    )
}

fn txs(vins: &[String]) -> String {
    let vins = vins.join(",");
    format!(
        r#"[{{"txid":"cb9b47ac04023b29fb633a8ef04af351ac9fd74c57c9a2163f683516274767e3","version":1,"locktime":0,"vin":[{vins}],"vout":[],"size":10,"weight":40,"sigops":0,"fee":1}}]"#
    )
}

/// A vin that does not spend `P2SH_ADDR` comes first. The matching spend is second.
fn matching_txs() -> String {
    let other = vin_json(OTHER_P2SH, "p2sh", "", Some("[]"));
    let spend = vin_json(P2SH_ADDR, "p2sh", SCRIPTSIG_P2WPKH, Some(&witness_pair()));
    txs(&[other, spend])
}

fn page(address: &str, txs_json: &str) -> String {
    address_rows_markup(&address_body(address), txs_json, "")
}

fn assert_no_wrapped_badge(markup: &str) {
    assert!(
        !markup.contains("P2SH-P2WPKH"),
        "P2SH-P2WPKH was painted: {markup}"
    );
    assert!(
        !markup.contains(PLAIN_P2SH_BADGE),
        "a plain P2SH badge was painted: {markup}"
    );
    assert!(
        !markup.contains(P2SH_P2WSH_BADGE),
        "P2SH-P2WSH was painted: {markup}"
    );
    assert!(!markup.contains(">P2SH-P2WSH<"), "{markup}");
}

/// No spend rewrote the type, so the plain badge is required once.
/// Previous-output type `P2SH` is a different span and stays.
fn assert_plain_p2sh_once(markup: &str) {
    assert_eq!(
        markup.matches(PLAIN_P2SH_BADGE).count(),
        1,
        "plain P2SH badge count: {markup}"
    );
    let previous = markup
        .matches("<span class=\"previous-output-type\">P2SH</span>")
        .count();
    let output = markup
        .matches("<span class=\"output-type\">P2SH</span>")
        .count();
    assert_eq!(
        markup.matches(">P2SH<").count() - previous - output,
        1,
        "plain P2SH badge text count: {markup}"
    );
    assert!(
        !markup.contains("P2SH-P2WPKH"),
        "P2SH-P2WPKH was painted: {markup}"
    );
    assert!(
        !markup.contains(P2SH_P2WSH_BADGE),
        "P2SH-P2WSH was painted: {markup}"
    );
    assert!(!markup.contains(">P2SH-P2WSH<"), "{markup}");
}

fn assert_spend_stays_off_the_page(markup: &str) {
    assert!(
        !markup.contains(SCRIPTSIG_P2WPKH),
        "the scriptsig was printed: {markup}"
    );
    assert!(
        !markup.contains(SCRIPTSIG_P2WSH),
        "the other scriptsig was printed: {markup}"
    );
    assert!(
        !markup.contains(WITNESS_0),
        "the witness was printed: {markup}"
    );
    assert!(
        !markup.contains(WITNESS_1),
        "the witness was printed: {markup}"
    );
}

#[test]
fn address_page_paints_p2sh_p2wpkh_after_no_query_txs() {
    assert!(
        mainnet_p2sh_shape(P2SH_ADDR),
        "fixture is not a mainnet address that detectAddressType classifies as p2sh"
    );
    assert!(
        SCRIPTSIG_P2WPKH.starts_with("160014"),
        "fixture scriptsig does not start with 160014"
    );
    let body = address_body(P2SH_ADDR);
    assert!(
        !body.contains("\"type\""),
        "address JSON must not carry a type field: {body}"
    );
    let calls = address_requests(P2SH_ADDR);
    assert!(
        calls.iter().any(|call| {
            call.method == "GET"
                && call.path == format!("/address/{P2SH_ADDR}/txs")
                && !call.path.contains('?')
                && !call.path.contains('#')
        }),
        "no-query GET /address/:script/txs is missing: {calls:?}"
    );
    assert!(
        calls.iter().all(|call| call.method == "GET"
            && !call.path.contains('?')
            && !call.path.contains('#')),
        "address read added a query or a hash: {calls:?}"
    );

    let txs_json = matching_txs();
    let markup = page(P2SH_ADDR, &txs_json);
    assert!(markup.contains(BADGE), "P2SH-P2WPKH is absent: {markup}");
    assert_eq!(
        markup.matches(BADGE).count(),
        1,
        "P2SH-P2WPKH badge count: {markup}"
    );
    assert_eq!(
        markup.matches(">P2SH-P2WPKH<").count(),
        1,
        "P2SH-P2WPKH badge text count: {markup}"
    );
    assert_no_wrapped_badge_except_the_match(&markup);
    assert!(!markup.contains("<span>P2PKH</span>"), "{markup}");
    assert!(!markup.contains("<span>P2WPKH</span>"), "{markup}");
    assert!(!markup.contains("<span>P2WSH</span>"), "{markup}");
    assert!(!markup.contains("<span>P2TR</span>"), "{markup}");
    assert!(!markup.contains(LOAD_ERROR), "{markup}");
    assert_spend_stays_off_the_page(&markup);
}

fn assert_no_wrapped_badge_except_the_match(markup: &str) {
    assert!(
        !markup.contains(PLAIN_P2SH_BADGE),
        "a plain P2SH badge was painted: {markup}"
    );
    assert!(
        !markup.contains(P2SH_P2WSH_BADGE),
        "P2SH-P2WSH was painted: {markup}"
    );
    assert!(!markup.contains(">P2SH-P2WSH<"), "{markup}");
}

#[test]
fn a_mainnet_3_address_without_a_matching_spend_does_not_paint_p2sh_p2wpkh() {
    assert!(mainnet_p2sh_shape(P2SH_ADDR));
    assert!(mainnet_p2sh_shape(OTHER_P2SH));
    assert_plain_p2sh_once(&page(P2SH_ADDR, ""));
    assert_plain_p2sh_once(&page(P2SH_ADDR, "[]"));
    assert_plain_p2sh_once(&page(OTHER_P2SH, &matching_txs()));

    let empty_witness = txs(&[vin_json(P2SH_ADDR, "p2sh", SCRIPTSIG_P2WPKH, Some("[]"))]);
    assert_plain_p2sh_once(&page(P2SH_ADDR, &empty_witness));
    assert_spend_stays_off_the_page(&page(P2SH_ADDR, &empty_witness));

    let omitted_witness = txs(&[vin_json(P2SH_ADDR, "p2sh", SCRIPTSIG_P2WPKH, None)]);
    assert_plain_p2sh_once(&page(P2SH_ADDR, &omitted_witness));

    let other_prefix = txs(&[vin_json(
        P2SH_ADDR,
        "p2sh",
        SCRIPTSIG_P2WSH,
        Some(&witness_pair()),
    )]);
    let other_markup = page(P2SH_ADDR, &other_prefix);
    assert_eq!(
        other_markup.matches(">P2SH-P2WSH<").count(),
        1,
        "P2SH-P2WSH is absent: {other_markup}"
    );
    assert!(
        !other_markup.contains(PLAIN_P2SH_BADGE),
        "a plain P2SH badge was painted: {other_markup}"
    );
    assert!(
        !other_markup.contains("P2SH-P2WPKH"),
        "P2SH-P2WPKH was painted on the 220020 spend: {other_markup}"
    );
    assert_spend_stays_off_the_page(&other_markup);

    let wrapped_sh = txs(&[vin_json(
        P2SH_P2WSH_ADDR,
        "p2sh",
        SCRIPTSIG_P2WSH,
        Some(&witness_pair()),
    )]);
    assert!(mainnet_p2sh_shape(P2SH_P2WSH_ADDR));
    let wrapped_markup = page(P2SH_P2WSH_ADDR, &wrapped_sh);
    assert_eq!(
        wrapped_markup.matches(">P2SH-P2WSH<").count(),
        1,
        "P2SH-P2WSH is absent: {wrapped_markup}"
    );
    assert!(
        !wrapped_markup.contains(PLAIN_P2SH_BADGE),
        "a plain P2SH badge was painted: {wrapped_markup}"
    );
    assert!(
        !wrapped_markup.contains("P2SH-P2WPKH"),
        "P2SH-P2WPKH was painted on the 220020 spend: {wrapped_markup}"
    );

    let spends_someone_else = txs(&[vin_json(
        OTHER_P2SH,
        "p2sh",
        SCRIPTSIG_P2WPKH,
        Some(&witness_pair()),
    )]);
    assert_plain_p2sh_once(&page(P2SH_ADDR, &spends_someone_else));
    assert_spend_stays_off_the_page(&page(P2SH_ADDR, &spends_someone_else));
}

#[test]
fn other_address_shapes_do_not_show_p2sh_p2wpkh() {
    assert!(mainnet_p2pkh_shape(P2PKH_ADDR));
    assert!(mainnet_bc1_shape(P2WPKH_ADDR, 'q', 38));
    assert!(mainnet_bc1_shape(P2WSH_ADDR, 'q', 58));
    assert!(mainnet_bc1_shape(P2TR_ADDR, 'p', 58));
    let txs_json = matching_txs();

    let p2pkh = page(P2PKH_ADDR, &txs_json);
    assert!(
        p2pkh.contains("<span class=\"badge bg-primary\"><span>P2PKH</span></span>"),
        "P2PKH badge is absent: {p2pkh}"
    );
    assert_no_wrapped_badge(&p2pkh);

    let p2wpkh = page(P2WPKH_ADDR, &txs_json);
    assert!(
        p2wpkh.contains("<span class=\"badge bg-primary\"><span>P2WPKH</span></span>"),
        "P2WPKH badge is absent: {p2wpkh}"
    );
    assert_no_wrapped_badge(&p2wpkh);

    let p2wsh = page(P2WSH_ADDR, &txs_json);
    assert!(
        p2wsh.contains("<span class=\"badge bg-primary\"><span>P2WSH</span></span>"),
        "P2WSH badge is absent: {p2wsh}"
    );
    assert_no_wrapped_badge(&p2wsh);

    let p2tr = page(P2TR_ADDR, &txs_json);
    assert!(
        p2tr.contains("<span class=\"badge bg-primary\"><span>P2TR</span></span>"),
        "P2TR badge is absent: {p2tr}"
    );
    assert_no_wrapped_badge(&p2tr);
    assert!(!p2tr.contains(">P2PKH<"), "{p2tr}");
}

#[test]
fn a_failed_address_load_keeps_the_error_and_skips_the_badge() {
    let path = format!("/address/{P2SH_ADDR}");
    let failed = indexer_failure_text("GET", &path, INDEXER_BODY);
    assert!(failed.contains(INDEXER_BODY), "{failed}");
    assert!(
        !failed.contains(LOAD_ERROR),
        "the indexer body became the span: {failed}"
    );
    let markup = address_rows_markup(&failed, &matching_txs(), "");
    assert!(
        markup.contains(LOAD_ERROR),
        "Error loading address data. is absent: {markup}"
    );
    assert!(
        markup.contains(INDEXER_BODY),
        "Invalid Bitcoin address was replaced or dropped: {markup}"
    );
    assert_no_wrapped_badge(&markup);
}
