//! Cypress it() title: `check op_return coinbase tooltip`.
//! File: `ref/mempool/frontend/cypress/e2e/mainnet/mainnet.spec.ts`.
//!
//! The it() visits one mainnet block, scrolls `.pagination` into view,
//! waits for the skeleton, then hovers the first anchor in the second
//! table row. The visible assertion is
//! `cy.get('.tooltip-inner').should('be.visible')`.
//! The template binding is `[ngbTooltip]="vin.scriptsig | hex2ascii"`.
//! `paint_check_op_return_coinbase_tooltip` renders
//! `splora_frontend_shared::check_op_return_coinbase_tooltip`. This does
//! not reimplement the hex pipe and does not read an indexer body.

use splora_frontend_shared::check_op_return_coinbase_tooltip as coinbase_tooltip;

#[test]
fn check_op_return_coinbase_tooltip() {
    let hello_sig = "48656c6c6f";
    let world_sig = "576f726c64";
    let hello = coinbase_tooltip(hello_sig);
    let world = coinbase_tooltip(world_sig);
    assert_ne!(
        hello, world,
        "two scriptsig inputs must decode to two different tooltips"
    );
    assert_eq!(hello, "Hello");
    assert_eq!(world, "World");
    assert_painted(hello_sig, &hello);
    assert_painted(world_sig, &world);

    assert_case("48656C6C6F", "Hello");
    assert_case("", "");
    assert_case("zz", "\0");
    assert_case("GG", "\0");
    assert_case("0G", "\0");
    assert_case("G0", "\0");
    assert_case("48656c6c6f4", "Hello\u{4}");
    let long_scriptsig = "41".repeat(201);
    let long_tooltip = "A".repeat(201);
    assert_case(&long_scriptsig, &long_tooltip);
    assert_case("410042", "A\0B");
    assert_case("5c30", "");
    assert_case("5c3041", "A");
    assert_case("41ff42", "AB");
    assert_case("c3", "");
    assert_case("c341", "A");
    assert_case("c3a9", "é");
    assert_case("f09f9880", "😀");
    assert_case("efbbbf41", "A");
    assert_case("41efbbbf42", "A\u{feff}B");
    assert_case("OP_PUSHBYTES_5 48656c6c6f", "Hello");
    assert_case(
        "OP_RETURN OP_PUSHBYTES_5 48656c6c6f OP_PUSHBYTES_3 616263",
        "Hello",
    );
    assert_case("OP_PUSHDATA1 41", "A");
}

fn assert_case(scriptsig: &str, expected: &str) {
    let shared = coinbase_tooltip(scriptsig);
    assert_eq!(
        shared, expected,
        "shared tooltip for {scriptsig:?} must match the shared test"
    );
    assert_painted(scriptsig, &shared);
}

fn assert_painted(scriptsig: &str, shared: &str) {
    let painted = splora_web::paint_check_op_return_coinbase_tooltip(scriptsig);
    if shared.is_empty() {
        assert!(
            tooltip_inner(&painted).is_none(),
            "an empty shared return must not paint .tooltip-inner, markup was {painted}"
        );
        assert!(
            !painted.contains("tooltip-inner"),
            "an empty shared return must not invent tooltip markup, markup was {painted}"
        );
        return;
    }
    assert_eq!(
        tooltip_inner(&painted),
        Some(shared),
        "painted .tooltip-inner text must equal the shared return, markup was {painted}"
    );
}

/// Text inside `.tooltip-inner`. Absent when that span is not painted.
fn tooltip_inner(markup: &str) -> Option<&str> {
    const OPEN: &str = "class=\"tooltip-inner\">";
    let start = markup.find(OPEN)? + OPEN.len();
    let rest = &markup[start..];
    let end = rest.find("</span>")?;
    Some(&rest[..end])
}
