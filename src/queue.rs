// SPDX-License-Identifier: Unlicense
//! Unauthenticated approval-queue HTTP service (npub + email).
//! This file is the queue. It is not the allowlist.

use crate::auth::{parse_pubkey_line, Allowlist};
use clap::{App, Arg, ArgMatches, SubCommand};
use serde_derive::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::{self, OpenOptions};
use std::io::{self, Write};
use std::net::{IpAddr, SocketAddr};
use std::os::unix::fs::FileTypeExt;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};
use tiny_http::{Header, Method, Response, Server, StatusCode};

#[derive(Debug, Clone)]
pub struct QueueCaps {
    pub max_rows: usize,
    pub max_bytes: usize,
    pub per_ip: u32,
    pub per_npub: u32,
    pub window_secs: u64,
}

impl Default for QueueCaps {
    fn default() -> Self {
        QueueCaps {
            max_rows: 10_000,
            max_bytes: 2 * 1024 * 1024,
            per_ip: 10,
            per_npub: 5,
            window_secs: 60,
        }
    }
}

/// Pending queue row. Disk is `npub,email` only. HTTP POST body may still be JSON.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct QueueRow {
    pub npub: String,
    pub email: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum QueueError {
    InvalidNpub,
    InvalidEmail,
    BadJson,
    RateLimited,
    CapExceeded,
    NotFound,
    Io(String),
}

impl std::fmt::Display for QueueError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            QueueError::InvalidNpub => write!(f, "invalid npub"),
            QueueError::InvalidEmail => write!(f, "email must not contain a comma"),
            QueueError::BadJson => write!(f, "invalid JSON body"),
            QueueError::RateLimited => write!(f, "rate limited"),
            QueueError::CapExceeded => write!(f, "queue cap exceeded"),
            QueueError::NotFound => write!(f, "npub not in queue"),
            QueueError::Io(s) => write!(f, "queue io: {}", s),
        }
    }
}

impl std::error::Error for QueueError {}

impl From<io::Error> for QueueError {
    fn from(e: io::Error) -> Self {
        QueueError::Io(e.to_string())
    }
}

#[derive(Debug, Deserialize)]
struct PostBody {
    npub: String,
    email: String,
}

/// In-memory rate counters plus on-disk queue path.
pub struct QueueStore {
    path: PathBuf,
    caps: QueueCaps,
    ip_hits: Mutex<HashMap<IpAddr, Vec<u64>>>,
    npub_hits: Mutex<HashMap<[u8; 32], Vec<u64>>>,
    /// Load-plus-write of the CSV. Unix HTTP can POST concurrently.
    disk: Mutex<()>,
}

impl QueueStore {
    pub fn open(path: impl Into<PathBuf>, caps: QueueCaps) -> Result<Self, QueueError> {
        let path = path.into();
        if !path.exists() {
            fs::write(&path, "")?;
        }
        Ok(QueueStore {
            path,
            caps,
            ip_hits: Mutex::new(HashMap::new()),
            npub_hits: Mutex::new(HashMap::new()),
            disk: Mutex::new(()),
        })
    }

    pub fn load(&self) -> Result<Vec<QueueRow>, QueueError> {
        load_queue(&self.path)
    }

    /// `ip` is the TCP peer or a forwarded client address. `None` skips the
    /// per-IP bucket (unix sockets with no `X-Forwarded-For`).
    pub fn submit(
        &self,
        ip: Option<IpAddr>,
        npub: &str,
        email: &str,
        now: u64,
    ) -> Result<QueueRow, QueueError> {
        submit_impl(self, ip, npub, email, now)
    }
}

/// Parse one pending line. Exactly two comma fields. JSON lines and extra commas are invalid.
fn parse_queue_line(line: &str) -> Option<QueueRow> {
    let mut parts = line.split(',');
    let npub = parts.next()?.trim();
    let email = parts.next()?.trim();
    if parts.next().is_some() || npub.is_empty() {
        return None;
    }
    Some(QueueRow {
        npub: npub.to_string(),
        email: email.to_string(),
    })
}

pub fn load_queue(path: &Path) -> Result<Vec<QueueRow>, QueueError> {
    let text = fs::read_to_string(path)?;
    let mut rows = Vec::new();
    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        if let Some(row) = parse_queue_line(line) {
            rows.push(row);
        }
    }
    Ok(rows)
}

fn write_queue(path: &Path, rows: &[QueueRow]) -> Result<(), QueueError> {
    let mut body = String::new();
    for row in rows {
        body.push_str(&row.npub);
        body.push(',');
        body.push_str(&row.email);
        body.push('\n');
    }
    let tmp = path.with_extension("tmp");
    {
        let mut f = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(&tmp)?;
        f.write_all(body.as_bytes())?;
        f.sync_all()?;
    }
    fs::rename(&tmp, path)?;
    if let Ok(f) = OpenOptions::new().read(true).open(path) {
        let _ = f.sync_all();
    }
    Ok(())
}

