//! Desktop explorer library. Tests do not open a window or touch the network.

pub mod api;
pub mod keystore;
pub mod popup;
pub mod screens;
pub mod signer;
pub mod theme;

pub use api::{
    ClientError, ClientRequest, ExplorerClient, ExplorerGet, ExplorerPost, UreqExplorer,
    ureq_client,
};
pub use keystore::{KeyStoreError, SploraNamedKeyStore, launch_npub};
pub use popup::{NpubPopupModel, PopupError, PopupWidgetKind};
pub use screens::{
    AddressScreen, BlockScreen, BlocksScreen, BroadcastScreen, Dashboard, DocPage, MempoolScreen,
    MultiAddressScreen, NetworkSwitcher, Screen, ScreenHost, StaticPage, StaticScreen,
    TestTransactionsScreen, TxScreen, known_doc_lines, network_label, open_doc_route,
    painted_rows, painted_shell_text, resolve_doc_route,
};
pub use signer::{SignedAuth, SignerError, sign_http};
pub use theme::{ACCENT, ACCENT_RGB, BACKGROUND, BACKGROUND_RGB, TEXT, TEXT_RGB};
