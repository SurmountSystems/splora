//! Native NIP-98 against this project's indexer HTTP URLs.
//!
//! `GET /tx/:txid` is an allowlisted indexer gate. Tip hash and tip height are
//! the public-health exception and are not this test. A nostr relay is not.

use std::io::Write;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use electrs::auth::{Allowlist, AuthError, verify_nip98};
use splora_api::{KIND_NIP98, Network, api_root};
use splora_frontend_shared::tx_path;
use splora_native::{
    ClientError, IndexerAuthCall, SignerError, call_indexer_with_signer, sign_http,
};

/// Same window `http_ws_auth_gate` passes to `verify_nip98`.
const WINDOW_SECS: u64 = 60;

const TXID: &str = "abc";

fn indexer_tx_url() -> String {
    format!("{}{}", api_root(Network::Mainnet), tx_path(TXID))
}

#[test]
fn native_nip98_missing_signer_skips_indexer_and_present_signer_sends_kind_27235() {
    let url = indexer_tx_url();
    assert_eq!(
        url, "https://splora.surmount.systems/api/tx/abc",
        "the signed URL is this project's indexer, not a nostr relay"
    );
    assert!(url.starts_with("https://splora.surmount.systems/"));
    assert!(!url.contains("wss://"));
    assert!(!url.contains("relay"));

    let mut missing_calls = 0u32;
    let missing = call_indexer_with_signer(None, &url, "GET", None, &mut |_call| {
        missing_calls += 1;
        Ok::<String, ClientError>(String::new())
    });
    assert!(
        matches!(missing, Err(SignerError::MissingSigner)),
        "a missing signer is a typed error, got {missing:?}"
    );
    assert_eq!(
        missing_calls, 0,
        "a missing signer must not call the indexer"
    );

    let keys = nostr::key::Keys::generate();
    let mut seen: Option<IndexerAuthCall> = None;
    let (signed, body) = call_indexer_with_signer(Some(&keys), &url, "GET", None, &mut |call| {
        seen = Some(call.clone());
        Ok("indexer-ok".to_string())
    })
    .expect("a present signer calls the indexer");
    assert_eq!(body, "indexer-ok");
    let call = seen.expect("the indexer transport ran once");
    assert_eq!(call.method, "GET");
    assert_eq!(call.url, url);
    assert_eq!(call.authorization, signed.authorization());
    assert!(call.authorization.starts_with("Nostr "));
    assert!(!call.authorization.contains("nsec1"));
    assert!(!signed.event_json().contains("nsec1"));

    assert_eq!(signed.kind(), 27235);
    assert_eq!(signed.kind(), KIND_NIP98);
    assert_eq!(signed.tag_value("u"), Some(url.as_str()));
    assert_eq!(signed.tag_value("method"), Some("GET"));
    assert!(signed.content().is_empty());
    signed.verify().expect("kind 27235 signature");

    let direct = sign_http(&keys, &url, "GET", None).expect("shipped signer");
    assert_eq!(direct.kind(), 27235);
    assert_eq!(direct.tag_value("u"), Some(url.as_str()));
    assert_eq!(direct.tag_value("method"), Some("GET"));
    assert!(direct.authorization().starts_with("Nostr "));

    let mut relay_calls = 0u32;
    let relay = call_indexer_with_signer(
        Some(&keys),
        "wss://relay.example/nostr",
        "GET",
        None,
        &mut |_call| {
            relay_calls += 1;
            Ok(String::new())
        },
    );
    assert!(
        matches!(relay, Err(SignerError::Url)),
        "a nostr relay is not an indexer URL, got {relay:?}"
    );
    assert_eq!(relay_calls, 0, "a relay URL must not call the indexer");

    let npub = nostr::nips::nip19::ToBech32::to_bech32(&keys.public_key()).expect("npub");
    let allow = allowlist_file(&[&npub]);
    let empty = allowlist_file(&[]);
    let now = now_unix();
    let header = signed.authorization();
    let other_url = format!("{}{}", api_root(Network::Mainnet), tx_path("def"));
    assert_ne!(other_url, url);

    let accepted = verify_nip98(
        Some(header),
        "GET",
        &url,
        b"",
        now,
        WINDOW_SECS,
        allow.as_ref(),
    );
    let pk = accepted.expect("the allowlisted key was accepted");
    assert_eq!(
        pk,
        *keys.public_key().as_bytes(),
        "the allowlisted key was accepted"
    );
    record_verifier_line("NIP98_VERIFY_ACCEPT");

    let empty_err = verify_nip98(
        Some(header),
        "GET",
        &url,
        b"",
        now,
        WINDOW_SECS,
        empty.as_ref(),
    )
    .expect_err("the empty allowlist was rejected");
    assert_eq!(
        empty_err,
        AuthError::NotAllowlisted,
        "the empty allowlist was rejected"
    );
    record_verifier_line("NIP98_VERIFY_REJECT_EMPTY_ALLOWLIST");

    let mismatch = verify_nip98(
        Some(header),
        "GET",
        &other_url,
        b"",
        now,
        WINDOW_SECS,
        allow.as_ref(),
    )
    .expect_err("the `u` tag mismatch was rejected");
    assert_eq!(
        mismatch,
        AuthError::UrlMismatch,
        "the `u` tag mismatch was rejected"
    );
    record_verifier_line("NIP98_VERIFY_REJECT_U_TAG");
}

fn now_unix() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_secs()
}

fn allowlist_file(npubs: &[&str]) -> Arc<Allowlist> {
    use std::sync::atomic::{AtomicU64, Ordering};
    static N: AtomicU64 = AtomicU64::new(0);
    let n = N.fetch_add(1, Ordering::Relaxed);
    let dir = std::env::temp_dir().join(format!("splora-native-nip98-{}-{n}", std::process::id()));
    std::fs::create_dir_all(&dir).expect("allowlist dir");
    let path = dir.join("allowlist");
    let mut body = String::new();
    for npub in npubs {
        body.push_str(npub);
        body.push('\n');
    }
    std::fs::write(&path, body).expect("allowlist file");
    Allowlist::load(&path).expect("load allowlist")
}

fn record_verifier_line(line: &str) {
    println!("{line}");
    let path = "/tmp/grok-goal-48e4d82c673f/implementer/nostr-ui-native.txt";
    if let Ok(mut file) = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
    {
        let _ = writeln!(file, "{line}");
    }
}
