pub fn details_open_after_click(open: bool) -> bool {
    !open
}

/// Block details after one click of the outline info button.
///
/// `open` is whether details are visible before the click. The next state is
/// `details_open_after_click`. Visible details use query `true` and fragment
/// `details`. Hidden details use query `false` and fragment `block`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BlockDetailsAfterClick {
    pub details_visible: bool,
    pub show_details: bool,
    pub query_show_details: &'static str,
    pub fragment: &'static str,
}

pub fn expands_and_collapses_the_block_details(open: bool) -> BlockDetailsAfterClick {
    let show_details = details_open_after_click(open);
    BlockDetailsAfterClick {
        details_visible: show_details,
        show_details,
        query_show_details: if show_details { "true" } else { "false" },
        fragment: if show_details { "details" } else { "block" },
    }
}
