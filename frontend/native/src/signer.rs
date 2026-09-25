//! NIP-98 kind 27235 signing. The header text is built by `splora-api`.

use std::str::FromStr;

use nostr::event::Event;
use nostr::event::FinalizeEvent;
use nostr::key::Keys;
use nostr::nips::nip98::{HttpData, HttpMethod, Sha256Hash};

/// A signed HTTP auth event plus the `Authorization` header value.
#[derive(Clone)]
pub struct SignedAuth {
    event: Event,
    event_json: String,
    authorization: String,
}

impl SignedAuth {
    pub fn kind(&self) -> u16 {
        self.event.kind.as_u16()
    }

    pub fn content(&self) -> &str {
        &self.event.content
    }

    pub fn event_json(&self) -> &str {
        &self.event_json
    }

    pub fn authorization(&self) -> &str {
        &self.authorization
    }

    pub fn verify(&self) -> Result<(), SignerError> {
        self.event.verify().map_err(|_| SignerError::Verify)
    }

    pub fn tag_value(&self, name: &str) -> Option<&str> {
        self.event
            .tags
            .iter()
            .find(|tag| tag.kind() == name)
            .and_then(|tag| tag.content())
    }

    pub fn has_tag(&self, name: &str) -> bool {
        self.tag_value(name).is_some()
    }
}

/// Signing failed. Variants do not carry key material.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SignerError {
    Url,
    Method,
    Payload,
    Sign,
    Verify,
    Header,
}

/// Sign an absolute URL. A non-empty body adds a `payload` tag of its SHA-256 hex.
pub fn sign_http(
    keys: &Keys,
    url: &str,
    method: &str,
    body: Option<&[u8]>,
) -> Result<SignedAuth, SignerError> {
    let parsed = url::Url::parse(url).map_err(|_| SignerError::Url)?;
    let method = HttpMethod::from_str(method).map_err(|_| SignerError::Method)?;
    let mut http = HttpData::new(parsed, method);
    if let Some(body) = body.filter(|body| !body.is_empty()) {
        let hex = sha256_hex(body);
        let hash = Sha256Hash::from_hex(&hex).map_err(|_| SignerError::Payload)?;
        http = http.payload(hash);
    }
    let event: Event = http.finalize(keys).map_err(|_| SignerError::Sign)?;
    event.verify().map_err(|_| SignerError::Verify)?;
    if event.kind.as_u16() != splora_api::KIND_NIP98 || !event.content.is_empty() {
        return Err(SignerError::Sign);
    }
    let event_json = event.as_json();
    if event_json.contains("nsec1") {
        return Err(SignerError::Header);
    }
    let authorization =
        splora_api::authorization_header(&event_json).map_err(|_| SignerError::Header)?;
    if !authorization.starts_with("Nostr ") || authorization.contains("nsec1") {
        return Err(SignerError::Header);
    }
    Ok(SignedAuth {
        event,
        event_json,
        authorization,
    })
}

pub fn sha256_hex(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let digest = sha256(bytes);
    let mut out = String::with_capacity(64);
    for byte in digest {
        out.push(HEX[(byte >> 4) as usize] as char);
        out.push(HEX[(byte & 0x0f) as usize] as char);
    }
    out
}

