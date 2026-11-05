//! Shared explorer cases. Each test calls the shipped functions and compares
//! the return value to the fixture body. It does not paint a screen.

use splora_frontend_shared::wire::{Block, MempoolSummary, Transaction};
use splora_frontend_shared::{
    BLOCK_TX_PAGE_LEN, address_view_path, adjacent_blocks, block_details_open, block_open,
    block_open_from_height, block_pages_differ, block_path, block_tx_page, block_tx_page_path,
    block_txs_start_index_path, home_view, indexer_path_is_plain, liquid_rows_painted,
    load_first_mempool_block_after_skeleton_loads, next_block_tx_path, prefix_hits, prefix_lookup,
    recent_capped, recent_unconfirmed, recent_while_paused, recent_with_arrival, shared_cases,
    status_from_mempool, transaction_view,
};

const MEMPOOL: &str =
    r#"{"count":2,"vsize":300,"total_fee":1500,"fee_histogram":[[10.5,200],[1,100]]}"#;
const BLOCKS: &str = include_str!("fixtures/blocks.json");
const BLOCK: &str = include_str!("fixtures/block.json");
const BLOCK_TXS: &str = include_str!("fixtures/block_txs.json");
const ADDRESS_TXS: &str = include_str!("fixtures/address_txs.json");
const TX: &str = include_str!("fixtures/tx.json");
const RECENT: &str = include_str!("fixtures/mempool_recent.json");
const HEIGHT_BODY: &str = include_str!("fixtures/block_height.txt");
const GENESIS: &str = r#"{"id":"0000000000000000000000000000000000000000000000000000000000000000","height":0,"version":1,"timestamp":0,"tx_count":1,"size":285,"weight":1140,"merkle_root":"m","mediantime":0}"#;

const ADDR: &str = "bc1qexample";
const OTHER: &str = "bc1qother";

const ADDRESS_BODY: &str = r#"[{"txid":"multi-in","version":2,"locktime":0,"vin":[{"txid":"prev","vout":0,"prevout":{"scriptpubkey":"0014aa","scriptpubkey_asm":"OP_0","scriptpubkey_type":"v0_p2wpkh","scriptpubkey_address":"bc1qexample","value":1},"scriptsig":"","scriptsig_asm":"","is_coinbase":false,"sequence":1},{"txid":"prev","vout":1,"prevout":{"scriptpubkey":"0014aa","scriptpubkey_asm":"OP_0","scriptpubkey_type":"v0_p2wpkh","scriptpubkey_address":"bc1qexample","value":1},"scriptsig":"","scriptsig_asm":"","is_coinbase":false,"sequence":1}],"vout":[{"scriptpubkey":"0014bb","scriptpubkey_asm":"OP_0","scriptpubkey_type":"v0_p2wpkh","scriptpubkey_address":"bc1qother","value":1}],"size":180,"weight":720,"sigops":1,"fee":100,"status":{"confirmed":false}},{"txid":"multi-out","version":2,"locktime":0,"vin":[{"txid":"prev","vout":0,"prevout":{"scriptpubkey":"0014bb","scriptpubkey_asm":"OP_0","scriptpubkey_type":"v0_p2wpkh","scriptpubkey_address":"bc1qother","value":1},"scriptsig":"","scriptsig_asm":"","is_coinbase":false,"sequence":1}],"vout":[{"scriptpubkey":"0014aa","scriptpubkey_asm":"OP_0","scriptpubkey_type":"v0_p2wpkh","scriptpubkey_address":"bc1qexample","value":1},{"scriptpubkey":"0014aa","scriptpubkey_asm":"OP_0","scriptpubkey_type":"v0_p2wpkh","scriptpubkey_address":"bc1qexample","value":1}],"size":180,"weight":720,"sigops":1,"fee":100,"status":{"confirmed":true,"block_height":5,"block_hash":"h5","block_time":5}},{"txid":"both-sides","version":2,"locktime":0,"vin":[{"txid":"prev","vout":0,"prevout":{"scriptpubkey":"0014aa","scriptpubkey_asm":"OP_0","scriptpubkey_type":"v0_p2wpkh","scriptpubkey_address":"bc1qexample","value":1},"scriptsig":"","scriptsig_asm":"","is_coinbase":false,"sequence":1}],"vout":[{"scriptpubkey":"0014aa","scriptpubkey_asm":"OP_0","scriptpubkey_type":"v0_p2wpkh","scriptpubkey_address":"bc1qexample","value":1}],"size":180,"weight":720,"sigops":1,"fee":100,"status":{"confirmed":true,"block_height":4,"block_hash":"h4","block_time":4}},{"txid":"single-in","version":2,"locktime":0,"vin":[{"txid":"prev","vout":0,"prevout":{"scriptpubkey":"0014aa","scriptpubkey_asm":"OP_0","scriptpubkey_type":"v0_p2wpkh","scriptpubkey_address":"bc1qexample","value":1},"scriptsig":"","scriptsig_asm":"","is_coinbase":false,"sequence":1}],"vout":[{"scriptpubkey":"0014bb","scriptpubkey_asm":"OP_0","scriptpubkey_type":"v0_p2wpkh","scriptpubkey_address":"bc1qother","value":1}],"size":180,"weight":720,"sigops":1,"fee":100,"status":{"confirmed":true,"block_height":3,"block_hash":"h3","block_time":3}},{"txid":"single-out","version":2,"locktime":0,"vin":[{"txid":"prev","vout":0,"prevout":{"scriptpubkey":"0014bb","scriptpubkey_asm":"OP_0","scriptpubkey_type":"v0_p2wpkh","scriptpubkey_address":"bc1qother","value":1},"scriptsig":"","scriptsig_asm":"","is_coinbase":false,"sequence":1}],"vout":[{"scriptpubkey":"0014aa","scriptpubkey_asm":"OP_0","scriptpubkey_type":"v0_p2wpkh","scriptpubkey_address":"bc1qexample","value":1}],"size":180,"weight":720,"sigops":1,"fee":100,"status":{"confirmed":true,"block_height":2,"block_hash":"h2","block_time":2}}]"#;

