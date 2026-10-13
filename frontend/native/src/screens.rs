//! View models for the desktop explorer. No GPUI types.

use splora_api::{Network, api_prefix, api_root};
use splora_frontend_shared::FiatCode::{Jpy, Usd};
use splora_frontend_shared::{
    BlockRow, CalculatorForm, FiatCode, RegistryAsset, TxRow, address_path, address_prefix_path,
    address_txs_chain_page_path, address_txs_mempool_path, address_txs_path,
    address_txs_summary_path, address_utxo_path, addresses_txs_body, addresses_txs_path,
    addresses_txs_summary_path, asset_path, asset_supply_decimal_path, asset_supply_path,
    asset_txs_chain_path, asset_txs_mempool_path, asset_txs_path, assets_registry_path,
    block_header_path, block_height_path, block_path, block_raw_path, block_status_path,
    block_txid_path, block_txids_path, block_txs_path, block_txs_start_index_path, blocks_path,
    blocks_start_height_path, broadcast_path, btc_fiat_jpy, btc_fiat_usd, btc_to_sats,
    calculator_field_names, cap_btc_supply, enter_btc, enter_fiat, fee_estimates_path, format_fiat,
    loaded_calculator, mempool_path, mempool_recent_path, mempool_txids_page_path,
    mempool_txids_path, package_path, parse_address_prefix, parse_address_txs_summary,
    parse_addresses_txs, parse_addresses_txs_summary, parse_asset_supply,
    parse_asset_supply_decimal, parse_assets_registry, parse_assets_registry_search, parse_block,
    parse_block_header, parse_block_height, parse_block_raw_hex, parse_block_status,
    parse_block_txid, parse_block_txids, parse_mempool_txids, parse_merkle_proof,
    parse_merkleblock_proof, parse_outspend, parse_outspends, parse_package, parse_scripthash,
    parse_scripthashes_txs, parse_tx, parse_tx_hex, parse_tx_raw_hex, route_availability,
    sat_per_vbyte_from_vsize, sat_per_vbyte_from_weight, sats_fiat_usd, sats_to_btc,
    scripthash_path, scripthash_txs_chain_page_path, scripthash_txs_mempool_path,
    scripthash_txs_path, scripthash_utxo_path, scripthashes_txs_body, scripthashes_txs_path,
    switch_usd_jpy_usd, test_txs_path, truncate_btc_decimals, tx_hex_path, tx_merkle_proof_path,
    tx_merkleblock_proof_path, tx_outspend_path, tx_outspends_path, tx_path, tx_raw_path,
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
    Calculator,
    Fiat,
    Statistics,
    Mining,
    Graphs,
    Hashrate,
    PriceChart,
    Lightning,
    Pools,
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
            Self::Calculator => "calculator",
            Self::Fiat => "fiat",
            Self::Statistics => "statistics",
            Self::Mining => "mining",
            Self::Graphs => "graphs",
            Self::Hashrate => "hashrate",
            Self::PriceChart => "price chart",
            Self::Lightning => "lightning",
            Self::Pools => "pools",
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
            Self::Calculator => "calculator",
            Self::Fiat => "fiat",
            Self::Statistics => "statistics",
            Self::Mining => "mining",
            Self::Graphs => "graphs",
            Self::Hashrate => "hashrate",
            Self::PriceChart => "price-chart",
            Self::Lightning => "lightning",
            Self::Pools => "pools",
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
            "calculator" => Some(Self::Calculator),
            "fiat" => Some(Self::Fiat),
            "statistics" => Some(Self::Statistics),
            "mining" => Some(Self::Mining),
            "graphs" => Some(Self::Graphs),
            "hashrate" => Some(Self::Hashrate),
            "price chart" => Some(Self::PriceChart),
            "lightning" => Some(Self::Lightning),
            "pools" => Some(Self::Pools),
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
    /// Typed on the dashboard. Enter opens a block, transaction, address, or search result.
    search: String,
    lines: String,
    rows: Vec<String>,
}

impl Dashboard {
    pub fn new() -> Self {
        Self {
            tip_height: String::new(),
            tip_hash: String::new(),
            search: String::new(),
            lines: String::new(),
            rows: Vec::new(),
        }
    }

    pub fn links(&self) -> [&'static str; 24] {
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
            "calculator",
            "fiat",
            "statistics",
            "mining",
            "graphs",
            "hashrate",
            "price chart",
            "lightning",
            "pools",
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
        match client.tip_height() {
            Ok(height) => self.tip_height = height,
            Err(ClientError::Http(message)) if message.starts_with("missing ") => {
                self.tip_height.clear();
            }
            Err(err) => return Err(err),
        }
        match client.tip_hash() {
            Ok(hash) => self.tip_hash = hash,
            Err(ClientError::Http(message)) if message.starts_with("missing ") => {
                self.tip_hash.clear();
            }
            Err(err) => return Err(err),
        }
        Ok(())
    }

    /// GET `/blocks`, `/mempool/recent`, the tip, `/fee-estimates`, and `/mempool`.
    /// The socket does not push blocks. The tip comes from the shared tip paths.
    pub fn load<G: ExplorerGet>(&mut self, client: &ExplorerClient<G>) -> Result<(), ClientError> {
        let blocks_route = blocks_path();
        let recent_route = mempool_recent_path();
        let blocks_body = read_home(client, blocks_route, "[]")?;
        let recent_body = read_home(client, recent_route, "[]")?;
        self.load_tip(client)?;
        let fees_route = fee_estimates_path();
        let fees_body = read_home(client, fees_route, "{}")?;
        let summary_route = mempool_path();
        let summary_body = read_home(
            client,
            summary_route,
            r#"{"count":0,"vsize":0,"total_fee":0,"fee_histogram":[]}"#,
        )?;
        let block_lines = render_block_list(&blocks_body)?;
        let recent_lines = render_recent(&recent_body)?;
        let fee_lines = render_fee_estimates(&fees_body)?;
        let summary_lines = render_mempool_summary(&summary_body)?;
        let mut rows = block_row_lines(&blocks_body)?;
        rows.extend(recent_row_lines(&recent_body)?);
        if !self.tip_hash.is_empty() {
            rows.push(format!("row tip hash {}", self.tip_hash));
        }
        self.rows = rows;
        self.lines = format!("{block_lines}\n{recent_lines}\n{fee_lines}\n{summary_lines}");
        if client.network() == Network::Liquid {
            if let Some(body) = read_optional(client, assets_registry_path())? {
                let names = registry_issued_lines(&body);
                if !names.is_empty() {
                    self.lines.push('\n');
                    self.lines.push_str(&names);
                }
                let cards = registry_featured_card_lines(&body);
                if !cards.is_empty() {
                    self.lines.push('\n');
                    self.lines.push_str(&cards);
                }
                for id in registry_asset_ids(&body) {
                    self.lines.push('\n');
                    self.lines.push_str(&format!("asset id {id}"));
                    self.rows.push(format!("row asset {id}"));
                }
            }
        }
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
    /// After a space, further digits are the txs page index, not more of the hash.
    typing_page: bool,
    pub body: String,
    rows: Vec<String>,
    /// Fields already on the loaded block, shown by the details toggle.
    details: String,
    details_open: bool,
}

impl BlockScreen {
    pub fn new() -> Self {
        Self {
            query: String::new(),
            start_index: String::new(),
            typing_page: false,
            body: String::new(),
            rows: Vec::new(),
            details: String::new(),
            details_open: false,
        }
    }

    pub fn summary(&self) -> String {
        format!("block: {} {}", self.query, self.body)
    }

    /// Hash: GET `/block/:hash`, GET `/block/:hash/txs`, and GET `/block/:hash/txids`.
    /// A numeric start index that is a multiple of 25 also GETs `/block/:hash/txs/:start_index`.
    /// All-digit id: GET `/block-height/:height`, then those hash routes.
    pub fn load<G: ExplorerGet>(&mut self, client: &ExplorerClient<G>) -> Result<(), ClientError> {
        self.rows.clear();
        self.details.clear();
        self.details_open = false;
        let mut lines = Vec::new();
        let hash = if is_numeric_id(&self.query) {
            let path = match self.query.parse::<u64>() {
                Ok(height) => block_height_path(height),
                Err(_) => format!("/block-height/{}", self.query),
            };
            let body = read_required(client, &path)?;
            let hash = parse_block_height(&body).map_err(http_parse)?;
            lines.push(format!("block-height {} hash {hash}", self.query));
            if hash.is_empty() || hash.contains('/') || hash.contains('?') || hash.contains(' ') {
                self.body = lines.join("\n");
                self.rows.clear();
                return Err(ClientError::Http("block height hash".into()));
            }
            hash
        } else if self.query.is_empty() || self.query.contains('/') {
            return Err(ClientError::BadPath);
        } else {
            self.query.clone()
        };
        let block_body = read_required(client, &block_path(&hash))?;
        let block = serde_json::from_str::<splora_frontend_shared::wire::Block>(&block_body)
            .map_err(http_parse)?;
        lines.push(render_block_line(&block));
        lines.push(splora_frontend_shared::transaction_count_heading(
            block.tx_count,
        ));
        let mut rows = vec![paint_block_row(&BlockRow::from(&block))];
        if let Some(prev) = block
            .previous_block_hash
            .as_deref()
            .filter(|prev| !prev.is_empty())
        {
            rows.push(format!("row previous_hash {prev}"));
        }
        rows.push("row block details".to_string());
        self.details = block_details_text(&block);
        let txs_body = read_required(client, &block_txs_path(&hash))?;
        lines.push(render_tx_list(&txs_body)?);
        rows.extend(transaction_row_lines(&txs_body)?);
        let txids_body = read_required(client, &block_txids_path(&hash))?;
        lines.push(render_txids(&txids_body)?);
        if let Some(status_body) = read_optional(client, &block_status_path(&hash))? {
            if let Ok(status) = parse_block_status(&status_body) {
                let height = status
                    .height
                    .map(|value| value.to_string())
                    .unwrap_or_default();
                let next = status.next_best.as_deref().unwrap_or("");
                lines.push(format!(
                    "block status in_best_chain {} height {height} next_best {next}",
                    yes_no(status.in_best_chain)
                ));
                if let Some(next) = status.next_best.as_deref().filter(|next| !next.is_empty()) {
                    rows.push(format!("row next_best {next}"));
                }
            }
        }
        if let Some(header) = read_optional(client, &block_header_path(&hash))? {
            if let Ok(hex) = parse_block_header(&header) {
                lines.push(format!("block header {hex}"));
            }
        }
        if let Some(raw) = read_optional(client, &block_raw_path(&hash))? {
            if let Ok(hex) = parse_block_raw_hex(&raw) {
                lines.push(format!("block raw {hex}"));
            }
        }
        if let Some(txid_body) = read_optional(client, &block_txid_path(&hash, 0))? {
            if let Ok(txid) = parse_block_txid(&txid_body) {
                lines.push(txid);
            }
        }
        if !self.start_index.is_empty() {
            if !is_numeric_id(&self.start_index) {
                return Err(ClientError::BadPath);
            }
            let start: u64 = self.start_index.parse().map_err(|_| ClientError::BadPath)?;
            if start % 25 != 0 {
                return Err(ClientError::BadPath);
            }
            let page_body = read_required(client, &block_txs_start_index_path(&hash, start))?;
            lines.push(render_tx_list(&page_body)?);
            rows.extend(transaction_row_lines(&page_body)?);
        }
        lines.extend(phone_pager_lines(block.tx_count, &self.start_index));
        let next_page = if self.start_index.is_empty() {
            25
        } else {
            self.start_index
                .parse::<u64>()
                .unwrap_or(0)
                .saturating_add(25)
        };
        rows.push(format!("row page {next_page}"));
        self.rows = rows;
        self.body = lines.join("\n");
        Ok(())
    }

    /// Show or hide the fields already stored from `GET /block/:hash`.
    pub fn toggle_details(&mut self) {
        self.details_open = !self.details_open;
        const MARK: &str = "\nblock details\n";
        if let Some(index) = self.body.find(MARK) {
            self.body.truncate(index);
        }
        if self.details_open && !self.details.is_empty() {
            if !self.body.is_empty() {
                self.body.push('\n');
            }
            self.body.push_str(&self.details);
        }
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
    rows: Vec<String>,
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
        let (request, body) = match client.get_path(&path) {
            Ok(pair) => pair,
            Err(ClientError::Http(message))
                if message.starts_with("missing ") && self.start_height.is_empty() =>
            {
                self.body = "blocks:".to_string();
                self.rows.clear();
                return Ok(());
            }
            Err(err) => return Err(err),
        };
        if request.method != "GET" || request.path != path || request.path.contains("v1") {
            return Err(ClientError::BadPath);
        }
        self.body = render_block_list(&body)?;
        self.rows = block_row_lines(&body)?;
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
    rows: Vec<String>,
}

impl TxScreen {
    pub fn new() -> Self {
        Self {
            txid: String::new(),
            body: String::new(),
            rows: Vec::new(),
        }
    }

    pub fn summary(&self) -> String {
        format!("tx: {} {}", self.txid, self.body)
    }

