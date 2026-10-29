use splora_frontend_shared::dashboard_page_initial;

#[test]
fn dashboard_loads_hides_skeleton_and_requests_six_gets() {
    let page = dashboard_page_initial();
    let expected_requests = [
        "GET /blocks",
        "GET /blocks/tip/hash",
        "GET /blocks/tip/height",
        "GET /fee-estimates",
        "GET /mempool",
        "GET /mempool/recent",
    ];
    assert_eq!(
        (page.skeleton_visible, page.requests.as_slice()),
        (false, expected_requests.as_slice())
    );
}
