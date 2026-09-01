//! Fixture bytes are the historical bincode 1.3.3 little-endian fixint
//! layout with trailing padding allowed. The codec must round-trip them
//! without rewriting `new_index` keys.

use super::{deserialize_little, serialize_little};

#[test]
fn bincode_settings() {
    let value = TestStruct::new();
    let mut large = [0_u8; 4096];
    let decoded = [
        8_u8, 7, 6, 5, 4, 3, 2, 1, 1, 2, 3, 4, 5, 6, 7, 8, 0, 0, 0, 0, 8, 7, 6, 5, 4, 3, 2, 1, 1,
        2, 3, 4, 5, 6, 7, 8, 12, 0, 0, 0, 0, 0, 0, 0, 72, 101, 108, 108, 111, 32, 87, 111, 114,
        108, 100, 33,
    ];
    large[0..56].copy_from_slice(&decoded);

    assert_eq!(serialize_little(&value).unwrap(), &decoded);
    assert_eq!(deserialize_little::<TestStruct>(&large).unwrap(), value);
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
struct TestStruct {
    a: u64,
    b: [u8; 8],
    c: TestData,
    d: String,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
enum TestData {
    Foo(FooStruct),
    Bar(BarStruct),
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
struct FooStruct {
    a: u64,
    b: [u8; 8],
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
struct BarStruct {
    a: u64,
    b: [u8; 8],
}

impl TestStruct {
    fn new() -> Self {
        Self {
            a: 0x0102030405060708,
            b: [1, 2, 3, 4, 5, 6, 7, 8],
            c: TestData::Foo(FooStruct {
                a: 0x0102030405060708,
                b: [1, 2, 3, 4, 5, 6, 7, 8],
            }),
            d: String::from("Hello World!"),
        }
    }
}
