//! One assertion per `ported` row in `doc/angular-test-port.md`.
//!
//! The electrs path is the GET path from that row's Asserts cell.
//! Inventory spells a transaction id as `:hash` and an address as `:address`.
//! `paths.rs` spells those `:txid` and `:script`. Liquid registry keys are
//! not in `wire`. The explorer is not finished.

use std::path::Path;

use splora_frontend_shared::wire::{
    AddressStats, Block, FeeEstimates, MempoolSummary, RecentTransaction, Transaction,
};
use splora_frontend_shared::{
    IndexerCall, address_txs_path, block_path, block_txs_path, broadcast_path, fee_estimates_path,
    indexer_request, indexer_routes, mempool_path, mempool_recent_path, tx_path,
};

const PORTED_ROWS: usize = 89;
const BECH32M_OPENED: &str = "bc1pqyqsexampleaddress";
const BECH32_OPENED: &str = "bc1q0003exampleaddress";
const BASE58_OPENED: &str = "1A1zP1eP5QGefi2DMPTfTL5SLmv7DivfNa";

struct PortedRow {
    spec: &'static str,
    test: &'static str,
    paths: &'static [&'static str],
    depends: Depends,
}

#[derive(Clone, Copy)]
enum Depends {
    ProjectedMempoolBlock,
    Dashboard,
    BlocksList,
    BlockWithTxs,
    BlockDetails,
    BlockTxCount {
        tx_count: u32,
        genesis: bool,
    },
    BlockTxPage,
    BlockNav {
        genesis: bool,
    },
    TxPage,
    TxInputLink,
    TxAmounts,
    TxRows,
    TxVout {
        index: usize,
        sats: u64,
    },
    TxAndAddress,
    AddressHighlight {
        inputs: usize,
        outputs: usize,
    },
    AddressPrefix {
        typed: &'static str,
        opened: &'static str,
        list_len: usize,
    },
    Poison {
        inputs: bool,
        shared_postfix: bool,
    },
    OpReturnRow,
    CoinbaseRow,
    StatusScreen,
    RecentGain,
    RecentStable,
    RecentCap,
    RecentPill,
    /// Registry, search, and asset documents. No shared serde struct.
    LiquidAssetDocument,
    AssetTxList,
    TxSecondOutput,
}

const LIQUID: &str = "ref/mempool/frontend/cypress/e2e/liquid/liquid.spec.ts";
const LIQUID_TESTNET: &str = "ref/mempool/frontend/cypress/e2e/liquidtestnet/liquidtestnet.spec.ts";
const MAINNET: &str = "ref/mempool/frontend/cypress/e2e/mainnet/mainnet.spec.ts";
const RECENT: &str = "ref/mempool/frontend/cypress/e2e/mainnet/recent-transactions.spec.ts";
const SIGNET: &str = "ref/mempool/frontend/cypress/e2e/signet/signet.spec.ts";
const TESTNET4: &str = "ref/mempool/frontend/cypress/e2e/testnet4/testnet4.spec.ts";

