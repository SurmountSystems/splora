#[test]
fn check_buttons_tablet() {
    let painted = splora_native::paint_check_buttons_tablet("ipad-2");
    let shared = splora_frontend_shared::check_buttons_tablet("ipad-2");
    assert_eq!(
        painted.small_buttons_second_child_visible,
        shared.small_buttons_second_child_visible
    );
    assert_eq!(painted.dropdown_fees_visible, shared.dropdown_fees_visible);
    assert_eq!(painted.btn_group_visible, shared.btn_group_visible);
    assert!(painted.small_buttons_second_child_visible);
    assert!(painted.dropdown_fees_visible);
    assert!(painted.btn_group_visible);
}
