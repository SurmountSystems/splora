//! Block transaction heading, details toggle, and pager.
//!
//! Cypress `ITEMS_PER_PAGE` is 25, and the indexer page size default is 25.
//! The wide pager shows at most five page numbers. The phone max of three
//! needs a viewport, so it is not asserted here.

use splora_web::{
    block_details_markup, block_load_plan, block_transactions_markup, block_tx_page_path,
    details_open_after_click,
};

const HASH: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
const PAGE_ONE: &str = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
const PAGE_TWO: &str = "cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc";

fn block_json(tx_count: u32) -> String {
    format!(
        r#"{{"id":"{HASH}","height":2,"version":536870912,"timestamp":1700000000,"tx_count":{tx_count},"size":100,"weight":400,"merkle_root":"merkle-root","previousblockhash":null,"mediantime":1699990000,"nonce":9,"bits":386547056,"difficulty":1}}"#,
    )
}

fn txs_json(txid: &str) -> String {
    format!(
        r#"[{{"txid":"{txid}","version":1,"locktime":0,"vin":[],"vout":[],"size":10,"weight":40,"sigops":0,"fee":1,"status":{{"confirmed":true,"block_height":2}}}}]"#,
    )
}

fn views_source() -> &'static str {
    include_str!("../src/views.rs")
}

fn component_body<'a>(src: &'a str, name: &str) -> &'a str {
    let marker = format!("fn {name}(");
    let start = src
        .find(&marker)
        .unwrap_or_else(|| panic!("{name} is missing from views.rs"));
    let after_name = &src[start..];
    let brace_rel = after_name
        .find('{')
        .unwrap_or_else(|| panic!("{name} has no body"));
    let body_start = start + brace_rel + 1;
    let mut depth = 1i32;
    for (offset, ch) in src[body_start..].char_indices() {
        match ch {
            '{' => depth += 1,
            '}' => {
                depth -= 1;
                if depth == 0 {
                    return &src[body_start..body_start + offset];
                }
            }
            _ => {}
        }
    }
    panic!("{name} body is not closed");
}

fn h2_text(markup: &str) -> String {
    let start = markup
        .find("<h2 class=\"text-start\">")
        .unwrap_or_else(|| panic!("missing transaction heading in {markup}"));
    let rest = &markup[start + "<h2 class=\"text-start\">".len()..];
    let end = rest.find("</h2>").expect("heading is not closed");
    rest[..end].to_string()
}

fn pagination_children(markup: &str) -> usize {
    let start = markup
        .find("<ul class=\"pagination\">")
        .expect("missing pager");
    let rest = &markup[start..];
    let end = rest.find("</ul>").expect("pager is not closed");
    rest[..end].matches("<li").count()
}

fn header_tx_text(markup: &str) -> String {
    let needle = "<div class=\"header-bg box\"><a class=\"tx-link\" href=\"/tx/";
    let start = markup
        .find(needle)
        .unwrap_or_else(|| panic!("missing transaction header in {markup}"));
    let rest = &markup[start + needle.len()..];
    let close = rest.find('"').expect("txid href is not closed");
    rest[..close].to_string()
}

fn details_is_hidden(markup: &str) -> bool {
    let start = markup.find("<div id=\"details\"").expect("missing details");
    let rest = &markup[start..];
    let end = rest.find('>').expect("details tag is not closed");
    rest[..end]
        .split_whitespace()
        .any(|token| token == "hidden")
}

fn li_after_active_href(markup: &str) -> String {
    let start = markup
        .find("class=\"page-item active\"")
        .expect("missing active page");
    let rest = &markup[start..];
    let next = rest
        .find("<li")
        .expect("no pager item after the active page");
    let item = &rest[next..];
    let href = item.find("href=\"").expect("next page has no href");
    let value = &item[href + "href=\"".len()..];
    let end = value.find('"').expect("href is not closed");
    value[..end].to_string()
}

#[test]
fn empty_block_heading_is_one_transaction() {
    let markup = block_transactions_markup(&block_json(1), &txs_json(PAGE_ONE), 1);
    assert_eq!(h2_text(&markup), "1 transaction");
    assert_eq!(pagination_children(&markup), 5);
    assert_eq!(block_tx_page_path(HASH, 1), format!("/block/{HASH}/txs"));
    assert!(
        markup.contains(&format!("data-path=\"/block/{HASH}/txs\"")),
        "{markup}"
    );
}

#[test]
fn one_page_heading_matches_mainnet_signet_and_testnet4() {
    for (count, heading) in [
        (19, "19 transactions"),
        (13, "13 transactions"),
        (18, "18 transactions"),
    ] {
        let markup = block_transactions_markup(&block_json(count), &txs_json(PAGE_ONE), 1);
        assert_eq!(h2_text(&markup), heading, "tx_count {count}");
        assert_eq!(pagination_children(&markup), 5, "tx_count {count}");
    }
}

