extern crate error_chain;
#[macro_use]
extern crate log;

extern crate electrs;

use error_chain::ChainedError;
use serde_json::json;
use std::process;
use std::sync::{Arc, RwLock};
use std::time::Duration;

use electrs::{
    auth::Allowlist,
    config::Config,
    daemon::Daemon,
    electrum::RPC as ElectrumRPC,
    errors::*,
    metrics::Metrics,
    mwck::{MwckHub, prefer_full_removed},
    new_index::{ChainQuery, FetchFrom, Indexer, Mempool, Query, Store, precache},
    rest,
    signal::Waiter,
};

#[cfg(feature = "liquid")]
use electrs::elements::AssetRegistry;

fn fetch_from(config: &Config, store: &Store) -> FetchFrom {
    let mut jsonrpc_import = config.jsonrpc_import;
    if !jsonrpc_import {
        // switch over to jsonrpc after the initial sync is done
        jsonrpc_import = store.done_initial_sync();
    }

    if jsonrpc_import {
        // slower, uses JSONRPC (good for incremental updates)
        FetchFrom::Bitcoind
    } else {
        // faster, uses blk*.dat files (good for initial indexing)
        FetchFrom::BlkFiles
    }
}

/// Mempool electrs#154 watchdog. `process::exit(0)` skips Drop, including
/// RocksDB close/flush and Electrum join. This tree flushes after Electrum
/// join today, so the watchdog is armed only after rest-stop, join, and flush.
const SHUTDOWN_WATCHDOG_TIMEOUT: Duration = Duration::from_secs(5);
const SHUTDOWN_WATCHDOG_POLL: Duration = Duration::from_millis(500);
const SHUTDOWN_WATCHDOG_THREAD: &str = "shutdown-watchdog";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ShutdownPhase {
    RestStop,
    ElectrumJoin,
    Flush,
    ArmWatchdog,
}

fn shutdown_phases() -> &'static [ShutdownPhase] {
    &[
        ShutdownPhase::RestStop,
        ShutdownPhase::ElectrumJoin,
        ShutdownPhase::Flush,
        ShutdownPhase::ArmWatchdog,
    ]
}

fn flush_index_store(store: &Store) {
    store.txstore_db().flush();
    store.history_db().flush();
    store.cache_db().flush();
}

fn spawn_shutdown_watchdog() {
    let timeout = SHUTDOWN_WATCHDOG_TIMEOUT;
    let interval = SHUTDOWN_WATCHDOG_POLL;
    electrs::util::spawn_thread(SHUTDOWN_WATCHDOG_THREAD, move || {
        let mut elapsed = Duration::ZERO;
        while elapsed < timeout {
            electrs::util::with_spawned_threads(|threads| {
                debug!("Threads during shutdown: {:?}", threads);
            });
            std::thread::sleep(interval);
            elapsed += interval;
        }

        // Skips destructors (no Drop for Store/Electrum). Flush and Electrum
        // join already ran; this only unsticks leftover threads.
        error!("graceful shutdown timed out after 5 seconds, forcing exit");
        process::exit(0);
    });
}

