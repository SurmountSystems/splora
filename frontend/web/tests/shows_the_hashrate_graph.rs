//! Static mining-dashboard host attribute for the hashrate graph element.
//!
//! Cypress only checks that the host exists. The attribute value is
//! `hashrate-graph`. The paint function is not defined in this file.
//! This test does not invent a hashrate figure.

#[test]
fn shows_the_hashrate_graph() {
    let host = splora_web::paint_shows_the_hashrate_graph();
    assert_eq!(host, "hashrate-graph");

    let path =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src/shows_the_hashrate_graph.rs");
    let product =
        std::fs::read_to_string(&path).unwrap_or_else(|error| panic!("read {path:?}: {error}"));
    assert!(
        !has_numeric_hashrate(&product),
        "frontend/web/src/shows_the_hashrate_graph.rs must not contain a numeric hashrate"
    );
}

/// A numeric hashrate is an ASCII digit. The host id has none.
fn has_numeric_hashrate(source: &str) -> bool {
    source.bytes().any(|byte| byte.is_ascii_digit())
}
