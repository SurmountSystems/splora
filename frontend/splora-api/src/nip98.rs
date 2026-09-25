//! NIP-98 kind 27235 `Authorization` header builder.
//! This module does not sign and does not generate keys.

/// HTTP auth event kind from NIP-98.
pub const KIND_NIP98: u16 = 27235;

const BASE64_ALPHABET: &[u8; 64] =
    b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

/// Rejection from the header builder. This crate has no signing path.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Nip98Error {
    pub message: &'static str,
}

/// Tags for a NIP-98 event: `u`, `method`, and `payload` only when a hash is present.
pub fn nip98_tags(url: &str, method: &str, payload_sha256_hex: Option<&str>) -> Vec<Vec<String>> {
    let mut tags = vec![
        vec!["u".to_string(), url.to_string()],
        vec!["method".to_string(), method.to_string()],
    ];
    if let Some(hash) = payload_sha256_hex {
        tags.push(vec!["payload".to_string(), hash.to_string()]);
    }
    tags
}

/// `Nostr ` plus standard base64 of `event_json`. A body that carries a secret is rejected.
pub fn authorization_header(event_json: &str) -> Result<String, Nip98Error> {
    if event_json.contains("nsec1") {
        return Err(Nip98Error {
            message: "this crate does not accept a secret",
        });
    }
    let encoded = standard_base64(event_json.as_bytes());
    Ok(format!("Nostr {encoded}"))
}

/// RFC 4648 standard base64 with padding and no line breaks.
fn standard_base64(input: &[u8]) -> String {
    let mut out = String::with_capacity(input.len().div_ceil(3) * 4);
    let mut chunks = input.chunks_exact(3);
    for chunk in chunks.by_ref() {
        push_quantum(&mut out, chunk[0], chunk[1], chunk[2], 4);
    }
    match chunks.remainder() {
        [b0] => push_quantum(&mut out, *b0, 0, 0, 2),
        [b0, b1] => push_quantum(&mut out, *b0, *b1, 0, 3),
        _ => {}
    }
    out
}

fn push_quantum(out: &mut String, b0: u8, b1: u8, b2: u8, significant: usize) {
    let n = (u32::from(b0) << 16) | (u32::from(b1) << 8) | u32::from(b2);
    let indexes = [
        ((n >> 18) & 0x3f) as usize,
        ((n >> 12) & 0x3f) as usize,
        ((n >> 6) & 0x3f) as usize,
        (n & 0x3f) as usize,
    ];
    for index in indexes.iter().take(significant) {
        out.push(BASE64_ALPHABET[*index] as char);
    }
    for _ in significant..4 {
        out.push('=');
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn kind_is_nip98() {
        assert_eq!(KIND_NIP98, 27235);
    }

    #[test]
    fn standard_base64_pads_one_two_and_three_bytes() {
        assert_eq!(standard_base64(b"f"), "Zg==");
        assert_eq!(standard_base64(b"fo"), "Zm8=");
        assert_eq!(standard_base64(b"foo"), "Zm9v");
    }

    #[test]
    fn authorization_header_is_nostr_plus_padded_base64() {
        let header = authorization_header(r#"{"kind":27235}"#).expect("json has no secret");
        assert_eq!(header, "Nostr eyJraW5kIjoyNzIzNX0=");
        assert!(!header.contains('\n'));
    }

    #[test]
    fn authorization_header_rejects_a_secret() {
        let rejected = authorization_header("body mentions nsec1");
        assert_eq!(
            rejected,
            Err(Nip98Error {
                message: "this crate does not accept a secret",
            })
        );
    }

    #[test]
    fn get_without_payload_has_u_then_method_only() {
        let tags = nip98_tags("https://splora.surmount.systems/api/tx/abc", "GET", None);
        assert_eq!(
            tags,
            vec![
                vec![
                    "u".to_string(),
                    "https://splora.surmount.systems/api/tx/abc".to_string()
                ],
                vec!["method".to_string(), "GET".to_string()],
            ]
        );
        assert!(tags.iter().all(|tag| tag[0] != "payload"));
    }

    #[test]
    fn post_payload_tag_keeps_the_given_hex() {
        let tags = nip98_tags(
            "https://splora.surmount.systems/api",
            "POST",
            Some("abc123"),
        );
        assert_eq!(tags.len(), 3);
        assert_eq!(tags[2], vec!["payload".to_string(), "abc123".to_string()]);
    }
}
