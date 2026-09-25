//! Thin electrs REST client. Paths are joined onto `splora_api::api_root`.
//!
//! Broadcast is `POST /tx`. Test transactions are `POST /txs/test`.
//! Each address lookup is its own `GET /address/:script`.

use std::fmt;

use splora_api::{Network, api_root};
use splora_frontend_shared::{
    address_path, block_path, blocks_tip_hash_path, blocks_tip_height_path, broadcast_body,
    broadcast_path, mempool_recent_path, parse_blocks_tip_hash, parse_blocks_tip_height,
    test_txs_body, test_txs_path, tx_path,
};

/// One indexer call the client built. `path` is the suffix after the network root.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientRequest {
    pub method: &'static str,
    pub path: String,
    pub url: String,
    pub content_type: &'static str,
    pub body: String,
}

/// HTTP GET used by the explorer. Tests supply a fake. The binary uses ureq.
pub trait ExplorerGet {
    fn get_text(&self, url: &str) -> Result<String, ClientError>;

    /// GET the request the client already built. The default checks the verb, then `get_text`.
    fn get_request(&self, request: &ClientRequest) -> Result<String, ClientError> {
        if request.method != "GET" {
            return Err(ClientError::BadPath);
        }
        self.get_text(&request.url)
    }
}

/// HTTP POST used by broadcast and test transactions.
pub trait ExplorerPost {
    fn post_text(&self, request: &ClientRequest) -> Result<String, ClientError>;
}

/// Why a REST read failed. The message does not include a nostr secret.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ClientError {
    Http(String),
    BadPath,
}

impl fmt::Display for ClientError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Http(message) => write!(f, "http: {message}"),
            Self::BadPath => write!(f, "bad path"),
        }
    }
}

impl std::error::Error for ClientError {}

/// Explorer bound to one network root.
pub struct ExplorerClient<G> {
    network: Network,
    get: G,
}

impl<G: ExplorerGet> ExplorerClient<G> {
    pub fn new(network: Network, get: G) -> Self {
        Self { network, get }
    }

    pub fn network(&self) -> Network {
        self.network
    }

    pub fn set_network(&mut self, network: Network) {
        self.network = network;
    }

    pub fn root(&self) -> String {
        api_root(self.network)
    }

    pub fn tip_url(&self) -> String {
        self.url(blocks_tip_height_path())
    }

    pub fn tip_hash_url(&self) -> String {
        self.url(blocks_tip_hash_path())
    }

    pub fn tip_height(&self) -> Result<String, ClientError> {
        let body = self.get.get_text(&self.tip_url())?;
        parse_blocks_tip_height(&body).map_err(|err| ClientError::Http(err.to_string()))
    }

    pub fn tip_hash(&self) -> Result<String, ClientError> {
        let body = self.get.get_text(&self.tip_hash_url())?;
        parse_blocks_tip_hash(&body).map_err(|err| ClientError::Http(err.to_string()))
    }

    pub fn block_url(&self, hash_or_height: &str) -> Result<String, ClientError> {
        Ok(self.url(&block_path(segment(hash_or_height)?)))
    }

    pub fn block(&self, hash_or_height: &str) -> Result<String, ClientError> {
        self.get.get_text(&self.block_url(hash_or_height)?)
    }

    pub fn tx_url(&self, txid: &str) -> Result<String, ClientError> {
        Ok(self.url(&tx_path(segment(txid)?)))
    }

    pub fn tx(&self, txid: &str) -> Result<String, ClientError> {
        self.get.get_text(&self.tx_url(txid)?)
    }

    pub fn address_request(&self, address: &str) -> Result<ClientRequest, ClientError> {
        let path = address_path(segment(address)?);
        Ok(ClientRequest {
            method: "GET",
            path: path.clone(),
            url: self.url(&path),
            content_type: "",
            body: String::new(),
        })
    }

    pub fn address_url(&self, address: &str) -> Result<String, ClientError> {
        Ok(self.address_request(address)?.url)
    }

    /// One `GET /address/:script`. This is not a batch call.
    pub fn fetch_address(&self, address: &str) -> Result<(ClientRequest, String), ClientError> {
        let request = self.address_request(address)?;
        if request.method != "GET" {
            return Err(ClientError::BadPath);
        }
        let body = self.get.get_request(&request)?;
        Ok((request, body))
    }

