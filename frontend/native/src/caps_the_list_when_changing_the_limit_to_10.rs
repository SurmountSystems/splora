//! Native render of the recent list at the default cap and after limit-10.
//! Visible ids come from the shared cap function. This file does not open a window.

/// Rows at the default list cap, then rows after the limit-10 control.
///
/// Both lists are the ids the shared function returns for `batches`.
pub fn paint_caps_the_list_when_changing_the_limit_to_10<'a>(
    batches: &[[&'a str; 6]; 9],
) -> (Vec<&'a str>, Vec<&'a str>) {
    splora_frontend_shared::caps_the_list_when_changing_the_limit_to_10(batches)
}
