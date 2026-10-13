//! `package_txid_markup` paints `error` from `POST /txs/package`.
//!
//! The broadcast page already posts that path for more than one raw
//! transaction and renders the body with `package_txid_markup`. A non-empty
//! `error` prints, including the text `0`. A missing key, JSON null, and an
//! empty string omit the label. This is not `reject-reason` on `POST /txs/test`
//! and not an indexer error sentence.

use splora_frontend_shared::broadcast_path;
use splora_web::{package_txid_markup, submission_links};

const TXID: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
const REASON: &str = "txn-mempool-conflict";

fn package_body(error_field: &str) -> String {
    format!(
        r#"{{"package_msg":"success","tx-results":{{"pkg-key":{{"txid":"{TXID}"{error_field}}}}}}}"#
    )
}

fn txid_link_count(markup: &str) -> usize {
    markup.matches(&format!("href=\"/tx/{TXID}\"")).count()
}

fn assert_omitted(markup: &str) {
    assert!(
        !markup.contains("package-error"),
        "an absent error invented a label: {markup}"
    );
    assert!(
        !markup.contains(REASON),
        "an absent error invented a reason: {markup}"
    );
    assert!(
        !markup.contains(">error <"),
        "an absent error invented an empty label: {markup}"
    );
    assert_eq!(txid_link_count(markup), 1, "{markup}");
    assert!(markup.contains("success"), "{markup}");
}

#[test]
fn package_error_prints_when_present_and_omits_when_missing() {
    let present = package_txid_markup(&package_body(&format!(r#","error":"{REASON}""#)));
    assert!(
        present.contains(&format!(
            "<span class=\"package-error\">error {REASON}</span>"
        )),
        "markup dropped the package error: {present}"
    );
    assert_eq!(txid_link_count(&present), 1, "{present}");
    assert!(present.contains("success"), "{present}");
    assert_eq!(
        submission_links(
            broadcast_path(),
            &package_body(&format!(r#","error":"{REASON}""#))
        ),
        present,
        "the broadcast page does not render the package error"
    );

    let zero = package_txid_markup(&package_body(r#","error":"0""#));
    assert!(
        zero.contains("<span class=\"package-error\">error 0</span>"),
        "the text 0 is a real error and was omitted: {zero}"
    );
    assert!(!zero.contains(REASON), "{zero}");

    let escaped = package_txid_markup(&package_body(r#","error":"a<b&c""#));
    assert!(
        escaped.contains("<span class=\"package-error\">error a&lt;b&amp;c</span>"),
        "the error was not escaped: {escaped}"
    );
    assert!(!escaped.contains("a<b"), "{escaped}");

    assert_omitted(&package_txid_markup(&package_body("")));
    assert_omitted(&package_txid_markup(&package_body(r#","error":null"#)));
    assert_omitted(&package_txid_markup(&package_body(r#","error":"""#)));
}
