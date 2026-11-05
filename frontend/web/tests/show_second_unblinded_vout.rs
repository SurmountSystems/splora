//! Cypress it() title: show second unblinded vout.
//!
//! The spec visits one transaction whose `#blinded` fragment is a single
//! group. The first field is 2364760 satoshis. Child 2 of `.table-tx-vout`
//! has class `assetBox`, and that row's `.amount` is `sats_to_btc(2364760)`
//! plus ` LBTC`. The first-vout fragment is the shared test's 100000
//! satoshi group. An empty fragment does not put `assetBox` on child 2.
//! This file does not fetch a transaction.

use splora_frontend_shared::sats_to_btc;

/// One `#blinded` group from the Cypress visit. The first field is 2364760.
const SECOND: &str = "2364760,6f0279e9ed041c3d710a9f57d0c02928416460c4b722ae3457a11eec381c526d,90c7a43b15b905bca045ca42a01271cfe71d2efe3133f4197792c24505cb32ed,12eb5959d9293b8842e7dd8bc9aa9639fd3fd031c5de3ba911adeca94eb57a3a";

/// First-vout fragment from the shared test. The first field is 100000.
const FIRST_VOUT: &str = "100000,6f0279e9ed041c3d710a9f57d0c02928416460c4b722ae3457a11eec381c526d,0ab9f70650f16b1db8dfada05237f7d0d65191c3a13183da8a2ddddfbde9a2ad,fd98b2edc5530d76acd553f206a431f4c1fab27e10e290ad719582af878e98fc";

#[test]
fn show_second_unblinded_vout() {
    let markup = splora_web::paint_show_second_unblinded_vout(SECOND);
    let row = nth_table_tx_vout_row(&markup, 2);
    assert!(
        tr_has_class(row, "assetBox"),
        "cy.get('.table-tx-vout tr:nth-child(2)').should('have.class', 'assetBox'), got {row}"
    );
    let amount = amount_text(row);
    let expected = format!("{} LBTC", sats_to_btc(2364760));
    assert_eq!(
        amount, expected,
        "child 2 amount equals sats_to_btc(2364760) plus \" LBTC\""
    );
    assert!(
        amount.contains("0.02364760 LBTC"),
        "cy.get('.table-tx-vout tr:nth-child(2) .amount').should('contain.text', '0.02364760 LBTC'), got {amount}"
    );

    let first = splora_web::paint_show_second_unblinded_vout(FIRST_VOUT);
    let first_amount = amount_text(nth_table_tx_vout_row(&first, 2));
    assert!(
        first_amount.contains("0.00100000 LBTC"),
        "child 2 amount for the first-vout fragment, got {first_amount}"
    );
    assert!(
        !first_amount.contains("0.02364760"),
        "child 2 amount must not contain 0.02364760, got {first_amount}"
    );

    let empty = splora_web::paint_show_second_unblinded_vout("");
    let empty_row = nth_table_tx_vout_row(&empty, 2);
    assert!(
        !tr_has_class(empty_row, "assetBox"),
        "empty fragment must not put class assetBox on child 2, got {empty_row}"
    );
}

fn nth_table_tx_vout_row(markup: &str, child: usize) -> &str {
    let vout_at = markup
        .find("class=\"table-tx-vout\"")
        .expect("table-tx-vout missing");
    let mut rest = &markup[vout_at..];
    for index in 1..=child {
        let row_at = rest
            .find("<tr")
            .unwrap_or_else(|| panic!("child {index} missing"));
        rest = &rest[row_at..];
        if index == child {
            let end = rest
                .find("</tr>")
                .map(|offset| offset + "</tr>".len())
                .unwrap_or(rest.len());
            return &rest[..end];
        }
        rest = &rest["<tr".len()..];
    }
    unreachable!("nth-child loop");
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
