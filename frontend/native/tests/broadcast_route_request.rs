//! Screen routes `/tx/push`, `/pushtx`, and `/tx/test` paint a broadcast request.
//!
//! `paint_broadcast_route_request` forwards the route name and the raw
//! transaction into `splora_frontend_shared::broadcast_route_request` and
//! returns that request. The value is a request, not a broadcast success.
//! This test does not open a window.
//! `paint_broadcast_route_request` is not defined on `splora_native`.
//! This test does not add it.

/// Distinctive raw transaction hex. It stays in this test argument.
const RAW_TX: &str = "c3cba932ee11";

#[test]
fn paint_broadcast_route_request_equals_shared_for_three_routes() {
    for route in ["/tx/push", "/pushtx", "/tx/test"] {
        let painted = splora_native::paint_broadcast_route_request(route, RAW_TX);
        let shared = splora_frontend_shared::broadcast_route_request(route, RAW_TX);
        assert_eq!(painted, shared, "{route}");
        let request = painted.expect("route request");
        let body = request.body.as_deref().expect("body");
        assert!(
            body.contains(RAW_TX),
            "{route} body {body} must contain the raw transaction"
        );
        let rendered = format!("{request:?}");
        assert!(!rendered.contains("txid"), "{route} {rendered}");
        assert!(!rendered.contains("200"), "{route} {rendered}");
        assert_ne!(request.method, "200", "{route}");
        assert_ne!(request.path, "200", "{route}");
        assert_ne!(body, "200", "{route}");
        assert!(
            request.method.parse::<u16>().is_err(),
            "{route} method {} is a success code",
            request.method
        );
    }
}
