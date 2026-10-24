//! Lowercase difficulty line on open block details.
//!
//! GPUI `block_screen_paints_difficulty_label` requires one line whose entire
//! text is `difficulty 1` when JSON difficulty is 1.0, beside `Difficulty 1`.
//! Closed details omit that lowercase line. A missing key paints no line that
//! starts with `Difficulty` or `difficulty`. JSON 1.5 paints `difficulty 1.5`
//! and `Difficulty 1.5`. The strings `difficulty 1.0` and `Difficulty 1.0`
//! stay off.

use splora_web::block_details_markup;

fn difficulty_block_json(id: &str, difficulty: Option<&str>) -> String {
    let difficulty_field = match difficulty {
        Some(value) => format!(r#","difficulty":{value}"#),
        None => String::new(),
    };
    format!(
        r#"{{"id":"{id}","height":2,"version":1,"timestamp":1700000000,"tx_count":1,"size":100,"weight":400,"merkle_root":"merkle-root","previousblockhash":null,"mediantime":1699990000,"nonce":9,"bits":2{difficulty_field}}}"#
    )
}

fn assert_no_painted_one_point_zero(markup: &str) {
    assert!(
        !markup
            .lines()
            .any(|line| line == "difficulty 1.0" || line == "Difficulty 1.0"),
        "plain_number painted 1.0: {markup}"
    );
    assert!(
        !markup.contains("difficulty 1.0"),
        "the string difficulty 1.0 appeared: {markup}"
    );
    assert!(
        !markup.contains("Difficulty 1.0"),
        "the string Difficulty 1.0 appeared: {markup}"
    );
}

/// Whole line `difficulty 1` from no-query block JSON when details are open.
#[test]
fn open_block_details_paint_difficulty_lower_line() {
    const LINE: &str = "difficulty 1";
    const CAPITAL: &str = "Difficulty 1";
    let id = "difflower1";

    let body = difficulty_block_json(id, Some("1.0"));
    let open = block_details_markup(&body, true);
    assert_eq!(
        open.lines().filter(|line| *line == LINE).count(),
        1,
        "{LINE} is absent: {open}"
    );
    assert_eq!(
        open.lines().filter(|line| *line == CAPITAL).count(),
        1,
        "{CAPITAL} was not one line: {open}"
    );
    assert_no_painted_one_point_zero(&open);

    let closed = block_details_markup(&body, false);
    assert!(
        !closed.lines().any(|line| line == LINE),
        "closed details painted {LINE}: {closed}"
    );
    assert_no_painted_one_point_zero(&closed);

    let missing = block_details_markup(&difficulty_block_json(id, None), true);
    assert!(
        !missing.lines().any(|line| line == LINE),
        "missing difficulty painted {LINE}: {missing}"
    );
    assert!(
        missing
            .lines()
            .all(|line| !line.starts_with("Difficulty") && !line.starts_with("difficulty")),
        "missing difficulty painted a difficulty line: {missing}"
    );
    assert_no_painted_one_point_zero(&missing);

    let fraction = block_details_markup(&difficulty_block_json(id, Some("1.5")), true);
    assert_eq!(
        fraction
            .lines()
            .filter(|line| *line == "difficulty 1.5")
            .count(),
        1,
        "difficulty 1.5 is absent: {fraction}"
    );
    assert_eq!(
        fraction
            .lines()
            .filter(|line| *line == "Difficulty 1.5")
            .count(),
        1,
        "Difficulty 1.5 is absent: {fraction}"
    );
    assert_no_painted_one_point_zero(&fraction);
}