fn valid_npub(npub: &str) -> Result<[u8; 32], QueueError> {
    if !npub.starts_with("npub1") {
        return Err(QueueError::InvalidNpub);
    }
    parse_pubkey_line(npub).map_err(|_| QueueError::InvalidNpub)
}

fn rate_ok(hits: &mut Vec<u64>, now: u64, window: u64, max: u32) -> bool {
    hits.retain(|t| now.saturating_sub(*t) < window);
    if hits.len() as u32 >= max {
        return false;
    }
    hits.push(now);
    true
}

#[allow(dead_code)]
fn submit_impl(
    store: &QueueStore,
    ip: Option<IpAddr>,
    npub: &str,
    email: &str,
    now: u64,
) -> Result<QueueRow, QueueError> {
    let pk = valid_npub(npub)?;
    if email.contains(',') {
        return Err(QueueError::InvalidEmail);
    }
    if let Some(ip) = ip {
        let mut ip_hits = store.ip_hits.lock().expect("ip hits");
        if !rate_ok(
            ip_hits.entry(ip).or_default(),
            now,
            store.caps.window_secs,
            store.caps.per_ip,
        ) {
            return Err(QueueError::RateLimited);
        }
    }
    {
        let mut npub_hits = store.npub_hits.lock().expect("npub hits");
        if !rate_ok(
            npub_hits.entry(pk).or_default(),
            now,
            store.caps.window_secs,
            store.caps.per_npub,
        ) {
            return Err(QueueError::RateLimited);
        }
    }
    let _disk = store.disk.lock().expect("queue disk");
    let mut rows = load_queue(&store.path)?;
    if let Some(existing) = rows.iter_mut().find(|r| r.npub == npub) {
        existing.email = email.to_string();
        let out = existing.clone();
        write_queue(&store.path, &rows)?;
        return Ok(out);
    }
    let line_bytes = npub.len() + 1 + email.len() + 1;
    let current_bytes = fs::metadata(&store.path).map(|m| m.len()).unwrap_or(0);
    if rows.len() >= store.caps.max_rows
        || current_bytes.saturating_add(line_bytes as u64) > store.caps.max_bytes as u64
    {
        return Err(QueueError::CapExceeded);
    }
    let row = QueueRow {
        npub: npub.to_string(),
        email: email.to_string(),
    };
    rows.push(row.clone());
    write_queue(&store.path, &rows)?;
    Ok(row)
}

/// Operator: delete the pending queue line, upsert the RO allowlist, fsync.
pub fn approve(queue_path: &Path, allowlist_path: &Path, npub: &str) -> Result<(), QueueError> {
    let _ = valid_npub(npub)?;
    let mut rows = if queue_path.exists() {
        load_queue(queue_path)?
    } else {
        Vec::new()
    };
    rows.retain(|r| r.npub != npub);
    if queue_path.exists() {
        write_queue(queue_path, &rows)?;
    }
    upsert_allowlist(allowlist_path, npub)?;
    Ok(())
}

/// Operator: delete the pending queue line. Do not write the allowlist.
pub fn reject(queue_path: &Path, npub: &str) -> Result<(), QueueError> {
    let _ = valid_npub(npub)?;
    let mut rows = load_queue(queue_path)?;
    let before = rows.len();
    rows.retain(|r| r.npub != npub);
    if rows.len() == before {
        return Err(QueueError::NotFound);
    }
    write_queue(queue_path, &rows)?;
    Ok(())
}

/// Operator: delete npub from the RO allowlist. Never kill splora.
pub fn remove_npub(allowlist_path: &Path, npub: &str) -> Result<(), QueueError> {
    let pk = valid_npub(npub)?;
    let text = if allowlist_path.exists() {
        fs::read_to_string(allowlist_path)?
    } else {
        String::new()
    };
    let mut kept = Vec::new();
    for raw in text.lines() {
        let line = raw.trim();
        if line.is_empty() || line.starts_with('#') {
            kept.push(raw.to_string());
            continue;
        }
        match parse_pubkey_line(line) {
            Ok(existing) if existing == pk => continue,
            _ => kept.push(raw.to_string()),
        }
    }
    fsync_lines(allowlist_path, &kept)?;
    Ok(())
}

