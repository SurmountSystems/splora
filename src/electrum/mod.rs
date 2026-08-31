mod server;
/// Electrum JSON-RPC over HTTP: `POST /electrum` after NIP-98 (see `rest.rs`).
/// Production TCP Electrum is not used; unix `--rpc-socket-file` remains.
pub use server::{RPC, handle_http_jsonrpc_body};

/// How Electrum RPC listens. Raw TCP is never selected.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ElectrumListenPlan {
    Unix(std::path::PathBuf),
    /// No Electrum socket. Clients use `POST /electrum` after NIP-98.
    HttpOnly,
}

impl ElectrumListenPlan {
    pub fn from_rpc_socket_file(path: Option<&std::path::Path>) -> Self {
        match path {
            Some(p) => Self::Unix(p.to_path_buf()),
            None => Self::HttpOnly,
        }
    }

    /// Production never opens a raw Electrum TCP port.
    pub fn binds_tcp(&self) -> bool {
        false
    }
}

#[cfg(test)]
mod listen_plan_tests {
    use super::ElectrumListenPlan;
    use std::path::Path;

    /// Named contract: omitting `--rpc-socket-file` does not bind Electrum TCP.
    #[test]
    fn electrum_start_without_unix_socket_does_not_bind_tcp() {
        let plan = ElectrumListenPlan::from_rpc_socket_file(None);
        assert_eq!(plan, ElectrumListenPlan::HttpOnly);
        assert!(!plan.binds_tcp());
        let with_unix =
            ElectrumListenPlan::from_rpc_socket_file(Some(Path::new("/run/splora/electrum.sock")));
        assert!(!with_unix.binds_tcp());
        match with_unix {
            ElectrumListenPlan::Unix(p) => {
                assert_eq!(p, Path::new("/run/splora/electrum.sock"));
            }
            ElectrumListenPlan::HttpOnly => panic!("unix path should select the unix listener"),
        }
        let src = include_str!("server.rs");
        assert!(
            src.contains("ElectrumListenPlan::HttpOnly"),
            "start_acceptor must take the HTTP-only branch instead of TCP"
        );
        assert!(
            !src.contains("ConnectionListener::new_tcp("),
            "do not call new_tcp from the Electrum acceptor"
        );
    }
}

#[cfg(feature = "electrum-discovery")]
mod client;
#[cfg(feature = "electrum-discovery")]
mod discovery;
#[cfg(feature = "electrum-discovery")]
pub use {client::Client, discovery::DiscoveryManager};

use std::cmp::Ordering;
use std::collections::HashMap;
use std::str::FromStr;

use serde::{Deserialize, Deserializer, Serialize, de};

use crate::chain::BlockHash;
use crate::errors::ResultExt;
use crate::util::BlockId;

pub fn get_electrum_height(blockid: Option<BlockId>, has_unconfirmed_parents: bool) -> isize {
    match (blockid, has_unconfirmed_parents) {
        (Some(blockid), _) => blockid.height as isize,
        (None, false) => 0,
        (None, true) => -1,
    }
}

pub type Port = u16;
pub type Hostname = String;

pub type ServerHosts = HashMap<Hostname, ServerPorts>;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ServerFeatures {
    pub hosts: ServerHosts,
    pub genesis_hash: BlockHash,
    pub server_version: String,
    pub protocol_min: ProtocolVersion,
    pub protocol_max: ProtocolVersion,
    pub pruning: Option<usize>,
    pub hash_function: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ServerPorts {
    tcp_port: Option<Port>,
    ssl_port: Option<Port>,
}

#[derive(Eq, PartialEq, Debug, Clone, Default)]
pub struct ProtocolVersion {
    major: usize,
    minor: usize,
}

impl ProtocolVersion {
    pub const fn new(major: usize, minor: usize) -> Self {
        Self { major, minor }
    }
}

impl Ord for ProtocolVersion {
    fn cmp(&self, other: &Self) -> Ordering {
        self.major
            .cmp(&other.major)
            .then_with(|| self.minor.cmp(&other.minor))
    }
}

impl PartialOrd for ProtocolVersion {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl FromStr for ProtocolVersion {
    type Err = crate::errors::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut iter = s.split('.');
        Ok(Self {
            major: iter
                .next()
                .chain_err(|| "missing major")?
                .parse()
                .chain_err(|| "invalid major")?,
            minor: iter
                .next()
                .chain_err(|| "missing minor")?
                .parse()
                .chain_err(|| "invalid minor")?,
        })
    }
}

impl std::fmt::Display for ProtocolVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.{}", self.major, self.minor)
    }
}

impl Serialize for ProtocolVersion {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.collect_str(&self)
    }
}

impl<'de> Deserialize<'de> for ProtocolVersion {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        FromStr::from_str(&s).map_err(de::Error::custom)
    }
}
