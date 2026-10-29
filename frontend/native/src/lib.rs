//! Desktop explorer library. Tests do not open a window or touch the network.

pub mod api;
pub mod api_screen;
pub mod caps_the_list_when_changing_the_limit_to_10;
pub mod docs_screen;
pub mod empty_block_screen;
pub mod graphs_screen;
pub mod keystore;
pub mod network_skeleton;
pub mod no_pagination_block_screen;
pub mod pause_icon_updates;
pub mod pools_screen;
pub mod popup;
mod regular_tx_unblinding;
pub mod screens;
mod show_unblinded_tx;
pub mod signer;
pub mod supports_pagination_on_the_block_screen;
pub mod theme;
pub mod usd_decimal;
pub mod usd_symbol;

pub use api::{
    ClientError, ClientRequest, ExplorerClient, ExplorerGet, ExplorerPost, UreqExplorer,
    ureq_client,
};
pub use api_screen::paint_loads_the_api_screen;
pub use caps_the_list_when_changing_the_limit_to_10::paint_caps_the_list_when_changing_the_limit_to_10;
pub use docs_screen::{DocsScreen, docs_screen_text};
pub use empty_block_screen::paint_shows_empty_blocks_properly;
pub use graphs_screen::{graphs_screen_lines, paint_loads_the_graphs_screen};
pub use keystore::{KeyStoreError, SploraNamedKeyStore, launch_npub};
pub use network_skeleton::paint_loads_skeleton_when_changes_between_networks;
pub use no_pagination_block_screen::paint_shows_blocks_with_no_pagination;
pub use pause_icon_updates::paint_pauses_updates_when_clicking_the_pause_icon;
pub use pools_screen::pools_screen_lines;
pub use popup::{NpubPopupModel, PopupError, PopupWidgetKind};
pub use regular_tx_unblinding::regular_tx_unblinding_text;
pub use screens::{
    AddressScreen, BlockScreen, BlocksScreen, BroadcastScreen, Dashboard, DocPage, MempoolScreen,
    MultiAddressScreen, NetworkSwitcher, Screen, ScreenHost, StaticPage, StaticScreen,
    TestTransactionsScreen, TxScreen, known_doc_lines, network_label, notice_from_error,
    open_doc_route, painted_rows, painted_shell_text, resolve_doc_route, transaction_id_in_row,
};
pub use show_unblinded_tx::show_unblinded_tx_text;
pub use signer::{IndexerAuthCall, SignedAuth, SignerError, call_indexer_with_signer, sign_http};
pub use supports_pagination_on_the_block_screen::paint_supports_pagination_on_the_block_screen;
pub use theme::{ACCENT, ACCENT_RGB, BACKGROUND, BACKGROUND_RGB, TEXT, TEXT_RGB};
pub use usd_decimal::paint_displays_usd_values_with_proper_decimal_format;
pub use usd_symbol::paint_displays_usd_values_with_correct_currency_symbol;
