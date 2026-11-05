//! `#blinded=` fragment check from `parseBlinders` and `verifyHex32`.

const INVALID_HEX: &str = "Error: Invalid blinding data (invalid hex)";

/// Fragment after `blinded=`.
///
/// `parseBlinders` splits on commas into amount, asset, value blinder, and
/// asset blinder. Amounts are numbers, so they are not checked as hex.
/// A missing asset or blinder, or one that is not 64 hex characters, returns
/// the invalid-hex sentence. When every required hex token passes, the
/// return is empty.
pub fn invalid_blinding_data_hex_error(fragment: &str) -> String {
    let mut parts = fragment.split(',');
    loop {
        let Some(_amount) = parts.next() else {
            break;
        };
        for _ in 0..3 {
            let Some(token) = parts.next() else {
                return INVALID_HEX.to_string();
            };
            if !is_hex32(token) {
                return INVALID_HEX.to_string();
            }
        }
    }
    String::new()
}

/// `verifyHex32`: `/^[0-9a-f]{64}$/i`.
fn is_hex32(token: &str) -> bool {
    let bytes = token.as_bytes();
    bytes.len() == 64
        && bytes
            .iter()
            .all(|byte| matches!(*byte, b'0'..=b'9' | b'a'..=b'f' | b'A'..=b'F'))
}
