use splora_frontend_shared::rbf_page_initial;

#[test]
fn rbf_page_gets_updated_over_websockets_starts_empty() {
    let page = rbf_page_initial();
    let expected_requests = [
        "GET /api/v1/replacements",
        "GET /api/v1/fullrbf/replacements",
    ];
    assert_eq!(
        (
            page.shows_no_replacements,
            page.replacement_tree_len,
            page.requests.as_slice(),
        ),
        (true, Some(0), expected_requests.as_slice()),
        "RBF page gets updated over websockets: the page shows no replacements, the replacement tree length is 0, and the page requests GET /api/v1/replacements and GET /api/v1/fullrbf/replacements"
    );
}
