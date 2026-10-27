//! Shipped `fetch_indexer` against routes the indexer NIP-98 gate checks.
//!
//! `verify_nip98` is `electrs::auth::verify_nip98` in `src/auth.rs`.
//! This file calls that function. It does not copy it.

use std::cell::RefCell;
use std::sync::Arc;

use electrs::auth::{Allowlist, AuthError, verify_nip98};
use nostr::event::{EventBuilder, FinalizeEvent, Kind, Tag};
use nostr::key::Keys;
use nostr::types::Timestamp;
use splora_api::{Network, authorization_header};
use splora_frontend_shared::blocks_tip_hash_path;
use splora_web::{ClientError, Nip98Signer, UnsignedNip98, fetch_indexer, nip98_u_url, tx_path};

/// Same window the indexer gate passes to `verify_nip98`.
const WINDOW_SECS: u64 = 60;

const ORIGIN: &str = "https://splora.surmount.systems";
const NOW: u64 = 1_700_000_000;
const TXID: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";

struct RecordingSigner {
    keys: Keys,
    unsigned: RefCell<Option<UnsignedNip98>>,
    signed_json: RefCell<Option<String>>,
}

impl Nip98Signer for RecordingSigner {
    fn public_key_hex(&self) -> Result<String, ClientError> {
        Ok(self.keys.public_key().to_hex())
    }

    fn sign_event_json(&self, unsigned: &UnsignedNip98) -> Result<String, ClientError> {
        let mut builder = EventBuilder::new(Kind::from(unsigned.kind), unsigned.content.clone())
            .custom_created_at(Timestamp::from(unsigned.created_at));
        for tag in &unsigned.tags {
            let parsed = Tag::parse(tag.iter().map(String::as_str))
                .map_err(|_| ClientError::RejectedEvent)?;
            builder = builder.tag(parsed);
        }
        let event = builder
            .finalize(&self.keys)
            .map_err(|_| ClientError::RejectedEvent)?;
        let json = event.as_json();
        if json.contains("nsec1") {
            return Err(ClientError::RejectedEvent);
        }
        *self.unsigned.borrow_mut() = Some(unsigned.clone());
        *self.signed_json.borrow_mut() = Some(json.clone());
        Ok(json)
    }
}

fn new_signer() -> RecordingSigner {
    RecordingSigner {
        keys: Keys::generate(),
        unsigned: RefCell::new(None),
        signed_json: RefCell::new(None),
    }
}

fn tag<'a>(event: &'a UnsignedNip98, name: &str) -> &'a str {
    event
        .tags
        .iter()
        .find(|row| row.first().map(String::as_str) == Some(name))
        .and_then(|row| row.get(1))
        .map(String::as_str)
        .unwrap_or_else(|| panic!("missing {name} tag"))
}

struct Call {
    url: String,
    authorization: String,
}

fn fetch_once(path: &str, signer: &RecordingSigner) -> Call {
    let mut calls = 0u32;
    let mut seen = None;
    let body = fetch_indexer(
        Some(signer),
        ORIGIN,
        Network::Mainnet,
        path,
        NOW,
        &mut |url, authorization| {
            calls += 1;
            seen = Some(Call {
                url: url.to_string(),
                authorization: authorization.to_string(),
            });
            Ok(b"ok".to_vec())
        },
    )
    .expect("present signer returns the transport body");
    assert_eq!(body, b"ok");
    assert_eq!(calls, 1, "indexer transport ran more than once");
    seen.expect("transport was called")
}

fn assert_gated_get(call: &Call, signed_json: &str, indexer_path: &str, unsigned: &UnsignedNip98) {
    // The public front strips `/api` before the indexer builds the absolute URL.
    let u = nip98_u_url(ORIGIN, indexer_path);
    assert_eq!(u, format!("{ORIGIN}{indexer_path}"));
    assert!(
        !u.contains("/api/"),
        "u must be the stripped indexer URL, got {u}"
    );
    assert_eq!(unsigned.kind, 27235);
    assert_eq!(unsigned.created_at, NOW);
    assert_eq!(unsigned.content, "");
    assert_eq!(tag(unsigned, "u"), u);
    assert_eq!(tag(unsigned, "method"), "GET");
    assert!(
        unsigned
            .tags
            .iter()
            .all(|row| row.first().map(String::as_str) != Some("payload")),
        "GET has no payload tag"
    );
    let header = authorization_header(signed_json).expect("shipped header encoder");
    assert_eq!(call.authorization, header);
    assert!(call.authorization.starts_with("Nostr "));
    assert!(!call.authorization.contains("nsec"));
    assert!(!signed_json.contains("nsec1"));
    let event: serde_json::Value = serde_json::from_str(signed_json).expect("event json");
    assert_eq!(event["kind"], 27235);
    assert_eq!(event["tags"][0][1], u);
    assert_eq!(event["tags"][1][1], "GET");
    let fetch = format!("{ORIGIN}{}{indexer_path}", Network::Mainnet.api_prefix());
    assert_eq!(call.url, fetch);
    assert_ne!(call.url, u);
}

