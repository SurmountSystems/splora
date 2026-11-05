//! Cypress it() title: check op_return tx tooltip.
//!
//! The spec visits one mainnet block, hovers the first anchor in the second
//! table row, and expects `.tooltip-inner` to be visible. That anchor wraps
//! the short OP_RETURN badge. The badge tooltip is the shared
//! `check_op_return_tx_tooltip` result: present only for `op_return`, not a
//! runestone, asm not exactly `OP_RETURN`, and decoded UTF-16 length at most
//! 200. This it() does not read a websocket body or an indexer body.
//! `paint_check_op_return_tx_tooltip` is not defined on `splora_native`.
//! This test does not open a window.
//!
//! Paint contract: one `tbody` line, then when the shared function returns
//! `Some(text)` a `tooltip-inner` line whose next line is exactly `text`
//! (an empty tooltip is an empty line, so the paint ends with a newline
//! after that text). When the shared function returns `None`, `tooltip-inner`
//! is absent.

const CYPRESS_IT: &str = "check op_return tx tooltip";

fn paint(
    network: &str,
    script_pubkey: &str,
    script_pubkey_type: &str,
    script_pubkey_asm: &str,
) -> String {
    splora_native::paint_check_op_return_tx_tooltip(
        network,
        script_pubkey,
        script_pubkey_type,
        script_pubkey_asm,
    )
}

