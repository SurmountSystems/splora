//! Tooltip text for the coinbase input scriptsig on the block transaction list.
//!
//! The Angular `hex2ascii` pipe is `vin.scriptsig | hex2ascii`. This is not
//! an indexer body and not a websocket body.

/// Tooltip inner text for one coinbase `scriptsig`.
///
/// The pipe returns `""` for an empty input. Each one- or two-code-unit
/// window is `parseInt` base 16. `NaN` becomes a NUL byte. The decoded
/// UTF-8 keeps that NUL. The pipe then deletes U+FFFD and the two-character
/// sequence backslash, digit zero. It does not stop at 200 code units.
pub fn check_op_return_coinbase_tooltip(scriptsig: &str) -> String {
    let chosen = first_op_push_payload(scriptsig);
    if chosen.is_empty() {
        return String::new();
    }
    let bytes = parse_hex_windows(chosen);
    let bytes = strip_one_leading_bom(&bytes);
    let decoded = utf8_without_ill_formed(&bytes);
    decoded.replace('\u{FFFD}', "").replace("\\0", "")
}

/// First token whose previous token starts with `OP_PUSH`, when that token
/// is non-empty. An empty token does not select a later push. Otherwise the
/// whole scriptsig is the hex.
fn first_op_push_payload(scriptsig: &str) -> &str {
    let tokens: Vec<&str> = scriptsig.split(' ').collect();
    for index in 1..tokens.len() {
        if tokens[index - 1].starts_with("OP_PUSH") {
            if !tokens[index].is_empty() {
                return tokens[index];
            }
            break;
        }
    }
    scriptsig
}

fn parse_hex_windows(hex: &str) -> Vec<u8> {
    let raw = hex.as_bytes();
    let mut bytes = Vec::with_capacity(raw.len().div_ceil(2));
    let mut index = 0;
    while index < raw.len() {
        let end = (index + 2).min(raw.len());
        let window = std::str::from_utf8(&raw[index..end]).unwrap_or("");
        bytes.push(to_uint8(parse_int_hex(window)));
        index += 2;
    }
    bytes
}

/// `TextDecoder` drops one leading UTF-8 BOM. A later BOM stays.
fn strip_one_leading_bom(bytes: &[u8]) -> &[u8] {
    if bytes.len() >= 3 && bytes[0] == 0xEF && bytes[1] == 0xBB && bytes[2] == 0xBF {
        &bytes[3..]
    } else {
        bytes
    }
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

/// Keep well-formed UTF-8 scalars, including NUL. Drop each ill-formed byte.
/// `TextDecoder` inserts U+FFFD for that byte, and the pipe then deletes
/// those replacement characters.
fn utf8_without_ill_formed(bytes: &[u8]) -> String {
    let mut out = String::new();
    let mut index = 0;
    while index < bytes.len() {
        let lead = bytes[index];
        if lead < 0x80 {
            out.push(lead as char);
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
