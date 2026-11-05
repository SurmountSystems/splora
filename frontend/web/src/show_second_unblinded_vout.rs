//! Second `.table-tx-vout` row for one `#blinded` group.
//!
//! Class, amount, and child index come from the shared row. That call keeps
//! fee at 0. This module does not parse the fragment.

/// Paints `show_second_unblinded_vout` for `fragment`.
pub fn paint_show_second_unblinded_vout(fragment: &str) -> String {
    let row = splora_frontend_shared::show_second_unblinded_vout(fragment);
    let mut body = String::new();
    let last = usize::from(row.child).max(1);
    for index in 1..=last {
        if index == usize::from(row.child) {
            body.push_str(&format!(
                "<tr class=\"{class}\"><td class=\"amount\">{amount}</td></tr>",
                class = row.row_class,
                amount = row.amount,
            ));
        } else {
            body.push_str("<tr><td class=\"amount\"></td></tr>");
        }
    }
    format!("<table class=\"table-tx-vout\"><tbody>{body}</tbody></table>")
}