#[test]
fn next_page_changes_the_transaction_header() {
    let block = block_json(41);
    let first = block_transactions_markup(&block, &txs_json(PAGE_ONE), 1);
    let second = block_transactions_markup(&block, &txs_json(PAGE_TWO), 2);
    assert_eq!(header_tx_text(&first), PAGE_ONE);
    assert_eq!(header_tx_text(&second), PAGE_TWO);
    assert_ne!(header_tx_text(&first), header_tx_text(&second));
    assert_eq!(h2_text(&first), "41 transactions");
    assert_eq!(block_tx_page_path(HASH, 2), format!("/block/{HASH}/txs/25"));
    assert!(
        second.contains(&format!("data-path=\"/block/{HASH}/txs/25\"")),
        "{second}"
    );
    assert_eq!(
        li_after_active_href(&first),
        format!("/block/{HASH}?start_index=25")
    );
}

#[test]
fn five_wide_pages_have_nine_pager_children() {
    let markup = block_transactions_markup(&block_json(125), &txs_json(PAGE_ONE), 1);
    assert_eq!(h2_text(&markup), "125 transactions");
    assert_eq!(pagination_children(&markup), 9);
}

#[test]
fn details_button_shows_then_hides_the_panel() {
    let block = block_json(1);
    assert!(!details_open_after_click(true));
    assert!(details_open_after_click(false));
    assert!(!details_open_after_click(details_open_after_click(false)));

    let closed = block_details_markup(&block, false);
    let open = block_details_markup(&block, true);
    assert!(details_is_hidden(&closed), "{closed}");
    assert!(!details_is_hidden(&open), "{open}");
    for label in ["Version", "Bits", "Merkle root", "Difficulty", "Nonce"] {
        assert!(open.contains(label), "{label} missing from {open}");
    }
    assert!(open.contains("536870912"), "{open}");
    assert!(open.contains("merkle-root"), "{open}");
    assert!(open.contains("386547056"), "{open}");
    assert!(open.contains(">1<"), "{open}");
    assert!(open.contains(">9<"), "{open}");
    assert!(
        open.contains(&format!("href=\"/block/{HASH}/header\"")),
        "{open}"
    );
    assert!(!open.contains("Fiat price last updated"), "{open}");

    let page = component_body(views_source(), "BlockPage");
    assert!(
        page.contains("class=\"btn btn-outline-info\""),
        "BlockPage is missing the details button"
    );
    assert!(
        page.contains("block_details_markup("),
        "BlockPage does not paint the details panel"
    );
    assert!(
        page.contains("details_open_after_click("),
        "BlockPage does not toggle the details panel"
    );
    assert!(
        page.contains("block_transactions_markup("),
        "BlockPage does not paint the transaction heading"
    );
    assert!(
        !page.contains("Fiat price last updated"),
        "BlockPage renders a fiat timestamp"
    );
}

fn difficulty_block_json(id: &str, difficulty: Option<&str>) -> String {
    let difficulty_field = match difficulty {
        Some(value) => format!(r#","difficulty":{value}"#),
        None => String::new(),
    };
    format!(
        r#"{{"id":"{id}","height":2,"version":1,"timestamp":1700000000,"tx_count":1,"size":100,"weight":400,"merkle_root":"merkle-root","previousblockhash":null,"mediantime":1699990000,"nonce":9,"bits":2{difficulty_field}}}"#
    )
}

/// Whole line `Difficulty 1` from no-query `GET /block/:hash` when details are open.
///
/// The Difficulty cell stays. `1.0` prints as `1`, not `1.0`. Closed details
/// omit the line. A missing key omits the cell and the line.
#[test]
fn open_block_details_paint_difficulty_1() {
    const LINE: &str = "Difficulty 1";
    let id = "diffblock1";
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

    let body = difficulty_block_json(id, Some("1.0"));
    let open = block_details_markup(&body, true);
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
        open.contains("<tr><td>Difficulty</td><td>1</td></tr>"),
        "{open}"
    );
    assert!(
        !open
            .lines()
            .any(|line| line == "Difficulty 1.0" || line == "difficulty 1.0"),
        "plain_number painted 1.0: {open}"
    );

    let closed = block_details_markup(&body, false);
    assert!(
        !closed.lines().any(|line| line == LINE),
        "closed details painted {LINE}: {closed}"
    );
    assert!(
        closed.contains("<tr><td>Difficulty</td><td>1</td></tr>"),
        "{closed}"
    );

    let missing = block_details_markup(&difficulty_block_json(id, None), true);
    assert!(
        !missing.lines().any(|line| line == LINE),
        "missing difficulty painted {LINE}: {missing}"
    );
    assert!(!missing.contains("<td>Difficulty</td>"), "{missing}");
}

