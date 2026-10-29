//! First mempool block after the dashboard skeleton is gone.
//!
//! Cypress: it "check first mempool block after skeleton loads".
//! Visit `/`, wait until the skeleton is gone, then
//! `#mempool-block-0 > .blockLink` exists.
//! Paint is `painted_shell_text` after `Screen::Dashboard` activate.

use std::collections::BTreeMap;

use splora_api::{Network, api_root};
use splora_frontend_shared::{MempoolBacklog, first_mempool_block_from_backlog};
use splora_native::{
    ClientError, ClientRequest, ExplorerClient, ExplorerGet, ExplorerPost, NpubPopupModel, Screen,
    ScreenHost, painted_shell_text,
};

#[derive(Clone)]
struct Rec {
    network: Network,
    bodies: BTreeMap<String, String>,
}

impl Rec {
    fn new(network: Network, bodies: BTreeMap<String, String>) -> Self {
        Self { network, bodies }
    }
}

fn indexer_path(network: Network, url: &str) -> String {
    let root = api_root(network);
    url.strip_prefix(&root).unwrap_or(url).to_string()
}

impl ExplorerGet for Rec {
    fn get_text(&self, url: &str) -> Result<String, ClientError> {
        let path = indexer_path(self.network, url);
        self.bodies
            .get(&path)
            .cloned()
            .ok_or_else(|| ClientError::Http(format!("missing {path}")))
    }

    fn get_request(&self, request: &ClientRequest) -> Result<String, ClientError> {
        assert_eq!(request.method, "GET");
        self.bodies
            .get(&request.path)
            .cloned()
            .ok_or_else(|| ClientError::Http(format!("missing {}", request.path)))
    }
}

impl ExplorerPost for Rec {
    fn post_text(&self, request: &ClientRequest) -> Result<String, ClientError> {
        Err(ClientError::Http(format!("missing {}", request.path)))
    }
}

fn host_with(network: Network, bodies: BTreeMap<String, String>) -> ScreenHost<Rec> {
    let rec = Rec::new(network, bodies);
    ScreenHost::new(ExplorerClient::new(network, rec))
}

fn painted(host: &ScreenHost<Rec>) -> String {
    let popup = NpubPopupModel::new("npub1restofindexer").expect("npub");
    painted_shell_text(host, &popup)
}

#[test]
fn check_first_mempool_block_after_skeleton_loads() {
    let mut host = host_with(Network::Mainnet, BTreeMap::new());
    let _opened = host.activate(Screen::Dashboard);
    let backlog = MempoolBacklog {
        count: 0,
        vsize: 0,
        total_fee: 0,
        fee_histogram_len: 0,
    };
    let view = first_mempool_block_from_backlog(backlog);
    host.set_mempool_backlog(backlog);
    let text = painted(&host);
    assert!(
        view.block_link_exists && text.contains(&view.link_id),
        "painted dashboard text is missing the shared link id\n{text}"
    );
    let skeleton_gone = text.contains("skeleton_visible=false");
    let has_block = text.contains("mempool-block-0");
    let has_block_link = text.contains("blockLink");
    assert!(
        skeleton_gone && has_block && has_block_link,
        "painted dashboard text is missing mempool-block-0 blockLink after the skeleton is gone\n{text}"
    );
}
