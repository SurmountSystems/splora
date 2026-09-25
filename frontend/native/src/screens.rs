//! View models for the desktop explorer. No GPUI types.

use splora_api::{Network, api_prefix, api_root};
use splora_frontend_shared::{
    Block, Tx, address_path, address_txs_path, address_utxo_path, block_height_path, block_path,
    block_txids_path, block_txs_path, block_txs_start_index_path, blocks_path,
    blocks_start_height_path, broadcast_path, fee_estimates_path, mempool_path,
    mempool_recent_path, parse_address, parse_address_txs, parse_address_utxos, parse_block,
    parse_block_height, parse_block_txids, parse_block_txs, parse_blocks, parse_broadcast,
    parse_fee_estimates, parse_mempool, parse_mempool_recent, parse_test_txs, parse_tx,
    test_txs_path, tx_path,
};

use crate::api::{ClientError, ClientRequest, ExplorerClient, ExplorerGet, ExplorerPost};
use crate::popup::NpubPopupModel;

/// Which screen the shell is showing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Screen {
    Popup,
    Dashboard,
    Block,
    Blocks,
    Tx,
    Address,
    Mempool,
    MultiAddress,
    Broadcast,
    TestTransactions,
    Terms,
    Privacy,
    Trademark,
    Docs,
    Faq,
    ApiRest,
    ApiWebsocket,
}

impl Screen {
    /// Continue leaves the npub popup for the dashboard shell.
    pub fn after_continue(self) -> Self {
        match self {
            Self::Popup => Self::Dashboard,
            other => other,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Popup => "popup",
            Self::Dashboard => "dashboard",
            Self::Block => "block",
            Self::Blocks => "blocks",
            Self::Tx => "tx",
            Self::Address => "address",
            Self::Mempool => "mempool",
            Self::MultiAddress => "addresses",
            Self::Broadcast => "broadcast",
            Self::TestTransactions => "test transactions",
            Self::Terms => "terms",
            Self::Privacy => "privacy",
            Self::Trademark => "trademark",
            Self::Docs => "docs",
            Self::Faq => "faq",
            Self::ApiRest => "api/rest",
            Self::ApiWebsocket => "api/websocket",
        }
    }

    pub fn element_id(self) -> &'static str {
        match self {
            Self::Popup => "popup",
            Self::Dashboard => "dashboard",
            Self::Block => "block",
            Self::Blocks => "blocks",
            Self::Tx => "tx",
            Self::Address => "address",
            Self::Mempool => "mempool",
            Self::MultiAddress => "addresses",
            Self::Broadcast => "broadcast",
            Self::TestTransactions => "test-transactions",
            Self::Terms => "terms",
            Self::Privacy => "privacy",
            Self::Trademark => "trademark",
            Self::Docs => "docs",
            Self::Faq => "faq",
            Self::ApiRest => "api-rest",
            Self::ApiWebsocket => "api-websocket",
        }
    }

    pub fn from_link(link: &str) -> Option<Self> {
        match link {
            "block" => Some(Self::Block),
            "blocks" => Some(Self::Blocks),
            "tx" => Some(Self::Tx),
            "address" => Some(Self::Address),
            "mempool" => Some(Self::Mempool),
            "addresses" => Some(Self::MultiAddress),
            "broadcast" => Some(Self::Broadcast),
            "test transactions" => Some(Self::TestTransactions),
            "terms" => Some(Self::Terms),
            "privacy" => Some(Self::Privacy),
            "trademark" => Some(Self::Trademark),
            "docs" => Some(Self::Docs),
            _ => Self::from_doc_path(link),
        }
    }

    /// Known documentation paths only. Unknown `api/:type` values are not screens.
    fn from_doc_path(link: &str) -> Option<Self> {
        if resolve_doc_route(link).status != 200 {
            return None;
        }
        match link {
            "faq" => Some(Self::Faq),
            "api/rest" => Some(Self::ApiRest),
            "api/websocket" => Some(Self::ApiWebsocket),
            _ => None,
        }
    }
}

/// The five indexer networks. Signet is not an entry.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NetworkSwitcher {
    selected: Network,
}

impl NetworkSwitcher {
    pub fn new(selected: Network) -> Self {
        Self { selected }
    }

    pub fn selected(self) -> Network {
        self.selected
    }

    pub fn select(&mut self, network: Network) {
        self.selected = network;
    }

    pub fn networks() -> &'static [Network] {
        &Network::ALL
    }

    pub fn labels(self) -> Vec<&'static str> {
        Network::ALL.iter().copied().map(network_label).collect()
    }

    pub fn prefixes(self) -> Vec<&'static str> {
        Network::ALL.iter().copied().map(api_prefix).collect()
    }

    pub fn api_root(self) -> String {
        api_root(self.selected)
    }

    pub fn summary(self) -> String {
        let labels = self.labels().join(", ");
        format!("network: {} ({labels})", network_label(self.selected))
    }
}

pub fn network_label(network: Network) -> &'static str {
    match network {
        Network::Mainnet => "mainnet",
        Network::Testnet3 => "testnet3",
        Network::Testnet4 => "testnet4",
        Network::Mutinynet => "mutinynet",
        Network::Liquid => "liquid",
    }
}

/// Dashboard shell. Links name the other screens. Data is electrs blocks and recent mempool.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Dashboard {
    pub tip_height: String,
    pub tip_hash: String,
    lines: String,
}

impl Dashboard {
    pub fn new() -> Self {
        Self {
            tip_height: String::new(),
            tip_hash: String::new(),
            lines: String::new(),
        }
    }

    pub fn links(&self) -> [&'static str; 15] {
        [
            "block",
            "tx",
            "address",
            "mempool",
            "blocks",
            "addresses",
            "broadcast",
            "test transactions",
            "terms",
            "privacy",
            "trademark",
            "docs",
            "faq",
            "api/rest",
            "api/websocket",
        ]
    }

    pub fn tip_line(&self) -> String {
        match (self.tip_height.as_str(), self.tip_hash.as_str()) {
            ("", "") => "tip height:".to_string(),
            (height, "") => format!("tip height: {height}"),
            ("", hash) => format!("tip hash: {hash}"),
            (height, hash) => format!("tip height: {height} hash: {hash}"),
        }
    }

    /// GET `/blocks/tip/height` and GET `/blocks/tip/hash`.
    pub fn load_tip<G: ExplorerGet>(
        &mut self,
        client: &ExplorerClient<G>,
    ) -> Result<(), ClientError> {
        self.tip_height = client.tip_height()?;
        self.tip_hash = client.tip_hash()?;
        Ok(())
    }

    /// GET `/blocks`, `/mempool/recent`, the tip, `/fee-estimates`, and `/mempool`.
    /// The socket does not push blocks. The tip comes from the shared tip paths.
    pub fn load<G: ExplorerGet>(&mut self, client: &ExplorerClient<G>) -> Result<(), ClientError> {
        let blocks_route = blocks_path();
        let recent_route = mempool_recent_path();
        let (blocks, blocks_body) = client.get_path(blocks_route)?;
        let (recent, recent_body) = client.get_path(recent_route)?;
        if blocks.method != "GET" || blocks.path != blocks_route {
            return Err(ClientError::BadPath);
        }
        if recent.method != "GET" || recent.path != recent_route {
            return Err(ClientError::BadPath);
        }
        self.load_tip(client)?;
        let fees_route = fee_estimates_path();
        let (fees, fees_body) = client.get_path(fees_route)?;
        if fees.method != "GET" || fees.path != fees_route {
            return Err(ClientError::BadPath);
        }
        let summary_route = mempool_path();
        let (summary, summary_body) = client.get_path(summary_route)?;
        if summary.method != "GET" || summary.path != summary_route {
            return Err(ClientError::BadPath);
        }
        let block_lines = render_block_list(&blocks_body)?;
        let recent_lines = render_recent(&recent_body)?;
        let fee_lines = render_fee_estimates(&fees_body)?;
        let summary_lines = render_mempool_summary(&summary_body)?;
        self.lines = format!("{block_lines}\n{recent_lines}\n{fee_lines}\n{summary_lines}");
        Ok(())
    }

    pub fn rendered(&self) -> String {
        if self.lines.is_empty() {
            self.tip_line()
        } else {
            format!("{}\n{}", self.tip_line(), self.lines)
        }
    }
}

impl Default for Dashboard {
    fn default() -> Self {
        Self::new()
    }
}

/// Block screen. Query is a hash or a height.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlockScreen {
    pub query: String,
    /// Empty means the block screen does not call the paged txs route.
    pub start_index: String,
    pub body: String,
}

impl BlockScreen {
    pub fn new() -> Self {
        Self {
            query: String::new(),
            start_index: String::new(),
            body: String::new(),
        }
    }

    pub fn summary(&self) -> String {
        format!("block: {} {}", self.query, self.body)
    }

    /// Hash: GET `/block/:hash`, GET `/block/:hash/txs`, and GET `/block/:hash/txids`.
    /// A numeric start index that is a multiple of 25 also GETs `/block/:hash/txs/:start_index`.
    /// All-digit id: GET `/block-height/:height`, then those hash routes.
    pub fn load<G: ExplorerGet>(&mut self, client: &ExplorerClient<G>) -> Result<(), ClientError> {
        let mut lines = Vec::new();
        let hash = if is_numeric_id(&self.query) {
            let height: u64 = self.query.parse().map_err(|_| ClientError::BadPath)?;
            let path = block_height_path(height);
            let (request, body) = client.get_path(&path)?;
            if request.method != "GET" || request.path != path {
                return Err(ClientError::BadPath);
            }
            let hash = parse_block_height(&body).map_err(http_parse)?;
            if !hash.bytes().all(|byte| byte.is_ascii_hexdigit()) {
                return Err(ClientError::Http("block height hash".into()));
            }
            lines.push(format!("block-height {} hash {hash}", self.query));
            hash
        } else if self.query.is_empty() || self.query.contains('/') {
            return Err(ClientError::BadPath);
        } else {
            self.query.clone()
        };
        let block_route = block_path(&hash);
        let (block_request, block_body) = client.get_path(&block_route)?;
        if block_request.method != "GET" || block_request.path != block_route {
            return Err(ClientError::BadPath);
        }
        lines.push(render_block_value(&block_body)?);
        let txs_route = block_txs_path(&hash);
        let (txs_request, txs_body) = client.get_path(&txs_route)?;
        if txs_request.method != "GET" || txs_request.path != txs_route {
            return Err(ClientError::BadPath);
        }
        let txs = render_tx_list(&txs_body)?;
        if !txs.is_empty() {
            lines.push(txs);
        }
        let txids_route = block_txids_path(&hash);
        let (txids_request, txids_body) = client.get_path(&txids_route)?;
        if txids_request.method != "GET" || txids_request.path != txids_route {
            return Err(ClientError::BadPath);
        }
        let txids = render_txids(&txids_body)?;
        if !txids.is_empty() {
            lines.push(txids);
        }
        if !self.start_index.is_empty() {
            if !is_numeric_id(&self.start_index) {
                return Err(ClientError::BadPath);
            }
            let start: u64 = self.start_index.parse().map_err(|_| ClientError::BadPath)?;
            if start % 25 != 0 {
                return Err(ClientError::BadPath);
            }
            let page_route = block_txs_start_index_path(&hash, start);
            let (page_request, page_body) = client.get_path(&page_route)?;
            if page_request.method != "GET" || page_request.path != page_route {
                return Err(ClientError::BadPath);
            }
            let page = render_tx_list(&page_body)?;
            if !page.is_empty() {
                lines.push(page);
            }
        }
        self.body = lines.join("\n");
        Ok(())
    }