/// Whole line `Difficulty 1.5` from no-query `GET /block/:hash` when details are open.
///
/// The Difficulty cell stays. The integer line `Difficulty 1` is not painted
/// again. Closed details omit the line. A missing key omits the cell and the line.
#[test]
fn open_block_details_paint_difficulty_1_5() {
    const LINE: &str = "Difficulty 1.5";
    let id = "diffblock15";
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

    let body = difficulty_block_json(id, Some("1.5"));
    let open = block_details_markup(&body, true);
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
        open.contains("<tr><td>Difficulty</td><td>1.5</td></tr>"),
        "{open}"
    );
    assert!(
        !open.lines().any(|line| line == "Difficulty 1"),
        "painted Difficulty 1 again: {open}"
    );

    let closed = block_details_markup(&body, false);
    assert!(
        !closed.lines().any(|line| line == LINE),
        "closed details painted {LINE}: {closed}"
    );
    assert!(
        closed.contains("<tr><td>Difficulty</td><td>1.5</td></tr>"),
        "{closed}"
    );

    let missing = block_details_markup(&difficulty_block_json(id, None), true);
    assert!(
        !missing.lines().any(|line| line == LINE),
        "missing difficulty painted {LINE}: {missing}"
    );
    assert!(!missing.contains("<td>Difficulty</td>"), "{missing}");
}

fn merkle_block_json(id: &str, merkle_root: &str) -> String {
    format!(
        r#"{{"id":"{id}","height":2,"version":1,"timestamp":1700000000,"tx_count":1,"size":100,"weight":400,"merkle_root":"{merkle_root}","previousblockhash":null,"mediantime":1699990000,"nonce":9,"bits":2,"difficulty":1}}"#
    )
}

/// Whole line `Merkle root mm` from no-query `GET /block/:hash` when details are open.
///
/// The Merkle root cell stays. Closed details omit the line. This is not Difficulty.
#[test]
fn open_block_details_paint_merkle_root_mm() {
    const LINE: &str = "Merkle root mm";
    let id = "merklemm";
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

    let body = merkle_block_json(id, "mm");
    let open = block_details_markup(&body, true);
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
        open.contains("<tr><td>Merkle root</td><td>mm</td></tr>"),
        "{open}"
    );

    let closed = block_details_markup(&body, false);
    assert!(
        !closed.lines().any(|line| line == LINE),
        "closed details painted {LINE}: {closed}"
    );
    assert!(
        closed.contains("<tr><td>Merkle root</td><td>mm</td></tr>"),
        "{closed}"
    );
}

fn previous_block_json(id: &str, previous: Option<&str>) -> String {
    let previous_field = match previous {
        Some(value) => format!(r#","previousblockhash":"{value}""#),
        None => String::new(),
    };
    format!(
        r#"{{"id":"{id}","height":800000,"version":1,"timestamp":1700000000,"tx_count":1,"size":20,"weight":80,"merkle_root":"mm","mediantime":1700000000,"nonce":1,"bits":2,"difficulty":1{previous_field}}}"#
    )
}

/// Whole line `previousblockhash prevblock` from no-query `GET /block/:hash` when details are open.
///
/// Closed details omit the line. A missing key omits the line. This is not Merkle root.
#[test]
fn open_block_details_paint_previousblockhash_prevblock() {
    const LINE: &str = "previousblockhash prevblock";
    let id = "oneblock";
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

    let body = previous_block_json(id, Some("prevblock"));
    let open = block_details_markup(&body, true);
    assert!(
        open.lines().any(|line| line == LINE),
        "{LINE} is absent: {open}"
    );
    assert_eq!(
        open.lines().filter(|line| *line == LINE).count(),
        1,
        "{LINE} was not one line: {open}"
    );

    let closed = block_details_markup(&body, false);
    assert!(
        !closed.lines().any(|line| line == LINE),
        "closed details painted {LINE}: {closed}"
    );

    let missing = block_details_markup(&previous_block_json(id, None), true);
    assert!(
        !missing.lines().any(|line| line == LINE),
        "missing previousblockhash painted {LINE}: {missing}"
    );
    assert!(
        !missing
            .lines()
            .any(|line| line.starts_with("previousblockhash")),
        "missing previousblockhash painted a previous line: {missing}"
    );
}

/// Whole line `mediantime 1700000000` from no-query `GET /block/:hash` when details are open.
///
/// Closed details omit the line. A missing key does not invent the line.
/// This is not the previousblockhash line.
#[test]
fn open_block_details_paint_mediantime_1700000000() {
    const LINE: &str = "mediantime 1700000000";
    let id = "oneblock";
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

    let body = previous_block_json(id, Some("prevblock"));
    let open = block_details_markup(&body, true);
    assert!(
        open.lines().any(|line| line == LINE),
        "{LINE} is absent: {open}"
    );
    assert_eq!(
        open.lines().filter(|line| *line == LINE).count(),
        1,
        "{LINE} was not one line: {open}"
    );

    let closed = block_details_markup(&body, false);
    assert!(
        !closed.lines().any(|line| line == LINE),
        "closed details painted {LINE}: {closed}"
    );

    let missing = r#"{"id":"oneblock","height":800000,"version":1,"timestamp":1700000000,"tx_count":1,"size":20,"weight":80,"merkle_root":"mm","previousblockhash":"prevblock","nonce":1,"bits":2,"difficulty":1}"#;
    let missing_paint = block_details_markup(missing, true);
    assert!(
        !missing_paint
            .lines()
            .any(|line| line.starts_with("mediantime")),
        "missing mediantime painted a median line: {missing_paint}"
    );
}
