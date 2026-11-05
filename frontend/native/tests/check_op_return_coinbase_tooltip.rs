//! Cypress it() title: check op_return coinbase tooltip.
//!
//! The spec visits one mainnet block, hovers the first anchor in the second
//! table row, and expects `.tooltip-inner` to be visible. That tooltip is
//! the shared `check_op_return_coinbase_tooltip` result for the coinbase
//! input `scriptsig` (`vin.scriptsig | hex2ascii`). This it() does not read
//! a websocket body or an indexer body.
//! `paint_check_op_return_coinbase_tooltip` is not defined on `splora_native`.
//! This test does not open a window.
//!
//! Paint contract: one `tbody` line, then a `tooltip-inner` line whose next
//! line is exactly the shared `String` (an empty tooltip is an empty line,
//! so the paint ends with a newline after that text). The shared function
//! always returns a string. It does not return `None`.

const CYPRESS_IT: &str = "check op_return coinbase tooltip";

fn paint(scriptsig: &str) -> String {
    splora_native::paint_check_op_return_coinbase_tooltip(scriptsig)
}

fn shared(scriptsig: &str) -> String {
    splora_frontend_shared::check_op_return_coinbase_tooltip(scriptsig)
}

fn has_line(painted: &str, line: &str) -> bool {
    painted.lines().any(|row| row == line)
}

/// The line immediately after `tooltip-inner` is the shared tooltip text.
fn tooltip_text_line(painted: &str, text: &str) -> bool {
    let lines: Vec<&str> = painted.lines().collect();
    lines.iter().enumerate().any(|(index, row)| {
        *row == "tooltip-inner" && lines.get(index + 1).is_some_and(|next| *next == text)
    })
}

fn assert_painted_matches_shared(scriptsig: &str, why: &str) {
    let expected = shared(scriptsig);
    let painted = paint(scriptsig);
    assert!(
        has_line(&painted, "tbody"),
        "{CYPRESS_IT}: {why}; the block table row is missing; got {painted:?}"
    );
    assert!(
        has_line(&painted, "tooltip-inner") && tooltip_text_line(&painted, &expected),
        "{CYPRESS_IT}: {why}; painted lines must contain the shared tooltip {expected:?}; got {painted:?}"
    );
    if !expected.is_empty() {
        assert!(
            painted.contains(&expected),
            "{CYPRESS_IT}: {why}; the paint must contain the shared return {expected:?}; got {painted:?}"
        );
    }
}

#[test]
fn check_op_return_coinbase_tooltip() {
    let hello_scriptsig = "48656c6c6f";
    let world_scriptsig = "576f726c64";
    let hello = shared(hello_scriptsig);
    let world = shared(world_scriptsig);
    assert_ne!(
        hello, world,
        "{CYPRESS_IT}: two scriptsig inputs must return two different shared tooltips"
    );
    assert!(
        !hello.is_empty() && !world.is_empty(),
        "{CYPRESS_IT}: the two scriptsigs must both decode to a non-empty shared tooltip"
    );

    let painted_hello = paint(hello_scriptsig);
    assert!(
        has_line(&painted_hello, "tbody")
            && has_line(&painted_hello, "tooltip-inner")
            && tooltip_text_line(&painted_hello, &hello)
            && !tooltip_text_line(&painted_hello, &world)
            && painted_hello.contains(&hello),
        "{CYPRESS_IT}: cy.get('.tooltip-inner') is visible and the painted line is the shared tooltip {hello:?}, not {world:?}; got {painted_hello:?}"
    );

    let painted_world = paint(world_scriptsig);
    assert!(
        tooltip_text_line(&painted_world, &world)
            && !tooltip_text_line(&painted_world, &hello)
            && painted_world.contains(&world),
        "{CYPRESS_IT}: a second scriptsig paints its own shared tooltip {world:?}; got {painted_world:?}"
    );

    assert_painted_matches_shared(
        "48656C6C6F",
        "hex digits in the scriptsig are case-insensitive",
    );
    assert_painted_matches_shared("", "an empty scriptsig is an empty tooltip");
    assert_painted_matches_shared(
        "zz",
        "a non-hex pair is parseInt NaN, and ToUint8(NaN) is a NUL byte",
    );
    assert_painted_matches_shared("GG", "uppercase non-hex is also parseInt NaN");
    assert_painted_matches_shared(
        "0G",
        "parseInt stops at the first non-hex digit, so 0G is zero",
    );
    assert_painted_matches_shared("G0", "a window that does not start with a hex digit is NaN");
    assert_painted_matches_shared(
        "48656c6c6f4",
        "an odd-length scriptsig decodes the trailing nibble as one byte",
    );

    let long_scriptsig = "41".repeat(201);
    assert_painted_matches_shared(
        &long_scriptsig,
        "the pipe decodes the whole scriptsig and does not cut off at 200",
    );

    assert_painted_matches_shared("410042", "a NUL byte stays in the tooltip");
    assert_painted_matches_shared(
        "5c30",
        "the pipe removes the two-character sequence backslash zero, not NUL",
    );
    assert_painted_matches_shared(
        "5c3041",
        "backslash zero is removed and the following byte stays",
    );
    assert_painted_matches_shared(
        "41ff42",
        "an invalid UTF-8 byte is dropped instead of kept as U+FFFD",
    );
    assert_painted_matches_shared("c3", "a truncated UTF-8 sequence is dropped");
    assert_painted_matches_shared(
        "c341",
        "a lead byte that is not followed by a continuation is dropped, and the next byte is kept",
    );
    assert_painted_matches_shared("c3a9", "the tooltip is UTF-8, not one byte per character");
    assert_painted_matches_shared(
        "f09f9880",
        "a four-byte UTF-8 sequence decodes to one scalar",
    );
    assert_painted_matches_shared("efbbbf41", "TextDecoder strips one leading UTF-8 BOM");
    assert_painted_matches_shared(
        "41efbbbf42",
        "a BOM that is not at the start stays as U+FEFF",
    );
    assert_painted_matches_shared(
        "OP_PUSHBYTES_5 48656c6c6f",
        "the first token after OP_PUSH is the hex that the pipe decodes",
    );
    assert_painted_matches_shared(
        "OP_RETURN OP_PUSHBYTES_5 48656c6c6f OP_PUSHBYTES_3 616263",
        "only the first OP_PUSH payload is the tooltip",
    );
    assert_painted_matches_shared("OP_PUSHDATA1 41", "OP_PUSHDATA is an OP_PUSH prefix");
}