fn upsert_allowlist(path: &Path, npub: &str) -> Result<(), QueueError> {
    let pk = valid_npub(npub)?;
    let text = if path.exists() {
        fs::read_to_string(path)?
    } else {
        String::new()
    };
    let mut kept = Vec::new();
    let mut found = false;
    for raw in text.lines() {
        let line = raw.trim();
        if line.is_empty() || line.starts_with('#') {
            kept.push(raw.to_string());
            continue;
        }
        match parse_pubkey_line(line) {
            Ok(existing) if existing == pk => {
                kept.push(npub.to_string());
                found = true;
            }
            _ => kept.push(raw.to_string()),
        }
    }
    if !found {
        kept.push(npub.to_string());
    }
    fsync_lines(path, &kept)
}

fn fsync_lines(path: &Path, lines: &[String]) -> Result<(), QueueError> {
    let mut body = String::new();
    for line in lines {
        body.push_str(line);
        body.push('\n');
    }
    let tmp = path.with_extension("tmp");
    {
        let mut f = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(&tmp)?;
        f.write_all(body.as_bytes())?;
        f.sync_all()?;
    }
    fs::rename(&tmp, path)?;
    if let Ok(f) = OpenOptions::new().read(true).open(path) {
        let _ = f.sync_all();
    }
    Ok(())
}

fn json_status(code: u16, msg: &str) -> Response<std::io::Cursor<Vec<u8>>> {
    let body = format!("{{\"error\":\"{}\"}}", msg);
    Response::from_string(body)
        .with_status_code(StatusCode(code))
        .with_header(Header::from_bytes(&b"Content-Type"[..], &b"application/json"[..]).unwrap())
}

/// Clap app for `splora-queue`. `--bind` (TCP) or `--socket-file` (unix), plus `--queue-file`.
pub fn queue_cli_app() -> App<'static, 'static> {
    App::new("splora-queue")
        .about("Unauthenticated import-queue HTTP POST. This is not the indexer.")
        .arg(
            Arg::with_name("bind")
                .long("bind")
                .takes_value(true)
                .required_unless("socket-file")
                .conflicts_with("socket-file")
                .help("TCP listen address. Do not combine with --socket-file."),
        )
        .arg(
            Arg::with_name("socket-file")
                .long("socket-file")
                .takes_value(true)
                .required_unless("bind")
                .conflicts_with("bind")
                .help("Unix socket path to listen on. Do not combine with --bind."),
        )
        .arg(
            Arg::with_name("queue-file")
                .long("queue-file")
                .takes_value(true)
                .required(true)
                .help("Queue CSV path (npub,email per pending line). Sibling .tmp writes go in this directory."),
        )
}

/// Clap app for `splora-import approve|reject|remove` with `--queue` / `--allowlist`.
pub fn import_cli_app() -> App<'static, 'static> {
    App::new("splora-import")
        .about("Approve, reject, or remove npubs. Writes the read-only allowlist and queue files. Does not kill splora.")
        .subcommand(
            SubCommand::with_name("approve")
                .about("Delete the queue line, upsert npub into the allowlist, and fsync.")
                .arg(
                    Arg::with_name("queue")
                        .long("queue")
                        .takes_value(true)
                        .required(true),
                )
                .arg(
                    Arg::with_name("allowlist")
                        .long("allowlist")
                        .takes_value(true)
                        .required(true),
                )
                .arg(Arg::with_name("npub").required(true).index(1)),
        )
        .subcommand(
            SubCommand::with_name("reject")
                .about("Delete the queue line. Does not write the allowlist.")
                .arg(
                    Arg::with_name("queue")
                        .long("queue")
                        .takes_value(true)
                        .required(true),
                )
                .arg(Arg::with_name("npub").required(true).index(1)),
        )
        .subcommand(
            SubCommand::with_name("remove")
                .about("Delete npub from the allowlist.")
                .arg(
                    Arg::with_name("allowlist")
                        .long("allowlist")
                        .takes_value(true)
                        .required(true),
                )
                .arg(Arg::with_name("npub").required(true).index(1)),
        )
}

fn listen_from_matches(m: &ArgMatches) -> Result<QueueListen, QueueError> {
    match (m.value_of("bind"), m.value_of("socket-file")) {
        (Some(_), Some(_)) => Err(QueueError::Io(
            "use --bind or --socket-file, not both".to_string(),
        )),
        (Some(bind), None) => {
            let addr: SocketAddr = bind
                .parse()
                .map_err(|e| QueueError::Io(format!("invalid --bind: {}", e)))?;
            Ok(QueueListen::Tcp(addr))
        }
        (None, Some(path)) => Ok(QueueListen::Unix(PathBuf::from(path))),
        (None, None) => Err(QueueError::Io("need --bind or --socket-file".to_string())),
    }
}

enum QueueListen {
    Tcp(SocketAddr),
    Unix(PathBuf),
}

