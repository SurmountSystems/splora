/// Forwards the route name and the raw transaction into the shared request.
pub fn paint_broadcast_route_request(
    route: &str,
    raw_tx: &str,
) -> Option<splora_frontend_shared::IndexerRequest> {
    splora_frontend_shared::broadcast_route_request(route, raw_tx)
}
