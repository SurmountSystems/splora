//! Address poisoning screen parts for one transaction.
//!
//! These parts are not fields of [`crate::rows::TxRow`].

use std::collections::{HashMap, HashSet};

use crate::wire::{Transaction, TxOutput};

/// Address text shows at most this many scalars of a matched prefix or postfix.
const DISPLAY_CAP: usize = 16;

const SIMILARITY_THRESHOLD: f64 = 1_000_000.0;

/// Left-hand prefix, infix, and postfix for one address.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AddressParts {
    pub prefix: String,
    pub infix: String,
    pub postfix: String,
}

/// One poisoning row.
///
/// `group` is the group stored with that row's best score.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PoisonAlert {
    pub address: String,
    pub script_type: String,
    pub parts: AddressParts,
    pub group: u32,
}

struct FuzzyMatch {
    score: f64,
    match_a: String,
    match_b: String,
}

struct Compared {
    score: f64,
    left_prefix: String,
    left_postfix: String,
    right_prefix: String,
    right_postfix: String,
}

struct StoredMatch {
    score: f64,
    parts: AddressParts,
    group: u32,
}

#[derive(Default)]
struct AddressSets<'a> {
    outputs: HashSet<&'a str>,
    inputs: HashSet<&'a str>,
}

/// Screen parts for `address` when it is similar to `other`.
///
/// `None` when the strings are identical, the script type cannot be compared,
/// a non-base58 type has no human-readable part on `network`, or the score is
/// not strictly above the threshold.
pub fn address_parts(
    address: &str,
    other: &str,
    script_type: &str,
    network: &str,
    unique_pairs: u64,
) -> Option<AddressParts> {
    let compared = compare_addresses(address, other, script_type, network)?;
    if compared.score > similarity_threshold(unique_pairs) {
        Some(screen_parts(
            address,
            &compared.left_prefix,
            &compared.left_postfix,
        ))
    } else {
        None
    }
}

/// Poisoning rows for `tx` on `network`, every matching input then every matching output.
pub fn poison_alerts(tx: &Transaction, network: &str) -> Vec<PoisonAlert> {
    let comparable_outputs = comparable_rows(tx.vout.iter().take(20));
    let comparable_inputs = comparable_rows(
        tx.vin
            .iter()
            .take(20)
            .filter_map(|input| input.prevout.as_ref()),
    );

    let mut type_count: HashMap<&str, AddressSets<'_>> = HashMap::new();
    for output in &comparable_outputs {
        let Some(address) = output.script_pubkey_address.as_deref() else {
            continue;
        };
        type_count
            .entry(output.script_pubkey_type.as_str())
            .or_default()
            .outputs
            .insert(address);
    }
    for input in &comparable_inputs {
        let Some(address) = input.script_pubkey_address.as_deref() else {
            continue;
        };
        type_count
            .entry(input.script_pubkey_type.as_str())
            .or_default()
            .inputs
            .insert(address);
    }

    let mut unique_pairs = 0u64;
    for sets in type_count.values() {
        let outputs = sets.outputs.len() as u64;
        let inputs = sets.inputs.len() as u64;
        unique_pairs += outputs * outputs.saturating_sub(1) / 2 + outputs * inputs;
    }
    let threshold = similarity_threshold(unique_pairs);

    let mut groups: HashMap<String, u32> = HashMap::new();
    let mut last_group = 0u32;
    let mut stored: HashMap<String, StoredMatch> = HashMap::new();

    for output in &comparable_outputs {
        let Some(address) = output.script_pubkey_address.as_deref() else {
            continue;
        };
        if stored.contains_key(address) {
            continue;
        }
        let script_type = output.script_pubkey_type.as_str();
        let mut others = Vec::new();
        for candidate in comparable_outputs.iter().chain(comparable_inputs.iter()) {
            if candidate.script_pubkey_type != script_type {
                continue;
            }
            let Some(other) = candidate.script_pubkey_address.as_deref() else {
                continue;
            };
            if other != address {
                others.push(other);
            }
        }
        for other in others {
            let Some(compared) = compare_addresses(address, other, script_type, network) else {
                continue;
            };
            // Equal to the threshold is not a match.
            if compared.score <= threshold {
                continue;
            }
            let group = assign_group(&mut groups, &mut last_group, address, other);
            store_if_better(
                &mut stored,
                address,
                compared.score,
                screen_parts(address, &compared.left_prefix, &compared.left_postfix),
                group,
            );
            store_if_better(
                &mut stored,
                other,
                compared.score,
                screen_parts(other, &compared.right_prefix, &compared.right_postfix),
                group,
            );
        }
    }

    let mut alerts = Vec::new();
    for input in &tx.vin {
        let Some(prevout) = input.prevout.as_ref() else {
            continue;
        };
        push_alert(&mut alerts, prevout, &stored);
    }
    for output in &tx.vout {
        push_alert(&mut alerts, output, &stored);
    }
    alerts
}

