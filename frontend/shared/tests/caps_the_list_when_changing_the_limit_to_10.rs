//! Cypress it() title: caps the list when changing the limit to 10.
//!
//! Nine mocked websocket batches of six transactions fill the recent
//! list. The page limit starts at 50, so 50 rows are visible. Clicking
//! `[data-cy="limit-10"]` leaves 10 rows. Those counts are
//! `live.slice(0, limit)` on the newest-first list. This it() does not
//! read a GET `/mempool/recent` body.

use splora_frontend_shared::caps_the_list_when_changing_the_limit_to_10 as cap_list;

fn mock_txid(batch: u8, index: u8) -> String {
    let mut id = format!("{batch:02x}{index:02x}");
    id.push_str(&"00".repeat(30));
    id
}

#[test]
fn caps_the_list_when_changing_the_limit_to_10() {
    let owned: [[String; 6]; 9] = std::array::from_fn(|batch| {
        std::array::from_fn(|index| mock_txid(batch as u8, index as u8))
    });
    let batches: [[&str; 6]; 9] =
        std::array::from_fn(|batch| std::array::from_fn(|index| owned[batch][index].as_str()));

    // The component inserts each batch from the end, then reverses the
    // map, so later batches come first and index 0 inside a batch is newer.
    let mut newest_first: Vec<&str> = Vec::with_capacity(54);
    for batch in (0..9).rev() {
        for index in 0..6 {
            newest_first.push(batches[batch][index]);
        }
    }

    let (rows_at_limit_50, rows_at_limit_10) = cap_list(&batches);

    assert_eq!(
        rows_at_limit_50,
        &newest_first[..50],
        "nine batches of six show 50 newest-first rows at the default limit"
    );
    assert_eq!(
        rows_at_limit_10,
        &newest_first[..10],
        "clicking limit-10 leaves the first 10 of those newest-first rows"
    );
    assert_eq!(rows_at_limit_50.len(), 50);
    assert_eq!(rows_at_limit_10.len(), 10);
}
