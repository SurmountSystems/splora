// SPDX-License-Identifier: Unlicense
//! Localhost import-queue HTTP process. This binary only calls `queue::run`.
//! It is not the indexer. Do not pass indexer flags here.

use electrs::queue::run_from_args;
use std::process;

fn main() {
    if let Err(e) = run_from_args() {
        eprintln!("{}", e);
        process::exit(1);
    }
}
