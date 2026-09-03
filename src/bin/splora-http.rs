// SPDX-License-Identifier: Unlicense
//! Public TLS path-routing front. Proxies Esplora REST and `/api/v1/ws` to
//! per-network HTTP unix sockets. Does not talk to `*.electrum.sock`.

use electrs::http_front::run_from_args;
use std::process;

fn main() {
    if let Err(e) = run_from_args() {
        eprintln!("{}", e);
        process::exit(1);
    }
}
