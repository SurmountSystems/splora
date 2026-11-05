//! Cypress it() title: check op_return coinbase tooltip.
//!
//! The spec visits one mainnet block, hovers the first anchor in the second
//! table row, and expects `.tooltip-inner` to be visible. That anchor's
//! tooltip is the `hex2ascii` pipe of the coinbase input `scriptsig`. The
//! pipe has no 200-code-unit cutoff. An empty scriptsig is `""`. A non-hex
//! window is `parseInt` `NaN`, which becomes a NUL byte. An odd length keeps
//! the trailing nibble. This it() does not read a websocket body or an
//! indexer body.

#[test]
fn check_op_return_coinbase_tooltip() {
    use splora_frontend_shared::check_op_return_coinbase_tooltip;

    let hello = check_op_return_coinbase_tooltip("48656c6c6f");
    let world = check_op_return_coinbase_tooltip("576f726c64");
    assert_ne!(
        hello, world,
        "two scriptsig inputs must decode to two different tooltips"
    );
    assert_eq!(
        hello, "Hello",
        "cy.get('.tooltip-inner') is hex2ascii of the coinbase scriptsig"
    );
    assert_eq!(
        world, "World",
        "a second scriptsig decodes to a different tooltip"
    );
    assert_eq!(
        check_op_return_coinbase_tooltip("48656C6C6F"),
        "Hello",
        "hex digits in the scriptsig are case-insensitive"
    );
    assert_eq!(
        check_op_return_coinbase_tooltip(""),
        "",
        "an empty scriptsig is an empty tooltip"
    );
    assert_eq!(
        check_op_return_coinbase_tooltip("zz"),
        "\0",
        "a non-hex pair is parseInt NaN, and ToUint8(NaN) is a NUL byte"
    );
    assert_eq!(
        check_op_return_coinbase_tooltip("GG"),
        "\0",
        "uppercase non-hex is also parseInt NaN"
    );
    assert_eq!(
        check_op_return_coinbase_tooltip("0G"),
        "\0",
        "parseInt stops at the first non-hex digit, so 0G is zero"
    );
    assert_eq!(
        check_op_return_coinbase_tooltip("G0"),
        "\0",
        "a window that does not start with a hex digit is NaN"
    );
    assert_eq!(
        check_op_return_coinbase_tooltip("48656c6c6f4"),
        "Hello\u{4}",
        "an odd-length scriptsig decodes the trailing nibble as one byte"
    );
    let long_scriptsig = "41".repeat(201);
    let long_tooltip = "A".repeat(201);
    assert_eq!(
        check_op_return_coinbase_tooltip(&long_scriptsig),
        long_tooltip,
        "the pipe decodes the whole scriptsig and does not cut off at 200"
    );
    assert_eq!(
        check_op_return_coinbase_tooltip("410042"),
        "A\0B",
        "a NUL byte stays in the tooltip"
    );
    assert_eq!(
        check_op_return_coinbase_tooltip("5c30"),
        "",
        "the pipe removes the two-character sequence backslash zero, not NUL"
    );
    assert_eq!(
        check_op_return_coinbase_tooltip("5c3041"),
        "A",
        "backslash zero is removed and the following byte stays"
    );
    assert_eq!(
        check_op_return_coinbase_tooltip("41ff42"),
        "AB",
        "an invalid UTF-8 byte is dropped instead of kept as U+FFFD"
    );
    assert_eq!(
        check_op_return_coinbase_tooltip("c3"),
        "",
        "a truncated UTF-8 sequence is dropped"
    );
    assert_eq!(
        check_op_return_coinbase_tooltip("c341"),
        "A",
        "a lead byte that is not followed by a continuation is dropped, and the next byte is kept"
    );
    assert_eq!(
        check_op_return_coinbase_tooltip("c3a9"),
        "é",
        "the tooltip is UTF-8, not one byte per character"
    );
    assert_eq!(
        check_op_return_coinbase_tooltip("f09f9880"),
        "😀",
        "a four-byte UTF-8 sequence decodes to one scalar"
    );
    assert_eq!(
        check_op_return_coinbase_tooltip("efbbbf41"),
        "A",
        "TextDecoder strips one leading UTF-8 BOM"
    );
    assert_eq!(
        check_op_return_coinbase_tooltip("41efbbbf42"),
        "A\u{feff}B",
        "a BOM that is not at the start stays as U+FEFF"
    );
    assert_eq!(
        check_op_return_coinbase_tooltip("OP_PUSHBYTES_5 48656c6c6f"),
        "Hello",
        "the first token after OP_PUSH is the hex that the pipe decodes"
    );
    assert_eq!(
        check_op_return_coinbase_tooltip(
            "OP_RETURN OP_PUSHBYTES_5 48656c6c6f OP_PUSHBYTES_3 616263"
        ),
        "Hello",
        "only the first OP_PUSH payload is the tooltip"
    );
    assert_eq!(
        check_op_return_coinbase_tooltip("OP_PUSHDATA1 41"),
        "A",
        "OP_PUSHDATA is an OP_PUSH prefix"
    );
}
