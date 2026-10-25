//! Whole line `Merkle root merkle-root` when the merkle root value is merkle-root.
//!
//! Open details call the shipped `block_details_markup`. A line match is the
//! whole line, not a substring inside a longer line. The Merkle root cell stays.

use splora_web::block_details_markup;

const LINE: &str = "Merkle root merkle-root";

fn block_json(id: &str, merkle_root: &str) -> String {
    format!(
        r#"{{"id":"{id}","height":2,"version":1,"timestamp":1700000000,"tx_count":1,"size":100,"weight":400,"merkle_root":"{merkle_root}","previousblockhash":null,"mediantime":1699990000,"nonce":9,"bits":2,"difficulty":1}}"#
    )
}

fn assert_one_line(markup: &str, line: &str) {
    assert!(
        markup.lines().any(|row| row == line),
        "{line} is absent: {markup}"
    );
    assert_eq!(
        markup.lines().filter(|row| *row == line).count(),
        1,
        "{line} was not one line: {markup}"
    );
}

/// Open details paint `Merkle root merkle-root` and keep the Merkle root cell.
///
/// Closed details keep the cell and omit the whole line.
#[test]
fn open_block_details_paint_merkle_root_merkle_root() {
    let id = "merklemerkle";
    let body = block_json(id, "merkle-root");
    let open = block_details_markup(&body, true);
    assert!(
        open.lines().any(|line| line == "Merkle root merkle-root"),
        "Merkle root merkle-root is absent: {open}"
    );
    assert_one_line(&open, LINE);
    assert_eq!(
        open.matches("<tr><td>Merkle root</td><td>merkle-root</td></tr>")
            .count(),
        1,
        "Merkle root cell was dropped: {open}"
    );

    let closed = block_details_markup(&body, false);
    assert!(
        !closed.lines().any(|line| line == LINE),
        "closed details painted {LINE}: {closed}"
    );
    assert!(
        closed.contains("<tr><td>Merkle root</td><td>merkle-root</td></tr>"),
        "closed details dropped the Merkle root cell: {closed}"
    );
}