    pub fn rendered(&self) -> String {
        if self.body.is_empty() {
            self.summary()
        } else {
            self.body.clone()
        }
    }
}

impl Default for BlockScreen {
    fn default() -> Self {
        Self::new()
    }
}

/// Blocks list. Electrs `GET /blocks`, or `GET /blocks/:start_height` when a height is set.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct BlocksScreen {
    pub start_height: String,
    pub body: String,
}

impl BlocksScreen {
    pub fn new() -> Self {
        Self::default()
    }

    /// GET `/blocks`, or GET `/blocks/:start_height` when `start_height` is set.
    /// This is not GET `/api/v1/blocks`.
    pub fn load<G: ExplorerGet>(&mut self, client: &ExplorerClient<G>) -> Result<(), ClientError> {
        let path = if self.start_height.is_empty() {
            blocks_path().to_string()
        } else if is_numeric_id(&self.start_height) {
            let height: u64 = self
                .start_height
                .parse()
                .map_err(|_| ClientError::BadPath)?;
            blocks_start_height_path(height)
        } else {
            return Err(ClientError::BadPath);
        };
        let (request, body) = client.get_path(&path)?;
        if request.method != "GET" || request.path != path || request.path.contains("v1") {
            return Err(ClientError::BadPath);
        }
        self.body = render_block_list(&body)?;
        Ok(())
    }

    pub fn rendered(&self) -> String {
        self.body.clone()
    }
}

/// Transaction screen.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TxScreen {
    pub txid: String,
    pub body: String,
}

impl TxScreen {
    pub fn new() -> Self {
        Self {
            txid: String::new(),
            body: String::new(),
        }
    }

    pub fn summary(&self) -> String {
        format!("tx: {} {}", self.txid, self.body)
    }

    pub fn load<G: ExplorerGet>(&mut self, client: &ExplorerClient<G>) -> Result<(), ClientError> {
        if self.txid.is_empty() || self.txid.contains('/') {
            return Err(ClientError::BadPath);
        }
        let path = tx_path(&self.txid);
        let (request, body) = client.get_path(&path)?;
        if request.method != "GET" || request.path != path {
            return Err(ClientError::BadPath);
        }
        self.body = render_one_tx(&body)?;
        Ok(())
    }

    pub fn rendered(&self) -> String {
        if self.body.is_empty() {
            self.summary()
        } else {
            self.body.clone()
        }
    }
}

impl Default for TxScreen {
    fn default() -> Self {
        Self::new()
    }
}

/// Address screen.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AddressScreen {
    pub address: String,
    pub body: String,
}

impl AddressScreen {
    pub fn new() -> Self {
        Self {
            address: String::new(),
            body: String::new(),
        }
    }

    pub fn summary(&self) -> String {
        format!("address: {} {}", self.address, self.body)
    }

    pub fn load<G: ExplorerGet>(&mut self, client: &ExplorerClient<G>) -> Result<(), ClientError> {
        if self.address.is_empty() || self.address.contains('/') {
            return Err(ClientError::BadPath);
        }
        let script = &self.address;
        let stats_path = address_path(script);
        let txs_path = address_txs_path(script);
        let utxo_path = address_utxo_path(script);
        let (stats_request, stats_body) = client.get_path(&stats_path)?;
        let (txs_request, txs_body) = client.get_path(&txs_path)?;
        let (utxo_request, utxo_body) = client.get_path(&utxo_path)?;
        if stats_request.method != "GET"
            || txs_request.method != "GET"
            || utxo_request.method != "GET"
            || stats_request.path != stats_path
            || txs_request.path != txs_path
            || utxo_request.path != utxo_path
        {
            return Err(ClientError::BadPath);
        }
        let mut lines = vec![render_address_stats(&stats_body)?];
        let txs = render_address_txs(&txs_body)?;
        if !txs.is_empty() {
            lines.push(txs);
        }
        let utxos = render_utxos(&utxo_body)?;
        if !utxos.is_empty() {
            lines.push(utxos);
        }
        self.body = lines.join("\n");
        Ok(())
    }

    pub fn rendered(&self) -> String {
        if self.body.is_empty() {
            self.summary()
        } else {
            self.body.clone()
        }
    }
}

impl Default for AddressScreen {
    fn default() -> Self {
        Self::new()
    }
}

/// Recent mempool entries.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MempoolScreen {
    pub body: String,
}

impl MempoolScreen {
    pub fn new() -> Self {
        Self {
            body: String::new(),
        }
    }

    pub fn summary(&self) -> String {
        format!("mempool: {}", self.body)
    }

    pub fn load<G: ExplorerGet>(&mut self, client: &ExplorerClient<G>) -> Result<(), ClientError> {
        let path = mempool_recent_path();
        let (request, body) = client.get_path(path)?;
        if request.method != "GET" || request.path != path {
            return Err(ClientError::BadPath);
        }
        self.body = render_recent(&body)?;
        Ok(())
    }

    pub fn rendered(&self) -> String {
        if self.body.is_empty() {
            self.summary()
        } else {
            self.body.clone()
        }
    }
}

impl Default for MempoolScreen {
    fn default() -> Self {
        Self::new()
    }
}

/// Split a multi-address field on commas and whitespace. Empty pieces are dropped.
pub fn split_addresses(input: &str) -> Vec<&str> {
    input
        .split([',', '\n', '\r', '\t', ' '])
        .filter(|part| !part.is_empty())
        .collect()
}

/// One `GET /address/:script` per entered address. No batch endpoint.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct MultiAddressScreen {
    pub input: String,
    pub bodies: Vec<String>,
    pub requests: Vec<ClientRequest>,
    field_lines: String,
}

impl MultiAddressScreen {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn entered(&self) -> Vec<&str> {
        split_addresses(&self.input)
    }

    pub fn summary(&self) -> String {
        if self.requests.is_empty() {
            format!("addresses: {}", self.input)
        } else {
            let calls = self
                .requests
                .iter()
                .map(|request| format!("{} {}", request.method, request.path))
                .collect::<Vec<_>>()
                .join(", ");
            format!("addresses: {calls} {}", self.bodies.join(" | "))
        }
    }

    pub fn load<G: ExplorerGet>(&mut self, client: &ExplorerClient<G>) -> Result<(), ClientError> {
        let entered = self
            .entered()
            .into_iter()
            .map(str::to_string)
            .collect::<Vec<_>>();
        let mut bodies = Vec::with_capacity(entered.len());
        let mut requests = Vec::with_capacity(entered.len());
        let mut field_lines = Vec::with_capacity(entered.len());
        for address in &entered {
            let (request, body) = client.fetch_address(address)?;
            let expected = address_path(address);
            if request.method != "GET" || request.path != expected {
                return Err(ClientError::BadPath);
            }
            match render_address_stats(&body) {
                Ok(line) => field_lines.push(line),
                Err(_) => field_lines.push(body.clone()),
            }
            requests.push(request);
            bodies.push(body);
        }
        self.requests = requests;
        self.bodies = bodies;
        self.field_lines = field_lines.join("\n");
        Ok(())
    }

    pub fn rendered(&self) -> String {
        if self.field_lines.is_empty() {
            self.summary()
        } else {
            self.field_lines.clone()
        }
    }
}

/// Broadcast screen. The only submit is `POST /tx`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct BroadcastScreen {
    pub raw_tx: String,
    pub result: String,
    pub request: Option<ClientRequest>,
}

impl BroadcastScreen {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn summary(&self) -> String {
        match &self.request {
            Some(request) => format!(
                "broadcast: {} {} {}",
                request.method, request.path, self.result
            ),
            None => format!("broadcast: POST {}", broadcast_path()),
        }
    }

    pub fn submit<G: ExplorerGet + ExplorerPost>(
        &mut self,
        client: &ExplorerClient<G>,
    ) -> Result<(), ClientError> {
        let (request, result) = client.broadcast(&self.raw_tx)?;
        if request.method != "POST" || request.path != broadcast_path() {
            return Err(ClientError::BadPath);
        }
        self.request = Some(request);
        self.result = result;
        Ok(())
    }

    pub fn rendered(&self) -> String {
        if self
            .request
            .as_ref()
            .is_some_and(|request| request.method == "POST" && request.path == broadcast_path())
            && !self.result.is_empty()
        {
            match parse_broadcast(&self.result) {
                Ok(txid) => format!("broadcast txid {txid}"),
                Err(_) => self.summary(),
            }
        } else {
            self.summary()
        }
    }
}

/// Test-transactions screen. The only submit is `POST /txs/test`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct TestTransactionsScreen {
    pub input: String,
    pub result: String,
    pub request: Option<ClientRequest>,
}

impl TestTransactionsScreen {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn raw_txs(&self) -> Vec<String> {
        split_addresses(&self.input)
            .into_iter()
            .map(str::to_string)
            .collect()
    }

    pub fn summary(&self) -> String {
        match &self.request {
            Some(request) => format!(
                "test transactions: {} {} {}",
                request.method, request.path, self.result
            ),
            None => format!("test transactions: POST {}", test_txs_path()),
        }
    }

    pub fn submit<G: ExplorerGet + ExplorerPost>(
        &mut self,
        client: &ExplorerClient<G>,
    ) -> Result<(), ClientError> {
        let raw_txs = self.raw_txs();
        let (request, result) = client.test_transactions(&raw_txs)?;
        if request.method != "POST" || request.path != test_txs_path() {
            return Err(ClientError::BadPath);
        }
        self.request = Some(request);
        self.result = result;
        Ok(())
    }

    pub fn rendered(&self) -> String {
        if self
            .request
            .as_ref()
            .is_some_and(|request| request.method == "POST" && request.path == test_txs_path())
        {
            render_test_accept(&self.result).unwrap_or_else(|_| self.summary())
        } else {
            self.summary()
        }
    }
}

