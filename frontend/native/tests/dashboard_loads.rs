//! Dashboard load from `dashboard_page_initial`.
//!
//! The opening state hides the skeleton and issues six indexer GETs, in order.
//! `Screen::Dashboard` paint is `painted_shell_text` after `activate`.
//! That paint does not apply `dashboard_page_initial`.

use std::collections::BTreeMap;

use splora_api::{Network, api_root};
use splora_frontend_shared::dashboard_page_initial;
use splora_native::{
    ClientError, ClientRequest, ExplorerClient, ExplorerGet, ExplorerPost, NpubPopupModel, Screen,
    ScreenHost, painted_shell_text,
};

const REQUESTS: &[&str] = &[
    "GET /blocks",
    "GET /blocks/tip/hash",
    "GET /blocks/tip/height",
    "GET /fee-estimates",
    "GET /mempool",
    "GET /mempool/recent",
];

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

fn reflected(text: &str) -> Vec<&str> {
    text.lines()
        .map(str::trim)
        .filter(|line| *line == "skeleton_visible=false" || REQUESTS.contains(line))
        .collect()
}

fn expected_paint() -> Vec<&'static str> {
    let mut lines = Vec::with_capacity(REQUESTS.len() + 1);
    lines.push("skeleton_visible=false");
    lines.extend_from_slice(REQUESTS);
    lines
}

#[test]
fn loads_the_dashboard() {
    let page = dashboard_page_initial();
    let mut host = host_with(Network::Mainnet, BTreeMap::new());
    let _opened = host.activate(Screen::Dashboard);
    let text = painted(&host);
    let paint = reflected(&text);
    let expected = expected_paint();
    assert_eq!(
        (
            page.skeleton_visible,
            page.requests.as_slice(),
            paint.as_slice()
        ),
        (false, REQUESTS, expected.as_slice()),
        "painted dashboard text does not reflect dashboard_page_initial\n{text}"
    );
}
