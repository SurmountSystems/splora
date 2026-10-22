//! Block nonce hex beside the decimal in the details panel.
//!
//! Angular `decimal2hex` defaults to 4 bytes: `0x` plus 8 lowercase hex
//! digits. Explicit 0 is `0x00000000`. A missing nonce key or JSON null is
//! `None` and does not paint the Nonce label or the hex cell. The decimal
//! cell stays when nonce is `Some`, because `block_transactions` asserts
//! `>9<` for nonce 9.

use splora_web::{block_details_markup, block_load_plan};

const ID: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";

fn block_json(nonce: &str) -> String {
    format!(
        r#"{{"id":"{ID}","height":2,"version":1,"timestamp":1700000000,"tx_count":1,"size":100,"weight":400,"merkle_root":"merkle-root","previousblockhash":null,"mediantime":1699990000,"nonce":{nonce},"bits":386547056,"difficulty":1}}"#
    )
}

fn missing_nonce_json() -> String {
    format!(
        r#"{{"id":"{ID}","height":2,"version":1,"timestamp":1700000000,"tx_count":1,"size":100,"weight":400,"merkle_root":"merkle-root","previousblockhash":null,"mediantime":1699990000,"bits":386547056,"difficulty":1}}"#
    )
}

fn null_nonce_json() -> String {
    format!(
        r#"{{"id":"{ID}","height":2,"version":1,"timestamp":1700000000,"tx_count":1,"size":100,"weight":400,"merkle_root":"merkle-root","previousblockhash":null,"mediantime":1699990000,"nonce":null,"bits":386547056,"difficulty":1}}"#
    )
}

fn hex_cell(digits: &str) -> String {
    format!("<td class=\"nonce-hex\">0x{digits}</td>")
}

fn assert_one(markup: &str, expected: &str) {
    assert_eq!(
        markup.matches(expected).count(),
        1,
        "expected one {expected} in {markup}"
    );
}

fn assert_no_nonce(markup: &str) {
    assert!(
        !markup.contains("nonce-hex"),
        "painted a nonce hex cell: {markup}"
    );
    assert!(
        !markup.contains("Nonce"),
        "painted the Nonce label: {markup}"
    );
    assert!(
        !markup.contains("0x00000000"),
        "painted zero hex for absent nonce: {markup}"
    );
}

#[test]
fn nonce_nine_paints_hex_beside_the_decimal() {
    let open = block_details_markup(&block_json("9"), true);
    assert_one(&open, &hex_cell("00000009"));
    assert!(open.contains("Nonce"), "{open}");
    assert!(open.contains(">9<"), "{open}");
    assert!(
        open.contains("<tr><td>Nonce</td><td>9</td><td class=\"nonce-hex\">0x00000009</td></tr>"),
        "{open}"
    );
}

#[test]
fn nonce_zero_paints_eight_zero_digits_and_the_decimal() {
    let open = block_details_markup(&block_json("0"), true);
    assert_one(&open, &hex_cell("00000000"));
    assert!(open.contains("Nonce"), "{open}");
    assert!(open.contains(">0<"), "{open}");
    assert!(
        open.contains("<tr><td>Nonce</td><td>0</td><td class=\"nonce-hex\">0x00000000</td></tr>"),
        "{open}"
    );
}

#[test]
fn nonce_one_paints_padded_one() {
    let open = block_details_markup(&block_json("1"), true);
    assert_one(&open, &hex_cell("00000001"));
    assert!(open.contains("<tr><td>Nonce</td><td>1</td>"), "{open}");
}

#[test]
fn missing_nonce_key_does_not_paint_hex_or_the_label() {
    let open = block_details_markup(&missing_nonce_json(), true);
    assert_no_nonce(&open);
}

#[test]
fn json_null_nonce_does_not_paint_hex_or_the_label() {
    // `Block.nonce` is `Option<u32>`. `#[serde(default)]` maps JSON null to None, same as a missing key.
    let open = block_details_markup(&null_nonce_json(), true);
    assert_no_nonce(&open);
}

/// Whole line `nonce 9 0x00000009` from no-query `GET /block/:hash` when details are open.
///
/// The Nonce cell and the eight-digit hex cell stay. Closed details omit the line.
/// This is not the version line and not the bits line.
#[test]
fn open_block_details_paint_nonce_9_hex() {
    const LINE: &str = "nonce 9 0x00000009";
    let id = "noncehex9";
    assert!(!id.bytes().all(|byte| byte.is_ascii_digit()), "{id}");
    assert!(!id.contains('?') && !id.contains('#'), "{id}");
    let plan = block_load_plan(id, None);
    assert!(
        plan.iter()
            .any(|req| req.method == "GET" && req.path == format!("/block/{id}")),
        "{plan:?}"
    );
    assert!(
        plan.iter()
            .all(|req| !req.path.contains('?') && !req.path.contains('#')),
        "{plan:?}"
    );

    let open = block_details_markup(&block_json("9"), true);
    assert!(
        open.lines().any(|line| line == LINE),
        "{LINE} is absent: {open}"
    );
    assert_eq!(
        open.lines().filter(|line| *line == LINE).count(),
        1,
        "{LINE} was not one line: {open}"
    );
    assert!(
        open.contains("<tr><td>Nonce</td><td>9</td><td class=\"nonce-hex\">0x00000009</td></tr>"),
        "{open}"
    );

    let closed = block_details_markup(&block_json("9"), false);
    assert!(
        !closed.lines().any(|line| line == LINE),
        "closed details painted {LINE}: {closed}"
    );
    assert!(
        closed.contains("<tr><td>Nonce</td><td>9</td><td class=\"nonce-hex\">0x00000009</td></tr>"),
        "{closed}"
    );
}

/// Whole line `Nonce 0x00000009` from no-query `GET /block/:hash` when details are open.
///
/// Angular `decimal2hex` is the value. The decimal stays in the Nonce cell and on
/// the raw `nonce 9 0x00000009` line. Closed details omit this line.
#[test]
fn open_block_details_paint_nonce_label_hex() {
    const LINE: &str = "Nonce 0x00000009";
    const RAW: &str = "nonce 9 0x00000009";
    let id = "noncelabel9";
    assert!(!id.bytes().all(|byte| byte.is_ascii_digit()), "{id}");
    assert!(!id.contains('?') && !id.contains('#'), "{id}");
    let plan = block_load_plan(id, None);
    assert!(
        plan.iter()
            .any(|req| req.method == "GET" && req.path == format!("/block/{id}")),
        "{plan:?}"
    );
    assert!(
        plan.iter()
            .all(|req| !req.path.contains('?') && !req.path.contains('#')),
        "{plan:?}"
    );

    let open = block_details_markup(&block_json("9"), true);
    assert!(
        open.lines().any(|line| line == LINE),
        "{LINE} is absent: {open}"
    );
    assert_eq!(
        open.lines().filter(|line| *line == LINE).count(),
        1,
        "{LINE} was not one line: {open}"
    );
    assert!(
        open.lines().any(|line| line == RAW),
        "raw {RAW} was dropped: {open}"
    );
    assert!(
        open.contains("<tr><td>Nonce</td><td>9</td><td class=\"nonce-hex\">0x00000009</td></tr>"),
        "{open}"
    );

    let closed = block_details_markup(&block_json("9"), false);
    assert!(
        !closed.lines().any(|line| line == LINE),
        "closed details painted {LINE}: {closed}"
    );
    assert!(
        closed.contains("<tr><td>Nonce</td><td>9</td><td class=\"nonce-hex\">0x00000009</td></tr>"),
        "{closed}"
    );
}