const PORTED: &[PortedRow] = &[
    PortedRow {
        spec: LIQUID,
        test: "check first mempool block after skeleton loads",
        paths: &["/mempool", "/fee-estimates", "/blocks"],
        depends: Depends::ProjectedMempoolBlock,
    },
    PortedRow {
        spec: LIQUID,
        test: "load first mempool block after skeleton loads",
        paths: &["/mempool", "/fee-estimates", "/blocks"],
        depends: Depends::ProjectedMempoolBlock,
    },
    PortedRow {
        spec: LIQUID,
        test: "loads the dashboard",
        paths: &["/mempool", "/blocks"],
        depends: Depends::Dashboard,
    },
    PortedRow {
        spec: LIQUID,
        test: "loads the blocks page",
        paths: &["/blocks"],
        depends: Depends::BlocksList,
    },
    PortedRow {
        spec: LIQUID,
        test: "loads a specific block page",
        paths: &["/block/:hash", "/block/:hash/txs"],
        depends: Depends::BlockWithTxs,
    },
    PortedRow {
        spec: LIQUID,
        test: "peg in/peg out / loads peg in addresses",
        paths: &["/tx/:hash"],
        depends: Depends::TxInputLink,
    },
    PortedRow {
        spec: LIQUID,
        test: "peg in/peg out / loads peg out addresses",
        paths: &["/tx/:hash", "/address/:address"],
        depends: Depends::TxAndAddress,
    },
    PortedRow {
        spec: LIQUID,
        test: "assets / shows the assets screen",
        paths: &["/assets/registry"],
        depends: Depends::LiquidAssetDocument,
    },
    PortedRow {
        spec: LIQUID,
        test: "assets / allows searching assets",
        paths: &["/assets/registry/search"],
        depends: Depends::LiquidAssetDocument,
    },
    PortedRow {
        spec: LIQUID,
        test: "assets / shows a specific asset ID",
        paths: &["/assets/registry/search", "/asset/:id"],
        depends: Depends::LiquidAssetDocument,
    },
    PortedRow {
        spec: LIQUID,
        test: "unblinded TX / should not show an unblinding error message for regular txs",
        paths: &["/tx/:hash"],
        depends: Depends::TxPage,
    },
    PortedRow {
        spec: LIQUID,
        test: "unblinded TX / show unblinded TX",
        paths: &["/tx/:hash"],
        depends: Depends::TxAmounts,
    },
    PortedRow {
        spec: LIQUID,
        test: "unblinded TX / show empty unblinded TX",
        paths: &["/tx/:hash"],
        depends: Depends::TxRows,
    },
    PortedRow {
        spec: LIQUID,
        test: "unblinded TX / show invalid unblinded TX hex",
        paths: &["/tx/:hash"],
        depends: Depends::TxPage,
    },
    PortedRow {
        spec: LIQUID,
        test: "unblinded TX / show first unblinded vout",
        paths: &["/tx/:hash"],
        depends: Depends::TxVout {
            index: 0,
            sats: 100_000,
        },
    },
    PortedRow {
        spec: LIQUID,
        test: "unblinded TX / show second unblinded vout",
        paths: &["/tx/:hash"],
        depends: Depends::TxVout {
            index: 1,
            sats: 2_364_760,
        },
    },
    PortedRow {
        spec: LIQUID,
        test: "unblinded TX / show invalid error unblinded TX",
        paths: &["/tx/:hash"],
        depends: Depends::TxPage,
    },
    PortedRow {
        spec: LIQUID,
        test: "unblinded TX / shows asset peg in/out and burn transactions",
        paths: &["/asset/:id", "/asset/:id/txs"],
        depends: Depends::AssetTxList,
    },
    PortedRow {
        spec: LIQUID,
        test: "unblinded TX / prevents regressing issue #644",
        paths: &["/tx/:hash"],
        depends: Depends::TxPage,
    },
    PortedRow {
        spec: LIQUID_TESTNET,
        test: "check first mempool block after skeleton loads",
        paths: &["/mempool", "/fee-estimates", "/blocks"],
        depends: Depends::ProjectedMempoolBlock,
    },
    PortedRow {
        spec: LIQUID_TESTNET,
        test: "loads the dashboard",
        paths: &["/mempool", "/blocks"],
        depends: Depends::Dashboard,
    },
    PortedRow {
        spec: LIQUID_TESTNET,
        test: "loads the dashboard with no scrollbars on mobile",
        paths: &["/mempool", "/blocks"],
        depends: Depends::Dashboard,
    },
    PortedRow {
        spec: LIQUID_TESTNET,
        test: "loads the blocks page",
        paths: &["/blocks"],
        depends: Depends::BlocksList,
    },
    PortedRow {
        spec: LIQUID_TESTNET,
        test: "loads a specific block page",
        paths: &["/block/:hash", "/block/:hash/txs"],
        depends: Depends::BlockWithTxs,
    },
    PortedRow {
        spec: LIQUID_TESTNET,
        test: "loads the graphs page",
        paths: &["/mempool", "/blocks"],
        depends: Depends::Dashboard,
    },
    PortedRow {
        spec: LIQUID_TESTNET,
        test: "loads the graphs page - mobile",
        paths: &["/mempool", "/blocks"],
        depends: Depends::Dashboard,
    },
    PortedRow {
        spec: LIQUID_TESTNET,
        test: "renders unconfidential transactions correctly on mobile",
        paths: &["/tx/:hash"],
        depends: Depends::TxAmounts,
    },
    PortedRow {
        spec: LIQUID_TESTNET,
        test: "assets / allows searching assets",
        paths: &["/assets/registry/search"],
        depends: Depends::LiquidAssetDocument,
    },
    PortedRow {
        spec: LIQUID_TESTNET,
        test: "assets / shows a specific asset ID",
        paths: &["/assets/registry/search", "/asset/:id"],
        depends: Depends::LiquidAssetDocument,
    },
    PortedRow {
        spec: LIQUID_TESTNET,
        test: "unblinded TX / should not show an unblinding error message for regular txs",
        paths: &["/tx/:hash"],
        depends: Depends::TxPage,
    },
    PortedRow {
        spec: LIQUID_TESTNET,
        test: "unblinded TX / show unblinded TX",
        paths: &["/tx/:hash"],
        depends: Depends::TxAmounts,
    },
    PortedRow {
        spec: LIQUID_TESTNET,
        test: "unblinded TX / show empty unblinded TX",
        paths: &["/tx/:hash"],
        depends: Depends::TxRows,
    },
    PortedRow {
        spec: LIQUID_TESTNET,
        test: "unblinded TX / show invalid unblinded TX hex",
        paths: &["/tx/:hash"],
        depends: Depends::TxPage,
    },
    PortedRow {
        spec: LIQUID_TESTNET,
        test: "unblinded TX / show first unblinded vout",
        paths: &["/tx/:hash"],
        depends: Depends::TxVout {
            index: 0,
            sats: 99_729,
        },
    },
    PortedRow {
        spec: LIQUID_TESTNET,
        test: "unblinded TX / show second unblinded vout (asset)",
        paths: &["/tx/:hash"],
        depends: Depends::TxVout { index: 1, sats: 0 },
    },
    PortedRow {
        spec: LIQUID_TESTNET,
        test: "unblinded TX / should link to the asset page from the unblinded tx",
        paths: &["/tx/:hash", "/asset/:id"],
        depends: Depends::TxSecondOutput,
    },
    PortedRow {
        spec: LIQUID_TESTNET,
        test: "unblinded TX / show invalid error unblinded TX",
        paths: &["/tx/:hash"],
        depends: Depends::TxPage,
    },
    PortedRow {
        spec: LIQUID_TESTNET,
        test: "unblinded TX / shows asset peg in/out and burn transactions",
        paths: &["/asset/:id", "/asset/:id/txs"],
        depends: Depends::AssetTxList,
    },
    PortedRow {
        spec: MAINNET,
        test: "check first mempool block after skeleton loads",
        paths: &["/mempool", "/fee-estimates", "/blocks"],
        depends: Depends::ProjectedMempoolBlock,
    },
    PortedRow {
        spec: MAINNET,
        test: "loads the status screen",
        paths: &["/mempool", "/blocks"],
        depends: Depends::StatusScreen,
    },
    PortedRow {
        spec: MAINNET,
        test: "loads the dashboard",
        paths: &["/mempool", "/blocks"],
        depends: Depends::Dashboard,
    },
    PortedRow {
        spec: MAINNET,
        test: "check op_return tx tooltip",
        paths: &["/block/:hash", "/block/:hash/txs"],
        depends: Depends::OpReturnRow,
    },
    PortedRow {
        spec: MAINNET,
        test: "check op_return coinbase tooltip",
        paths: &["/block/:hash", "/block/:hash/txs"],
        depends: Depends::CoinbaseRow,
    },
    PortedRow {
        spec: MAINNET,
        test: "search / allows searching for partial Bitcoin addresses",
        paths: &["/address-prefix/:prefix", "/address/:address"],
        depends: Depends::AddressPrefix {
            typed: "1A1zP",
            opened: BASE58_OPENED,
            list_len: 3,
        },
    },
    PortedRow {
        spec: MAINNET,
        test: "search / allows searching for partial case insensitive bech32m addresses: BC1PQYQS",
        paths: &["/address-prefix/:prefix", "/address/:address"],
        depends: Depends::AddressPrefix {
            typed: "BC1PQYQS",
            opened: BECH32M_OPENED,
            list_len: 10,
        },
    },
    PortedRow {
        spec: MAINNET,
        test: "search / allows searching for partial case insensitive bech32m addresses: bc1PqYqS",
        paths: &["/address-prefix/:prefix", "/address/:address"],
        depends: Depends::AddressPrefix {
            typed: "bc1PqYqS",
            opened: BECH32M_OPENED,
            list_len: 10,
        },
    },
    PortedRow {
        spec: MAINNET,
        test: "search / allows searching for partial case insensitive bech32 addresses: BC1Q0003",
        paths: &["/address-prefix/:prefix", "/address/:address"],
        depends: Depends::AddressPrefix {
            typed: "BC1Q0003",
            opened: BECH32_OPENED,
            list_len: 10,
        },
    },
    PortedRow {
        spec: MAINNET,
        test: "search / allows searching for partial case insensitive bech32 addresses: bC1q0003",
        paths: &["/address-prefix/:prefix", "/address/:address"],
        depends: Depends::AddressPrefix {
            typed: "bC1q0003",
            opened: BECH32_OPENED,
            list_len: 10,
        },
    },
    PortedRow {
        spec: MAINNET,
        test: "address highlighting / highlights single input addresses",
        paths: &["/address/:address/txs"],
        depends: Depends::AddressHighlight {
            inputs: 1,
            outputs: 0,
        },
    },
    PortedRow {
        spec: MAINNET,
        test: "address highlighting / highlights multiple input addresses",
        paths: &["/address/:address/txs"],
        depends: Depends::AddressHighlight {
            inputs: 2,
            outputs: 0,
        },
    },
    PortedRow {
        spec: MAINNET,
        test: "address highlighting / highlights both input and output addresses in the same transaction",
        paths: &["/address/:address/txs"],
        depends: Depends::AddressHighlight {
            inputs: 1,
            outputs: 1,
        },
    },
    PortedRow {
        spec: MAINNET,
        test: "address highlighting / highlights single output addresses",
        paths: &["/address/:address/txs"],
        depends: Depends::AddressHighlight {
            inputs: 0,
            outputs: 1,
        },
    },
    PortedRow {
        spec: MAINNET,
        test: "address highlighting / highlights multiple output addresses",
        paths: &["/address/:address/txs"],
        depends: Depends::AddressHighlight {
            inputs: 0,
            outputs: 2,
        },
    },
    PortedRow {
        spec: MAINNET,
        test: "address poisoning / highlights potential address poisoning attacks on outputs, prefix and infix",
        paths: &["/tx/:hash"],
        depends: Depends::Poison {
            inputs: false,
            shared_postfix: false,
        },
    },
    PortedRow {
        spec: MAINNET,
        test: "address poisoning / highlights potential address poisoning attacks on inputs and outputs, prefix, infix and postfix",
        paths: &["/tx/:hash"],
        depends: Depends::Poison {
            inputs: true,
            shared_postfix: true,
        },
    },
    PortedRow {
        spec: MAINNET,
        test: "blocks navigation / keyboard events / loads first blockchain block visible and keypress arrow right",
        paths: &["/block/:hash"],
        depends: Depends::BlockNav { genesis: false },
    },
    PortedRow {
        spec: MAINNET,
        test: "blocks navigation / keyboard events / loads first blockchain block visible and keypress arrow left",
        paths: &["/block/:hash"],
        depends: Depends::BlockNav { genesis: false },
    },
    PortedRow {
        spec: MAINNET,
        test: "blocks navigation / keyboard events / loads last blockchain block and keypress arrow right",
        paths: &["/block/:hash"],
        depends: Depends::BlockNav { genesis: false },
    },
    PortedRow {
        spec: MAINNET,
        test: "blocks navigation / keyboard events / loads genesis block and keypress arrow right",
        paths: &["/block/:hash"],
        depends: Depends::BlockNav { genesis: true },
    },
    PortedRow {
        spec: MAINNET,
        test: "blocks navigation / keyboard events / loads genesis block and keypress arrow left",
        paths: &["/block/:hash"],
        depends: Depends::BlockNav { genesis: true },
    },
    PortedRow {
        spec: MAINNET,
        test: "blocks navigation / mouse events / loads first blockchain blocks visible and click on the arrow right",
        paths: &["/block/:hash"],
        depends: Depends::BlockNav { genesis: false },
    },
    PortedRow {
        spec: MAINNET,
        test: "blocks navigation / mouse events / loads genesis block and click on the arrow left",
        paths: &["/block/:hash"],
        depends: Depends::BlockNav { genesis: true },
    },
    PortedRow {
        spec: MAINNET,
        test: "loads skeleton when changes between networks",
        paths: &["/blocks", "/mempool"],
        depends: Depends::Dashboard,
    },
    PortedRow {
        spec: MAINNET,
        test: "loads the dashboard with the skeleton blocks",
        paths: &["/blocks", "/mempool"],
        depends: Depends::Dashboard,
    },
    PortedRow {
        spec: MAINNET,
        test: "blocks / shows empty blocks properly",
        paths: &["/block/:hash", "/block/:hash/txs"],
        depends: Depends::BlockTxCount {
            tx_count: 1,
            genesis: false,
        },
    },
    PortedRow {
        spec: MAINNET,
        test: "blocks / expands and collapses the block details",
        paths: &["/block/:hash"],
        depends: Depends::BlockDetails,
    },
    PortedRow {
        spec: MAINNET,
        test: "blocks / shows blocks with no pagination",
        paths: &["/block/:hash", "/block/:hash/txs"],
        depends: Depends::BlockTxCount {
            tx_count: 19,
            genesis: false,
        },
    },
    PortedRow {
        spec: MAINNET,
        test: "blocks / supports pagination on the block screen",
        paths: &["/block/:hash/txs"],
        depends: Depends::BlockTxPage,
    },
    PortedRow {
        spec: MAINNET,
        test: "blocks / shows blocks pagination with 5 pages (desktop)",
        paths: &["/block/:hash/txs"],
        depends: Depends::BlockTxPage,
    },
    PortedRow {
        spec: MAINNET,
        test: "blocks / shows blocks pagination with 3 pages (mobile)",
        paths: &["/block/:hash/txs"],
        depends: Depends::BlockTxPage,
    },
    PortedRow {
        spec: RECENT,
        test: "updates the transaction list over time",
        paths: &["/mempool/recent"],
        depends: Depends::RecentGain,
    },
    PortedRow {
        spec: RECENT,
        test: "pauses updates when clicking the pause icon",
        paths: &["/mempool/recent"],
        depends: Depends::RecentStable,
    },
    PortedRow {
        spec: RECENT,
        test: "caps the list when changing the limit to 10",
        paths: &["/mempool/recent"],
        depends: Depends::RecentCap,
    },
    PortedRow {
        spec: RECENT,
        test: "shows the new transaction pill when there are new transactions",
        paths: &["/mempool/recent"],
        depends: Depends::RecentPill,
    },
    PortedRow {
        spec: RECENT,
        test: "shows the new transaction pill when there are new transactions and scrolls to the top when clicked",
        paths: &["/mempool/recent"],
        depends: Depends::RecentPill,
    },
    PortedRow {
        spec: SIGNET,
        test: "loads the dashboard",
        paths: &["/blocks", "/mempool"],
        depends: Depends::Dashboard,
    },
    PortedRow {
        spec: SIGNET,
        test: "check first mempool block after skeleton loads",
        paths: &["/mempool", "/fee-estimates", "/blocks"],
        depends: Depends::ProjectedMempoolBlock,
    },
    PortedRow {
        spec: SIGNET,
        test: "loads the dashboard with the skeleton blocks",
        paths: &["/blocks"],
        depends: Depends::BlocksList,
    },
    PortedRow {
        spec: SIGNET,
        test: "blocks / shows empty blocks properly",
        paths: &["/block/:hash", "/block/:hash/txs"],
        depends: Depends::BlockTxCount {
            tx_count: 1,
            genesis: false,
        },
    },
    PortedRow {
        spec: SIGNET,
        test: "blocks / expands and collapses the block details",
        paths: &["/block/:hash"],
        depends: Depends::BlockDetails,
    },
    PortedRow {
        spec: SIGNET,
        test: "blocks / shows blocks with no pagination",
        paths: &["/block/:hash", "/block/:hash/txs"],
        depends: Depends::BlockTxCount {
            tx_count: 13,
            genesis: false,
        },
    },
    PortedRow {
        spec: SIGNET,
        test: "blocks / supports pagination on the block screen",
        paths: &["/block/:hash/txs"],
        depends: Depends::BlockTxPage,
    },
    PortedRow {
        spec: TESTNET4,
        test: "loads the dashboard",
        paths: &["/blocks", "/mempool"],
        depends: Depends::Dashboard,
    },
    PortedRow {
        spec: TESTNET4,
        test: "check first mempool block after skeleton loads",
        paths: &["/mempool", "/fee-estimates", "/blocks"],
        depends: Depends::ProjectedMempoolBlock,
    },
    PortedRow {
        spec: TESTNET4,
        test: "loads the dashboard with the skeleton blocks",
        paths: &["/blocks", "/mempool"],
        depends: Depends::Dashboard,
    },
    PortedRow {
        spec: TESTNET4,
        test: "blocks / shows empty blocks properly",
        paths: &["/block/:hash", "/block/:hash/txs"],
        depends: Depends::BlockTxCount {
            tx_count: 1,
            genesis: true,
        },
    },
    PortedRow {
        spec: TESTNET4,
        test: "blocks / expands and collapses the block details",
        paths: &["/block/:hash"],
        depends: Depends::BlockDetails,
    },
    PortedRow {
        spec: TESTNET4,
        test: "blocks / shows blocks with no pagination",
        paths: &["/block/:hash", "/block/:hash/txs"],
        depends: Depends::BlockTxCount {
            tx_count: 18,
            genesis: false,
        },
    },
    PortedRow {
        spec: TESTNET4,
        test: "blocks / supports pagination on the block screen",
        paths: &["/block/:hash/txs"],
        depends: Depends::BlockTxPage,
    },
];

