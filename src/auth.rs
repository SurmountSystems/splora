// SPDX-License-Identifier: Unlicense
//! Read-only npub allowlist and NIP-98 HTTP auth.
//!
//! NIP-98: https://github.com/nostr-protocol/nips/blob/master/98.md (accessed: 2026-08-28)

use notify::{EventKind, RecommendedWatcher, RecursiveMode, Watcher, event::ModifyKind};
use sha2::{Digest, Sha256};
use signal_hook::consts::SIGHUP;
use std::collections::HashSet;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};
use std::thread;
use std::time::Duration;

/// Parsed allowlist of 32-byte x-only pubkeys.
#[derive(Debug)]
pub struct Allowlist {
    path: PathBuf,
    keys: RwLock<HashSet<[u8; 32]>>,
}

/// Handle that keeps the inotify / SIGHUP reload thread alive.
pub struct AllowlistWatch {
    _watcher: RecommendedWatcher,
    _hup: thread::JoinHandle<()>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AuthError {
    MissingHeader,
    BadEncoding,
    BadJson,
    BadSignature,
    WrongKind,
    UrlMismatch,
    MethodMismatch,
    Expired,
    PayloadMismatch,
    NotAllowlisted,
    Io(String),
}

impl std::fmt::Display for AuthError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AuthError::MissingHeader => write!(f, "missing Authorization header"),
            AuthError::BadEncoding => write!(f, "bad Authorization encoding"),
            AuthError::BadJson => write!(f, "bad Authorization JSON"),
            AuthError::BadSignature => write!(f, "bad NIP-98 signature"),
            AuthError::WrongKind => write!(f, "NIP-98 kind is not 27235"),
            AuthError::UrlMismatch => write!(f, "NIP-98 u tag does not match URL"),
            AuthError::MethodMismatch => write!(f, "NIP-98 method tag does not match"),
            AuthError::Expired => write!(f, "NIP-98 created_at outside window"),
            AuthError::PayloadMismatch => write!(f, "NIP-98 payload does not match body"),
            AuthError::NotAllowlisted => write!(f, "pubkey is not on the allowlist"),
            AuthError::Io(s) => write!(f, "allowlist io: {}", s),
        }
    }
}

impl std::error::Error for AuthError {}

impl From<io::Error> for AuthError {
    fn from(e: io::Error) -> Self {
        AuthError::Io(e.to_string())
    }
}

impl Allowlist {
    /// Nobody is authorized. Used when `--allow-npubs-file` is omitted.
    /// Localhost is not an exception.
    pub fn deny_all() -> Arc<Self> {
        Arc::new(Allowlist {
            path: PathBuf::new(),
            keys: RwLock::new(HashSet::new()),
        })
    }

    /// Load the allowlist file. Missing path is a hard error. Empty file is an empty set.
    pub fn load(path: &Path) -> Result<Arc<Self>, AuthError> {
        if !path.exists() {
            return Err(AuthError::Io(format!(
                "allowlist file missing: {}",
                path.display()
            )));
        }
        let keys = parse_allowlist_file(path)?;
        Ok(Arc::new(Allowlist {
            path: path.to_path_buf(),
            keys: RwLock::new(keys),
        }))
    }

    pub fn contains(&self, pubkey32: &[u8; 32]) -> bool {
        self.keys.read().expect("allowlist lock").contains(pubkey32)
    }

    pub fn snapshot(&self) -> HashSet<[u8; 32]> {
        self.keys.read().expect("allowlist lock").clone()
    }

    pub fn reload(&self) -> Result<(), AuthError> {
        let keys = parse_allowlist_file(&self.path)?;
        *self.keys.write().expect("allowlist lock") = keys;
        Ok(())
    }

