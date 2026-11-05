//! Tooltip text for one OP_RETURN badge on the block transaction list.
//!
//! The Angular `hex2ascii` pipe and the transactions-list template decide
//! whether `ngbTooltip` is in the document. This is not an indexer body
//! and not a websocket body.

/// Tooltip inner text for one output, or `None` when that span is absent.
///
/// `network` is `stateService.network`. An empty string is mainnet.
/// `script_pubkey` is that output's script hex. On any network other than
/// `liquid` and `liquidtestnet`, a script that starts with `6a5d` is the
/// runestone the list marks, and a runestone renders a button instead of
/// the tooltip. `script_pubkey_type` must be `op_return`.
/// `script_pubkey_asm` is decoded by the `hex2ascii` pipe. The tooltip
/// span is present only when that asm is not exactly `OP_RETURN` and the
/// decoded text's UTF-16 length is at most 200.
pub fn check_op_return_tx_tooltip(
    network: &str,
    script_pubkey: &str,
    script_pubkey_type: &str,
    script_pubkey_asm: &str,
) -> Option<String> {
    if script_pubkey_type != "op_return" {
        return None;
    }
    if is_runestone(network, script_pubkey) {
        return None;
    }
    let decoded = hex2ascii(script_pubkey_asm);
    if utf16_len(&decoded) > 200 {
        return None;
    }
    if script_pubkey_asm == "OP_RETURN" {
        return None;
    }
    Some(decoded)
}

/// The list sets `isRunestone` only outside Liquid, and only when the
/// script hex starts with `6a5d` (`OP_RETURN` then `OP_13`).
fn is_runestone(network: &str, script_pubkey: &str) -> bool {
    network != "liquid" && network != "liquidtestnet" && script_pubkey.starts_with("6a5d")
}

/// `hex2ascii` pipe: the first token after an `OP_PUSH*` opcode, or the
/// whole string when that token is absent or empty. Each one- or two-code
/// unit window is `parseInt` base 16, then UTF-8 with replacement
/// characters and NUL bytes removed.
fn hex2ascii(asm: &str) -> String {
    let tokens: Vec<&str> = asm.split(' ').collect();
    let mut chosen = asm;
    for index in 1..tokens.len() {
        if tokens[index - 1].starts_with("OP_PUSH") {
            if !tokens[index].is_empty() {
                chosen = tokens[index];
            }
            break;
        }
    }
    if chosen.is_empty() {
        return String::new();
    }
    let raw = chosen.as_bytes();
    let mut bytes = Vec::with_capacity(raw.len().div_ceil(2));
    let mut index = 0;
    while index < raw.len() {
        let end = (index + 2).min(raw.len());
        let window = std::str::from_utf8(&raw[index..end]).unwrap_or("");
        bytes.push(to_uint8(parse_int_hex(window)));
        index += 2;
    }
    utf8_without_replacement_or_nul(&bytes)
}

fn utf16_len(text: &str) -> usize {
    text.encode_utf16().count()
}

/// `parseInt(text, 16)`. `None` is JavaScript `NaN`.
fn parse_int_hex(text: &str) -> Option<i32> {
    let bytes = text.as_bytes();
    let mut index = 0;
    while index < bytes.len() && is_js_trim_ascii(bytes[index]) {
        index += 1;
    }
    if index >= bytes.len() {
        return None;
    }
    let mut sign = 1i32;
    if bytes[index] == b'+' || bytes[index] == b'-' {
        if bytes[index] == b'-' {
            sign = -1;
        }
        index += 1;
    }
    if index >= bytes.len() || hex_digit(bytes[index]).is_none() {
        return None;
    }
    let mut value: i32 = 0;
    while index < bytes.len() {
        let Some(digit) = hex_digit(bytes[index]) else {
            break;
        };
        value = value.saturating_mul(16).saturating_add(i32::from(digit));
        index += 1;
    }
    Some(sign.saturating_mul(value))
}

fn to_uint8(parsed: Option<i32>) -> u8 {
    match parsed {
        Some(value) => value.rem_euclid(256) as u8,
        None => 0,
    }
}

fn hex_digit(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'a'..=b'f' => Some(byte - b'a' + 10),
        b'A'..=b'F' => Some(byte - b'A' + 10),
        _ => None,
    }
}

fn is_js_trim_ascii(byte: u8) -> bool {
    matches!(byte, b' ' | b'\t' | b'\n' | b'\r' | 0x0b | 0x0c)
}

/// Keep well-formed UTF-8 scalars. Drop each ill-formed byte and each NUL.
/// `TextDecoder` inserts U+FFFD for an ill-formed byte, and the pipe then
/// deletes those replacement characters and NUL.
fn utf8_without_replacement_or_nul(bytes: &[u8]) -> String {
    let mut out = String::new();
    let mut index = 0;
    while index < bytes.len() {
        let lead = bytes[index];
        if lead < 0x80 {
            if lead != 0 {
                out.push(lead as char);
            }
            index += 1;
            continue;
        }
        let (width, second_min, second_max) = match lead {
            0xC2..=0xDF => (2, 0x80, 0xBF),
            0xE0 => (3, 0xA0, 0xBF),
            0xED => (3, 0x80, 0x9F),
            0xE1..=0xEC | 0xEE..=0xEF => (3, 0x80, 0xBF),
            0xF0 => (4, 0x90, 0xBF),
            0xF1..=0xF3 => (4, 0x80, 0xBF),
            0xF4 => (4, 0x80, 0x8F),
            _ => {
                index += 1;
                continue;
            }
        };
        if index + width > bytes.len()
            || !continuations_fit(&bytes[index + 1..index + width], second_min, second_max)
        {
            index += 1;
            continue;
        }
        let sequence = &bytes[index..index + width];
        if let Ok(text) = std::str::from_utf8(sequence) {
            out.push_str(text);
            index += width;
        } else {
            index += 1;
        }
    }
    out
}

fn continuations_fit(rest: &[u8], second_min: u8, second_max: u8) -> bool {
    if rest.is_empty() || rest[0] < second_min || rest[0] > second_max {
        return false;
    }
    rest[1..].iter().all(|byte| (0x80..=0xBF).contains(byte))
}