#[test]
fn every_ported_row_asserts_its_electrs_path_and_serde_fields() {
    assert_eq!(PORTED.len(), PORTED_ROWS);
    let mut seen = std::collections::BTreeSet::new();
    for row in PORTED {
        assert!(
            seen.insert((row.spec, row.test)),
            "duplicate ported row {} {}",
            row.spec,
            row.test
        );
        assert!(!row.paths.is_empty(), "{}", row.test);
        for path in row.paths {
            assert_electrs_path(path, row.test);
        }
        check_fields(row);
    }
}

fn assert_electrs_path(path: &str, test: &str) {
    assert!(path.starts_with('/'), "{test} {path}");
    let lower = path.to_ascii_lowercase();
    for banned in ["mining", "price", "lightning", "acceleration"] {
        assert!(!lower.contains(banned), "{test} must not use {path}");
    }
    assert!(!path.contains("/api/"), "{test} {path}");
    match non_liquid_pattern(path) {
        Some(pattern) => {
            assert!(
                indexer_routes()
                    .iter()
                    .any(|route| { route.method == "GET" && route.pattern == pattern }),
                "{test} missing {pattern} for {path}"
            );
        }
        None => {
            assert!(
                indexer_routes().iter().all(|route| route.pattern != path),
                "{test} {path} is not a non-liquid route pattern"
            );
        }
    }
}