/// Parse argv and run the queue HTTP server. Called only from `splora-queue`.
pub fn run_from_args() -> Result<(), QueueError> {
    let m = queue_cli_app().get_matches();
    let path = PathBuf::from(m.value_of("queue-file").unwrap());
    match listen_from_matches(&m)? {
        QueueListen::Tcp(bind) => run(bind, path, QueueCaps::default()),
        QueueListen::Unix(socket) => run_unix(socket, path, QueueCaps::default()),
    }
}

/// Blocking tiny_http server. POST JSON `{npub, email}` only.
/// Writes only `queue_path` (and its sibling `.tmp`). Does not write the allowlist.
pub fn run(bind: SocketAddr, queue_path: PathBuf, caps: QueueCaps) -> Result<(), QueueError> {
    let store = Arc::new(QueueStore::open(queue_path, caps)?);
    let server = Server::http(bind).map_err(|e| QueueError::Io(e.to_string()))?;
    for mut request in server.incoming_requests() {
        let method = request.method().clone();
        let ip = request.remote_addr().ip();
        if method != Method::Post {
            let _ = request.respond(json_status(405, "method not allowed"));
            continue;
        }
        let mut body = String::new();
        if request.as_reader().read_to_string(&mut body).is_err() {
            let _ = request.respond(json_status(400, "bad body"));
            continue;
        }
        let parsed: Result<PostBody, _> = serde_json::from_str(&body);
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        let response = match parsed {
            Err(_) => json_status(400, "invalid json"),
            Ok(p) => match store.submit(Some(ip), &p.npub, &p.email, now) {
                Ok(row) => {
                    let s = serde_json::to_string(&row).unwrap_or_else(|_| "{}".into());
                    Response::from_string(s).with_header(
                        Header::from_bytes(&b"Content-Type"[..], &b"application/json"[..]).unwrap(),
                    )
                }
                Err(QueueError::InvalidNpub) => json_status(400, "invalid npub"),
                Err(QueueError::InvalidEmail) => json_status(400, "email must not contain a comma"),
                Err(QueueError::RateLimited) => json_status(429, "rate limited"),
                Err(QueueError::CapExceeded) => json_status(507, "queue cap exceeded"),
                Err(e) => json_status(500, &e.to_string()),
            },
        };
        let _ = request.respond(response);
    }
    Ok(())
}

/// Blocking HTTP on a unix socket. Same POST JSON `{npub, email}` contract as TCP.
pub fn run_unix(
    socket_path: PathBuf,
    queue_path: PathBuf,
    caps: QueueCaps,
) -> Result<(), QueueError> {
    let store = Arc::new(QueueStore::open(queue_path, caps)?);
    if let Ok(meta) = fs::metadata(&socket_path) {
        if meta.file_type().is_socket() {
            fs::remove_file(&socket_path).ok();
        }
    }
    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .map_err(|e| QueueError::Io(e.to_string()))?;
    rt.block_on(serve_unix(socket_path, store))
}

async fn serve_unix(socket_path: PathBuf, store: Arc<QueueStore>) -> Result<(), QueueError> {
    use hyper::service::{make_service_fn, service_fn};
    use hyperlocal::UnixServerExt;

    let make = make_service_fn(move |_| {
        let store = Arc::clone(&store);
        async move {
            Ok::<_, hyper::Error>(service_fn(move |req: hyper::Request<hyper::Body>| {
                let store = Arc::clone(&store);
                async move { Ok::<_, hyper::Error>(handle_unix_http(store, req).await) }
            }))
        }
    });
    hyper::Server::bind_unix(&socket_path)
        .map_err(|e| QueueError::Io(e.to_string()))?
        .serve(make)
        .await
        .map_err(|e| QueueError::Io(e.to_string()))
}

async fn handle_unix_http(
    store: Arc<QueueStore>,
    req: hyper::Request<hyper::Body>,
) -> hyper::Response<hyper::Body> {
    if req.method() != hyper::Method::POST {
        return hyper_json_status(405, "method not allowed");
    }
    let ip = unix_client_ip(&req);
    let body = match hyper::body::to_bytes(req.into_body()).await {
        Ok(b) => b,
        Err(_) => return hyper_json_status(400, "bad body"),
    };
    let body = match std::str::from_utf8(&body) {
        Ok(s) => s,
        Err(_) => return hyper_json_status(400, "bad body"),
    };
    let parsed: Result<PostBody, _> = serde_json::from_str(body);
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    match parsed {
        Err(_) => hyper_json_status(400, "invalid json"),
        Ok(p) => match store.submit(ip, &p.npub, &p.email, now) {
            Ok(row) => {
                let s = serde_json::to_string(&row).unwrap_or_else(|_| "{}".into());
                hyper::Response::builder()
                    .header("Content-Type", "application/json")
                    .body(hyper::Body::from(s))
                    .unwrap_or_else(|_| hyper_json_status(500, "response"))
            }
            Err(QueueError::InvalidNpub) => hyper_json_status(400, "invalid npub"),
            Err(QueueError::InvalidEmail) => {
                hyper_json_status(400, "email must not contain a comma")
            }
            Err(QueueError::RateLimited) => hyper_json_status(429, "rate limited"),
            Err(QueueError::CapExceeded) => hyper_json_status(507, "queue cap exceeded"),
            Err(e) => hyper_json_status(500, &e.to_string()),
        },
    }
}

