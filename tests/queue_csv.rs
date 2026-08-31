// SPDX-License-Identifier: Unlicense
//! Named two-file CSV queue contracts (public API). Lives here so the tests
//! can run while other crate modules still have in-flight test compile errors.

use electrs::queue::{
    approve, load_queue, queue_cli_app, reject, QueueCaps, QueueError, QueueStore,
};
use nostr::prelude::*;
use std::fs;
use std::net::IpAddr;
use std::sync::Arc;

fn npub_of(keys: &Keys) -> String {
    keys.public_key().to_bech32().unwrap()
}

fn store_with(caps: QueueCaps) -> (tempfile::TempDir, std::path::PathBuf, QueueStore) {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("queue.csv");
    let store = QueueStore::open(&path, caps).unwrap();
    (dir, path, store)
}

/// Disk is one `npub,email` CSV line. Not JSON. No status column.
#[test]
fn queue_persists_npub_comma_email_one_line() {
    let keys = Keys::generate();
    let npub = npub_of(&keys);
    let (_dir, path, store) = store_with(QueueCaps::default());
    store
        .submit(
            Some(IpAddr::from([127, 0, 0, 1])),
            &npub,
            "ops@example.com",
            1,
        )
        .unwrap();
    let text = fs::read_to_string(&path).unwrap();
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
    let (_dir, path, store) = store_with(QueueCaps::default());
    let err = store
        .submit(
            Some(IpAddr::from([127, 0, 0, 1])),
            &npub,
            "ops@example.com,evil",
            1,
        )
        .unwrap_err();
    assert_eq!(err, QueueError::InvalidEmail);
    let text = fs::read_to_string(&path).unwrap();
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
    let (_dir, path, store) = store_with(QueueCaps::default());
    let ip = IpAddr::from([127, 0, 0, 1]);
    store
        .submit(Some(ip), &npub, "a@example.com", 1_000)
        .unwrap();
    store
        .submit(Some(ip), &npub, "b@example.com", 1_001)
        .unwrap();
    let text = fs::read_to_string(&path).unwrap();
    let lines: Vec<&str> = text.lines().filter(|l| !l.is_empty()).collect();
    assert_eq!(lines.len(), 1);
    assert_eq!(lines[0], format!("{},{}", npub, "b@example.com"));
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

/// Two unix-style submits with no client IP must not share one IP bucket.
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
    let (_dir, _path, store) = store_with(caps);
    store
        .submit(None, &npub_of(&a), "a@example.com", 1)
        .unwrap();
    store
        .submit(None, &npub_of(&b), "b@example.com", 1)
        .expect("two unix POSTs must not share one IP bucket");
    assert_eq!(store.load().unwrap().len(), 2);
}
