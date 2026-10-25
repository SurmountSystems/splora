//! Block version hex beside the decimal in the details panel.
//!
//! Angular `decimal2hex` defaults to 4 bytes: `0x` plus 8 lowercase hex
//! digits. Explicit 0 is `0x00000000`. A missing version key fails the
//! block parse and does not paint the hex cell.

use splora_web::{block_details_markup, block_load_plan};

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

/// Whole line `version 1 0x00000001` from no-query `GET /block/:hash` when details are open.
///
/// The Version cell and the eight-digit hex cell stay. Closed details omit the line.
/// This is not Difficulty and not Merkle root.
#[test]
fn open_block_details_paint_version_1_hex() {
    const LINE: &str = "version 1 0x00000001";
    let id = "verhex1";
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
        open.contains(
            "<tr><td>Version</td><td>1</td><td class=\"version-hex\">0x00000001</td></tr>"
        ),
        "{open}"
    );

    let closed = block_details_markup(&block_json("1"), false);
    assert!(
        !closed.lines().any(|line| line == LINE),
        "closed details painted {LINE}: {closed}"
    );
    assert!(
        closed.contains(
            "<tr><td>Version</td><td>1</td><td class=\"version-hex\">0x00000001</td></tr>"
        ),
        "{closed}"
    );
}

/// Whole line `version 0 0x00000000` when open details carry numeric version 0.
///
/// Native `paint_block_version("verhex0", 0)` opens block details on a
/// no-query `GET /block/:id` whose JSON `version` is 0. Version 536870912
/// and version 1 are different lines. Closed details and a missing version
/// key omit this line. The Version cell and the eight-digit hex cell stay.
#[test]
fn open_block_details_paint_version_0_0x00000000() {
    const LINE: &str = "version 0 0x00000000";
    let id = "verhex0";
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

    let open = block_details_markup(&block_json("0"), true);
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
        open.contains(
            "<tr><td>Version</td><td>0</td><td class=\"version-hex\">0x00000000</td></tr>"
        ),
        "{open}"
    );

    let segwit = block_details_markup(&block_json("536870912"), true);
    assert!(
        !segwit.lines().any(|line| line == LINE),
        "version 536870912 painted {LINE}: {segwit}"
    );
    let one = block_details_markup(&block_json("1"), true);
    assert!(
        !one.lines().any(|line| line == LINE),
        "version 1 painted {LINE}: {one}"
    );
    let closed = block_details_markup(&block_json("0"), false);
    assert!(
        !closed.lines().any(|line| line == LINE),
        "closed details painted {LINE}: {closed}"
    );
    assert!(
        closed.contains(
            "<tr><td>Version</td><td>0</td><td class=\"version-hex\">0x00000000</td></tr>"
        ),
        "{closed}"
    );
    let missing = block_details_markup(&missing_version_json(), true);
    assert!(
        !missing.lines().any(|line| line == LINE),
        "missing version painted {LINE}: {missing}"
    );
}

/// Whole line `version 536870912 0x20000000` when open details carry version 536870912.
///
/// Native `paint_block_version("verhex536", 536870912)` opens block details on a
/// no-query `GET /block/:id` whose JSON `version` is 536870912. Version 0 and
/// version 1 stay their own lines. Closed details and a missing version key
/// omit this line. The Version cell and the eight-digit hex cell stay.
#[test]
fn open_block_details_paint_version_536870912_0x20000000() {
    const LINE: &str = "version 536870912 0x20000000";
    let id = "verhex536";
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

    let open = block_details_markup(&block_json("536870912"), true);
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
        open.contains(
            "<tr><td>Version</td><td>536870912</td><td class=\"version-hex\">0x20000000</td></tr>"
        ),
        "{open}"
    );

    let zero = block_details_markup(&block_json("0"), true);
    assert!(
        !zero.lines().any(|line| line == LINE),
        "version 0 painted {LINE}: {zero}"
    );
    let one = block_details_markup(&block_json("1"), true);
    assert!(
        !one.lines().any(|line| line == LINE),
        "version 1 painted {LINE}: {one}"
    );
    let closed = block_details_markup(&block_json("536870912"), false);
    assert!(
        !closed.lines().any(|line| line == LINE),
        "closed details painted {LINE}: {closed}"
    );
    assert!(
        closed.contains(
            "<tr><td>Version</td><td>536870912</td><td class=\"version-hex\">0x20000000</td></tr>"
        ),
        "{closed}"
    );
    let missing = block_details_markup(&missing_version_json(), true);
    assert!(
        !missing.lines().any(|line| line == LINE),
        "missing version painted {LINE}: {missing}"
    );
}