#[test]
fn missing_signer_does_not_call_gated_get_tx() {
    let path = tx_path(TXID);
    assert_eq!(path, format!("/tx/{TXID}"));
    let mut calls = 0u32;
    let result = fetch_indexer(
        None,
        ORIGIN,
        Network::Mainnet,
        &path,
        NOW,
        &mut |_url, _authorization| {
            calls += 1;
            Ok(Vec::new())
        },
    );
    assert!(matches!(result, Err(ClientError::MissingSigner)));
    assert_eq!(calls, 0, "missing signer called the indexer");
}

#[test]
fn present_signer_header_binds_gated_get_tx_and_tip_hash() {
    let tx = tx_path(TXID);
    let tip = blocks_tip_hash_path();
    assert_eq!(tip, "/blocks/tip/hash");

    let signer = new_signer();
    let tx_call = fetch_once(&tx, &signer);
    let tx_unsigned = signer.unsigned.borrow().clone().expect("tx unsigned");
    let tx_json = signer.signed_json.borrow().clone().expect("tx json");
    assert_gated_get(&tx_call, &tx_json, &tx, &tx_unsigned);

    let signer = new_signer();
    let tip_call = fetch_once(tip, &signer);
    let tip_unsigned = signer.unsigned.borrow().clone().expect("tip unsigned");
    let tip_json = signer.signed_json.borrow().clone().expect("tip json");
    assert_gated_get(&tip_call, &tip_json, tip, &tip_unsigned);

    assert_ne!(tag(&tx_unsigned, "u"), tag(&tip_unsigned, "u"));
    assert_ne!(tx_call.authorization, tip_call.authorization);
    println!("tx_fetch_url={}", tx_call.url);
    println!("tx_u={}", tag(&tx_unsigned, "u"));
    println!("tx_authorization={}", tx_call.authorization);
    println!("tip_fetch_url={}", tip_call.url);
    println!("tip_u={}", tag(&tip_unsigned, "u"));

    let u = tag(&tx_unsigned, "u");
    let header = tx_call.authorization.as_str();
    let allow = allowlist_file(&tx_unsigned.pubkey);
    let empty = Allowlist::deny_all();
    let other = nip98_u_url(
        ORIGIN,
        "/tx/bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
    );
    assert_ne!(other, u);

    let accepted = verify_nip98(
        Some(header),
        "GET",
        u,
        b"",
        NOW,
        WINDOW_SECS,
        allow.as_ref(),
    );
    let pk = accepted.expect("verifier accept for an allowlisted key");
    assert!(
        allow.snapshot().contains(&pk),
        "verifier accept for an allowlisted key"
    );
    println!("verifier accept for an allowlisted key");

    let empty_err = verify_nip98(
        Some(header),
        "GET",
        u,
        b"",
        NOW,
        WINDOW_SECS,
        empty.as_ref(),
    )
    .expect_err("verify_nip98 reject when the allowlist is empty");
    assert_eq!(
        empty_err,
        AuthError::NotAllowlisted,
        "verify_nip98 reject when the allowlist is empty"
    );
    println!("verify_nip98 reject when the allowlist is empty");

    let mismatch = verify_nip98(
        Some(header),
        "GET",
        &other,
        b"",
        NOW,
        WINDOW_SECS,
        allow.as_ref(),
    )
    .expect_err("verify_nip98 reject when the u tag does not match the indexer URL");
    assert_eq!(
        mismatch,
        AuthError::UrlMismatch,
        "verify_nip98 reject when the u tag does not match the indexer URL"
    );
    println!("verify_nip98 reject when the u tag does not match the indexer URL");
}

fn allowlist_file(pubkey_hex: &str) -> Arc<Allowlist> {
    let dir = std::env::temp_dir().join(format!(
        "splora-web-nip98-{}-{}",
        std::process::id(),
        pubkey_hex
    ));
    std::fs::create_dir_all(&dir).expect("allowlist dir");
    let path = dir.join("allowlist");
    std::fs::write(&path, format!("{pubkey_hex}\n")).expect("allowlist file");
    Allowlist::load(&path).expect("load allowlist")
}