    pub fn load<G: ExplorerGet>(&mut self, client: &ExplorerClient<G>) -> Result<(), ClientError> {
        if self.txid.is_empty() || self.txid.contains('/') {
            return Err(ClientError::BadPath);
        }
        let body = read_required(client, &tx_path(&self.txid))?;
        let document: serde_json::Value = serde_json::from_str(&body).map_err(http_parse)?;
        let tx = transaction_keeping_absent_vin_keys(&serde_sigops_for_shared_type(
            &serde_weight_for_shared_type(&document),
        ))?;
        let vin_json = vin_objects(&document);
        let mut lines = vec![render_tx_row(
            &tx,
            &vin_json,
            json_u64_at(&document, "weight"),
        )];
        let mut rows = vec![paint_tx_row(&TxRow::from(&tx))];
        if let Some(hash) = tx
            .status
            .as_ref()
            .and_then(|status| status.block_hash.as_deref())
            .filter(|hash| !hash.is_empty())
        {
            rows.push(format!("row confirming {hash}"));
        }
        for input in &tx.vin {
            if let Some(address) = input
                .prevout
                .as_ref()
                .and_then(|prev| prev.script_pubkey_address.as_deref())
                .filter(|address| !address.is_empty())
            {
                rows.push(format!("row input address {address}"));
            }
        }
        for output in &tx.vout {
            if let Some(address) = output
                .script_pubkey_address
                .as_deref()
                .filter(|address| !address.is_empty())
            {
                rows.push(format!("row output address {address}"));
            }
        }
        if let Some(spends_body) = read_optional(client, &tx_outspends_path(&self.txid))? {
            if let Ok(spends) = parse_outspends(&spends_body) {
                for spend in spends {
                    push_spend(&mut lines, &mut rows, spend.spent, spend.txid.as_deref());
                }
            }
        }
        for (index, _) in tx.vout.iter().enumerate().take(50) {
            let path = tx_outspend_path(&self.txid, index as u64);
            if let Some(body) = read_optional(client, &path)? {
                if let Ok(spend) = parse_outspend(&body) {
                    push_spend(&mut lines, &mut rows, spend.spent, spend.txid.as_deref());
                }
            }
        }
        if let Some(proof_body) = read_optional(client, &tx_merkle_proof_path(&self.txid))? {
            if let Ok(proof) = parse_merkle_proof(&proof_body) {
                lines.push(format!(
                    "merkle block_height {} pos {}",
                    proof.block_height, proof.pos
                ));
                for hash in proof.merkle {
                    if !hash.is_empty() {
                        lines.push(hash.clone());
                        rows.push(format!("row merkle {hash}"));
                    }
                }
            }
        }
        if let Some(hex_body) = read_optional(client, &tx_hex_path(&self.txid))? {
            if let Ok(hex) = parse_tx_hex(&hex_body) {
                if even_hex(&hex) {
                    lines.push(format!("tx hex {hex}"));
                } else {
                    lines.push(hex);
                }
            }
        }
        if let Some(raw_body) = read_optional(client, &tx_raw_path(&self.txid))? {
            if let Ok(hex) = parse_tx_raw_hex(&raw_body) {
                lines.push(format!("tx raw {hex}"));
            }
        }
        if let Some(proof_body) = read_optional(client, &tx_merkleblock_proof_path(&self.txid))? {
            if let Ok(hex) = parse_merkleblock_proof(&proof_body) {
                lines.push(format!("merkleblock {hex}"));
            }
        }
        lines.extend(peg_lines(&body));
        lines.extend(poison_lines_for_tx(
            &tx,
            poison_network_name(client.network()),
        ));
        if let Some(words) = output_total_words(&tx) {
            lines.push(words);
        }
        lines.push(fee_words(tx.fee));
        lines.push(size_words(tx.size));
        if let Some(words) = virtual_size_words(&document) {
            lines.push(words);
        }
        if let Some(words) = adjusted_vsize_words(&document) {
            lines.push(words);
        }
        if let Some(words) = sigops_words(&document) {
            lines.push(words);
        }
        if let Some(words) = weight_words(&document) {
            lines.push(words);
        }
        let tip_height = tip_height_for_confirmations(client)?;
        let block_height = tx.status.as_ref().and_then(|status| status.block_height);
        if let Some(words) = confirmation_words(block_height, tip_height) {
            lines.push(words);
        }
        self.body = lines.join("\n");
        self.rows = rows;
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
    /// Confirmed history cursor for `GET /address/:script/txs/chain/:last_seen_txid`.
    last_seen: String,
    /// After a space, further characters are the chain cursor, not more of the address.
    typing_page: bool,
    /// A 64-hex scripthash uses `/scripthash/` instead of `/address/`.
    use_scripthash: bool,
    rows: Vec<String>,
}

impl AddressScreen {
    pub fn new() -> Self {
        Self {
            address: String::new(),
            body: String::new(),
            last_seen: String::new(),
            typing_page: false,
            use_scripthash: false,
            rows: Vec::new(),
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
        let stats_path = if self.use_scripthash {
            scripthash_path(script)
        } else {
            address_path(script)
        };
        let utxo_path = if self.use_scripthash {
            scripthash_utxo_path(script)
        } else {
            address_utxo_path(script)
        };
        let mempool_route = if self.use_scripthash {
            scripthash_txs_mempool_path(script)
        } else {
            address_txs_mempool_path(script)
        };
        let txs_path = if self.last_seen.is_empty() {
            if self.use_scripthash {
                scripthash_txs_path(script)
            } else {
                address_txs_path(script)
            }
        } else if self.use_scripthash {
            scripthash_txs_chain_page_path(script, &self.last_seen)
        } else {
            address_txs_chain_page_path(script, &self.last_seen)
        };
        let stats_body = read_required(client, &stats_path)?;
        let txs_body = read_required(client, &txs_path)?;
        let utxo_body = read_required(client, &utxo_path)?;
        let mempool_body = read_optional(client, &mempool_route)?;
        let mut lines = vec![render_address_stats(&stats_body)?];
        let mut rows = vec![address_funded_row(&stats_body)?];
        rows.push(format!("row script {script}"));
        if let Some(last) = paint_address_tx_page(&txs_body, &mut lines, &mut rows)? {
            rows.push(format!("row chain {last}"));
        }
        lines.push(render_utxos(&utxo_body)?);
        rows.extend(utxo_value_rows(&utxo_body)?);
        if let Some(body) = mempool_body {
            if body.trim() != "[]" {
                if let Ok(text) = render_tx_list(&body) {
                    if text != "The transaction list is empty." {
                        lines.push(text);
                    }
                }
                if let Ok(extra) = transaction_row_lines(&body) {
                    rows.extend(extra);
                }
            }
        }
        if !self.use_scripthash {
            if let Some(summary_body) = read_optional(client, &address_txs_summary_path(script))? {
                let _ = paint_address_tx_page(&summary_body, &mut lines, &mut rows)?;
            }
        }
        lines.extend(highlight_lines(&txs_body, script));
        lines.extend(peg_lines(&txs_body));
        lines.extend(poison_lines_for_txs(
            &txs_body,
            poison_network_name(client.network()),
        ));
        self.rows = rows;
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

/// Mempool backlog and recent entries.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MempoolScreen {
    pub body: String,
    rows: Vec<String>,
    last_seen: String,
}

impl MempoolScreen {
    pub fn new() -> Self {
        Self {
            body: String::new(),
            rows: Vec::new(),
            last_seen: String::new(),
        }
    }

    pub fn summary(&self) -> String {
        format!("mempool: {}", self.body)
    }

    /// GET `/mempool` and GET `/mempool/recent`.
    pub fn load<G: ExplorerGet>(&mut self, client: &ExplorerClient<G>) -> Result<(), ClientError> {
        let summary_route = mempool_path();
        let (summary, summary_body) = client.get_path(summary_route)?;
        if summary.method != "GET" || summary.path != summary_route {
            return Err(ClientError::BadPath);
        }
        let recent_route = mempool_recent_path();
        let (recent, recent_body) = client.get_path(recent_route)?;
        if recent.method != "GET" || recent.path != recent_route {
            return Err(ClientError::BadPath);
        }
        let summary_lines = render_mempool_summary(&summary_body)?;
        let recent_lines = render_recent(&recent_body)?;
        let mut rows = recent_row_lines(&recent_body)?;
        let mut lines = vec![summary_lines, recent_lines];
        if let Some(body) = read_optional(client, mempool_txids_path())? {
            if let Ok(txids) = parse_mempool_txids(&body) {
                if let Some(last) = txids.last() {
                    rows.push(format!("row mempool page {last}"));
                }
                for txid in &txids {
                    rows.push(format!("row mempool txid {txid}"));
                }
            }
        }
        if !self.last_seen.is_empty() {
            let page = mempool_txids_page_path(&self.last_seen);
            if let Some(body) = read_optional(client, &page)? {
                if let Ok(txids) = parse_mempool_txids(&body) {
                    for txid in txids {
                        lines.push(txid.clone());
                        rows.push(format!("row mempool txid {txid}"));
                    }
                }
            }
        }
        self.rows = rows;
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

/// One `GET /address/:script` and one `GET /address/:script/utxo` per entered address.
/// `POST /addresses/txs` for addresses and `POST /scripthashes/txs` for 64-hex scripthashes.
/// Also `POST /addresses/txs/summary` when at least one value was entered.
/// A missing history or summary does not drop the per-address reads.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct MultiAddressScreen {
    pub input: String,
    pub bodies: Vec<String>,
    pub requests: Vec<ClientRequest>,
    field_lines: String,
    rows: Vec<String>,
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

    pub fn load<G: ExplorerGet + ExplorerPost>(
        &mut self,
        client: &ExplorerClient<G>,
    ) -> Result<(), ClientError> {
        let entered = self
            .entered()
            .into_iter()
            .map(str::to_string)
            .collect::<Vec<_>>();
        let mut bodies = Vec::with_capacity(entered.len() * 2);
        let mut requests = Vec::with_capacity(entered.len() * 2);
        let mut field_lines = Vec::with_capacity(entered.len() * 2);
        let mut rows = Vec::new();
        for address in &entered {
            let (request, body) = client.fetch_address(address)?;
            let expected = address_path(address);
            if request.method != "GET" || request.path != expected {
                return Err(ClientError::BadPath);
            }
            match render_address_stats(&body) {
                Ok(line) => {
                    field_lines.push(line);
                    rows.push(address_funded_row(&body)?);
                }
                Err(_) => field_lines.push(body.clone()),
            }
            let utxo_path = address_utxo_path(address);
            let (utxo_request, utxo_body) = client.get_path(&utxo_path)?;
            if utxo_request.method != "GET" || utxo_request.path != utxo_path {
                return Err(ClientError::BadPath);
            }
            let utxos = render_utxos(&utxo_body)?;
            rows.extend(utxo_value_rows(&utxo_body)?);
            if !utxos.is_empty() {
                let labeled = utxos
                    .lines()
                    .map(|line| format!("address {address} {line}"))
                    .collect::<Vec<_>>()
                    .join("\n");
                field_lines.push(labeled);
            }
            requests.push(request);
            requests.push(utxo_request);
            bodies.push(body);
            bodies.push(utxo_body);
        }
        if !entered.is_empty() {
            let refs: Vec<&str> = entered.iter().map(String::as_str).collect();
            let body = addresses_txs_body(&refs);
            if let Some(response) = post_optional(
                client,
                addresses_txs_summary_path(),
                "application/json",
                body,
            )? {
                if let Ok(summaries) = parse_addresses_txs_summary(&response) {
                    for summary in summaries {
                        rows.push(format!(
                            "row summary txid {} height {} value {} time {} tx_position {}",
                            summary.txid,
                            summary.height,
                            summary.value,
                            summary.time,
                            summary.tx_position
                        ));
                    }
                }
            }
            let mut address_refs = Vec::new();
            let mut hash_refs = Vec::new();
            for value in &entered {
                if is_64_hex(value) {
                    hash_refs.push(value.to_ascii_lowercase());
                } else {
                    address_refs.push(value.clone());
                }
            }
            if !address_refs.is_empty() {
                let refs: Vec<&str> = address_refs.iter().map(String::as_str).collect();
                let body = addresses_txs_body(&refs);
                if let Some(response) =
                    post_optional(client, addresses_txs_path(), "application/json", body)?
                {
                    if let Ok(txs) = parse_addresses_txs(&response) {
                        for tx in txs {
                            rows.push(format!("row tx txid {} fee {}", tx.txid, tx.fee));
                        }
                    }
                }
            }
            if !hash_refs.is_empty() {
                let refs: Vec<&str> = hash_refs.iter().map(String::as_str).collect();
                let body = scripthashes_txs_body(&refs);
                if let Some(response) =
                    post_optional(client, scripthashes_txs_path(), "application/json", body)?
                {
                    if let Ok(txs) = parse_scripthashes_txs(&response) {
                        for tx in txs {
                            rows.push(format!("row tx txid {} fee {}", tx.txid, tx.fee));
                        }
                    }
                }
            }
        }
        for address in &entered {
            rows.push(format!("row entered {address}"));
        }
        self.requests = requests;
        self.bodies = bodies;
        self.rows = rows;
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
    rows: Vec<String>,
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
        let parts = broadcast_parts(&self.raw_tx);
        let (request, result) = if parts.len() > 1 {
            client.broadcast_package(&parts)?
        } else {
            client.broadcast(&self.raw_tx)?
        };
        if request.method != "POST" {
            return Err(ClientError::BadPath);
        }
        if parts.len() > 1 {
            if request.path != package_path() {
                return Err(ClientError::BadPath);
            }
        } else if request.path != broadcast_path() {
            return Err(ClientError::BadPath);
        }
        self.request = Some(request);
        self.result = result;
        self.rows = if parts.len() > 1 {
            package_txid_rows(&self.result)
        } else {
            Vec::new()
        };
        Ok(())
    }

    pub fn rendered(&self) -> String {
        if let Some(request) = &self.request {
            if request.method == "POST"
                && request.path == broadcast_path()
                && !self.result.is_empty()
            {
                return render_broadcast_result(&self.result).unwrap_or_else(|_| self.summary());
            }
            if request.method == "POST" && request.path == package_path() && !self.result.is_empty()
            {
                return render_package(&self.result).unwrap_or_else(|_| {
                    let trimmed = self.result.trim();
                    if trimmed.is_empty() {
                        "broadcast package".to_string()
                    } else {
                        trimmed.to_string()
                    }
                });
            }
        }
        self.summary()
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
A multi-address lookup sends one address request and one UTXO request per address you enter. \
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
It also reads each address UTXO list with GET /address/ and then that address and /utxo. \
It does not use a batch endpoint. \
Broadcast is a POST of /tx. The body is the raw transaction hex, as text. \
Test transactions are a POST of /txs/test. \
The body is a JSON array of raw transaction hex, at most 25 items. \
Signet is not a backend in this shell. \
Two raw transactions use POST /txs/package.";

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

fn is_64_hex(value: &str) -> bool {
    value.len() == 64 && value.bytes().all(|byte| byte.is_ascii_hexdigit())
}

fn even_hex(text: &str) -> bool {
    !text.is_empty() && text.len() % 2 == 0 && text.bytes().all(|byte| byte.is_ascii_hexdigit())
}

/// Bech32-style addresses are longer than 26 and are not a pure 64-hex id.
fn looks_like_address(value: &str) -> bool {
    value.len() >= 26
        && !is_64_hex(value)
        && value.bytes().all(|byte| byte.is_ascii_alphanumeric())
        && value.bytes().any(|byte| !byte.is_ascii_hexdigit())
}

const INDEXER_SENTENCES: &[&str] = &[
    "Transaction not found or is unconfirmed",
    "Transaction not found",
    "Block not found",
    "address search disabled",
    "Asset id not found",
    "endpoint does not exist",
];

fn indexer_sentence(body: &str) -> Option<&'static str> {
    let trimmed = body.trim();
    INDEXER_SENTENCES
        .iter()
        .copied()
        .find(|sentence| trimmed.starts_with(sentence))
}

/// Shell notice. An indexer sentence is shown without the `http:` prefix.
pub fn notice_from_error(err: &ClientError) -> String {
    match err {
        ClientError::BadPath => "bad path".to_string(),
        ClientError::Http(message) => {
            if let Some(sentence) = indexer_sentence(message) {
                sentence.to_string()
            } else {
                format!("http: {message}")
            }
        }
    }
}

fn read_home<G: ExplorerGet>(
    client: &ExplorerClient<G>,
    path: &str,
    empty: &str,
) -> Result<String, ClientError> {
    match client.get_path(path) {
        Ok((request, body)) => {
            if request.method != "GET" || request.path != path {
                return Err(ClientError::BadPath);
            }
            if let Some(sentence) = indexer_sentence(&body) {
                return Err(ClientError::Http(sentence.to_string()));
            }
            Ok(body)
        }
        Err(ClientError::Http(message)) if message.starts_with("missing ") => Ok(empty.to_string()),
        Err(err) => Err(err),
    }
}

fn read_required<G: ExplorerGet>(
    client: &ExplorerClient<G>,
    path: &str,
) -> Result<String, ClientError> {
    let (request, body) = client.get_path(path)?;
    if request.method != "GET" || request.path != path {
        return Err(ClientError::BadPath);
    }
    if let Some(sentence) = indexer_sentence(&body) {
        return Err(ClientError::Http(sentence.to_string()));
    }
    Ok(body)
}

fn read_optional<G: ExplorerGet>(
    client: &ExplorerClient<G>,
    path: &str,
) -> Result<Option<String>, ClientError> {
    match client.get_path(path) {
        Ok((request, body)) => {
            if request.method != "GET" || request.path != path {
                return Err(ClientError::BadPath);
            }
            if indexer_sentence(&body).is_some() {
                return Ok(None);
            }
            Ok(Some(body))
        }
        Err(ClientError::BadPath) => Err(ClientError::BadPath),
        Err(ClientError::Http(_)) => Ok(None),
    }
}

fn post_optional<G: ExplorerGet + ExplorerPost>(
    client: &ExplorerClient<G>,
    path: &str,
    content_type: &'static str,
    body: String,
) -> Result<Option<String>, ClientError> {
    match client.post_path(path, content_type, body) {
        Ok((request, response)) => {
            if request.method != "POST" || request.path != path {
                return Err(ClientError::BadPath);
            }
            Ok(Some(response))
        }
        Err(ClientError::Http(_)) => Ok(None),
        Err(err) => Err(err),
    }
}

enum ProbeBody {
    Hit(String),
    Miss,
}

fn read_probe<G: ExplorerGet>(
    client: &ExplorerClient<G>,
    path: &str,
) -> Result<ProbeBody, ClientError> {
    match client.get_path(path) {
        Ok((request, body)) => {
            if request.method != "GET" || request.path != path {
                return Err(ClientError::BadPath);
            }
            Ok(ProbeBody::Hit(body))
        }
        Err(ClientError::Http(message)) if message.starts_with("missing ") => Ok(ProbeBody::Miss),
        Err(err) => Err(err),
    }
}

enum HexHit {
    Tx(String),
    Block(String),
    Script(String),
    Asset {
        name: String,
        ticker: String,
        asset_id: String,
        txs: String,
        chain: String,
        mempool: String,
        supply: String,
        supply_decimal: String,
        stats: String,
        issuance_txid: String,
    },
    Sentence(String),
    NotServed,
    Miss,
}

fn asset_name_ticker(body: &str) -> Option<(String, String)> {
    let value: serde_json::Value = serde_json::from_str(body).ok()?;
    let name = value.get("name")?.as_str()?.to_string();
    let ticker = value.get("ticker")?.as_str()?.to_string();
    if name.is_empty() || ticker.is_empty() {
        return None;
    }
    Some((name, ticker))
}

fn asset_document_id(body: &str) -> Option<String> {
    let value: serde_json::Value = serde_json::from_str(body).ok()?;
    let id = value.get("asset_id")?.as_str()?;
    if id.is_empty() {
        None
    } else {
        Some(id.to_string())
    }
}

fn classify_hex<G: ExplorerGet>(
    client: &ExplorerClient<G>,
    token: &str,
) -> Result<HexHit, ClientError> {
    if let ProbeBody::Hit(body) = read_probe(client, &tx_path(token))? {
        if indexer_sentence(&body).is_none() && parse_tx(&body).is_ok() {
            return Ok(HexHit::Tx(token.to_string()));
        }
    }
    if let ProbeBody::Hit(body) = read_probe(client, &block_path(token))? {
        if indexer_sentence(&body).is_none() && parse_block(&body).is_ok() {
            return Ok(HexHit::Block(token.to_string()));
        }
    }
    if client.network() == Network::Liquid {
        let path = asset_path(token);
        if let ProbeBody::Hit(body) = read_probe(client, &path)? {
            if indexer_sentence(&body).is_some_and(|sentence| sentence == "endpoint does not exist")
            {
                return Ok(HexHit::NotServed);
            }
            if indexer_sentence(&body).is_none() {
                let named = asset_name_ticker(&body);
                // A pegged asset has chain stats and no name. An issued asset
                // may have a name. Either document is this route.
                if named.is_some() || one_registry_asset(&body).is_some() {
                    let (name, ticker) = named.unwrap_or_else(|| (String::new(), String::new()));
                    let (stats, issuance_txid) = asset_stats_lines(&body);
                    let txs_path = asset_txs_path(token);
                    let txs = match read_probe(client, &txs_path)? {
                        ProbeBody::Hit(txs) if indexer_sentence(&txs).is_none() => txs,
                        _ => String::new(),
                    };
                    let asset_id = asset_document_id(&body).unwrap_or_else(|| token.to_string());
                    return Ok(HexHit::Asset {
                        name,
                        ticker,
                        asset_id,
                        txs,
                        chain: optional_text(client, &asset_txs_chain_path(token))?,
                        mempool: optional_text(client, &asset_txs_mempool_path(token))?,
                        supply: optional_text(client, &asset_supply_path(token))?,
                        supply_decimal: optional_text(client, &asset_supply_decimal_path(token))?,
                        stats,
                        issuance_txid,
                    });
                }
            }
        }
    }
    let path = scripthash_path(token);
    match read_probe(client, &path)? {
        ProbeBody::Hit(body) => {
            if let Some(sentence) = indexer_sentence(&body) {
                return Ok(HexHit::Sentence(sentence.to_string()));
            }
            if parse_scripthash(&body).is_ok() {
                return Ok(HexHit::Script(token.to_string()));
            }
            Ok(HexHit::Miss)
        }
        ProbeBody::Miss => Ok(HexHit::Miss),
    }
}

fn broadcast_parts(raw: &str) -> Vec<String> {
    raw.split([',', '\n', '\r', '\t', ' '])
        .filter(|part| !part.is_empty())
        .map(str::to_string)
        .collect()
}

fn page_index_in_row(row: &str) -> Option<&str> {
    let token = row.strip_prefix("row page ")?.split_whitespace().next()?;
    if is_numeric_id(token) {
        Some(token)
    } else {
        None
    }
}

fn chain_cursor_in_row(row: &str) -> Option<&str> {
    let token = row.strip_prefix("row chain ")?.split_whitespace().next()?;
    if token.is_empty() || token.contains('/') {
        None
    } else {
        Some(token)
    }
}

fn block_hash_in_row(row: &str) -> Option<&str> {
    if let Some(rest) = row.strip_prefix("row previous_hash ") {
        return rest.split_whitespace().next();
    }
    if let Some(rest) = row.strip_prefix("row confirming ") {
        return rest.split_whitespace().next();
    }
    if let Some(rest) = row.strip_prefix("row next_best ") {
        return rest.split_whitespace().next();
    }
    if let Some(rest) = row.strip_prefix("row block ") {
        let rest = rest.split_once(" hash ")?.1;
        return rest.split_whitespace().next();
    }
    None
}

fn bare_height_in_row(row: &str) -> Option<&str> {
    if row.contains(" hash ") {
        return None;
    }
    let token = row
        .strip_prefix("row block height ")?
        .split_whitespace()
        .next()?;
    if is_numeric_id(token) {
        Some(token)
    } else {
        None
    }
}

fn mempool_page_in_row(row: &str) -> Option<&str> {
    let token = row
        .strip_prefix("row mempool page ")?
        .split_whitespace()
        .next()?;
    if token.is_empty() || token.contains('/') {
        None
    } else {
        Some(token)
    }
}

fn tip_hash_in_row(row: &str) -> Option<&str> {
    row.strip_prefix("row tip hash ")?.split_whitespace().next()
}

fn merkle_hash_in_row(row: &str) -> Option<&str> {
    let token = row.strip_prefix("row merkle ")?.split_whitespace().next()?;
    if token.is_empty() || token.contains('/') {
        None
    } else {
        Some(token)
    }
}

fn address_in_row(row: &str) -> Option<&str> {
    let rest = if let Some(rest) = row.strip_prefix("row input address ") {
        rest
    } else if let Some(rest) = row.strip_prefix("row output address ") {
        rest
    } else if let Some(rest) = row.strip_prefix("row prefix ") {
        rest
    } else if let Some(rest) = row.strip_prefix("row entered ") {
        rest
    } else {
        row.strip_prefix("row script ")?
    };
    let token = rest.split_whitespace().next()?;
    if token.is_empty() || token == "funded_sum" {
        None
    } else {
        Some(token)
    }
}

fn yes_no(value: bool) -> &'static str {
    if value { "true" } else { "false" }
}

fn height_text(height: Option<u64>) -> String {
    height.map(|value| value.to_string()).unwrap_or_default()
}

/// Satoshis when the transaction JSON has a value. A missing amount stays blank.
fn amount_text(value: Option<u64>) -> String {
    value.map(|amount| amount.to_string()).unwrap_or_default()
}

/// Confidential is the label for a missing value that still has a commitment.
/// A published value stays that number. A missing value with no commitment
/// stays blank. The commitment hex is not an amount.
fn blinded_value_text(output: &splora_frontend_shared::wire::TxOutput) -> String {
    let commitment = output
        .valuecommitment
        .as_deref()
        .is_some_and(|text| !text.is_empty())
        || output
            .assetcommitment
            .as_deref()
            .is_some_and(|text| !text.is_empty());
    if output.value.is_none() && commitment {
        "Confidential".to_string()
    } else {
        amount_text(output.value)
    }
}

/// Same amount rule as [`blinded_value_text`], for a UTXO rather than an output.
fn utxo_amount_text(utxo: &splora_frontend_shared::wire::Utxo) -> String {
    let commitment = utxo
        .valuecommitment
        .as_deref()
        .is_some_and(|text| !text.is_empty())
        || utxo
            .assetcommitment
            .as_deref()
            .is_some_and(|text| !text.is_empty());
    if utxo.value.is_none() && commitment {
        "Confidential".to_string()
    } else {
        amount_text(utxo.value)
    }
}

/// Decimal text for a JSON number. Whole values stay integers (`3`, not `3.0`).
fn plain_number(value: f64) -> String {
    if value.is_finite() && value.fract() == 0.0 {
        format!("{}", value as i64)
    } else {
        format!("{value}")
    }
}

/// Pages of 25. Zero transactions is not a page.
fn tx_page_count(tx_count: u32) -> u32 {
    if tx_count == 0 {
        0
    } else {
        tx_count.div_ceil(25)
    }
}

/// Phone window: at most three page numbers. `page` and the results are 1-based.
fn phone_page_numbers(page: u32, page_count: u32) -> Vec<u32> {
    const MAX_SIZE: u32 = 3;
    if page_count == 0 {
        return Vec::new();
    }
    let page = page.clamp(1, page_count);
    let mut start = 0u32;
    let mut end = page_count;
    if page_count > MAX_SIZE {
        let left_offset = MAX_SIZE / 2;
        let right_offset = left_offset;
        if page <= left_offset {
            end = MAX_SIZE;
        } else if page_count - page < left_offset {
            start = page_count - MAX_SIZE;
        } else {
            start = page - left_offset - 1;
            end = page + right_offset;
        }
    }
    (start..end).map(|index| index + 1).collect()
}

/// First, previous, the visible page numbers, next, and last.
/// Five pages paint all five numbers, nine children.
/// Every other page count uses the phone window of at most three numbers.
fn phone_pager_lines(tx_count: u32, start_index: &str) -> Vec<String> {
    let pages = tx_page_count(tx_count);
    let current_start = if start_index.is_empty() {
        0
    } else {
        start_index.parse::<u32>().unwrap_or(0)
    };
    let active = if pages == 0 {
        1
    } else {
        (current_start / 25 + 1).clamp(1, pages)
    };
    let numbers = if pages == 5 {
        (1..=pages).collect()
    } else {
        phone_page_numbers(active, pages)
    };
    let children = if pages == 0 {
        0
    } else {
        numbers.len() as u32 + 4
    };
    let mut lines = vec![format!("pagination children {children}")];
    if pages == 0 {
        return lines;
    }
    lines.push("pagination control first".to_string());
    lines.push("pagination control previous".to_string());
    for number in numbers {
        let start = (number - 1) * 25;
        if number == active {
            lines.push(format!(
                "pagination control page {number} start {start} active"
            ));
        } else {
            lines.push(format!("pagination control page {number} start {start}"));
        }
    }
    lines.push("pagination control next".to_string());
    lines.push("pagination control last".to_string());
    lines
}

fn render_block_line(block: &splora_frontend_shared::wire::Block) -> String {
    // A missing key is None. Some(0) still prints. Do not substitute zero.
    let mut line = format!(
        "block hash {} height {} tx_count {} timestamp {} size {} weight {} previous_hash {} median_time {} version {} merkle_root {}",
        block.id,
        block.height,
        block.tx_count,
        block.timestamp,
        block.size,
        block.weight,
        block.previous_block_hash.as_deref().unwrap_or(""),
        block.median_time,
        block.version,
        block.merkle_root,
    );
    if let Some(nonce) = block.nonce {
        line.push_str(&format!(" nonce {nonce}"));
    }
    if let Some(bits) = block.bits {
        line.push_str(&format!(" bits {bits}"));
    }
    if let Some(difficulty) = block.difficulty {
        line.push_str(" difficulty ");
        line.push_str(&plain_number(difficulty));
    }
    line
}

fn render_block_list(body: &str) -> Result<String, ClientError> {
    let rows = serde_json::from_str::<Vec<splora_frontend_shared::wire::Block>>(body)
        .map_err(http_parse)?;
    if rows.is_empty() {
        return Ok("blocks:".to_string());
    }
    Ok(rows
        .iter()
        .map(render_block_line)
        .collect::<Vec<_>>()
        .join("\n"))
}

/// One row per block. Height, hash (`BlockRow.id`), tx count, and time.
/// Not the flat field sentence.
fn paint_block_row(row: &BlockRow) -> String {
    format!(
        "row block height {} hash {} tx_count {} time {}",
        row.height, row.id, row.tx_count, row.timestamp
    )
}

fn paint_tx_row(row: &TxRow) -> String {
    format!("row tx txid {} fee {}", row.txid, row.fee)
}

fn block_row_lines(body: &str) -> Result<Vec<String>, ClientError> {
    let blocks = serde_json::from_str::<Vec<splora_frontend_shared::wire::Block>>(body)
        .map_err(http_parse)?;
    Ok(blocks
        .into_iter()
        .map(|block| paint_block_row(&BlockRow::from(block)))
        .collect())
}

/// Recent mempool entries carry txid and fee. They have no version, weight, or block height.
fn tx_row_from_recent(tx: splora_frontend_shared::wire::RecentTransaction) -> TxRow {
    TxRow {
        txid: tx.txid,
        version: 0,
        lock_time: 0,
        size: 0,
        weight: 0,
        sigops: 0,
        fee: tx.fee,
        confirmed: false,
        block_height: None,
        block_hash: None,
        block_time: None,
    }
}

fn recent_row_lines(body: &str) -> Result<Vec<String>, ClientError> {
    let txs = serde_json::from_str::<Vec<splora_frontend_shared::wire::RecentTransaction>>(body)
        .map_err(http_parse)?;
    Ok(splora_frontend_shared::cap_recent_transactions(&txs)
        .iter()
        .cloned()
        .map(|tx| paint_tx_row(&tx_row_from_recent(tx)))
        .collect())
}

fn transaction_row_lines(body: &str) -> Result<Vec<String>, ClientError> {
    let txs = serde_json::from_str::<Vec<splora_frontend_shared::wire::Transaction>>(body)
        .map_err(http_parse)?;
    Ok(txs
        .into_iter()
        .map(|tx| paint_tx_row(&TxRow::from(tx)))
        .collect())
}

fn utxo_value_rows(body: &str) -> Result<Vec<String>, ClientError> {
    let utxos = serde_json::from_str::<Vec<splora_frontend_shared::wire::Utxo>>(body)
        .map_err(http_parse)?;
    Ok(utxos
        .iter()
        .map(|utxo| format!("row utxo value {}", utxo_amount_text(utxo)))
        .collect())
}

fn present_sum(sum: Option<u64>) -> String {
    match sum {
        Some(sum) => sum.to_string(),
        None => String::new(),
    }
}

/// Chain funded minus chain spent. A missing sum is not zero. The difference is signed.
fn confirmed_balance_words(funded: Option<u64>, spent: Option<u64>) -> Option<String> {
    match (funded, spent) {
        (Some(funded), Some(spent)) => {
            let balance = funded as i128 - spent as i128;
            Some(format!("Confirmed balance {balance}"))
        }
        _ => None,
    }
}

/// A JSON number, including zero. A missing key, null, or non-number is not a count.
fn json_u64_at(value: &serde_json::Value, key: &str) -> Option<u64> {
    match value.get(key) {
        Some(serde_json::Value::Number(number)) => number.as_u64(),
        _ => None,
    }
}

fn chain_side(body: &str) -> Option<serde_json::Value> {
    let value: serde_json::Value = serde_json::from_str(body).ok()?;
    value.get("chain_stats").cloned()
}

fn mempool_side(body: &str) -> Option<serde_json::Value> {
    let value: serde_json::Value = serde_json::from_str(body).ok()?;
    value.get("mempool_stats").cloned()
}

/// Either chain count key is absent. Mempool counts do not count.
fn chain_count_absent(body: &str) -> bool {
    let Some(chain) = chain_side(body) else {
        return false;
    };
    json_u64_at(&chain, "funded_txo_count").is_none()
        || json_u64_at(&chain, "spent_txo_count").is_none()
}

/// Either mempool count key is absent. Chain counts do not count.
fn mempool_count_absent(body: &str) -> bool {
    let Some(mempool) = mempool_side(body) else {
        return false;
    };
    json_u64_at(&mempool, "funded_txo_count").is_none()
        || json_u64_at(&mempool, "spent_txo_count").is_none()
}

/// Either tx_count key is absent. A missing key is not zero.
fn tx_count_absent(body: &str) -> bool {
    let Some(chain) = chain_side(body) else {
        return false;
    };
    let Some(mempool) = mempool_side(body) else {
        return false;
    };
    json_u64_at(&chain, "tx_count").is_none() || json_u64_at(&mempool, "tx_count").is_none()
}

/// Chain funded count minus chain spent count. A missing count is not zero.
/// The difference is signed. Mempool counts are not this line.
fn confirmed_utxo_words(body: &str) -> Option<String> {
    let chain = chain_side(body)?;
    let funded = json_u64_at(&chain, "funded_txo_count")?;
    let spent = json_u64_at(&chain, "spent_txo_count")?;
    let count = i128::from(funded) - i128::from(spent);
    Some(format!("Confirmed UTXOs {count}"))
}

/// Chain funded sum. A missing sum is not this line. This is not the balance.
fn total_received_words(funded: Option<u64>) -> Option<String> {
    funded.map(|sum| format!("Total received {sum}"))
}

/// Chain spent plus mempool spent. A missing sum is not this line.
/// The sum is signed so two `u64` values do not wrap.
fn total_sent_words(chain_spent: Option<u64>, mempool_spent: Option<u64>) -> Option<String> {
    match (chain_spent, mempool_spent) {
        (Some(chain_spent), Some(mempool_spent)) => {
            let sum = chain_spent as i128 + mempool_spent as i128;
            Some(format!("Total sent {sum}"))
        }
        _ => None,
    }
}

/// Mempool funded count minus mempool spent count. A missing count is not zero.
/// The difference is signed. Chain counts are not this line.
fn pending_utxo_words(body: &str) -> Option<String> {
    let mempool = mempool_side(body)?;
    let funded = json_u64_at(&mempool, "funded_txo_count")?;
    let spent = json_u64_at(&mempool, "spent_txo_count")?;
    let count = i128::from(funded) - i128::from(spent);
    Some(format!("Pending UTXOs {count}"))
}

/// Chain tx count plus mempool tx count. A missing count is not zero.
/// The sum is signed so two `u64` values do not wrap.
fn transaction_count_words(body: &str) -> Option<String> {
    let chain = chain_side(body)?;
    let mempool = mempool_side(body)?;
    let chain_count = json_u64_at(&chain, "tx_count")?;
    let mempool_count = json_u64_at(&mempool, "tx_count")?;
    let sum = chain_count as i128 + mempool_count as i128;
    Some(format!("Transactions {sum}"))
}

/// Mempool funded sum minus mempool spent sum. A missing sum is not zero.
/// The difference is signed so two `u64` values do not wrap.
/// This is not Pending UTXOs and not the confirmed balance.
fn pending_amount_words(funded: Option<u64>, spent: Option<u64>) -> Option<String> {
    match (funded, spent) {
        (Some(funded), Some(spent)) => {
            let amount = i128::from(funded) - i128::from(spent);
            Some(format!("Pending {amount}"))
        }
        _ => None,
    }
}

/// Chain funded plus mempool funded, minus chain spent plus mempool spent.
/// A missing sum is not zero. The difference is signed so the sums do not wrap.
/// This is not Confirmed balance and not Pending.
fn combined_balance_words(
    chain_funded: Option<u64>,
    mempool_funded: Option<u64>,
    chain_spent: Option<u64>,
    mempool_spent: Option<u64>,
) -> Option<String> {
    match (chain_funded, mempool_funded, chain_spent, mempool_spent) {
        (Some(chain_funded), Some(mempool_funded), Some(chain_spent), Some(mempool_spent)) => {
            let balance = i128::from(chain_funded) + i128::from(mempool_funded)
                - i128::from(chain_spent)
                - i128::from(mempool_spent);
            Some(format!("Balance {balance}"))
        }
        _ => None,
    }
}

fn show_num(value: Option<u64>) -> String {
    value.map(|number| number.to_string()).unwrap_or_default()
}

/// Address line when a chain or mempool count key is absent. The missing count stays blank.
fn lenient_address_stats_line(body: &str) -> Result<String, ClientError> {
    let value: serde_json::Value = serde_json::from_str(body).map_err(http_parse)?;
    let address = value
        .get("address")
        .and_then(|item| item.as_str())
        .or_else(|| value.get("scripthash").and_then(|item| item.as_str()))
        .unwrap_or("");
    let chain = value
        .get("chain_stats")
        .cloned()
        .unwrap_or(serde_json::Value::Null);
    let mempool = value
        .get("mempool_stats")
        .cloned()
        .unwrap_or(serde_json::Value::Null);
    let funded_sum = json_u64_at(&chain, "funded_txo_sum");
    let spent_sum = json_u64_at(&chain, "spent_txo_sum");
    let mempool_spent = json_u64_at(&mempool, "spent_txo_sum");
    let mut line = format!(
        "address {address} chain_tx_count {} funded_sum {} mempool_tx_count {} mempool_funded_sum {} chain_funded_txo_count {} chain_spent_txo_count {} chain_spent_txo_sum {} mempool_funded_txo_count {} mempool_spent_txo_count {} mempool_spent_txo_sum {}",
        show_num(json_u64_at(&chain, "tx_count")),
        show_num(funded_sum),
        show_num(json_u64_at(&mempool, "tx_count")),
        show_num(json_u64_at(&mempool, "funded_txo_sum")),
        show_num(json_u64_at(&chain, "funded_txo_count")),
        show_num(json_u64_at(&chain, "spent_txo_count")),
        show_num(spent_sum),
        show_num(json_u64_at(&mempool, "funded_txo_count")),
        show_num(json_u64_at(&mempool, "spent_txo_count")),
        show_num(mempool_spent),
    );
    if let Some(words) = confirmed_balance_words(funded_sum, spent_sum) {
        line.push(' ');
        line.push_str(&words);
    }
    if let Some(words) = confirmed_utxo_words(body) {
        line.push(' ');
        line.push_str(&words);
    }
    if let Some(words) = total_received_words(funded_sum) {
        line.push(' ');
        line.push_str(&words);
    }
    if let Some(words) = total_sent_words(spent_sum, mempool_spent) {
        line.push(' ');
        line.push_str(&words);
    }
    if let Some(words) = pending_utxo_words(body) {
        line.push(' ');
        line.push_str(&words);
    }
    if let Some(words) = transaction_count_words(body) {
        line.push(' ');
        line.push_str(&words);
    }
    if let Some(words) = pending_amount_words(
        json_u64_at(&mempool, "funded_txo_sum"),
        json_u64_at(&mempool, "spent_txo_sum"),
    ) {
        line.push(' ');
        line.push_str(&words);
    }
    if let Some(words) = combined_balance_words(
        funded_sum,
        json_u64_at(&mempool, "funded_txo_sum"),
        spent_sum,
        mempool_spent,
    ) {
        line.push(' ');
        line.push_str(&words);
    }
    Ok(line)
}

fn address_funded_row(body: &str) -> Result<String, ClientError> {
    let sum = match serde_json::from_str::<splora_frontend_shared::wire::AddressStats>(body) {
        Ok(stats) => stats.chain_stats.funded_txo_sum,
        Err(err) => {
            if !chain_count_absent(body) && !mempool_count_absent(body) && !tx_count_absent(body) {
                return Err(http_parse(err));
            }
            let value: serde_json::Value = serde_json::from_str(body).map_err(http_parse)?;
            let chain = value
                .get("chain_stats")
                .cloned()
                .unwrap_or(serde_json::Value::Null);
            json_u64_at(&chain, "funded_txo_sum")
        }
    };
    Ok(format!("row address funded_sum {}", present_sum(sum)))
}

fn render_recent(body: &str) -> Result<String, ClientError> {
    let rows = serde_json::from_str::<Vec<splora_frontend_shared::wire::RecentTransaction>>(body)
        .map_err(http_parse)?;
    let rows = splora_frontend_shared::cap_recent_transactions(&rows);
    if rows.is_empty() {
        return Ok("recent:".to_string());
    }
    Ok(rows
        .iter()
        .map(|tx| {
            format!(
                "recent txid {} fee {} vsize {} value {}{}",
                tx.txid,
                tx.fee,
                tx.vsize,
                splora_frontend_shared::wire::recent_value_text(tx.value, ""),
                fee_rate_suffix(sat_per_vbyte_from_vsize(tx.fee, tx.vsize))
            )
        })
        .collect::<Vec<_>>()
        .join("\n"))
}

fn render_fee_estimates(body: &str) -> Result<String, ClientError> {
    let estimates = serde_json::from_str::<splora_frontend_shared::wire::FeeEstimates>(body)
        .map_err(http_parse)?;
    if estimates.0.is_empty() {
        return Ok("fee:".to_string());
    }
    Ok(estimates
        .0
        .iter()
        .map(|(target, rate)| format!("fee target {target} rate {}", plain_number(*rate)))
        .collect::<Vec<_>>()
        .join("\n"))
}

fn render_mempool_summary(body: &str) -> Result<String, ClientError> {
    let stats = serde_json::from_str::<splora_frontend_shared::wire::MempoolSummary>(body)
        .map_err(http_parse)?;
    let mut lines = vec![format!(
        "mempool count {} vsize {} total_fee {}",
        stats.count, stats.vsize, stats.total_fee
    )];
    for (rate, vsize) in stats.fee_histogram {
        lines.push(format!(
            "fee_histogram {} {vsize}",
            plain_number(f64::from(rate))
        ));
    }
    lines.push("The incoming virtual-bytes-per-second rate is not in the payload.".to_string());
    Ok(lines.join("\n"))
}

fn render_txids(body: &str) -> Result<String, ClientError> {
    let txids = parse_block_txids(body).map_err(http_parse)?;
    if txids.is_empty() {
        return Ok("The txid list is empty.".to_string());
    }
    Ok(txids
        .iter()
        .map(|txid| format!("txid {txid}"))
        .collect::<Vec<_>>()
        .join("\n"))
}

fn tx_confirmed_and_height(
    tx: &splora_frontend_shared::wire::Transaction,
) -> (&'static str, String) {
    match tx.status.as_ref() {
        Some(status) => (yes_no(status.confirmed), height_text(status.block_height)),
        None => ("false", String::new()),
    }
}

fn block_hash_and_time(status: Option<&splora_frontend_shared::wire::TxStatus>) -> String {
    match status {
        Some(status) => format!(
            " block_hash {} block_time {}",
            status.block_hash.as_deref().unwrap_or(""),
            status
                .block_time
                .map(|time| time.to_string())
                .unwrap_or_default()
        ),
        None => " block_hash  block_time ".to_string(),
    }
}

fn fee_rate_suffix(rate: Option<f64>) -> String {
    match rate {
        Some(rate) => format!(" fee_rate {}", plain_number(rate)),
        None => String::new(),
    }
}

fn tx_extra_fields(tx: &splora_frontend_shared::wire::Transaction) -> String {
    format!(
        " version {} locktime {} sigops {}{}",
        tx.version,
        tx.lock_time,
        tx.sigops,
        fee_rate_suffix(sat_per_vbyte_from_weight(tx.fee, tx.weight))
    )
}

fn render_tx_parties(
    tx: &splora_frontend_shared::wire::Transaction,
    vin_json: &[serde_json::Value],
) -> Vec<String> {
    let mut lines = Vec::new();
    for (index, input) in tx.vin.iter().enumerate() {
        lines.push(format!("input outpoint {}:{}", input.txid, input.vout));
        if let Some(prevout) = &input.prevout {
            let address = prevout.script_pubkey_address.as_deref().unwrap_or("");
            lines.push(format!(
                "input address {address} value {}",
                blinded_value_text(prevout)
            ));
            push_script_type(&mut lines, "input", &prevout.script_pubkey_type);
        }
        let raw = vin_json.get(index);
        push_input_coinbase(&mut lines, raw);
        push_input_sequence(&mut lines, raw);
        if let Some(issuance) = &input.issuance {
            lines.extend(issuance_lines(issuance));
        }
    }
    for output in &tx.vout {
        let address = output.script_pubkey_address.as_deref().unwrap_or("");
        lines.push(format!(
            "output address {address} value {}",
            blinded_value_text(output)
        ));
        push_script_type(&mut lines, "output", &output.script_pubkey_type);
    }
    lines
}

/// `scriptpubkey_type` as the indexer spelled it. An empty string is omitted.
/// The label does not repeat the script bytes or the asm.
fn push_script_type(lines: &mut Vec<String>, side: &str, script_type: &str) {
    if script_type.is_empty() {
        return;
    }
    lines.push(format!("{side} script type {script_type}"));
}

/// Shared `is_coinbase` is a bool, so a missing key is not false.
/// Only JSON `true` paints. False, null, and a missing key stay off the screen.
fn push_input_coinbase(lines: &mut Vec<String>, raw: Option<&serde_json::Value>) {
    let Some(raw) = raw else {
        return;
    };
    if raw.get("is_coinbase").and_then(|value| value.as_bool()) == Some(true) {
        lines.push("input coinbase".to_string());
    }
}

/// Shared `sequence` is `u32`, so a missing key is not zero.
/// A present integer paints, including zero. A missing key omits the label.
fn push_input_sequence(lines: &mut Vec<String>, raw: Option<&serde_json::Value>) {
    let Some(raw) = raw else {
        return;
    };
    let Some(sequence) = json_u64_at(raw, "sequence") else {
        return;
    };
    lines.push(format!("sequence {sequence}"));
}

fn vin_objects(value: &serde_json::Value) -> Vec<serde_json::Value> {
    value
        .get("vin")
        .and_then(|vin| vin.as_array())
        .cloned()
        .unwrap_or_default()
}

/// Shared `Transaction.weight` is `u32`, so a missing key or JSON null cannot load.
/// The copy lets the rest of the transaction parse. Virtual size, the Weight row,
/// and the header weight read the original document and do not use this placeholder.
fn serde_weight_for_shared_type(value: &serde_json::Value) -> serde_json::Value {
    let missing = matches!(value.get("weight"), None | Some(serde_json::Value::Null));
    if !missing {
        return value.clone();
    }
    let mut copy = value.clone();
    if let Some(object) = copy.as_object_mut() {
        object.insert("weight".to_string(), serde_json::Value::from(0_u32));
    }
    copy
}

/// Shared `Transaction.sigops` is `u32`, so a missing key or JSON null cannot load.
/// The copy lets the rest of the transaction parse. Adjusted vsize and the Sigops
/// label read the original document and do not use this placeholder.
fn serde_sigops_for_shared_type(value: &serde_json::Value) -> serde_json::Value {
    let missing = matches!(value.get("sigops"), None | Some(serde_json::Value::Null));
    if !missing {
        return value.clone();
    }
    let mut copy = value.clone();
    if let Some(object) = copy.as_object_mut() {
        object.insert("sigops".to_string(), serde_json::Value::from(0_u32));
    }
    copy
}

/// Shared `TxInput` cannot load when `is_coinbase` or `sequence` is absent.
/// The copy lets the rest of the transaction parse. Paint reads the original
/// object and does not use this copy.
fn transaction_keeping_absent_vin_keys(
    value: &serde_json::Value,
) -> Result<splora_frontend_shared::wire::Transaction, ClientError> {
    match serde_json::from_value(value.clone()) {
        Ok(tx) => Ok(tx),
        Err(err) => {
            if !vin_omits_coinbase_or_sequence(value) {
                return Err(http_parse(err));
            }
            serde_json::from_value(vin_keys_for_shared_type(value)).map_err(http_parse)
        }
    }
}

fn vin_omits_coinbase_or_sequence(value: &serde_json::Value) -> bool {
    let Some(vins) = value.get("vin").and_then(|vin| vin.as_array()) else {
        return false;
    };
    vins.iter().any(|vin| {
        let Some(object) = vin.as_object() else {
            return false;
        };
        !object.contains_key("is_coinbase") || !object.contains_key("sequence")
    })
}

fn vin_keys_for_shared_type(value: &serde_json::Value) -> serde_json::Value {
    let mut copy = value.clone();
    let Some(vins) = copy.get_mut("vin").and_then(|vin| vin.as_array_mut()) else {
        return copy;
    };
    for vin in vins {
        let Some(object) = vin.as_object_mut() else {
            continue;
        };
        object
            .entry("is_coinbase")
            .or_insert(serde_json::Value::Bool(false));
        object
            .entry("sequence")
            .or_insert(serde_json::Value::from(0_u32));
    }
    copy
}

/// Fields from `TxInput.issuance`, after that input's prevout line.
///
/// A missing amount is not zero. A commitment hex is not an amount and is
/// not painted. An empty string omits that line. `is_reissuance` is always
/// painted when the object is present.
fn issuance_lines(issuance: &splora_frontend_shared::wire::Issuance) -> Vec<String> {
    let mut lines = Vec::new();
    if !issuance.asset_id.is_empty() {
        lines.push(format!("issuance asset {}", issuance.asset_id));
    }
    lines.push(format!(
        "issuance reissuance {}",
        yes_no(issuance.is_reissuance)
    ));
    if let Some(hash) = issuance
        .contract_hash
        .as_deref()
        .filter(|hash| !hash.is_empty())
    {
        lines.push(format!("issuance contract {hash}"));
    }
    if !issuance.asset_entropy.is_empty() {
        lines.push(format!("issuance entropy {}", issuance.asset_entropy));
    }
    if let Some(nonce) = issuance
        .asset_blinding_nonce
        .as_deref()
        .filter(|nonce| !nonce.is_empty())
    {
        lines.push(format!("issuance blinding nonce {nonce}"));
    }
    if let Some(line) = issuance_amount_line(
        "asset",
        issuance.assetamount,
        issuance.assetamountcommitment.as_deref(),
    ) {
        lines.push(line);
    }
    if let Some(line) = issuance_amount_line(
        "token",
        issuance.tokenamount,
        issuance.tokenamountcommitment.as_deref(),
    ) {
        lines.push(line);
    }
    lines
}

fn issuance_amount_line(
    kind: &str,
    amount: Option<u64>,
    commitment: Option<&str>,
) -> Option<String> {
    if let Some(amount) = amount {
        return Some(format!("issuance {kind} amount {amount}"));
    }
    if commitment.is_some_and(|text| !text.is_empty()) {
        return Some(format!("issuance {kind} amount Confidential"));
    }
    None
}

/// Sum of output `value` fields. Every present value is added, including zero.
/// An empty output list is 0. A missing value omits the total and is not zero.
/// The add is `u128` so two `u64::MAX` values do not wrap. A commitment is not an amount.
fn output_total_words(tx: &splora_frontend_shared::wire::Transaction) -> Option<String> {
    let mut sum: u128 = 0;
    for output in &tx.vout {
        let Some(value) = output.value else {
            return None;
        };
        sum += u128::from(value);
    }
    Some(format!("output total {sum}"))
}

/// Details fee. Angular `number` pipe, then `sats`. Explicit 0 paints.
/// Thousands separators, no fraction digits. The raw header `fee N` stays ungrouped.
fn fee_words(fee: u64) -> String {
    format!("Fee {} sats", grouped_count(fee))
}

/// en-US integer grouping. `1,500` not `1500`. No fraction digits.
fn grouped_count(value: u64) -> String {
    let digits = value.to_string();
    let mut reversed = String::with_capacity(digits.len() + digits.len() / 3);
    for (index, ch) in digits.chars().rev().enumerate() {
        if index > 0 && index.is_multiple_of(3) {
            reversed.push(',');
        }
        reversed.push(ch);
    }
    reversed.chars().rev().collect()
}

/// Transaction size on the details row. The Angular `bytes` pipe with 2 decimals.
/// Powers of 1000, chosen before rounding. Not 1024. The raw header `size N` stays.
fn size_words(bytes: u32) -> String {
    format!("Size {}", bytes_pipe_text(u64::from(bytes)))
}

/// Plaintext `bytes: 2`: the number, a space, and the unit. Trailing zeros drop.
/// Divide in f64, then `(value * 100).round() / 100`. The unit is chosen from
/// the raw count before that round. Powers of 1000, not 1024. Weight stays on
/// the integer scaler so 1005 is not the weight contract.
fn bytes_pipe_text(bytes: u64) -> String {
    let (scale, unit): (f64, &str) = if bytes < 1_000 {
        (1.0, "B")
    } else if bytes < 1_000_000 {
        (1_000.0, "kB")
    } else if bytes < 1_000_000_000 {
        (1_000_000.0, "MB")
    } else if bytes < 1_000_000_000_000 {
        (1_000_000_000.0, "GB")
    } else {
        (1_000_000_000_000.0, "TB")
    };
    let value = (bytes as f64) / scale;
    let hundredths = (value * 100.0).round() as u64;
    format!("{} {unit}", js_number_from_hundredths(hundredths))
}

/// Plaintext `wuBytes: 2`. Same 1000 thresholds as size. Integer half-up, not the size f64 round.
fn wu_bytes_pipe_text(weight: u64) -> String {
    scaled_count_text(weight, ["WU", "kWU", "MWU", "GWU", "TWU"])
}

/// Powers of 1000, chosen from the raw count before rounding. Not 1024.
fn scaled_count_text(count: u64, units: [&str; 5]) -> String {
    let (number, unit) = if count < 1_000 {
        (count.to_string(), units[0])
    } else if count < 1_000_000 {
        (
            js_number_from_hundredths(scale_hundredths(count, 1_000)),
            units[1],
        )
    } else if count < 1_000_000_000 {
        (
            js_number_from_hundredths(scale_hundredths(count, 1_000_000)),
            units[2],
        )
    } else if count < 1_000_000_000_000 {
        (
            js_number_from_hundredths(scale_hundredths(count, 1_000_000_000)),
            units[3],
        )
    } else {
        (
            js_number_from_hundredths(scale_hundredths(count, 1_000_000_000_000)),
            units[4],
        )
    };
    format!("{number} {unit}")
}

/// `Math.round(bytes / divisor * 100)` for a positive count, half away from zero.
fn scale_hundredths(bytes: u64, divisor: u64) -> u64 {
    let numerator = u128::from(bytes) * 100;
    let divisor = u128::from(divisor);
    let mut hundredths = numerator / divisor;
    if (numerator % divisor) * 2 >= divisor {
        hundredths += 1;
    }
    u64::try_from(hundredths).unwrap_or(u64::MAX)
}

/// JavaScript `Number.toString` for an integer count of hundredths.
fn js_number_from_hundredths(hundredths: u64) -> String {
    let whole = hundredths / 100;
    let frac = (hundredths % 100) as u8;
    if frac == 0 {
        format!("{whole}")
    } else if frac.is_multiple_of(10) {
        format!("{whole}.{}", frac / 10)
    } else {
        format!("{whole}.{frac:02}")
    }
}

/// Virtual size is `weight / 4` from the original document, integer division.
/// A missing key or JSON null omits the line. A present zero stays zero.
/// `sat_per_vbyte_from_weight` hides a zero quotient, so this does not call it.
fn virtual_size_words(document: &serde_json::Value) -> Option<String> {
    let weight = json_u64_at(document, "weight")?;
    Some(format!("virtual size {}", weight / 4))
}

/// Details weight from the original document. A missing key or JSON null omits
/// the line. The serde placeholder 0 is not used. The raw header `weight N` stays.
fn weight_words(document: &serde_json::Value) -> Option<String> {
    let weight = json_u64_at(document, "weight")?;
    Some(format!("Weight {}", wu_bytes_pipe_text(weight)))
}

/// Details sigops from the original document. Angular `number` pipe, no fraction.
/// A missing key or JSON null omits the line. Explicit 0 paints. The serde
/// placeholder 0 is not used. The raw header `sigops N` stays lowercase.
fn sigops_words(document: &serde_json::Value) -> Option<String> {
    let sigops = json_u64_at(document, "sigops")?;
    Some(format!("Sigops {}", grouped_count(sigops)))
}

/// Adjusted vsize is `max(weight / 4, sigops * 5)` from the original document.
/// The division is floating point, then half away from zero to 2 decimal places.
/// A missing key or JSON null omits the line. The serde placeholder 0 is not used.
/// This is not the integer virtual-size line.
fn adjusted_vsize_words(document: &serde_json::Value) -> Option<String> {
    let weight = json_u64_at(document, "weight")?;
    let sigops = json_u64_at(document, "sigops")?;
    let from_weight = (weight as f64) / 4.0;
    let from_sigops = (sigops as f64) * 5.0;
    Some(format!(
        "Adjusted vsize {}",
        two_decimals_half_away_from_zero(from_weight.max(from_sigops))
    ))
}

/// Two digits after the decimal. Halfway cases go away from zero.
fn two_decimals_half_away_from_zero(value: f64) -> String {
    let negative = value < 0.0;
    let cents = (value.abs() * 100.0).round() as u64;
    let text = format!("{}.{:02}", cents / 100, cents % 100);
    if negative { format!("-{text}") } else { text }
}

/// Tip height minus the transaction block height, plus 1.
/// A missing block height or a missing tip omits the line. An explicit zero
/// height is present. The subtraction is `i128` so it does not wrap.
fn confirmation_words(block_height: Option<u64>, tip_height: Option<u64>) -> Option<String> {
    let block_height = block_height?;
    let tip_height = tip_height?;
    let count = i128::from(tip_height) - i128::from(block_height) + 1;
    Some(format!("confirmations {count}"))
}

fn absent_tip_message(message: &str) -> bool {
    message.starts_with("missing ")
        || message == "tip height was empty"
        || message == "tip height was not an integer"
}

/// `ExplorerClient::tip_height` reads `GET /blocks/tip/height`. A missing route,
/// an empty body, or a non-integer body omits the count. It does not become 0.
fn tip_height_for_confirmations<G: ExplorerGet>(
    client: &ExplorerClient<G>,
) -> Result<Option<u64>, ClientError> {
    match client.tip_height() {
        Ok(text) => Ok(text.parse().ok()),
        Err(ClientError::Http(message)) if absent_tip_message(&message) => Ok(None),
        Err(err) => Err(err),
    }
}

/// Header weight from the original document. A present zero stays `weight 0`.
/// A missing key or JSON null omits the label instead of printing the placeholder.
fn weight_header(weight: Option<u64>) -> String {
    match weight {
        Some(weight) => format!(" weight {weight}"),
        None => String::new(),
    }
}

fn render_tx_row(
    tx: &splora_frontend_shared::wire::Transaction,
    vin_json: &[serde_json::Value],
    document_weight: Option<u64>,
) -> String {
    let (confirmed, block_height) = tx_confirmed_and_height(tx);
    let mut lines = vec![format!(
        "tx txid {} fee {} confirmed {} block_height {} size {}{}{}{}",
        tx.txid,
        tx.fee,
        confirmed,
        block_height,
        tx.size,
        weight_header(document_weight),
        block_hash_and_time(tx.status.as_ref()),
        tx_extra_fields(tx)
    )];
    lines.extend(render_tx_parties(tx, vin_json));
    lines.join("\n")
}

fn render_tx_list(body: &str) -> Result<String, ClientError> {
    let rows = serde_json::from_str::<Vec<splora_frontend_shared::wire::Transaction>>(body)
        .map_err(http_parse)?;
    if rows.is_empty() {
        return Ok("The transaction list is empty.".to_string());
    }
    let values: Vec<serde_json::Value> = serde_json::from_str(body).map_err(http_parse)?;
    Ok(rows
        .iter()
        .enumerate()
        .map(|(index, tx)| {
            let vin = values.get(index).map(vin_objects).unwrap_or_default();
            render_tx_row(tx, &vin, Some(u64::from(tx.weight)))
        })
        .collect::<Vec<_>>()
        .join("\n"))
}

fn render_address_stats(body: &str) -> Result<String, ClientError> {
    let stats = match serde_json::from_str::<splora_frontend_shared::wire::AddressStats>(body) {
        Ok(stats) => stats,
        Err(err) => {
            if chain_count_absent(body) || mempool_count_absent(body) || tx_count_absent(body) {
                return lenient_address_stats_line(body);
            }
            return Err(http_parse(err));
        }
    };
    let address = stats
        .address
        .as_deref()
        .or(stats.scripthash.as_deref())
        .unwrap_or("");
    let mut line = format!(
        "address {} chain_tx_count {} funded_sum {} mempool_tx_count {} mempool_funded_sum {} chain_funded_txo_count {} chain_spent_txo_count {} chain_spent_txo_sum {} mempool_funded_txo_count {} mempool_spent_txo_count {} mempool_spent_txo_sum {}",
        address,
        stats.chain_stats.tx_count,
        present_sum(stats.chain_stats.funded_txo_sum),
        stats.mempool_stats.tx_count,
        present_sum(stats.mempool_stats.funded_txo_sum),
        stats.chain_stats.funded_txo_count,
        stats.chain_stats.spent_txo_count,
        present_sum(stats.chain_stats.spent_txo_sum),
        stats.mempool_stats.funded_txo_count,
        stats.mempool_stats.spent_txo_count,
        present_sum(stats.mempool_stats.spent_txo_sum)
    );
    if let Some(words) = confirmed_balance_words(
        stats.chain_stats.funded_txo_sum,
        stats.chain_stats.spent_txo_sum,
    ) {
        line.push(' ');
        line.push_str(&words);
    }
    if let Some(words) = confirmed_utxo_words(body) {
        line.push(' ');
        line.push_str(&words);
    }
    if let Some(words) = total_received_words(stats.chain_stats.funded_txo_sum) {
        line.push(' ');
        line.push_str(&words);
    }
    if let Some(words) = total_sent_words(
        stats.chain_stats.spent_txo_sum,
        stats.mempool_stats.spent_txo_sum,
    ) {
        line.push(' ');
        line.push_str(&words);
    }
    if let Some(words) = pending_utxo_words(body) {
        line.push(' ');
        line.push_str(&words);
    }
    if let Some(words) = transaction_count_words(body) {
        line.push(' ');
        line.push_str(&words);
    }
    if let Some(words) = pending_amount_words(
        stats.mempool_stats.funded_txo_sum,
        stats.mempool_stats.spent_txo_sum,
    ) {
        line.push(' ');
        line.push_str(&words);
    }
    if let Some(words) = combined_balance_words(
        stats.chain_stats.funded_txo_sum,
        stats.mempool_stats.funded_txo_sum,
        stats.chain_stats.spent_txo_sum,
        stats.mempool_stats.spent_txo_sum,
    ) {
        line.push(' ');
        line.push_str(&words);
    }
    Ok(line)
}

fn render_address_txs(body: &str) -> Result<String, ClientError> {
    let rows = serde_json::from_str::<Vec<splora_frontend_shared::wire::Transaction>>(body)
        .map_err(http_parse)?;
    if rows.is_empty() {
        return Ok("The address transaction list is empty.".to_string());
    }
    let values: Vec<serde_json::Value> = serde_json::from_str(body).map_err(http_parse)?;
    Ok(rows
        .iter()
        .enumerate()
        .map(|(index, tx)| {
            let vin = values.get(index).map(vin_objects).unwrap_or_default();
            let (confirmed, block_height) = tx_confirmed_and_height(tx);
            let mut lines = vec![format!(
                "address tx txid {} confirmed {} block_height {} fee {} size {} weight {}{}{}",
                tx.txid,
                confirmed,
                block_height,
                tx.fee,
                tx.size,
                tx.weight,
                block_hash_and_time(tx.status.as_ref()),
                tx_extra_fields(tx)
            )];
            lines.extend(render_tx_parties(tx, &vin));
            lines.join("\n")
        })
        .collect::<Vec<_>>()
        .join("\n"))
}

fn paint_address_tx_page(
    body: &str,
    lines: &mut Vec<String>,
    rows: &mut Vec<String>,
) -> Result<Option<String>, ClientError> {
    if body.contains("\"tx_position\"") {
        let summaries = parse_address_txs_summary(body).map_err(http_parse)?;
        if summaries.is_empty() {
            lines.push("The address transaction list is empty.".to_string());
            return Ok(None);
        }
        let mut last = None;
        for summary in &summaries {
            rows.push(format!(
                "row summary txid {} height {} value {} time {} tx_position {}",
                summary.txid, summary.height, summary.value, summary.time, summary.tx_position
            ));
            last = Some(summary.txid.clone());
        }
        return Ok(last);
    }
    lines.push(render_address_txs(body)?);
    if body.trim() == "[]" {
        return Ok(None);
    }
    rows.extend(transaction_row_lines(body)?);
    let txs = serde_json::from_str::<Vec<splora_frontend_shared::wire::Transaction>>(body)
        .map_err(http_parse)?;
    Ok(txs.last().map(|tx| tx.txid.clone()))
}

fn render_utxos(body: &str) -> Result<String, ClientError> {
    let rows = serde_json::from_str::<Vec<splora_frontend_shared::wire::Utxo>>(body)
        .map_err(http_parse)?;
    if rows.is_empty() {
        return Ok("The utxo list is empty.".to_string());
    }
    Ok(rows
        .iter()
        .map(|utxo| {
            format!(
                "utxo txid {} vout {} value {} confirmed {} block_height {}{}",
                utxo.txid,
                utxo.vout,
                utxo_amount_text(utxo),
                yes_no(utxo.status.confirmed),
                height_text(utxo.status.block_height),
                block_hash_and_time(Some(&utxo.status))
            )
        })
        .collect::<Vec<_>>()
        .join("\n"))
}

fn push_spend(lines: &mut Vec<String>, rows: &mut Vec<String>, spent: bool, txid: Option<&str>) {
    if !spent {
        lines.push("spent false".to_string());
        return;
    }
    if let Some(txid) = txid.filter(|txid| !txid.is_empty()) {
        lines.push(format!("spend txid {txid}"));
        rows.push(format!("row spend txid {txid}"));
    }
}

fn optional_text<G: ExplorerGet>(
    client: &ExplorerClient<G>,
    path: &str,
) -> Result<String, ClientError> {
    Ok(read_optional(client, path)?.unwrap_or_default())
}

fn append_tx_body(lines: &mut String, body: &str) {
    if body.is_empty() {
        return;
    }
    if let Ok(rendered) = render_tx_list(body) {
        if rendered != "The transaction list is empty." {
            lines.push('\n');
            lines.push_str(&rendered);
        }
    }
}

fn block_details_text(block: &splora_frontend_shared::wire::Block) -> String {
    // A missing key is None. Some(0) still prints. Do not substitute zero.
    let mut text = format!(
        "block details\nid {}\nheight {}\nversion {}\ntimestamp {}\ntx_count {}\nsize {}\nweight {}\nmerkle_root {}\npreviousblockhash {}\nmediantime {}",
        block.id,
        block.height,
        block.version,
        block.timestamp,
        block.tx_count,
        block.size,
        block.weight,
        block.merkle_root,
        block.previous_block_hash.as_deref().unwrap_or(""),
        block.median_time,
    );
    if let Some(nonce) = block.nonce {
        text.push_str(&format!("\nnonce {nonce}"));
    }
    if let Some(bits) = block.bits {
        text.push_str(&format!("\nbits {bits}"));
    }
    if let Some(difficulty) = block.difficulty {
        text.push_str("\ndifficulty ");
        text.push_str(&plain_number(difficulty));
    }
    text
}

/// Peg-in and Liquid Peg Out from the parsed transaction.
///
/// A missing `is_pegin` is `None`, not false. `Some(false)` does not paint a
/// peg-in. A missing `pegout` is none. The body is one transaction or a list.
/// Each label is emitted once for that body.
fn peg_lines(body: &str) -> Vec<String> {
    let txs = peg_transactions(body);
    let mut lines = Vec::new();
    if txs
        .iter()
        .any(|tx| tx.vin.iter().any(|input| input.is_pegin == Some(true)))
    {
        lines.push("Peg-in".to_string());
    }
    if txs
        .iter()
        .any(|tx| tx.vout.iter().any(|output| output.pegout.is_some()))
    {
        lines.push("Liquid Peg Out".to_string());
    }
    lines
}

fn peg_transactions(body: &str) -> Vec<splora_frontend_shared::wire::Transaction> {
    if let Ok(txs) = serde_json::from_str::<Vec<splora_frontend_shared::wire::Transaction>>(body) {
        return txs;
    }
    match serde_json::from_str::<splora_frontend_shared::wire::Transaction>(body) {
        Ok(tx) => vec![tx],
        Err(_) => Vec::new(),
    }
}

/// One object from `GET /asset/:id`, the same untagged document as a registry row.
fn one_registry_asset(body: &str) -> Option<RegistryAsset> {
    let wrapped = format!("[{}]", body.trim());
    parse_assets_registry(&wrapped).ok()?.into_iter().next()
}

fn stats_amount(blinded: bool, amount: u64) -> String {
    if blinded {
        "Confidential".to_string()
    } else {
        amount.to_string()
    }
}

/// Precision, issuer, issuance transaction, and chain amounts from `GET /asset/:id`.
///
/// Chain stats only. A blinded issuance stays Confidential and is not an amount.
/// A zero issued or pegged figure stays off the row the asset page hides.
/// The integers stay the document values.
fn asset_stats_lines(body: &str) -> (String, String) {
    match one_registry_asset(body) {
        Some(RegistryAsset::Issued(issued)) => issued_asset_stats(&issued),
        Some(RegistryAsset::Native(native)) => (pegged_asset_stats(&native), String::new()),
        None => (String::new(), String::new()),
    }
}

fn issued_asset_stats(issued: &splora_frontend_shared::IssuedRegistryAsset) -> (String, String) {
    let blinded =
        issued.chain_stats.has_blinded_issuances || issued.mempool_stats.has_blinded_issuances;
    let mut lines = Vec::new();
    if let Some(precision) = issued.precision {
        lines.push(format!("Precision {precision}"));
    }
    if let Some(domain) = issued
        .entity
        .get("domain")
        .and_then(|value| value.as_str())
        .filter(|domain| !domain.is_empty())
    {
        lines.push(format!("Issuer {domain}"));
    }
    let issuance = issued.issuance_txin.txid.clone();
    if !issuance.is_empty() {
        lines.push(format!("Issuance TX {issuance}"));
    }
    let chain = &issued.chain_stats;
    if chain.issued_amount != 0 {
        lines.push(format!(
            "Issued amount {}",
            stats_amount(blinded, chain.issued_amount)
        ));
    }
    lines.push(format!(
        "Burned amount {}",
        stats_amount(blinded, chain.burned_amount)
    ));
    if chain.issued_amount != 0 {
        let circulating = chain.issued_amount.saturating_sub(chain.burned_amount);
        lines.push(format!(
            "Circulating amount {}",
            stats_amount(blinded, circulating)
        ));
    }
    (lines.join("\n"), issuance)
}

fn pegged_asset_stats(native: &splora_frontend_shared::NativeRegistryAsset) -> String {
    let chain = &native.chain_stats;
    let mut lines = Vec::new();
    if chain.peg_in_amount != 0 {
        lines.push(format!("Pegged in {}", chain.peg_in_amount));
    }
    if chain.peg_out_amount != 0 {
        lines.push(format!("Pegged out {}", chain.peg_out_amount));
    }
    lines.push(format!("Burned amount {}", chain.burned_amount));
    if chain.peg_in_amount != 0 {
        let circulating = chain
            .peg_in_amount
            .saturating_sub(chain.peg_out_amount)
            .saturating_sub(chain.burned_amount);
        lines.push(format!("Circulating amount {circulating}"));
    }
    lines.join("\n")
}

/// Liquid Bitcoin on liquid. Liquid testnet and every other asset use issuance.
fn asset_transaction_heading(asset_id: &str) -> &'static str {
    const NATIVE_LIQUID_ASSET: &str =
        "6f0279e9ed041c3d710a9f57d0c02928416460c4b722ae3457a11eec381c526d";
    if asset_id == NATIVE_LIQUID_ASSET {
        "Peg In/Out and Burn Transactions"
    } else {
        "Issuance and Burn Transactions"
    }
}

/// Input and output rows for `GET /asset/:id/txs`. Not an asset box.
fn asset_peg_burn_lines(asset_id: &str, txs_body: &str) -> String {
    let mut lines = vec![asset_transaction_heading(asset_id).to_string()];
    for tx in peg_transactions(txs_body) {
        if tx.vin.is_empty() && tx.vout.is_empty() {
            continue;
        }
        for input in &tx.vin {
            lines.push(asset_party_line("table-tx-vin", input.prevout.as_ref()));
        }
        for output in &tx.vout {
            lines.push(asset_party_line("table-tx-vout", Some(output)));
        }
    }
    lines.join("\n")
}

/// A missing amount stays blank. Confidential is not a commitment and not an amount.
fn asset_party_line(
    class: &str,
    output: Option<&splora_frontend_shared::wire::TxOutput>,
) -> String {
    let Some(output) = output else {
        return format!("{class}  value ");
    };
    let address = output.script_pubkey_address.as_deref().unwrap_or("");
    let amount = amount_text(output.value);
    let mut line = format!("{class} {address} value {amount}");
    let commitment = output
        .valuecommitment
        .as_deref()
        .is_some_and(|text| !text.is_empty())
        || output
            .assetcommitment
            .as_deref()
            .is_some_and(|text| !text.is_empty());
    if output.value.is_none() && commitment {
        line.push_str(" Confidential");
    }
    if let Some(asset) = output.asset.as_deref().filter(|text| !text.is_empty()) {
        line.push_str(" asset ");
        line.push_str(asset);
    }
    line
}

fn highlight_lines(body: &str, viewed: &str) -> Vec<String> {
    let Ok(txs) = serde_json::from_str::<Vec<splora_frontend_shared::wire::Transaction>>(body)
    else {
        return Vec::new();
    };
    let mut lines = Vec::new();
    for count in splora_frontend_shared::highlight_counts(viewed, &txs) {
        for _ in 0..count.inputs {
            lines.push(format!("highlight input {viewed}"));
        }
        for _ in 0..count.outputs {
            lines.push(format!("highlight output {viewed}"));
        }
    }
    lines
}

fn poison_network_name(network: Network) -> &'static str {
    match network {
        Network::Mainnet => "mainnet",
        Network::Testnet3 => "testnet",
        Network::Testnet4 => "testnet4",
        Network::Mutinynet => "signet",
        Network::Liquid => "liquid",
    }
}

fn poison_lines_for_tx(
    tx: &splora_frontend_shared::wire::Transaction,
    network: &str,
) -> Vec<String> {
    let alerts = splora_frontend_shared::poison_alerts(tx, network);
    let mut found = Vec::new();
    let mut seen_groups = Vec::new();
    for alert in &alerts {
        if seen_groups.contains(&alert.group) {
            continue;
        }
        seen_groups.push(alert.group);
        let mut members: Vec<&splora_frontend_shared::PoisonAlert> = Vec::new();
        for row in alerts.iter().filter(|row| row.group == alert.group) {
            if members.iter().all(|kept| kept.address != row.address) {
                members.push(row);
            }
        }
        if members.len() < 2 {
            continue;
        }
        push_poison_line(
            &mut found,
            &members[0].address,
            &members[1].address,
            &members[0].script_type,
            network,
        );
    }
    let parties = poison_parties(tx);
    for left in 0..parties.len() {
        for right in (left + 1)..parties.len() {
            if parties[left].1 != parties[right].1 {
                continue;
            }
            push_poison_line(
                &mut found,
                &parties[left].0,
                &parties[right].0,
                &parties[left].1,
                network,
            );
        }
    }
    if found.is_empty() {
        return Vec::new();
    }
    let mut lines =
        vec!["Warning! This transaction involves deceptively similar addresses.".to_string()];
    lines.extend(found);
    lines
}

fn poison_parties(tx: &splora_frontend_shared::wire::Transaction) -> Vec<(String, String)> {
    let mut parties = Vec::new();
    for input in &tx.vin {
        if let Some(prev) = &input.prevout {
            if let Some(address) = prev
                .script_pubkey_address
                .as_deref()
                .filter(|address| !address.is_empty())
            {
                parties.push((address.to_string(), prev.script_pubkey_type.clone()));
            }
        }
    }
    for output in &tx.vout {
        if let Some(address) = output
            .script_pubkey_address
            .as_deref()
            .filter(|address| !address.is_empty())
        {
            parties.push((address.to_string(), output.script_pubkey_type.clone()));
        }
    }
    parties
}

fn push_poison_line(
    found: &mut Vec<String>,
    left: &str,
    right: &str,
    script_type: &str,
    network: &str,
) {
    let Some(left_parts) =
        splora_frontend_shared::address_parts(left, right, script_type, network, 1)
    else {
        return;
    };
    let Some(right_parts) =
        splora_frontend_shared::address_parts(right, left, script_type, network, 1)
    else {
        return;
    };
    let suffix = if left_parts.postfix.is_empty() {
        right_parts.postfix.as_str()
    } else {
        left_parts.postfix.as_str()
    };
    let line = if suffix.is_empty() {
        format!(
            "poison prefix {} infix {} infix {}",
            left_parts.prefix, left_parts.infix, right_parts.infix
        )
    } else {
        format!(
            "poison prefix {} infix {} infix {} suffix {suffix}",
            left_parts.prefix, left_parts.infix, right_parts.infix
        )
    };
    if !found.contains(&line) {
        found.push(line);
    }
}

fn poison_lines_for_txs(body: &str, network: &str) -> Vec<String> {
    let Ok(txs) = serde_json::from_str::<Vec<splora_frontend_shared::wire::Transaction>>(body)
    else {
        return Vec::new();
    };
    let mut lines = Vec::new();
    for tx in &txs {
        for line in poison_lines_for_tx(tx, network) {
            if !lines.contains(&line) {
                lines.push(line);
            }
        }
    }
    lines
}

fn asset_id_in_row(row: &str) -> Option<&str> {
    let token = row.strip_prefix("row asset ")?.split_whitespace().next()?;
    if is_64_hex(token) { Some(token) } else { None }
}

fn registry_asset_ids(body: &str) -> Vec<String> {
    let Ok(assets) = parse_assets_registry(body) else {
        return Vec::new();
    };
    assets
        .into_iter()
        .filter_map(|asset| {
            let id = match asset {
                RegistryAsset::Issued(issued) => issued.asset_id,
                RegistryAsset::Native(native) => native.asset_id,
            };
            if id.is_empty() { None } else { Some(id) }
        })
        .collect()
}

fn registry_issued_lines(body: &str) -> String {
    let Ok(assets) = parse_assets_registry(body) else {
        return String::new();
    };
    let mut lines = Vec::new();
    for asset in assets {
        if let RegistryAsset::Issued(issued) = asset {
            if let Some(name) = issued.name.filter(|name| !name.is_empty()) {
                lines.push(name);
            }
            if let Some(ticker) = issued.ticker.filter(|ticker| !ticker.is_empty()) {
                lines.push(ticker);
            }
        }
    }
    lines.join("\n")
}

/// One featured card per named issued row from `GET /assets/registry`.
/// A pegged row has no name and is not a card. The line is the name and the
/// ticker. Amounts, prices, and block graphics stay off the card.
fn registry_featured_card_lines(body: &str) -> String {
    let Ok(assets) = parse_assets_registry(body) else {
        return String::new();
    };
    let mut lines = Vec::new();
    for asset in assets {
        let RegistryAsset::Issued(issued) = asset else {
            continue;
        };
        let Some(name) = issued.name.filter(|name| !name.is_empty()) else {
            continue;
        };
        let mut line = format!("featured card {name}");
        if let Some(ticker) = issued.ticker.filter(|ticker| !ticker.is_empty()) {
            line.push(' ');
            line.push_str(&ticker);
        }
        lines.push(line);
    }
    lines.join("\n")
}

fn package_txid_rows(body: &str) -> Vec<String> {
    match parse_package(body) {
        Ok(parsed) => parsed
            .tx_results
            .values()
            .filter(|tx| !tx.txid.is_empty())
            .map(|tx| {
                let mut row = format!("row tx txid {}", tx.txid);
                // Missing and "" are not a witness id. Do not copy txid into that spot.
                if let Some(wtxid) = tx.other_wtxid.as_deref().filter(|wtxid| !wtxid.is_empty()) {
                    row.push_str(" wtxid ");
                    row.push_str(wtxid);
                }
                row
            })
            .collect(),
        Err(_) => Vec::new(),
    }
}

fn render_broadcast_result(body: &str) -> Result<String, ClientError> {
    let trimmed = body.trim();
    if let Some(sentence) = indexer_sentence(trimmed) {
        return Ok(sentence.to_string());
    }
    if let Ok(result) =
        serde_json::from_str::<splora_frontend_shared::wire::BroadcastResult>(trimmed)
    {
        if !result.txid.is_empty() {
            return Ok(format!("broadcast txid {}", result.txid));
        }
    }
    // POST /tx returns the txid as text/plain. A JSON object uses BroadcastResult above.
    // A sentence with a space stays the indexer text.
    if trimmed.is_empty() || trimmed.starts_with('{') || trimmed.starts_with('[') {
        return Err(ClientError::Http("broadcast result was empty".into()));
    }
    if trimmed.contains(char::is_whitespace) {
        return Ok(trimmed.to_string());
    }
    Ok(format!("broadcast txid {trimmed}"))
}

fn render_package(body: &str) -> Result<String, ClientError> {
    let parsed = parse_package(body).map_err(http_parse)?;
    let mut lines = vec![format!("broadcast package {}", parsed.package_msg)];
    for tx in parsed.tx_results.values() {
        lines.push(tx.txid.clone());
        // Missing and "" are not a witness id. Do not copy txid into that spot.
        if let Some(wtxid) = tx.other_wtxid.as_deref().filter(|wtxid| !wtxid.is_empty()) {
            lines.push(wtxid.to_string());
        }
        // A missing key, JSON null, and "" omit the label. The text 0 still prints.
        // This is PackageTx.error, not reject-reason. The text is not escaped.
        if let Some(error) = tx.error.as_deref().filter(|error| !error.is_empty()) {
            lines.push(format!("error {error}"));
        }
    }
    Ok(lines.join("\n"))
}

fn render_test_accept(body: &str) -> Result<String, ClientError> {
    let rows = serde_json::from_str::<Vec<splora_frontend_shared::wire::TestTxResult>>(body)
        .map_err(http_parse)?;
    if rows.is_empty() {
        return Ok("test:".to_string());
    }
    Ok(rows
        .iter()
        .map(|row| {
            let allowed = match row.allowed {
                Some(true) => "true",
                Some(false) => "false",
                None => "",
            };
            let mut line = format!("test txid {} allowed {allowed}", row.txid);
            if let Some(reason) = row
                .reject_reason
                .as_deref()
                .filter(|reason| !reason.is_empty())
            {
                line.push_str(" reject-reason ");
                line.push_str(reason);
            }
            if let Some(fees) = &row.fees {
                line.push_str(&format!(
                    " fee {} effective_feerate {}",
                    plain_number(fees.base),
                    plain_number(fees.effective_feerate)
                ));
            }
            line
        })
        .collect::<Vec<_>>()
        .join("\n"))
}

/// USD per BTC fixture. Shown only through [`format_fiat`].
pub const USD_PER_BTC: &str = "123456";

/// JPY per BTC fixture. Shown only through [`format_fiat`].
pub const JPY_PER_BTC: &str = "11057757";

pub fn one_btc_sats_text() -> String {
    btc_to_sats("1").expect("one btc converts").to_string()
}

pub fn truncated_btc_text(input: &str) -> String {
    truncate_btc_decimals(input)
}

pub fn capped_btc_text(input: &str) -> String {
    cap_btc_supply(input).btc
}

pub fn usd_fixture_text() -> String {
    format_fiat(Usd, USD_PER_BTC)
}

pub fn jpy_fixture_text() -> String {
    format_fiat(Jpy, JPY_PER_BTC)
}

pub fn switched_fiat_symbols() -> [char; 3] {
    let shown = switch_usd_jpy_usd("1500");
    [
        shown[0].chars().next().expect("usd symbol"),
        shown[1].chars().next().expect("jpy symbol"),
        shown[2].chars().next().expect("usd symbol"),
    ]
}

pub fn statistics_page_text() -> &'static str {
    route_availability("/api/v1/statistics")
}

pub fn mining_page_text() -> &'static str {
    route_availability("/v1/mining")
}

pub fn lightning_page_text() -> &'static str {
    route_availability("/lightning")
}

pub fn graphs_page_text() -> &'static str {
    route_availability("/api/v1/statistics")
}

