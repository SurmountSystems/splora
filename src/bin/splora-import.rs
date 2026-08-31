// SPDX-License-Identifier: Unlicense
//! Operator-only localhost CLI for the npub approval queue.
//! Never restarts or kills splora. The running indexer only watches the allowlist file.
//!
//! Usage:
//!   splora-import approve --queue <path> --allowlist <path> <npub>
//!   splora-import reject --queue <path> <npub>
//!   splora-import remove --allowlist <path> <npub>

use clap::ArgMatches;
use electrs::queue::{approve, import_cli_app, reject, remove_npub};
use std::path::Path;
use std::process;

fn main() {
    let matches = import_cli_app().get_matches();
    let result = dispatch(&matches);

    if let Err(e) = result {
        eprintln!("{}", e);
        process::exit(1);
    }
}

fn dispatch(matches: &ArgMatches) -> Result<(), electrs::queue::QueueError> {
    if let Some(m) = matches.subcommand_matches("approve") {
        approve(
            Path::new(m.value_of("queue").unwrap()),
            Path::new(m.value_of("allowlist").unwrap()),
            m.value_of("npub").unwrap(),
        )
    } else if let Some(m) = matches.subcommand_matches("reject") {
        reject(
            Path::new(m.value_of("queue").unwrap()),
            m.value_of("npub").unwrap(),
        )
    } else if let Some(m) = matches.subcommand_matches("remove") {
        remove_npub(
            Path::new(m.value_of("allowlist").unwrap()),
            m.value_of("npub").unwrap(),
        )
    } else {
        eprintln!("usage: splora-import <approve|reject|remove> ...");
        process::exit(2);
    }
}
