//! Liquid block header extra data on the shared block type.
//!
//! `BlockValue` in `src/rest.rs` emits `ext` only when the liquid feature is
//! on. A Bitcoin body omits the key. The shared crate has no `liquid` feature.
//! A missing key stays absent. The object is kept as JSON. There is no
//! elements crate and no unblinding.

use splora_frontend_shared::wire::Block;

fn bitcoin_block() -> &'static str {
    r#"{
        "id": "block-id-1",
        "height": 100,
        "version": 536870912,
        "timestamp": 1600000000,
        "tx_count": 12,
        "size": 1500,
        "weight": 4000,
        "merkle_root": "merkle-root-1",
        "previousblockhash": "prev-block-1",
        "mediantime": 1599990000,
        "nonce": 7,
        "bits": 386604799,
        "difficulty": 1.0
    }"#
}

fn block_with_proof_ext() -> String {
    format!(
        r#"{{
            "id": "liquid-proof",
            "height": 149999,
            "version": 536870912,
            "timestamp": 1640871853,
            "tx_count": 3,
            "size": 4380,
            "weight": 8097,
            "merkle_root": "merkle-root-proof",
            "previousblockhash": "prev-proof",
            "mediantime": 1640871554,
            "nonce": 0,
            "bits": 0,
            "difficulty": 0.0,
            "ext": {{
                "challenge": "51",
                "solution": "00"
            }}
        }}"#
    )
}

fn block_with_dynafed_ext() -> String {
    format!(
        r#"{{
            "id": "liquid-dynafed",
            "height": 150000,
            "version": 536870912,
            "timestamp": 1640871913,
            "tx_count": 2,
            "size": 3527,
            "weight": 7430,
            "merkle_root": "merkle-root-dynafed",
            "previousblockhash": "prev-dynafed",
            "mediantime": 1640871614,
            "nonce": 0,
            "bits": 0,
            "difficulty": 0.0,
            "ext": {{
                "current": {{"signblockscript": "51"}},
                "proposed": {{}},
                "signblock_witness": [[]]
            }}
        }}"#
    )
}

#[test]
fn block_parses_ext_and_omits_it_when_absent() {
    let plain: Block = serde_json::from_str(bitcoin_block()).expect("bitcoin block");
    assert_eq!(plain.ext, None);
    let encoded = serde_json::to_string(&plain).expect("encode bitcoin block");
    assert!(
        !encoded.contains("\"ext\""),
        "a missing ext key must stay absent, got {encoded}"
    );

    let proof: Block = serde_json::from_str(&block_with_proof_ext()).expect("proof ext");
    let ext = proof.ext.as_ref().expect("proof ext");
    assert_eq!(ext["challenge"], "51");
    assert_eq!(ext["solution"], "00");
    let proof_again: Block =
        serde_json::from_str(&serde_json::to_string(&proof).expect("encode proof"))
            .expect("round trip");
    assert_eq!(
        proof_again.ext.as_ref().expect("kept ext")["challenge"],
        "51"
    );
    assert_eq!(
        proof_again.ext.as_ref().expect("kept ext")["solution"],
        "00"
    );

    let dynafed: Block = serde_json::from_str(&block_with_dynafed_ext()).expect("dynafed ext");
    let ext = dynafed.ext.as_ref().expect("dynafed ext");
    assert_eq!(ext["current"]["signblockscript"], "51");
    assert!(ext["proposed"].is_object());
    assert_eq!(ext["signblock_witness"], serde_json::json!([[]]));
}
