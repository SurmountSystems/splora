use splora_frontend_shared::after_next_page_click_supports_pagination_on_the_block_screen;

#[test]
fn supports_pagination_on_the_block_screen() {
    let view = after_next_page_click_supports_pagination_on_the_block_screen();
    assert!(
        view.later_page_header_differs,
        "on the block screen, the next pagination control changes the visible transaction header"
    );
}
