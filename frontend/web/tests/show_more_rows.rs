//! Show more / Show all on extra inputs and outputs.
//!
//! Angular `transactions-list.component.html` paints the button only when
//! `tx.vin.length > getVinLimit(tx)` (outputs use `getVoutLimit`). The label
//! is `Show all` when `getVinLimit(tx, true) >= tx.vin.length`, and `Show more`
//! otherwise, then `({{ leftover }} remaining)` where leftover is
//! `tx.vin.length - getVinLimit(tx)`.
//!
//! `inputRowLimit` and `rowLimit` default to 12. `showMoreIncrement` is 1000.
//! `@vinLimit` starts unset, so the first limit is `min(12, length)`. When
//! `length - limit <= 5`, `getVinLimit` expands to `length` and the button
//! stays off. The same rule decides the next limit, which is `min(1000, length)`
//! before that tail. Routes stay `GET /tx/:txid`, `GET /block/:hash/txs`, and
//! `GET /address/:address/txs` with no query string and no fragment.

use splora_web::{
    address_requests, address_rows_markup, block_load_plan, block_rows_markup,
    transaction_requests, transaction_row_markup,
};

const TXID: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
const PREV: &str = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
const BLOCK: &str = "cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc";
const ADDRESS: &str = "bc1qshowmore";
const SCRIPT_HEX: &str = "SHOWMORESCRIPTHEXDEAD";
const SCRIPT_ASM: &str = "SHOWMORESCRIPTASMDEAD";
const SCRIPT_SIG: &str = "SHOWMORESCRIPTSIGDEAD";
const SCRIPT_SIG_ASM: &str = "SHOWMORESIGASMDEAD";
const WITNESS: &str = "SHOWMOREWITNESSDEAD";
const INNER_TAPSCRIPT: &str = "SHOWMORETAPSCRIPTDEAD";
const LEAF_VERSION: &str = "SHOWMORELEAFVERSIONDEAD";
const REDEEM_ASM: &str = "OP_DUP OP_HASH160";