    pub fn address(&self, address: &str) -> Result<String, ClientError> {
        Ok(self.fetch_address(address)?.1)
    }

    /// `POST /tx` with the raw transaction hex as `text/plain`. Not a package submit.
    pub fn broadcast_request(&self, raw_tx: &str) -> Result<ClientRequest, ClientError> {
        let raw_tx = broadcast_body(raw_tx);
        raw_hex(&raw_tx)?;
        let path = broadcast_path();
        Ok(ClientRequest {
            method: "POST",
            path: path.to_string(),
            url: self.url(path),
            content_type: "text/plain",
            body: raw_tx,
        })
    }

    /// `POST /txs/test` with a JSON array of raw transaction hex. At most 25, matching the indexer.
    pub fn test_transactions_request(
        &self,
        raw_txs: &[String],
    ) -> Result<ClientRequest, ClientError> {
        if raw_txs.is_empty() || raw_txs.len() > 25 {
            return Err(ClientError::BadPath);
        }
        let mut hexes = Vec::with_capacity(raw_txs.len());
        for raw_tx in raw_txs {
            hexes.push(raw_hex(raw_tx)?.to_string());
        }
        let hex_refs: Vec<&str> = hexes.iter().map(String::as_str).collect();
        let body = test_txs_body(&hex_refs);
        let path = test_txs_path();
        Ok(ClientRequest {
            method: "POST",
            path: path.to_string(),
            url: self.url(path),
            content_type: "application/json",
            body,
        })
    }

    /// GET an indexer path such as `/blocks` or `/block/:hash/txs`.
    /// `path` is the suffix after the network root, not a `/api/v1` URL.
    pub fn get_path(&self, path: &str) -> Result<(ClientRequest, String), ClientError> {
        indexer_path(path)?;
        let request = ClientRequest {
            method: "GET",
            path: path.to_string(),
            url: self.url(path),
            content_type: "",
            body: String::new(),
        };
        let body = self.get.get_request(&request)?;
        Ok((request, body))
    }

    pub fn mempool_url(&self) -> String {
        self.url(mempool_recent_path())
    }

    pub fn mempool_recent(&self) -> Result<String, ClientError> {
        self.get.get_text(&self.mempool_url())
    }

    fn url(&self, path: &str) -> String {
        format!("{}{path}", self.root())
    }
}

impl<G: ExplorerGet + ExplorerPost> ExplorerClient<G> {
    pub fn broadcast(&self, raw_tx: &str) -> Result<(ClientRequest, String), ClientError> {
        let request = self.broadcast_request(raw_tx)?;
        let body = self.get.post_text(&request)?;
        Ok((request, body))
    }

    pub fn test_transactions(
        &self,
        raw_txs: &[String],
    ) -> Result<(ClientRequest, String), ClientError> {
        let request = self.test_transactions_request(raw_txs)?;
        let body = self.get.post_text(&request)?;
        Ok((request, body))
    }
}

fn indexer_path(path: &str) -> Result<(), ClientError> {
    if !path.starts_with('/') || path.contains("//") {
        return Err(ClientError::BadPath);
    }
    for part in path.split('/').skip(1) {
        if part.is_empty()
            || part.contains('?')
            || part.contains('#')
            || part.contains(' ')
            || part.contains('%')
        {
            return Err(ClientError::BadPath);
        }
    }
    Ok(())
}

fn segment(value: &str) -> Result<&str, ClientError> {
    if value.is_empty()
        || value.contains('/')
        || value.contains(' ')
        || value.contains('?')
        || value.contains('#')
    {
        return Err(ClientError::BadPath);
    }
    Ok(value)
}

fn raw_hex(value: &str) -> Result<&str, ClientError> {
    if value.is_empty() || !value.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Err(ClientError::BadPath);
    }
    Ok(value)
}

/// ureq-backed GET for the desktop binary.
#[derive(Debug, Default, Clone, Copy)]
pub struct UreqExplorer;

impl ExplorerGet for UreqExplorer {
    fn get_text(&self, url: &str) -> Result<String, ClientError> {
        let response = ureq::get(url)
            .call()
            .map_err(|err| ClientError::Http(err.to_string()))?;
        response
            .into_string()
            .map_err(|err| ClientError::Http(err.to_string()))
    }
}

