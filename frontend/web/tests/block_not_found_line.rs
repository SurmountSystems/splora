//! Whole lines `Block not found` and `Error loading block data.` on a failed block load.
//!
//! GPUI requires one line whose entire text equals `Block not found` and one
//! line whose entire text equals `Error loading block data.`. The span stays
//! inside the http-error div. The italic stays `(Block not found)`. That
//! parenthesized text is not the whole line, and the span text is not the
//! whole line. A loaded block omits both lines.

use splora_web::{block_rows_markup, indexer_failure_text};

const LINE: &str = "Block not found";
const ERROR_LINE: &str = "Error loading block data.";
const SPAN: &str = "<span>Error loading block data.</span>";
const ITALIC: &str = "<i class=\"small\">(Block not found)</i>";

fn failed_markup() -> String {
    let id = "nope";
    let failed = indexer_failure_text("GET", &format!("/block/{id}"), LINE);
    block_rows_markup(&failed, "")
}

#[test]
fn failed_block_load_paints_block_not_found_and_the_error_as_their_own_lines() {
    let markup = failed_markup();
    assert!(
        markup.contains(SPAN),
        "the span is not element text: {markup}"
    );
    assert!(
        markup.contains(ITALIC),
        "the italic HTTP body dropped: {markup}"
    );
    assert_eq!(
        markup.split('\n').filter(|line| *line == LINE).count(),
        1,
        "no line equals {LINE}: {markup}"
    );
    assert_eq!(
        markup
            .split('\n')
            .filter(|line| *line == ERROR_LINE)
            .count(),
        1,
        "no line equals {ERROR_LINE}: {markup}"
    );
    let italic_at = markup.find(ITALIC).expect("italic");
    let body_at = markup.find(LINE).expect("indexer body");
    assert!(
        italic_at < body_at && body_at < italic_at + ITALIC.len(),
        "the first Block not found is not inside the italic: {markup}"
    );
    let own_at = markup.find(&format!("\n{LINE}\n")).expect("own line");
    assert!(
        italic_at < own_at,
        "the whole line is not after the italic parentheses: {markup}"
    );
    let span_at = markup.find(SPAN).expect("span");
    let sentence_at = markup.find(ERROR_LINE).expect("span sentence");
    assert!(
        span_at < sentence_at && sentence_at < span_at + SPAN.len(),
        "the first Error loading block data. is not inside the span: {markup}"
    );
    let error_own = markup
        .find(&format!("\n{ERROR_LINE}\n"))
        .expect("error line");
    assert!(
        span_at < error_own,
        "the error whole line is not after the span: {markup}"
    );

    let loaded = r#"{"id":"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa","height":1,"version":1,"timestamp":1,"tx_count":1,"size":1,"weight":4,"merkle_root":"merkle","mediantime":1}"#;
    let painted = block_rows_markup(loaded, "[]");
    assert!(
        !painted.split('\n').any(|line| line == LINE),
        "a loaded block painted {LINE}: {painted}"
    );
    assert!(
        !painted.split('\n').any(|line| line == ERROR_LINE),
        "a loaded block painted {ERROR_LINE}: {painted}"
    );
    assert!(
        !painted.contains(LINE),
        "a loaded block painted {LINE}: {painted}"
    );
    assert!(
        !painted.contains(ERROR_LINE),
        "a loaded block painted the error span: {painted}"
    );

    let bare = indexer_failure_text("GET", "/block/nope", "");
    let bare_markup = block_rows_markup(&bare, "");
    assert!(
        !bare_markup.split('\n').any(|line| line == LINE),
        "an empty HTTP body painted {LINE}: {bare_markup}"
    );
    assert_eq!(
        bare_markup.matches("\n\n").count(),
        0,
        "an empty HTTP body pushed a blank line: {bare_markup}"
    );
}
