//! Block bits hex beside the decimal in the details panel.
//!
//! Angular `decimal2hex` defaults to 4 bytes: `0x` plus 8 lowercase hex
//! digits. Explicit 0 is `0x00000000`. A missing bits key or JSON null is
//! `None` and does not paint the Bits label or the hex cell.

use splora_web::{block_details_markup, block_load_plan};

const ID: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";

fn block_json(bits: &str) -> String {
    format!(
        r#"{{"id":"{ID}","height":2,"version":1,"timestamp":1700000000,"tx_count":1,"size":100,"weight":400,"merkle_root":"merkle-root","previousblockhash":null,"mediantime":1699990000,"nonce":9,"bits":{bits},"difficulty":1}}"#
    )
}

fn missing_bits_json() -> String {
    format!(
        r#"{{"id":"{ID}","height":2,"version":1,"timestamp":1700000000,"tx_count":1,"size":100,"weight":400,"merkle_root":"merkle-root","previousblockhash":null,"mediantime":1699990000,"nonce":9,"difficulty":1}}"#
    )
}

fn null_bits_json() -> String {
    format!(
        r#"{{"id":"{ID}","height":2,"version":1,"timestamp":1700000000,"tx_count":1,"size":100,"weight":400,"merkle_root":"merkle-root","previousblockhash":null,"mediantime":1699990000,"nonce":9,"bits":null,"difficulty":1}}"#
    )
}

fn hex_cell(digits: &str) -> String {
    format!("<td class=\"bits-hex\">0x{digits}</td>")
}

fn assert_one(markup: &str, expected: &str) {
    assert_eq!(
        markup.matches(expected).count(),
        1,
        "expected one {expected} in {markup}"
    );
}

fn assert_no_bits(markup: &str) {
    assert!(
        !markup.contains("bits-hex"),
        "painted a bits hex cell: {markup}"
    );
    assert!(!markup.contains("Bits"), "painted the Bits label: {markup}");
    assert!(
        !markup.contains("0x00000000"),
        "painted zero hex for absent bits: {markup}"
    );
}

#[test]
fn bits_386547056_paints_hex_beside_the_decimal() {
    let open = block_details_markup(&block_json("386547056"), true);
    assert_one(&open, &hex_cell("170a3d70"));
    assert!(open.contains("386547056"), "{open}");
    assert!(open.contains("Bits"), "{open}");
    assert!(
        open.contains(
            "<tr><td>Bits</td><td>386547056</td><td class=\"bits-hex\">0x170a3d70</td></tr>"
        ),
        "{open}"
    );
}

#[test]
fn bits_zero_paints_eight_zero_digits_and_the_decimal() {
    let open = block_details_markup(&block_json("0"), true);
    assert_one(&open, &hex_cell("00000000"));
    assert!(open.contains("Bits"), "{open}");
    assert!(open.contains(">0<"), "{open}");
    assert!(
        open.contains("<tr><td>Bits</td><td>0</td><td class=\"bits-hex\">0x00000000</td></tr>"),
        "{open}"
    );
    assert!(
        open.lines().any(|line| line == "bits 0 0x00000000"),
        "bits 0 0x00000000 is absent: {open}"
    );
}

#[test]
fn bits_one_paints_padded_one() {
    let open = block_details_markup(&block_json("1"), true);
    assert_one(&open, &hex_cell("00000001"));
    assert!(open.contains("<tr><td>Bits</td><td>1</td>"), "{open}");
}

#[test]
fn missing_bits_key_does_not_paint_hex_or_the_label() {
    let open = block_details_markup(&missing_bits_json(), true);
    assert_no_bits(&open);
}

#[test]
fn json_null_bits_does_not_paint_hex_or_the_label() {
    // `Block.bits` is `Option<u32>`. Serde maps JSON null to None, same as a missing key.
    let open = block_details_markup(&null_bits_json(), true);
    assert_no_bits(&open);
}

/// Whole line `bits 1 0x00000001` from no-query `GET /block/:hash` when details are open.
///
/// The Bits cell and the eight-digit hex cell stay. Closed details omit the line.
/// This is not the version line and not Difficulty.
#[test]
fn open_block_details_paint_bits_1_hex() {
    const LINE: &str = "bits 1 0x00000001";
    let id = "bitshex1";
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

    let open = block_details_markup(&block_json("1"), true);
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
        open.contains("<tr><td>Bits</td><td>1</td><td class=\"bits-hex\">0x00000001</td></tr>"),
        "{open}"
    );

    let closed = block_details_markup(&block_json("1"), false);
    assert!(
        !closed.lines().any(|line| line == LINE),
        "closed details painted {LINE}: {closed}"
    );
    assert!(
        closed.contains("<tr><td>Bits</td><td>1</td><td class=\"bits-hex\">0x00000001</td></tr>"),
        "{closed}"
    );
}
