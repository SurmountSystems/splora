//! Screen routes `/tx/push`, `/pushtx`, and `/tx/test` build the indexer
//! request. The return is that request. It is not an HTTP success code
//! and it is not a broadcast result.

use splora_frontend_shared::{IndexerRequest, broadcast_route_request};

/// Distinctive raw transaction text. It stays in this test argument.
const RAW_TX: &str = "c3cba932";

#[test]
fn broadcast_route_request_returns_method_path_and_body() {
    assert_broadcast_route("/tx/push");
    assert_broadcast_route("/pushtx");
    assert_test_route("/tx/test");
}

fn assert_broadcast_route(route: &str) {
    let built = request(route);
    assert_eq!(built.method, "POST", "{route}");
    assert_eq!(built.path, "/tx", "{route}");
    let body = built.body.as_deref().expect("body");
    assert!(body.contains(RAW_TX), "{route} {body}");
    assert_eq!(body, RAW_TX, "{route}");
    refuse_success_and_txid(&built, body);
}

fn assert_test_route(route: &str) {
    let built = request(route);
    assert_eq!(built.method, "POST", "{route}");
    assert_eq!(built.path, "/txs/test", "{route}");
    let body = built.body.as_deref().expect("body");
    assert!(body.contains(RAW_TX), "{route} {body}");
    assert_eq!(body, format!("[\"{RAW_TX}\"]"), "{route}");
    refuse_success_and_txid(&built, body);
}

fn request(route: &str) -> IndexerRequest {
    let built = broadcast_route_request(route, RAW_TX);
    let rendered = format!("{built:?}");
    assert_ne!(rendered, "200", "{route}");
    assert!(!rendered.contains("txid"), "{route} {rendered}");
    built.expect("route request")
}

fn refuse_success_and_txid(built: &IndexerRequest, body: &str) {
    assert_ne!(built.method, "200");
    assert_ne!(built.path, "200");
    assert_ne!(body, "200");
    assert!(built.method.parse::<u16>().is_err());
    let rendered = format!("{built:?}");
    assert!(!rendered.contains("txid"), "{rendered}");
    assert!(!rendered.contains("200"), "{rendered}");
    assert!(!rendered.contains("success"), "{rendered}");
}
