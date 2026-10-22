//! Status footer label from `GET /mempool`.
//!
//! Angular `footer.component.html` paints Mempool size from the mempool
//! payload. This indexer serves that payload as `vsize` on `GET /mempool`
//! with no query string. The digits stay the raw virtual-byte count. They
//! are not kB or MB, and they are not a projected block count. Incoming
//! Transactions stays off this line. An empty body or a body that does not
//! parse omits the label.

use splora_web::{mempool_paths, mempool_size_markup};

const LABEL: &str = "<span class=\"mempoolSize\">Mempool size:</span>";

fn mempool_json(vsize: u32) -> String {
    format!(
        r#"{{"count":2,"vsize":{vsize},"total_fee":1500,"fee_histogram":[[10.5,200],[1,100]]}}"#
    )
}

fn views_source() -> &'static str {
    include_str!("../src/views.rs")
}

fn function_body<'a>(src: &'a str, name: &str) -> &'a str {
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
    panic!("{name} body did not close")
}

fn assert_one(markup: &str, expected: &str) {
    assert_eq!(
        markup.matches(expected).count(),
        1,
        "expected one {expected} in {markup}"
    );
}

fn assert_absent(markup: &str, needle: &str) {
    assert!(
        !markup.contains(needle),
        "{needle} leaked into the markup: {markup}"
    );
}

#[test]
fn status_footer_paints_mempool_size_from_vsize() {
    let markup = mempool_size_markup(&mempool_json(300));
    assert!(markup.contains(LABEL), "Mempool size is absent: {markup}");
    assert_one(&markup, LABEL);
    assert_one(&markup, "<div class=\"sub-text\">300</div>");
    assert_eq!(markup.matches("Mempool size").count(), 1, "{markup}");
    assert!(
        markup.find(LABEL).unwrap() < markup.find("<div class=\"sub-text\">300</div>").unwrap(),
        "the label belongs before the raw vsize: {markup}"
    );
    assert_absent(&markup, "kB");
    assert_absent(&markup, "MB");
    assert_absent(&markup, "block");
    assert_absent(&markup, "Incoming Transactions");
    assert_absent(&markup, "Unconfirmed");
    assert_absent(&markup, "300,");
    assert!(!markup.contains('?'), "{markup}");

    let zero = mempool_size_markup(&mempool_json(0));
    assert_one(&zero, LABEL);
    assert_one(&zero, "<div class=\"sub-text\">0</div>");

    let wide = mempool_size_markup(&mempool_json(1_000_000));
    assert_one(&wide, "<div class=\"sub-text\">1000000</div>");
    assert_absent(&wide, "1,000,000");

    for body in [
        "",
        "not-json",
        "{}",
        r#"{"count":2,"total_fee":1,"fee_histogram":[]}"#,
    ] {
        let omitted = mempool_size_markup(body);
        assert_absent(&omitted, "Mempool size");
        assert_absent(&omitted, LABEL);
    }

    let paths = mempool_paths();
    assert_eq!(paths[0], "/mempool");
    assert!(!paths[0].contains('?'), "{}", paths[0]);
    assert!(!paths[1].contains('?'), "{}", paths[1]);

    let dashboard = function_body(views_source(), "Dashboard");
    assert!(
        dashboard.contains("mempool_size_markup("),
        "Dashboard does not paint Mempool size"
    );
    assert!(
        !dashboard.contains('?'),
        "Dashboard read a query string: {dashboard}"
    );
    let mempool_page = function_body(views_source(), "MempoolPage");
    assert!(
        mempool_page.contains("mempool_size_markup("),
        "MempoolPage does not paint Mempool size"
    );
    assert!(
        !mempool_page.contains('?'),
        "MempoolPage read a query string: {mempool_page}"
    );
}
