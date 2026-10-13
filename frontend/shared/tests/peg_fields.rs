//! Liquid peg-in and peg-out keys on the shared transaction JSON type.
//!
//! The indexer emits `is_pegin` and `pegout` only when its `liquid` feature
//! is on. A Bitcoin body omits both. The shared crate has no `liquid` feature.

use splora_frontend_shared::wire::Transaction;

const GENESIS_HASH: &str = "000000000019d6689c085ae165831e934ff763ae46a2a6c172b3f1b60a8ce26f";
const SCRIPT_PUBKEY: &str = "76a91489abcdefabbaabbaabbaabbaabbaabbaabbaabba88ac";
const SCRIPT_PUBKEY_ASM: &str = "OP_DUP OP_HASH160 OP_EQUALVERIFY OP_CHECKSIG";
const SCRIPT_PUBKEY_ADDRESS: &str = "1BxoGcMg14oaH3CwHD2hF4gU9VcfgX5yoR";

fn transaction_body() -> String {
    format!(
        r#"{{
            "txid": "peg-tx",
            "version": 2,
            "locktime": 0,
            "vin": [
                {{
                    "txid": "prev-true",
                    "vout": 0,
                    "prevout": null,
                    "scriptsig": "",
                    "scriptsig_asm": "",
                    "is_coinbase": false,
                    "sequence": 4294967295,
                    "is_pegin": true
                }},
                {{
                    "txid": "prev-false",
                    "vout": 1,
                    "prevout": null,
                    "scriptsig": "",
                    "scriptsig_asm": "",
                    "is_coinbase": false,
                    "sequence": 4294967295,
                    "is_pegin": false
                }},
                {{
                    "txid": "prev-omit",
                    "vout": 2,
                    "prevout": null,
                    "scriptsig": "",
                    "scriptsig_asm": "",
                    "is_coinbase": false,
                    "sequence": 4294967295
                }}
            ],
            "vout": [
                {{
                    "scriptpubkey": "51",
                    "scriptpubkey_asm": "OP_TRUE",
                    "scriptpubkey_type": "op_return",
                    "value": 0,
                    "pegout": {{
                        "genesis_hash": "{GENESIS_HASH}",
                        "scriptpubkey": "{SCRIPT_PUBKEY}",
                        "scriptpubkey_asm": "{SCRIPT_PUBKEY_ASM}",
                        "scriptpubkey_address": "{SCRIPT_PUBKEY_ADDRESS}"
                    }}
                }},
                {{
                    "scriptpubkey": "76a914bb88ac",
                    "scriptpubkey_asm": "OP_DUP",
                    "scriptpubkey_type": "p2pkh",
                    "value": 1000
                }},
                {{
                    "scriptpubkey": "6a",
                    "scriptpubkey_asm": "OP_RETURN",
                    "scriptpubkey_type": "op_return",
                    "value": 0,
                    "pegout": {{
                        "genesis_hash": "{GENESIS_HASH}",
                        "scriptpubkey": "{SCRIPT_PUBKEY}",
                        "scriptpubkey_asm": "{SCRIPT_PUBKEY_ASM}"
                    }}
                }}
            ],
            "size": 200,
            "weight": 800,
            "sigops": 1,
            "fee": 100
        }}"#
    )
}

#[test]
fn transaction_parses_is_pegin_and_pegout() {
    let tx: Transaction = serde_json::from_str(&transaction_body()).expect("peg transaction");

    assert_eq!(tx.vin[0].is_pegin, true);
    assert_eq!(tx.vin[1].is_pegin, false);
    assert_eq!(tx.vin[2].is_pegin, false);

    let pegout = tx.vout[0].pegout.as_ref().expect("pegout");
    assert_eq!(pegout.genesis_hash, GENESIS_HASH);
    assert_eq!(pegout.script_pubkey, SCRIPT_PUBKEY);
    assert_eq!(pegout.script_pubkey_asm, SCRIPT_PUBKEY_ASM);
    assert_eq!(
        pegout.script_pubkey_address.as_deref(),
        Some(SCRIPT_PUBKEY_ADDRESS)
    );

    assert_eq!(tx.vout[1].pegout, None);

    let without_address = tx.vout[2].pegout.as_ref().expect("pegout without address");
    assert_eq!(without_address.genesis_hash, GENESIS_HASH);
    assert_eq!(without_address.script_pubkey, SCRIPT_PUBKEY);
    assert_eq!(without_address.script_pubkey_asm, SCRIPT_PUBKEY_ASM);
    assert_eq!(without_address.script_pubkey_address, None);
}