fn shared(
    network: &str,
    script_pubkey: &str,
    script_pubkey_type: &str,
    script_pubkey_asm: &str,
) -> Option<String> {
    splora_frontend_shared::check_op_return_tx_tooltip(
        network,
        script_pubkey,
        script_pubkey_type,
        script_pubkey_asm,
    )
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

fn assert_painted_matches_shared(
    network: &str,
    script_pubkey: &str,
    script_pubkey_type: &str,
    script_pubkey_asm: &str,
    why: &str,
) {
    let expected = shared(
        network,
        script_pubkey,
        script_pubkey_type,
        script_pubkey_asm,
    );
    let painted = paint(
        network,
        script_pubkey,
        script_pubkey_type,
        script_pubkey_asm,
    );
    assert!(
        has_line(&painted, "tbody"),
        "{CYPRESS_IT}: {why}; the block table row is missing; got {painted:?}"
    );
    match expected {
        Some(text) => {
            assert!(
                has_line(&painted, "tooltip-inner") && tooltip_text_line(&painted, &text),
                "{CYPRESS_IT}: {why}; painted lines must contain the shared tooltip {text:?}; got {painted:?}"
            );
        }
        None => {
            assert!(
                !has_line(&painted, "tooltip-inner"),
                "{CYPRESS_IT}: {why}; .tooltip-inner is absent when the shared function returns None; got {painted:?}"
            );
        }
    }
}

#[test]
fn check_op_return_tx_tooltip() {
    let hello_script = "6a0548656c6c6f";
    let hello_asm = "OP_RETURN OP_PUSHBYTES_5 48656c6c6f";

    let hello = shared("", hello_script, "op_return", hello_asm);
    let world = shared(
        "mainnet",
        hello_script,
        "op_return",
        "OP_RETURN OP_PUSHBYTES_5 576f726c64",
    );
    assert!(
        hello.is_some() && world.is_some() && hello != world,
        "{CYPRESS_IT}: two qualifying payloads must return two different shared tooltips"
    );

    let painted_hello = paint("", hello_script, "op_return", hello_asm);
    let hello_text = hello.as_deref().unwrap();
    let world_text = world.as_deref().unwrap();
    assert!(
        has_line(&painted_hello, "tbody")
            && has_line(&painted_hello, "tooltip-inner")
            && tooltip_text_line(&painted_hello, hello_text)
            && !has_line(&painted_hello, world_text),
        "{CYPRESS_IT}: cy.get('.tooltip-inner') is visible and the painted line is the shared tooltip {hello_text:?}, not {world_text:?}; got {painted_hello:?}"
    );

    let painted_world = paint(
        "mainnet",
        hello_script,
        "op_return",
        "OP_RETURN OP_PUSHBYTES_5 576f726c64",
    );
    assert!(
        tooltip_text_line(&painted_world, world_text) && !has_line(&painted_world, hello_text),
        "{CYPRESS_IT}: a second payload paints its own shared tooltip {world_text:?}; got {painted_world:?}"
    );

    assert_painted_matches_shared(
        "",
        hello_script,
        "op_return",
        "OP_RETURN OP_PUSHBYTES_5 48656C6C6F",
        "hex digits in the push are case-insensitive",
    );
    assert_painted_matches_shared(
        "",
        hello_script,
        "op_return",
        "OP_RETURN OP_PUSHBYTES_5 48656c6c6f OP_PUSHBYTES_3 616263",
        "only the first OP_PUSH payload is the tooltip",
    );
    assert_painted_matches_shared(
        "",
        hello_script,
        "op_return",
        "48656c6c6f",
        "an asm with no OP_PUSH token decodes the whole string",
    );
    assert_painted_matches_shared(
        "",
        hello_script,
        "v0_p2wpkh",
        hello_asm,
        "a non-op_return output does not render the tooltip span",
    );
    assert_painted_matches_shared(
        "",
        "6a",
        "op_return",
        "OP_RETURN",
        "asm that is exactly OP_RETURN omits the badge span",
    );
    assert_painted_matches_shared(
        "",
        "6a5d061234567890abcd",
        "op_return",
        hello_asm,
        "a mainnet script that starts with 6a5d is a runestone and has no tooltip",
    );
    assert_painted_matches_shared(
        "testnet4",
        "6a5d061234567890abcd",
        "op_return",
        hello_asm,
        "runestone detection is not limited to the empty mainnet network name",
    );
    assert_painted_matches_shared(
        "liquid",
        "6a5d061234567890abcd",
        "op_return",
        hello_asm,
        "liquid does not mark runestones, so the tooltip stays",
    );
    assert_painted_matches_shared(
        "liquidtestnet",
        "6a5d061234567890abcd",
        "op_return",
        hello_asm,
        "liquid testnet does not mark runestones either",
    );
    assert_painted_matches_shared(
        "",
        "6a",
        "op_return",
        "",
        "an empty asm still binds the tooltip when the badge span is present",
    );
    assert_painted_matches_shared(
        "",
        "6a02c3a9",
        "op_return",
        "OP_RETURN OP_PUSHBYTES_2 c3a9",
        "the tooltip is UTF-8, not one byte per character",
    );
    assert_painted_matches_shared(
        "",
        "6a03410042",
        "op_return",
        "OP_RETURN OP_PUSHBYTES_3 410042",
        "a NUL byte is removed from the tooltip",
    );
    assert_painted_matches_shared(
        "",
        "6a0341ff42",
        "op_return",
        "OP_RETURN OP_PUSHBYTES_3 41ff42",
        "an invalid UTF-8 byte is dropped instead of kept as a replacement",
    );

    let two_hundred = vec![b'A'; 200];
    let two_hundred_asm = format!("OP_RETURN OP_PUSHDATA1 {}", hex_encode(&two_hundred));
    assert_painted_matches_shared(
        "",
        "6a",
        "op_return",
        &two_hundred_asm,
        "200 UTF-16 code units still use the short tooltip",
    );

    let two_hundred_one = vec![b'A'; 201];
    let two_hundred_one_asm = format!("OP_RETURN OP_PUSHDATA1 {}", hex_encode(&two_hundred_one));
    assert_painted_matches_shared(
        "",
        "6a",
        "op_return",
        &two_hundred_one_asm,
        "201 UTF-16 code units use the long badge and have no ngbTooltip",
    );

    let mut under_emoji = vec![b'A'; 198];
    under_emoji.extend_from_slice(&[0xF0, 0x9F, 0x98, 0x80]);
    let under_emoji_asm = format!("OP_RETURN OP_PUSHDATA1 {}", hex_encode(&under_emoji));
    assert_painted_matches_shared(
        "",
        "6a",
        "op_return",
        &under_emoji_asm,
        "198 ASCII characters plus one emoji are 200 UTF-16 code units",
    );

    let mut over_emoji = vec![b'A'; 199];
    over_emoji.extend_from_slice(&[0xF0, 0x9F, 0x98, 0x80]);
    let over_emoji_asm = format!("OP_RETURN OP_PUSHDATA1 {}", hex_encode(&over_emoji));
    assert_painted_matches_shared(
        "",
        "6a",
        "op_return",
        &over_emoji_asm,
        "199 ASCII characters plus one emoji are 201 UTF-16 code units",
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
