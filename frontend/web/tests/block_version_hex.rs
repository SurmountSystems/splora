//! Block version hex beside the decimal in the details panel.
//!
//! Angular `decimal2hex` defaults to 4 bytes: `0x` plus 8 lowercase hex
//! digits. Explicit 0 is `0x00000000`. A missing version key fails the
//! block parse and does not paint the hex cell.

use splora_web::block_details_markup;

const ID: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";

fn block_json(version: &str) -> String {
    format!(
        r#"{{"id":"{ID}","height":2,"version":{version},"timestamp":1700000000,"tx_count":1,"size":100,"weight":400,"merkle_root":"merkle-root","previousblockhash":null,"mediantime":1699990000,"nonce":9,"bits":386547056,"difficulty":1}}"#
    )
}

fn missing_version_json() -> String {
    format!(
        r#"{{"id":"{ID}","height":2,"timestamp":1700000000,"tx_count":1,"size":100,"weight":400,"merkle_root":"merkle-root","previousblockhash":null,"mediantime":1699990000,"nonce":9,"bits":386547056,"difficulty":1}}"#
    )
}

fn hex_cell(digits: &str) -> String {
    format!("<td class=\"version-hex\">0x{digits}</td>")
}

fn assert_one(markup: &str, expected: &str) {
    assert_eq!(
        markup.matches(expected).count(),
        1,
        "expected one {expected} in {markup}"
    );
}

#[test]
fn version_536870912_paints_hex_beside_the_decimal() {
    let open = block_details_markup(&block_json("536870912"), true);
    assert_one(&open, &hex_cell("20000000"));
    assert!(open.contains("536870912"), "{open}");
    assert!(open.contains("Version"), "{open}");
}

#[test]
fn version_zero_paints_eight_zero_digits() {
    let open = block_details_markup(&block_json("0"), true);
    assert_one(&open, &hex_cell("00000000"));
    assert!(open.contains("Version"), "{open}");
}

#[test]
fn version_one_paints_padded_one() {
    let open = block_details_markup(&block_json("1"), true);
    assert_one(&open, &hex_cell("00000001"));
    assert!(open.contains(">1<"), "{open}");
}

#[test]
fn missing_version_key_does_not_paint_hex() {
    let open = block_details_markup(&missing_version_json(), true);
    assert!(
        !open.contains("version-hex"),
        "missing version painted a hex cell: {open}"
    );
}