impl ExplorerPost for UreqExplorer {
    fn post_text(&self, request: &ClientRequest) -> Result<String, ClientError> {
        if request.method != "POST" {
            return Err(ClientError::BadPath);
        }
        let response = ureq::post(&request.url)
            .set("Content-Type", request.content_type)
            .send_string(&request.body)
            .map_err(|err| ClientError::Http(err.to_string()))?;
        response
            .into_string()
            .map_err(|err| ClientError::Http(err.to_string()))
    }
}

pub fn ureq_client(network: Network) -> ExplorerClient<UreqExplorer> {
    ExplorerClient::new(network, UreqExplorer)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;

    struct MapExplorer {
        map: BTreeMap<String, String>,
    }

    impl ExplorerGet for MapExplorer {
        fn get_text(&self, url: &str) -> Result<String, ClientError> {
            self.map
                .get(url)
                .cloned()
                .ok_or_else(|| ClientError::Http(format!("missing {url}")))
        }
    }

    #[test]
    fn fake_client_uses_mainnet_and_testnet4_tip_urls() {
        let mainnet = "https://splora.surmount.systems/api/blocks/tip/height";
        let testnet4 = "https://splora.surmount.systems/testnet4/api/blocks/tip/height";
        let main_hash = "https://splora.surmount.systems/api/blocks/tip/hash";
        let testnet_hash = "https://splora.surmount.systems/testnet4/api/blocks/tip/hash";
        let map = BTreeMap::from([
            (mainnet.to_string(), "100".to_string()),
            (testnet4.to_string(), "4".to_string()),
            (main_hash.to_string(), " abc\n".to_string()),
            (testnet_hash.to_string(), "def".to_string()),
        ]);
        let main_client = ExplorerClient::new(Network::Mainnet, MapExplorer { map: map.clone() });
        assert_eq!(main_client.tip_url(), mainnet);
        assert_eq!(main_client.tip_hash_url(), main_hash);
        assert_eq!(main_client.root(), "https://splora.surmount.systems/api");
        assert_eq!(main_client.tip_height().expect("mainnet"), "100");
        assert_eq!(main_client.tip_hash().expect("mainnet hash"), "abc");

        let mut testnet_client = ExplorerClient::new(Network::Mainnet, MapExplorer { map });
        testnet_client.set_network(Network::Testnet4);
        assert_eq!(
            testnet_client.root(),
            "https://splora.surmount.systems/testnet4/api"
        );
        assert_eq!(testnet_client.tip_url(), testnet4);
        assert_eq!(testnet_client.tip_hash_url(), testnet_hash);
        assert_eq!(testnet_client.tip_height().expect("testnet4"), "4");
        assert_eq!(testnet_client.tip_hash().expect("testnet4 hash"), "def");
        assert_eq!(
            testnet_client.block_url("abc").expect("block"),
            "https://splora.surmount.systems/testnet4/api/block/abc"
        );
        assert_eq!(
            testnet_client.tx_url("def").expect("tx"),
            "https://splora.surmount.systems/testnet4/api/tx/def"
        );
        assert_eq!(
            testnet_client.address_url("addr").expect("address"),
            "https://splora.surmount.systems/testnet4/api/address/addr"
        );
        assert_eq!(
            testnet_client.mempool_url(),
            "https://splora.surmount.systems/testnet4/api/mempool/recent"
        );
    }

    const BANNED_URL_PARTS: &[&str] = &[
        "/v1/mining",
        "/historical-price",
        "/lightning",
        "/accelerator",
        "/api/v1/txs/package",
        "/txs/package",
    ];

    fn assert_url_allowed(url: &str) {
        for banned in BANNED_URL_PARTS {
            assert!(!url.contains(banned), "{url} contains {banned}");
        }
    }

    #[derive(Clone, Default)]
    struct Rec {
        gets: std::rc::Rc<std::cell::RefCell<Vec<ClientRequest>>>,
        posts: std::rc::Rc<std::cell::RefCell<Vec<ClientRequest>>>,
    }

    impl ExplorerGet for Rec {
        fn get_text(&self, url: &str) -> Result<String, ClientError> {
            Err(ClientError::Http(format!("unexpected get {url}")))
        }

        fn get_request(&self, request: &ClientRequest) -> Result<String, ClientError> {
            self.gets.borrow_mut().push(request.clone());
            Ok(format!("address-body:{}", request.path))
        }
    }

    impl ExplorerPost for Rec {
        fn post_text(&self, request: &ClientRequest) -> Result<String, ClientError> {
            self.posts.borrow_mut().push(request.clone());
            Ok(format!("posted:{}", request.path))
        }
    }

    #[test]
    fn broadcast_is_post_tx_and_not_a_package() {
        let rec = Rec::default();
        let client = ExplorerClient::new(Network::Mainnet, rec.clone());
        let built = client.broadcast_request("abcd").expect("request");
        assert_eq!(built.method, "POST");
        assert_eq!(built.path, "/tx");
        assert_eq!(built.content_type, "text/plain");
        assert_eq!(built.body, "abcd");
        assert_eq!(built.url, "https://splora.surmount.systems/api/tx");
        assert_url_allowed(&built.url);
        let (sent, response) = client.broadcast("abcd").expect("post");
        assert_eq!(sent, built);
        assert_eq!(response, "posted:/tx");
        let posts = rec.posts.borrow();
        assert_eq!(posts.len(), 1);
        assert_eq!(posts[0].method, "POST");
        assert_eq!(posts[0].path, "/tx");
        assert!(posts[0].url.ends_with("/tx"));
        assert!(!posts[0].url.ends_with("/txs/test"));
        assert!(rec.gets.borrow().is_empty());
    }

    #[test]
    fn test_transactions_are_post_txs_test() {
        let rec = Rec::default();
        let client = ExplorerClient::new(Network::Testnet4, rec.clone());
        let raw = vec!["abcd".to_string(), "ef01".to_string()];
        let built = client.test_transactions_request(&raw).expect("request");
        assert_eq!(built.method, "POST");
        assert_eq!(built.path, "/txs/test");
        assert_eq!(built.content_type, "application/json");
        assert_eq!(built.body, r#"["abcd","ef01"]"#);
        assert_eq!(
            built.url,
            "https://splora.surmount.systems/testnet4/api/txs/test"
        );
        assert_url_allowed(&built.url);
        let (sent, response) = client.test_transactions(&raw).expect("post");
        assert_eq!(sent, built);
        assert_eq!(response, "posted:/txs/test");
        let posts = rec.posts.borrow();
        assert_eq!(posts.len(), 1);
        assert_eq!(posts[0].method, "POST");
        assert_eq!(posts[0].path, "/txs/test");
        assert!(posts[0].url.ends_with("/txs/test"));
        assert!(rec.gets.borrow().is_empty());
        assert!(client.test_transactions_request(&Vec::new()).is_err());
        let too_many = vec!["aa".to_string(); 26];
        assert!(client.test_transactions_request(&too_many).is_err());
    }

    #[test]
    fn multi_address_issues_one_get_per_address() {
        let rec = Rec::default();
        let client = ExplorerClient::new(Network::Mainnet, rec.clone());
        let entered = ["addrA", "addrB", "addrC"];
        for address in entered {
            let (request, body) = client.fetch_address(address).expect("address");
            assert_eq!(request.method, "GET");
            assert_eq!(request.path, format!("/address/{address}"));
            assert_eq!(
                request.url,
                format!("https://splora.surmount.systems/api/address/{address}")
            );
            assert_eq!(body, format!("address-body:/address/{address}"));
            assert_url_allowed(&request.url);
        }
        let gets = rec.gets.borrow();
        assert_eq!(gets.len(), entered.len());
        assert!(rec.posts.borrow().is_empty());
        for (request, address) in gets.iter().zip(entered) {
            assert_eq!(request.method, "GET");
            assert_eq!(request.path, format!("/address/{address}"));
            assert!(!request.path.contains(','));
        }
    }

    #[test]
    fn built_urls_omit_mining_price_lightning_accelerator_and_package() {
        let client = ExplorerClient::new(
            Network::Liquid,
            MapExplorer {
                map: BTreeMap::new(),
            },
        );
        let mut urls = vec![
            client.root(),
            client.tip_url(),
            client.tip_hash_url(),
            client.block_url("abc").expect("block"),
            client.tx_url("def").expect("tx"),
            client.address_url("addr").expect("address"),
            client.mempool_url(),
            client.broadcast_request("abcd").expect("broadcast").url,
            client
                .test_transactions_request(&["abcd".to_string()])
                .expect("test")
                .url,
        ];
        for address in ["one", "two"] {
            urls.push(client.address_request(address).expect("address").url);
        }
        assert!(urls.len() >= 8);
        for url in urls {
            assert_url_allowed(&url);
            assert!(url.starts_with("https://splora.surmount.systems/liquid/api"));
        }
    }
}