fn input(index: usize) -> String {
    let address = format!("vin-{index:04}");
    let kind = if index == 0 { "v1_p2tr" } else { "p2pkh" };
    let redeem = if index == 1 {
        format!(r#","inner_redeemscript_asm":"{REDEEM_ASM}""#)
    } else {
        String::new()
    };
    format!(
        r#"{{"txid":"{PREV}","vout":{index},"prevout":{{"scriptpubkey":"{SCRIPT_HEX}","scriptpubkey_asm":"{SCRIPT_ASM}","scriptpubkey_type":"{kind}","scriptpubkey_address":"{address}","value":1}},"scriptsig":"{SCRIPT_SIG}","scriptsig_asm":"{SCRIPT_SIG_ASM}","witness":["{WITNESS}","{LEAF_VERSION}"],"is_coinbase":false,"sequence":1,"inner_witnessscript_asm":"{INNER_TAPSCRIPT}"{redeem}}}"#
    )
}

fn output(index: usize) -> String {
    let address = format!("vout-{index:04}");
    let kind = match index {
        0 => "fee",
        1 => "anchor",
        _ => "p2pkh",
    };
    format!(
        r#"{{"scriptpubkey":"{SCRIPT_HEX}","scriptpubkey_asm":"{SCRIPT_ASM}","scriptpubkey_type":"{kind}","scriptpubkey_address":"{address}","value":1}}"#
    )
}

fn tx(inputs: usize, outputs: usize) -> String {
    let vin = (0..inputs).map(input).collect::<Vec<_>>().join(",");
    let vout = (0..outputs).map(output).collect::<Vec<_>>().join(",");
    format!(
        r#"{{"txid":"{TXID}","version":1,"locktime":0,"vin":[{vin}],"vout":[{vout}],"size":10,"weight":40,"sigops":0,"fee":1,"status":{{"confirmed":true,"block_height":2,"block_hash":"{BLOCK}","block_time":1700000000}}}}"#
    )
}

fn txs(inputs: usize, outputs: usize) -> String {
    format!("[{}]", tx(inputs, outputs))
}

fn block_json() -> String {
    format!(
        r#"{{"id":"{BLOCK}","height":2,"version":1,"timestamp":1700000000,"tx_count":1,"size":100,"weight":400,"merkle_root":"merkle","previousblockhash":null,"mediantime":1699990000,"nonce":1,"bits":1,"difficulty":1}}"#
    )
}

fn address_anchor(address: &str) -> String {
    format!("<a class=\"address\" href=\"/address/{address}\">{address}</a>")
}

fn assert_one(markup: &str, expected: &str) {
    let count = markup.matches(expected).count();
    assert_eq!(count, 1, "expected one {expected}, found {count}");
}

fn assert_absent(markup: &str, needle: &str) {
    assert!(
        !markup.contains(needle),
        "{needle} leaked into the markup: {markup}"
    );
}

fn assert_button_words_absent(markup: &str) {
    assert_absent(markup, "Show all");
    assert_absent(markup, "Show more");
    assert_absent(markup, "remaining");
}

fn assert_visible_addresses(markup: &str, prefix: &str, shown: usize, total: usize) {
    for index in 0..shown {
        let address = format!("{prefix}-{index:04}");
        assert!(
            markup.contains(&address_anchor(&address)),
            "missing {address} in {markup}"
        );
    }
    for index in shown..total {
        let address = format!("{prefix}-{index:04}");
        assert_absent(markup, &address);
    }
}

fn assert_kept_labels(markup: &str, shown_inputs: usize) {
    // The span and the whole line each contain the words, once per shown input.
    assert_eq!(
        markup.matches("Previous output script").count(),
        shown_inputs * 2,
        "Previous output script count in {markup}"
    );
    assert_one(
        markup,
        "<span class=\"p2tr-tapscript-label\">P2TR tapscript</span>",
    );
    // The span and the whole line each contain the label.
    assert_eq!(
        markup.matches("P2SH redeem script").count(),
        2,
        "P2SH redeem script count in {markup}"
    );
    assert!(
        markup.contains(REDEEM_ASM),
        "P2SH redeem script asm was dropped: {markup}"
    );
    assert_one(markup, "Transaction fee");
    assert_one(markup, ">ANCHOR<");
    for needle in [
        SCRIPT_HEX,
        SCRIPT_ASM,
        SCRIPT_SIG,
        SCRIPT_SIG_ASM,
        WITNESS,
        INNER_TAPSCRIPT,
        LEAF_VERSION,
    ] {
        assert_absent(markup, needle);
    }
}

fn assert_no_query(path: &str) {
    assert!(!path.contains('?'), "{path}");
    assert!(!path.contains('#'), "{path}");
}

#[test]
fn routes_stay_get_without_a_query_or_fragment() {
    let tx_requests = transaction_requests(TXID);
    assert_eq!(tx_requests.len(), 1, "{tx_requests:?}");
    assert_eq!(tx_requests[0].method, "GET");
    assert_eq!(tx_requests[0].path, format!("/tx/{TXID}"));
    assert_no_query(&tx_requests[0].path);

    let block_requests = block_load_plan(BLOCK, None);
    let block_txs = block_requests
        .iter()
        .find(|request| request.path == format!("/block/{BLOCK}/txs"))
        .unwrap_or_else(|| panic!("missing block txs route: {block_requests:?}"));
    assert_eq!(block_txs.method, "GET");
    assert_no_query(&block_txs.path);

    let address_calls = address_requests(ADDRESS);
    let address_txs = address_calls
        .iter()
        .find(|request| request.path == format!("/address/{ADDRESS}/txs"))
        .unwrap_or_else(|| panic!("missing address txs route: {address_calls:?}"));
    assert_eq!(address_txs.method, "GET");
    assert_no_query(&address_txs.path);
}

#[test]
fn twelve_inputs_show_every_input_and_omit_the_button() {
    let markup = transaction_row_markup(&tx(12, 2));
    assert_button_words_absent(&markup);
    assert_visible_addresses(&markup, "vin", 12, 12);
    assert_visible_addresses(&markup, "vout", 2, 2);
    assert_eq!(markup.matches("class=\"party\"").count(), 14, "{markup}");
    assert_kept_labels(&markup, 12);
}

#[test]
fn leftover_of_five_or_fewer_inputs_expands_and_omits_the_button() {
    for inputs in [13_usize, 17] {
        let markup = transaction_row_markup(&tx(inputs, 2));
        assert_button_words_absent(&markup);
        assert_visible_addresses(&markup, "vin", inputs, inputs);
        assert_visible_addresses(&markup, "vout", 2, 2);
        assert_eq!(
            markup.matches("class=\"party\"").count(),
            inputs + 2,
            "{inputs} inputs: {markup}"
        );
        assert_kept_labels(&markup, inputs);
    }
}

#[test]
fn eighteen_inputs_paint_show_all_and_six_remaining() {
    let markup = transaction_row_markup(&tx(18, 2));
    assert_one(&markup, "Show all (6 remaining)");
    assert_absent(&markup, "Show more");
    assert_visible_addresses(&markup, "vin", 12, 18);
    assert_visible_addresses(&markup, "vout", 2, 2);
    assert_eq!(markup.matches("class=\"party\"").count(), 14, "{markup}");
    assert_kept_labels(&markup, 12);
}

#[test]
fn one_thousand_five_inputs_still_say_show_all() {
    let markup = transaction_row_markup(&tx(1005, 2));
    assert_one(&markup, "Show all (993 remaining)");
    assert_absent(&markup, "Show more");
    assert_visible_addresses(&markup, "vin", 12, 1005);
    assert_eq!(markup.matches("class=\"party\"").count(), 14, "{markup}");
}

#[test]
fn one_thousand_six_inputs_paint_show_more() {
    let markup = transaction_row_markup(&tx(1006, 2));
    assert_one(&markup, "Show more (994 remaining)");
    assert_absent(&markup, "Show all");
    assert_visible_addresses(&markup, "vin", 12, 1006);
}

#[test]
fn one_thousand_twelve_inputs_paint_show_more_and_one_thousand_remaining() {
    let markup = transaction_row_markup(&tx(1012, 2));
    assert_one(&markup, "Show more (1000 remaining)");
    assert_absent(&markup, "Show all");
    assert_visible_addresses(&markup, "vin", 12, 1012);
    assert_visible_addresses(&markup, "vout", 2, 2);
    assert_kept_labels(&markup, 12);
}

#[test]
fn twelve_outputs_show_every_output_and_omit_the_button() {
    let markup = transaction_row_markup(&tx(1, 12));
    assert_button_words_absent(&markup);
    assert_visible_addresses(&markup, "vout", 12, 12);
    assert_visible_addresses(&markup, "vin", 1, 1);
    assert_eq!(markup.matches("class=\"party\"").count(), 13, "{markup}");
}

#[test]
fn leftover_of_five_or_fewer_outputs_expands_and_omits_the_button() {
    for outputs in [13_usize, 17] {
        let markup = transaction_row_markup(&tx(1, outputs));
        assert_button_words_absent(&markup);
        assert_visible_addresses(&markup, "vout", outputs, outputs);
        assert_eq!(
            markup.matches("class=\"party\"").count(),
            outputs + 1,
            "{outputs} outputs: {markup}"
        );
    }
}

#[test]
fn eighteen_outputs_paint_show_all_and_six_remaining() {
    let markup = transaction_row_markup(&tx(1, 18));
    assert_one(&markup, "Show all (6 remaining)");
    assert_absent(&markup, "Show more");
    assert_visible_addresses(&markup, "vout", 12, 18);
    assert_visible_addresses(&markup, "vin", 1, 1);
    assert_eq!(markup.matches("class=\"party\"").count(), 13, "{markup}");
    assert_one(&markup, "Transaction fee");
    assert_one(&markup, ">ANCHOR<");
}

#[test]
fn one_thousand_twelve_outputs_paint_show_more_and_one_thousand_remaining() {
    let markup = transaction_row_markup(&tx(1, 1012));
    assert_one(&markup, "Show more (1000 remaining)");
    assert_absent(&markup, "Show all");
    assert_visible_addresses(&markup, "vout", 12, 1012);
    assert_visible_addresses(&markup, "vin", 1, 1);
}

#[test]
fn inputs_and_outputs_each_get_their_own_button() {
    let markup = transaction_row_markup(&tx(18, 1012));
    assert_one(&markup, "Show all (6 remaining)");
    assert_one(&markup, "Show more (1000 remaining)");
    let inputs_at = markup.find("Show all (6 remaining)").expect("input button");
    let outputs_at = markup
        .find("Show more (1000 remaining)")
        .expect("output button");
    assert!(
        inputs_at < outputs_at,
        "output button was painted before the input button"
    );
    assert_visible_addresses(&markup, "vin", 12, 18);
    assert_visible_addresses(&markup, "vout", 12, 1012);
}

#[test]
fn block_and_address_lists_use_the_same_button() {
    let body = txs(18, 1012);
    let block = block_rows_markup(&block_json(), &body);
    let address = address_rows_markup("", &body, "");
    for markup in [&block, &address] {
        assert_one(markup, "Show all (6 remaining)");
        assert_one(markup, "Show more (1000 remaining)");
        assert_visible_addresses(markup, "vin", 12, 18);
        assert_visible_addresses(markup, "vout", 12, 1012);
    }
}

/// One input whose previous txid is `prev` and whose previous output index is `index`.
/// `prevout` stays null, matching the show-more input that GPUI paints as an outpoint line.
fn outpoint_input(index: u32) -> String {
    format!(
        r#"{{"txid":"prev","vout":{index},"prevout":null,"scriptsig":"","scriptsig_asm":"","is_coinbase":false,"sequence":1}}"#
    )
}

/// Index 0 stays in the list and paints `input outpoint prev:0`.
/// Index 11 paints `input outpoint prev:11`.
fn prev_11_body() -> String {
    let vin = format!("{},{}", outpoint_input(0), outpoint_input(11));
    format!(
        r#"{{"txid":"{TXID}","version":1,"locktime":0,"vin":[{vin}],"vout":[{{"scriptpubkey":"","scriptpubkey_asm":"","scriptpubkey_type":"","value":1}}],"size":10,"weight":40,"sigops":0,"fee":1,"status":{{"confirmed":false}}}}"#
    )
}

/// Whole line `input outpoint prev:11` from `GET /tx/:txid`.
///
/// GPUI `transaction_screen_paints_show_more_inputs` requires one line whose
/// entire text is `input outpoint prev:11`. The previous output index is 11
/// and the previous txid is `prev`. The index 0 input stays in the list and
/// paints `input outpoint prev:0` as its own line. The txid anchor and the
/// vout span stay. This calls `transaction_row_markup`.
#[test]
fn input_outpoint_prev_11_is_its_own_line() {
    let requests = transaction_requests(TXID);
    assert_eq!(requests.len(), 1, "{requests:?}");
    assert_eq!(requests[0].method, "GET");
    assert_eq!(requests[0].path, format!("/tx/{TXID}"));
    assert_no_query(&requests[0].path);

    let markup = transaction_row_markup(&prev_11_body());
    assert!(
        markup.lines().any(|line| line == "input outpoint prev:11"),
        "no line equals input outpoint prev:11: {markup}"
    );
    assert_eq!(
        markup
            .lines()
            .filter(|line| *line == "input outpoint prev:11")
            .count(),
        1,
        "input outpoint prev:11 was dropped or repeated: {markup}"
    );
    assert!(
        markup.contains("<a href=\"/tx/prev\">prev</a>"),
        "the previous txid anchor was dropped: {markup}"
    );
    assert!(
        markup.contains("<span class=\"vout\">11</span>"),
        "vout 11 was dropped: {markup}"
    );
    assert!(
        markup.contains("<span class=\"vout\">0</span>"),
        "the index 0 input was dropped: {markup}"
    );
    assert!(
        markup.lines().any(|line| line == "input outpoint prev:0"),
        "no line equals input outpoint prev:0: {markup}"
    );
    assert_eq!(
        markup
            .lines()
            .filter(|line| *line == "input outpoint prev:0")
            .count(),
        1,
        "input outpoint prev:0 was dropped or repeated: {markup}"
    );
    assert!(
        !markup
            .lines()
            .any(|line| line == "output script type op_return"),
        "output script type op_return was painted: {markup}"
    );
    assert!(
        !markup
            .lines()
            .any(|line| line == "output address lq1qqout1 value 9"),
        "output address lq1qqout1 value 9 was painted: {markup}"
    );
}

/// Whole line `input outpoint prev:0` from `GET /tx/:txid`.
///
/// Same fixture shape as `prev_11_body`, but the previous output index is 0,
/// not 11. The previous txid is `prev`. This sentence is distinct from
/// `input outpoint prev:11`. This calls `transaction_row_markup`.
fn prev_0_body() -> String {
    let vin = outpoint_input(0);
    format!(
        r#"{{"txid":"{TXID}","version":1,"locktime":0,"vin":[{vin}],"vout":[{{"scriptpubkey":"","scriptpubkey_asm":"","scriptpubkey_type":"","value":1}}],"size":10,"weight":40,"sigops":0,"fee":1,"status":{{"confirmed":false}}}}"#
    )
}

#[test]
fn input_outpoint_prev_0_is_its_own_line() {
    let requests = transaction_requests(TXID);
    assert_eq!(requests.len(), 1, "{requests:?}");
    assert_eq!(requests[0].method, "GET");
    assert_eq!(requests[0].path, format!("/tx/{TXID}"));
    assert_no_query(&requests[0].path);

    let markup = transaction_row_markup(&prev_0_body());
    assert!(
        markup.lines().any(|line| line == "input outpoint prev:0"),
        "no line equals input outpoint prev:0: {markup}"
    );
    assert_eq!(
        markup
            .lines()
            .filter(|line| *line == "input outpoint prev:0")
            .count(),
        1,
        "input outpoint prev:0 was dropped or repeated: {markup}"
    );
    assert!(
        markup.contains("<a href=\"/tx/prev\">prev</a>"),
        "the previous txid anchor was dropped: {markup}"
    );
    assert!(
        markup.contains("<span class=\"vout\">0</span>"),
        "vout 0 was dropped: {markup}"
    );
    assert!(
        !markup.lines().any(|line| line == "input outpoint prev:11"),
        "input outpoint prev:11 was painted on the index 0 fixture: {markup}"
    );
}
