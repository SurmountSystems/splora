//! NIP-98 signing gate. A missing signer never calls the indexer.
//!
//! `frontend/splora-api` is the shared prefix table and header encoder.

use crate::paths::allowed_post_path;
use splora_api::{KIND_NIP98, Network, authorization_header, nip98_tags};

pub const FAIL_CLOSED: &str = "A NIP-07 signer is required. The indexer was not called.";

pub fn text_after_signed_get(result: Result<String, ClientError>) -> String {
    match result {
        Ok(text) => text,
        Err(_) => FAIL_CLOSED.to_string(),
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ClientError {
    MissingSigner,
    RejectedEvent,
}

impl ClientError {
    pub fn authorization(&self) -> Option<&str> {
        None
    }

    pub fn fetched(&self) -> bool {
        false
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UnsignedNip98 {
    pub pubkey: String,
    pub created_at: u64,
    pub kind: u16,
    pub tags: Vec<Vec<String>>,
    pub content: String,
}

pub trait Nip98Signer {
    fn public_key_hex(&self) -> Result<String, ClientError>;
    fn sign_event_json(&self, unsigned: &UnsignedNip98) -> Result<String, ClientError>;
}

pub struct MissingSigner;

impl Nip98Signer for MissingSigner {
    fn public_key_hex(&self) -> Result<String, ClientError> {
        Err(ClientError::MissingSigner)
    }

    fn sign_event_json(&self, _unsigned: &UnsignedNip98) -> Result<String, ClientError> {
        Err(ClientError::MissingSigner)
    }
}

pub struct BrowserPresence;

pub fn signer_from_presence(present: bool) -> Result<BrowserPresence, ClientError> {
    if present {
        Ok(BrowserPresence)
    } else {
        Err(ClientError::MissingSigner)
    }
}

pub fn nip98_u_url(origin: &str, indexer_path: &str) -> String {
    format!("{origin}{indexer_path}")
}

pub fn browser_fetch_url(origin: &str, network: Network, indexer_path: &str) -> String {
    format!("{origin}{}{indexer_path}", network.api_prefix())
}

pub(crate) fn unsigned_method(
    pubkey: &str,
    created_at: u64,
    origin: &str,
    indexer_path: &str,
    method: &str,
) -> UnsignedNip98 {
    let u = nip98_u_url(origin, indexer_path);
    UnsignedNip98 {
        pubkey: pubkey.to_string(),
        created_at,
        kind: KIND_NIP98,
        tags: nip98_tags(&u, method, None),
        content: String::new(),
    }
}

pub fn unsigned_get(
    pubkey: &str,
    created_at: u64,
    origin: &str,
    indexer_path: &str,
) -> UnsignedNip98 {
    unsigned_method(pubkey, created_at, origin, indexer_path, "GET")
}

pub fn fetch_indexer(
    signer: Option<&dyn Nip98Signer>,
    origin: &str,
    network: Network,
    indexer_path: &str,
    now_unix: u64,
    transport: &mut dyn FnMut(&str, &str) -> Result<Vec<u8>, ClientError>,
) -> Result<Vec<u8>, ClientError> {
    let signer = signer.ok_or(ClientError::MissingSigner)?;
    let pubkey = signer.public_key_hex()?;
    let unsigned = unsigned_get(&pubkey, now_unix, origin, indexer_path);
    let signed = signer.sign_event_json(&unsigned)?;
    let header = authorization_header(&signed).map_err(|_| ClientError::RejectedEvent)?;
    let url = browser_fetch_url(origin, network, indexer_path);
    transport(&url, &header)
}

pub fn post_indexer(
    signer: Option<&dyn Nip98Signer>,
    origin: &str,
    network: Network,
    indexer_path: &str,
    now_unix: u64,
    payload: &str,
    transport: &mut dyn FnMut(&str, &str, &str, &str) -> Result<Vec<u8>, ClientError>,
) -> Result<Vec<u8>, ClientError> {
    if payload.contains("nsec1") || !allowed_post_path(indexer_path) {
        return Err(ClientError::RejectedEvent);
    }
    let signer = signer.ok_or(ClientError::MissingSigner)?;
    let pubkey = signer.public_key_hex()?;
    let unsigned = unsigned_method(&pubkey, now_unix, origin, indexer_path, "POST");
    let signed = signer.sign_event_json(&unsigned)?;
    if signed.contains("nsec1") {
        return Err(ClientError::RejectedEvent);
    }
    let header = authorization_header(&signed).map_err(|_| ClientError::RejectedEvent)?;
    let url = browser_fetch_url(origin, network, indexer_path);
    transport("POST", &url, &header, payload)
}
