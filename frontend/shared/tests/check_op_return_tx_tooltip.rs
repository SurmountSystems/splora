//! Cypress it() title: check op_return tx tooltip.
//!
//! The spec visits one mainnet block, hovers the first anchor in the second
//! table row, and expects `.tooltip-inner` to be visible. That anchor wraps
//! the short OP_RETURN badge. The badge tooltip is the `hex2ascii` pipe of
//! `scriptpubkey_asm` when the output type is `op_return`, the script is not
//! a runestone, the asm is not exactly `OP_RETURN`, and the decoded UTF-16
//! length is at most 200. This it() does not read a websocket body or an
//! indexer body.

use splora_frontend_shared::check_op_return_tx_tooltip as op_return_tx_tooltip;

#[test]
fn check_op_return_tx_tooltip() {
    let hello_script = "6a0548656c6c6f";
    let hello_asm = "OP_RETURN OP_PUSHBYTES_5 48656c6c6f";

    assert_eq!(
        op_return_tx_tooltip("", hello_script, "op_return", hello_asm).as_deref(),
        Some("Hello"),
        "cy.get('.tooltip-inner') is hex2ascii of the first OP_PUSH payload"
    );
    assert_eq!(
        op_return_tx_tooltip(
            "mainnet",
            hello_script,
            "op_return",
            "OP_RETURN OP_PUSHBYTES_5 576f726c64"
        )
        .as_deref(),
        Some("World"),
        "a second payload decodes to a different tooltip"
    );
    assert_eq!(
        op_return_tx_tooltip(
            "",
            hello_script,
            "op_return",
            "OP_RETURN OP_PUSHBYTES_5 48656C6C6F"
        )
        .as_deref(),
        Some("Hello"),
        "hex digits in the push are case-insensitive"
    );
    assert_eq!(
        op_return_tx_tooltip(
            "",
            hello_script,
            "op_return",
            "OP_RETURN OP_PUSHBYTES_5 48656c6c6f OP_PUSHBYTES_3 616263"
        )
        .as_deref(),
        Some("Hello"),
        "only the first OP_PUSH payload is the tooltip"
    );
    assert_eq!(
        op_return_tx_tooltip("", hello_script, "op_return", "48656c6c6f").as_deref(),
        Some("Hello"),
        "an asm with no OP_PUSH token decodes the whole string"
    );
    assert_eq!(
        op_return_tx_tooltip("", hello_script, "v0_p2wpkh", hello_asm).as_deref(),
        None,
        "a non-op_return output does not render the tooltip span"
    );
    assert_eq!(
        op_return_tx_tooltip("", "6a", "op_return", "OP_RETURN").as_deref(),
        None,
        "asm that is exactly OP_RETURN omits the badge span"
    );
    assert_eq!(
        op_return_tx_tooltip("", "6a5d061234567890abcd", "op_return", hello_asm).as_deref(),
        None,
        "a mainnet script that starts with 6a5d is a runestone and has no tooltip"
    );
    assert_eq!(
        op_return_tx_tooltip("testnet4", "6a5d061234567890abcd", "op_return", hello_asm).as_deref(),
        None,
        "runestone detection is not limited to the empty mainnet network name"
    );
    assert_eq!(
        op_return_tx_tooltip("liquid", "6a5d061234567890abcd", "op_return", hello_asm).as_deref(),
        Some("Hello"),
        "liquid does not mark runestones, so the tooltip stays"
    );
    assert_eq!(
        op_return_tx_tooltip(
            "liquidtestnet",
            "6a5d061234567890abcd",
            "op_return",
            hello_asm
        )
        .as_deref(),
        Some("Hello"),
        "liquid testnet does not mark runestones either"
    );
    assert_eq!(
        op_return_tx_tooltip("", "6a", "op_return", "").as_deref(),
        Some(""),
        "an empty asm still binds the tooltip when the badge span is present"
    );
    assert_eq!(
        op_return_tx_tooltip("", "6a02c3a9", "op_return", "OP_RETURN OP_PUSHBYTES_2 c3a9")
            .as_deref(),
        Some("é"),
        "the tooltip is UTF-8, not one byte per character"
    );
    assert_eq!(
        op_return_tx_tooltip(
            "",
            "6a03410042",
            "op_return",
            "OP_RETURN OP_PUSHBYTES_3 410042"
        )
        .as_deref(),
        Some("AB"),
        "a NUL byte is removed from the tooltip"
    );
    assert_eq!(
        op_return_tx_tooltip(
            "",
            "6a0341ff42",
            "op_return",
            "OP_RETURN OP_PUSHBYTES_3 41ff42"
        )
        .as_deref(),
        Some("AB"),
        "an invalid UTF-8 byte is dropped instead of kept as a replacement"
    );

    let two_hundred = vec![b'A'; 200];
    let two_hundred_asm = format!("OP_RETURN OP_PUSHDATA1 {}", hex_encode(&two_hundred));
    assert_eq!(
        op_return_tx_tooltip("", "6a", "op_return", &two_hundred_asm).as_deref(),
        Some(std::str::from_utf8(&two_hundred).unwrap()),
        "200 UTF-16 code units still use the short tooltip"
    );

    let two_hundred_one = vec![b'A'; 201];
    let two_hundred_one_asm = format!("OP_RETURN OP_PUSHDATA1 {}", hex_encode(&two_hundred_one));
    assert_eq!(
        op_return_tx_tooltip("", "6a", "op_return", &two_hundred_one_asm).as_deref(),
        None,
        "201 UTF-16 code units use the long badge and have no ngbTooltip"
    );

    let mut under_emoji = vec![b'A'; 198];
    under_emoji.extend_from_slice(&[0xF0, 0x9F, 0x98, 0x80]);
    let under_emoji_asm = format!("OP_RETURN OP_PUSHDATA1 {}", hex_encode(&under_emoji));
    let under_emoji_text = format!("{}😀", "A".repeat(198));
    assert_eq!(
        op_return_tx_tooltip("", "6a", "op_return", &under_emoji_asm).as_deref(),
        Some(under_emoji_text.as_str()),
        "198 ASCII characters plus one emoji are 200 UTF-16 code units"
    );

    let mut over_emoji = vec![b'A'; 199];
    over_emoji.extend_from_slice(&[0xF0, 0x9F, 0x98, 0x80]);
    let over_emoji_asm = format!("OP_RETURN OP_PUSHDATA1 {}", hex_encode(&over_emoji));
    assert_eq!(
        op_return_tx_tooltip("", "6a", "op_return", &over_emoji_asm).as_deref(),
        None,
        "199 ASCII characters plus one emoji are 201 UTF-16 code units"
    );
}

fn hex_encode(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut out = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        out.push(HEX[(byte >> 4) as usize] as char);
        out.push(HEX[(byte & 0x0f) as usize] as char);
    }
    out
}
