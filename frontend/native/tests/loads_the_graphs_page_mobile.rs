#[test]
fn loads_the_graphs_page_mobile() {
    // paint_loads_the_graphs_page_mobile is not defined yet.
    let painted = splora_native::paint_loads_the_graphs_page_mobile("iphone-6");
    let shared = splora_frontend_shared::loads_the_graphs_page_mobile("iphone-6");
    assert!(!painted.tv_only_exists);
    assert_eq!(painted, shared);
}