fn case_names() -> Vec<&'static str> {
    shared_cases().iter().map(|case| case.name).collect()
}

fn assert_plain(path: &str) {
    assert!(indexer_path_is_plain(path), "{path}");
}

#[test]
fn shared_case_list_is_the_served_observables() {
    let names = case_names();
    assert_eq!(names.len(), 29);
    assert!(names.contains(&"home_projected_mempool_block"));
    assert!(names.contains(&"search_bech32_case_insensitive"));
    assert!(names.contains(&"search_bech32m_case_insensitive"));
    assert!(names.contains(&"block_height_zero"));
    assert!(names.contains(&"recent_pause"));
    assert!(names.contains(&"status_mempool_vsize"));
    assert!(!names.iter().any(|name| name.contains("liquid")));
    assert!(!liquid_rows_painted());
}

#[test]
fn home_projects_a_mempool_block_and_opens_confirmed_blocks() {
    let parsed: Vec<Block> = serde_json::from_str(BLOCKS).unwrap();
    let home = home_view(MEMPOOL, BLOCKS).unwrap();
    assert!(home.projected_navigable);
    assert_eq!(
        home.projected_open_path,
        load_first_mempool_block_after_skeleton_loads().route
    );
    assert!(!home.projected_link_id.is_empty());
    assert_plain(&home.projected_open_path);
    assert_eq!(home.confirmed.len(), parsed.len());
    assert_eq!(home.confirmed[0].id, parsed[0].id);
    assert_eq!(home.confirmed[0].height, parsed[0].height);
    assert_eq!(home.confirmed[0].open_path, block_path(&parsed[0].id));
    assert_plain(&home.confirmed[0].open_path);
}

#[test]
fn search_resolves_a_legacy_prefix_and_narrows() {
    let body = r#"["1wizSAYSbuyXbt9d8JV8ytm5acqq2TorC","1wizOTHER000000000000000000000000","3other00000000000000000000000000"]"#;
    let typed = prefix_lookup("1wiz").unwrap();
    assert!(!typed.folded);
    assert_eq!(typed.path, "/address-prefix/1wiz");
    assert_plain(&typed.path);
    let short = prefix_hits("1wiz", body).unwrap();
    let longer = prefix_hits("1wizS", body).unwrap();
    assert!(longer.len() < short.len());
    assert!(longer.iter().all(|hit| short.iter().any(|row| row == hit)));
    assert_eq!(
        address_view_path(&short[0]),
        format!("/address/{}", short[0])
    );
    assert_plain(&address_view_path(&short[0]));
    let kept = prefix_lookup("1Wiz").unwrap();
    assert_eq!(kept.path, "/address-prefix/1Wiz");
    assert_ne!(kept.path, typed.path);
}

