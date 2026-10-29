//! Graphs screen after the testnet4 dashboard skeleton is gone.
//!
//! Cypress: it "loads the graphs screen" in
//! `ref/mempool/frontend/cypress/e2e/testnet4/testnet4.spec.ts` lines 55-61.
//! Visit `/testnet4`, wait until the skeleton is gone, click `#btn-graphs`,
//! and wait 1000 milliseconds.
//! Paint for the open is `painted_shell_text` after `Screen::Dashboard` activate.
//! No native paint path shows that control. This test calls `graphs_screen_lines`.

use std::collections::BTreeMap;

use splora_api::{Network, api_root};
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
fn loads_the_graphs_screen() {
    let mut host = host_with(Network::Testnet4, BTreeMap::new());
    let _opened = host.activate(Screen::Dashboard);
    let opened = painted(&host);
    let root = host.root();
    assert!(
        root.contains("/testnet4"),
        "painted shell did not open /testnet4: {root}\n{opened}"
    );
    assert!(
        opened.contains("skeleton_visible=false"),
        "painted shell still shows the skeleton after /testnet4\n{opened}"
    );
    assert!(
        !opened.contains("skeleton-loader"),
        "skeleton-loader remains in the painted shell after /testnet4\n{opened}"
    );

    let lines: Vec<String> = splora_native::graphs_screen_lines();
    let text = painted(&host);
    for line in ["btn-graphs", "screen: graphs", "wait_ms=1000"] {
        assert!(
            lines.iter().any(|got| got == line),
            "graphs_screen_lines is missing {line}: {lines:?}"
        );
        assert!(
            text.contains(line),
            "activating the graphs control did not show {line} in the painted shell text\n{text}"
        );
    }
}
