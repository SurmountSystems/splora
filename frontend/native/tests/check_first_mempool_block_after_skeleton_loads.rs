//! Row 2. Cypress it() title: check first mempool block after skeleton loads.
//!
//! `ref/mempool/frontend/cypress/e2e/testnet4/testnet4.spec.ts` visits `/`,
//! waits until the skeleton is gone, then requires
//! `#mempool-block-0 > .blockLink`. The parent id is the ngFor render index
//! on a received `GET /mempool` backlog, including a non-zero summary.
//! Paint is `painted_shell_text` after `Screen::Dashboard` activate.
//! This test does not call `set_mempool_backlog`.

use std::collections::BTreeMap;

use splora_api::{Network, api_root};
use splora_frontend_shared::{MempoolBacklog, first_mempool_block_from_backlog, mempool_path};
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

/// `#parent > .blockLink` is the next painted line after the parent id.
fn direct_child_block_link(text: &str, link_id: &str) -> bool {
    let lines: Vec<&str> = text.lines().collect();
    lines
        .windows(2)
        .any(|pair| pair[0] == link_id && pair[1] == "blockLink")
}

#[test]
fn check_first_mempool_block_after_skeleton_loads() {
    let count: u32 = 4;
    let vsize: u32 = 500;
    let total_fee: u64 = 90;
    let histogram: [(u32, u32); 1] = [(2, 100)];
    let histogram_json = histogram
        .iter()
        .map(|(rate, size)| format!("[{rate},{size}]"))
        .collect::<Vec<_>>()
        .join(",");
    let body = format!(
        r#"{{"count":{count},"vsize":{vsize},"total_fee":{total_fee},"fee_histogram":[{histogram_json}]}}"#
    );
    let view = first_mempool_block_from_backlog(MempoolBacklog {
        count,
        vsize,
        total_fee,
        fee_histogram_len: histogram.len(),
    });
    assert!(
        view.block_link_exists && !view.link_id.is_empty(),
        "a received GET /mempool backlog projects a first block link, got {view:?}"
    );

    let mut host = host_with(
        Network::Mainnet,
        BTreeMap::from([(mempool_path().to_string(), body)]),
    );
    host.activate(Screen::Dashboard)
        .expect("dashboard activate with GET /mempool");
    let text = painted(&host);
    let summary = format!("mempool count {count} vsize {vsize} total_fee {total_fee}");
    let skeleton_gone = text.lines().any(|line| line == "skeleton_visible=false");
    let child = direct_child_block_link(&text, &view.link_id);
    assert!(
        text.contains(&summary) && skeleton_gone && child,
        "after cy.visit('/') and cy.waitForSkeletonGone(), #{} > .blockLink is missing from the painted dashboard\n{text}",
        view.link_id
    );
}
