#[test]
fn check_buttons_desktop() {
    let painted = splora_native::paint_check_buttons_desktop("macbook-16");
    let shared = splora_frontend_shared::check_buttons_desktop("macbook-16");
    assert_eq!(painted, shared);
    assert!(painted.small_buttons_second_child_visible);
    assert!(painted.dropdown_fees_visible);
    assert!(painted.btn_group_visible);
}
