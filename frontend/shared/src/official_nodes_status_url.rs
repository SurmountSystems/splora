//! Official nodes status URL built in the browser.
//!
//! The last six characters of the pathname are the route suffix.
//! This function does not use the network.

/// `https://` plus the page host or the configured host, the pathname
/// without its last six characters, and `/status`.
///
/// A socket host uses `page_hostname`. Otherwise the host is the
/// hostname of `host_host`, with scheme, userinfo, port, and path removed.
pub fn official_nodes_status_url(
    page_hostname: &str,
    pathname: &str,
    host_socket: bool,
    host_host: &str,
) -> String {
    let host = if host_socket {
        page_hostname
    } else {
        hostname_of(host_host)
    };
    let subpath = route_subpath(pathname);
    format!("https://{host}{subpath}/status")
}

/// Drop the last six characters when the pathname is long enough.
/// Shorter paths contribute an empty prefix. The cut is by length, not
/// by searching for a route word.
fn route_subpath(pathname: &str) -> &str {
    let count = pathname.chars().count();
    if count < 6 {
        return "";
    }
    let end = pathname
        .char_indices()
        .nth(count - 6)
        .map(|(index, _)| index)
        .unwrap_or(pathname.len());
    &pathname[..end]
}

/// Hostname only: skip a `://` scheme, skip userinfo through `@`, then
/// stop at `/`, `?`, `#`, or a port `:`.
fn hostname_of(host_host: &str) -> &str {
    let after_scheme = match host_host.find("://") {
        Some(index) => &host_host[index + 3..],
        None => host_host,
    };
    let after_userinfo = match after_scheme.find('@') {
        Some(index) => &after_scheme[index + 1..],
        None => after_scheme,
    };
    let end = after_userinfo
        .find(|unit: char| unit == '/' || unit == '?' || unit == '#' || unit == ':')
        .unwrap_or(after_userinfo.len());
    &after_userinfo[..end]
}