/// First `X-Forwarded-For` hop when the Surmount edge forwarded a client
/// address. No header means rate-limit by npub only (do not share 127.0.0.1).
fn unix_client_ip(req: &hyper::Request<hyper::Body>) -> Option<IpAddr> {
    req.headers()
        .get("x-forwarded-for")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| {
            s.split(',')
                .next()
                .map(str::trim)
                .filter(|h| !h.is_empty())
                .and_then(|h| h.parse().ok())
        })
}

fn hyper_json_status(code: u16, msg: &str) -> hyper::Response<hyper::Body> {
    let body = format!("{{\"error\":\"{}\"}}", msg);
    hyper::Response::builder()
        .status(code)
        .header("Content-Type", "application/json")
        .body(hyper::Body::from(body.clone()))
        .unwrap_or_else(|_| hyper::Response::new(hyper::Body::from(body)))
}

// Silence unused Allowlist import until rest.rs wires NIP-98 plus import.
#[allow(dead_code)]
fn _allowlist_type_hint(_: &Allowlist) {}

#[cfg(test)]
mod tests {
    use super::*;
    use nostr::prelude::*;

    fn npub_of(keys: &Keys) -> String {
        keys.public_key().to_bech32().unwrap()
    }

    fn store_with(caps: QueueCaps) -> (tempfile::TempDir, QueueStore) {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("queue.csv");
        let store = QueueStore::open(&path, caps).unwrap();
        (dir, store)
    }