/// Local pages. They do not call the indexer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StaticPage {
    Terms,
    Privacy,
    Trademark,
    Docs,
}

impl StaticPage {
    pub const ALL: [StaticPage; 4] = [
        StaticPage::Terms,
        StaticPage::Privacy,
        StaticPage::Trademark,
        StaticPage::Docs,
    ];

    pub fn title(self) -> &'static str {
        match self {
            Self::Terms => "Terms",
            Self::Privacy => "Privacy",
            Self::Trademark => "Trademark",
            Self::Docs => "Docs",
        }
    }

    pub fn calls_indexer(self) -> bool {
        false
    }

    pub fn body(self) -> &'static str {
        match self {
            Self::Terms => TERMS,
            Self::Privacy => PRIVACY,
            Self::Trademark => TRADEMARK,
            Self::Docs => DOCS,
        }
    }
}

const TERMS: &str = "\
Splora is the desktop block explorer for the public indexer at https://splora.surmount.systems. \
It shows public chain data for the five networks in the network list. \
You can look up a block, a transaction, an address, or the recent mempool. \
You can paste a raw transaction and broadcast it. \
You are responsible for any transaction you submit. \
Indexer data can be delayed or unavailable. \
The software is provided as-is, without a warranty of fitness or uninterrupted service.";

const PRIVACY: &str = "\
The first screen shows your public npub and has no private-key field. \
The nostr secret is not typed into that popup. \
Address, block, transaction, and mempool views send only the lookup you asked for to the indexer. \
A multi-address lookup sends one address request per address you enter. \
Broadcast sends the raw transaction hex you entered. \
Test transactions send the raw transaction hex list you entered. \
Terms, privacy, trademark, and docs are local text in this app. \
They are not loaded from the indexer. \
The indexer can see your network address and the requests your computer makes.";

const TRADEMARK: &str = "\
Splora names this explorer. \
Dogecoin and DOGE are names of their owners. \
The other network names in the switcher belong to their owners. \
Nothing in this app gives you a license to those marks. \
Do not use the Splora name, or the look of this shell, to imply that another product is Splora \
or is endorsed by the operator of splora.surmount.systems.";

const DOCS: &str = "\
Choose a network in the shell. The client joins that network's prefix to a public indexer path. \
Tip height is a GET of /blocks/tip/height. \
A block is a GET of /block/ and then the hash or height. \
A transaction is a GET of /tx/ and then the transaction id. \
An address is a GET of /address/ and then the script or address. \
Recent mempool entries are a GET of /mempool/recent. \
Multi-address reads each entered address with its own GET of /address/ and then that address. \
It does not use a batch endpoint. \
Broadcast is a POST of /tx. The body is the raw transaction hex, as text. \
Test transactions are a POST of /txs/test. \
The body is a JSON array of raw transaction hex, at most 25 items. \
Signet is not a backend in this shell. \
This shell does not submit a transaction package.";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StaticScreen {
    pub page: StaticPage,
}

impl StaticScreen {
    pub fn new(page: StaticPage) -> Self {
        Self { page }
    }

    /// Local copy only. This method does not take a client and does not call the indexer.
    pub fn show(self) -> &'static str {
        self.page.body()
    }

    pub fn summary(self) -> String {
        format!("{}: {}", self.page.title(), self.page.body())
    }
}

const FAQ_DOC: &str = "\
FAQ. This page is part of the app. Opening it does not read chain data. \
The first screen shows your public npub and has no private-key field. \
A block lookup uses a hash or a height. \
A transaction lookup uses the transaction id. \
An address lookup uses one address. \
Broadcast sends only the raw transaction you paste. \
If a chain screen is empty, this page still shows the same answers.";

const REST_DOC: &str = "\
REST API. This page is local text. Opening it does not read chain data. \
Tip height is a GET of /blocks/tip/height. \
A block is a GET of /block/ and then the hash or height. \
A transaction is a GET of /tx/ and then the transaction id. \
An address is a GET of /address/ and then the address. \
Recent mempool entries are a GET of /mempool/recent. \
Broadcast is a POST of /tx. The body is the raw transaction hex, as text. \
Test transactions are a POST of /txs/test. \
The body is a JSON array of raw transaction hex, at most 25 items. \
Signet is not a backend in this shell.";

const WEBSOCKET_DOC: &str = "\
Websocket API. This page is local text. Opening it does not read chain data. \
It describes a websocket document for following an address on a socket. \
The socket is not a substitute for the REST reads on the REST page. \
This page does not open a socket and does not load chain data.";

const DOC_NOT_FOUND: &str = "Not found.";

/// One documentation route result. The body is static. Status 200 is a known page.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DocPage {
    pub status: u16,
    pub body: &'static str,
}

impl DocPage {
    fn ok(body: &'static str) -> Self {
        Self { status: 200, body }
    }

    fn not_found() -> Self {
        Self {
            status: 404,
            body: DOC_NOT_FOUND,
        }
    }

    pub fn calls_indexer(self) -> bool {
        false
    }
}

fn single_api_type(path: &str) -> Option<&str> {
    let kind = path.strip_prefix("api/")?;
    if kind.is_empty() || kind.contains('/') {
        None
    } else {
        Some(kind)
    }
}

/// `api/:type`. Only `rest` and `websocket` are pages. Any other type is not found.
fn route_api_type(path: &str) -> DocPage {
    match single_api_type(path) {
        Some("rest") => DocPage::ok(REST_DOC),
        Some("websocket") => DocPage::ok(WEBSOCKET_DOC),
        Some(_) | None => DocPage::not_found(),
    }
}

/// Match a documentation path. Native links have no leading slash.
/// `faq` is the FAQ page. `api/rest` is the REST page.
/// `api/:type` also serves REST when the type is `rest`, and websocket when the type is `websocket`.
pub fn resolve_doc_route(path: &str) -> DocPage {
    match path {
        "faq" => DocPage::ok(FAQ_DOC),
        "api/rest" => DocPage::ok(REST_DOC),
        _ => route_api_type(path),
    }
}

/// Serve a documentation path. Reads the selected network only. Does not GET or POST.
pub fn open_doc_route<G: ExplorerGet + ExplorerPost>(
    path: &str,
    client: &ExplorerClient<G>,
) -> DocPage {
    let _ = client.network();
    let page = resolve_doc_route(path);
    debug_assert!(!page.calls_indexer());
    page
}

