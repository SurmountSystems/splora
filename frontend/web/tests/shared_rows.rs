//! Block and transaction views build `BlockRow` and `TxRow` from
//! `splora_frontend_shared`, not a private struct with the same fields.

fn web_source(name: &str) -> &'static str {
    match name {
        "auth.rs" => include_str!("../src/auth.rs"),
        "browser.rs" => include_str!("../src/browser.rs"),
        "lib.rs" => include_str!("../src/lib.rs"),
        "models.rs" => include_str!("../src/models.rs"),
        "paths.rs" => include_str!("../src/paths.rs"),
        "rows.rs" => include_str!("../src/rows.rs"),
        "screens.rs" => include_str!("../src/screens.rs"),
        "theme.rs" => include_str!("../src/theme.rs"),
        "views.rs" => include_str!("../src/views.rs"),
        other => panic!("unknown web source {other}"),
    }
}

fn item_body<'a>(src: &'a str, name: &str) -> &'a str {
    let marker = format!("fn {name}(");
    let start = src
        .find(&marker)
        .unwrap_or_else(|| panic!("{name} is missing"));
    let rest = &src[start..];
    let after_sig = rest.find('\n').unwrap_or(0) + 1;
    let tail = &rest[after_sig..];
    let end_in_tail = ["\nfn ", "\n#[component]", "\nconst ", "\n#[cfg"]
        .iter()
        .filter_map(|needle| tail.find(needle))
        .min()
        .unwrap_or(tail.len());
    &rest[..after_sig + end_in_tail]
}

fn imports_shared_type(src: &str, ty: &str) -> bool {
    let qualified = format!("splora_frontend_shared::{ty}");
    if src.contains(&qualified) {
        return true;
    }
    src.lines().any(|line| {
        let line = line.trim();
        line.starts_with("use splora_frontend_shared::") && line.contains(ty)
    })
}

/// `BlockPage` and `TransactionPage` render through `block_row` and `tx_row`.
/// Those builders must construct the shared row types and must read height,
/// hash (`BlockRow.id`), tx count, time (`BlockRow.timestamp`), txid, and fee
/// from them.
#[test]
fn block_and_transaction_views_construct_shared_block_row_and_tx_row() {
    let rows = web_source("rows.rs");
    let views = web_source("views.rs");
    let block_builder = item_body(rows, "block_row");
    let tx_builder = item_body(rows, "tx_row");

    let block_from = block_builder.contains("BlockRow::from");
    let tx_from = tx_builder.contains("TxRow::from");
    let block_still_copies_wire = block_builder.contains("block.height")
        || block_builder.contains("block.id")
        || block_builder.contains("block.tx_count")
        || block_builder.contains("block.timestamp");
    let tx_still_copies_wire = tx_builder.contains("tx.txid") || tx_builder.contains("tx.fee");
    assert!(
        block_from && tx_from && !block_still_copies_wire && !tx_still_copies_wire,
        "block and transaction views do not build splora_frontend_shared::BlockRow and TxRow (BlockRow::from={block_from}, wire height/hash/tx_count/timestamp still copied={block_still_copies_wire}, TxRow::from={tx_from}, wire txid/fee still copied={tx_still_copies_wire})"
    );

    assert!(
        block_builder.contains("row.height")
            && block_builder.contains("row.id")
            && block_builder.contains("row.tx_count")
            && block_builder.contains("row.timestamp"),
        "block view dropped height, hash, tx count, or time off BlockRow"
    );
    assert!(
        tx_builder.contains("row.txid") && tx_builder.contains("row.fee"),
        "transaction view dropped txid or fee off TxRow"
    );
    assert!(
        imports_shared_type(rows, "BlockRow"),
        "BlockRow is not the splora_frontend_shared type"
    );
    assert!(
        imports_shared_type(rows, "TxRow"),
        "TxRow is not the splora_frontend_shared type"
    );

    for name in [
        "auth.rs",
        "browser.rs",
        "lib.rs",
        "models.rs",
        "paths.rs",
        "rows.rs",
        "screens.rs",
        "theme.rs",
        "views.rs",
    ] {
        let src = web_source(name);
        assert!(
            !src.contains("struct BlockRow") && !src.contains("struct TxRow"),
            "{name} still defines a private BlockRow or TxRow"
        );
    }

    let block_page = item_body(views, "BlockPage");
    let blocks_page = item_body(views, "BlocksPage");
    let transaction_page = item_body(views, "TransactionPage");
    assert!(
        block_page.contains("block_rows_markup("),
        "BlockPage does not render block rows"
    );
    assert!(
        blocks_page.contains("blocks_rows_markup("),
        "BlocksPage does not render block rows"
    );
    assert!(
        transaction_page.contains("transaction_row_markup("),
        "TransactionPage does not render the transaction row"
    );
    assert!(
        item_body(rows, "block_rows_markup").contains("block_row")
            && item_body(rows, "blocks_rows_markup").contains("block_row"),
        "block views do not build rows through block_row"
    );
    assert!(
        item_body(rows, "transaction_row_markup").contains("tx_row"),
        "transaction view does not build rows through tx_row"
    );
}