    #[test]
    fn post_updates_existing_npub_in_place() {
        let keys = Keys::generate();
        let npub = npub_of(&keys);
        let (_dir, store) = store_with(QueueCaps::default());
        let ip = IpAddr::from([127, 0, 0, 1]);
        store
            .submit(Some(ip), &npub, "a@example.com", 1_000)
            .unwrap();
        store
            .submit(Some(ip), &npub, "b@example.com", 1_001)
            .unwrap();
        let rows = store.load().unwrap();
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].email, "b@example.com");
        assert_eq!(rows[0].npub, npub);
    }

    /// Disk is one `npub,email` CSV line. Not JSON. No status column.
    #[test]
    fn queue_persists_npub_comma_email_one_line() {
        let keys = Keys::generate();
        let npub = npub_of(&keys);
        let (_dir, store) = store_with(QueueCaps::default());
        store
            .submit(
                Some(IpAddr::from([127, 0, 0, 1])),
                &npub,
                "ops@example.com",
                1,
            )
            .unwrap();
        let text = fs::read_to_string(&store.path).unwrap();
        assert_eq!(text, format!("{},{}\n", npub, "ops@example.com"));
        assert!(!text.contains('{'), "queue file must not be JSON lines");
        assert!(
            !text.contains("pending") && !text.contains("status"),
            "queue file must not store a status column"
        );
        let rows = store.load().unwrap();
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].npub, npub);
        assert_eq!(rows[0].email, "ops@example.com");
    }

    #[test]
    fn queue_rejects_email_containing_comma() {
        let keys = Keys::generate();
        let npub = npub_of(&keys);
        let (_dir, store) = store_with(QueueCaps::default());
        let err = store
            .submit(
                Some(IpAddr::from([127, 0, 0, 1])),
                &npub,
                "ops@example.com,evil",
                1,
            )
            .unwrap_err();
        assert_eq!(err, QueueError::InvalidEmail);
        let text = fs::read_to_string(&store.path).unwrap();
        assert!(
            text.trim().is_empty(),
            "comma email must not be written: {:?}",
            text
        );
    }

    #[test]
    fn queue_same_npub_updates_that_line() {
        let keys = Keys::generate();
        let npub = npub_of(&keys);
        let (_dir, store) = store_with(QueueCaps::default());
        let ip = IpAddr::from([127, 0, 0, 1]);
        store
            .submit(Some(ip), &npub, "a@example.com", 1_000)
            .unwrap();
        store
            .submit(Some(ip), &npub, "b@example.com", 1_001)
            .unwrap();
        let text = fs::read_to_string(&store.path).unwrap();
        let lines: Vec<&str> = text.lines().filter(|l| !l.is_empty()).collect();
        assert_eq!(lines.len(), 1);
        assert_eq!(lines[0], format!("{},{}", npub, "b@example.com"));
    }

    #[test]
    fn garbage_npub_is_rejected() {
        let (_dir, store) = store_with(QueueCaps::default());
        let err = store
            .submit(Some(IpAddr::from([1, 2, 3, 4])), "not-an-npub", "a@b.c", 1)
            .unwrap_err();
        assert_eq!(err, QueueError::InvalidNpub);
    }

    #[test]
    fn cap_refuses_new_npub_but_updates_existing() {
        let a = Keys::generate();
        let b = Keys::generate();
        let caps = QueueCaps {
            max_rows: 1,
            max_bytes: 2 * 1024 * 1024,
            per_ip: 100,
            per_npub: 100,
            window_secs: 60,
        };
        let (_dir, store) = store_with(caps);
        let ip = IpAddr::from([10, 0, 0, 1]);
        store
            .submit(Some(ip), &npub_of(&a), "a@example.com", 10)
            .unwrap();
        let err = store
            .submit(Some(ip), &npub_of(&b), "b@example.com", 11)
            .unwrap_err();
        assert_eq!(err, QueueError::CapExceeded);
        store
            .submit(Some(ip), &npub_of(&a), "a-new@example.com", 12)
            .unwrap();
        let rows = store.load().unwrap();
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].email, "a-new@example.com");
    }

    #[test]
    fn import_approve_fsyncs_allowlist() {
        let keys = Keys::generate();
        let npub = npub_of(&keys);
        let dir = tempfile::tempdir().unwrap();
        let queue = dir.path().join("queue.csv");
        let allow = dir.path().join("allowlist");
        fs::write(&queue, "").unwrap();
        fs::write(&allow, "").unwrap();
        let store = QueueStore::open(&queue, QueueCaps::default()).unwrap();
        store
            .submit(
                Some(IpAddr::from([127, 0, 0, 1])),
                &npub,
                "ops@example.com",
                1,
            )
            .unwrap();
        approve(&queue, &allow, &npub).unwrap();
        let text = fs::read_to_string(&allow).unwrap();
        assert!(
            text.contains(&npub),
            "allowlist should contain npub after approve"
        );
        let rows = load_queue(&queue).unwrap();
        assert!(rows.is_empty(), "approve must delete the queue line");
    }

    #[test]
    fn approve_moves_to_allowlist_deletes_queue_line() {
        let keys = Keys::generate();
        let npub = npub_of(&keys);
        let dir = tempfile::tempdir().unwrap();
        let queue = dir.path().join("queue.csv");
        let allow = dir.path().join("allowlist");
        fs::write(&queue, "").unwrap();
        fs::write(&allow, "").unwrap();
        let store = QueueStore::open(&queue, QueueCaps::default()).unwrap();
        store
            .submit(
                Some(IpAddr::from([127, 0, 0, 1])),
                &npub,
                "ops@example.com",
                1,
            )
            .unwrap();
        approve(&queue, &allow, &npub).unwrap();
        let allow_text = fs::read_to_string(&allow).unwrap();
        assert_eq!(allow_text.trim(), npub);
        let queue_text = fs::read_to_string(&queue).unwrap();
        assert!(
            !queue_text.contains(&npub),
            "approve must delete the pending line, not mark status: {:?}",
            queue_text
        );
        assert!(!queue_text.contains("approved"));
        assert!(load_queue(&queue).unwrap().is_empty());
    }

    #[test]
    fn reject_deletes_queue_line_only() {
        let pending = Keys::generate();
        let already = Keys::generate();
        let npub = npub_of(&pending);
        let keep = npub_of(&already);
        let dir = tempfile::tempdir().unwrap();
        let queue = dir.path().join("queue.csv");
        let allow = dir.path().join("allowlist");
        fs::write(&allow, format!("{}\n", keep)).unwrap();
        let store = QueueStore::open(&queue, QueueCaps::default()).unwrap();
        store
            .submit(
                Some(IpAddr::from([127, 0, 0, 1])),
                &npub,
                "ops@example.com",
                1,
            )
            .unwrap();
        reject(&queue, &npub).unwrap();
        let queue_text = fs::read_to_string(&queue).unwrap();
        assert!(
            !queue_text.contains(&npub),
            "reject must delete the queue line: {:?}",
            queue_text
        );
        assert!(!queue_text.contains("rejected"));
        assert!(load_queue(&queue).unwrap().is_empty());
        let allow_text = fs::read_to_string(&allow).unwrap();
        assert_eq!(allow_text, format!("{}\n", keep));
        assert!(!allow_text.contains(&npub));
    }

    #[test]
    fn import_remove_drops_npub() {
        let keys = Keys::generate();
        let npub = npub_of(&keys);
        let dir = tempfile::tempdir().unwrap();
        let allow = dir.path().join("allowlist");
        fs::write(&allow, format!("{}\n", npub)).unwrap();
        remove_npub(&allow, &npub).unwrap();
        let text = fs::read_to_string(&allow).unwrap();
        assert!(!text.contains(&npub));
        assert!(crate::auth::Allowlist::load(&allow)
            .unwrap()
            .snapshot()
            .is_empty());
    }

    /// Named contract: `splora-queue --bind --queue-file` is the queue HTTP
    /// entrypoint. The indexer clap app does not accept a `queue` subcommand.
    #[test]
    fn queue_http_entrypoint_binds_without_indexer_flags() {
        let m = queue_cli_app()
            .get_matches_from_safe(vec![
                "splora-queue",
                "--bind",
                "127.0.0.1:18493",
                "--queue-file",
                "/var/lib/splora/queue/import-queue",
            ])
            .expect("queue argv is bind plus queue-file only");
        assert_eq!(m.value_of("bind").unwrap(), "127.0.0.1:18493");
        assert_eq!(
            m.value_of("queue-file").unwrap(),
            "/var/lib/splora/queue/import-queue"
        );
        assert!(queue_cli_app()
            .get_matches_from_safe(vec!["splora-queue", "--network", "mainnet"])
            .is_err());
        assert!(crate::config::Config::indexer_clap_app()
            .get_matches_from_safe(vec!["splora", "queue", "--bind", "127.0.0.1:18493"])
            .is_err());
    }

    #[test]
    fn queue_cli_accepts_socket_file_without_bind() {
        let m = queue_cli_app()
            .get_matches_from_safe(vec![
                "splora-queue",
                "--socket-file",
                "/run/splora/queue.sock",
                "--queue-file",
                "/var/lib/splora/queue/import-queue",
            ])
            .expect("unix socket listen does not require --bind");
        assert_eq!(m.value_of("socket-file").unwrap(), "/run/splora/queue.sock");
        assert!(m.value_of("bind").is_none());
        assert_eq!(
            m.value_of("queue-file").unwrap(),
            "/var/lib/splora/queue/import-queue"
        );
    }

    #[test]
    fn queue_cli_refuses_bind_and_socket_file_together() {
        let err = queue_cli_app()
            .get_matches_from_safe(vec![
                "splora-queue",
                "--bind",
                "127.0.0.1:18493",
                "--socket-file",
                "/run/splora/queue.sock",
                "--queue-file",
                "/var/lib/splora/queue/import-queue",
            ])
            .expect_err("TCP --bind and --socket-file must not combine");
        let msg = format!("{}", err);
        assert!(
            msg.contains("bind") && msg.contains("socket-file"),
            "refuse combo with a clear error, got: {}",
            msg
        );
    }

    /// Named contract: `splora-import` help names approve, reject, remove and
    /// requires `--queue` / `--allowlist` on the subcommands that write those files.
    #[test]
    fn import_cli_help_names_approve_reject_remove() {
        let app = import_cli_app();
        let mut buf = Vec::new();
        app.write_help(&mut buf).unwrap();
        let help = String::from_utf8(buf).unwrap();
        assert!(help.contains("splora-import"));
        assert!(help.contains("approve"));
        assert!(help.contains("reject"));
        assert!(help.contains("remove"));
        assert!(import_cli_app()
            .get_matches_from_safe(vec!["splora-import", "approve", "npub1abc"])
            .is_err());
        let m = import_cli_app()
            .get_matches_from_safe(vec![
                "splora-import",
                "approve",
                "--queue",
                "/q",
                "--allowlist",
                "/a",
                "npub1abc",
            ])
            .expect("approve with --queue and --allowlist");
        let a = m.subcommand_matches("approve").unwrap();
        assert_eq!(a.value_of("queue").unwrap(), "/q");
        assert_eq!(a.value_of("allowlist").unwrap(), "/a");
    }

    /// Unix HTTP can POST concurrently. Both npubs must persist.
    #[test]
    fn queue_unix_concurrent_posts_keep_both_rows() {
        let a = Keys::generate();
        let b = Keys::generate();
        let npub_a = npub_of(&a);
        let npub_b = npub_of(&b);
        let caps = QueueCaps {
            max_rows: 100,
            max_bytes: 2 * 1024 * 1024,
            per_ip: 100,
            per_npub: 100,
            window_secs: 60,
        };
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("queue.csv");
        let store = Arc::new(QueueStore::open(&path, caps).unwrap());
        let sa = Arc::clone(&store);
        let sb = Arc::clone(&store);
        let na = npub_a.clone();
        let nb = npub_b.clone();
        let ha = std::thread::spawn(move || {
            sa.submit(Some(IpAddr::from([127, 0, 0, 1])), &na, "a@example.com", 1)
        });
        let hb = std::thread::spawn(move || {
            sb.submit(Some(IpAddr::from([127, 0, 0, 1])), &nb, "b@example.com", 1)
        });
        ha.join().unwrap().expect("first concurrent submit");
        hb.join().unwrap().expect("second concurrent submit");
        let rows = store.load().unwrap();
        assert_eq!(rows.len(), 2, "concurrent submits must both persist");
        let npubs: std::collections::HashSet<_> = rows.into_iter().map(|r| r.npub).collect();
        assert!(npubs.contains(&npub_a));
        assert!(npubs.contains(&npub_b));
    }

    /// Unix with no client address skips the per-IP bucket. Two npubs are
    /// not both charged to 127.0.0.1.
    #[test]
    fn queue_unix_rate_limit_by_npub_when_ip_unknown() {
        let caps = QueueCaps {
            max_rows: 100,
            max_bytes: 2 * 1024 * 1024,
            per_ip: 1,
            per_npub: 10,
            window_secs: 60,
        };
        let a = Keys::generate();
        let b = Keys::generate();
        let (_dir, store) = store_with(caps);
        store
            .submit(None, &npub_of(&a), "a@example.com", 1)
            .unwrap();
        store
            .submit(None, &npub_of(&b), "b@example.com", 1)
            .expect("two unix POSTs must not share one IP bucket");
        assert_eq!(store.load().unwrap().len(), 2);
        let same = Keys::generate();
        let npub = npub_of(&same);
        store
            .submit(Some(IpAddr::from([10, 0, 0, 1])), &npub, "c@example.com", 2)
            .unwrap();
        let other = Keys::generate();
        let err = store
            .submit(
                Some(IpAddr::from([10, 0, 0, 1])),
                &npub_of(&other),
                "d@example.com",
                2,
            )
            .unwrap_err();
        assert_eq!(err, QueueError::RateLimited);
    }

    #[test]
    fn unix_forwarded_for_first_hop_is_client_ip() {
        let req = hyper::Request::builder()
            .method("POST")
            .uri("/")
            .header("x-forwarded-for", "203.0.113.9, 10.0.0.1")
            .body(hyper::Body::empty())
            .unwrap();
        assert_eq!(unix_client_ip(&req), Some("203.0.113.9".parse().unwrap()));
        let bare = hyper::Request::builder()
            .method("POST")
            .uri("/")
            .body(hyper::Body::empty())
            .unwrap();
        assert_eq!(unix_client_ip(&bare), None);
    }

    fn wait_for_unix_socket(path: &Path) {
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
        while std::time::Instant::now() < deadline {
            if path.exists() {
                return;
            }
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
        panic!("unix queue socket did not appear: {:?}", path);
    }

    fn unix_post_json(sock: &Path, npub: &str, email: &str) -> hyper::Response<hyper::Body> {
        use hyperlocal::UnixClientExt;
        let client = hyper::Client::unix();
        let body = format!(r#"{{"npub":"{}","email":"{}"}}"#, npub, email);
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
        loop {
            let uri: hyper::Uri = hyperlocal::Uri::new(sock, "/").into();
            let req = hyper::Request::post(uri)
                .header("content-type", "application/json")
                .body(hyper::Body::from(body.clone()))
                .unwrap();
            match rt.block_on(client.request(req)) {
                Ok(resp) => return resp,
                Err(e) if std::time::Instant::now() < deadline => {
                    let _ = e;
                    std::thread::sleep(std::time::Duration::from_millis(20));
                }
                Err(e) => panic!("unix POST failed: {}", e),
            }
        }
    }

    /// Two unix POSTs without X-Forwarded-For must both succeed at per_ip=1.
    #[test]
    fn queue_unix_posts_do_not_share_one_ip_bucket() {
        let caps = QueueCaps {
            max_rows: 100,
            max_bytes: 2 * 1024 * 1024,
            per_ip: 1,
            per_npub: 10,
            window_secs: 60,
        };
        let a = Keys::generate();
        let b = Keys::generate();
        let dir = tempfile::tempdir().unwrap();
        let sock = dir.path().join("queue.sock");
        let queue = dir.path().join("queue.csv");
        std::thread::spawn({
            let sock = sock.clone();
            let queue = queue.clone();
            move || run_unix(sock, queue, caps).expect("unix queue")
        });
        wait_for_unix_socket(&sock);
        let ra = unix_post_json(&sock, &npub_of(&a), "a@example.com");
        let rb = unix_post_json(&sock, &npub_of(&b), "b@example.com");
        assert_eq!(ra.status().as_u16(), 200);
        assert_eq!(rb.status().as_u16(), 200);
        assert_eq!(load_queue(&queue).unwrap().len(), 2);
    }
}
