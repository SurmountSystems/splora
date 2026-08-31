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
    let rest_server = rest::start(
        Arc::clone(&config),
        Arc::clone(&query),
        &metrics,
        Arc::clone(&allow),
        Arc::clone(&hub),
    );
    let electrum_server = ElectrumRPC::start(Arc::clone(&config), Arc::clone(&query), &metrics);

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

            electrs::util::spawn_thread("shutdown-thread-checker", || {
                let mut counter = 40;
                let interval_ms = 500;

                while counter > 0 {
                    electrs::util::with_spawned_threads(|threads| {
                        debug!("Threads during shutdown: {:?}", threads);
                    });
                    std::thread::sleep(std::time::Duration::from_millis(interval_ms));
                    counter -= 1;
                }
            });

            rest_server.stop();
            // the electrum server is stopped when dropped
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
                if let Some(tx) = query.lookup_txn(txid) {
                    if let Some(full) =
                        rest::transactions_as_json(vec![(tx, None)], &query, &config)
                            .into_iter()
                            .next()
                    {
                        available.insert(txid.to_string(), full);
                    }
                }
            }
            let removed = prefer_full_removed(&removed_stubs, &available);
            if !added.is_empty() || !removed.is_empty() {
                hub.notify_mempool(&added, &removed);
            }
        }

        // Update subscribed clients
        electrum_server.notify();
    }
    info!("server stopped");
    Ok(())
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
