//! Cypress it() title: show first unblinded vout.
//!
//! The spec visits one transaction whose `#blinded` fragment is a single
//! group. The skeleton wait is not an assertion. The first `.table-tx-vout`
//! row has class `assetBox`, and that row's `.amount` contains
//! `0.00100000 LBTC`. This file does not assert a block hash, a fee, a
//! price, or the heading "1 transaction".

/// One complete `#blinded` group from the Cypress visit.
const BLINDED: &str = "100000,6f0279e9ed041c3d710a9f57d0c02928416460c4b722ae3457a11eec381c526d,0ab9f70650f16b1db8dfada05237f7d0d65191c3a13183da8a2ddddfbde9a2ad,fd98b2edc5530d76acd553f206a431f4c1fab27e10e290ad719582af878e98fc";

#[test]
fn show_first_unblinded_vout() {
    let markup = splora_web::paint_show_first_unblinded_vout(BLINDED);
    let row = first_table_tx_vout_row(&markup);
    assert!(
        tr_has_class(row, "assetBox"),
        "cy.get('.table-tx-vout tr:nth-child(1)').should('have.class', 'assetBox'), got {row}"
    );
    let amount = amount_text(row);
    assert!(
        amount.contains("0.00100000 LBTC"),
        "cy.get('.table-tx-vout tr:nth-child(1) .amount').should('contain.text', '0.00100000 LBTC'), got {amount}"
    );
}

fn first_table_tx_vout_row(markup: &str) -> &str {
    let vout_at = markup
        .find("class=\"table-tx-vout\"")
        .expect("table-tx-vout missing");
    let vout = &markup[vout_at..];
    let row_at = vout.find("<tr").expect("first vout row missing");
    let row = &vout[row_at..];
    let end = row
        .find("</tr>")
        .map(|index| index + 5)
        .unwrap_or(row.len());
    &row[..end]
}

fn tr_has_class(row: &str, class_name: &str) -> bool {
    let Some(tag_end) = row.find('>') else {
        return false;
    };
    let tag = &row[..tag_end];
    let Some(class_at) = tag.find("class=\"") else {
        return false;
    };
    let class_value = &tag[class_at + "class=\"".len()..];
    let Some(end) = class_value.find('"') else {
        return false;
    };
    class_value[..end]
        .split_whitespace()
        .any(|class| class == class_name)
}

fn amount_text(row: &str) -> &str {
    let key = "class=\"amount\"";
    let at = row.find(key).expect("amount missing");
    let after = &row[at + key.len()..];
    let start = after.find('>').expect("amount tag") + 1;
    let rest = &after[start..];
    let end = rest.find('<').unwrap_or(rest.len());
    &rest[..end]
}
