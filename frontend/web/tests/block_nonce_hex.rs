//! Block nonce hex beside the decimal in the details panel.
//!
//! Angular `decimal2hex` defaults to 4 bytes: `0x` plus 8 lowercase hex
//! digits. Explicit 0 is `0x00000000`. A missing nonce key or JSON null is
//! `None` and does not paint the Nonce label or the hex cell. The decimal
//! cell stays when nonce is `Some`, because `block_transactions` asserts
//! `>9<` for nonce 9.

use splora_web::block_details_markup;

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
