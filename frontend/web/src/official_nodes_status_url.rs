//! Browser paint of the official nodes status URL.
//! The text is `splora_frontend_shared::official_nodes_status_url`.

/// Forwards the shared official nodes status URL.
pub fn paint_official_nodes_status_url(
    page_hostname: &str,
    pathname: &str,
    host_socket: bool,
    host_host: &str,
) -> String {
    splora_frontend_shared::official_nodes_status_url(
        page_hostname,
        pathname,
        host_socket,
        host_host,
    )
}
