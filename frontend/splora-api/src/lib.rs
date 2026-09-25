//! Prefix table and NIP-98 kind 27235 header builder for splora.
//! Signing stays outside this crate. No secret key is accepted.

mod nip98;
mod prefix;

pub use nip98::{KIND_NIP98, Nip98Error, authorization_header, nip98_tags};
pub use prefix::{
    BASE_URL, Network, SIGNET_PATH, SIGNET_STATUS, api_prefix, api_root, is_backend_network,
    signet_is_backend,
};
