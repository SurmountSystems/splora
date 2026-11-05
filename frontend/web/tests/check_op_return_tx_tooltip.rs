//! Cypress it() title: `check op_return tx tooltip`.
//! File: `ref/mempool/frontend/cypress/e2e/mainnet/mainnet.spec.ts`.
//!
//! The it() visits one mainnet block, scrolls `.pagination` into view,
//! waits for the skeleton, then hovers the first anchor in the second
//! table row. The visible assertion is
//! `cy.get('.tooltip-inner').should('be.visible')`.
//! The tooltip text is `splora_frontend_shared::check_op_return_tx_tooltip`
//! for that output. This does not read an indexer body.
//! `paint_check_op_return_tx_tooltip` renders that shared return value.

use splora_frontend_shared::check_op_return_tx_tooltip as op_return_tx_tooltip;

#[test]
fn check_op_return_tx_tooltip() {
    let hello_script = "6a0548656c6c6f";
    let hello_asm = "OP_RETURN OP_PUSHBYTES_5 48656c6c6f";

    let qualifying = op_return_tx_tooltip("", hello_script, "op_return", hello_asm)
        .expect("a qualifying op_return payload returns tooltip text");
    assert!(
        !qualifying.is_empty(),
        "the qualifying tooltip is decoded payload text, not an empty span"
    );
    let painted = paint("", hello_script, "op_return", hello_asm);
    assert_eq!(
        tooltip_inner(&painted),
        Some(qualifying.as_str()),
        "cy.get('.tooltip-inner').should('be.visible') with the shared tooltip, markup was {painted}"
    );

    let other_asm = "OP_RETURN OP_PUSHBYTES_5 576f726c64";
    let other = op_return_tx_tooltip("mainnet", hello_script, "op_return", other_asm)
        .expect("a different op_return payload returns its own tooltip text");
    assert_ne!(
        other, qualifying,
        "a different payload must not produce the qualifying tooltip"
    );
    let other_painted = paint("mainnet", hello_script, "op_return", other_asm);
    assert_eq!(
        tooltip_inner(&other_painted),
        Some(other.as_str()),
        "a different payload paints its own shared tooltip, markup was {other_painted}"
    );
    assert!(
        !other_painted.contains(&qualifying),
        "a different payload must not produce the qualifying tooltip, markup was {other_painted}"
    );

    let runestone_script = "6a5d061234567890abcd";
    assert_eq!(
        op_return_tx_tooltip("", runestone_script, "op_return", hello_asm),
        None,
        "a mainnet script that starts with 6a5d is a runestone and has no tooltip"
    );
    let runestone = paint("", runestone_script, "op_return", hello_asm);
    assert!(
        tooltip_inner(&runestone).is_none(),
        "a runestone must not paint .tooltip-inner, markup was {runestone}"
    );
    assert!(
        !runestone.contains(&qualifying),
        "a runestone must not produce the qualifying tooltip, markup was {runestone}"
    );

    assert_eq!(
        op_return_tx_tooltip("", "6a", "op_return", "OP_RETURN"),
        None,
        "asm that is exactly OP_RETURN omits the badge span"
    );
    let bare = paint("", "6a", "op_return", "OP_RETURN");
    assert!(
        tooltip_inner(&bare).is_none(),
        "asm that is exactly OP_RETURN must not paint .tooltip-inner, markup was {bare}"
    );
    assert!(
        !bare.contains(&qualifying),
        "asm that is exactly OP_RETURN must not produce the qualifying tooltip, markup was {bare}"
    );

    assert_eq!(
        op_return_tx_tooltip("", hello_script, "v0_p2wpkh", hello_asm),
        None,
        "a non-op_return output does not render the tooltip span"
    );
    let other_type = paint("", hello_script, "v0_p2wpkh", hello_asm);
    assert!(
        tooltip_inner(&other_type).is_none(),
        "a non-op_return type must not paint .tooltip-inner, markup was {other_type}"
    );
    assert!(
        !other_type.contains(&qualifying),
        "a non-op_return type must not produce the qualifying tooltip, markup was {other_type}"
    );

    let long_text = "A".repeat(201);
    let long_asm = format!(
        "OP_RETURN OP_PUSHDATA1 {}",
        hex_encode(long_text.as_bytes())
    );
    assert_eq!(
        op_return_tx_tooltip("", "6a", "op_return", &long_asm),
        None,
        "201 UTF-16 code units use the long badge and have no ngbTooltip"
    );
    let long_painted = paint("", "6a", "op_return", &long_asm);
    assert!(
        tooltip_inner(&long_painted).is_none(),
        "decoded text over 200 UTF-16 code units must not paint .tooltip-inner, markup was {long_painted}"
    );
    assert!(
        !long_painted.contains(&long_text),
        "decoded text over 200 UTF-16 code units must not appear as a tooltip, markup was {long_painted}"
    );
    assert!(
        !long_painted.contains(&qualifying),
        "decoded text over 200 UTF-16 code units must not produce the qualifying tooltip, markup was {long_painted}"
    );
}

fn paint(
    network: &str,
    script_pubkey: &str,
    script_pubkey_type: &str,
    script_pubkey_asm: &str,
) -> String {
    splora_web::paint_check_op_return_tx_tooltip(
        network,
        script_pubkey,
        script_pubkey_type,
        script_pubkey_asm,
    )
}

/// Text inside `.tooltip-inner`. Absent when that span is not painted.
fn tooltip_inner(markup: &str) -> Option<&str> {
    const OPEN: &str = "class=\"tooltip-inner\">";
    let start = markup.find(OPEN)? + OPEN.len();
    let rest = &markup[start..];
    let end = rest.find("</span>")?;
    Some(&rest[..end])
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