fn push_alert(
    alerts: &mut Vec<PoisonAlert>,
    output: &TxOutput,
    stored: &HashMap<String, StoredMatch>,
) {
    let Some(address) = output.script_pubkey_address.as_deref() else {
        return;
    };
    let Some(matched) = stored.get(address) else {
        return;
    };
    alerts.push(PoisonAlert {
        address: address.to_string(),
        script_type: output.script_pubkey_type.clone(),
        parts: matched.parts.clone(),
        group: matched.group,
    });
}

fn store_if_better(
    stored: &mut HashMap<String, StoredMatch>,
    address: &str,
    score: f64,
    parts: AddressParts,
    group: u32,
) {
    let replace = stored
        .get(address)
        .is_none_or(|existing| existing.score < score);
    if replace {
        stored.insert(
            address.to_string(),
            StoredMatch {
                score,
                parts,
                group,
            },
        );
    }
}

fn assign_group(
    groups: &mut HashMap<String, u32>,
    last_group: &mut u32,
    address: &str,
    other: &str,
) -> u32 {
    let group = match (groups.get(address).copied(), groups.get(other).copied()) {
        (Some(left), Some(right)) => {
            let group = left.min(right);
            if left != right {
                let higher = left.max(right);
                for existing in groups.values_mut() {
                    if *existing == higher {
                        *existing = group;
                    }
                }
            }
            group
        }
        (Some(left), None) => left,
        (None, Some(right)) => right,
        (None, None) => {
            let group = *last_group;
            *last_group += 1;
            group
        }
    };
    groups.insert(address.to_string(), group);
    groups.insert(other.to_string(), group);
    group
}

fn comparable_rows<'a>(rows: impl Iterator<Item = &'a TxOutput>) -> Vec<&'a TxOutput> {
    rows.filter(|output| {
        let Some(address) = output.script_pubkey_address.as_deref() else {
            return false;
        };
        walker_type(output) && !is_repeated_fragment(address)
    })
    .collect()
}

/// The similarity walk does not include wrapped segwit (`p2sh-p2wpkh`, `p2sh-p2wsh`).
fn walker_type(output: &TxOutput) -> bool {
    matches!(
        output.script_pubkey_type.as_str(),
        "p2pkh" | "p2sh" | "v0_p2wpkh" | "v0_p2wsh" | "v1_p2tr"
    )
}

/// `(.+?)\1{11,}`: one contiguous fragment repeated 12 times.
fn is_repeated_fragment(address: &str) -> bool {
    let chars: Vec<char> = address.chars().collect();
    let len = chars.len();
    if len < 12 {
        return false;
    }
    for start in 0..len {
        let max_unit = (len - start) / 12;
        for unit in 1..=max_unit {
            let piece = &chars[start..start + unit];
            let repeated = (1..12).all(|repeat| {
                let at = start + repeat * unit;
                &chars[at..at + unit] == piece
            });
            if repeated {
                return true;
            }
        }
    }
    false
}

fn similarity_threshold(unique_pairs: u64) -> f64 {
    if unique_pairs > 0 {
        SIMILARITY_THRESHOLD * (unique_pairs as f64)
    } else {
        SIMILARITY_THRESHOLD
    }
}

fn compare_addresses(
    address: &str,
    other: &str,
    script_type: &str,
    network: &str,
) -> Option<Compared> {
    if address == other || !comparable_script(script_type) {
        return None;
    }
    let left = fuzzy_prefix_match(address, other, false);
    let right = fuzzy_prefix_match(address, other, true);
    let score = normalized_score(script_type, network, left.score, right.score)?;
    Some(Compared {
        score,
        left_prefix: left.match_a,
        left_postfix: right.match_a,
        right_prefix: left.match_b,
        right_postfix: right.match_b,
    })
}