    /// Watch the allowlist path with inotify. SIGHUP also reloads if the watch misses.
    pub fn watch(self: Arc<Self>) -> Result<AllowlistWatch, AuthError> {
        let path = self.path.clone();
        let parent = path
            .parent()
            .filter(|p| !p.as_os_str().is_empty())
            .map(|p| p.to_path_buf())
            .unwrap_or_else(|| PathBuf::from("."));
        let file_name = path.file_name().map(|s| s.to_os_string());

        let allow = Arc::clone(&self);
        let (tx, rx) = std::sync::mpsc::channel();
        let mut watcher =
            notify::recommended_watcher(tx).map_err(|e| AuthError::Io(e.to_string()))?;
        watcher
            .watch(&parent, RecursiveMode::NonRecursive)
            .map_err(|e| AuthError::Io(e.to_string()))?;

        let allow_fs = Arc::clone(&allow);
        thread::spawn(move || {
            while let Ok(ev) = rx.recv() {
                match ev {
                    Ok(event) => {
                        let hits = match event.kind {
                            EventKind::Modify(_)
                            | EventKind::Create(_)
                            | EventKind::Remove(_)
                            | EventKind::Any => true,
                            _ => false,
                        };
                        if !hits {
                            continue;
                        }
                        let relevant = match &file_name {
                            None => true,
                            Some(name) => event
                                .paths
                                .iter()
                                .any(|p| p.file_name() == Some(name.as_os_str()) || p == &path),
                        };
                        if relevant {
                            let _ = allow_fs.reload();
                        }
                    }
                    Err(_) => {}
                }
            }
        });

        let flag = Arc::new(std::sync::atomic::AtomicBool::new(false));
        signal_hook::flag::register(SIGHUP, Arc::clone(&flag))
            .map_err(|e| AuthError::Io(e.to_string()))?;
        let allow_hup = Arc::clone(&allow);
        let hup = thread::spawn(move || {
            loop {
                if flag.swap(false, std::sync::atomic::Ordering::SeqCst) {
                    let _ = allow_hup.reload();
                }
                thread::sleep(Duration::from_millis(200));
            }
        });

        // Keep the ModifyKind import used so clippy does not complain later if we tighten kinds.
        let _ = ModifyKind::Data(notify::event::DataChange::Any);

        Ok(AllowlistWatch {
            _watcher: watcher,
            _hup: hup,
        })
    }
}

fn parse_allowlist_file(path: &Path) -> Result<HashSet<[u8; 32]>, AuthError> {
    let text = fs::read_to_string(path)?;
    let mut set = HashSet::new();
    for raw in text.lines() {
        let line = raw.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        match parse_pubkey_line(line) {
            Ok(pk) => {
                set.insert(pk);
            }
            Err(_) => {
                return Err(AuthError::Io(format!(
                    "invalid allowlist line in {}: {}",
                    path.display(),
                    line
                )));
            }
        }
    }
    Ok(set)
}

pub fn parse_pubkey_line(line: &str) -> Result<[u8; 32], AuthError> {
    let pk = nostr::PublicKey::parse(line).map_err(|e| AuthError::Io(e.to_string()))?;
    pubkey_to_bytes(&pk)
}

fn pubkey_to_bytes(pk: &nostr::PublicKey) -> Result<[u8; 32], AuthError> {
    Ok(*pk.as_bytes())
}

/// NIP-98 events are small metadata. Match nostr 0.44.8
/// ([RUSTSEC-2026-0229](https://rustsec.org/advisories/RUSTSEC-2026-0229), accessed: 2026-08-31).
const MAX_NIP98_AUTH_EVENT_BYTES: usize = 64 * 1024;
const MAX_NIP98_AUTH_ENCODED_BYTES: usize = (MAX_NIP98_AUTH_EVENT_BYTES / 3
    + if MAX_NIP98_AUTH_EVENT_BYTES % 3 == 0 {
        0
    } else {
        1
    })
    * 4;

/// Verify `Authorization: Nostr <base64 event>` per NIP-98, then the allowlist.
pub fn verify_nip98(
    header: Option<&str>,
    method: &str,
    absolute_url: &str,
    body: &[u8],
    now_unix: u64,
    window_secs: u64,
    allow: &Allowlist,
) -> Result<[u8; 32], AuthError> {
    verify_nip98_impl(
        header,
        method,
        absolute_url,
        body,
        now_unix,
        window_secs,
        allow,
    )
}

