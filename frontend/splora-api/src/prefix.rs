/// Public splora origin. Network prefixes are path-only.
pub const BASE_URL: &str = "https://splora.surmount.systems";

/// Signet is an HTTP redirect, not an indexer backend.
pub const SIGNET_PATH: &str = "/signet";

/// Signet answers 307. It is not proxied to an indexer.
pub const SIGNET_STATUS: u16 = 307;

/// Networks that have an indexer prefix. Signet is intentionally absent.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Network {
    Mainnet,
    Testnet3,
    Testnet4,
    Mutinynet,
    Liquid,
}

impl Network {
    pub const ALL: [Network; 5] = [
        Network::Mainnet,
        Network::Testnet3,
        Network::Testnet4,
        Network::Mutinynet,
        Network::Liquid,
    ];

    pub fn api_prefix(self) -> &'static str {
        match self {
            Network::Mainnet => "/api",
            Network::Testnet3 => "/testnet/api",
            Network::Testnet4 => "/testnet4/api",
            Network::Mutinynet => "/mutinynet/api",
            Network::Liquid => "/liquid/api",
        }
    }
}

pub fn api_prefix(network: Network) -> &'static str {
    network.api_prefix()
}

pub fn api_root(network: Network) -> String {
    format!("{BASE_URL}{}", network.api_prefix())
}

pub fn signet_is_backend() -> bool {
    false
}

/// True only for the five indexer networks, by name or by public prefix.
/// `signet`, `Signet`, `/signet`, and `/signet/api` are false.
pub fn is_backend_network(name: &str) -> bool {
    matches!(
        name,
        "mainnet"
            | "Mainnet"
            | "/api"
            | "testnet"
            | "testnet3"
            | "Testnet3"
            | "/testnet/api"
            | "testnet4"
            | "Testnet4"
            | "/testnet4/api"
            | "mutinynet"
            | "Mutinynet"
            | "/mutinynet/api"
            | "liquid"
            | "Liquid"
            | "/liquid/api"
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn prefixes_match_the_table() {
        assert_eq!(api_prefix(Network::Mainnet), "/api");
        assert_eq!(api_prefix(Network::Testnet3), "/testnet/api");
        assert_eq!(api_prefix(Network::Testnet4), "/testnet4/api");
        assert_eq!(api_prefix(Network::Mutinynet), "/mutinynet/api");
        assert_eq!(api_prefix(Network::Liquid), "/liquid/api");
    }

    #[test]
    fn api_root_uses_the_public_origin_and_prefix() {
        for network in Network::ALL {
            let root = api_root(network);
            assert!(root.starts_with(BASE_URL));
            assert!(root.ends_with(network.api_prefix()));
            assert_eq!(root, format!("{BASE_URL}{}", network.api_prefix()));
        }
    }

    #[test]
    fn signet_is_not_a_backend() {
        assert!(!signet_is_backend());
        assert_eq!(SIGNET_STATUS, 307);
        assert_eq!(SIGNET_PATH, "/signet");
        assert!(!is_backend_network("signet"));
        assert!(!is_backend_network("Signet"));
        assert!(!is_backend_network("/signet"));
        assert!(!is_backend_network("/signet/api"));
    }

    #[test]
    fn the_five_networks_are_backends() {
        assert!(is_backend_network("mainnet"));
        assert!(is_backend_network("Mainnet"));
        assert!(is_backend_network("testnet"));
        assert!(is_backend_network("testnet3"));
        assert!(is_backend_network("Testnet3"));
        assert!(is_backend_network("testnet4"));
        assert!(is_backend_network("Testnet4"));
        assert!(is_backend_network("mutinynet"));
        assert!(is_backend_network("Mutinynet"));
        assert!(is_backend_network("liquid"));
        assert!(is_backend_network("Liquid"));
        assert!(is_backend_network("/api"));
        assert!(is_backend_network("/testnet/api"));
        assert!(is_backend_network("/testnet4/api"));
        assert!(is_backend_network("/mutinynet/api"));
        assert!(is_backend_network("/liquid/api"));
    }
}
