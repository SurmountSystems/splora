use crate::chain::{BlockHash, OutPoint, Transaction, TxIn, TxOut, Txid};
use crate::errors;
use crate::util::{BlockId, IsProvablyUnspendable};

use std::collections::HashMap;

#[cfg(feature = "liquid")]
lazy_static! {
    static ref REGTEST_INITIAL_ISSUANCE_PREVOUT: Txid =
        "50cdc410c9d0d61eeacc531f52d2c70af741da33af127c364e52ac1ee7c030a5"
            .parse()
            .unwrap();
    static ref TESTNET_INITIAL_ISSUANCE_PREVOUT: Txid =
        "0c52d2526a5c9f00e9fb74afd15dd3caaf17c823159a514f929ae25193a43a52"
            .parse()
            .unwrap();
}

#[derive(Serialize, Deserialize)]
pub struct TransactionStatus {
    pub confirmed: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub block_height: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub block_hash: Option<BlockHash>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub block_time: Option<u32>,
}

impl From<Option<BlockId>> for TransactionStatus {
    fn from(blockid: Option<BlockId>) -> TransactionStatus {
        match blockid {
            Some(b) => TransactionStatus {
                confirmed: true,
                block_height: Some(b.height),
                block_hash: Some(b.hash),
                block_time: Some(b.time),
            },
            None => TransactionStatus {
                confirmed: false,
                block_height: None,
                block_hash: None,
                block_time: None,
            },
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct TxInput {
    pub txid: Txid,
    pub vin: u32,
}

pub fn is_coinbase(txin: &TxIn) -> bool {
    #[cfg(not(feature = "liquid"))]
    return txin.previous_output.is_null();
    #[cfg(feature = "liquid")]
    return txin.is_coinbase();
}

pub fn has_prevout(txin: &TxIn) -> bool {
    #[cfg(not(feature = "liquid"))]
    return !txin.previous_output.is_null();
    #[cfg(feature = "liquid")]
    return !txin.is_coinbase()
        && !txin.is_pegin
        && txin.previous_output.txid != *REGTEST_INITIAL_ISSUANCE_PREVOUT
        && txin.previous_output.txid != *TESTNET_INITIAL_ISSUANCE_PREVOUT;
}

pub fn is_spendable(txout: &TxOut) -> bool {
    #[cfg(not(feature = "liquid"))]
    return !txout.script_pubkey.is_provably_unspendable_();
    #[cfg(feature = "liquid")]
    return !txout.is_fee() && !txout.script_pubkey.is_provably_unspendable_();
}

/// Extract the previous TxOuts of a Transaction's TxIns
///
/// # Errors
///
/// This function MUST NOT return an error variant when allow_missing is true.
/// If allow_missing is false, it will return an error when any Outpoint is
/// missing from the keys of the txos argument's HashMap.
pub fn extract_tx_prevouts<'a>(
    tx: &Transaction,
    txos: &'a HashMap<OutPoint, TxOut>,
) -> Result<HashMap<u32, &'a TxOut>, errors::Error> {
    tx.input
        .iter()
        .enumerate()
        .filter(|(_, txi)| has_prevout(txi))
        .map(|(index, txi)| {
            Ok((
                index as u32,
                match txos.get(&txi.previous_output) {
                    Some(txo) => txo,
                    None => {
                        return Err(format!("missing outpoint {:?}", txi.previous_output).into());
                    }
                },
            ))
        })
        .collect()
}

pub fn serialize_outpoint<S>(outpoint: &OutPoint, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::ser::Serializer,
{
    use serde::ser::SerializeStruct;
    let mut s = serializer.serialize_struct("OutPoint", 2)?;
    s.serialize_field("txid", &outpoint.txid)?;
    s.serialize_field("vout", &outpoint.vout)?;
    s.end()
}

pub(super) mod sigops {
    use crate::chain::{
        Transaction, TxOut, Witness,
        opcodes::all::{OP_CHECKMULTISIG, OP_CHECKMULTISIGVERIFY, OP_CHECKSIG, OP_CHECKSIGVERIFY},
        script::{self, Instruction},
    };
    #[cfg(not(feature = "liquid"))]
    use bitcoin::opcodes::Opcode;
    #[cfg(feature = "liquid")]
    use elements::opcodes::All as Opcode;
    use std::collections::HashMap;

    /// Get sigop count for transaction. prevout_map must have all the prevouts
    /// except peg-in inputs, which have no sidechain prevout.
    pub fn transaction_sigop_count(
        tx: &Transaction,
        prevout_map: &HashMap<u32, &TxOut>,
    ) -> Result<usize, script::Error> {
        let input_count = tx.input.len();
        let mut prevouts = Vec::with_capacity(input_count);

        // Peg-in inputs have no sidechain UTXO. Keep the vector aligned with
        // `tx.input` so a peg-in does not skip witness/P2SH on the rest of the
        // transaction. Elements still counts witness sigops against the claim
        // script; that walk does not use this dummy.
        #[cfg(feature = "liquid")]
        let empty_pegin_prevout = TxOut::default();

        // Coinbase has no prevouts. Peg-in is not coinbase.
        if !tx.is_coinbase() {
            for idx in 0..input_count {
                #[cfg(feature = "liquid")]
                if tx.input[idx].is_pegin {
                    prevouts.push(&empty_pegin_prevout);
                    continue;
                }
                prevouts.push(
                    *prevout_map
                        .get(&(idx as u32))
                        .ok_or(script::Error::EarlyEndOfScript)?,
                );
            }
        }

        get_sigop_cost(tx, &prevouts, true, true)
    }

    fn decode_pushnum(op: &Opcode) -> Option<u8> {
        // 81 = OP_1, 96 = OP_16
        // 81 -> 1, so... 81 - 80 -> 1
        #[cfg(not(feature = "liquid"))]
        let self_u8 = op.to_u8();
        #[cfg(feature = "liquid")]
        let self_u8 = op.into_u8();
        match self_u8 {
            81..=96 => Some(self_u8 - 80),
            _ => None,
        }
    }

    fn count_sigops(script: &script::Script, accurate: bool) -> usize {
        let mut n = 0;
        let mut pushnum_cache = None;
        for inst in script.instructions() {
            match inst {
                Ok(Instruction::Op(opcode)) => {
                    match opcode {
                        OP_CHECKSIG | OP_CHECKSIGVERIFY => {
                            n += 1;
                        }
                        OP_CHECKMULTISIG | OP_CHECKMULTISIGVERIFY => {
                            match (accurate, pushnum_cache) {
                                (true, Some(pushnum)) => {
                                    // Add the number of pubkeys in the multisig as sigop count
                                    n += usize::from(pushnum);
                                }
                                _ => {
                                    // MAX_PUBKEYS_PER_MULTISIG from Bitcoin Core
                                    // https://github.com/bitcoin/bitcoin/blob/v25.0/src/script/script.h#L29-L30
                                    n += 20;
                                }
                            }
                        }
                        _ => {
                            pushnum_cache = decode_pushnum(&opcode);
                        }
                    }
                }
                // We ignore errors as well as pushdatas
                _ => {
                    pushnum_cache = None;
                }
            }
        }

        n
    }

    /// Get the sigop count for legacy transactions
    fn get_legacy_sigop_count(tx: &Transaction) -> usize {
        let mut n = 0;
        for input in &tx.input {
            n += count_sigops(&input.script_sig, false);
        }
        for output in &tx.output {
            n += count_sigops(&output.script_pubkey, false);
        }
        n
    }

    fn get_p2sh_sigop_count(tx: &Transaction, previous_outputs: &[&TxOut]) -> usize {
        #[cfg(not(feature = "liquid"))]
        if tx.is_coinbase() {
            return 0;
        }
        #[cfg(feature = "liquid")]
        if tx.is_coinbase() {
            return 0;
        }
        let mut n = 0;
        for (input, prevout) in tx.input.iter().zip(previous_outputs.iter()) {
            // Elements: peg-in inputs are segwit-only. Do not count P2SH
            // against a claim script.
            #[cfg(feature = "liquid")]
            if input.is_pegin {
                continue;
            }
            if prevout.script_pubkey.is_p2sh()
                && let Some(Ok(script::Instruction::PushBytes(redeem))) =
                    input.script_sig.instructions().last()
            {
                #[cfg(not(feature = "liquid"))]
                let script = script::Script::from_bytes(redeem.as_bytes());
                #[cfg(feature = "liquid")]
                let script = script::Script::from(redeem.to_vec());
                #[allow(clippy::needless_borrow)]
                {
                    n += count_sigops(&script, true);
                }
            }
        }
        n
    }

    fn get_witness_sigop_count(tx: &Transaction, previous_outputs: &[&TxOut]) -> usize {
        let mut n = 0;

        #[inline]
        fn is_push_only(script: &script::Script) -> bool {
            for inst in script.instructions() {
                match inst {
                    Err(_) => return false,
                    Ok(Instruction::Op(_)) => return false,
                    Ok(Instruction::PushBytes(_)) => {}
                }
            }
            true
        }

        #[inline]
        fn last_pushdata(script: &script::Script) -> Option<&[u8]> {
            match script.instructions().last() {
                #[cfg(not(feature = "liquid"))]
                Some(Ok(Instruction::PushBytes(bytes))) => Some(bytes.as_bytes()),
                #[cfg(feature = "liquid")]
                Some(Ok(Instruction::PushBytes(bytes))) => Some(bytes),
                _ => None,
            }
        }

        #[inline]
        fn count_with_prevout(
            script_pubkey: &script::Script,
            script_sig: &script::Script,
            witness: &Witness,
        ) -> usize {
            let mut n = 0;

            let script_owned;
            let script: &script::Script = if script_pubkey.is_witness_program() {
                script_pubkey
            } else if script_pubkey.is_p2sh() && is_push_only(script_sig) && !script_sig.is_empty()
            {
                #[cfg(not(feature = "liquid"))]
                {
                    script_owned =
                        script::ScriptBuf::from(last_pushdata(script_sig).unwrap().to_vec());
                }
                #[cfg(feature = "liquid")]
                {
                    script_owned =
                        script::Script::from(last_pushdata(script_sig).unwrap().to_vec());
                }
                &script_owned
            } else {
                return 0;
            };

            #[cfg(not(feature = "liquid"))]
            if script.is_p2wsh() {
                let bytes = script.as_bytes();
                n += sig_ops(witness, bytes[0], &bytes[2..]);
            } else if script.is_p2wpkh() {
                n += 1;
            }
            #[cfg(feature = "liquid")]
            if script.is_v0_p2wsh() {
                let bytes = script.as_bytes();
                n += sig_ops(witness, bytes[0], &bytes[2..]);
            } else if script.is_v0_p2wpkh() {
                n += 1;
            }
            n
        }

        for (input, prevout) in tx.input.iter().zip(previous_outputs.iter()) {
            #[cfg(feature = "liquid")]
            if input.is_pegin {
                // Elements GetTransactionSigOpCost uses pegin_witness stack[3]
                // (claim_script) as scriptPubKey for CountWitnessSigOps when
                // the stack has at least four items. rust-elements 0.26 still
                // stores that claim at pegin_witness[3].
                if input.witness.pegin_witness.len() < 4 {
                    continue;
                }
                let claim_script = script::Script::from(input.witness.pegin_witness[3].clone());
                n += count_with_prevout(&claim_script, &input.script_sig, &input.witness);
                continue;
            }
            n += count_with_prevout(&prevout.script_pubkey, &input.script_sig, &input.witness);
        }
        n
    }

    /// Get the sigop cost for this transaction.
    fn get_sigop_cost(
        tx: &Transaction,
        previous_outputs: &[&TxOut],
        verify_p2sh: bool,
        verify_witness: bool,
    ) -> Result<usize, script::Error> {
        let mut n_sigop_cost = get_legacy_sigop_count(tx) * 4;
        if tx.is_coinbase() {
            return Ok(n_sigop_cost);
        }
        if tx.input.len() != previous_outputs.len() {
            return Err(script::Error::EarlyEndOfScript);
        }
        if verify_witness && !verify_p2sh {
            return Err(script::Error::EarlyEndOfScript);
        }
        if verify_p2sh {
            n_sigop_cost += get_p2sh_sigop_count(tx, previous_outputs) * 4;
        }
        if verify_witness {
            n_sigop_cost += get_witness_sigop_count(tx, previous_outputs);
        }

        Ok(n_sigop_cost)
    }

    /// Get sigops for the Witness
    ///
    /// witness_version is the raw opcode. OP_0 is 0, OP_1 is 81, etc.
    #[allow(clippy::redundant_closure)]
    fn sig_ops(witness: &Witness, witness_version: u8, witness_program: &[u8]) -> usize {
        #[cfg(feature = "liquid")]
        let last_witness = witness.script_witness.last();
        #[cfg(not(feature = "liquid"))]
        let last_witness = witness.last();
        match (witness_version, witness_program.len()) {
            (0, 20) => 1,
            (0, 32) => {
                #[cfg(not(feature = "liquid"))]
                {
                    #[allow(clippy::needless_borrow)]
                    last_witness
                        .map(|sl| script::Script::from_bytes(sl))
                        .map(|s| count_sigops(s, true))
                        .unwrap_or_default()
                }
                #[cfg(feature = "liquid")]
                {
                    last_witness
                        .map(|sl| script::Script::from(sl.clone()))
                        .map(|s| count_sigops(&s, true))
                        .unwrap_or_default()
                }
            }
            _ => 0,
        }
    }
}

/// Liquid REST `sigops` is BIP141 cost: legacy and P2SH counts times 4, plus
/// witness sigops. A peg-in input has no sidechain prevout, so it must not
/// make the whole transaction skip those extra counts. Elements
/// `GetTransactionSigOpCost` still walks sibling inputs and counts witness
/// sigops against the peg-in claim script (`pegin_witness[3]`).
#[cfg(all(test, feature = "liquid"))]
mod pegin_sigop_cost_tests {
    use super::sigops::transaction_sigop_count;
    use crate::chain::{
        OutPoint, Script, Transaction, TxIn, TxOut, Txid, Witness, hashes::Hash, opcodes,
    };
    use elements::{LockTime, Sequence};
    use std::collections::HashMap;

    fn p2wpkh_script_pubkey() -> Script {
        let mut bytes = vec![0x00, 0x14];
        bytes.extend_from_slice(&[0x11u8; 20]);
        Script::from(bytes)
    }

    fn p2wsh_script_pubkey() -> Script {
        let mut bytes = vec![0x00, 0x20];
        bytes.extend_from_slice(&[0x22u8; 32]);
        Script::from(bytes)
    }

    fn p2sh_script_pubkey() -> Script {
        let mut bytes = vec![opcodes::all::OP_HASH160.into_u8(), 0x14];
        bytes.extend_from_slice(&[0x33u8; 20]);
        bytes.push(opcodes::all::OP_EQUAL.into_u8());
        Script::from(bytes)
    }

    fn pegin_input(claim_script: Script, script_witness: Vec<Vec<u8>>) -> TxIn {
        TxIn {
            previous_output: OutPoint::new(Txid::from_byte_array([1u8; 32]), 0),
            is_pegin: true,
            script_sig: Script::new(),
            sequence: Sequence::MAX,
            asset_issuance: Default::default(),
            witness: Witness {
                amount_rangeproof: None,
                inflation_keys_rangeproof: None,
                script_witness,
                pegin_witness: vec![Vec::new(), Vec::new(), Vec::new(), claim_script.to_bytes()],
            },
        }
    }

    fn spend_input(vout: u32, script_sig: Script, script_witness: Vec<Vec<u8>>) -> TxIn {
        TxIn {
            previous_output: OutPoint::new(Txid::from_byte_array([2u8; 32]), vout),
            is_pegin: false,
            script_sig,
            sequence: Sequence::MAX,
            asset_issuance: Default::default(),
            witness: Witness {
                amount_rangeproof: None,
                inflation_keys_rangeproof: None,
                script_witness,
                pegin_witness: Vec::new(),
            },
        }
    }

    fn tx_from_inputs(input: Vec<TxIn>) -> Transaction {
        Transaction {
            version: 2,
            lock_time: LockTime::from_consensus(0),
            input,
            output: Vec::new(),
        }
    }

    fn prevout(script_pubkey: Script) -> TxOut {
        TxOut {
            script_pubkey,
            ..TxOut::default()
        }
    }

    /// Owed outcome: one peg-in input must not make `transaction_sigop_count`
    /// return only legacy cost for the whole transaction. A sibling P2WPKH
    /// spend still adds one BIP141 witness sigop.
    #[test]
    fn pegin_input_does_not_skip_witness_sigops_on_a_sibling_p2wpkh_spend() {
        let pegin = pegin_input(Script::new(), Vec::new());
        let spend = spend_input(1, Script::new(), vec![vec![0x00], vec![0x00; 33]]);
        let tx = tx_from_inputs(vec![pegin, spend]);
        let p2wpkh = prevout(p2wpkh_script_pubkey());
        let mut prevouts = HashMap::new();
        prevouts.insert(1, &p2wpkh);

        let cost = transaction_sigop_count(&tx, &prevouts).expect("sigop cost");
        assert_eq!(
            cost, 1,
            "a peg-in sibling must not drop the P2WPKH witness sigop"
        );
    }

    /// Owed outcome: Elements counts witness sigops for a peg-in against the
    /// claim script at `pegin_witness[3]`. A P2WPKH claim adds one.
    #[test]
    fn pegin_claim_script_p2wpkh_counts_one_witness_sigop_like_elements() {
        let tx = tx_from_inputs(vec![pegin_input(p2wpkh_script_pubkey(), Vec::new())]);
        let cost = transaction_sigop_count(&tx, &HashMap::new()).expect("sigop cost");
        assert_eq!(cost, 1, "peg-in P2WPKH claim script is one witness sigop");
    }

    /// Owed outcome: a peg-in input must not skip P2SH sigops on other inputs.
    /// The redeem is a single CHECKSIG, so P2SH adds 1 * 4. scriptSig only
    /// pushes that redeem, so legacy does not see CHECKSIG as an opcode.
    #[test]
    fn pegin_input_does_not_skip_p2sh_sigops_on_a_sibling_p2sh_spend() {
        let redeem = Script::from(vec![opcodes::all::OP_CHECKSIG.into_u8()]);
        let mut script_sig = vec![redeem.len() as u8];
        script_sig.extend_from_slice(redeem.as_bytes());
        let pegin = pegin_input(Script::new(), Vec::new());
        let spend = spend_input(1, Script::from(script_sig), Vec::new());
        let tx = tx_from_inputs(vec![pegin, spend]);
        let p2sh = prevout(p2sh_script_pubkey());
        let mut prevouts = HashMap::new();
        prevouts.insert(1, &p2sh);

        let cost = transaction_sigop_count(&tx, &prevouts).expect("sigop cost");
        assert_eq!(
            cost, 4,
            "a peg-in sibling must not drop P2SH sigops (P2SH cost 1 times 4)"
        );
    }

    /// Owed outcome: Elements counts a P2WSH peg-in claim from the last
    /// script-witness item. Accurate 1-of-1 CHECKMULTISIG is one sigop.
    #[test]
    fn pegin_p2wsh_claim_counts_accurate_multisig_in_script_witness() {
        let witness_script = {
            let mut bytes = vec![opcodes::all::OP_PUSHNUM_1.into_u8(), 33];
            bytes.extend_from_slice(&[0x02u8; 33]);
            bytes.push(opcodes::all::OP_PUSHNUM_1.into_u8());
            bytes.push(opcodes::all::OP_CHECKMULTISIG.into_u8());
            bytes
        };
        let tx = tx_from_inputs(vec![pegin_input(
            p2wsh_script_pubkey(),
            vec![vec![0x00], witness_script],
        )]);
        let cost = transaction_sigop_count(&tx, &HashMap::new()).expect("sigop cost");
        assert_eq!(cost, 1, "peg-in P2WSH claim uses accurate witness sigops");
    }

    /// Owed outcome: coinbase still returns legacy cost only. A CHECKSIG in
    /// the coinbase scriptSig is 1 * 4. Witness and P2SH walks do not run.
    #[test]
    fn coinbase_still_returns_legacy_sigop_cost_only() {
        let coinbase = TxIn {
            previous_output: OutPoint::null(),
            is_pegin: false,
            script_sig: Script::from(vec![opcodes::all::OP_CHECKSIG.into_u8()]),
            sequence: Sequence::MAX,
            asset_issuance: Default::default(),
            witness: Witness::empty(),
        };
        let tx = tx_from_inputs(vec![coinbase]);
        let cost = transaction_sigop_count(&tx, &HashMap::new()).expect("sigop cost");
        assert_eq!(cost, 4, "coinbase sigop cost stays legacy times four");
    }
}
