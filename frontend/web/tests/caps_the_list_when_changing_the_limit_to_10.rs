//! Cypress it() title: caps the list when changing the limit to 10.
//!
//! Nine mocked websocket batches of six transactions fill the recent
//! list. The rendered transaction ids must be the ids
//! `splora_frontend_shared::caps_the_list_when_changing_the_limit_to_10`
//! returns for that same batch list, at the default cap and after the
//! limit-10 control. This test does not hard-code txids and it does not
//! treat a row count as the pass rule.
//! `paint_caps_the_list_when_changing_the_limit_to_10` is not in this crate.
//! The call is the missing paint.

use splora_frontend_shared::caps_the_list_when_changing_the_limit_to_10 as shared_list_cap;

fn mock_txid(batch: u8, index: u8) -> String {
    let mut id = format!("{batch:02x}{index:02x}");
    id.push_str(&"00".repeat(30));
    id
}

#[test]
fn caps_the_list_when_changing_the_limit_to_10() {
    let owned: [[String; 6]; 9] = std::array::from_fn(|batch| {
        std::array::from_fn(|index| mock_txid(batch as u8, index as u8))
    });
    let batches: [[&str; 6]; 9] =
        std::array::from_fn(|batch| std::array::from_fn(|index| owned[batch][index].as_str()));

    let (rows_at_limit_50, rows_at_limit_10) = shared_list_cap(&batches);

    let markup = splora_web::paint_caps_the_list_when_changing_the_limit_to_10(&batches);
    let rendered_at_limit_50 = section_txids(&markup, "data-cy=\"before-limit-control\"");
    let rendered_at_limit_10 = section_txids(&markup, "data-cy=\"after-limit-control\"");

    assert!(
        markup.contains("data-cy=\"transactions-list\""),
        "the limit control did not leave the transactions list Cypress queries: {markup}"
    );
    assert!(
        markup.contains("data-cy=\"limit-10\""),
        "the limit-10 control Cypress clicks is missing: {markup}"
    );
    assert_eq!(
        rendered_at_limit_50, rows_at_limit_50,
        "rendered ids at the default cap are not the ids the shared function returns: {markup}"
    );
    assert_eq!(
        rendered_at_limit_10, rows_at_limit_10,
        "rendered ids after the limit-10 control are not the ids the shared function returns: {markup}"
    );
}

/// Transaction ids inside one painted section.
fn section_txids<'a>(markup: &'a str, open_marker: &str) -> Vec<&'a str> {
    let Some(start) = markup.find(open_marker) else {
        panic!("missing {open_marker} in {markup}");
    };
    let rest = &markup[start..];
    let Some(close) = rest.find("</section>") else {
        panic!("missing section close after {open_marker} in {markup}");
    };
    visible_txid_texts(&rest[..close])
}

/// Text of each `.table-cell-txid a`, which is what Cypress reads.
fn visible_txid_texts(markup: &str) -> Vec<&str> {
    const CLASS_NAME: &str = "class=\"table-cell-txid\"";
    let mut texts = Vec::new();
    let mut search_from = 0;
    while let Some(relative) = markup[search_from..].find(CLASS_NAME) {
        let cell_start = search_from + relative + CLASS_NAME.len();
        let cell_end = markup[cell_start..]
            .find("</t")
            .map(|end| cell_start + end)
            .unwrap_or(markup.len());
        if let Some(text) = anchor_text(&markup[cell_start..cell_end]) {
            if !text.is_empty() {
                texts.push(text);
            }
        }
        search_from = cell_end;
        if search_from >= markup.len() {
            break;
        }
    }
    texts
}

fn anchor_text(cell: &str) -> Option<&str> {
    let anchor_at = cell.find("<a")?;
    let after_anchor = &cell[anchor_at..];
    let content_at = after_anchor.find('>')? + 1;
    let end = after_anchor[content_at..].find("</a>")?;
    Some(after_anchor[content_at..content_at + end].trim())
}