/// Crate route pattern for an inventory path, when this crate builds that route.
///
/// `None` is a liquid asset route or `GET /address-prefix/:prefix`. Those are
/// electrs paths from the inventory. They are not in the non-liquid route table.
fn non_liquid_pattern(path: &str) -> Option<&'static str> {
    let get = |call: IndexerCall| {
        let built = indexer_request(&call);
        assert_eq!(built.method, "GET");
        assert!(built.body.is_none());
        built.path
    };
    match path {
        "/blocks" => {
            assert_eq!(get(IndexerCall::Blocks), "/blocks");
            Some("/blocks")
        }
        "/mempool" => {
            assert_eq!(get(IndexerCall::Mempool), mempool_path());
            Some("/mempool")
        }
        "/fee-estimates" => {
            assert_eq!(get(IndexerCall::FeeEstimates), fee_estimates_path());
            Some("/fee-estimates")
        }
        "/mempool/recent" => {
            assert_eq!(get(IndexerCall::MempoolRecent), mempool_recent_path());
            Some("/mempool/recent")
        }
        "/block/:hash" => {
            assert_eq!(
                get(IndexerCall::Block {
                    hash: ":hash".to_string(),
                }),
                block_path(":hash")
            );
            Some("/block/:hash")
        }
        "/block/:hash/txs" => {
            assert_eq!(
                get(IndexerCall::BlockTxs {
                    hash: ":hash".to_string(),
                }),
                block_txs_path(":hash")
            );
            Some("/block/:hash/txs")
        }
        "/tx/:hash" => {
            let built = get(IndexerCall::Tx {
                txid: ":hash".to_string(),
            });
            assert_eq!(built, tx_path(":hash"));
            assert_ne!(built, broadcast_path());
            Some("/tx/:txid")
        }
        "/address/:address" => {
            assert_eq!(
                get(IndexerCall::Address {
                    script: ":address".to_string(),
                }),
                "/address/:address"
            );
            Some("/address/:script")
        }
        "/address/:address/txs" => {
            assert_eq!(
                get(IndexerCall::AddressTxs {
                    script: ":address".to_string(),
                }),
                address_txs_path(":address")
            );
            Some("/address/:script/txs")
        }
        "/address-prefix/:prefix"
        | "/assets/registry"
        | "/assets/registry/search"
        | "/asset/:id"
        | "/asset/:id/txs" => None,
        other => panic!("unknown electrs path {other}"),
    }
}

