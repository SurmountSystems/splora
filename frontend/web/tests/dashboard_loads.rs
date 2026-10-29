//! Dashboard opening state after the six indexer GETs are named.
//!
//! `dashboard_page_initial` is the shared page value. The skeleton is not
//! visible, and the request list is exactly those six GETs.

#[test]
fn dashboard_loads_hides_skeleton_and_requests_six_gets() {
    let page = splora_frontend_shared::dashboard_page_initial();
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
