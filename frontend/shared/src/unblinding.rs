//! A regular Bitcoin transaction has no confidential output to unblind.

const UNBLINDING_ERROR: &str = "unblinding error";

/// `None` when `tx_hex` is one complete regular Bitcoin transaction.
///
/// `Some` when the hex is invalid or those bytes are not that transaction.
/// The text contains `unblinding error`.
pub fn unblinding_error_for_regular_tx(tx_hex: &str) -> Option<String> {
    let Some(bytes) = decode_hex(tx_hex) else {
        return Some(UNBLINDING_ERROR.to_string());
    };
    if parse_bitcoin_transaction(&bytes) {
        None
    } else {
        Some(UNBLINDING_ERROR.to_string())
    }
}

fn decode_hex(tx_hex: &str) -> Option<Vec<u8>> {
    let bytes = tx_hex.as_bytes();
    if bytes.len() % 2 != 0 {
        return None;
    }
    let mut out = Vec::with_capacity(bytes.len() / 2);
    let mut index = 0;
    while index < bytes.len() {
        let high = hex_nibble(bytes[index])?;
        let low = hex_nibble(bytes[index + 1])?;
        out.push((high << 4) | low);
        index += 2;
    }
    Some(out)
}

fn hex_nibble(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'a'..=b'f' => Some(byte - b'a' + 10),
        b'A'..=b'F' => Some(byte - b'A' + 10),
        _ => None,
    }
}

/// BIP144: a `0x00` marker and a nonzero flag after the version introduce the witness.
fn parse_bitcoin_transaction(data: &[u8]) -> bool {
    let mut index = 0;
    if skip(data, &mut index, 4).is_none() {
        return false;
    }
    let segwit = take_segwit_marker(data, &mut index);
    let Some(input_count) = read_compact_size(data, &mut index) else {
        return false;
    };
    let Some(inputs) = fits(input_count, 41, data.len().saturating_sub(index)) else {
        return false;
    };
    for _ in 0..inputs {
        if !read_input(data, &mut index) {
            return false;
        }
    }
    let Some(output_count) = read_compact_size(data, &mut index) else {
        return false;
    };
    let Some(outputs) = fits(output_count, 9, data.len().saturating_sub(index)) else {
        return false;
    };
    for _ in 0..outputs {
        if !read_output(data, &mut index) {
            return false;
        }
    }
    if segwit {
        for _ in 0..inputs {
            if !read_witness(data, &mut index) {
                return false;
            }
        }
    }
    if skip(data, &mut index, 4).is_none() {
        return false;
    }
    index == data.len()
}

fn take_segwit_marker(data: &[u8], index: &mut usize) -> bool {
    if *index + 1 < data.len() && data[*index] == 0x00 && data[*index + 1] != 0x00 {
        *index += 2;
        true
    } else {
        false
    }
}

fn read_input(data: &[u8], index: &mut usize) -> bool {
    // Previous outpoint: 32-byte txid and 4-byte index.
    if skip(data, index, 36).is_none() {
        return false;
    }
    let Some(script_len) = read_compact_size(data, index) else {
        return false;
    };
    if skip(data, index, script_len).is_none() {
        return false;
    }
    skip(data, index, 4).is_some()
}

fn read_output(data: &[u8], index: &mut usize) -> bool {
    if skip(data, index, 8).is_none() {
        return false;
    }
    let Some(script_len) = read_compact_size(data, index) else {
        return false;
    };
    skip(data, index, script_len).is_some()
}

fn read_witness(data: &[u8], index: &mut usize) -> bool {
    let Some(item_count) = read_compact_size(data, index) else {
        return false;
    };
    let Some(items) = fits(item_count, 1, data.len().saturating_sub(*index)) else {
        return false;
    };
    for _ in 0..items {
        let Some(item_len) = read_compact_size(data, index) else {
            return false;
        };
        if skip(data, index, item_len).is_none() {
            return false;
        }
    }
    true
}

fn read_compact_size(data: &[u8], index: &mut usize) -> Option<u64> {
    if *index >= data.len() {
        return None;
    }
    let first = data[*index];
    *index += 1;
    match first {
        size @ 0x00..=0xfc => Some(u64::from(size)),
        0xfd => {
            if *index + 2 > data.len() {
                return None;
            }
            let size = u64::from(u16::from_le_bytes([data[*index], data[*index + 1]]));
            *index += 2;
            if size < 0xfd { None } else { Some(size) }
        }
        0xfe => {
            if *index + 4 > data.len() {
                return None;
            }
            let size = u64::from(u32::from_le_bytes([
                data[*index],
                data[*index + 1],
                data[*index + 2],
                data[*index + 3],
            ]));
            *index += 4;
            if size < 0x1_0000 { None } else { Some(size) }
        }
        0xff => {
            if *index + 8 > data.len() {
                return None;
            }
            let size = u64::from_le_bytes([
                data[*index],
                data[*index + 1],
                data[*index + 2],
                data[*index + 3],
                data[*index + 4],
                data[*index + 5],
                data[*index + 6],
                data[*index + 7],
            ]);
            *index += 8;
            if size < 0x1_0000_0000 {
                None
            } else {
                Some(size)
            }
        }
    }
}

fn skip(data: &[u8], index: &mut usize, len: u64) -> Option<()> {
    let len = usize::try_from(len).ok()?;
    let end = (*index).checked_add(len)?;
    if end > data.len() {
        return None;
    }
    *index = end;
    Some(())
}

fn fits(count: u64, min_each: usize, remaining: usize) -> Option<usize> {
    let count = usize::try_from(count).ok()?;
    if min_each == 0 || remaining / min_each < count {
        None
    } else {
        Some(count)
    }
}