#[allow(dead_code)]
fn verify_nip98_impl(
    header: Option<&str>,
    method: &str,
    absolute_url: &str,
    body: &[u8],
    now_unix: u64,
    window_secs: u64,
    allow: &Allowlist,
) -> Result<[u8; 32], AuthError> {
    let header = header.ok_or(AuthError::MissingHeader)?;
    let encoded = header
        .strip_prefix("Nostr ")
        .or_else(|| header.strip_prefix("nostr "))
        .ok_or(AuthError::MissingHeader)?
        .trim();
    if encoded.is_empty() {
        return Err(AuthError::MissingHeader);
    }
    // Bound attacker-controlled input before Base64 allocates its decoded buffer.
    if encoded.len() > MAX_NIP98_AUTH_ENCODED_BYTES {
        return Err(AuthError::BadEncoding);
    }
    let raw = base64::decode(encoded).map_err(|_| AuthError::BadEncoding)?;
    // Keep the JSON parser bounded even for non-canonical Base64.
    if raw.len() > MAX_NIP98_AUTH_EVENT_BYTES {
        return Err(AuthError::BadEncoding);
    }
    let value: serde_json::Value = serde_json::from_slice(&raw).map_err(|_| AuthError::BadJson)?;
    let event: nostr::Event = serde_json::from_slice(&raw).map_err(|_| AuthError::BadJson)?;
    if event.verify().is_err() {
        return Err(AuthError::BadSignature);
    }
    let kind = value
        .get("kind")
        .and_then(|k| k.as_u64())
        .ok_or(AuthError::WrongKind)?;
    if kind != 27235 {
        return Err(AuthError::WrongKind);
    }
    let created_at = value
        .get("created_at")
        .and_then(|c| c.as_u64())
        .ok_or(AuthError::Expired)?;
    let delta = if now_unix >= created_at {
        now_unix - created_at
    } else {
        created_at - now_unix
    };
    if delta > window_secs {
        return Err(AuthError::Expired);
    }
    let tags = value
        .get("tags")
        .and_then(|t| t.as_array())
        .ok_or(AuthError::UrlMismatch)?;
    let mut u_tag: Option<&str> = None;
    let mut method_tag: Option<&str> = None;
    let mut payload_tag: Option<&str> = None;
    for tag in tags {
        let arr = match tag.as_array() {
            Some(a) if !a.is_empty() => a,
            _ => continue,
        };
        let name = arr[0].as_str().unwrap_or("");
        let val = arr.get(1).and_then(|v| v.as_str());
        match name {
            "u" if u_tag.is_none() => u_tag = val,
            "method" if method_tag.is_none() => method_tag = val,
            "payload" if payload_tag.is_none() => payload_tag = val,
            _ => {}
        }
    }
    match u_tag {
        Some(u) if u == absolute_url => {}
        _ => return Err(AuthError::UrlMismatch),
    }
    match method_tag {
        Some(m) if m.eq_ignore_ascii_case(method) => {}
        _ => return Err(AuthError::MethodMismatch),
    }
    if let Some(payload) = payload_tag {
        let digest = Sha256::digest(body);
        let expected = hex::encode(digest);
        if payload.to_ascii_lowercase() != expected {
            return Err(AuthError::PayloadMismatch);
        }
    }
    let pubkey32 = pubkey_to_bytes(&event.pubkey)?;
    if !allow.contains(&pubkey32) {
        return Err(AuthError::NotAllowlisted);
    }
    Ok(pubkey32)
}

