//! `package_txid_markup` paints `other-wtxid` when that witness id is present.
//!
//! A missing key and an empty string are not a witness id. The txid is not
//! copied into that spot.

use splora_web::package_txid_markup;

/// 32-byte witness id. Not the entry's `txid` and not the `tx-results` map key.
const OTHER_WTXID: &str = "cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc";
const TXID: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
/// A witness id this package never sends. Markup must not invent it.
const MADE_UP_WTXID: &str = "dddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddd";

fn package_body(txid: &str, other_wtxid: Option<&str>) -> String {
    let fields = match other_wtxid {
        Some(wtxid) => format!(r#""txid":"{txid}","other-wtxid":"{wtxid}""#),
        None => format!(r#""txid":"{txid}""#),
    };
    format!(r#"{{"package_msg":"success","tx-results":{{"pkg-key":{{{fields}}}}}}}"#)
}

fn txid_link_count(markup: &str, txid: &str) -> usize {
    markup.matches(&format!("href=\"/tx/{txid}\"")).count()
}

/// A labeled witness slot, when the markup has one, must not be the txid.
fn witness_slot_copies_txid(markup: &str, txid: &str) -> bool {
    for label in ["other-wtxid", "witness", "wtxid"] {
        let Some(slot) = markup.split(label).nth(1) else {
            continue;
        };
        let rest = slot.trim_start_matches(|ch: char| !ch.is_ascii_alphanumeric());
        if rest.starts_with(txid) {
            return true;
        }
    }
    false
}

#[test]
fn package_txid_markup_includes_other_wtxid_when_present() {
    assert_ne!(OTHER_WTXID, TXID);
    assert_ne!(MADE_UP_WTXID, TXID);
    assert_ne!(MADE_UP_WTXID, OTHER_WTXID);

    let present = package_txid_markup(&package_body(TXID, Some(OTHER_WTXID)));
    assert!(
        present.contains(OTHER_WTXID),
        "markup dropped the witness id: {present}"
    );
    assert!(present.contains("success"), "{present}");
    assert_eq!(
        txid_link_count(&present, TXID),
        1,
        "the txid link is still the txid, not the witness id: {present}"
    );

    let missing = package_txid_markup(&package_body(TXID, None));
    assert!(missing.contains("success"), "{missing}");
    assert_eq!(
        txid_link_count(&missing, TXID),
        1,
        "a missing other-wtxid still links the txid once: {missing}"
    );
    assert!(
        !missing.contains(OTHER_WTXID),
        "a missing other-wtxid invented a witness id: {missing}"
    );
    assert!(
        !missing.contains(MADE_UP_WTXID),
        "a missing other-wtxid invented a witness id: {missing}"
    );
    assert!(
        !witness_slot_copies_txid(&missing, TXID),
        "a missing other-wtxid copied the txid into the witness slot: {missing}"
    );
    assert_eq!(
        missing.matches(TXID).count(),
        2,
        "the txid is the link href and the link text, not a second witness id: {missing}"
    );

    let empty = package_txid_markup(&package_body(TXID, Some("")));
    assert_eq!(
        empty, missing,
        "an empty other-wtxid is not a witness id: empty={empty} missing={missing}"
    );
    assert!(
        !empty.contains(OTHER_WTXID) && !empty.contains(MADE_UP_WTXID),
        "{empty}"
    );
    assert!(
        !witness_slot_copies_txid(&empty, TXID),
        "an empty other-wtxid copied the txid into the witness slot: {empty}"
    );
}
