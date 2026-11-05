//! `/signet` is HTTP 307 to `/mutinynet`.
//!
//! Calls the shipped prefix table. Does not call an indexer and does not
//! treat signet as a backend.

use splora_api::{
    SIGNET_PATH, SIGNET_STATUS, is_backend_network, signet_is_backend, signet_redirect,
};

#[test]
fn signet_is_http_307_to_mutinynet() {
    assert_eq!(SIGNET_PATH, "/signet");
    assert_eq!(SIGNET_STATUS, 307);
    assert!(!signet_is_backend());
    assert!(!is_backend_network("signet"));
    assert!(!is_backend_network(SIGNET_PATH));
    assert!(is_backend_network("mutinynet"));

    let (status, location) = signet_redirect();
    assert_eq!(location, "/mutinynet");
    assert_eq!(status, 307);
}