fn comparable_script(script_type: &str) -> bool {
    matches!(
        script_type,
        "p2pkh" | "p2sh" | "p2sh-p2wpkh" | "p2sh-p2wsh" | "v0_p2wpkh" | "v0_p2wsh" | "v1_p2tr"
    )
}

fn normalized_score(
    script_type: &str,
    network: &str,
    left_score: f64,
    right_score: f64,
) -> Option<f64> {
    let base58 = script_type == "p2pkh" || script_type == "p2sh";
    let prefix_score = if base58 {
        1.0
    } else {
        bech32_prefix_discount(network)?
    };
    let total_score = left_score + right_score - prefix_score;
    // Base58 uses a 58-character alphabet. Segwit and wrapped segwit use 32.
    let alphabet: f64 = if base58 { 58.0 } else { 32.0 };
    Some(alphabet.powf(total_score))
}

fn bech32_prefix_discount(network: &str) -> Option<f64> {
    let network = if network.is_empty() {
        "mainnet"
    } else {
        network
    };
    let hrp_len = match network {
        "mainnet" => 3,
        "testnet" | "testnet4" | "signet" => 3,
        "regtest" => 5,
        "liquid" => 3,
        "liquidtestnet" => 4,
        _ => return None,
    };
    // The guaranteed prefix is the human-readable part plus one character.
    Some((hrp_len + 1) as f64)
}

fn fuzzy_prefix_match(address: &str, other: &str, rtl: bool) -> FuzzyMatch {
    let mut address_chars: Vec<char> = address.chars().collect();
    let mut other_chars: Vec<char> = other.chars().collect();
    if rtl {
        address_chars.reverse();
        other_chars.reverse();
    }

    let mut score = 0.0;
    let mut gap = false;
    let mut done = false;
    let mut discounted = false;
    let mut address_index = 0;
    let mut other_index = 0;
    let mut match_a = Vec::new();
    let mut match_b = Vec::new();

    while address_index < address_chars.len() && other_index < other_chars.len() && !done {
        if address_chars[address_index] == other_chars[other_index] {
            match_a.push(address_chars[address_index]);
            match_b.push(other_chars[other_index]);
            score += if discounted { 0.5 } else { 1.0 };
            discounted = false;
            address_index += 1;
            other_index += 1;
        } else if !gap {
            let next_address = address_index + 1 < address_chars.len();
            let next_other = other_index + 1 < other_chars.len();
            let next_match_both = next_address
                && next_other
                && address_chars[address_index + 1] == other_chars[other_index + 1];
            let next_match_address =
                next_address && address_chars[address_index + 1] == other_chars[other_index];
            let next_match_other =
                next_other && address_chars[address_index] == other_chars[other_index + 1];
            if next_match_both {
                match_a.push(address_chars[address_index]);
                match_b.push(other_chars[other_index]);
                address_index += 1;
                other_index += 1;
            } else if next_match_address {
                match_a.push(address_chars[address_index]);
                address_index += 1;
            } else if next_match_other {
                match_b.push(other_chars[other_index]);
                other_index += 1;
            } else {
                address_index += 1;
                other_index += 1;
            }
            gap = true;
            discounted = true;
        } else {
            done = true;
        }
    }

    if rtl {
        match_a.reverse();
        match_b.reverse();
    }
    FuzzyMatch {
        score,
        match_a: match_a.into_iter().collect(),
        match_b: match_b.into_iter().collect(),
    }
}

fn screen_parts(address: &str, match_prefix: &str, match_postfix: &str) -> AddressParts {
    let address_chars: Vec<char> = address.chars().collect();
    let prefix_chars: Vec<char> = match_prefix.chars().collect();
    let postfix_chars: Vec<char> = match_postfix.chars().collect();
    let prefix: String = prefix_chars.iter().take(DISPLAY_CAP).collect();
    let postfix_start = postfix_chars.len().saturating_sub(DISPLAY_CAP);
    let postfix: String = postfix_chars[postfix_start..].iter().collect();
    let start = prefix_chars.len().min(DISPLAY_CAP);
    let cut = postfix_chars.len().min(DISPLAY_CAP);
    // A zero postfix cut is JavaScript `-0 || undefined`: the infix runs to the end.
    let end = if cut == 0 {
        address_chars.len()
    } else {
        address_chars.len().saturating_sub(cut)
    };
    let infix = if start >= end {
        String::new()
    } else {
        address_chars[start..end].iter().collect()
    };
    AddressParts {
        prefix,
        infix,
        postfix,
    }
}