fn run_server(config: Arc<Config>) -> Result<()> {
    let signal = Waiter::start();
    let metrics = Metrics::new(config.monitoring_addr);
    metrics.start();

    let daemon = Arc::new(Daemon::new(
        config.daemon_dir.clone(),
        config.blocks_dir.clone(),
        config.daemon_rpc_addr,
        config.cookie_getter(),
        config.network_type,
        config.magic,
        signal.clone(),
        &metrics,
    )?);
    let store = Arc::new(Store::open(&config.db_path.join("newindex"), &config));
    let mut indexer = Indexer::open(
        Arc::clone(&store),
        fetch_from(&config, &store),
        &config,
        &metrics,
    );
    let mut tip = indexer.update(&daemon)?;

    let chain = Arc::new(ChainQuery::new(
        Arc::clone(&store),
        Arc::clone(&daemon),
        &config,
        &metrics,
    ));

    let mempool = Arc::new(RwLock::new(Mempool::new(
        Arc::clone(&chain),
        &metrics,
        Arc::clone(&config),
    )));
    loop {
        match Mempool::update(&mempool, &daemon) {
            Ok(_) => break,
            Err(e) => {
                warn!(
                    "Error performing initial mempool update, trying again in 5 seconds: {}",
                    e.display_chain()
                );
                signal.wait(Duration::from_secs(5), false)?;
            }
        }
    }

    #[cfg(feature = "liquid")]
    let asset_db = config.asset_db_path.as_ref().map(|db_dir| {
        let asset_db = Arc::new(RwLock::new(AssetRegistry::new(db_dir.clone())));
        AssetRegistry::spawn_sync(asset_db.clone());
        asset_db
    });

    let query = Arc::new(Query::new(
        Arc::clone(&chain),
        Arc::clone(&mempool),
        Arc::clone(&daemon),
        Arc::clone(&config),
        #[cfg(feature = "liquid")]
        asset_db,
    ));

    let allow = if let Some(path) = config.allow_npubs_file.as_ref() {
        Allowlist::load(path).chain_err(|| "failed to load --allow-npubs-file")?
    } else {
        warn!("no --allow-npubs-file: HTTP REST, POST /electrum, and /api/v1/ws authorize nobody");
        Allowlist::deny_all()
    };
    let _allow_watch = if config.allow_npubs_file.is_some() {
        Some(
            Arc::clone(&allow)
                .watch()
                .chain_err(|| "failed to watch --allow-npubs-file")?,
        )
    } else {
        None
    };
    let hub = MwckHub::new(Arc::clone(&query));

    // Queue HTTP is a separate binary. Do not put unauthenticated POST on the indexer.
    let mut rest_server = Some(rest::start(
        Arc::clone(&config),
        Arc::clone(&query),
        &metrics,
        Arc::clone(&allow),
        Arc::clone(&hub),
    ));
    let mut electrum_server = Some(ElectrumRPC::start(
        Arc::clone(&config),
        Arc::clone(&query),
        &metrics,
    ));

    if let Some(ref precache_file) = config.precache_scripts {
        let precache_scripthashes = precache::scripthashes_from_file(precache_file.to_string())
            .expect("cannot load scripts to precache");
        precache::precache(
            Arc::clone(&chain),
            precache_scripthashes,
            config.precache_threads,
        );
    }

    loop {
        if let Err(err) = signal.wait(Duration::from_millis(config.main_loop_delay), true) {
            info!("stopping server: {}", err);

            for phase in shutdown_phases() {
                match phase {
                    ShutdownPhase::RestStop => {
                        rest_server.take().expect("REST handle").stop();
                    }
                    ShutdownPhase::ElectrumJoin => {
                        // Electrum RPC joins on Drop (Notification::Exit).
                        drop(electrum_server.take());
                    }
                    ShutdownPhase::Flush => {
                        flush_index_store(&store);
                    }
                    ShutdownPhase::ArmWatchdog => {
                        spawn_shutdown_watchdog();
                    }
                }
            }
            break;
        }

        // Index new blocks
        let prev_height = chain.best_height();
        let prev_txids = mempool.read().unwrap().unique_txids();
        let current_tip = daemon.getbestblockhash()?;
        if current_tip != tip {
            indexer.update(&daemon)?;
            tip = current_tip;
            let new_height = chain.best_height();
            hub.notify_new_tip(prev_height, new_height);
        };

        // Update mempool
        if let Err(e) = Mempool::update(&mempool, &daemon) {
            // Log the error if the result is an Err
            warn!(
                "Error updating mempool, skipping mempool update: {}",
                e.display_chain()
            );
        } else {
            let now_txids = mempool.read().unwrap().unique_txids();
            let added: Vec<serde_json::Value> = now_txids
                .difference(&prev_txids)
                .filter_map(|txid| {
                    query.lookup_txn(txid).and_then(|tx| {
                        rest::transactions_as_json(vec![(tx, None)], &query, &config)
                            .into_iter()
                            .next()
                    })
                })
                .collect();
            let removed_stubs: Vec<serde_json::Value> = prev_txids
                .difference(&now_txids)
                .map(|txid| json!({ "txid": txid.to_string() }))
                .collect();
            let mut available = std::collections::BTreeMap::new();
            for txid in prev_txids.difference(&now_txids) {
                if let Some(tx) = query.lookup_txn(txid)
                    && let Some(full) =
                        rest::transactions_as_json(vec![(tx, None)], &query, &config)
                            .into_iter()
                            .next()
                {
                    available.insert(txid.to_string(), full);
                }
            }
            let removed = prefer_full_removed(&removed_stubs, &available);
            if !added.is_empty() || !removed.is_empty() {
                hub.notify_mempool(&added, &removed);
            }
        }

        // Update subscribed clients
        electrum_server
            .as_ref()
            .expect("Electrum RPC still running")
            .notify();
    }
    info!("server stopped");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shutdown_watchdog_armed_after_rest_stop_join_and_flush() {
        assert_eq!(SHUTDOWN_WATCHDOG_TIMEOUT, Duration::from_secs(5));
        assert_eq!(SHUTDOWN_WATCHDOG_POLL, Duration::from_millis(500));
        assert_eq!(SHUTDOWN_WATCHDOG_THREAD, "shutdown-watchdog");

        let phases = shutdown_phases();
        let rest = phases
            .iter()
            .position(|p| *p == ShutdownPhase::RestStop)
            .expect("rest-stop");
        let join = phases
            .iter()
            .position(|p| *p == ShutdownPhase::ElectrumJoin)
            .expect("electrum-join");
        let flush = phases
            .iter()
            .position(|p| *p == ShutdownPhase::Flush)
            .expect("flush");
        let arm = phases
            .iter()
            .position(|p| *p == ShutdownPhase::ArmWatchdog)
            .expect("arm-watchdog");

        assert!(rest < join, "stop order is rest-stop then join");
        assert!(
            join < flush,
            "RocksDB flush stays after Electrum join, as today"
        );
        assert!(
            flush < arm,
            "watchdog (process::exit) is armed only after flush"
        );
        assert_eq!(
            phases,
            &[
                ShutdownPhase::RestStop,
                ShutdownPhase::ElectrumJoin,
                ShutdownPhase::Flush,
                ShutdownPhase::ArmWatchdog,
            ]
        );
    }
}

fn main() {
    let config = Arc::new(Config::from_args());
    if let Err(e) = run_server(config) {
        error!("server failed: {}", e.display_chain());
        process::exit(1);
    }
    electrs::util::with_spawned_threads(|threads| {
        debug!("Threads before closing: {:?}", threads);
    });
}
