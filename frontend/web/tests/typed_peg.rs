//! Peg-in and peg-out come from the parsed transaction, not a raw key lookup.

#[test]
fn peg_labels_read_the_parsed_is_pegin_field() {
    let src = include_str!("../src/rows.rs");
    assert!(
        !src.contains(".get(\"is_pegin\")"),
        "rows.rs still scrapes the raw is_pegin key"
    );
    assert!(
        !src.contains(".get(\"pegout\")"),
        "rows.rs still scrapes the raw pegout key"
    );
    let start = src
        .find("fn peg_labels_from_tx")
        .expect("peg labels are not read from the parsed transaction");
    let rest = &src[start..];
    let end = rest.find("\nfn ").unwrap_or(rest.len());
    let body = &rest[..end];
    assert!(
        body.contains("is_pegin"),
        "parsed peg paint does not read is_pegin: {body}"
    );
    assert!(
        body.contains("pegout"),
        "parsed peg paint does not read pegout: {body}"
    );
    assert!(
        !body.contains("serde_json"),
        "parsed peg paint still scans raw JSON: {body}"
    );
    assert!(!body.contains("Fiat price last updated"), "{body}");
}