#[cfg(test)]
mod tests {
    use super::*;
    use nostr::event::EventBuilder;
    use nostr::hashes::{Hash, sha256};
    use nostr::nips::nip98::{HttpData, HttpMethod};
    use nostr::prelude::*;
    use sha2::{Digest, Sha256};
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn now() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
    }

    fn pubkey32(keys: &Keys) -> [u8; 32] {
        pubkey_to_bytes(&keys.public_key()).unwrap()
    }

    fn npub_of(keys: &Keys) -> String {
        keys.public_key().to_bech32().unwrap()
    }

    fn write_allowlist(dir: &tempfile::TempDir, npubs: &[&str]) -> std::path::PathBuf {
        let path = dir.path().join("allowlist");
        let mut body = String::new();
        for n in npubs {
            body.push_str(n);
            body.push('\n');
        }
        fs::write(&path, body).unwrap();
        path
    }

    fn sign_header(
        keys: &Keys,
        method: HttpMethod,
        url: &str,
        created_at: u64,
        body: Option<&[u8]>,
    ) -> String {
        let parsed = nostr::Url::parse(url).expect("url");
        let mut data = HttpData::new(parsed, method);
        if let Some(b) = body {
            let digest = Sha256::digest(b);
            let hash = sha256::Hash::from_slice(&digest).expect("sha256");
            data = data.payload(hash);
        }
        let event = EventBuilder::http_auth(data)
            .custom_created_at(Timestamp::from(created_at))
            .sign_with_keys(keys)
            .expect("sign");
        let json = serde_json::to_vec(&event).expect("json");
        format!("Nostr {}", base64::encode(&json))
    }

    fn sign_kind(keys: &Keys, kind: u16, method: &str, url: &str, created_at: u64) -> String {
        let event = EventBuilder::new(Kind::from(kind), "")
            .tag(Tag::parse(["u", url]).unwrap())
            .tag(Tag::parse(["method", method]).unwrap())
            .custom_created_at(Timestamp::from(created_at))
            .sign_with_keys(keys)
            .expect("sign");
        let json = serde_json::to_vec(&event).expect("json");
        format!("Nostr {}", base64::encode(&json))
    }

    #[test]
    fn missing_allowlist_file_is_hard_error() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("nope");
        let err = Allowlist::load(&path).unwrap_err();
        match err {
            AuthError::Io(_) => {}
            other => panic!("expected io, got {:?}", other),
        }
    }

    #[test]
    fn empty_allowlist_file_is_empty_set() {
        let dir = tempfile::tempdir().unwrap();
        let path = write_allowlist(&dir, &[]);
        let allow = Allowlist::load(&path).unwrap();
        assert!(allow.snapshot().is_empty());
    }

    #[test]
    fn good_sig_listed_npub_matching_u_method_accepted() {
        let keys = Keys::generate();
        let dir = tempfile::tempdir().unwrap();
        let npub = npub_of(&keys);
        let path = write_allowlist(&dir, &[&npub]);
        let allow = Allowlist::load(&path).unwrap();
        let url = "https://splora.example/tx/abc";
        let now = now();
        let header = sign_header(&keys, HttpMethod::GET, url, now, None);
        let pk = verify_nip98(Some(&header), "GET", url, b"", now, 60, &allow).unwrap();
        assert_eq!(pk, pubkey32(&keys));
    }

    #[test]
    fn replay_outside_window_is_401() {
        let keys = Keys::generate();
        let dir = tempfile::tempdir().unwrap();
        let path = write_allowlist(&dir, &[&npub_of(&keys)]);
        let allow = Allowlist::load(&path).unwrap();
        let url = "https://splora.example/tx";
        let now = now();
        let header = sign_header(&keys, HttpMethod::GET, url, now - 120, None);
        let err = verify_nip98(Some(&header), "GET", url, b"", now, 60, &allow).unwrap_err();
        assert_eq!(err, AuthError::Expired);
    }

    #[test]
    fn wrong_url_is_401() {
        let keys = Keys::generate();
        let dir = tempfile::tempdir().unwrap();
        let path = write_allowlist(&dir, &[&npub_of(&keys)]);
        let allow = Allowlist::load(&path).unwrap();
        let now = now();
        let header = sign_header(
            &keys,
            HttpMethod::GET,
            "https://splora.example/a",
            now,
            None,
        );
        let err = verify_nip98(
            Some(&header),
            "GET",
            "https://splora.example/b",
            b"",
            now,
            60,
            &allow,
        )
        .unwrap_err();
        assert_eq!(err, AuthError::UrlMismatch);
    }

    #[test]
    fn unknown_npub_is_401() {
        let keys = Keys::generate();
        let stranger = Keys::generate();
        let dir = tempfile::tempdir().unwrap();
        let path = write_allowlist(&dir, &[&npub_of(&keys)]);
        let allow = Allowlist::load(&path).unwrap();
        let url = "https://splora.example/tx";
        let now = now();
        let header = sign_header(&stranger, HttpMethod::GET, url, now, None);
        let err = verify_nip98(Some(&header), "GET", url, b"", now, 60, &allow).unwrap_err();
        assert_eq!(err, AuthError::NotAllowlisted);
    }

    #[test]
    fn missing_header_is_401() {
        let dir = tempfile::tempdir().unwrap();
        let path = write_allowlist(&dir, &[]);
        let allow = Allowlist::load(&path).unwrap();
        let err = verify_nip98(None, "GET", "https://x/", b"", now(), 60, &allow).unwrap_err();
        assert_eq!(err, AuthError::MissingHeader);
    }

    #[test]
    fn npub_removed_after_load_is_401() {
        let keys = Keys::generate();
        let dir = tempfile::tempdir().unwrap();
        let path = write_allowlist(&dir, &[&npub_of(&keys)]);
        let allow = Allowlist::load(&path).unwrap();
        fs::write(&path, "").unwrap();
        allow.reload().unwrap();
        let url = "https://splora.example/tx";
        let now = now();
        let header = sign_header(&keys, HttpMethod::GET, url, now, None);
        let err = verify_nip98(Some(&header), "GET", url, b"", now, 60, &allow).unwrap_err();
        assert_eq!(err, AuthError::NotAllowlisted);
    }

    #[test]
    fn reload_after_file_write_accepts_without_restart() {
        let keys = Keys::generate();
        let dir = tempfile::tempdir().unwrap();
        let path = write_allowlist(&dir, &[]);
        let allow = Allowlist::load(&path).unwrap();
        let url = "https://splora.example/tx";
        let now = now();
        let header = sign_header(&keys, HttpMethod::GET, url, now, None);
        let err = verify_nip98(Some(&header), "GET", url, b"", now, 60, &allow).unwrap_err();
        assert_eq!(err, AuthError::NotAllowlisted);

        fs::write(&path, format!("{}\n", npub_of(&keys))).unwrap();
        allow.reload().unwrap();
        let pk = verify_nip98(Some(&header), "GET", url, b"", now, 60, &allow).unwrap();
        assert_eq!(pk, pubkey32(&keys));
    }

    #[test]
    fn wrong_kind_is_401() {
        let keys = Keys::generate();
        let dir = tempfile::tempdir().unwrap();
        let path = write_allowlist(&dir, &[&npub_of(&keys)]);
        let allow = Allowlist::load(&path).unwrap();
        let url = "https://splora.example/tx";
        let now = now();
        let header = sign_kind(&keys, 1, "GET", url, now);
        let err = verify_nip98(Some(&header), "GET", url, b"", now, 60, &allow).unwrap_err();
        assert_eq!(err, AuthError::WrongKind);
    }

    #[test]
    fn payload_mismatch_is_401() {
        let keys = Keys::generate();
        let dir = tempfile::tempdir().unwrap();
        let path = write_allowlist(&dir, &[&npub_of(&keys)]);
        let allow = Allowlist::load(&path).unwrap();
        let url = "https://splora.example/tx";
        let now = now();
        let header = sign_header(&keys, HttpMethod::POST, url, now, Some(b"hello"));
        let err = verify_nip98(Some(&header), "POST", url, b"other", now, 60, &allow).unwrap_err();
        assert_eq!(err, AuthError::PayloadMismatch);
    }

    #[test]
    fn oversized_nip98_authorization_is_rejected_before_json_parse() {
        let dir = tempfile::tempdir().unwrap();
        let path = write_allowlist(&dir, &[]);
        let allow = Allowlist::load(&path).unwrap();
        // One Base64 quantum past the crate encoded cap (nostr 0.44.8, RUSTSEC-2026-0229).
        // Length stays a multiple of 4 so decode would succeed; JSON parse would be BadJson.
        let encoded = "A".repeat(MAX_NIP98_AUTH_ENCODED_BYTES + 4);
        let header = format!("Nostr {}", encoded);
        let err = verify_nip98(
            Some(&header),
            "GET",
            "https://splora.example/tx",
            b"",
            now(),
            60,
            &allow,
        )
        .unwrap_err();
        assert_eq!(err, AuthError::BadEncoding);
    }

    #[test]
    fn oversized_decoded_nip98_authorization_is_rejected_before_json_parse() {
        let dir = tempfile::tempdir().unwrap();
        let path = write_allowlist(&dir, &[]);
        let allow = Allowlist::load(&path).unwrap();
        let decoded = vec![b' '; MAX_NIP98_AUTH_EVENT_BYTES + 1];
        let encoded = base64::encode(&decoded);
        assert!(encoded.len() <= MAX_NIP98_AUTH_ENCODED_BYTES);
        let header = format!("Nostr {}", encoded);
        let err = verify_nip98(
            Some(&header),
            "GET",
            "https://splora.example/tx",
            b"",
            now(),
            60,
            &allow,
        )
        .unwrap_err();
        assert_eq!(err, AuthError::BadEncoding);
    }
}
