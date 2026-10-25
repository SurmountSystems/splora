//! Whole line `bits 386547056 0x170a3d70` when open block details carry bits 386547056.
//!
//! The Bits table cell is a different check. This file asserts the whole line only.

use splora_web::block_details_markup;

const ID: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";

fn block_json() -> String {
    format!(
        r#"{{"id":"{ID}","height":2,"version":1,"timestamp":1700000000,"tx_count":1,"size":100,"weight":400,"merkle_root":"merkle-root","previousblockhash":null,"mediantime":1699990000,"nonce":9,"bits":386547056,"difficulty":1}}"#
    )
}

#[test]
fn open_block_details_paint_bits_386547056_0x170a3d70() {
    let open = block_details_markup(&block_json(), true);
    assert!(
        open.lines().any(|line| line == "bits 386547056 0x170a3d70"),
        "bits 386547056 0x170a3d70 is absent: {open}"
    );
}