#[test]
fn search_folds_partial_bech32_and_bech32m() {
    let bech32 = r#"["bc1q0003match000000000000000000000000"]"#;
    let bech32m = r#"["bc1pqyqsmatchone000000000000000000"]"#;
    let upper_q = prefix_lookup("BC1Q0003").unwrap();
    let mixed_q = prefix_lookup("bC1q0003").unwrap();
    assert!(upper_q.folded);
    assert_eq!(upper_q.path, mixed_q.path);
    assert_eq!(upper_q.path, "/address-prefix/bc1q0003");
    assert_plain(&upper_q.path);
    assert_eq!(
        prefix_hits("BC1Q0003", bech32).unwrap(),
        prefix_hits("bC1q0003", bech32).unwrap()
    );
    let upper_p = prefix_lookup("BC1PQYQS").unwrap();
    let mixed_p = prefix_lookup("bc1PqYqS").unwrap();
    assert_eq!(upper_p.path, mixed_p.path);
    assert_eq!(upper_p.path, "/address-prefix/bc1pqyqs");
    assert_eq!(
        prefix_hits("BC1PQYQS", bech32m).unwrap(),
        prefix_hits("bc1PqYqS", bech32m).unwrap()
    );
    assert!(prefix_lookup("bc1 q").is_err());
    assert!(prefix_lookup("bc1?q").is_err());
    assert!(prefix_lookup("bc1#q").is_err());
    assert!(prefix_lookup("bc1%q").is_err());
}

#[test]
fn address_list_keeps_served_order_and_marks_the_address() {
    let parsed: Vec<Transaction> = serde_json::from_str(ADDRESS_BODY).unwrap();
    let list = splora_frontend_shared::address_tx_list(ADDR, ADDRESS_BODY).unwrap();
    let ids: Vec<String> = parsed.iter().map(|tx| tx.txid.clone()).collect();
    assert_eq!(list.txids, ids);
    assert_eq!(list.txids[0], "multi-in");
    assert_eq!(list.marks[0].inputs, 2);
    assert_eq!(list.marks[0].outputs, 0);
    assert_eq!(list.marks[1].inputs, 0);
    assert_eq!(list.marks[1].outputs, 2);
    assert_eq!(list.marks[2].inputs, 1);
    assert_eq!(list.marks[2].outputs, 1);
    assert_eq!(list.marks[3].inputs, 1);
    assert_eq!(list.marks[3].outputs, 0);
    assert_eq!(list.marks[4].inputs, 0);
    assert_eq!(list.marks[4].outputs, 1);
    assert_ne!(ADDR, OTHER);
}

#[test]
fn transaction_view_reads_inputs_outputs_and_confirmation() {
    let confirmed = transaction_view(TX).unwrap();
    let parsed: Transaction = serde_json::from_str(TX).unwrap();
    assert_eq!(confirmed.txid, parsed.txid);
    assert_eq!(confirmed.input_count, parsed.vin.len());
    assert_eq!(confirmed.output_count, parsed.vout.len());
    assert_eq!(
        confirmed.confirmed,
        parsed
            .status
            .as_ref()
            .map(|status| status.confirmed)
            .unwrap_or(false)
    );
    assert_eq!(confirmed.block_hash, parsed.status.unwrap().block_hash);

    let unconfirmed_rows: Vec<Transaction> = serde_json::from_str(ADDRESS_TXS).unwrap();
    let unconfirmed = transaction_view(ADDRESS_TXS).expect_err("one object, not an array");
    let _ = unconfirmed;
    let one = serde_json::to_string(&unconfirmed_rows[0]).unwrap();
    let view = transaction_view(&one).unwrap();
    assert!(!view.confirmed);
    assert!(view.block_hash.is_none());
    assert_eq!(view.input_count, unconfirmed_rows[0].vin.len());
    assert_eq!(view.output_count, unconfirmed_rows[0].vout.len());

    let block_rows: Vec<Transaction> = serde_json::from_str(BLOCK_TXS).unwrap();
    let from_block = transaction_view(&serde_json::to_string(&block_rows[0]).unwrap()).unwrap();
    assert!(from_block.confirmed);
    assert_eq!(from_block.txid, block_rows[0].txid);
    assert_eq!(from_block.block_hash.as_deref(), Some("block-id-1"));
}