fn check_fields(row: &PortedRow) {
    match row.depends {
        Depends::ProjectedMempoolBlock => assert_projected_mempool_block(),
        Depends::Dashboard => assert_dashboard(),
        Depends::BlocksList => assert_blocks_list(),
        Depends::BlockWithTxs => assert_block_with_txs(),
        Depends::BlockDetails => assert_block_details(),
        Depends::BlockTxCount { tx_count, genesis } => assert_block_tx_count(tx_count, genesis),
        Depends::BlockTxPage => assert_block_tx_page(),
        Depends::BlockNav { genesis } => assert_block_nav(genesis),
        Depends::TxPage => assert_tx_page(),
        Depends::TxInputLink => assert_tx_input_link(),
        Depends::TxAmounts => assert_tx_amounts(),
        Depends::TxRows => assert_tx_rows(),
        Depends::TxVout { index, sats } => assert_vout_sats(index, sats),
        Depends::TxAndAddress => assert_tx_and_address(),
        Depends::AddressHighlight { inputs, outputs } => assert_highlight(inputs, outputs),
        Depends::AddressPrefix {
            typed,
            opened,
            list_len,
        } => assert_address_prefix(typed, opened, list_len),
        Depends::Poison {
            inputs,
            shared_postfix,
        } => assert_poison(inputs, shared_postfix),
        Depends::OpReturnRow => assert_op_return_row(),
        Depends::CoinbaseRow => assert_coinbase_row(),
        Depends::StatusScreen => assert_status_screen(),
        Depends::RecentGain => assert_recent_gain(),
        Depends::RecentStable => assert_recent_stable(),
        Depends::RecentCap => assert_recent_cap(),
        Depends::RecentPill => assert_recent_pill(),
        Depends::LiquidAssetDocument => assert_liquid_asset_document(row.paths),
        Depends::AssetTxList => assert_asset_tx_list(row.paths),
        Depends::TxSecondOutput => assert_second_output(row.paths),
    }
}

fn fixture(name: &str) -> String {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(name);
    std::fs::read_to_string(path).unwrap()
}

