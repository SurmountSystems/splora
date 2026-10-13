//! Raw block and raw transaction bodies are the indexer octet-stream.
//! They render beside the rows. A failed block-height body is a notice.

use splora_web::{block_raw_beside, height_body_markup, tx_raw_beside};

fn view_fn_body(name: &str) -> &'static str {
    let src = include_str!("../src/views.rs");
    let marker = format!("fn {name}(");
    let start = src
        .find(&marker)
        .unwrap_or_else(|| panic!("{name} is missing from views.rs"));
    let rest = &src[start..];
    let after_sig = rest.find('\n').unwrap_or(0) + 1;
    let tail = &rest[after_sig..];
    let end_in_tail = ["\nfn ", "\n#[component]", "\nconst "]
        .iter()
        .filter_map(|needle| tail.find(needle))
        .min();
    let end = after_sig + end_in_tail.unwrap_or(tail.len());
    &rest[..end]
}

#[test]
fn raw_octet_stream_shows_beside_the_block_and_transaction_rows() {
    let rows = "<tr class=\"block-row\">fields</tr>";
    let block_bytes = [0xaa, 0x55, 0x00, 0xff];
    let block = block_raw_beside(rows, &block_bytes);
    assert!(block.contains(rows), "{block}");
    let rows_at = block.find(rows).expect("rows");
    let raw_at = block.find("block-raw").expect("raw block");
    assert!(rows_at < raw_at, "{block}");
    assert!(
        block.contains("<pre class=\"block-raw\">aa5500ff</pre>"),
        "{block}"
    );

    let tx_rows = "<tr class=\"tx-row\">fields</tr>";
    let tx_bytes = [0x02, 0x00, 0x00, 0x00, 0x01];
    let tx = tx_raw_beside(tx_rows, &tx_bytes);
    assert!(tx.contains(tx_rows), "{tx}");
    let tx_rows_at = tx.find(tx_rows).expect("tx rows");
    let tx_raw_at = tx.find("tx-raw").expect("raw tx");
    assert!(tx_rows_at < tx_raw_at, "{tx}");
    assert!(
        tx.contains("<pre class=\"tx-raw\">0200000001</pre>"),
        "{tx}"
    );

    assert_eq!(block_raw_beside(rows, b""), rows);
    let failed = b"Block not found\nindexer error GET /block/abc/raw";
    let notice = block_raw_beside(rows, failed);
    assert!(notice.contains(rows), "{notice}");
    assert!(notice.contains("Block not found"), "{notice}");
    assert!(notice.contains("class=\"notice\""), "{notice}");
    assert!(!notice.contains("<pre"), "{notice}");

    let block_page = view_fn_body("BlockPage");
    assert!(block_page.contains("block_rows_markup("), "{block_page}");
    assert!(block_page.contains("block_raw_beside("), "{block_page}");
    let tx_page = view_fn_body("TransactionPage");
    assert!(tx_page.contains("transaction_row_markup("), "{tx_page}");
    assert!(tx_page.contains("tx_raw_beside("), "{tx_page}");

    let browser = include_str!("../src/browser.rs");
    assert!(
        browser.contains("array_buffer"),
        "raw octet-stream is still read with response.text()"
    );
}

#[test]
fn failed_block_height_body_is_a_notice() {
    let failed = "Block not found\nindexer error GET /block-height/100";
    let html = height_body_markup(failed);
    assert!(html.contains("Block not found"), "{html}");
    assert!(
        html.contains("indexer error GET /block-height/100"),
        "{html}"
    );
    assert!(html.contains("class=\"notice\""), "{html}");
    assert!(!html.contains("<pre"), "{html}");

    let hash = "ab".repeat(32);
    let shown = height_body_markup(&hash);
    assert!(!shown.contains("<pre"), "{shown}");

    let block_page = view_fn_body("BlockPage");
    assert!(block_page.contains("height_body_markup("), "{block_page}");
    assert!(
        !block_page.contains("<pre"),
        "BlockPage still preformats the height body: {block_page}"
    );
}
