//! RBF page after a websocket fixture arrives.
//!
//! Cypress it "RBF page gets updated over websockets" delivers
//! `rbf_page/rbf_01.json`, then `rbf_page/rbf_02.json`. Native paints
//! `screen=rbf tree_count=N`, where N is `rbf_socket_trees` on that body
//! (`rbfLatest` length). This test delivers those bodies to the web render.
//! It does not reimplement that count.

use splora_frontend_shared::rbf_socket_trees;
use splora_web::rbf_page_socket_markup;

const RBF_01: &str =
    include_str!("../../../ref/mempool/frontend/cypress/fixtures/rbf_page/rbf_01.json");
const RBF_02: &str =
    include_str!("../../../ref/mempool/frontend/cypress/fixtures/rbf_page/rbf_02.json");

fn assert_socket_tree_count(body: &str, which: &str) {
    let tree_len = rbf_socket_trees(body)
        .unwrap_or_else(|err| panic!("{which} fixture: {err}"))
        .tree_len;
    let painted = rbf_page_socket_markup(body);
    assert_eq!(
        painted,
        format!("screen=rbf tree_count={tree_len}"),
        "socket tree count is not rbf_socket_trees({which}).tree_len ({tree_len}): {painted}"
    );
}

#[test]
fn rbf_page_gets_updated_over_websockets_tree_count_from_rbf_socket_trees() {
    assert_socket_tree_count(RBF_01, "rbf_01");
    assert_socket_tree_count(RBF_02, "rbf_02");
}
