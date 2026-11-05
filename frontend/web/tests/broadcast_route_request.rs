//! The web paint forwards a screen route and a raw transaction into the
//! shared broadcast request. The return is that request. It is not an HTTP
//! status and it is not a broadcast result.

/// Distinctive raw transaction text. It stays in this test argument.
const RAW_TX: &str = "e4b7c3cb91a0d2f6";

const ROUTES: [&str; 3] = ["/tx/push", "/pushtx", "/tx/test"];

#[test]
fn broadcast_route_request_paint_equals_shared_for_each_route() {
    for route in ROUTES {
        let painted = splora_web::paint_broadcast_route_request(route, RAW_TX);
        let shared = splora_frontend_shared::broadcast_route_request(route, RAW_TX);
        assert_eq!(painted, shared, "{route}");
        let built = painted.expect("route request");
        let body = built.body.as_deref().expect("body");
        assert!(body.contains(RAW_TX), "{route} {body}");
        refuse_status_and_txid(&built, body, route);
    }
}

fn refuse_status_and_txid(built: &splora_frontend_shared::IndexerRequest, body: &str, route: &str) {
    assert_ne!(built.method, "200", "{route}");
    assert_ne!(built.path, "200", "{route}");
    assert_ne!(body, "200", "{route}");
    assert!(built.method.parse::<u16>().is_err(), "{route}");
    let rendered = format!("{built:?}");
    assert_ne!(rendered, "200", "{route}");
    assert!(!rendered.contains("txid"), "{route} {rendered}");
    assert!(!rendered.contains("200"), "{route} {rendered}");
    assert!(!rendered.contains("success"), "{route} {rendered}");
}