#[test]
fn block_opens_by_hash_or_height_including_zero_and_pages() {
    let parsed: Block = serde_json::from_str(BLOCK).unwrap();
    let opened = block_open(BLOCK).unwrap();
    assert_eq!(opened.hash, parsed.id);
    assert_eq!(opened.height, parsed.height);
    assert_eq!(opened.tx_count, parsed.tx_count);
    assert_eq!(opened.open_path, block_path(&parsed.id));
    assert_plain(&opened.open_path);
    let from_height = block_open_from_height(parsed.height, &parsed.id, BLOCK).unwrap();
    assert_eq!(from_height.hash, parsed.id);
    assert!(block_open_from_height(0, &parsed.id, BLOCK).is_err());
    assert!(block_open_from_height(parsed.height, HEIGHT_BODY, BLOCK).is_err());

    let genesis: Block = serde_json::from_str(GENESIS).unwrap();
    let zero = block_open_from_height(0, &genesis.id, GENESIS).unwrap();
    assert_eq!(zero.height, 0);
    assert_eq!(zero.tx_count, 1);
    assert_eq!(
        zero.heading,
        splora_frontend_shared::transaction_count_heading(genesis.tx_count)
    );
    let coinbase = format!(
        r#"[{{"txid":"coinbase","version":1,"locktime":0,"vin":[],"vout":[],"size":285,"weight":1140,"sigops":0,"fee":0,"status":{{"confirmed":true,"block_height":0,"block_hash":"{}","block_time":0}}}}]"#,
        genesis.id
    );
    let page = block_tx_page(&coinbase).unwrap();
    assert_eq!(page.txids.len(), 1);
    assert_eq!(page.confirmed, vec![true]);
    assert_eq!(zero.tx_count as usize, page.txids.len());

    let next = BLOCK_TXS.replace("block-tx-1", "block-tx-2");
    assert!(block_pages_differ(BLOCK_TXS, &next).unwrap());
    assert!(!block_pages_differ(BLOCK_TXS, BLOCK_TXS).unwrap());
    let first_path = block_tx_page_path(&parsed.id, 0);
    let next_path = next_block_tx_path(&parsed.id);
    assert_eq!(
        next_path,
        block_txs_start_index_path(&parsed.id, BLOCK_TX_PAGE_LEN)
    );
    assert_ne!(first_path, next_path);
    assert_plain(&first_path);
    assert_plain(&next_path);
    assert_eq!(BLOCK_TX_PAGE_LEN, 25);
}

#[test]
fn block_edges_and_details_toggle() {
    let genesis = adjacent_blocks(0, 100);
    assert!(!genesis.previous_shown);
    assert!(genesis.previous_height.is_none());
    assert!(genesis.next_shown);
    assert_eq!(genesis.next_height, Some(1));
    let tip = adjacent_blocks(100, 100);
    assert!(tip.previous_shown);
    assert_eq!(tip.previous_height, Some(99));
    assert!(!tip.next_shown);
    assert!(tip.next_height.is_none());
    let middle = adjacent_blocks(50, 100);
    assert!(middle.previous_shown);
    assert!(middle.next_shown);
    assert_eq!(middle.previous_height, Some(49));
    assert_eq!(middle.next_height, Some(51));
    assert!(block_details_open(false));
    assert!(!block_details_open(true));
    assert_eq!(block_details_open(block_details_open(false)), false);
}

#[test]
fn recent_list_shows_arrival_pause_and_cap() {
    let ids = recent_unconfirmed(RECENT).unwrap();
    let parsed: Vec<splora_frontend_shared::wire::RecentTransaction> =
        serde_json::from_str(RECENT).unwrap();
    assert_eq!(
        ids,
        parsed
            .iter()
            .map(|row| row.txid.clone())
            .collect::<Vec<_>>()
    );
    let arrived = "just-arrived";
    let with_new = recent_with_arrival(&ids, arrived);
    assert_eq!(with_new[0], arrived);
    assert_eq!(&with_new[1..], ids.as_slice());
    let paused = recent_while_paused(&ids, &[arrived.to_string()]);
    assert_eq!(paused, ids);
    let repeated: Vec<String> = ids.iter().cycle().take(12).cloned().collect();
    let capped = recent_capped(&repeated, 10);
    assert_eq!(capped.len(), 10);
    assert!(capped.iter().all(|id| id == &ids[0]));
}

#[test]
fn status_reports_count_and_vsize_only() {
    let parsed: MempoolSummary = serde_json::from_str(MEMPOOL).unwrap();
    let status = status_from_mempool(MEMPOOL).unwrap();
    assert_eq!(status.unconfirmed_count, parsed.count);
    assert_eq!(status.vsize, parsed.vsize);
    assert_eq!(status.total_fee_sats, parsed.total_fee);
    assert!(case_names().contains(&"status_unconfirmed_count"));
    assert!(!case_names().iter().any(|name| name.contains("median")));
    assert!(!case_names().iter().any(|name| name.contains("ntx")));
}
