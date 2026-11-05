//! Web paint of the signet dashboard redirect.
//! Status and location come from `splora_api::signet_redirect`.

/// Forwards the shipped signet redirect: HTTP status and location.
pub fn paint_signet_redirect() -> (u16, &'static str) {
    splora_api::signet_redirect()
}