fn sha256(message: &[u8]) -> [u8; 32] {
    const K: [u32; 64] = [
        0x428A2F98, 0x71374491, 0xB5C0FBCF, 0xE9B5DBA5, 0x3956C25B, 0x59F111F1, 0x923F82A4,
        0xAB1C5ED5, 0xD807AA98, 0x12835B01, 0x243185BE, 0x550C7DC3, 0x72BE5D74, 0x80DEB1FE,
        0x9BDC06A7, 0xC19BF174, 0xE49B69C1, 0xEFBE4786, 0x0FC19DC6, 0x240CA1CC, 0x2DE92C6F,
        0x4A7484AA, 0x5CB0A9DC, 0x76F988DA, 0x983E5152, 0xA831C66D, 0xB00327C8, 0xBF597FC7,
        0xC6E00BF3, 0xD5A79147, 0x06CA6351, 0x14292967, 0x27B70A85, 0x2E1B2138, 0x4D2C6DFC,
        0x53380D13, 0x650A7354, 0x766A0ABB, 0x81C2C92E, 0x92722C85, 0xA2BFE8A1, 0xA81A664B,
        0xC24B8B70, 0xC76C51A3, 0xD192E819, 0xD6990624, 0xF40E3585, 0x106AA070, 0x19A4C116,
        0x1E376C08, 0x2748774C, 0x34B0BCB5, 0x391C0CB3, 0x4ED8AA4A, 0x5B9CCA4F, 0x682E6FF3,
        0x748F82EE, 0x78A5636F, 0x84C87814, 0x8CC70208, 0x90BEFFFA, 0xA4506CEB, 0xBEF9A3F7,
        0xC67178F2,
    ];
    let mut state: [u32; 8] = [
        0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a, 0x510e527f, 0x9b05688c, 0x1f83d9ab,
        0x5be0cd19,
    ];
    let bit_len = (message.len() as u64).saturating_mul(8);
    let mut data = Vec::with_capacity(message.len() + 72);
    data.extend_from_slice(message);
    data.push(0x80);
    while data.len() % 64 != 56 {
        data.push(0);
    }
    data.extend_from_slice(&bit_len.to_be_bytes());
    for chunk in data.chunks_exact(64) {
        let mut words = [0u32; 64];
        for (index, block) in chunk.chunks_exact(4).take(16).enumerate() {
            words[index] = u32::from_be_bytes([block[0], block[1], block[2], block[3]]);
        }
        for index in 16..64 {
            let s0 = words[index - 15].rotate_right(7)
                ^ words[index - 15].rotate_right(18)
                ^ (words[index - 15] >> 3);
            let s1 = words[index - 2].rotate_right(17)
                ^ words[index - 2].rotate_right(19)
                ^ (words[index - 2] >> 10);
            words[index] = words[index - 16]
                .wrapping_add(s0)
                .wrapping_add(words[index - 7])
                .wrapping_add(s1);
        }
        let mut a = state[0];
        let mut b = state[1];
        let mut c = state[2];
        let mut d = state[3];
        let mut e = state[4];
        let mut f = state[5];
        let mut g = state[6];
        let mut h = state[7];
        for index in 0..64 {
            let s1 = e.rotate_right(6) ^ e.rotate_right(11) ^ e.rotate_right(25);
            let ch = (e & f) ^ (!e & g);
            let temp1 = h
                .wrapping_add(s1)
                .wrapping_add(ch)
                .wrapping_add(K[index])
                .wrapping_add(words[index]);
            let s0 = a.rotate_right(2) ^ a.rotate_right(13) ^ a.rotate_right(22);
            let maj = (a & b) ^ (a & c) ^ (b & c);
            let temp2 = s0.wrapping_add(maj);
            h = g;
            g = f;
            f = e;
            e = d.wrapping_add(temp1);
            d = c;
            c = b;
            b = a;
            a = temp1.wrapping_add(temp2);
        }
        state[0] = state[0].wrapping_add(a);
        state[1] = state[1].wrapping_add(b);
        state[2] = state[2].wrapping_add(c);
        state[3] = state[3].wrapping_add(d);
        state[4] = state[4].wrapping_add(e);
        state[5] = state[5].wrapping_add(f);
        state[6] = state[6].wrapping_add(g);
        state[7] = state[7].wrapping_add(h);
    }
    let mut out = [0u8; 32];
    for (index, word) in state.iter().enumerate() {
        out[index * 4..index * 4 + 4].copy_from_slice(&word.to_be_bytes());
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use splora_api::{KIND_NIP98, Network, api_root};
    use splora_frontend_shared::blocks_tip_height_path;

    #[test]
    fn sha256_hex_matches_known_vectors() {
        assert_eq!(
            sha256_hex(b""),
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
        assert_eq!(
            sha256_hex(b"abc"),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
        assert_eq!(
            sha256_hex(b"hello"),
            "2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824"
        );
    }

    #[test]
    fn signer_produces_nip98_kind_27235() {
        let keys = Keys::generate();
        let url = format!("{}{}", api_root(Network::Mainnet), blocks_tip_height_path());
        let signed = sign_http(&keys, &url, "GET", None).expect("sign");
        assert_eq!(signed.kind(), 27235);
        assert_eq!(signed.kind(), KIND_NIP98);
        assert!(signed.has_tag("u"));
        assert!(signed.has_tag("method"));
        assert!(!signed.has_tag("payload"));
        assert_eq!(signed.tag_value("u"), Some(url.as_str()));
        assert_eq!(signed.tag_value("method"), Some("GET"));
        signed.verify().expect("signature");
        assert!(signed.content().is_empty());
        assert!(signed.authorization().starts_with("Nostr "));
        assert!(!signed.event_json().contains("nsec1"));
        assert!(!signed.authorization().contains("nsec1"));
    }

    #[test]
    fn signer_post_payload_is_body_sha256() {
        let keys = Keys::generate();
        let url = format!("{}/tx", api_root(Network::Mainnet));
        let body = b"hello";
        let signed = sign_http(&keys, &url, "POST", Some(body)).expect("sign");
        assert_eq!(signed.tag_value("payload"), Some(sha256_hex(body).as_str()));
        assert_eq!(
            signed.tag_value("payload"),
            Some("2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824")
        );
        assert_eq!(signed.tag_value("method"), Some("POST"));
        signed.verify().expect("signature");
        assert!(!signed.event_json().contains("nsec1"));
    }
}