pub fn hashrate_page_text() -> &'static str {
    route_availability("/v1/mining")
}

pub fn pools_page_text() -> &'static str {
    route_availability("/v1/mining")
}

pub fn price_chart_page_text() -> &'static str {
    route_availability("/api/v1/historical-price")
}

/// Calculator body the GPUI shell paints through [`ScreenHost::rendered`].
pub fn calculator_screen_text() -> String {
    format!(
        "Calculator
Copy
Copy
Copy
₿
{}
{}
{}
fiat {}
fiat {}
BTC {}
sats {}
{}
A live price feed is not served.",
        one_btc_sats_text(),
        truncated_btc_text("1.123456789"),
        capped_btc_text("25000000"),
        btc_fiat_usd("0.33"),
        btc_fiat_usd("0.00000001"),
        btc_fiat_jpy("1"),
        sats_fiat_usd(10_000),
        route_availability("/v1/prices"),
    )
}

/// Fiat body the GPUI shell paints through [`ScreenHost::rendered`].
pub fn fiat_screen_text() -> String {
    format!("{}\n{}", usd_fixture_text(), jpy_fixture_text())
}

fn calculator_entered_lines(form: &CalculatorForm) -> String {
    format!(
        "entered fiat {}\nentered bitcoin {}\nentered satoshis {}\nchosen {}",
        form.fiat, form.btc, form.sats, form.fiat_text
    )
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
    pub calculator: CalculatorForm,
    /// Characters the shell key path has typed but not submitted.
    calculator_draft: String,
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
            calculator: loaded_calculator(Usd),
            calculator_draft: String::new(),
            client,
            notice: String::new(),
        }
    }

    pub fn enter_calculator_amount(&mut self, field: &str, amount: &str) {
        let names = calculator_field_names();
        if field == names[0] {
            if let Some(next) = enter_fiat(&self.calculator, amount) {
                self.calculator = next;
            }
        } else if field == names[1] {
            self.calculator = enter_btc(&self.calculator, amount);
        } else if field == names[2] {
            if let Ok(sats) = amount.parse::<u64>() {
                let btc = sats_to_btc(sats);
                self.calculator = enter_btc(&self.calculator, &btc);
            }
        }
    }

    pub fn choose_calculator_fiat(&mut self, code: FiatCode) {
        let btc = self.calculator.btc.clone();
        self.calculator = loaded_calculator(code);
        self.calculator = enter_btc(&self.calculator, &btc);
    }

    /// Commit the shell draft. `jpy` selects yen. `fiat`, `bitcoin`, or `satoshis`
    /// plus an amount calls the shared calculator functions. No currency-rate match.
    fn apply_calculator_draft(&mut self) {
        let draft = std::mem::take(&mut self.calculator_draft);
        let draft = draft.trim();
        if draft.is_empty() {
            return;
        }
        if draft.eq_ignore_ascii_case("jpy") {
            self.choose_calculator_fiat(Jpy);
            return;
        }
        let names = calculator_field_names();
        let mut parts = draft.split_whitespace();
        let Some(field) = parts.next() else {
            return;
        };
        let Some(amount) = parts.next() else {
            return;
        };
        if parts.next().is_some() {
            return;
        }
        if names.contains(&field) {
            self.enter_calculator_amount(field, amount);
        }
    }

    pub fn query_text(&self) -> String {
        match self.screen {
            Screen::Dashboard => self.dashboard.search.clone(),
            Screen::Block => self.block.query.clone(),
            Screen::Blocks => self.blocks.start_height.clone(),
            Screen::Tx => self.tx.txid.clone(),
            Screen::Address => self.address.address.clone(),
            Screen::MultiAddress => self.multi_address.input.clone(),
            Screen::Broadcast => self.broadcast.raw_tx.clone(),
            Screen::TestTransactions => self.test_txs.input.clone(),
            Screen::Calculator => self.calculator_draft.clone(),
            _ => String::new(),
        }
    }

    pub fn set_query(&mut self, text: &str) {
        let trimmed = text.trim();
        match self.screen {
            Screen::Dashboard => self.dashboard.search = trimmed.to_string(),
            Screen::Block => {
                let mut parts = trimmed.split_whitespace();
                self.block.query = parts.next().unwrap_or("").to_string();
                let page = parts.next().unwrap_or("");
                if page.is_empty() || !is_numeric_id(page) || parts.next().is_some() {
                    self.block.start_index.clear();
                    self.block.typing_page = false;
                } else {
                    self.block.start_index = page.to_string();
                    self.block.typing_page = true;
                }
            }
            Screen::Blocks => {
                self.blocks.start_height =
                    trimmed.chars().filter(|ch| ch.is_ascii_digit()).collect();
            }
            Screen::Tx => self.tx.txid = one_query_token(trimmed),
            Screen::Address => {
                self.address.address = one_query_token(trimmed);
                self.address.last_seen.clear();
                self.address.typing_page = false;
            }
            Screen::MultiAddress => self.multi_address.input = trimmed.to_string(),
            Screen::Broadcast => self.broadcast.raw_tx = trimmed.to_string(),
            Screen::TestTransactions => self.test_txs.input = trimmed.to_string(),
            Screen::Calculator => self.calculator_draft = trimmed.to_string(),
            _ => {}
        }
    }

    pub fn push_str(&mut self, text: &str) {
        match self.screen {
            Screen::Dashboard => self.dashboard.search.push_str(text),
            Screen::Block => {
                for ch in text.chars() {
                    if ch.is_whitespace() {
                        self.block.typing_page = true;
                    } else if self.block.typing_page {
                        if ch.is_ascii_digit() {
                            self.block.start_index.push(ch);
                        }
                    } else {
                        self.block.query.push(ch);
                    }
                }
            }
            Screen::Blocks => {
                for ch in text.chars() {
                    if ch.is_ascii_digit() {
                        self.blocks.start_height.push(ch);
                    }
                }
            }
            Screen::Tx => self.tx.txid.push_str(text),
            Screen::Address => {
                for ch in text.chars() {
                    if ch.is_whitespace() {
                        self.address.typing_page = true;
                    } else if self.address.typing_page {
                        self.address.last_seen.push(ch);
                    } else {
                        self.address.address.push(ch);
                    }
                }
            }
            Screen::MultiAddress => self.multi_address.input.push_str(text),
            Screen::Broadcast => self.broadcast.raw_tx.push_str(text),
            Screen::TestTransactions => self.test_txs.input.push_str(text),
            Screen::Calculator => self.calculator_draft.push_str(text),
            _ => {}
        }
    }

    pub fn backspace(&mut self) {
        match self.screen {
            Screen::Dashboard => {
                self.dashboard.search.pop();
            }
            Screen::Block => {
                if self.block.typing_page {
                    if self.block.start_index.pop().is_none() {
                        self.block.typing_page = false;
                    }
                } else {
                    self.block.query.pop();
                }
            }
            Screen::Blocks => {
                self.blocks.start_height.pop();
            }
            Screen::Tx => {
                self.tx.txid.pop();
            }
            Screen::Address => {
                if self.address.typing_page {
                    if self.address.last_seen.pop().is_none() {
                        self.address.typing_page = false;
                    }
                } else {
                    self.address.address.pop();
                }
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
            Screen::Calculator => {
                self.calculator_draft.pop();
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
        if screen == Screen::Dashboard {
            self.dashboard.search.clear();
        }
        self.screen = screen;
        self.reload()
    }

    /// Open `GET /tx/:txid` from a painted row. Empty and slash-bearing ids are rejected.
    pub fn open_transaction(&mut self, txid: &str) -> Result<(), ClientError> {
        let token = one_query_token(txid);
        if token.is_empty() || token.contains('/') {
            return self.finish(Err(ClientError::BadPath));
        }
        self.tx.txid = token;
        self.screen = Screen::Tx;
        let loaded = self.tx.load(&self.client);
        if let Err(ClientError::Http(message)) = &loaded {
            if message.starts_with("missing ") {
                self.tx.body.clear();
                self.tx.rows.clear();
                return self.finish(Ok(()));
            }
        }
        self.finish(loaded)
    }

    /// Open `GET /block/:hash`, or `GET /block-height/:height` when `id` is all digits.
    pub fn open_block(&mut self, id: &str) -> Result<(), ClientError> {
        let token = one_query_token(id);
        if token.is_empty() || token.contains('/') {
            return self.finish(Err(ClientError::BadPath));
        }
        self.block.query = token;
        self.block.start_index.clear();
        self.block.typing_page = false;
        self.activate(Screen::Block)
    }

    /// Open `GET /address/:script` plus its transactions, utxos, and mempool transactions.
    pub fn open_address(&mut self, script: &str) -> Result<(), ClientError> {
        let token = one_query_token(script);
        if token.is_empty() || token.contains('/') {
            return self.finish(Err(ClientError::BadPath));
        }
        self.address.use_scripthash = is_64_hex(&token);
        self.address.address = token;
        self.address.last_seen.clear();
        self.address.typing_page = false;
        self.activate(Screen::Address)
    }

    /// One painted row. Transaction ids still use [`Self::open_transaction`].
    /// A row that is not a block, address, page, or chain cursor does nothing.
    pub fn open_row(&mut self, row: &str) -> Result<(), ClientError> {
        if let Some(cursor) = mempool_page_in_row(row) {
            let cursor = cursor.to_string();
            self.mempool.last_seen = cursor;
            return self.activate(Screen::Mempool);
        }
        if let Some(hash) = tip_hash_in_row(row) {
            let hash = hash.to_string();
            return self.open_block(&hash);
        }
        if let Some(hash) = merkle_hash_in_row(row) {
            let hash = hash.to_string();
            return self.open_transaction(&hash);
        }
        if let Some(index) = page_index_in_row(row) {
            let index = index.to_string();
            return self.open_block_page(&index);
        }
        if let Some(cursor) = chain_cursor_in_row(row) {
            let cursor = cursor.to_string();
            return self.open_chain_page(&cursor);
        }
        if let Some(txid) = transaction_id_in_row(row) {
            let txid = txid.to_string();
            return self.open_transaction(&txid);
        }
        if let Some(hash) = block_hash_in_row(row) {
            let hash = hash.to_string();
            return self.open_block(&hash);
        }
        if let Some(height) = bare_height_in_row(row) {
            let height = height.to_string();
            return self.open_block(&height);
        }
        if let Some(address) = address_in_row(row) {
            let address = address.to_string();
            return self.open_address(&address);
        }
        if row == "row block details" {
            self.block.toggle_details();
            return self.finish(Ok(()));
        }
        if let Some(asset) = asset_id_in_row(row) {
            let asset = asset.to_string();
            if self.client.network() == Network::Liquid {
                let hit = classify_hex(&self.client, &asset)?;
                return self.apply_hex_hit(hit);
            }
        }
        self.finish(Ok(()))
    }

    fn open_block_page(&mut self, index: &str) -> Result<(), ClientError> {
        if self.block.query.is_empty() || !is_numeric_id(index) {
            return self.finish(Err(ClientError::BadPath));
        }
        let start: u64 = match index.parse() {
            Ok(start) => start,
            Err(_) => return self.finish(Err(ClientError::BadPath)),
        };
        if start % 25 != 0 {
            return self.finish(Err(ClientError::BadPath));
        }
        self.block.start_index = index.to_string();
        self.block.typing_page = true;
        self.activate(Screen::Block)
    }

    fn open_chain_page(&mut self, txid: &str) -> Result<(), ClientError> {
        if self.address.address.is_empty() || txid.is_empty() || txid.contains('/') {
            return self.finish(Err(ClientError::BadPath));
        }
        self.address.last_seen = txid.to_string();
        self.address.typing_page = true;
        self.activate(Screen::Address)
    }

    pub fn reload(&mut self) -> Result<(), ClientError> {
        let result = self.reload_current();
        self.finish(result)
    }

    fn reload_current(&mut self) -> Result<(), ClientError> {
        match self.screen {
            Screen::Dashboard => {
                if self.dashboard.search.trim().is_empty() {
                    self.dashboard.load(&self.client)
                } else {
                    self.search_dashboard()
                }
            }
            Screen::Blocks => self.blocks.load(&self.client),
            Screen::Block => self.block.load(&self.client),
            Screen::Tx => self.tx.load(&self.client),
            Screen::Address => self.address.load(&self.client),
            Screen::Mempool => self.mempool.load(&self.client),
            Screen::MultiAddress => self.multi_address.load(&self.client),
            Screen::Calculator => {
                self.apply_calculator_draft();
                Ok(())
            }
            Screen::Broadcast
            | Screen::TestTransactions
            | Screen::Popup
            | Screen::Terms
            | Screen::Privacy
            | Screen::Trademark
            | Screen::Docs
            | Screen::Faq
            | Screen::ApiRest
            | Screen::ApiWebsocket
            | Screen::Fiat
            | Screen::Statistics
            | Screen::Mining
            | Screen::Graphs
            | Screen::Hashrate
            | Screen::PriceChart
            | Screen::Lightning
            | Screen::Pools => Ok(()),
        }
    }

    fn search_dashboard(&mut self) -> Result<(), ClientError> {
        let query = self.dashboard.search.trim().to_string();
        // All digits are a block height, including 64 digits that do not fit in u64.
        // A 64-digit id whose height lookup is missing can still be a tx, block,
        // asset, or scripthash. A 64-hex id that is not all digits takes that path too.
        if is_numeric_id(&query) && query.len() != 64 {
            return self.open_block(&query);
        }
        if is_numeric_id(&query) && query.len() == 64 {
            let path = format!("/block-height/{query}");
            if let ProbeBody::Hit(body) = read_probe(&self.client, &path)? {
                if indexer_sentence(&body).is_none() && parse_block_height(&body).is_ok() {
                    return self.open_block(&query);
                }
            }
        }
        if is_64_hex(&query) {
            let hit = classify_hex(&self.client, &query)?;
            return self.apply_hex_hit(hit);
        }
        if looks_like_address(&query) {
            return self.open_address(&query);
        }
        if self.client.network() == Network::Liquid {
            if let Ok(()) = self.show_liquid_search() {
                return Ok(());
            }
        }
        self.show_prefix(&query)
    }

    fn apply_hex_hit(&mut self, hit: HexHit) -> Result<(), ClientError> {
        match hit {
            HexHit::Tx(txid) => self.open_transaction(&txid),
            HexHit::Block(hash) => self.open_block(&hash),
            HexHit::Script(hash) => self.open_address(&hash),
            HexHit::Asset {
                name,
                ticker,
                asset_id,
                txs,
                chain,
                mempool,
                supply,
                supply_decimal,
                stats,
                issuance_txid,
            } => {
                let mut lines = format!("asset id {asset_id}\n");
                if !name.is_empty() {
                    lines.push_str(&format!("asset name {name}\n"));
                }
                if !ticker.is_empty() {
                    lines.push_str(&format!("ticker {ticker}\n"));
                }
                if !stats.is_empty() {
                    lines.push_str(&stats);
                    lines.push('\n');
                }
                lines.push_str(&asset_peg_burn_lines(&asset_id, &txs));
                append_tx_body(&mut lines, &txs);
                append_tx_body(&mut lines, &chain);
                append_tx_body(&mut lines, &mempool);
                if let Ok(text) = parse_asset_supply(&supply) {
                    lines.push('\n');
                    lines.push_str(&text);
                }
                if let Ok(text) = parse_asset_supply_decimal(&supply_decimal) {
                    lines.push('\n');
                    lines.push_str(&text);
                }
                self.dashboard.lines = lines;
                self.dashboard.rows = if issuance_txid.is_empty() {
                    Vec::new()
                } else {
                    vec![format!("row tx txid {issuance_txid}")]
                };
                self.screen = Screen::Dashboard;
                Ok(())
            }
            HexHit::Sentence(sentence) => {
                self.dashboard.lines = sentence;
                self.dashboard.rows.clear();
                self.screen = Screen::Dashboard;
                Ok(())
            }
            HexHit::NotServed => {
                self.dashboard.lines = route_availability("/v1/mining").to_string();
                self.dashboard.rows.clear();
                self.screen = Screen::Dashboard;
                Ok(())
            }
            HexHit::Miss => Err(ClientError::Http("missing search".into())),
        }
    }

    fn show_prefix(&mut self, prefix: &str) -> Result<(), ClientError> {
        // The address index stores bech32 mainnet prefixes in lowercase.
        let folded = prefix.to_ascii_lowercase();
        let requested = if folded.starts_with("bc1") {
            folded.as_str()
        } else {
            prefix
        };
        let path = address_prefix_path(requested);
        match read_probe(&self.client, &path)? {
            ProbeBody::Hit(body) => {
                if let Some(sentence) = indexer_sentence(&body) {
                    self.dashboard.lines = sentence.to_string();
                    self.dashboard.rows.clear();
                    self.screen = Screen::Dashboard;
                    return Ok(());
                }
                let addresses = parse_address_prefix(&body).map_err(http_parse)?;
                self.dashboard.lines = addresses.join("\n");
                self.dashboard.rows = addresses
                    .iter()
                    .map(|address| format!("row prefix {address}"))
                    .collect();
                self.screen = Screen::Dashboard;
                Ok(())
            }
            ProbeBody::Miss => Err(ClientError::Http(format!("missing {path}"))),
        }
    }

    /// Liquid registry search. The shared helper appends `?q=`, and this client
    /// rejects `?`, so the request is `GET /assets/registry/search` with no query.
    fn show_liquid_search(&mut self) -> Result<(), ClientError> {
        let path = "/assets/registry/search";
        match read_probe(&self.client, path)? {
            ProbeBody::Hit(body) => {
                if let Some(sentence) = indexer_sentence(&body) {
                    self.dashboard.lines = sentence.to_string();
                    self.dashboard.rows.clear();
                    self.screen = Screen::Dashboard;
                    return Ok(());
                }
                let hits = parse_assets_registry_search(&body).map_err(http_parse)?;
                let mut lines = Vec::new();
                let mut rows = Vec::new();
                // One typeahead window for the whole hit list. An empty body paints none.
                // Each hit is one button. The row is not a second window.
                if !hits.is_empty() {
                    lines.push("suggestion window".to_string());
                }
                for hit in &hits {
                    if !hit.name.is_empty() {
                        lines.push(hit.name.clone());
                    }
                    if let Some(ticker) = hit.ticker.as_deref().filter(|ticker| !ticker.is_empty())
                    {
                        lines.push(ticker.to_string());
                    }
                    if is_64_hex(&hit.asset_id) {
                        rows.push(format!("row asset {}", hit.asset_id));
                    }
                }
                self.dashboard.lines = lines.join("\n");
                self.dashboard.rows = rows;
                self.screen = Screen::Dashboard;
                Ok(())
            }
            ProbeBody::Miss => Err(ClientError::Http(format!("missing {path}"))),
        }
    }

    fn finish(&mut self, result: Result<(), ClientError>) -> Result<(), ClientError> {
        self.notice = match &result {
            Ok(()) => String::new(),
            Err(err) => notice_from_error(err),
        };
        result
    }

    pub fn submit(&mut self) -> Result<(), ClientError> {
        let result = match self.screen {
            Screen::Broadcast => self.broadcast.submit(&self.client),
            Screen::TestTransactions => self.test_txs.submit(&self.client),
            Screen::Calculator => {
                self.apply_calculator_draft();
                Ok(())
            }
            _ => Err(ClientError::BadPath),
        };
        self.finish(result)
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
            Screen::Calculator => format!(
                "{}\n{}",
                calculator_screen_text(),
                calculator_entered_lines(&self.calculator)
            ),
            Screen::Fiat => fiat_screen_text(),
            Screen::Statistics => statistics_page_text().to_string(),
            Screen::Mining => mining_page_text().to_string(),
            Screen::Graphs => graphs_page_text().to_string(),
            Screen::Hashrate => hashrate_page_text().to_string(),
            Screen::PriceChart => price_chart_page_text().to_string(),
            Screen::Lightning => lightning_page_text().to_string(),
            Screen::Pools => pools_page_text().to_string(),
            Screen::Popup => String::new(),
        }
    }

    /// One row per block or transaction on the active screen, plus address funded
    /// sums and each utxo value. Empty on screens that do not list those.
    pub fn painted_rows(&self) -> Vec<String> {
        match self.screen {
            Screen::Dashboard => self.dashboard.rows.clone(),
            Screen::Blocks => self.blocks.rows.clone(),
            Screen::Block => self.block.rows.clone(),
            Screen::Tx => self.tx.rows.clone(),
            Screen::Address => self.address.rows.clone(),
            Screen::Mempool => self.mempool.rows.clone(),
            Screen::MultiAddress => self.multi_address.rows.clone(),
            Screen::Broadcast => self.broadcast.rows.clone(),
            Screen::TestTransactions
            | Screen::Popup
            | Screen::Terms
            | Screen::Privacy
            | Screen::Trademark
            | Screen::Docs
            | Screen::Faq
            | Screen::ApiRest
            | Screen::ApiWebsocket
            | Screen::Calculator
            | Screen::Fiat
            | Screen::Statistics
            | Screen::Mining
            | Screen::Graphs
            | Screen::Hashrate
            | Screen::PriceChart
            | Screen::Lightning
            | Screen::Pools => Vec::new(),
        }
    }
}

/// Rows `Shell::render` paints, one element per block, transaction, or utxo value.
pub fn painted_rows<G: ExplorerGet + ExplorerPost>(host: &ScreenHost<G>) -> Vec<String> {
    host.painted_rows()
}

/// Token after `txid ` in a painted row. Block rows and utxo value rows have none.
pub fn transaction_id_in_row(row: &str) -> Option<&str> {
    let rest = row.split_once("txid ")?.1;
    rest.split_whitespace().next()
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
        lines.extend(host.painted_rows());
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

    #[test]
    fn utxo_omitted_value_paints_confidential_blank_or_zero() {
        let commitment = "a".repeat(66);
        let body = format!(
            r#"[
                {{"txid":"blind","vout":1,"status":{{"confirmed":false}},"valuecommitment":"{commitment}"}},
                {{"txid":"plain","vout":1,"status":{{"confirmed":false}}}},
                {{"txid":"emptyc","vout":1,"status":{{"confirmed":false}},"valuecommitment":"","assetcommitment":""}},
                {{"txid":"zeroed","vout":1,"status":{{"confirmed":true}},"value":0}}
            ]"#
        );
        let painted = render_utxos(&body).expect("paint utxos");
        let rows = utxo_value_rows(&body).expect("utxo rows");
        let line = |txid: &str| {
            painted
                .lines()
                .find(|line| line.contains(&format!("txid {txid} ")))
                .unwrap_or_else(|| panic!("{txid} missing from {painted}"))
                .to_string()
        };

        let blind = line("blind");
        assert!(blind.contains("value Confidential"), "{blind}");
        assert!(!blind.contains(&commitment), "{blind}");
        assert!(!blind.contains('0'), "{blind}");
        assert!(!blind.contains("Some("), "{blind}");
        assert!(!blind.contains("0.001"), "{blind}");
        assert!(
            rows.iter().any(|row| row == "row utxo value Confidential"),
            "{rows:?}"
        );

        let plain = line("plain");
        assert!(!plain.contains("Confidential"), "{plain}");
        assert!(plain.contains("value  confirmed"), "{plain}");
        assert!(!plain.contains('0'), "{plain}");

        let emptyc = line("emptyc");
        assert!(!emptyc.contains("Confidential"), "{emptyc}");
        assert!(emptyc.contains("value  confirmed"), "{emptyc}");
        assert!(!emptyc.contains('0'), "{emptyc}");
        assert!(rows.iter().any(|row| row == "row utxo value "), "{rows:?}");

        let zeroed = line("zeroed");
        assert!(zeroed.contains("value 0 confirmed"), "{zeroed}");
        assert!(!zeroed.contains("Confidential"), "{zeroed}");
        assert!(!zeroed.contains("Some("), "{zeroed}");
        assert!(rows.iter().any(|row| row == "row utxo value 0"), "{rows:?}");
    }

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
            if request.path.ends_with("/utxo") {
                Ok("[]".to_string())
            } else {
                Ok(format!("body {}", request.path))
            }
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
        assert_eq!(Screen::from_link("mining"), Some(Screen::Mining));
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
        assert_eq!(screen.requests.len(), 6);
        assert_eq!(rec.gets.borrow().len(), 6);
        assert_eq!(rec.posts.borrow().len(), 2);
        assert_eq!(rec.posts.borrow()[0].0, "POST");
        assert_eq!(rec.posts.borrow()[0].1, "/addresses/txs/summary");
        assert_eq!(rec.posts.borrow()[1].0, "POST");
        assert_eq!(rec.posts.borrow()[1].1, "/addresses/txs");
        assert!(!rec.posts.borrow()[1].1.contains('?'));
        for (index, address) in ["addrA", "addrB", "addrC"].iter().enumerate() {
            let request = &screen.requests[index * 2];
            assert_eq!(request.method, "GET");
            assert_eq!(request.path, format!("/address/{address}"));
            assert_eq!(
                request.url,
                format!("https://splora.surmount.systems/api/address/{address}")
            );
            assert_eq!(rec.gets.borrow()[index * 2], request.url);
            assert_url_allowed(&request.url);
            let utxo = &screen.requests[index * 2 + 1];
            assert_eq!(utxo.method, "GET");
            assert_eq!(utxo.path, format!("/address/{address}/utxo"));
            assert_eq!(
                utxo.url,
                format!("https://splora.surmount.systems/api/address/{address}/utxo")
            );
            assert_eq!(rec.gets.borrow()[index * 2 + 1], utxo.url);
            assert_url_allowed(&utxo.url);
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
                if page == StaticPage::Docs && *banned == "/txs/package" {
                    continue;
                }
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
        assert!(docs.contains("Two raw transactions use POST /txs/package."));
        assert!(!docs.contains("This shell does not submit a transaction package."));
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
    const ADDR_A_UTXO_JSON: &str =
        r#"[{"txid":"utxoa","vout":0,"status":{"confirmed":true,"block_height":11},"value":4242}]"#;
    const ADDR_B_UTXO_JSON: &str =
        r#"[{"txid":"utxob","vout":2,"status":{"confirmed":false},"value":9090}]"#;
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
                format!("/block/{BLOCK_HASH}/status"),
                format!("/block/{BLOCK_HASH}/header"),
                format!("/block/{BLOCK_HASH}/raw"),
                format!("/block/{BLOCK_HASH}/txid/0"),
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
                format!("/block/{BLOCK_HASH}/status"),
                format!("/block/{BLOCK_HASH}/header"),
                format!("/block/{BLOCK_HASH}/raw"),
                format!("/block/{BLOCK_HASH}/txid/0"),
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
        assert_eq!(
            rec.get_paths(),
            vec![
                "/tx/txidabc".to_string(),
                "/tx/txidabc/outspends".to_string(),
                "/tx/txidabc/merkle-proof".to_string(),
                "/tx/txidabc/hex".to_string(),
                "/tx/txidabc/raw".to_string(),
                "/tx/txidabc/merkleblock-proof".to_string(),
                "/blocks/tip/height".to_string(),
            ]
        );
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
                format!("/address/{script}/txs/mempool"),
                format!("/address/{script}/txs/summary"),
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
                (
                    "/address/addrA/utxo".to_string(),
                    ADDR_A_UTXO_JSON.to_string(),
                ),
                (
                    "/address/addrB/utxo".to_string(),
                    ADDR_B_UTXO_JSON.to_string(),
                ),
            ]),
        );
        host.multi_address.input = "addrA, addrB".to_string();
        host.activate(Screen::MultiAddress).expect("multi");
        assert_eq!(
            rec.get_paths(),
            vec![
                "/address/addrA".to_string(),
                "/address/addrA/utxo".to_string(),
                "/address/addrB".to_string(),
                "/address/addrB/utxo".to_string(),
            ]
        );
        assert_eq!(
            rec.post_paths(),
            vec![
                "/addresses/txs/summary".to_string(),
                "/addresses/txs".to_string(),
            ]
        );
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
                "address addrA utxo txid utxoa vout 0 value 4242",
                "address addrB utxo txid utxob vout 2 value 9090",
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
            BTreeMap::from([
                (
                    splora_frontend_shared::mempool_path().to_string(),
                    r#"{"count":4,"vsize":500,"total_fee":90,"fee_histogram":[[2,100]]}"#
                        .to_string(),
                ),
                (
                    splora_frontend_shared::mempool_recent_path().to_string(),
                    RECENT_JSON.to_string(),
                ),
            ]),
        );
        host.activate(Screen::Mempool).expect("mempool");
        assert_eq!(
            rec.get_paths(),
            vec![
                splora_frontend_shared::mempool_path().to_string(),
                splora_frontend_shared::mempool_recent_path().to_string(),
                splora_frontend_shared::mempool_txids_path().to_string(),
            ]
        );
        assert!(rec.post_paths().is_empty());
        assert_paths_allowed(&rec.get_paths());
        assert_painted_screen(&host, "mempool");
        assert_view(
            &host,
            &[
                "mempool count 4 vsize 500 total_fee 90",
                "fee_histogram 2 100",
                "recent txid txidrecent222 fee 321 vsize 111 value 999",
            ],
        );
        assert!(
            painted_of(&host)
                .contains("The incoming virtual-bytes-per-second rate is not in the payload.")
        );
        assert_no_banned_products(&painted_of(&host));
    }

    #[test]
    fn painted_test_transactions_show_reject_reason() {
        let body = r#"[{"txid":"rejectedtxid888","wtxid":"rejectedwtxid888","allowed":false,"reject-reason":"dust"}]"#;
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
            "Two raw transactions use POST /txs/package.",
        );
        let (mut host, rec) = host_with(Network::Mainnet, BTreeMap::new());
        host.activate(Screen::Docs).expect("docs");
        let painted = painted_of(&host);
        assert!(
            !painted.contains("This shell does not submit a transaction package."),
            "{painted}"
        );
        assert!(rec.get_paths().is_empty(), "{:?}", rec.get_paths());
        assert!(rec.post_paths().is_empty(), "{:?}", rec.post_paths());
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

    fn production_screens_source() -> &'static str {
        let text = include_str!("screens.rs");
        let marker = "#[cfg(test)]\nmod tests {";
        let end = text.find(marker).expect("screens tests module");
        &text[..end]
    }

    fn serde_from_str_types(source: &str) -> Vec<String> {
        let needle = concat!("serde_json::", "from_str::<");
        let mut types = Vec::new();
        let mut rest = source;
        while let Some(pos) = rest.find(needle) {
            rest = &rest[pos + needle.len()..];
            let mut depth = 1usize;
            let mut end = None;
            for (index, ch) in rest.char_indices() {
                if ch == '<' {
                    depth += 1;
                } else if ch == '>' {
                    depth -= 1;
                    if depth == 0 {
                        end = Some(index);
                        break;
                    }
                }
            }
            let end = end.expect("turbofish close");
            types.push(rest[..end].split_whitespace().collect::<String>());
            rest = &rest[end + 1..];
        }
        types
    }

    #[test]
    fn v1_ship_json_screens_use_shared_structs_not_private_ones() {
        let parses = serde_from_str_types(production_screens_source());
        let required = [
            ("block", "splora_frontend_shared::wire::Block"),
            ("transaction", "splora_frontend_shared::wire::Transaction"),
            (
                "address stats",
                "splora_frontend_shared::wire::AddressStats",
            ),
            ("utxo", "splora_frontend_shared::wire::Utxo"),
            (
                "recent transaction",
                "splora_frontend_shared::wire::RecentTransaction",
            ),
            (
                "mempool summary",
                "splora_frontend_shared::wire::MempoolSummary",
            ),
            (
                "fee estimates",
                "splora_frontend_shared::wire::FeeEstimates",
            ),
            (
                "broadcast result",
                "splora_frontend_shared::wire::BroadcastResult",
            ),
            (
                "test-tx result",
                "splora_frontend_shared::wire::TestTxResult",
            ),
        ];
        for (screen, shared) in required {
            assert!(
                parses.iter().any(|ty| ty.contains(shared)),
                "{screen} must deserialize with {shared}, parsed types were {parses:?}"
            );
        }
        for ty in &parses {
            assert!(
                ty.contains("splora_frontend_shared::wire::"),
                "ship-in-v1 screen parsed with a private struct: {ty}"
            );
        }
        assert!(
            parses.iter().any(|ty| ty.contains("wire::TestTxResult")),
            "test-tx result must use the shared struct so reject-reason stays optional"
        );
    }

    /// Block and transaction list rows are `BlockRow` and `TxRow` from the shared
    /// crate. A private row struct in this module does not count.
    fn builds_shared_row(source: &str, name: &str) -> bool {
        if source.contains(&format!("struct {name}")) {
            return false;
        }
        let qualified = format!("splora_frontend_shared::{name}");
        let in_use_group = source
            .find("use splora_frontend_shared::{")
            .and_then(|start| {
                let rest = &source[start..];
                rest.find("};").map(|end| &rest[..end])
            })
            .is_some_and(|group| {
                group
                    .split(|ch: char| !ch.is_ascii_alphanumeric() && ch != '_')
                    .any(|token| token == name)
            });
        let names_shared_type = source.contains(&qualified)
            || source.contains(&format!("splora_frontend_shared::rows::{name}"))
            || in_use_group;
        if !names_shared_type {
            return false;
        }
        source.contains(&format!("{qualified}::from"))
            || source.contains(&format!("{qualified} {{"))
            || source.contains(&format!("{name}::from"))
            || source.contains(&format!("{name} {{"))
    }

    #[test]
    fn screen_module_builds_block_and_tx_rows_with_shared_types() {
        let source = production_screens_source();
        assert!(
            builds_shared_row(source, "BlockRow"),
            "screen module does not use BlockRow from splora_frontend_shared"
        );
        assert!(
            builds_shared_row(source, "TxRow"),
            "screen module does not use TxRow from splora_frontend_shared"
        );
    }

    fn dashboard_bodies() -> BTreeMap<String, String> {
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
                r#"{"count":4,"vsize":500,"total_fee":90,"fee_histogram":[[2,100]]}"#.to_string(),
            ),
        ])
    }

    const RICH_TX_JSON: &str = r#"{
        "txid":"txidabc",
        "version":2,
        "locktime":0,
        "vin":[{
            "txid":"prev",
            "vout":0,
            "prevout":{
                "scriptpubkey":"0014aa",
                "scriptpubkey_asm":"OP_0",
                "scriptpubkey_type":"v0_p2wpkh",
                "scriptpubkey_address":"bc1qinput",
                "value":610677
            },
            "scriptsig":"",
            "scriptsig_asm":"",
            "is_coinbase":false,
            "sequence":4294967295
        }],
        "vout":[{
            "scriptpubkey":"0014bb",
            "scriptpubkey_asm":"OP_0",
            "scriptpubkey_type":"v0_p2wpkh",
            "scriptpubkey_address":"bc1qoutput",
            "value":344697
        }],
        "size":120,
        "weight":480,
        "sigops":1,
        "fee":10,
        "status":{"confirmed":true,"block_height":800000,"block_hash":"block-id-1","block_time":1600000000}
    }"#;

    #[test]
    fn indexer_field_dashboard_paints_median_time() {
        let (mut host, _) = host_with(Network::Mainnet, dashboard_bodies());
        host.activate(Screen::Dashboard).expect("dashboard");
        assert_painted_screen(&host, "dashboard");
        assert_view(&host, &["median_time 1700000000", "timestamp 1700000000"]);
        assert_no_banned_products(&painted_of(&host));
    }

    #[test]
    fn indexer_field_blocks_paints_median_time() {
        let (mut host, _) = host_with(
            Network::Mainnet,
            BTreeMap::from([("/blocks".to_string(), BLOCKS_JSON.to_string())]),
        );
        host.activate(Screen::Blocks).expect("blocks");
        assert_painted_screen(&host, "blocks");
        assert_view(
            &host,
            &[
                "block hash abc123def456 height 800000 tx_count 2",
                "timestamp 1700000000",
                "median_time 1700000000",
            ],
        );
    }

    /// Some(0) prints the digits. None omits the label. Do not substitute zero.
    #[test]
    fn block_option_fields_paint_zero_and_omit_missing_labels() {
        let zero = r#"{"id":"abc","height":1,"version":1,"timestamp":2,"tx_count":3,"size":4,"weight":5,"merkle_root":"mm","previousblockhash":"pp","mediantime":6,"nonce":0,"bits":0,"difficulty":0}"#;
        let zero_block: splora_frontend_shared::wire::Block =
            serde_json::from_str(zero).expect("zero block");
        for painted in [
            render_block_line(&zero_block),
            block_details_text(&zero_block),
        ] {
            assert!(painted.contains("nonce 0"), "{painted}");
            assert!(painted.contains("bits 0"), "{painted}");
            assert!(painted.contains("difficulty 0"), "{painted}");
            assert!(!painted.contains("difficulty 0.0"), "{painted}");
        }
        let omitted = r#"{"id":"abc","height":1,"version":1,"timestamp":2,"tx_count":3,"size":4,"weight":5,"merkle_root":"mm","previousblockhash":"pp","mediantime":6}"#;
        let omitted_block: splora_frontend_shared::wire::Block =
            serde_json::from_str(omitted).expect("omitted block");
        for painted in [
            render_block_line(&omitted_block),
            block_details_text(&omitted_block),
        ] {
            assert!(!painted.contains("nonce"), "{painted}");
            assert!(!painted.contains("bits"), "{painted}");
            assert!(!painted.contains("difficulty"), "{painted}");
        }
    }

    #[test]
    fn indexer_field_block_paints_tx_hash_time_and_address() {
        let (mut host, _) = host_with(
            Network::Mainnet,
            BTreeMap::from([
                (format!("/block/{BLOCK_HASH}"), BLOCK_JSON.to_string()),
                (
                    format!("/block/{BLOCK_HASH}/txs"),
                    format!("[{RICH_TX_JSON}]"),
                ),
                (
                    format!("/block/{BLOCK_HASH}/txids"),
                    r#"["txidabc"]"#.to_string(),
                ),
            ]),
        );
        host.block.query = BLOCK_HASH.to_string();
        host.activate(Screen::Block).expect("block");
        assert_painted_screen(&host, "block");
        assert_view(
            &host,
            &[
                "median_time 1700000000",
                "tx txid txidabc fee 10",
                "block_hash block-id-1",
                "block_time 1600000000",
                "input address bc1qinput value 610677",
                "output address bc1qoutput value 344697",
            ],
        );
        assert_no_banned_products(&painted_of(&host));
    }

    #[test]
    fn indexer_field_transaction_paints_hash_time_and_address() {
        let (mut host, _) = host_with(
            Network::Mainnet,
            BTreeMap::from([("/tx/txidabc".to_string(), RICH_TX_JSON.to_string())]),
        );
        host.tx.txid = "txidabc".to_string();
        host.activate(Screen::Tx).expect("tx");
        assert_painted_screen(&host, "tx");
        assert_view(
            &host,
            &[
                "tx txid txidabc fee 10 confirmed true block_height 800000",
                "block_hash block-id-1",
                "block_time 1600000000",
                "input address bc1qinput value 610677",
                "output address bc1qoutput value 344697",
            ],
        );
        assert_no_banned_products(&painted_of(&host));
    }

    #[test]
    fn indexer_field_address_paints_utxo_and_funded_sum() {
        let script = "DDogAddress111";
        let stats = r#"{"address":"DDogAddress111","chain_stats":{"tx_count":7,"funded_txo_count":4,"spent_txo_count":1,"funded_txo_sum":9000,"spent_txo_sum":1000},"mempool_stats":{"tx_count":3,"funded_txo_count":1,"spent_txo_count":0,"funded_txo_sum":2500,"spent_txo_sum":0}}"#;
        let txs = format!("[{RICH_TX_JSON}]");
        let utxo = r#"[{"txid":"utxotx444","vout":1,"status":{"confirmed":true,"block_height":10,"block_hash":"block-id-1","block_time":1600000000},"value":5000}]"#;
        let (mut host, _) = host_with(
            Network::Mainnet,
            BTreeMap::from([
                (format!("/address/{script}"), stats.to_string()),
                (format!("/address/{script}/txs"), txs),
                (format!("/address/{script}/utxo"), utxo.to_string()),
            ]),
        );
        host.address.address = script.to_string();
        host.activate(Screen::Address).expect("address");
        assert_painted_screen(&host, "address");
        assert_view(
            &host,
            &[
                "address DDogAddress111 chain_tx_count 7 funded_sum 9000 mempool_tx_count 3",
                "mempool_funded_sum 2500",
                "address tx txid txidabc",
                "fee 10",
                "block_hash block-id-1",
                "block_time 1600000000",
                "output address bc1qoutput value 344697",
                "utxo txid utxotx444 vout 1 value 5000 confirmed true",
                "block_height 10",
                "block_hash block-id-1",
                "block_time 1600000000",
            ],
        );
        assert_no_banned_products(&painted_of(&host));
    }

    #[test]
    fn indexer_fields_paint_block_tx_address_mempool_and_txid_opens_tx() {
        let rate_tx = r#"{
            "txid":"rate-tx",
            "version":2,
            "locktime":9,
            "vin":[{
                "txid":"prev",
                "vout":1,
                "prevout":{
                    "scriptpubkey":"0014scriptpubkeyhex",
                    "scriptpubkey_asm":"asmspecial",
                    "scriptpubkey_type":"v0_p2wpkh",
                    "scriptpubkey_address":"bc1qinput",
                    "value":610677
                },
                "scriptsig":"scriptsigsecret",
                "scriptsig_asm":"asmspecial",
                "witness":["abcdwitnesshex"],
                "is_coinbase":false,
                "sequence":1
            }],
            "vout":[{
                "scriptpubkey":"0014outputscriptpubkeyhex",
                "scriptpubkey_asm":"asmspecial",
                "scriptpubkey_type":"v0_p2wpkh",
                "scriptpubkey_address":"bc1qoutput",
                "value":344697
            }],
            "size":200,
            "weight":400,
            "sigops":3,
            "fee":50,
            "status":{"confirmed":true,"block_height":800000,"block_hash":"blockhash1","block_time":1700001111}
        }"#;
        let (mut host, rec) = host_with(
            Network::Mainnet,
            BTreeMap::from([
                (format!("/block/{BLOCK_HASH}"), BLOCK_JSON.to_string()),
                (format!("/block/{BLOCK_HASH}/txs"), format!("[{rate_tx}]")),
                (
                    format!("/block/{BLOCK_HASH}/txids"),
                    r#"["rate-tx"]"#.to_string(),
                ),
                ("/tx/rate-tx".to_string(), rate_tx.to_string()),
            ]),
        );
        host.block.query = BLOCK_HASH.to_string();
        host.activate(Screen::Block).expect("block");
        assert_painted_screen(&host, "block");
        assert_view(
            &host,
            &[
                "block hash abc123def456 height 800000 tx_count 2 timestamp 1700000000 size 200 weight 800 previous_hash pp median_time 1700000000 version 1 merkle_root mm nonce 1 bits 2 difficulty 3",
                "tx txid rate-tx",
            ],
        );
        let block_painted = painted_of(&host);
        assert!(!block_painted.contains("difficulty 3.0"), "{block_painted}");
        let block_lower = block_painted.to_ascii_lowercase();
        for word in ["chart", "canvas", "svg"] {
            assert!(!block_lower.contains(word), "{block_painted}");
        }
        assert_no_private_key(&block_painted);

        let rows = host.painted_rows();
        let tx_row = rows
            .iter()
            .find(|row| row.starts_with("row tx "))
            .expect("tx row");
        assert_eq!(tx_row, "row tx txid rate-tx fee 50");
        assert_eq!(transaction_id_in_row(tx_row), Some("rate-tx"));
        assert_eq!(transaction_id_in_row("row tx txid abc fee 10"), Some("abc"));
        let block_row = rows
            .iter()
            .find(|row| row.starts_with("row block "))
            .expect("block row");
        assert_eq!(transaction_id_in_row(block_row), None);
        assert_eq!(
            transaction_id_in_row("row block height 10 hash aaaa tx_count 2 time 1000"),
            None
        );
        assert_eq!(transaction_id_in_row("row utxo value 5000"), None);
        assert_eq!(host.screen, Screen::Block);
        assert!(host.open_transaction("").is_err());
        assert!(host.open_transaction("aa/bb").is_err());
        assert_eq!(host.screen, Screen::Block);
        let txid = transaction_id_in_row(tx_row).expect("txid").to_string();
        host.open_transaction(&txid).expect("open tx");
        assert_eq!(host.screen, Screen::Tx);
        assert_ne!(host.screen, Screen::Block);
        assert!(rec.get_paths().iter().any(|path| path == "/tx/rate-tx"));
        assert_painted_screen(&host, "tx");
        assert_view(
            &host,
            &[
                "tx txid rate-tx fee 50 confirmed true block_height 800000 size 200 weight 400 block_hash blockhash1 block_time 1700001111 version 2 locktime 9 sigops 3 fee_rate 0.5",
                "input outpoint prev:1",
                "input address bc1qinput value 610677",
                "output address bc1qoutput value 344697",
            ],
        );
        let tx_painted = painted_of(&host);
        for secret in [
            "0014scriptpubkeyhex",
            "0014outputscriptpubkeyhex",
            "abcdwitnesshex",
            "scriptsigsecret",
            "asmspecial",
            "scriptpubkey",
            "witness",
        ] {
            assert!(!tx_painted.contains(secret), "{secret} in {tx_painted}");
        }
        assert_no_private_key(&tx_painted);

        let zero_tx = r#"{
            "txid":"zerowt",
            "version":1,
            "locktime":0,
            "vin":[{
                "txid":"lonely",
                "vout":4,
                "prevout":null,
                "scriptsig":"",
                "scriptsig_asm":"",
                "is_coinbase":true,
                "sequence":0
            }],
            "vout":[],
            "size":10,
            "weight":0,
            "sigops":0,
            "fee":25,
            "status":{"confirmed":false}
        }"#;
        let (mut host, _) = host_with(
            Network::Mainnet,
            BTreeMap::from([("/tx/zerowt".to_string(), zero_tx.to_string())]),
        );
        host.tx.txid = "zerowt".to_string();
        host.activate(Screen::Tx).expect("zero weight");
        assert_painted_screen(&host, "tx");
        assert_view(
            &host,
            &[
                "tx txid zerowt",
                "weight 0",
                "fee 25",
                "input outpoint lonely:4",
            ],
        );
        let zero_painted = painted_of(&host);
        assert!(!zero_painted.contains("fee_rate"), "{zero_painted}");
        assert!(!zero_painted.contains("input address"), "{zero_painted}");
        assert_no_private_key(&zero_painted);

        let stats = r#"{"address":"DDogAddress111","chain_stats":{"tx_count":7,"funded_txo_count":4,"spent_txo_count":1,"funded_txo_sum":9000,"spent_txo_sum":1000},"mempool_stats":{"tx_count":3,"funded_txo_count":8,"spent_txo_count":2,"funded_txo_sum":2500,"spent_txo_sum":600}}"#;
        let script = "DDogAddress111";
        let (mut host, _) = host_with(
            Network::Mainnet,
            BTreeMap::from([
                (format!("/address/{script}"), stats.to_string()),
                (format!("/address/{script}/txs"), format!("[{rate_tx}]")),
                (
                    format!("/address/{script}/utxo"),
                    r#"[{"txid":"utxotx444","vout":1,"status":{"confirmed":true,"block_height":10},"value":5000}]"#.to_string(),
                ),
            ]),
        );
        host.address.address = script.to_string();
        host.activate(Screen::Address).expect("address");
        assert_painted_screen(&host, "address");
        assert_view(
            &host,
            &[
                "address DDogAddress111 chain_tx_count 7 funded_sum 9000 mempool_tx_count 3 mempool_funded_sum 2500 chain_funded_txo_count 4 chain_spent_txo_count 1 chain_spent_txo_sum 1000 mempool_funded_txo_count 8 mempool_spent_txo_count 2 mempool_spent_txo_sum 600",
                "address tx txid rate-tx confirmed true block_height 800000 fee 50 size 200 weight 400 block_hash blockhash1 block_time 1700001111 version 2 locktime 9 sigops 3 fee_rate 0.5",
                "utxo txid utxotx444 vout 1 value 5000 confirmed true",
            ],
        );
        let address_painted = painted_of(&host);
        assert!(
            address_painted.contains(
                "address DDogAddress111 chain_tx_count 7 funded_sum 9000 mempool_tx_count 3"
            ),
            "{address_painted}"
        );
        assert!(
            address_painted.contains("mempool_funded_sum 2500"),
            "{address_painted}"
        );
        assert!(
            host.painted_rows()
                .iter()
                .any(|row| row == "row utxo value 5000")
        );
        assert!(host.painted_rows().iter().all(|row| {
            if row.starts_with("row utxo ") {
                transaction_id_in_row(row).is_none()
            } else {
                true
            }
        }));
        assert_no_private_key(&address_painted);

        let recent = r#"[{"txid":"recent-fast","fee":200,"vsize":50,"value":9},{"txid":"recent-zero","fee":80,"vsize":0,"value":1}]"#;
        let (mut host, _) = host_with(
            Network::Mainnet,
            BTreeMap::from([
                (
                    "/mempool".to_string(),
                    r#"{"count":4,"vsize":500,"total_fee":90,"fee_histogram":[[10.5,200]]}"#
                        .to_string(),
                ),
                ("/mempool/recent".to_string(), recent.to_string()),
            ]),
        );
        host.activate(Screen::Mempool).expect("mempool");
        assert_painted_screen(&host, "mempool");
        assert_view(
            &host,
            &[
                "mempool count 4 vsize 500 total_fee 90",
                "fee_histogram 10.5 200",
                "The incoming virtual-bytes-per-second rate is not in the payload.",
                "recent txid recent-fast fee 200 vsize 50 value 9 fee_rate 4",
            ],
        );
        let mempool_painted = painted_of(&host);
        let zero_recent: Vec<_> = mempool_painted
            .lines()
            .filter(|line| line.starts_with("recent ") && line.contains("recent-zero"))
            .collect();
        assert_eq!(
            zero_recent,
            vec!["recent txid recent-zero fee 80 vsize 0 value 1"]
        );
        assert_eq!(
            lines_with(&mempool_painted, "row tx "),
            vec![
                "row tx txid recent-fast fee 200",
                "row tx txid recent-zero fee 80",
            ]
        );
        for line in lines_with(&mempool_painted, "row tx ") {
            assert!(!line.contains("fee_rate"), "{line}");
        }
        assert_no_private_key(&mempool_painted);

        let (mut host, _) = host_with(
            Network::Mainnet,
            BTreeMap::from([
                ("/blocks/tip/height".to_string(), "100".to_string()),
                ("/blocks/tip/hash".to_string(), "abc".to_string()),
                ("/blocks".to_string(), "[]".to_string()),
                ("/mempool/recent".to_string(), "[]".to_string()),
                (
                    "/fee-estimates".to_string(),
                    r#"{"1":12.5,"6":5}"#.to_string(),
                ),
                (
                    "/mempool".to_string(),
                    r#"{"count":0,"vsize":0,"total_fee":0,"fee_histogram":[]}"#.to_string(),
                ),
            ]),
        );
        host.activate(Screen::Dashboard).expect("dashboard");
        assert_painted_screen(&host, "dashboard");
        assert_view(&host, &["fee target 1 rate 12.5", "fee target 6 rate 5"]);
        assert_no_private_key(&painted_of(&host));

        let render = source_fn(include_str!("main.rs"), "fn render(", "\nfn main");
        assert!(
            render.contains("open_transaction"),
            "shell row click does not open the transaction: {render}"
        );
    }

    #[test]
    fn indexer_field_multi_address_paints_mempool_funded_sum() {
        let stats = r#"{"address":"addrA","chain_stats":{"tx_count":2,"funded_txo_count":2,"spent_txo_count":0,"funded_txo_sum":20,"spent_txo_sum":0},"mempool_stats":{"tx_count":1,"funded_txo_count":1,"spent_txo_count":0,"funded_txo_sum":2500,"spent_txo_sum":0}}"#;
        let (mut host, rec) = host_with(
            Network::Mainnet,
            BTreeMap::from([
                ("/address/addrA".to_string(), stats.to_string()),
                (
                    "/address/addrA/utxo".to_string(),
                    ADDR_A_UTXO_JSON.to_string(),
                ),
            ]),
        );
        host.multi_address.input = "addrA".to_string();
        host.activate(Screen::MultiAddress).expect("multi");
        assert_eq!(
            rec.get_paths(),
            vec![
                "/address/addrA".to_string(),
                "/address/addrA/utxo".to_string(),
            ]
        );
        assert_painted_screen(&host, "addresses");
        assert_view(
            &host,
            &[
                "address addrA chain_tx_count 2 funded_sum 20 mempool_tx_count 1",
                "mempool_funded_sum 2500",
                "address addrA utxo txid utxoa vout 0 value 4242",
            ],
        );
    }

    #[test]
    fn indexer_field_multi_address_paints_per_address_utxo_value() {
        let stats_a = r#"{"address":"addrA","chain_stats":{"tx_count":2,"funded_txo_count":2,"spent_txo_count":0,"funded_txo_sum":20,"spent_txo_sum":0},"mempool_stats":{"tx_count":1,"funded_txo_count":1,"spent_txo_count":0,"funded_txo_sum":2500,"spent_txo_sum":0}}"#;
        let stats_b = r#"{"address":"addrB","chain_stats":{"tx_count":4,"funded_txo_count":4,"spent_txo_count":1,"funded_txo_sum":40,"spent_txo_sum":5},"mempool_stats":{"tx_count":0,"funded_txo_count":0,"spent_txo_count":0,"funded_txo_sum":0,"spent_txo_sum":0}}"#;
        let utxo_a = r#"[{"txid":"utxoa","vout":0,"status":{"confirmed":true,"block_height":11},"value":4242}]"#;
        let utxo_b = r#"[{"txid":"utxob","vout":2,"status":{"confirmed":false},"value":9090}]"#;
        let (mut host, rec) = host_with(
            Network::Mainnet,
            BTreeMap::from([
                ("/address/addrA".to_string(), stats_a.to_string()),
                ("/address/addrB".to_string(), stats_b.to_string()),
                ("/address/addrA/utxo".to_string(), utxo_a.to_string()),
                ("/address/addrB/utxo".to_string(), utxo_b.to_string()),
            ]),
        );
        host.multi_address.input = "addrA, addrB".to_string();
        host.activate(Screen::MultiAddress).expect("multi");
        assert_painted_screen(&host, "addresses");
        assert_view(
            &host,
            &[
                "address addrA chain_tx_count 2 funded_sum 20 mempool_tx_count 1",
                "mempool_funded_sum 2500",
                "address addrA utxo txid utxoa vout 0 value 4242",
                "address addrB chain_tx_count 4 funded_sum 40 mempool_tx_count 0",
                "address addrB utxo txid utxob vout 2 value 9090",
            ],
        );
        let painted = painted_of(&host);
        let value_a = painted
            .find("address addrA utxo txid utxoa vout 0 value 4242")
            .expect("addrA utxo value");
        let value_b = painted
            .find("address addrB utxo txid utxob vout 2 value 9090")
            .expect("addrB utxo value");
        assert!(value_a < value_b, "{painted}");
        assert_eq!(
            rec.get_paths(),
            vec![
                "/address/addrA".to_string(),
                "/address/addrA/utxo".to_string(),
                "/address/addrB".to_string(),
                "/address/addrB/utxo".to_string(),
            ]
        );
    }

    #[test]
    fn indexer_field_test_transactions_paint_fee() {
        let body = r#"[{"txid":"test-tx-1","wtxid":"test-wtxid-1","allowed":false,"vsize":141,"fees":{"base":0.00001,"effective-feerate":1.0,"effective-includes":["test-tx-1"]},"reject-reason":"min relay fee not met"}]"#;
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
        assert!(rec.get_paths().is_empty());
        assert_painted_screen(&host, "test transactions");
        assert_view(
            &host,
            &[
                "test txid test-tx-1 allowed false",
                "fee 0.00001",
                "effective_feerate 1",
                "reject-reason min relay fee not met",
            ],
        );
        assert_no_banned_products(&painted_of(&host));
    }

    fn lines_with<'a>(text: &'a str, prefix: &str) -> Vec<&'a str> {
        text.lines()
            .filter(|line| line.starts_with(prefix))
            .collect()
    }

    fn assert_no_private_key(text: &str) {
        assert!(!text.contains("nsec"), "{text}");
        assert!(!text.to_ascii_lowercase().contains("private key"), "{text}");
        assert!(!text.contains("private-key"), "{text}");
    }

    /// Dashboard, blocks, block, transaction, address, and mempool each paint
    /// one row per block or transaction. A flat field sentence is not a row.
    #[test]
    fn painted_shell_paints_one_row_per_block_or_transaction() {
        let blocks = r#"[{"id":"aaaa","height":10,"version":1,"timestamp":1000,"tx_count":2,"size":1,"weight":1,"merkle_root":"m","previousblockhash":"p","mediantime":1,"nonce":1,"bits":1,"difficulty":1.0},{"id":"bbbb","height":11,"version":1,"timestamp":2000,"tx_count":3,"size":1,"weight":1,"merkle_root":"m","mediantime":1,"nonce":1,"bits":1,"difficulty":1.0}]"#;
        let recent = r#"[{"txid":"txxx","fee":5,"vsize":10,"value":1},{"txid":"txxy","fee":9,"vsize":11,"value":2}]"#;
        let (mut host, _) = host_with(
            Network::Mainnet,
            BTreeMap::from([
                ("/blocks".to_string(), blocks.to_string()),
                ("/mempool/recent".to_string(), recent.to_string()),
                ("/blocks/tip/height".to_string(), "11".to_string()),
                ("/blocks/tip/hash".to_string(), "bbbb".to_string()),
                ("/fee-estimates".to_string(), r#"{"2":1.1}"#.to_string()),
                (
                    "/mempool".to_string(),
                    r#"{"count":2,"vsize":21,"total_fee":14,"fee_histogram":[]}"#.to_string(),
                ),
            ]),
        );
        host.activate(Screen::Dashboard).expect("dashboard");
        let painted = painted_of(&host);
        assert_eq!(
            lines_with(&painted, "row block "),
            vec![
                "row block height 10 hash aaaa tx_count 2 time 1000",
                "row block height 11 hash bbbb tx_count 3 time 2000",
            ],
            "painted view has no row per block: {painted}"
        );
        assert_eq!(
            lines_with(&painted, "row tx "),
            vec!["row tx txid txxx fee 5", "row tx txid txxy fee 9"],
            "painted view has no row per transaction: {painted}"
        );
        for line in lines_with(&painted, "row block ") {
            assert!(!line.contains("size "), "{line}");
            assert!(!line.contains("weight "), "{line}");
            assert!(!line.contains("previous_hash"), "{line}");
            assert!(!line.contains("median_time"), "{line}");
        }
        assert_no_private_key(&painted);
        assert_no_banned_products(&painted);

        let (mut host, _) = host_with(
            Network::Mainnet,
            BTreeMap::from([("/blocks".to_string(), blocks.to_string())]),
        );
        host.activate(Screen::Blocks).expect("blocks");
        let painted = painted_of(&host);
        assert_eq!(
            lines_with(&painted, "row block "),
            vec![
                "row block height 10 hash aaaa tx_count 2 time 1000",
                "row block height 11 hash bbbb tx_count 3 time 2000",
            ],
            "blocks list is one flat sentence: {painted}"
        );
        assert!(lines_with(&painted, "row tx ").is_empty(), "{painted}");
        assert_no_private_key(&painted);
        assert_no_banned_products(&painted);

        let block = r#"{"id":"aaaa","height":10,"version":1,"timestamp":1000,"tx_count":2,"size":1,"weight":1,"merkle_root":"m","previousblockhash":"p","mediantime":1,"nonce":1,"bits":1,"difficulty":1.0}"#;
        let txs = r#"[{"txid":"one","version":1,"locktime":0,"vin":[],"vout":[],"size":1,"weight":1,"sigops":0,"fee":15,"status":{"confirmed":true}},{"txid":"two","version":1,"locktime":0,"vin":[],"vout":[],"size":1,"weight":1,"sigops":0,"fee":25,"status":{"confirmed":true}}]"#;
        let (mut host, _) = host_with(
            Network::Mainnet,
            BTreeMap::from([
                ("/block/aaaa".to_string(), block.to_string()),
                ("/block/aaaa/txs".to_string(), txs.to_string()),
                (
                    "/block/aaaa/txids".to_string(),
                    r#"["one","two"]"#.to_string(),
                ),
            ]),
        );
        host.block.query = "aaaa".to_string();
        host.activate(Screen::Block).expect("block");
        let painted = painted_of(&host);
        assert_eq!(
            lines_with(&painted, "row block height "),
            vec!["row block height 10 hash aaaa tx_count 2 time 1000"],
            "block screen is one flat sentence: {painted}"
        );
        assert_eq!(
            lines_with(&painted, "row tx "),
            vec!["row tx txid one fee 15", "row tx txid two fee 25"],
            "block transactions are one flat sentence: {painted}"
        );
        assert_no_private_key(&painted);
        assert_no_banned_products(&painted);

        let (mut host, _) = host_with(
            Network::Mainnet,
            BTreeMap::from([("/tx/txidabc".to_string(), TX_JSON.to_string())]),
        );
        host.tx.txid = "txidabc".to_string();
        host.activate(Screen::Tx).expect("tx");
        let painted = painted_of(&host);
        assert_eq!(
            lines_with(&painted, "row tx "),
            vec!["row tx txid txidabc fee 10"],
            "transaction screen is one flat sentence: {painted}"
        );
        assert_no_private_key(&painted);
        assert_no_banned_products(&painted);

        let utxos = r#"[{"txid":"utxotx444","vout":1,"status":{"confirmed":true,"block_height":10},"value":5000},{"txid":"utxotx555","vout":0,"status":{"confirmed":false},"value":7000}]"#;
        let script = "DDogAddress111";
        let (mut host, _) = host_with(
            Network::Mainnet,
            BTreeMap::from([
                (format!("/address/{script}"), ADDRESS_JSON.to_string()),
                (
                    format!("/address/{script}/txs"),
                    ADDRESS_TXS_JSON.to_string(),
                ),
                (format!("/address/{script}/utxo"), utxos.to_string()),
            ]),
        );
        host.address.address = script.to_string();
        host.activate(Screen::Address).expect("address");
        let painted = painted_of(&host);
        assert!(
            painted.contains("funded_sum 9000"),
            "address omits funded sum: {painted}"
        );
        assert_eq!(
            lines_with(&painted, "row address "),
            vec!["row address funded_sum 9000"],
            "address funded sum is not its own row: {painted}"
        );
        assert_eq!(
            lines_with(&painted, "row utxo "),
            vec!["row utxo value 5000", "row utxo value 7000"],
            "address does not show each utxo value: {painted}"
        );
        assert_eq!(
            lines_with(&painted, "row tx "),
            vec!["row tx txid addresstx333 fee 1"],
            "address transaction is one flat sentence: {painted}"
        );
        assert_no_private_key(&painted);
        assert_no_banned_products(&painted);

        let (mut host, _) = host_with(
            Network::Mainnet,
            BTreeMap::from([
                ("/address/addrA".to_string(), ADDR_A_JSON.to_string()),
                ("/address/addrB".to_string(), ADDR_B_JSON.to_string()),
                (
                    "/address/addrA/utxo".to_string(),
                    ADDR_A_UTXO_JSON.to_string(),
                ),
                (
                    "/address/addrB/utxo".to_string(),
                    ADDR_B_UTXO_JSON.to_string(),
                ),
            ]),
        );
        host.multi_address.input = "addrA, addrB".to_string();
        host.activate(Screen::MultiAddress).expect("multi");
        let painted = painted_of(&host);
        assert_eq!(
            lines_with(&painted, "row utxo "),
            vec!["row utxo value 4242", "row utxo value 9090"],
            "multi-address does not show each utxo value: {painted}"
        );
        assert_eq!(
            lines_with(&painted, "row address "),
            vec!["row address funded_sum 20", "row address funded_sum 40",],
            "{painted}"
        );
        assert_no_private_key(&painted);
        assert_no_banned_products(&painted);

        let (mut host, rec) = host_with(
            Network::Mainnet,
            BTreeMap::from([
                (
                    splora_frontend_shared::mempool_path().to_string(),
                    r#"{"count":2,"vsize":21,"total_fee":14,"fee_histogram":[]}"#.to_string(),
                ),
                (
                    splora_frontend_shared::mempool_recent_path().to_string(),
                    recent.to_string(),
                ),
            ]),
        );
        host.activate(Screen::Mempool).expect("mempool");
        assert_eq!(
            rec.get_paths(),
            vec![
                splora_frontend_shared::mempool_path().to_string(),
                splora_frontend_shared::mempool_recent_path().to_string(),
                splora_frontend_shared::mempool_txids_path().to_string(),
            ]
        );
        let painted = painted_of(&host);
        assert_eq!(
            lines_with(&painted, "row tx "),
            vec!["row tx txid txxx fee 5", "row tx txid txxy fee 9"],
            "mempool recent transactions are one flat sentence: {painted}"
        );
        assert_no_private_key(&painted);
        assert_no_banned_products(&painted);

        let render = source_fn(include_str!("main.rs"), "fn render(", "\nfn main");
        assert!(
            render.contains("for row in painted_rows("),
            "Shell::render paints one flat sentence and no row per block or transaction:\n{render}"
        );
        assert!(
            !render.to_ascii_lowercase().contains("private key"),
            "{render}"
        );
        assert!(!render.contains("private-key"), "{render}");
        assert!(!render.contains("nsec"), "{render}");
    }
    #[test]
    fn calculator_typing_updates_bitcoin_satoshis_and_yen() {
        let (mut host, rec) = host_with(Network::Mainnet, BTreeMap::new());
        host.activate(Screen::Calculator).expect("calculator");
        assert!(rec.get_paths().is_empty(), "{:?}", rec.get_paths());
        assert!(rec.post_paths().is_empty());
        assert_eq!(host.screen, Screen::Calculator);

        let usd = splora_frontend_shared::loaded_calculator(splora_frontend_shared::FiatCode::Usd);
        let from_fiat = splora_frontend_shared::enter_fiat(&usd, "100").expect("100 usd");
        let fixed = calculator_screen_text();
        assert!(
            !fixed.contains(&format!("entered bitcoin {}", from_fiat.btc)),
            "fixed calculator already paints typed bitcoin"
        );

        host.enter_calculator_amount("fiat", "100");
        let painted = host.rendered();
        assert!(
            painted.contains(&format!("entered bitcoin {}", from_fiat.btc)),
            "entering fiat did not change bitcoin: {painted}"
        );
        assert!(
            painted.contains(&format!("entered satoshis {}", from_fiat.sats)),
            "entering fiat did not change satoshis: {painted}"
        );
        assert_ne!(from_fiat.btc, usd.btc);
        assert_ne!(from_fiat.sats, usd.sats);

        let from_btc = splora_frontend_shared::enter_btc(&from_fiat, "2");
        host.enter_calculator_amount("bitcoin", "2");
        let painted = host.rendered();
        assert!(
            painted.contains(&format!("entered fiat {}", from_btc.fiat)),
            "entering bitcoin did not change fiat: {painted}"
        );
        assert!(
            painted.contains(&format!("entered satoshis {}", from_btc.sats)),
            "entering bitcoin did not change satoshis: {painted}"
        );
        assert_ne!(from_btc.fiat, from_fiat.fiat);
        assert_ne!(from_btc.sats, from_fiat.sats);

        let btc_from_sats = splora_frontend_shared::sats_to_btc(50_000);
        let from_sats = splora_frontend_shared::enter_btc(&from_btc, &btc_from_sats);
        host.enter_calculator_amount("satoshis", "50000");
        let painted = host.rendered();
        assert!(
            painted.contains(&format!("entered bitcoin {}", from_sats.btc)),
            "entering satoshis did not change bitcoin: {painted}"
        );
        assert!(
            painted.contains(&format!("entered fiat {}", from_sats.fiat)),
            "entering satoshis did not change fiat: {painted}"
        );
        assert_ne!(from_sats.btc, from_btc.btc);
        assert_ne!(from_sats.fiat, from_btc.fiat);

        host.choose_calculator_fiat(splora_frontend_shared::FiatCode::Jpy);
        host.enter_calculator_amount("bitcoin", "1");
        let yen = splora_frontend_shared::btc_fiat_jpy("1");
        assert_eq!(yen, "\u{00A5}11,057,757");
        let painted = host.rendered();
        assert!(
            painted.contains(&format!("chosen {yen}")),
            "JPY choice did not use the fixture yen rate: {painted}"
        );
        assert!(
            !calculator_screen_text().contains("chosen "),
            "chosen line must come from typing, not the fixed calculator string"
        );
        assert!(!painted.contains("Fiat price last updated"), "{painted}");
        assert_no_private_key(&painted);
        assert_eq!(host.screen, Screen::Calculator);

        let source = production_screens_source();
        assert!(
            source.contains("enter_fiat("),
            "native calculator does not call enter_fiat"
        );
        assert!(
            source.contains("enter_btc("),
            "native calculator does not call enter_btc"
        );
        assert!(
            source.contains("sats_to_btc("),
            "satoshi entry does not convert through sats_to_btc"
        );
        assert!(
            !source.contains("fn enter_fiat(") && !source.contains("fn enter_btc("),
            "shared enter functions were copied into the native shell"
        );
        assert!(
            !source.contains("FiatCode::Usd") && !source.contains("FiatCode::Jpy"),
            "native crate copied the USD versus JPY match"
        );
    }

    /// Shell keys call `push_str` for characters and `submit` to commit.
    /// Draft tokens are `fiat <amount>`, `bitcoin <amount>`, `satoshis <amount>`, and `jpy`.
    /// The `jpy` token calls `choose_calculator_fiat` for the fixture yen rate.
    #[test]
    fn calculator_shell_submit_updates_bitcoin_satoshis_and_yen() {
        let (mut host, rec) = host_with(Network::Mainnet, BTreeMap::new());
        host.activate(Screen::Calculator).expect("calculator");
        assert!(rec.get_paths().is_empty(), "{:?}", rec.get_paths());
        assert!(rec.post_paths().is_empty());
        assert_eq!(host.screen, Screen::Calculator);

        let usd = splora_frontend_shared::loaded_calculator(splora_frontend_shared::FiatCode::Usd);
        let from_fiat = splora_frontend_shared::enter_fiat(&usd, "100").expect("100 usd");
        host.push_str("fiat 100");
        let _ = host.submit();
        let painted = host.rendered();
        assert!(
            painted.contains(&format!("entered bitcoin {}", from_fiat.btc)),
            "shell submit of fiat did not change bitcoin: {painted}"
        );
        assert!(
            painted.contains(&format!("entered satoshis {}", from_fiat.sats)),
            "shell submit of fiat did not change satoshis: {painted}"
        );

        let from_btc = splora_frontend_shared::enter_btc(&from_fiat, "2");
        host.push_str("bitcoin 2");
        let _ = host.submit();
        let painted = host.rendered();
        assert!(
            painted.contains(&format!("entered fiat {}", from_btc.fiat)),
            "shell submit of bitcoin did not change fiat: {painted}"
        );
        assert!(
            painted.contains(&format!("entered satoshis {}", from_btc.sats)),
            "shell submit of bitcoin did not change satoshis: {painted}"
        );

        let btc_from_sats = splora_frontend_shared::sats_to_btc(50_000);
        let from_sats = splora_frontend_shared::enter_btc(&from_btc, &btc_from_sats);
        host.push_str("satoshis 50000");
        let _ = host.submit();
        let painted = host.rendered();
        assert!(
            painted.contains(&format!("entered bitcoin {}", from_sats.btc)),
            "shell submit of satoshis did not change bitcoin: {painted}"
        );
        assert!(
            painted.contains(&format!("entered fiat {}", from_sats.fiat)),
            "shell submit of satoshis did not change fiat: {painted}"
        );

        host.push_str("jpy");
        let _ = host.submit();
        host.push_str("bitcoin 1");
        let _ = host.submit();
        let yen = splora_frontend_shared::btc_fiat_jpy("1");
        assert_eq!(yen, "\u{00A5}11,057,757");
        let painted = host.rendered();
        assert!(
            painted.contains(&format!("chosen {yen}")),
            "shell jpy token did not use the fixture yen rate: {painted}"
        );
        assert!(!painted.contains("Fiat price last updated"), "{painted}");
        assert_no_private_key(&painted);
        assert_eq!(host.screen, Screen::Calculator);

        let main = include_str!("main.rs");
        let start = main.find("on_key_down").expect("key handler");
        let end = main[start..]
            .find("cx.notify()")
            .expect("end of key handler");
        let handler = &main[start..start + end];
        assert!(
            handler.contains("Screen::Calculator"),
            "main.rs key handler does not reach the calculator: {handler}"
        );
        assert!(
            handler.contains("submit()"),
            "main.rs key handler does not submit the calculator: {handler}"
        );
    }

    #[test]
    fn calculator_fiat_screens_call_shared_functions() {
        let (mut host, rec) = host_with(Network::Mainnet, BTreeMap::new());
        host.activate(Screen::Calculator).expect("calculator");
        assert!(rec.get_paths().is_empty(), "{:?}", rec.get_paths());
        assert!(rec.post_paths().is_empty());
        assert_painted_screen(&host, "calculator");
        assert_view(&host, &["100000000", "1.12345678", "21000000"]);
        assert_no_private_key(&painted_of(&host));

        host.activate(Screen::Fiat).expect("fiat");
        assert!(rec.get_paths().is_empty(), "{:?}", rec.get_paths());
        assert!(rec.post_paths().is_empty());
        assert_painted_screen(&host, "fiat");
        assert_view(&host, &["$123456", "\u{00A5}11057757"]);
        let jpy = jpy_fixture_text();
        assert_eq!(jpy.chars().next(), Some('¥'));
        assert!(!jpy.contains('.'));
        assert!(!jpy.contains('\u{FFE5}'));
        assert_eq!(jpy, "\u{00A5}11057757");
        assert_eq!(switched_fiat_symbols(), ['$', '\u{00A5}', '$']);
        assert_eq!(usd_fixture_text(), "$123456");
        assert_no_private_key(&painted_of(&host));

        for (screen, label, page) in [
            (Screen::Statistics, "statistics", statistics_page_text()),
            (Screen::Mining, "mining", mining_page_text()),
            (Screen::Lightning, "lightning", lightning_page_text()),
        ] {
            let (mut page_host, page_rec) = host_with(Network::Mainnet, BTreeMap::new());
            page_host.activate(screen).expect(label);
            assert!(
                page_rec.get_paths().is_empty(),
                "{label} {:?}",
                page_rec.get_paths()
            );
            assert!(page_rec.post_paths().is_empty(), "{label}");
            assert_eq!(page, "not-served");
            assert_painted_screen(&page_host, label);
            assert_view(&page_host, &["not-served"]);
            let painted = painted_of(&page_host);
            assert_no_private_key(&painted);
            let lower = page_host.rendered().to_ascii_lowercase();
            for banned in ["chart", "canvas", "svg", "channel"] {
                assert!(!lower.contains(banned), "{label} contains {banned}");
            }
        }

        let source = production_screens_source();
        for name in [
            "btc_to_sats",
            "truncate_btc_decimals",
            "cap_btc_supply",
            "format_fiat",
            "btc_fiat_usd",
            "btc_fiat_jpy",
            "sats_fiat_usd",
            "switch_usd_jpy_usd",
            "route_availability",
        ] {
            assert!(
                source.contains(&format!("{name}(")),
                "{name} is not called from the shell screens"
            );
            assert!(
                !source.contains(&format!("fn {name}(")),
                "{name} is copied into the native shell"
            );
        }
        let render = source_fn(include_str!("main.rs"), "fn render(", "\nfn main");
        let render_compact: String = render.chars().filter(|ch| !ch.is_whitespace()).collect();
        assert!(render_compact.contains(".child(self.body_text())"));
        assert!(render.contains("screen_choices"));
        assert!(!render.to_ascii_lowercase().contains("private key"));
        assert!(!render.contains("private-key"));
    }

    #[test]
    fn painted_calculator_includes_shared_fixture_fiat_amounts() {
        let third = splora_frontend_shared::btc_fiat_usd("0.33");
        let one_sat = splora_frontend_shared::btc_fiat_usd("0.00000001");
        let yen = splora_frontend_shared::btc_fiat_jpy("1");
        let ten_thousand_sats = splora_frontend_shared::sats_fiat_usd(10_000);
        assert_eq!(third, "$40,740.48");
        assert_eq!(one_sat, "$0.00");
        assert_eq!(yen, "\u{00A5}11,057,757");
        assert_eq!(ten_thousand_sats, "$12.35");

        let (mut host, rec) = host_with(Network::Mainnet, BTreeMap::new());
        host.activate(Screen::Calculator).expect("calculator");
        assert!(rec.get_paths().is_empty(), "{:?}", rec.get_paths());
        assert!(rec.post_paths().is_empty());
        assert_painted_screen(&host, "calculator");
        let painted = painted_of(&host);
        let rendered = host.rendered();
        for amount in [&third, &one_sat, &yen, &ten_thousand_sats] {
            assert!(
                painted.contains(amount.as_str()),
                "painted calculator missing shared fiat {amount}: {painted}"
            );
            assert!(
                rendered.contains(amount.as_str()),
                "calculator render missing shared fiat {amount}: {rendered}"
            );
        }
        assert_no_private_key(&painted);
        assert!(
            painted.contains("A live price feed is not served."),
            "{painted}"
        );
        assert!(painted.contains("Copy"), "{painted}");
        assert!(painted.contains("₿"), "{painted}");

        let body = source_fn(
            production_screens_source(),
            "fn calculator_screen_text(",
            "fn fiat_screen_text(",
        );
        for call in [
            "btc_fiat_usd(",
            "btc_fiat_jpy(",
            "sats_fiat_usd(",
            "route_availability(\"/v1/prices\")",
        ] {
            assert!(
                body.contains(call),
                "calculator_screen_text does not call {call}"
            );
        }
        for literal in ["$40,740.48", "$0.00", "11,057,757", "$12.35"] {
            assert!(
                !body.contains(literal),
                "calculator_screen_text copies {literal} instead of calling shared fiat"
            );
        }
    }

    fn page_fn_body<'a>(source: &'a str, name: &str) -> &'a str {
        let signature = format!("fn {name}(");
        let start = source
            .find(&signature)
            .unwrap_or_else(|| panic!("missing {name}"));
        let rest = &source[start..];
        let end = rest
            .find("\n}\n")
            .unwrap_or_else(|| panic!("missing end of {name}"));
        &rest[..end]
    }

    #[test]
    fn later_routes_show_the_shared_not_served_sentence() {
        let cases = [
            (
                Screen::Statistics,
                "statistics",
                statistics_page_text(),
                "/api/v1/statistics",
            ),
            (Screen::Mining, "mining", mining_page_text(), "/v1/mining"),
            (
                Screen::Graphs,
                "graphs",
                graphs_page_text(),
                "/api/v1/statistics",
            ),
            (
                Screen::Hashrate,
                "hashrate",
                hashrate_page_text(),
                "/v1/mining",
            ),
            (
                Screen::PriceChart,
                "price chart",
                price_chart_page_text(),
                "/api/v1/historical-price",
            ),
            (
                Screen::Lightning,
                "lightning",
                lightning_page_text(),
                "/lightning",
            ),
        ];
        for (screen, label, page, path) in cases {
            let shared = splora_frontend_shared::route_availability(path);
            assert_eq!(page, shared, "{label}");
            let (mut host, rec) = host_with(Network::Mainnet, BTreeMap::new());
            host.activate(screen).expect(label);
            assert!(
                rec.get_paths().is_empty(),
                "{label} fetched {:?}",
                rec.get_paths()
            );
            assert!(rec.post_paths().is_empty(), "{label}");
            assert_eq!(host.query_text(), "", "{label}");
            assert!(host.painted_rows().is_empty(), "{label}");
            assert_painted_screen(&host, label);
            assert_eq!(host.rendered(), shared, "{label}");
            assert_view(&host, &[shared]);
            let painted = painted_of(&host);
            assert_no_private_key(&painted);
            assert!(!painted.contains("mempool.space"), "{painted}");
            let body = host.rendered();
            let lower = body.to_ascii_lowercase();
            for banned in ["canvas", "svg", "polyline", "channel", "mempool.space"] {
                assert!(!lower.contains(banned), "{label} contains {banned}: {body}");
            }
            assert!(
                !lower.contains("chart"),
                "{label} body paints a chart: {body}"
            );
            if label == "price chart" {
                let without_label = painted.replace("screen: price chart", "screen: price");
                assert!(
                    !without_label.to_ascii_lowercase().contains("chart"),
                    "{painted}"
                );
            } else {
                assert!(!painted.to_ascii_lowercase().contains("chart"), "{painted}");
            }
        }

        assert_eq!(statistics_page_text(), "not-served");
        assert_eq!(mining_page_text(), "not-served");
        assert_eq!(graphs_page_text(), "not-served");
        assert_eq!(hashrate_page_text(), "not-served");
        assert_eq!(lightning_page_text(), "not-served");
        assert_eq!(price_chart_page_text(), "not-checked");

        let links = Dashboard::new().links();
        for link in [
            "statistics",
            "mining",
            "graphs",
            "hashrate",
            "price chart",
            "lightning",
        ] {
            assert!(links.contains(&link), "{link}");
            let screen = Screen::from_link(link).expect(link);
            assert_eq!(screen.label(), link);
        }

        let source = production_screens_source();
        assert!(!source.contains("fn route_availability("));
        for (name, path) in [
            ("graphs_page_text", "/api/v1/statistics"),
            ("hashrate_page_text", "/v1/mining"),
            ("price_chart_page_text", "/api/v1/historical-price"),
            ("statistics_page_text", "/api/v1/statistics"),
            ("mining_page_text", "/v1/mining"),
            ("lightning_page_text", "/lightning"),
        ] {
            let body = page_fn_body(source, name);
            assert!(
                body.contains("route_availability("),
                "{name} does not call route_availability"
            );
            assert!(body.contains(path), "{name} does not pass {path}");
            assert!(
                !body.contains("\"not-served\""),
                "{name} hardcodes the sentence"
            );
            assert!(
                !body.contains("\"not-checked\""),
                "{name} hardcodes the sentence"
            );
            let lower = body.to_ascii_lowercase();
            for banned in [
                "canvas",
                "svg",
                "polyline",
                "nsec",
                "private-key",
                "private key",
            ] {
                assert!(!lower.contains(banned), "{name} contains {banned}");
            }
        }
        assert!(source.contains("Screen::Graphs => graphs_page_text()"));
        assert!(source.contains("Screen::Hashrate => hashrate_page_text()"));
        assert!(source.contains("Screen::PriceChart => price_chart_page_text()"));
    }

    #[test]
    fn painted_pools_shows_route_availability_for_mining() {
        let path = "/v1/mining";
        let shared = splora_frontend_shared::route_availability(path);
        assert_eq!(pools_page_text(), shared, "pools paint");
        let (mut host, rec) = host_with(Network::Mainnet, BTreeMap::new());
        host.activate(Screen::Pools).expect("pools");
        assert!(rec.get_paths().is_empty(), "{:?}", rec.get_paths());
        assert!(rec.post_paths().is_empty());
        assert_painted_screen(&host, "pools");
        assert_eq!(host.rendered(), shared);
        assert_view(&host, &[shared]);
        let painted = painted_of(&host);
        assert_no_private_key(&painted);
        assert!(!painted.contains("mempool.space"), "{painted}");
        assert!(!painted.to_ascii_lowercase().contains("chart"), "{painted}");
        let source = production_screens_source();
        let body = page_fn_body(source, "pools_page_text");
        assert!(body.contains("route_availability("));
        assert!(body.contains(path));
        assert!(!body.contains("\"not-served\""));
        assert!(!body.contains("\"not-checked\""));
        assert!(source.contains("Screen::Pools => pools_page_text()"));
    }

    fn hex64(byte: &str) -> String {
        byte.repeat(32)
    }

    /// Row clicks, dashboard search, pages, spends, package broadcast, and liquid assets.
    #[test]
    fn row_click_opens_block_address_search_pages_spends_and_package() {
        let block_hash = "abc123def456";
        let prev_hash = "prevhash1";
        let next_best = "nextbest1";
        let confirming = "confhash1";
        let block = format!(
            r#"{{"id":"{block_hash}","height":800000,"version":1,"timestamp":1700000000,"tx_count":1,"size":200,"weight":800,"merkle_root":"mm","previousblockhash":"{prev_hash}","mediantime":1700000000,"nonce":1,"bits":2,"difficulty":3.0}}"#
        );
        let prev_block = format!(
            r#"{{"id":"{prev_hash}","height":799999,"version":1,"timestamp":1699999999,"tx_count":1,"size":10,"weight":40,"merkle_root":"pm","previousblockhash":"genesis","mediantime":1699999999,"nonce":1,"bits":2,"difficulty":1.0}}"#
        );
        let next_block = format!(
            r#"{{"id":"{next_best}","height":800001,"version":1,"timestamp":1700000001,"tx_count":1,"size":10,"weight":40,"merkle_root":"nm","previousblockhash":"{block_hash}","mediantime":1700000001,"nonce":1,"bits":2,"difficulty":1.0}}"#
        );
        let conf_block = format!(
            r#"{{"id":"{confirming}","height":800000,"version":1,"timestamp":1700000000,"tx_count":1,"size":10,"weight":40,"merkle_root":"cm","previousblockhash":"older","mediantime":1700000000,"nonce":1,"bits":2,"difficulty":1.0}}"#
        );
        let empty_txs = "[]";
        let status =
            format!(r#"{{"in_best_chain":true,"height":800000,"next_best":"{next_best}"}}"#);
        let (mut host, rec) = host_with(
            Network::Mainnet,
            BTreeMap::from([
                (format!("/block/{block_hash}"), block.clone()),
                (format!("/block/{block_hash}/txs"), empty_txs.to_string()),
                (format!("/block/{block_hash}/txids"), empty_txs.to_string()),
                (format!("/block/{block_hash}/status"), status),
                (format!("/block/{prev_hash}"), prev_block),
                (format!("/block/{prev_hash}/txs"), empty_txs.to_string()),
                (format!("/block/{prev_hash}/txids"), empty_txs.to_string()),
                (format!("/block/{next_best}"), next_block),
                (format!("/block/{next_best}/txs"), empty_txs.to_string()),
                (format!("/block/{next_best}/txids"), empty_txs.to_string()),
                (format!("/block/{confirming}"), conf_block),
                (format!("/block/{confirming}/txs"), empty_txs.to_string()),
                (format!("/block/{confirming}/txids"), empty_txs.to_string()),
                ("/block-height/800000".to_string(), block_hash.to_string()),
                ("/block/800000".to_string(), block.clone()),
            ]),
        );
        host.open_row("row block height 800000 hash abc123def456 tx_count 1 time 1700000000")
            .expect("open block row");
        assert_eq!(host.screen, Screen::Block);
        assert!(
            rec.get_paths()
                .iter()
                .any(|path| path == "/block/abc123def456")
        );
        let painted = painted_of(&host);
        assert!(
            painted.contains("The transaction list is empty."),
            "{painted}"
        );
        assert!(painted.contains("The txid list is empty."), "{painted}");
        assert!(painted.contains("next_best nextbest1"), "{painted}");
        let rows = host.painted_rows();
        let previous = rows
            .iter()
            .find(|row| row.starts_with("row previous_hash "))
            .expect("previous hash row")
            .clone();
        host.open_row(&previous).expect("open previous");
        assert_eq!(host.block.query, prev_hash);
        assert!(
            rec.get_paths()
                .iter()
                .any(|path| path == "/block/prevhash1")
        );
        let next_row = host
            .painted_rows()
            .into_iter()
            .find(|row| row.starts_with("row next_best "))
            .unwrap_or_else(|| "row next_best nextbest1".to_string());
        host.open_block("abc123def456").expect("back");
        host.open_row(&next_row).expect("open next best");
        assert_eq!(host.block.query, next_best);
        assert!(
            rec.get_paths()
                .iter()
                .any(|path| path == "/block/nextbest1")
        );
        host.open_row("row confirming confhash1")
            .expect("confirming");
        assert_eq!(host.screen, Screen::Block);
        assert_eq!(host.block.query, confirming);
        host.open_row("row block height 800000")
            .expect("bare height");
        assert!(
            rec.get_paths()
                .iter()
                .any(|path| path == "/block-height/800000"),
            "{:?}",
            rec.get_paths()
        );

        let script = "bc1qw508d6qejxtdg4y5r3zarvary0c5xw7kv8f3t4";
        let stats = format!(
            r#"{{"address":"{script}","chain_stats":{{"tx_count":1,"funded_txo_count":1,"spent_txo_count":0,"funded_txo_sum":10,"spent_txo_sum":0}},"mempool_stats":{{"tx_count":1,"funded_txo_count":1,"spent_txo_count":0,"funded_txo_sum":2,"spent_txo_sum":0}}}}"#
        );
        let addr_tx = r#"{"txid":"addrconfirmed","version":1,"locktime":0,"vin":[],"vout":[],"size":10,"weight":40,"sigops":0,"fee":1,"status":{"confirmed":true,"block_height":3,"block_hash":"confhash1","block_time":9}}"#;
        let mempool_tx = r#"{"txid":"addrmempool","version":1,"locktime":0,"vin":[],"vout":[],"size":10,"weight":40,"sigops":0,"fee":2,"status":{"confirmed":false}}"#;
        let chain_summary =
            r#"[{"txid":"chainsummary","height":4,"value":15,"time":11,"tx_position":2}]"#;
        let (mut host, rec) = host_with(
            Network::Mainnet,
            BTreeMap::from([
                (format!("/address/{script}"), stats),
                (format!("/address/{script}/txs"), format!("[{addr_tx}]")),
                (format!("/address/{script}/utxo"), "[]".to_string()),
                (
                    format!("/address/{script}/txs/mempool"),
                    format!("[{mempool_tx}]"),
                ),
                (
                    format!("/address/{script}/txs/chain/addrconfirmed"),
                    chain_summary.to_string(),
                ),
                (
                    "/tx/chainsummary".to_string(),
                    r#"{"txid":"chainsummary","version":1,"locktime":0,"vin":[],"vout":[],"size":1,"weight":4,"sigops":0,"fee":3,"status":{"confirmed":true}}"#.to_string(),
                ),
                (
                    "/block/confhash1".to_string(),
                    r#"{"id":"confhash1","height":3,"version":1,"timestamp":9,"tx_count":1,"size":1,"weight":4,"merkle_root":"m","previousblockhash":"p","mediantime":9,"nonce":1,"bits":1,"difficulty":1.0}"#.to_string(),
                ),
                ("/block/confhash1/txs".to_string(), "[]".to_string()),
                ("/block/confhash1/txids".to_string(), "[]".to_string()),
            ]),
        );
        host.open_row(&format!("row input address {script}"))
            .expect("input address");
        assert_eq!(host.screen, Screen::Address);
        assert!(rec.get_paths().contains(&format!("/address/{script}")));
        assert!(rec.get_paths().contains(&format!("/address/{script}/txs")));
        assert!(rec.get_paths().contains(&format!("/address/{script}/utxo")));
        assert!(
            rec.get_paths()
                .contains(&format!("/address/{script}/txs/mempool"))
        );
        let painted = painted_of(&host);
        assert!(painted.contains("The utxo list is empty."), "{painted}");
        assert!(painted.contains("addrmempool"), "{painted}");
        host.open_row("row output address bc1qotheroutputaddress000000000000000")
            .expect_err("missing output address still requests");
        host.open_address(script).expect("address again");
        host.open_row("row chain addrconfirmed")
            .expect("chain page");
        assert!(rec.get_paths().iter().any(|path| {
            path == "/address/bc1qw508d6qejxtdg4y5r3zarvary0c5xw7kv8f3t4/txs/chain/addrconfirmed"
        }));
        let summary = host
            .painted_rows()
            .into_iter()
            .find(|row| row.contains("chainsummary"))
            .expect("summary row");
        assert!(summary.contains("height 4"), "{summary}");
        assert!(summary.contains("value 15"), "{summary}");
        assert!(summary.contains("time 11"), "{summary}");
        assert!(summary.contains("tx_position 2"), "{summary}");
        assert!(!summary.contains('?'), "{summary}");
        host.open_row(&summary).expect("summary txid");
        assert_eq!(host.screen, Screen::Tx);
        assert_eq!(host.tx.txid, "chainsummary");
        host.open_address(script).expect("address for keys");
        host.push_str(" ");
        host.push_str("addrconfirmed");
        host.reload().expect("chain from keys");
        assert!(
            rec.get_paths()
                .iter()
                .any(|path| path.ends_with("/txs/chain/addrconfirmed"))
        );

        let page_tx = r#"{"txid":"page25tx","version":1,"locktime":0,"vin":[],"vout":[],"size":1,"weight":4,"sigops":0,"fee":8,"status":{"confirmed":true}}"#;
        let (mut host, rec) = host_with(
            Network::Mainnet,
            BTreeMap::from([
                (format!("/block/{block_hash}"), block),
                (format!("/block/{block_hash}/txs"), empty_txs.to_string()),
                (
                    format!("/block/{block_hash}/txids"),
                    "[\"onlyone\"]".to_string(),
                ),
                (
                    format!("/block/{block_hash}/txs/25"),
                    format!("[{page_tx}]"),
                ),
                ("/blocks/800000".to_string(), "[]".to_string()),
            ]),
        );
        host.open_block(block_hash).expect("block");
        host.open_row("row page 25").expect("page click");
        assert!(
            rec.get_paths()
                .iter()
                .any(|path| path == "/block/abc123def456/txs/25")
        );
        assert!(
            painted_of(&host).contains("page25tx"),
            "{}",
            painted_of(&host)
        );
        host.block.start_index.clear();
        host.push_str(" ");
        host.push_str("25");
        host.reload().expect("page keys");
        assert!(
            rec.get_paths()
                .iter()
                .filter(|path| path.as_str() == "/block/abc123def456/txs/25")
                .count()
                >= 2
        );
        host.activate(Screen::Blocks).expect("blocks");
        host.push_str("800000");
        host.reload().expect("blocks height");
        assert!(rec.get_paths().iter().any(|path| path == "/blocks/800000"));

        let tx_hex = hex64("ab");
        let block_hex = hex64("cd");
        let script_hex = hex64("ef");
        let script_stats = format!(
            r#"{{"scripthash":"{script_hex}","chain_stats":{{"tx_count":0,"funded_txo_count":0,"spent_txo_count":0,"funded_txo_sum":0,"spent_txo_sum":0}},"mempool_stats":{{"tx_count":0,"funded_txo_count":0,"spent_txo_count":0,"funded_txo_sum":0,"spent_txo_sum":0}}}}"#
        );
        let found_tx = format!(
            r#"{{"txid":"{tx_hex}","version":1,"locktime":0,"vin":[],"vout":[],"size":1,"weight":4,"sigops":0,"fee":1,"status":{{"confirmed":false}}}}"#
        );
        let found_block = format!(
            r#"{{"id":"{block_hex}","height":9,"version":1,"timestamp":1,"tx_count":0,"size":1,"weight":4,"merkle_root":"m","previousblockhash":"p","mediantime":1,"nonce":1,"bits":1,"difficulty":1.0}}"#
        );
        let (mut host, rec) = host_with(
            Network::Mainnet,
            BTreeMap::from([
                (format!("/tx/{tx_hex}"), found_tx),
                (format!("/block/{block_hex}"), found_block),
                ("/block-height/42".to_string(), block_hex.clone()),
                (format!("/block/{block_hex}/txs"), "[]".to_string()),
                (format!("/block/{block_hex}/txids"), "[]".to_string()),
                (
                    format!("/tx/{block_hex}"),
                    "Transaction not found".to_string(),
                ),
                (format!("/scripthash/{script_hex}"), script_stats),
                (format!("/scripthash/{script_hex}/txs"), "[]".to_string()),
                (format!("/scripthash/{script_hex}/utxo"), "[]".to_string()),
                (
                    "/address-prefix/bc1qzz".to_string(),
                    "address search disabled".to_string(),
                ),
                (
                    format!("/address/{script}"),
                    format!(
                        r#"{{"address":"{script}","chain_stats":{{"tx_count":0,"funded_txo_count":0,"spent_txo_count":0,"funded_txo_sum":0,"spent_txo_sum":0}},"mempool_stats":{{"tx_count":0,"funded_txo_count":0,"spent_txo_count":0,"funded_txo_sum":0,"spent_txo_sum":0}}}}"#
                    ),
                ),
                (format!("/address/{script}/txs"), "[]".to_string()),
                (format!("/address/{script}/utxo"), "[]".to_string()),
            ]),
        );
        host.activate(Screen::Dashboard).expect("dashboard");
        host.push_str(&tx_hex);
        host.reload().expect("search tx");
        assert_eq!(host.screen, Screen::Tx);
        assert_eq!(host.tx.txid, tx_hex);
        host.activate(Screen::Dashboard).expect("dashboard");
        host.push_str(&block_hex);
        host.reload().expect("search block");
        assert_eq!(host.screen, Screen::Block);
        assert_eq!(host.block.query, block_hex);
        host.activate(Screen::Dashboard).expect("dashboard");
        host.push_str("42");
        host.reload().expect("search height");
        assert_eq!(host.screen, Screen::Block);
        assert!(
            rec.get_paths()
                .iter()
                .any(|path| path == "/block-height/42")
        );
        host.activate(Screen::Dashboard).expect("dashboard");
        host.push_str(script);
        host.reload().expect("search address");
        assert_eq!(host.screen, Screen::Address);
        assert_eq!(host.address.address, script);
        let painted = painted_of(&host);
        assert!(
            painted.contains("The address transaction list is empty."),
            "{painted}"
        );
        host.activate(Screen::Dashboard).expect("dashboard");
        host.push_str(&script_hex);
        host.reload().expect("search scripthash");
        assert_eq!(host.screen, Screen::Address);
        assert!(
            rec.get_paths()
                .iter()
                .any(|path| *path == format!("/scripthash/{script_hex}"))
        );
        assert!(rec.get_paths().iter().all(|path| !path.contains('?')));
        host.activate(Screen::Dashboard).expect("dashboard");
        host.push_str("bc1qzz");
        host.reload().expect("prefix");
        let painted = painted_of(&host);
        assert!(painted.contains("address search disabled"), "{painted}");
        assert_ne!(host.notice, "bad path");
        assert!(!painted.contains("asset name"), "{painted}");

        let spend_tx = "spendtx111";
        let raw_hex = "deadbeef";
        let tx_body = r#"{"txid":"rate-tx","version":2,"locktime":9,"vin":[{"txid":"prev","vout":1,"prevout":{"scriptpubkey":"0014scriptpubkeyhex","scriptpubkey_asm":"asmspecial","scriptpubkey_type":"v0_p2wpkh","scriptpubkey_address":"bc1qinput","value":610677},"scriptsig":"scriptsigsecret","scriptsig_asm":"asmspecial","witness":["abcdwitnesshex"],"is_coinbase":false,"sequence":1}],"vout":[{"scriptpubkey":"0014outputscriptpubkeyhex","scriptpubkey_asm":"asmspecial","scriptpubkey_type":"v0_p2wpkh","scriptpubkey_address":"bc1qoutput","value":344697}],"size":200,"weight":400,"sigops":3,"fee":50,"status":{"confirmed":true,"block_height":800000,"block_hash":"confhash1","block_time":1700001111}}"#;
        let (mut host, _rec) = host_with(
            Network::Mainnet,
            BTreeMap::from([
                ("/tx/rate-tx".to_string(), tx_body.to_string()),
                (
                    "/tx/rate-tx/outspends".to_string(),
                    format!(
                        r#"[{{"spent":true,"txid":"{spend_tx}","vin":0,"status":{{"confirmed":true,"block_height":4,"block_hash":"confhash1","block_time":8}}}}]"#
                    ),
                ),
                (
                    "/tx/rate-tx/merkle-proof".to_string(),
                    r#"{"block_height":800000,"merkle":["abcd"],"pos":1}"#.to_string(),
                ),
                ("/tx/rate-tx/hex".to_string(), raw_hex.to_string()),
                (
                    format!("/tx/{spend_tx}"),
                    format!(
                        r#"{{"txid":"{spend_tx}","version":1,"locktime":0,"vin":[],"vout":[],"size":1,"weight":4,"sigops":0,"fee":1,"status":{{"confirmed":true}}}}"#
                    ),
                ),
                (
                    "/block/confhash1".to_string(),
                    r#"{"id":"confhash1","height":800000,"version":1,"timestamp":1,"tx_count":1,"size":1,"weight":4,"merkle_root":"m","previousblockhash":"p","mediantime":1,"nonce":1,"bits":1,"difficulty":1.0}"#.to_string(),
                ),
                ("/block/confhash1/txs".to_string(), "[]".to_string()),
                ("/block/confhash1/txids".to_string(), "[]".to_string()),
            ]),
        );
        host.open_transaction("rate-tx").expect("tx");
        let painted = painted_of(&host);
        assert!(painted.contains("spend txid spendtx111"), "{painted}");
        assert!(
            painted.contains("merkle block_height 800000 pos 1"),
            "{painted}"
        );
        assert!(painted.contains("tx hex deadbeef"), "{painted}");
        let short = host
            .painted_rows()
            .into_iter()
            .find(|row| row.starts_with("row tx "))
            .expect("short row");
        assert_eq!(short, "row tx txid rate-tx fee 50");
        assert!(!short.contains(raw_hex), "{short}");
        assert!(!short.contains("0014scriptpubkeyhex"), "{short}");
        assert!(!short.contains("abcdwitnesshex"), "{short}");
        host.open_row("row spend txid spendtx111").expect("spend");
        assert_eq!(host.screen, Screen::Tx);
        assert_eq!(host.tx.txid, spend_tx);
        host.open_row("row confirming confhash1")
            .expect("confirming from tx");
        assert_eq!(host.screen, Screen::Block);

        let (mut host, _) = host_with(
            Network::Mainnet,
            BTreeMap::from([
                ("/block/nope".to_string(), "Block not found".to_string()),
                ("/tx/nope".to_string(), "Transaction not found".to_string()),
            ]),
        );
        let err = host.open_block("nope").expect_err("block missing");
        assert!(
            err.to_string().contains("Block not found") || host.notice.contains("Block not found")
        );
        let painted = painted_of(&host);
        assert!(painted.contains("Block not found"), "{painted}");
        assert_ne!(painted.trim(), "bad path");
        assert_ne!(host.notice, "bad path");
        assert_ne!(host.notice, "http:");
        let err = host.open_transaction("nope").expect_err("tx missing");
        let painted = painted_of(&host);
        assert!(painted.contains("Transaction not found"), "{painted} {err}");
        assert!(!host.notice.starts_with("http:"), "{}", host.notice);

        let package = r#"{"package_msg":"success","tx-results":{"aa":{"txid":"pkgtx1","vsize":100,"error":null}},"replaced-transactions":["oldtx"]}"#;
        let (mut host, rec) = host_with(
            Network::Mainnet,
            BTreeMap::from([("/txs/package".to_string(), package.to_string())]),
        );
        host.activate(Screen::Broadcast).expect("broadcast");
        host.broadcast.raw_tx = "abcd,ef01".to_string();
        host.submit().expect("package");
        assert_eq!(rec.post_paths(), vec!["/txs/package".to_string()]);
        let painted = painted_of(&host);
        assert!(painted.contains("broadcast package success"), "{painted}");
        assert!(painted.contains("pkgtx1"), "{painted}");
        assert!(!painted.contains("replace-by-fee"), "{painted}");
        assert!(!painted.contains("replacements"), "{painted}");
        assert!(Screen::from_link("replacements").is_none());

        let asset = hex64("12");
        let asset_body = r#"{"name":"Tether USD","ticker":"USDt","precision":8}"#;
        let asset_tx = format!(
            r#"[{{"txid":"assettx1","version":1,"locktime":0,"vin":[],"vout":[],"size":1,"weight":4,"sigops":0,"fee":1,"status":{{"confirmed":true}}}}]"#
        );
        let (mut host, rec) = host_with(
            Network::Liquid,
            BTreeMap::from([
                (format!("/tx/{asset}"), "Transaction not found".to_string()),
                (format!("/block/{asset}"), "Block not found".to_string()),
                (format!("/asset/{asset}"), asset_body.to_string()),
                (format!("/asset/{asset}/txs"), asset_tx),
            ]),
        );
        host.activate(Screen::Dashboard).expect("liquid dashboard");
        host.push_str(&asset);
        host.reload().expect("asset");
        let painted = painted_of(&host);
        assert!(painted.contains("asset name Tether USD"), "{painted}");
        assert!(painted.contains("ticker USDt"), "{painted}");
        assert!(painted.contains("assettx1"), "{painted}");
        assert!(
            rec.get_paths()
                .iter()
                .any(|path| *path == format!("/asset/{asset}"))
        );
        assert!(
            rec.get_paths()
                .iter()
                .any(|path| *path == format!("/asset/{asset}/txs"))
        );
        let (mut host, rec) = host_with(
            Network::Mainnet,
            BTreeMap::from([
                (format!("/tx/{asset}"), "Transaction not found".to_string()),
                (format!("/block/{asset}"), "Block not found".to_string()),
                (format!("/asset/{asset}"), asset_body.to_string()),
                (
                    format!("/scripthash/{asset}"),
                    "address search disabled".to_string(),
                ),
            ]),
        );
        host.activate(Screen::Dashboard).expect("mainnet dashboard");
        host.push_str(&asset);
        let _ = host.reload();
        let painted = painted_of(&host);
        assert!(
            rec.get_paths().iter().all(|path| !path.contains("/asset/")),
            "{:?}",
            rec.get_paths()
        );
        assert!(!painted.contains("Tether USD"), "{painted}");
        assert!(!painted.contains("USDt"), "{painted}");
        let (mut host, _) = host_with(
            Network::Liquid,
            BTreeMap::from([
                (format!("/tx/{asset}"), "Transaction not found".to_string()),
                (format!("/block/{asset}"), "Block not found".to_string()),
                (
                    format!("/asset/{asset}"),
                    "endpoint does not exist /asset".to_string(),
                ),
            ]),
        );
        host.activate(Screen::Dashboard).expect("liquid");
        host.push_str(&asset);
        let _ = host.reload();
        let painted = painted_of(&host);
        assert!(painted.contains("not-served"), "{painted}");
        assert!(!painted.contains("Tether USD"), "{painted}");
    }
}
