//! First `.table-tx-vout` row for one `#blinded` group.
//!
//! Class and amount come from the shared row. That call keeps fee at 0.
//! This module does not parse the fragment.

/// Paints `show_first_unblinded_vout` for `fragment`.
pub fn paint_show_first_unblinded_vout(fragment: &str) -> String {
    let row = splora_frontend_shared::show_first_unblinded_vout(fragment);
    format!(
        "<table class=\"table-tx-vout\"><tbody><tr class=\"{class}\"><td class=\"amount\">{amount}</td></tr></tbody></table>",
        class = row.row_class,
        amount = row.amount,
    )
}