/// Static copy for the three known documentation screens.
pub fn known_doc_lines() -> String {
    ["faq", "api/rest", "api/websocket"]
        .into_iter()
        .map(|path| {
            let page = resolve_doc_route(path);
            format!("{path}: {}", page.body)
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn http_parse(err: impl std::fmt::Display) -> ClientError {
    ClientError::Http(err.to_string())
}

fn is_numeric_id(value: &str) -> bool {
    !value.is_empty() && value.bytes().all(|byte| byte.is_ascii_digit())
}

fn render_block_line(block: &Block) -> String {
    format!(
        "block hash {} height {} tx_count {} timestamp {} size {} weight {} previous_hash {}",
        block.id,
        block.height,
        block.tx_count,
        block.timestamp,
        block.size,
        block.weight,
        block.previous_hash
    )
}

fn render_block_list(body: &str) -> Result<String, ClientError> {
    let rows = parse_blocks(body).map_err(http_parse)?;
    if rows.is_empty() {
        return Ok("blocks:".to_string());
    }
    Ok(rows
        .iter()
        .map(render_block_line)
        .collect::<Vec<_>>()
        .join("\n"))
}

fn render_block_value(body: &str) -> Result<String, ClientError> {
    let block = parse_block(body).map_err(http_parse)?;
    Ok(render_block_line(&block))
}

fn render_recent(body: &str) -> Result<String, ClientError> {
    let rows = parse_mempool_recent(body).map_err(http_parse)?;
    if rows.is_empty() {
        return Ok("recent:".to_string());
    }
    Ok(rows
        .iter()
        .map(|tx| {
            format!(
                "recent txid {} fee {} vsize {} value {}",
                tx.txid, tx.fee, tx.vsize, tx.value
            )
        })
        .collect::<Vec<_>>()
        .join("\n"))
}

fn render_fee_estimates(body: &str) -> Result<String, ClientError> {
    let rows = parse_fee_estimates(body).map_err(http_parse)?;
    if rows.is_empty() {
        return Ok("fee:".to_string());
    }
    Ok(rows
        .iter()
        .map(|row| format!("fee target {} rate {}", row.target, row.rate))
        .collect::<Vec<_>>()
        .join("\n"))
}

fn render_mempool_summary(body: &str) -> Result<String, ClientError> {
    let stats = parse_mempool(body).map_err(http_parse)?;
    let mut lines = vec![format!(
        "mempool count {} vsize {} total_fee {}",
        stats.count, stats.vsize, stats.total_fee
    )];
    for (rate, vsize) in stats.fee_histogram {
        lines.push(format!("fee_histogram {rate} {vsize}"));
    }
    Ok(lines.join("\n"))
}

fn render_txids(body: &str) -> Result<String, ClientError> {
    let txids = parse_block_txids(body).map_err(http_parse)?;
    if txids.is_empty() {
        return Ok(String::new());
    }
    Ok(txids
        .iter()
        .map(|txid| format!("txid {txid}"))
        .collect::<Vec<_>>()
        .join("\n"))
}

fn render_tx_row(tx: &Tx) -> String {
    format!(
        "tx txid {} fee {} confirmed {} block_height {} size {} weight {}",
        tx.txid, tx.fee, tx.confirmed, tx.block_height, tx.size, tx.weight
    )
}

fn render_tx_list(body: &str) -> Result<String, ClientError> {
    let rows = parse_block_txs(body).map_err(http_parse)?;
    if rows.is_empty() {
        return Ok(String::new());
    }
    Ok(rows
        .iter()
        .map(render_tx_row)
        .collect::<Vec<_>>()
        .join("\n"))
}

fn render_one_tx(body: &str) -> Result<String, ClientError> {
    let tx = parse_tx(body).map_err(http_parse)?;
    Ok(render_tx_row(&tx))
}

fn render_address_stats(body: &str) -> Result<String, ClientError> {
    let stats = parse_address(body).map_err(http_parse)?;
    Ok(format!(
        "address {} chain_tx_count {} funded_sum {} mempool_tx_count {}",
        stats.address, stats.chain_tx_count, stats.funded_sum, stats.mempool_tx_count
    ))
}

fn render_address_txs(body: &str) -> Result<String, ClientError> {
    let rows = parse_address_txs(body).map_err(http_parse)?;
    Ok(rows
        .iter()
        .map(|tx| {
            format!(
                "address tx txid {} confirmed {} block_height {} fee {} size {} weight {}",
                tx.txid, tx.confirmed, tx.block_height, tx.fee, tx.size, tx.weight
            )
        })
        .collect::<Vec<_>>()
        .join("\n"))
}

fn render_utxos(body: &str) -> Result<String, ClientError> {
    let rows = parse_address_utxos(body).map_err(http_parse)?;
    Ok(rows
        .iter()
        .map(|utxo| {
            format!(
                "utxo txid {} vout {} value {} confirmed {}",
                utxo.txid, utxo.vout, utxo.value, utxo.confirmed
            )
        })
        .collect::<Vec<_>>()
        .join("\n"))
}

fn render_test_accept(body: &str) -> Result<String, ClientError> {
    let rows = parse_test_txs(body).map_err(http_parse)?;
    if rows.is_empty() {
        return Ok("test:".to_string());
    }
    Ok(rows
        .iter()
        .map(|row| {
            let mut line = format!("test txid {} allowed {}", row.txid, row.allowed);
            if !row.reject_reason.is_empty() {
                line.push_str(" reject-reason ");
                line.push_str(&row.reject_reason);
            }
            line
        })
        .collect::<Vec<_>>()
        .join("\n"))
}

/// The GPUI shell calls `activate`, `reload`, and `submit`. Tests use the same entry.
pub struct ScreenHost<G> {
    pub screen: Screen,
    pub dashboard: Dashboard,
    pub blocks: BlocksScreen,
    pub block: BlockScreen,
    pub tx: TxScreen,
    pub address: AddressScreen,
    pub mempool: MempoolScreen,
    pub multi_address: MultiAddressScreen,
    pub broadcast: BroadcastScreen,
    pub test_txs: TestTransactionsScreen,
    client: ExplorerClient<G>,
    pub notice: String,
}

impl<G> ScreenHost<G> {
    pub fn new(client: ExplorerClient<G>) -> Self {
        Self {
            screen: Screen::Popup,
            dashboard: Dashboard::new(),
            blocks: BlocksScreen::new(),
            block: BlockScreen::new(),
            tx: TxScreen::new(),
            address: AddressScreen::new(),
            mempool: MempoolScreen::new(),
            multi_address: MultiAddressScreen::new(),
            broadcast: BroadcastScreen::new(),
            test_txs: TestTransactionsScreen::new(),
            client,
            notice: String::new(),
        }
    }

    pub fn query_text(&self) -> String {
        match self.screen {
            Screen::Block => self.block.query.clone(),
            Screen::Blocks => self.blocks.start_height.clone(),
            Screen::Tx => self.tx.txid.clone(),
            Screen::Address => self.address.address.clone(),
            Screen::MultiAddress => self.multi_address.input.clone(),
            Screen::Broadcast => self.broadcast.raw_tx.clone(),
            Screen::TestTransactions => self.test_txs.input.clone(),
            _ => String::new(),
        }
    }

    pub fn set_query(&mut self, text: &str) {
        let trimmed = text.trim();
        match self.screen {
            Screen::Block => {
                let mut parts = trimmed.split_whitespace();
                self.block.query = parts.next().unwrap_or("").to_string();
                let page = parts.next().unwrap_or("");
                if page.is_empty() || !is_numeric_id(page) || parts.next().is_some() {
                    self.block.start_index.clear();
                } else {
                    self.block.start_index = page.to_string();
                }
            }
            Screen::Blocks => {
                self.blocks.start_height =
                    trimmed.chars().filter(|ch| ch.is_ascii_digit()).collect();
            }
            Screen::Tx => self.tx.txid = one_query_token(trimmed),
            Screen::Address => self.address.address = one_query_token(trimmed),
            Screen::MultiAddress => self.multi_address.input = trimmed.to_string(),
            Screen::Broadcast => {
                self.broadcast.raw_tx = trimmed.chars().filter(|ch| !ch.is_whitespace()).collect();
            }
            Screen::TestTransactions => self.test_txs.input = trimmed.to_string(),
            _ => {}
        }
    }

    pub fn push_str(&mut self, text: &str) {
        match self.screen {
            Screen::Block => self.block.query.push_str(text),
            Screen::Blocks => {
                for ch in text.chars() {
                    if ch.is_ascii_digit() {
                        self.blocks.start_height.push(ch);
                    }
                }
            }
            Screen::Tx => self.tx.txid.push_str(text),
            Screen::Address => self.address.address.push_str(text),
            Screen::MultiAddress => self.multi_address.input.push_str(text),
            Screen::Broadcast => {
                for ch in text.chars() {
                    if !ch.is_whitespace() {
                        self.broadcast.raw_tx.push(ch);
                    }
                }
            }
            Screen::TestTransactions => self.test_txs.input.push_str(text),
            _ => {}
        }
    }

    pub fn backspace(&mut self) {
        match self.screen {
            Screen::Block => {
                self.block.query.pop();
            }
            Screen::Blocks => {
                self.blocks.start_height.pop();
            }
            Screen::Tx => {
                self.tx.txid.pop();
            }
            Screen::Address => {
                self.address.address.pop();
            }
            Screen::MultiAddress => {
                self.multi_address.input.pop();
            }
            Screen::Broadcast => {
                self.broadcast.raw_tx.pop();
            }
            Screen::TestTransactions => {
                self.test_txs.input.pop();
            }
            _ => {}
        }
    }
}

fn one_query_token(text: &str) -> String {
    text.split_whitespace().next().unwrap_or("").to_string()
}

impl<G: ExplorerGet + ExplorerPost> ScreenHost<G> {
    pub fn root(&self) -> String {
        self.client.root()
    }

    pub fn set_network(&mut self, network: Network) {
        self.client.set_network(network);
    }

    pub fn activate(&mut self, screen: Screen) -> Result<(), ClientError> {
        self.screen = screen;
        self.reload()
    }

    pub fn reload(&mut self) -> Result<(), ClientError> {
        match self.screen {
            Screen::Dashboard => self.dashboard.load(&self.client),
            Screen::Blocks => self.blocks.load(&self.client),
            Screen::Block => self.block.load(&self.client),
            Screen::Tx => self.tx.load(&self.client),
            Screen::Address => self.address.load(&self.client),
            Screen::Mempool => self.mempool.load(&self.client),
            Screen::MultiAddress => self.multi_address.load(&self.client),
            Screen::Broadcast
            | Screen::TestTransactions
            | Screen::Popup
            | Screen::Terms
            | Screen::Privacy
            | Screen::Trademark
            | Screen::Docs
            | Screen::Faq
            | Screen::ApiRest
            | Screen::ApiWebsocket => Ok(()),
        }
    }

    pub fn submit(&mut self) -> Result<(), ClientError> {
        match self.screen {
            Screen::Broadcast => self.broadcast.submit(&self.client),
            Screen::TestTransactions => self.test_txs.submit(&self.client),
            _ => Err(ClientError::BadPath),
        }
    }

    pub fn rendered(&self) -> String {
        match self.screen {
            Screen::Dashboard => self.dashboard.rendered(),
            Screen::Blocks => self.blocks.rendered(),
            Screen::Block => self.block.rendered(),
            Screen::Tx => self.tx.rendered(),
            Screen::Address => self.address.rendered(),
            Screen::Mempool => self.mempool.rendered(),
            Screen::MultiAddress => self.multi_address.rendered(),
            Screen::Broadcast => self.broadcast.rendered(),
            Screen::TestTransactions => self.test_txs.rendered(),
            Screen::Terms => StaticScreen::new(StaticPage::Terms).show().to_string(),
            Screen::Privacy => StaticScreen::new(StaticPage::Privacy).show().to_string(),
            Screen::Trademark => StaticScreen::new(StaticPage::Trademark).show().to_string(),
            Screen::Docs => StaticScreen::new(StaticPage::Docs).show().to_string(),
            Screen::Faq => resolve_doc_route("faq").body.to_string(),
            Screen::ApiRest => resolve_doc_route("api/rest").body.to_string(),
            Screen::ApiWebsocket => resolve_doc_route("api/websocket").body.to_string(),
            Screen::Popup => String::new(),
        }
    }
}

/// Text `Shell::render` passes to `.child` for the screen body.
pub fn painted_shell_text<G: ExplorerGet + ExplorerPost>(
    host: &ScreenHost<G>,
    popup: &NpubPopupModel,
) -> String {
    if host.screen == Screen::Popup {
        format!("Splora\n{}", popup.npub())
    } else {
        let mut lines = vec![
            format!("screen: {}", host.screen.label()),
            format!("query: {}", host.query_text()),
            host.rendered(),
        ];
        if !host.notice.is_empty() {
            lines.push(host.notice.clone());
        }
        lines.join("\n")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::{ClientError, ExplorerClient, ExplorerGet, ExplorerPost};
    use crate::popup::NpubPopupModel;
    use splora_api::{
        Network, SIGNET_PATH, api_prefix, api_root, is_backend_network, signet_is_backend,
    };
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
    fn prefix_table_has_five_networks_and_no_signet_backend() {
        assert_eq!(Network::ALL.len(), 5);
        assert_eq!(NetworkSwitcher::networks().len(), 5);
        let switcher = NetworkSwitcher::new(Network::Mainnet);
        assert_eq!(
            switcher.prefixes(),
            vec![
                "/api",
                "/testnet/api",
                "/testnet4/api",
                "/mutinynet/api",
                "/liquid/api",
            ]
        );
        for network in Network::ALL {
            assert_eq!(api_prefix(network), network.api_prefix());
        }
        assert!(!signet_is_backend());
        assert_eq!(SIGNET_PATH, "/signet");
        let labels = switcher.labels();
        assert_eq!(
            labels,
            vec!["mainnet", "testnet3", "testnet4", "mutinynet", "liquid"]
        );
        assert!(labels.iter().all(|label| !label.contains("signet")));
        assert!(
            switcher
                .prefixes()
                .iter()
                .all(|prefix| *prefix != SIGNET_PATH)
        );
        assert!(!switcher.summary().contains("signet"));
    }

    #[test]
    fn switcher_changes_the_client_root() {
        let mut switcher = NetworkSwitcher::new(Network::Mainnet);
        assert_eq!(switcher.api_root(), api_root(Network::Mainnet));
        switcher.select(Network::Testnet4);
        let client = ExplorerClient::new(
            switcher.selected(),
            MapExplorer {
                map: BTreeMap::new(),
            },
        );
        assert_eq!(client.root(), api_root(Network::Testnet4));
        assert_eq!(client.root(), switcher.api_root());
        assert_eq!(Screen::Popup.after_continue(), Screen::Dashboard);
    }

    #[test]
    fn select_changes_the_active_prefix() {
        let mut switcher = NetworkSwitcher::new(Network::Mainnet);
        let mut client = ExplorerClient::new(
            switcher.selected(),
            MapExplorer {
                map: BTreeMap::new(),
            },
        );
        let root = client.root();
        assert!(root.ends_with("/api"), "{root}");
        assert!(!root.ends_with("/testnet/api"), "{root}");
        assert!(!root.contains("/testnet/api"), "{root}");
        assert_eq!(root, api_root(Network::Mainnet));
        assert_eq!(api_prefix(switcher.selected()), "/api");
        assert_eq!(root.strip_prefix(splora_api::BASE_URL), Some("/api"));

        let steps = [
            (Network::Testnet3, "/testnet/api"),
            (Network::Testnet4, "/testnet4/api"),
            (Network::Mutinynet, "/mutinynet/api"),
            (Network::Liquid, "/liquid/api"),
        ];
        for (network, prefix) in steps {
            switcher.select(network);
            client.set_network(switcher.selected());
            assert_eq!(switcher.selected(), network);
            assert_eq!(api_prefix(network), prefix);
            assert!(client.root().ends_with(prefix), "{}", client.root());
            assert_eq!(client.root(), api_root(network));
            assert_eq!(client.root(), switcher.api_root());
            assert_eq!(
                client.root().strip_prefix(splora_api::BASE_URL),
                Some(prefix)
            );
        }

        assert!(!signet_is_backend());
        assert!(!is_backend_network("signet"));
        assert!(!is_backend_network("/signet"));
        assert!(!is_backend_network("/signet/api"));
        assert_eq!(NetworkSwitcher::networks().len(), 5);
        assert!(
            switcher
                .prefixes()
                .iter()
                .all(|prefix| *prefix != SIGNET_PATH && !prefix.contains("signet"))
        );
        assert!(
            switcher
                .labels()
                .iter()
                .all(|label| !label.contains("signet"))
        );
        assert!(
            NetworkSwitcher::networks()
                .iter()
                .all(|network| api_prefix(*network) != SIGNET_PATH)
        );
    }

    #[derive(Clone, Default)]
    struct Rec {
        gets: std::rc::Rc<std::cell::RefCell<Vec<String>>>,
        posts: std::rc::Rc<std::cell::RefCell<Vec<(String, String, String)>>>,
    }

    impl ExplorerGet for Rec {
        fn get_text(&self, url: &str) -> Result<String, ClientError> {
            Err(ClientError::Http(format!("unexpected get {url}")))
        }

        fn get_request(&self, request: &crate::api::ClientRequest) -> Result<String, ClientError> {
            self.gets.borrow_mut().push(request.url.clone());
            Ok(format!("body {}", request.path))
        }
    }

    impl ExplorerPost for Rec {
        fn post_text(&self, request: &crate::api::ClientRequest) -> Result<String, ClientError> {
            self.posts.borrow_mut().push((
                request.method.to_string(),
                request.path.clone(),
                request.url.clone(),
            ));
            Ok(format!("posted {}", request.path))
        }
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

    #[test]
    fn dashboard_links_cover_the_v1_screens_and_not_signet() {
        let links = Dashboard::new().links();
        assert_eq!(&links[..4], &["block", "tx", "address", "mempool"][..]);
        for link in links {
            let screen = Screen::from_link(link).expect(link);
            assert_eq!(screen.label(), link);
        }
        assert!(Screen::from_link("signet").is_none());
        assert!(Screen::from_link("mining").is_none());
        assert_eq!(NetworkSwitcher::networks().len(), 5);
        assert!(!signet_is_backend());
    }

    #[test]
    fn multi_address_screen_gets_one_address_per_entry() {
        let rec = Rec::default();
        let client = ExplorerClient::new(Network::Mainnet, rec.clone());
        let mut screen = MultiAddressScreen::new();
        screen.input = "addrA, addrB\naddrC".to_string();
        assert_eq!(screen.entered(), vec!["addrA", "addrB", "addrC"]);
        screen.load(&client).expect("load");
        assert_eq!(screen.requests.len(), 3);
        assert_eq!(rec.gets.borrow().len(), 3);
        assert!(rec.posts.borrow().is_empty());
        for (index, address) in ["addrA", "addrB", "addrC"].iter().enumerate() {
            let request = &screen.requests[index];
            assert_eq!(request.method, "GET");
            assert_eq!(request.path, format!("/address/{address}"));
            assert_eq!(
                request.url,
                format!("https://splora.surmount.systems/api/address/{address}")
            );
            assert_eq!(rec.gets.borrow()[index], request.url);
            assert_url_allowed(&request.url);
        }
        assert!(!screen.summary().contains("package"));
    }

    #[test]
    fn broadcast_screen_posts_tx() {
        let rec = Rec::default();
        let client = ExplorerClient::new(Network::Mainnet, rec.clone());
        let mut screen = BroadcastScreen::new();
        assert_eq!(screen.summary(), "broadcast: POST /tx");
        screen.raw_tx = "abcd".to_string();
        screen.submit(&client).expect("broadcast");
        let request = screen.request.expect("request");
        assert_eq!(request.method, "POST");
        assert_eq!(request.path, "/tx");
        assert_eq!(request.body, "abcd");
        assert!(request.url.ends_with("/tx"));
        assert_url_allowed(&request.url);
        let posts = rec.posts.borrow();
        assert_eq!(posts.len(), 1);
        assert_eq!(posts[0].0, "POST");
        assert_eq!(posts[0].1, "/tx");
        assert_eq!(posts[0].2, request.url);
        assert!(rec.gets.borrow().is_empty());
    }

    #[test]
    fn test_transactions_screen_posts_txs_test() {
        let rec = Rec::default();
        let client = ExplorerClient::new(Network::Mutinynet, rec.clone());
        let mut screen = TestTransactionsScreen::new();
        assert_eq!(screen.summary(), "test transactions: POST /txs/test");
        screen.input = "abcd, ef01".to_string();
        screen.submit(&client).expect("test");
        let request = screen.request.expect("request");
        assert_eq!(request.method, "POST");
        assert_eq!(request.path, "/txs/test");
        assert_eq!(request.content_type, "application/json");
        assert_eq!(request.body, r#"["abcd","ef01"]"#);
        assert!(request.url.ends_with("/mutinynet/api/txs/test"));
        assert_url_allowed(&request.url);
        let posts = rec.posts.borrow();
        assert_eq!(posts.len(), 1);
        assert_eq!(posts[0].0, "POST");
        assert_eq!(posts[0].1, "/txs/test");
        assert!(rec.gets.borrow().is_empty());
    }

    #[test]
    fn static_pages_do_not_call_the_indexer() {
        let rec = Rec::default();
        let client = ExplorerClient::new(Network::Mainnet, rec.clone());
        let _root = client.root();
        assert_eq!(StaticPage::ALL.len(), 4);
        for page in StaticPage::ALL {
            assert!(!page.calls_indexer());
            let screen = StaticScreen::new(page);
            let text = screen.show();
            assert!(text.len() > 80, "{}", page.title());
            assert!(!text.contains('{'), "{}", page.title());
            assert!(!text.contains("chain_stats"));
            assert!(!text.is_empty());
            assert_eq!(screen.summary().starts_with(page.title()), true);
            for banned in BANNED_URL_PARTS {
                assert!(!text.contains(banned), "{} contains {banned}", page.title());
            }
        }
        assert!(rec.gets.borrow().is_empty());
        assert!(rec.posts.borrow().is_empty());
        let docs = StaticScreen::new(StaticPage::Docs).show();
        assert!(docs.contains("POST of /tx"));
        assert!(docs.contains("POST of /txs/test"));
        assert!(docs.contains("GET of /address/"));
        assert!(docs.contains("Signet is not a backend"));
    }

    #[derive(Clone, Default)]
    struct DocSpy {
        calls: std::rc::Rc<std::cell::RefCell<Vec<String>>>,
    }

    impl ExplorerGet for DocSpy {
        fn get_text(&self, url: &str) -> Result<String, ClientError> {
            self.calls.borrow_mut().push(format!("GET {url}"));
            Err(ClientError::Http(format!("unexpected get {url}")))
        }

        fn get_request(&self, request: &crate::api::ClientRequest) -> Result<String, ClientError> {
            self.calls.borrow_mut().push(format!("GET {}", request.url));
            Err(ClientError::Http(format!("unexpected get {}", request.url)))
        }
    }

    impl ExplorerPost for DocSpy {
        fn post_text(&self, request: &crate::api::ClientRequest) -> Result<String, ClientError> {
            self.calls
                .borrow_mut()
                .push(format!("POST {}", request.url));
            Err(ClientError::Http(format!(
                "unexpected post {}",
                request.url
            )))
        }
    }

    fn assert_static_doc(page: DocPage) {
        assert_eq!(page.status, 200);
        assert!(!page.calls_indexer());
        assert!(page.body.len() > 80);
        assert!(!page.body.contains("nsec"));
        assert!(!page.body.contains('{'));
        for banned in BANNED_URL_PARTS {
            assert!(!page.body.contains(banned), "{}", page.body);
        }
    }

    #[test]
    fn doc_routes_do_not_call_the_indexer() {
        let spy = DocSpy::default();
        let client = ExplorerClient::new(Network::Mainnet, spy.clone());
        let faq = open_doc_route("faq", &client);
        let rest = open_doc_route("api/rest", &client);
        let rest_type = route_api_type("api/rest");
        let websocket = open_doc_route("api/websocket", &client);
        assert_static_doc(faq);
        assert_static_doc(rest);
        assert_static_doc(websocket);
        assert_eq!(rest_type, rest);
        assert_eq!(rest.body, REST_DOC);
        assert_eq!(websocket.body, WEBSOCKET_DOC);
        assert!(faq.body.contains("FAQ"));
        assert!(rest.body.contains("POST of /tx"));
        assert!(rest.body.contains("POST of /txs/test"));
        assert!(websocket.body.to_ascii_lowercase().contains("websocket"));
        assert_eq!(Screen::from_link("faq"), Some(Screen::Faq));
        assert_eq!(Screen::from_link("api/rest"), Some(Screen::ApiRest));
        assert_eq!(
            Screen::from_link("api/websocket"),
            Some(Screen::ApiWebsocket)
        );
        assert_eq!(Screen::from_link("faq").expect("faq").label(), "faq");
        assert_eq!(
            Screen::from_link("api/rest").expect("rest").label(),
            "api/rest"
        );
        assert_eq!(
            Screen::from_link("api/websocket")
                .expect("websocket")
                .label(),
            "api/websocket"
        );

        for path in ["api/other", "api/", "api", "faq/extra", "/faq"] {
            let page = open_doc_route(path, &client);
            assert_eq!(page.status, 404, "{path}");
            assert_eq!(page.body, DOC_NOT_FOUND);
            assert!(!page.calls_indexer());
            assert!(Screen::from_link(path).is_none(), "{path}");
            for banned in BANNED_URL_PARTS {
                assert!(!page.body.contains(banned), "{path}");
            }
        }
        for banned in BANNED_URL_PARTS {
            let kind = banned.rsplit('/').next().expect("segment");
            let path = format!("api/{kind}");
            let page = open_doc_route(&path, &client);
            assert_eq!(page.status, 404, "{path}");
            assert_eq!(page.body, DOC_NOT_FOUND);
            assert!(!page.body.contains(banned), "{path}");
            assert!(Screen::from_link(&path).is_none(), "{path}");
        }
        assert!(spy.calls.borrow().is_empty());
        let lines = known_doc_lines();
        assert!(lines.contains("FAQ"));
        assert!(lines.contains("REST API"));
        assert!(lines.contains("Websocket API"));
        assert!(client.root().ends_with("/api"));
    }

    const BLOCK_HASH: &str = "abc123def456";
    const BLOCK_HEIGHT: &str = "800000";
    const BLOCK_JSON: &str = r#"{"id":"abc123def456","height":800000,"version":1,"timestamp":1700000000,"tx_count":2,"size":200,"weight":800,"merkle_root":"mm","previousblockhash":"pp","mediantime":1700000000,"nonce":1,"bits":2,"difficulty":3.0}"#;
    const BLOCKS_JSON: &str = r#"[{"id":"abc123def456","height":800000,"version":1,"timestamp":1700000000,"tx_count":2,"size":200,"weight":800,"merkle_root":"mm","previousblockhash":"pp","mediantime":1700000000,"nonce":1,"bits":2,"difficulty":3.0}]"#;
    const BLOCK_TXS_JSON: &str = r#"[{"txid":"txidintx999","version":1,"locktime":0,"vin":[],"vout":[],"size":100,"weight":400,"sigops":1,"fee":50,"status":{"confirmed":true}}]"#;
    const RECENT_JSON: &str = r#"[{"txid":"txidrecent222","fee":321,"vsize":111,"value":999}]"#;
    const TX_JSON: &str = r#"{"txid":"txidabc","version":2,"locktime":0,"vin":[],"vout":[],"size":120,"weight":480,"sigops":1,"fee":10,"status":{"confirmed":true,"block_height":800000}}"#;
    const ADDRESS_JSON: &str = r#"{"address":"DDogAddress111","chain_stats":{"tx_count":7,"funded_txo_count":4,"spent_txo_count":1,"funded_txo_sum":9000,"spent_txo_sum":1000},"mempool_stats":{"tx_count":0,"funded_txo_count":0,"spent_txo_count":0,"funded_txo_sum":0,"spent_txo_sum":0}}"#;
    const ADDRESS_TXS_JSON: &str = r#"[{"txid":"addresstx333","version":1,"locktime":0,"vin":[],"vout":[],"size":10,"weight":40,"sigops":0,"fee":1,"status":{"confirmed":false}}]"#;
    const ADDRESS_UTXO_JSON: &str = r#"[{"txid":"utxotx444","vout":1,"status":{"confirmed":true,"block_height":10},"value":5000}]"#;
    const ADDR_A_JSON: &str = r#"{"address":"addrA","chain_stats":{"tx_count":2,"funded_txo_count":2,"spent_txo_count":0,"funded_txo_sum":20,"spent_txo_sum":0},"mempool_stats":{"tx_count":0,"funded_txo_count":0,"spent_txo_count":0,"funded_txo_sum":0,"spent_txo_sum":0}}"#;
    const ADDR_B_JSON: &str = r#"{"address":"addrB","chain_stats":{"tx_count":4,"funded_txo_count":4,"spent_txo_count":1,"funded_txo_sum":40,"spent_txo_sum":5},"mempool_stats":{"tx_count":0,"funded_txo_count":0,"spent_txo_count":0,"funded_txo_sum":0,"spent_txo_sum":0}}"#;
    const TEST_TX_JSON: &str =
        r#"[{"txid":"testtxid666","wtxid":"wwww","allowed":true,"vsize":80,"reject-reason":""}]"#;

    #[derive(Clone)]
    struct BodyRec {
        network: Network,
        gets: std::rc::Rc<std::cell::RefCell<Vec<String>>>,
        posts: std::rc::Rc<std::cell::RefCell<Vec<String>>>,
        bodies: std::rc::Rc<BTreeMap<String, String>>,
    }

    impl BodyRec {
        fn new(network: Network, bodies: BTreeMap<String, String>) -> Self {
            Self {
                network,
                gets: std::rc::Rc::new(std::cell::RefCell::new(Vec::new())),
                posts: std::rc::Rc::new(std::cell::RefCell::new(Vec::new())),
                bodies: std::rc::Rc::new(bodies),
            }
        }

        fn get_paths(&self) -> Vec<String> {
            self.gets.borrow().clone()
        }

        fn post_paths(&self) -> Vec<String> {
            self.posts.borrow().clone()
        }
    }

    fn indexer_path(network: Network, url: &str) -> String {
        let root = api_root(network);
        url.strip_prefix(&root).unwrap_or(url).to_string()
    }

    impl ExplorerGet for BodyRec {
        fn get_text(&self, url: &str) -> Result<String, ClientError> {
            let path = indexer_path(self.network, url);
            self.gets.borrow_mut().push(path.clone());
            self.bodies
                .get(&path)
                .cloned()
                .ok_or_else(|| ClientError::Http(format!("missing {path}")))
        }

        fn get_request(&self, request: &crate::api::ClientRequest) -> Result<String, ClientError> {
            assert_eq!(request.method, "GET");
            self.gets.borrow_mut().push(request.path.clone());
            self.bodies
                .get(&request.path)
                .cloned()
                .ok_or_else(|| ClientError::Http(format!("missing {}", request.path)))
        }
    }

    impl ExplorerPost for BodyRec {
        fn post_text(&self, request: &crate::api::ClientRequest) -> Result<String, ClientError> {
            assert_eq!(request.method, "POST");
            self.posts.borrow_mut().push(request.path.clone());
            self.bodies
                .get(&request.path)
                .cloned()
                .ok_or_else(|| ClientError::Http(format!("missing {}", request.path)))
        }
    }

    fn assert_paths_allowed(paths: &[String]) {
        for path in paths {
            assert!(!path.contains("/api/v1/"), "{path}");
            assert!(!path.contains("/v1/blocks"), "{path}");
            assert!(!path.contains("ws"), "{path}");
            for banned in BANNED_URL_PARTS {
                assert!(!path.contains(banned), "{path} contains {banned}");
            }
        }
    }

    fn host_with(
        network: Network,
        bodies: BTreeMap<String, String>,
    ) -> (ScreenHost<BodyRec>, BodyRec) {
        let rec = BodyRec::new(network, bodies);
        let host = ScreenHost::new(ExplorerClient::new(network, rec.clone()));
        (host, rec)
    }

    fn painted_of(host: &ScreenHost<BodyRec>) -> String {
        let popup = NpubPopupModel::new("npub1paintedshell").expect("npub");
        painted_shell_text(host, &popup)
    }

    fn assert_view(host: &ScreenHost<BodyRec>, needles: &[&str]) {
        let host_text = host.rendered();
        let painted = painted_of(host);
        for needle in needles {
            assert!(
                painted.contains(needle),
                "painted shell missing {needle}: {painted}"
            );
            assert!(
                host_text.contains(needle),
                "host rendered missing {needle}: {host_text}"
            );
        }
    }

    fn assert_painted_screen(host: &ScreenHost<BodyRec>, label: &str) {
        let painted = painted_of(host);
        let line = format!("screen: {label}\n");
        assert!(
            painted.contains(&line),
            "painted shell missing {line}: {painted}"
        );
        assert!(painted.contains("query:"), "{painted}");
    }

    fn assert_no_banned_products(text: &str) {
        for word in ["mining", "prices", "lightning", "acceleration"] {
            assert!(
                !text.to_ascii_lowercase().contains(word),
                "{text} contains {word}"
            );
        }
    }

    fn source_fn<'a>(source: &'a str, signature: &str, end_marker: &str) -> &'a str {
        let start = source
            .find(signature)
            .unwrap_or_else(|| panic!("missing {signature}"));
        let rest = &source[start..];
        let end = rest
            .find(end_marker)
            .unwrap_or_else(|| panic!("missing {end_marker} after {signature}"));
        &rest[..end]
    }

    #[test]
    fn v1_dashboard_requests_blocks_and_mempool_recent() {
        let (mut host, rec) = host_with(
            Network::Testnet4,
            BTreeMap::from([
                ("/blocks/tip/height".to_string(), "100".to_string()),
                ("/blocks/tip/hash".to_string(), "abc".to_string()),
                ("/blocks".to_string(), BLOCKS_JSON.to_string()),
                ("/mempool/recent".to_string(), RECENT_JSON.to_string()),
                (
                    "/fee-estimates".to_string(),
                    r#"{"2":1.1,"6":3}"#.to_string(),
                ),
                (
                    "/mempool".to_string(),
                    r#"{"count":4,"vsize":500,"total_fee":90,"fee_histogram":[[2,100]]}"#
                        .to_string(),
                ),
            ]),
        );
        host.activate(Screen::Dashboard).expect("dashboard");
        assert_eq!(
            rec.get_paths(),
            vec![
                "/blocks".to_string(),
                "/mempool/recent".to_string(),
                "/blocks/tip/height".to_string(),
                "/blocks/tip/hash".to_string(),
                "/fee-estimates".to_string(),
                "/mempool".to_string(),
            ]
        );
        assert!(rec.post_paths().is_empty());
        assert_paths_allowed(&rec.get_paths());
        assert_painted_screen(&host, "dashboard");
        assert_view(
            &host,
            &[
                "block hash abc123def456 height 800000",
                "recent txid txidrecent222 fee 321 vsize 111 value 999",
                "block hash abc123def456 height 800000 tx_count 2 timestamp 1700000000 size 200 weight 800 previous_hash pp",
            ],
        );
        let painted = painted_of(&host);
        assert_no_banned_products(&painted);
        assert!(!painted.contains("nsec"));
        assert!(host.root().ends_with("/testnet4/api"));
    }

    #[test]
    fn v1_blocks_list_requests_blocks() {
        let (mut host, rec) = host_with(
            Network::Mainnet,
            BTreeMap::from([("/blocks".to_string(), BLOCKS_JSON.to_string())]),
        );
        host.activate(Screen::Blocks).expect("blocks");
        assert_eq!(rec.get_paths(), vec!["/blocks".to_string()]);
        assert!(rec.post_paths().is_empty());
        assert_paths_allowed(&rec.get_paths());
        assert!(!rec.get_paths().iter().any(|path| path.contains("v1")));
        assert_painted_screen(&host, "blocks");
        assert_view(
            &host,
            &[
                "block hash abc123def456 height 800000",
                "block hash abc123def456 height 800000 tx_count 2 timestamp 1700000000 size 200 weight 800 previous_hash pp",
            ],
        );
    }

    #[test]
    fn v1_blocks_list_requests_start_height() {
        let (mut host, rec) = host_with(
            Network::Liquid,
            BTreeMap::from([("/blocks/800000".to_string(), BLOCKS_JSON.to_string())]),
        );
        host.blocks.start_height = BLOCK_HEIGHT.to_string();
        host.activate(Screen::Blocks).expect("blocks height");
        assert_eq!(rec.get_paths(), vec!["/blocks/800000".to_string()]);
        assert_paths_allowed(&rec.get_paths());
        assert!(host.root().ends_with("/liquid/api"));
        assert_painted_screen(&host, "blocks");
        assert_view(
            &host,
            &[
                "block hash abc123def456 height 800000",
                "block hash abc123def456 height 800000 tx_count 2 timestamp 1700000000 size 200 weight 800 previous_hash pp",
            ],
        );
    }

    #[test]
    fn v1_block_requests_hash_and_txs() {
        let (mut host, rec) = host_with(
            Network::Mainnet,
            BTreeMap::from([
                (format!("/block/{BLOCK_HASH}"), BLOCK_JSON.to_string()),
                (
                    format!("/block/{BLOCK_HASH}/txs"),
                    BLOCK_TXS_JSON.to_string(),
                ),
                (
                    format!("/block/{BLOCK_HASH}/txids"),
                    r#"["onlyintxids"]"#.to_string(),
                ),
            ]),
        );
        host.block.query = BLOCK_HASH.to_string();
        host.activate(Screen::Block).expect("block");
        assert_eq!(
            rec.get_paths(),
            vec![
                format!("/block/{BLOCK_HASH}"),
                format!("/block/{BLOCK_HASH}/txs"),
                format!("/block/{BLOCK_HASH}/txids"),
            ]
        );
        assert!(
            rec.get_paths()
                .iter()
                .all(|path| !path.starts_with("/block-height/"))
        );
        assert_paths_allowed(&rec.get_paths());
        assert_painted_screen(&host, "block");
        assert_view(
            &host,
            &[
                "block hash abc123def456 height 800000",
                "tx txid txidintx999 fee 50 confirmed true block_height",
                "txid onlyintxids",
                "block hash abc123def456 height 800000 tx_count 2 timestamp 1700000000 size 200 weight 800 previous_hash pp",
                "size 100",
                "weight 400",
            ],
        );
    }

    #[test]
    fn v1_block_numeric_id_requests_block_height() {
        let (mut host, rec) = host_with(
            Network::Mutinynet,
            BTreeMap::from([
                (
                    format!("/block-height/{BLOCK_HEIGHT}"),
                    BLOCK_HASH.to_string(),
                ),
                (format!("/block/{BLOCK_HASH}"), BLOCK_JSON.to_string()),
                (
                    format!("/block/{BLOCK_HASH}/txs"),
                    BLOCK_TXS_JSON.to_string(),
                ),
                (
                    format!("/block/{BLOCK_HASH}/txids"),
                    r#"["onlyintxids"]"#.to_string(),
                ),
                (format!("/block/{BLOCK_HEIGHT}"), BLOCK_JSON.to_string()),
            ]),
        );
        host.block.query = BLOCK_HEIGHT.to_string();
        host.activate(Screen::Block).expect("height");
        assert_eq!(
            rec.get_paths(),
            vec![
                format!("/block-height/{BLOCK_HEIGHT}"),
                format!("/block/{BLOCK_HASH}"),
                format!("/block/{BLOCK_HASH}/txs"),
                format!("/block/{BLOCK_HASH}/txids"),
            ]
        );
        assert_paths_allowed(&rec.get_paths());
        assert_painted_screen(&host, "block");
        assert_view(
            &host,
            &[
                "block-height 800000 hash abc123def456",
                "block hash abc123def456 height 800000",
                "tx txid txidintx999 fee 50 confirmed true block_height",
                "txid onlyintxids",
                "block hash abc123def456 height 800000 tx_count 2 timestamp 1700000000 size 200 weight 800 previous_hash pp",
                "size 100",
                "weight 400",
            ],
        );
        assert!(host.root().ends_with("/mutinynet/api"));
    }

    #[test]
    fn v1_tx_requests_tx_and_renders_txid() {
        let (mut host, rec) = host_with(
            Network::Mainnet,
            BTreeMap::from([("/tx/txidabc".to_string(), TX_JSON.to_string())]),
        );
        host.tx.txid = "txidabc".to_string();
        host.activate(Screen::Tx).expect("tx");
        assert_eq!(rec.get_paths(), vec!["/tx/txidabc".to_string()]);
        assert!(rec.post_paths().is_empty());
        assert_paths_allowed(&rec.get_paths());
        assert_painted_screen(&host, "tx");
        assert_view(
            &host,
            &[
                "tx txid txidabc fee 10 confirmed true block_height 800000",
                "tx txid txidabc fee 10 confirmed true block_height 800000 size 120 weight 480",
            ],
        );
    }

    #[test]
    fn v1_address_requests_script_txs_and_utxo() {
        let script = "DDogAddress111";
        let (mut host, rec) = host_with(
            Network::Mainnet,
            BTreeMap::from([
                (format!("/address/{script}"), ADDRESS_JSON.to_string()),
                (
                    format!("/address/{script}/txs"),
                    ADDRESS_TXS_JSON.to_string(),
                ),
                (
                    format!("/address/{script}/utxo"),
                    ADDRESS_UTXO_JSON.to_string(),
                ),
            ]),
        );
        host.address.address = script.to_string();
        host.activate(Screen::Address).expect("address");
        assert_eq!(
            rec.get_paths(),
            vec![
                format!("/address/{script}"),
                format!("/address/{script}/txs"),
                format!("/address/{script}/utxo"),
            ]
        );
        assert_paths_allowed(&rec.get_paths());
        assert_painted_screen(&host, "address");
        assert_view(
            &host,
            &[
                "address DDogAddress111 chain_tx_count 7",
                "address tx txid addresstx333 confirmed false block_height",
                "utxo txid utxotx444 vout 1 value 5000 confirmed true",
                "address DDogAddress111 chain_tx_count 7 funded_sum 9000 mempool_tx_count 0",
                "fee 1",
                "size 10",
                "weight 40",
            ],
        );
    }

    #[test]
    fn v1_multi_address_renders_each_script() {
        let (mut host, rec) = host_with(
            Network::Mainnet,
            BTreeMap::from([
                ("/address/addrA".to_string(), ADDR_A_JSON.to_string()),
                ("/address/addrB".to_string(), ADDR_B_JSON.to_string()),
            ]),
        );
        host.multi_address.input = "addrA, addrB".to_string();
        host.activate(Screen::MultiAddress).expect("multi");
        assert_eq!(
            rec.get_paths(),
            vec!["/address/addrA".to_string(), "/address/addrB".to_string()]
        );
        assert!(rec.post_paths().is_empty());
        assert!(rec.get_paths().iter().all(|path| !path.contains(',')));
        assert_paths_allowed(&rec.get_paths());
        assert_painted_screen(&host, "addresses");
        assert_view(
            &host,
            &[
                "address addrA chain_tx_count 2",
                "address addrB chain_tx_count 4",
                "address addrA chain_tx_count 2 funded_sum 20 mempool_tx_count 0",
                "address addrB chain_tx_count 4 funded_sum 40 mempool_tx_count 0",
            ],
        );
    }

    #[test]
    fn v1_broadcast_renders_txid() {
        let (mut host, rec) = host_with(
            Network::Mainnet,
            BTreeMap::from([("/tx".to_string(), "broadcasttxid555".to_string())]),
        );
        host.broadcast.raw_tx = "abcd".to_string();
        host.activate(Screen::Broadcast).expect("open");
        assert!(rec.get_paths().is_empty());
        assert!(rec.post_paths().is_empty());
        host.submit().expect("broadcast");
        assert_eq!(rec.post_paths(), vec!["/tx".to_string()]);
        assert!(rec.get_paths().is_empty());
        assert_paths_allowed(&rec.post_paths());
        assert_painted_screen(&host, "broadcast");
        assert_view(&host, &["broadcast txid broadcasttxid555"]);
        let painted = painted_of(&host);
        assert!(!painted.contains("/txs/package"));
        assert_no_banned_products(&painted);
    }

    #[test]
    fn v1_test_transactions_render_result() {
        let (mut host, rec) = host_with(
            Network::Testnet3,
            BTreeMap::from([("/txs/test".to_string(), TEST_TX_JSON.to_string())]),
        );
        host.test_txs.input = "abcd, ef01".to_string();
        host.activate(Screen::TestTransactions).expect("open");
        assert!(rec.post_paths().is_empty());
        host.submit().expect("test");
        assert_eq!(rec.post_paths(), vec!["/txs/test".to_string()]);
        assert!(rec.get_paths().is_empty());
        assert_paths_allowed(&rec.post_paths());
        assert!(host.root().ends_with("/testnet/api"));
        assert_painted_screen(&host, "test transactions");
        assert_view(&host, &["test txid testtxid666 allowed true"]);
        let painted = painted_of(&host);
        assert!(!painted.contains("reject-reason"), "{painted}");
        assert!(!painted.contains("/txs/package"));
        assert_no_banned_products(&painted);
    }

    #[test]
    fn v1_test_transactions_accept_a_row_that_omits_reject_reason() {
        let body = r#"[{"txid":"testtxid666","wtxid":"wwww","allowed":true,"vsize":80}]"#;
        let (mut host, rec) = host_with(
            Network::Testnet3,
            BTreeMap::from([("/txs/test".to_string(), body.to_string())]),
        );
        host.test_txs.input = "abcd".to_string();
        host.activate(Screen::TestTransactions).expect("open");
        host.submit().expect("test");
        assert_eq!(rec.post_paths(), vec!["/txs/test".to_string()]);
        assert_painted_screen(&host, "test transactions");
        assert_view(&host, &["test txid testtxid666 allowed true"]);
        let painted = painted_of(&host);
        assert!(!painted.contains("reject-reason"), "{painted}");
        assert!(!painted.contains("missing reject-reason"), "{painted}");
        assert!(
            !host.rendered().contains("reject-reason"),
            "{}",
            host.rendered()
        );
    }

    #[test]
    fn dashboard_load_tip_calls_shared_tip_height_and_tip_hash() {
        let rec = BodyRec::new(
            Network::Mainnet,
            BTreeMap::from([
                ("/blocks/tip/height".to_string(), " 100\n".to_string()),
                ("/blocks/tip/hash".to_string(), " abc\n".to_string()),
            ]),
        );
        let client = ExplorerClient::new(Network::Mainnet, rec.clone());
        let mut dashboard = Dashboard::new();
        assert_eq!(dashboard.tip_line(), "tip height:");
        dashboard.load_tip(&client).expect("tip");
        assert_eq!(
            rec.get_paths(),
            vec![
                "/blocks/tip/height".to_string(),
                "/blocks/tip/hash".to_string(),
            ]
        );
        assert_eq!(dashboard.tip_height, "100");
        assert_eq!(dashboard.tip_hash, "abc");
        assert_eq!(dashboard.tip_line(), "tip height: 100 hash: abc");
        assert_paths_allowed(&rec.get_paths());
    }

    #[test]
    fn dashboard_open_requests_fee_estimates_mempool_and_tip() {
        let (mut host, rec) = host_with(
            Network::Mainnet,
            BTreeMap::from([
                ("/blocks".to_string(), BLOCKS_JSON.to_string()),
                ("/mempool/recent".to_string(), RECENT_JSON.to_string()),
                ("/blocks/tip/height".to_string(), "100".to_string()),
                ("/blocks/tip/hash".to_string(), "abc".to_string()),
                (
                    "/fee-estimates".to_string(),
                    r#"{"2":1.1,"6":3}"#.to_string(),
                ),
                (
                    "/mempool".to_string(),
                    r#"{"count":4,"vsize":500,"total_fee":90,"fee_histogram":[[2,100]]}"#
                        .to_string(),
                ),
            ]),
        );
        host.activate(Screen::Dashboard).expect("dashboard");
        let paths = rec.get_paths();
        assert!(
            paths.contains(&splora_frontend_shared::fee_estimates_path().to_string()),
            "{paths:?}"
        );
        assert!(
            paths.contains(&splora_frontend_shared::mempool_path().to_string()),
            "{paths:?}"
        );
        assert!(
            paths.contains(&splora_frontend_shared::blocks_tip_height_path().to_string()),
            "{paths:?}"
        );
        assert!(
            paths.contains(&splora_frontend_shared::blocks_tip_hash_path().to_string()),
            "{paths:?}"
        );
        assert_painted_screen(&host, "dashboard");
        assert_view(
            &host,
            &[
                "tip height: 100",
                "hash: abc",
                "fee target 2 rate 1.1",
                "fee target 6 rate 3",
                "fee_histogram 2 100",
                "mempool count 4 vsize 500 total_fee 90",
            ],
        );
        let painted = painted_of(&host);
        assert_no_banned_products(&painted);
        assert!(!painted.contains("nsec"));
        assert_paths_allowed(&paths);
    }

    #[test]
    fn block_open_requests_txids_and_paged_txs() {
        let (mut host, rec) = host_with(
            Network::Mainnet,
            BTreeMap::from([
                (format!("/block/{BLOCK_HASH}"), BLOCK_JSON.to_string()),
                (
                    format!("/block/{BLOCK_HASH}/txs"),
                    BLOCK_TXS_JSON.to_string(),
                ),
                (
                    format!("/block/{BLOCK_HASH}/txids"),
                    r#"["onlyintxids"]"#.to_string(),
                ),
                (
                    format!("/block/{BLOCK_HASH}/txs/25"),
                    r#"[{"txid":"txidpage25","version":1,"locktime":0,"vin":[],"vout":[],"size":10,"weight":40,"sigops":0,"fee":7,"status":{"confirmed":true,"block_height":800000}}]"#.to_string(),
                ),
            ]),
        );
        host.block.query = BLOCK_HASH.to_string();
        host.block.start_index = "25".to_string();
        host.activate(Screen::Block).expect("block");
        let paths = rec.get_paths();
        assert!(
            paths.contains(&splora_frontend_shared::block_txids_path(BLOCK_HASH)),
            "{paths:?}"
        );
        assert!(
            paths.contains(&splora_frontend_shared::block_txs_start_index_path(
                BLOCK_HASH, 25
            )),
            "{paths:?}"
        );
        assert_painted_screen(&host, "block");
        assert_view(
            &host,
            &[
                "txid onlyintxids",
                "tx txid txidpage25",
                "tx txid txidpage25 fee 7 confirmed true block_height 800000 size 10 weight 40",
                "size 100",
                "weight 400",
            ],
        );
        assert_paths_allowed(&paths);
    }

    #[test]
    fn shell_render_passes_body_text_from_painted_shell_text() {
        let source = include_str!("main.rs");
        let render = source_fn(source, "fn render(", "\nfn main");
        let body = source_fn(source, "fn body_text(", "\n    fn ");
        let render_compact: String = render.chars().filter(|ch| !ch.is_whitespace()).collect();
        let body_compact: String = body.chars().filter(|ch| !ch.is_whitespace()).collect();
        assert!(
            render_compact.contains(".child(self.body_text())"),
            "Shell::render dropped body_text from .child"
        );
        assert!(
            render_compact.contains(".child(network_line)"),
            "Shell::render dropped network_line from .child"
        );
        assert!(
            render.contains("network_choices"),
            "Shell::render dropped network_choices"
        );
        assert!(
            render.contains("self.switcher.summary()"),
            "Shell::render dropped switcher.summary"
        );
        assert!(
            body_compact.contains("painted_shell_text("),
            "body_text stopped calling painted_shell_text"
        );
        assert!(render.contains("Continue"));
        assert!(!render.contains("nsec"), "{render}");
        assert!(
            !render.to_ascii_lowercase().contains("private key"),
            "{render}"
        );
        assert!(!render.contains("private-key"), "{render}");
    }

    #[test]
    fn painted_popup_shows_splora_and_npub_not_a_private_key() {
        let (host, rec) = host_with(Network::Mainnet, BTreeMap::new());
        assert_eq!(host.screen, Screen::Popup);
        let popup = NpubPopupModel::new("npub1splorapopupvalue").expect("npub");
        let painted = painted_shell_text(&host, &popup);
        assert_eq!(painted, "Splora\nnpub1splorapopupvalue");
        assert!(painted.contains("Splora"), "{painted}");
        assert!(painted.contains(popup.npub()), "{painted}");
        assert!(!painted.contains("nsec"), "{painted}");
        assert!(
            !painted.to_ascii_lowercase().contains("private key"),
            "{painted}"
        );
        assert!(!painted.contains("private-key"), "{painted}");
        assert!(!painted.contains("screen:"), "{painted}");
        assert!(rec.get_paths().is_empty());
        assert!(rec.post_paths().is_empty());
        let render = source_fn(include_str!("main.rs"), "fn render(", "\nfn main");
        assert!(render.contains("Continue"));
    }

    #[test]
    fn painted_mempool_shows_recent_tx() {
        let (mut host, rec) = host_with(
            Network::Mainnet,
            BTreeMap::from([(
                splora_frontend_shared::mempool_recent_path().to_string(),
                RECENT_JSON.to_string(),
            )]),
        );
        host.activate(Screen::Mempool).expect("mempool");
        assert_eq!(
            rec.get_paths(),
            vec![splora_frontend_shared::mempool_recent_path().to_string()]
        );
        assert!(rec.post_paths().is_empty());
        assert_paths_allowed(&rec.get_paths());
        assert_painted_screen(&host, "mempool");
        assert_view(
            &host,
            &["recent txid txidrecent222 fee 321 vsize 111 value 999"],
        );
        assert_no_banned_products(&painted_of(&host));
    }

    #[test]
    fn painted_test_transactions_show_reject_reason() {
        let body = r#"[{"txid":"rejectedtxid888","allowed":false,"reject-reason":"dust"}]"#;
        let (mut host, rec) = host_with(
            Network::Testnet3,
            BTreeMap::from([(
                splora_frontend_shared::test_txs_path().to_string(),
                body.to_string(),
            )]),
        );
        host.test_txs.input = "abcd".to_string();
        host.activate(Screen::TestTransactions).expect("open");
        host.submit().expect("test");
        assert_eq!(
            rec.post_paths(),
            vec![splora_frontend_shared::test_txs_path().to_string()]
        );
        assert_painted_screen(&host, "test transactions");
        assert_view(
            &host,
            &[
                "test txid rejectedtxid888 allowed false",
                "reject-reason dust",
            ],
        );
    }

    fn assert_local_page(screen: Screen, label: &str, sentence: &str) {
        let (mut host, rec) = host_with(Network::Mainnet, BTreeMap::new());
        host.activate(screen).expect(label);
        assert!(rec.get_paths().is_empty(), "{:?}", rec.get_paths());
        assert!(rec.post_paths().is_empty(), "{:?}", rec.post_paths());
        assert_painted_screen(&host, label);
        assert_view(&host, &[sentence]);
        assert_no_banned_products(&painted_of(&host));
    }

    #[test]
    fn painted_terms_shows_unique_sentence() {
        assert_local_page(
            Screen::Terms,
            "terms",
            "You are responsible for any transaction you submit.",
        );
    }

    #[test]
    fn painted_privacy_shows_unique_sentence() {
        assert_local_page(
            Screen::Privacy,
            "privacy",
            "The nostr secret is not typed into that popup.",
        );
    }

    #[test]
    fn painted_trademark_shows_unique_sentence() {
        assert_local_page(
            Screen::Trademark,
            "trademark",
            "Nothing in this app gives you a license to those marks.",
        );
    }

    #[test]
    fn painted_docs_shows_unique_sentence() {
        assert_local_page(
            Screen::Docs,
            "docs",
            "This shell does not submit a transaction package.",
        );
    }

    #[test]
    fn painted_faq_shows_unique_sentence() {
        assert_local_page(
            Screen::Faq,
            "faq",
            "If a chain screen is empty, this page still shows the same answers.",
        );
    }

    #[test]
    fn painted_rest_shows_unique_sentence() {
        assert_local_page(
            Screen::ApiRest,
            "api/rest",
            "REST API. This page is local text.",
        );
    }

    #[test]
    fn painted_websocket_shows_unique_sentence() {
        assert_local_page(
            Screen::ApiWebsocket,
            "api/websocket",
            "The socket is not a substitute for the REST reads on the REST page.",
        );
    }
}
