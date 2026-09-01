//! On-disk Mempool schema codec.
//!
//! Layout matches historical bincode 1.3.3: fixed-width integers, u64 length
//! prefixes, u32 enum tags, unlimited size, trailing bytes allowed.
//! [wincode](https://crates.io/crates/wincode) / [serde-wincode](https://crates.io/crates/serde-wincode)
//! (accessed: 2026-08-31) replace unmaintained bincode
//! ([RUSTSEC-2025-0141](https://rustsec.org/advisories/RUSTSEC-2025-0141),
//! accessed: 2026-08-31). Do not rewrite `new_index` keys.
//!
//! +--------------+--------+------------+----------------+------------+
//! |              | Endian | Int Length | Allow Trailing | Byte Limit |
//! +--------------+--------+------------+----------------+------------+
//! | TxHistoryRow | big    | fixed      | allow          | unlimited  |
//! | All others   | little | fixed      | allow          | unlimited  |
//! +--------------+--------+------------+----------------+------------+

use serde_wincode::SerdeCompat;
use wincode::config::{Configuration, Deserialize, Serialize};

pub type Error = Box<dyn std::error::Error + Send + Sync + 'static>;

fn little_cfg() -> impl wincode::config::Config + Copy {
    Configuration::default()
        .disable_preallocation_size_limit()
        .with_little_endian()
        .with_fixint_encoding()
}

fn big_cfg() -> impl wincode::config::Config + Copy {
    Configuration::default()
        .disable_preallocation_size_limit()
        .with_big_endian()
        .with_fixint_encoding()
}

pub fn serialize_big<T>(value: &T) -> Result<Vec<u8>, Error>
where
    T: ?Sized + serde::Serialize,
{
    // SerdeCompat requires Sized; `&T` is always Sized and serde-delegates.
    <SerdeCompat<&T> as Serialize<_>>::serialize(&value, big_cfg())
        .map_err(|e| e.to_string().into())
}

pub fn deserialize_big<'a, T>(bytes: &'a [u8]) -> Result<T, Error>
where
    T: serde::Deserialize<'a>,
{
    <SerdeCompat<T> as Deserialize<'_, _>>::deserialize(bytes, big_cfg())
        .map_err(|e| e.to_string().into())
}

pub fn serialize_little<T>(value: &T) -> Result<Vec<u8>, Error>
where
    T: ?Sized + serde::Serialize,
{
    <SerdeCompat<&T> as Serialize<_>>::serialize(&value, little_cfg())
        .map_err(|e| e.to_string().into())
}

pub fn deserialize_little<'a, T>(bytes: &'a [u8]) -> Result<T, Error>
where
    T: serde::Deserialize<'a>,
{
    <SerdeCompat<T> as Deserialize<'_, _>>::deserialize(bytes, little_cfg())
        .map_err(|e| e.to_string().into())
}

#[cfg(test)]
#[path = "./bincode_tests.rs"]
mod bincode_tests;