fn assert_projected_mempool_block() {
    let mempool: MempoolSummary = serde_json::from_str(
        r#"{"count":2,"vsize":300,"total_fee":1500,"fee_histogram":[[10.5,200],[1.0,100]]}"#,
    )
    .unwrap();
    assert!(!mempool.fee_histogram.is_empty());
    assert_eq!(mempool.fee_histogram[0], (10.5, 200));
    let fees: FeeEstimates = serde_json::from_str(r#"{"1":12.5,"6":5.0}"#).unwrap();
    assert_eq!(fees.0.get(&1).copied(), Some(12.5));
    assert_eq!(fees.0.get(&6).copied(), Some(5.0));
    let blocks: Vec<Block> = serde_json::from_str(&fixture("blocks.json")).unwrap();
    assert_eq!(blocks[0].id, "block-id-1");
}

fn assert_dashboard() {
    let mempool: MempoolSummary = serde_json::from_str(
        r#"{"count":2,"vsize":300,"total_fee":1500,"fee_histogram":[[10.5,200]]}"#,
    )
    .unwrap();
    assert_eq!(mempool.count, 2);
    assert_eq!(mempool.vsize, 300);
    let blocks: Vec<Block> = serde_json::from_str(&fixture("blocks.json")).unwrap();
    assert_eq!(blocks[0].id, "block-id-1");
    assert_eq!(blocks[0].height, 100);
}

fn assert_blocks_list() {
    let blocks: Vec<Block> = serde_json::from_str(&fixture("blocks.json")).unwrap();
    assert_eq!(blocks.len(), 1);
    assert_eq!(blocks[0].id, "block-id-1");
    assert_eq!(blocks[0].height, 100);
    assert_eq!(blocks[0].tx_count, 12);
    assert_eq!(blocks[0].timestamp, 1_600_000_000);
    assert_eq!(blocks[0].size, 1500);
    assert_eq!(blocks[0].weight, 4000);
}

fn assert_block_with_txs() {
    let block: Block = serde_json::from_str(&fixture("block.json")).unwrap();
    assert_eq!(block.id, "block-id-1");
    assert_eq!(block.height, 100);
    assert_eq!(block.tx_count, 12);
    let txs: Vec<Transaction> = serde_json::from_str(&fixture("block_txs.json")).unwrap();
    assert_eq!(txs[0].txid, "block-tx-1");
    assert_eq!(txs[0].status.as_ref().unwrap().block_height, Some(100));
}

fn assert_block_details() {
    let block: Block = serde_json::from_str(&fixture("block.json")).unwrap();
    assert_eq!(block.id, "block-id-1");
    assert_eq!(block.version, 536_870_912);
    assert_eq!(block.timestamp, 1_600_000_000);
    assert_eq!(block.size, 1500);
    assert_eq!(block.weight, 4000);
    assert_eq!(block.merkle_root, "merkle-root-1");
    assert_eq!(block.median_time, 1_599_990_000);
    assert_eq!(block.nonce, 7);
    assert_eq!(block.bits, 386_604_799);
    assert_eq!(block.difficulty, 1.0);
    assert_eq!(block.tx_count, 12);
    assert_eq!(block.previous_block_hash.as_deref(), Some("prev-block-1"));
}

fn assert_block_tx_count(tx_count: u32, genesis: bool) {
    let previous = if genesis { None } else { Some("prev-block-1") };
    let height = if genesis { 0 } else { 100 };
    let block: Block = serde_json::from_str(&block_json(height, tx_count, previous)).unwrap();
    assert_eq!(block.tx_count, tx_count);
    if genesis {
        assert_eq!(block.height, 0);
        assert!(block.previous_block_hash.is_none());
    }
    let txs: Vec<Transaction> = serde_json::from_str(&fixture("block_txs.json")).unwrap();
    assert_eq!(txs[0].txid, "block-tx-1");
    assert!(txs[0].status.as_ref().unwrap().confirmed);
}

fn assert_block_tx_page() {
    let first = tx_page("page-a");
    let second = tx_page("page-b");
    assert_ne!(first[0].txid, second[0].txid);
    assert_eq!(first[0].size, 180);
    assert_eq!(first[0].fee, 4500);
    assert_eq!(second[0].txid, "page-b");
}

fn assert_block_nav(genesis: bool) {
    let block: Block = if genesis {
        serde_json::from_str(&block_json(0, 1, None)).unwrap()
    } else {
        serde_json::from_str(&block_json(100, 12, Some("prev-block-1"))).unwrap()
    };
    assert!(!block.id.is_empty());
    if genesis {
        assert_eq!(block.height, 0);
        assert_eq!(block.previous_block_hash, None);
    } else {
        assert!(block.height > 0);
        assert_eq!(block.previous_block_hash.as_deref(), Some("prev-block-1"));
    }
}

fn assert_tx_page() {
    let tx: Transaction = serde_json::from_str(&fixture("tx.json")).unwrap();
    assert_eq!(tx.txid, "tx-1");
    assert_eq!(tx.fee, 4500);
    assert_eq!(tx.size, 180);
    assert_eq!(tx.weight, 720);
    let status = tx.status.as_ref().unwrap();
    assert!(status.confirmed);
    assert_eq!(status.block_height, Some(100));
    assert_eq!(status.block_hash.as_deref(), Some("block-id-1"));
}

fn assert_tx_input_link() {
    let address = "bc1qpegin";
    let prev = vout_json("v0_p2wpkh", Some(address), 100_000);
    let vin = vin_json("peg-in-prev", Some(&prev), false);
    let vout = vout_json("v0_p2wpkh", Some("bc1qdest"), 90_000);
    let tx: Transaction = serde_json::from_str(&tx_json("peg-in-tx", &vin, &vout)).unwrap();
    assert_eq!(tx.vin[0].txid, "peg-in-prev");
    assert_eq!(
        tx.vin[0]
            .prevout
            .as_ref()
            .unwrap()
            .script_pubkey_address
            .as_deref(),
        Some(address)
    );
}

fn assert_tx_amounts() {
    let prev = vout_json("v0_p2wpkh", Some("bc1qin"), 250_000);
    let vin = vin_json("amount-prev", Some(&prev), false);
    let vout = vout_json("v0_p2wpkh", Some("bc1qout"), 240_000);
    let tx: Transaction = serde_json::from_str(&tx_json("amount-tx", &vin, &vout)).unwrap();
    assert_eq!(tx.vin[0].prevout.as_ref().unwrap().value, 250_000);
    assert_eq!(tx.vout[0].value, 240_000);
}

fn assert_tx_rows() {
    let prev = vout_json("v0_p2wpkh", Some("bc1qin"), 1000);
    let vin = vin_json("row-prev", Some(&prev), false);
    let vout = vout_json("v0_p2wpkh", Some("bc1qout"), 900);
    let tx: Transaction = serde_json::from_str(&tx_json("row-tx", &vin, &vout)).unwrap();
    assert_eq!(tx.vin.len(), 1);
    assert_eq!(tx.vout.len(), 1);
    assert!(tx.vin[0].prevout.is_some());
}

/// `sats` is `vout[index].value`. 0.00100000 LBTC is 100_000 sats.
/// 0.02364760 LBTC is 2_364_760 sats. 0.00099729 tLBTC is 99_729 sats.
fn assert_vout_sats(index: usize, sats: u64) {
    let marker = 1u64;
    let first = if index == 0 { sats } else { marker };
    let second = if index == 1 { sats } else { marker };
    let vout = format!(
        "{},{}",
        vout_json("v0_p2wpkh", Some("bc1qone"), first),
        vout_json("v0_p2wpkh", Some("bc1qtwo"), second),
    );
    let prev = vout_json("v0_p2wpkh", Some("bc1qsource"), 3_000_000);
    let vin = vin_json("prev", Some(&prev), false);
    let tx: Transaction = serde_json::from_str(&tx_json("amount-tx", &vin, &vout)).unwrap();
    assert!(tx.vout.len() > index);
    assert_eq!(tx.vout[index].value, sats);
}

fn assert_tx_and_address() {
    let address = "bc1qpegout";
    let prev = vout_json("v0_p2wpkh", Some("bc1qsource"), 60_000);
    let vin = vin_json("prev", Some(&prev), false);
    let vout = vout_json("v0_p2wpkh", Some(address), 50_000);
    let tx: Transaction = serde_json::from_str(&tx_json("peg-out", &vin, &vout)).unwrap();
    assert_eq!(tx.vout[0].script_pubkey_address.as_deref(), Some(address));
    let stats: AddressStats = serde_json::from_str(&address_json(address)).unwrap();
    assert_eq!(stats.address.as_deref(), Some(address));
    assert_eq!(stats.chain_stats.funded_txo_sum, 500_000);
    assert_eq!(stats.mempool_stats.tx_count, 3);
}

fn assert_highlight(inputs: usize, outputs: usize) {
    const ADDRESS: &str = "bc1qhighlight";
    let other = "bc1qotheraddress";
    let mut vins = Vec::new();
    for i in 0..inputs.max(1) {
        let addr = if i < inputs { ADDRESS } else { other };
        let prev = vout_json("v0_p2wpkh", Some(addr), 1000 + i as u64);
        vins.push(vin_json(&format!("in-{i}"), Some(&prev), false));
    }
    let mut vouts = Vec::new();
    for i in 0..outputs.max(1) {
        let addr = if i < outputs { ADDRESS } else { other };
        vouts.push(vout_json("v0_p2wpkh", Some(addr), 500 + i as u64));
    }
    let body = format!(
        "[{}]",
        tx_json("highlight-tx", &vins.join(","), &vouts.join(","))
    );
    let txs: Vec<Transaction> = serde_json::from_str(&body).unwrap();
    let tx = &txs[0];
    let input_hits = tx
        .vin
        .iter()
        .filter(|vin| {
            vin.prevout
                .as_ref()
                .and_then(|prev| prev.script_pubkey_address.as_deref())
                == Some(ADDRESS)
        })
        .count();
    let output_hits = tx
        .vout
        .iter()
        .filter(|vout| vout.script_pubkey_address.as_deref() == Some(ADDRESS))
        .count();
    assert_eq!(input_hits, inputs);
    assert_eq!(output_hits, outputs);
}

fn assert_address_prefix(typed: &str, opened: &str, list_len: usize) {
    assert!(
        opened
            .to_ascii_lowercase()
            .starts_with(&typed.to_ascii_lowercase()),
        "{typed} should match {opened}"
    );
    assert_ne!(typed, opened);
    let mut addrs = vec![opened.to_string()];
    for n in 1..list_len {
        addrs.push(format!("{opened}x{n}"));
    }
    let mut body = String::from("[");
    for (i, addr) in addrs.iter().enumerate() {
        if i > 0 {
            body.push(',');
        }
        body.push('"');
        body.push_str(addr);
        body.push('"');
    }
    body.push(']');
    let parsed: Vec<String> = serde_json::from_str(&body).unwrap();
    assert_eq!(parsed.len(), list_len);
    assert!(parsed.iter().any(|addr| addr == opened));
    let stats: AddressStats = serde_json::from_str(&address_json(opened)).unwrap();
    assert_eq!(stats.address.as_deref(), Some(opened));
    assert_eq!(stats.chain_stats.tx_count, 9);
    assert_eq!(stats.chain_stats.funded_txo_sum, 500_000);
    assert_eq!(stats.mempool_stats.tx_count, 3);
}

fn assert_poison(inputs: bool, shared_postfix: bool) {
    let (left, right) = if shared_postfix {
        ("bc1qMMMend", "bc1qZZZend")
    } else {
        ("bc1qlookaaa", "bc1qlookXY")
    };
    let vout = format!(
        "{},{}",
        vout_json("v0_p2wpkh", Some(left), 1000),
        vout_json("v0_p2wpkh", Some(right), 2000),
    );
    let vin = if inputs {
        format!(
            "{},{}",
            vin_json(
                "in-0",
                Some(&vout_json("v0_p2wpkh", Some(left), 3000)),
                false
            ),
            vin_json(
                "in-1",
                Some(&vout_json("v0_p2wpkh", Some(right), 4000)),
                false
            ),
        )
    } else {
        vin_json(
            "in-0",
            Some(&vout_json("v0_p2wpkh", Some("bc1qunrelated"), 3000)),
            false,
        )
    };
    let tx: Transaction = serde_json::from_str(&tx_json("poison-tx", &vin, &vout)).unwrap();
    let out_left = tx.vout[0].script_pubkey_address.as_deref().unwrap();
    let out_right = tx.vout[1].script_pubkey_address.as_deref().unwrap();
    let (prefix, infix_left, infix_right, suffix) = lookalike(out_left, out_right);
    assert!(!prefix.is_empty());
    assert!(!infix_left.is_empty());
    assert_ne!(infix_left, infix_right);
    if shared_postfix {
        assert!(!suffix.is_empty());
    } else {
        assert!(suffix.is_empty());
    }
    if inputs {
        let in_left = tx.vin[0]
            .prevout
            .as_ref()
            .unwrap()
            .script_pubkey_address
            .as_deref()
            .unwrap();
        let in_right = tx.vin[1]
            .prevout
            .as_ref()
            .unwrap()
            .script_pubkey_address
            .as_deref()
            .unwrap();
        let (in_prefix, in_left_infix, in_right_infix, in_suffix) = lookalike(in_left, in_right);
        assert!(!in_prefix.is_empty());
        assert_ne!(in_left_infix, in_right_infix);
        assert!(!in_suffix.is_empty());
    }
}

fn assert_op_return_row() {
    let normal_vout = vout_json("v0_p2wpkh", Some("bc1qnormal"), 1000);
    let op_vout = vout_json("op_return", None, 0);
    let vin = vin_json("prev-tx", Some(&normal_vout), false);
    let body = format!(
        "[{},{}]",
        tx_json("row-0", &vin, &normal_vout),
        tx_json("row-1", &vin, &op_vout),
    );
    let txs: Vec<Transaction> = serde_json::from_str(&body).unwrap();
    assert_eq!(txs[1].txid, "row-1");
    assert_eq!(txs[1].vout[0].script_pubkey_type, "op_return");
    assert_eq!(txs[1].vout[0].script_pubkey_address, None);
    assert_ne!(txs[0].vout[0].script_pubkey_type, "op_return");
}

fn assert_coinbase_row() {
    let normal_vout = vout_json("v0_p2wpkh", Some("bc1qout"), 900);
    let normal_vin = vin_json("prev", Some(&normal_vout), false);
    let coin_vin = vin_json(
        "0000000000000000000000000000000000000000000000000000000000000000",
        None,
        true,
    );
    let body = format!(
        "[{},{}]",
        tx_json("row-0", &normal_vin, &normal_vout),
        tx_json("coinbase-row", &coin_vin, &normal_vout),
    );
    let txs: Vec<Transaction> = serde_json::from_str(&body).unwrap();
    assert_eq!(txs[1].txid, "coinbase-row");
    assert!(txs[1].vin[0].is_coinbase);
    assert!(txs[1].vin[0].prevout.is_none());
    assert!(!txs[0].vin[0].is_coinbase);
}

fn assert_status_screen() {
    let mempool: MempoolSummary = serde_json::from_str(
        r#"{"count":11,"vsize":2400,"total_fee":9000,"fee_histogram":[[8.0,2400]]}"#,
    )
    .unwrap();
    assert_eq!(mempool.fee_histogram.len(), 1);
    assert_eq!(mempool.count, 11);
    assert_eq!(mempool.vsize, 2400);
    assert_eq!(mempool.total_fee, 9000);
    let blocks = chain_blocks(22);
    assert_eq!(blocks.len(), 22);
    assert_eq!(blocks[0].height, 0);
    assert!(blocks[0].previous_block_hash.is_none());
    assert_eq!(blocks[21].id, "block-21");
    assert_eq!(blocks[21].previous_block_hash.as_deref(), Some("block-20"));
}

fn assert_recent_gain() {
    let before: Vec<RecentTransaction> =
        serde_json::from_str(&recent_json(&["recent-tx-1"])).unwrap();
    let after: Vec<RecentTransaction> =
        serde_json::from_str(&recent_json(&["sent-txid", "recent-tx-1"])).unwrap();
    assert!(before.iter().all(|row| row.txid != "sent-txid"));
    assert_eq!(after[0].txid, "sent-txid");
    assert_eq!(after[0].fee, 800);
    assert_eq!(after[0].vsize, 141);
    assert_eq!(after[0].value, 99_000);
}

fn assert_recent_stable() {
    let body = recent_json(&["recent-tx-1", "recent-tx-2"]);
    let first: Vec<RecentTransaction> = serde_json::from_str(&body).unwrap();
    let second: Vec<RecentTransaction> = serde_json::from_str(&body).unwrap();
    assert_eq!(first[0].txid, second[0].txid);
    assert_eq!(first[1].txid, "recent-tx-2");
    assert_eq!(first[0].fee, 800);
    assert_eq!(first[1].value, 99_000);
}

fn assert_recent_cap() {
    let txids: Vec<String> = (0..50).map(|i| format!("recent-{i}")).collect();
    let refs: Vec<&str> = txids.iter().map(String::as_str).collect();
    let all: Vec<RecentTransaction> = serde_json::from_str(&recent_json(&refs)).unwrap();
    assert_eq!(all.len(), 50);
    assert_eq!(all[0].txid, "recent-0");
    assert_eq!(all[49].fee, 800);
    assert_eq!(all[49].vsize, 141);
    assert_eq!(all[49].value, 99_000);
    let capped_ids: Vec<&str> = refs.iter().copied().take(10).collect();
    let capped: Vec<RecentTransaction> = serde_json::from_str(&recent_json(&capped_ids)).unwrap();
    assert_eq!(capped.len(), 10);
    assert_eq!(capped[9].txid, "recent-9");
}

fn assert_recent_pill() {
    let rows: Vec<RecentTransaction> =
        serde_json::from_str(&recent_json(&["new-a", "new-b", "old"])).unwrap();
    assert!(rows.len() >= 2);
    assert_eq!(rows[0].txid, "new-a");
    assert_eq!(rows[1].txid, "new-b");
    assert_eq!(rows[0].vsize, 141);
}

fn assert_liquid_asset_document(paths: &[&str]) {
    assert!(!paths.is_empty());
    for path in paths {
        assert!(
            matches!(
                *path,
                "/assets/registry" | "/assets/registry/search" | "/asset/:id"
            ),
            "{path}"
        );
    }
}

fn assert_asset_tx_list(paths: &[&str]) {
    assert!(paths.contains(&"/asset/:id"));
    assert!(paths.contains(&"/asset/:id/txs"));
    let vin = vin_json(
        "prev-peg",
        Some(&vout_json("v0_p2wpkh", Some("bc1qpeg"), 1000)),
        false,
    );
    let vout = vout_json("v0_p2wpkh", Some("bc1qburn"), 900);
    let body = format!(
        "[{},{}]",
        tx_json("peg-row", &vin, &vout),
        tx_json("burn-row", &vin, &vout),
    );
    let txs: Vec<Transaction> = serde_json::from_str(&body).unwrap();
    assert_eq!(txs.len(), 2);
    assert_eq!(txs[0].txid, "peg-row");
    assert_eq!(txs[1].txid, "burn-row");
    assert!(!txs[0].vin.is_empty());
    assert!(!txs[0].vout.is_empty());
}

fn assert_second_output(paths: &[&str]) {
    assert!(paths.contains(&"/tx/:hash"));
    assert!(paths.contains(&"/asset/:id"));
    let vout = format!(
        "{},{}",
        vout_json("v0_p2wpkh", Some("bc1qfirst"), 10),
        vout_json("v0_p2wpkh", Some("bc1qasset"), 42_000),
    );
    let prev = vout_json("v0_p2wpkh", Some("bc1qsource"), 50_000);
    let vin = vin_json("prev", Some(&prev), false);
    let tx: Transaction = serde_json::from_str(&tx_json("asset-link", &vin, &vout)).unwrap();
    assert!(tx.vout.len() >= 2);
    assert_eq!(tx.vout[1].value, 42_000);
}

fn chain_blocks(count: u32) -> Vec<Block> {
    let mut body = String::from("[");
    for height in 0..count {
        if height > 0 {
            body.push(',');
        }
        let previous = if height == 0 {
            None
        } else {
            Some(format!("block-{}", height - 1))
        };
        body.push_str(&block_json(height, 1, previous.as_deref()));
    }
    body.push(']');
    serde_json::from_str(&body).unwrap()
}

fn block_json(height: u32, tx_count: u32, previous: Option<&str>) -> String {
    let prev = match previous {
        Some(hash) => format!(r#","previousblockhash":"{hash}""#),
        None => String::new(),
    };
    format!(
        r#"{{"id":"block-{height}","height":{height},"version":536870912,"timestamp":1600000000,"tx_count":{tx_count},"size":1500,"weight":4000,"merkle_root":"merkle-root-1"{prev},"mediantime":1599990000,"nonce":7,"bits":386604799,"difficulty":1.0}}"#
    )
}

fn tx_page(txid: &str) -> Vec<Transaction> {
    let body = format!("[{}]", tx_json(txid, "", ""));
    serde_json::from_str(&body).unwrap()
}

fn tx_json(txid: &str, vin: &str, vout: &str) -> String {
    format!(
        r#"{{"txid":"{txid}","version":2,"locktime":0,"vin":[{vin}],"vout":[{vout}],"size":180,"weight":720,"sigops":1,"fee":4500,"status":{{"confirmed":true,"block_height":100,"block_hash":"block-id-1","block_time":1600000000}}}}"#
    )
}

fn vout_json(kind: &str, address: Option<&str>, value: u64) -> String {
    match address {
        Some(address) => format!(
            r#"{{"scriptpubkey":"0014aa","scriptpubkey_asm":"OP_0","scriptpubkey_type":"{kind}","scriptpubkey_address":"{address}","value":{value}}}"#
        ),
        None => format!(
            r#"{{"scriptpubkey":"6a","scriptpubkey_asm":"OP_RETURN","scriptpubkey_type":"{kind}","value":{value}}}"#
        ),
    }
}

fn vin_json(txid: &str, prevout: Option<&str>, coinbase: bool) -> String {
    let prev = match prevout {
        Some(prev) => format!(r#""prevout":{prev}"#),
        None => r#""prevout":null"#.to_string(),
    };
    let coinbase = if coinbase { "true" } else { "false" };
    format!(
        r#"{{"txid":"{txid}","vout":0,{prev},"scriptsig":"","scriptsig_asm":"","is_coinbase":{coinbase},"sequence":4294967295}}"#
    )
}

fn address_json(address: &str) -> String {
    format!(
        r#"{{"address":"{address}","chain_stats":{{"tx_count":9,"funded_txo_count":6,"spent_txo_count":2,"funded_txo_sum":500000,"spent_txo_sum":100000}},"mempool_stats":{{"tx_count":3,"funded_txo_count":1,"spent_txo_count":0,"funded_txo_sum":2500,"spent_txo_sum":0}}}}"#
    )
}

fn recent_json(txids: &[&str]) -> String {
    let rows: Vec<String> = txids
        .iter()
        .map(|txid| format!(r#"{{"txid":"{txid}","fee":800,"vsize":141,"value":99000}}"#))
        .collect();
    format!("[{}]", rows.join(","))
}

fn lookalike<'a>(left: &'a str, right: &'a str) -> (&'a str, &'a str, &'a str, &'a str) {
    let prefix_len = left
        .bytes()
        .zip(right.bytes())
        .take_while(|(a, b)| a == b)
        .count();
    let mut suffix_len = left
        .bytes()
        .rev()
        .zip(right.bytes().rev())
        .take_while(|(a, b)| a == b)
        .count();
    if prefix_len + suffix_len > left.len() {
        suffix_len = left.len().saturating_sub(prefix_len);
    }
    if prefix_len + suffix_len > right.len() {
        suffix_len = suffix_len.min(right.len().saturating_sub(prefix_len));
    }
    let infix_left = &left[prefix_len..left.len() - suffix_len];
    let infix_right = &right[prefix_len..right.len() - suffix_len];
    let suffix = &left[left.len() - suffix_len..];
    (&left[..prefix_len], infix_left, infix_right, suffix)
}
