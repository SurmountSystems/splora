//! GPUI shell. The first view is the npub popup. Continue opens the dashboard.
//! Screen changes call `ScreenHost`, which requests the electrs route and renders its fields.

use gpui::{
    App, Context, FocusHandle, Focusable, IntoElement, KeyDownEvent, MouseButton, Render, Window,
    WindowOptions, div, prelude::*, rgb,
};
use gpui_platform::application;
use splora_api::Network;
use splora_native::{
    ACCENT_RGB, BACKGROUND_RGB, ClientError, ExplorerClient, ExplorerGet, ExplorerPost,
    NetworkSwitcher, NpubPopupModel, Screen, ScreenHost, TEXT_RGB, launch_npub, network_label,
    notice_from_error, painted_rows, painted_shell_text, transaction_id_in_row, ureq_client,
};

struct Shell<G> {
    focus: FocusHandle,
    popup: NpubPopupModel,
    switcher: NetworkSwitcher,
    host: ScreenHost<G>,
}

impl<G: ExplorerGet + ExplorerPost + 'static> Shell<G> {
    fn note(&mut self, result: Result<(), ClientError>) {
        self.host.notice = match result {
            Ok(()) => String::new(),
            Err(err) => notice_from_error(&err),
        };
    }

    fn body_text(&self) -> String {
        painted_shell_text(&self.host, &self.popup)
    }

    fn screen_button(&self, cx: &mut Context<Self>, link: &'static str) -> impl IntoElement {
        let screen = Screen::from_link(link).expect("screen link");
        let text_color = if self.host.screen == screen {
            rgb(ACCENT_RGB)
        } else {
            rgb(TEXT_RGB)
        };
        div()
            .id(screen.element_id())
            .px_3()
            .py_2()
            .border_1()
            .border_color(rgb(ACCENT_RGB))
            .text_color(text_color)
            .child(link)
            .on_click(cx.listener(move |this, _event, _window, cx| {
                if let Some(next) = Screen::from_link(link) {
                    let result = this.host.activate(next);
                    this.note(result);
                }
                cx.notify();
            }))
    }

    fn screen_choices(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let mut row = div().flex().flex_row().flex_wrap().gap_3();
        for link in self.host.dashboard.links() {
            row = row.child(self.screen_button(cx, link));
        }
        row
    }

    fn network_button(&self, cx: &mut Context<Self>, network: Network) -> impl IntoElement {
        let label = network_label(network);
        let text_color = if self.switcher.selected() == network {
            rgb(ACCENT_RGB)
        } else {
            rgb(TEXT_RGB)
        };
        div()
            .id(label)
            .px_3()
            .py_2()
            .border_1()
            .border_color(rgb(ACCENT_RGB))
            .text_color(text_color)
            .child(label)
            .on_click(cx.listener(move |this, _event, _window, cx| {
                this.switcher.select(network);
                this.host.set_network(this.switcher.selected());
                let result = this.host.reload();
                this.note(result);
                cx.notify();
            }))
    }

    fn network_choices(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let mut row = div().flex().flex_row().gap_3();
        for network in NetworkSwitcher::networks().iter().copied() {
            let button = self.network_button(cx, network);
            row = row.child(button);
        }
        row
    }

    fn command_button(
        &self,
        cx: &mut Context<Self>,
        id: &'static str,
        label: &'static str,
        submit: bool,
    ) -> impl IntoElement {
        div()
            .id(id)
            .px_3()
            .py_2()
            .border_1()
            .border_color(rgb(ACCENT_RGB))
            .text_color(rgb(ACCENT_RGB))
            .child(label)
            .on_click(cx.listener(move |this, _event, _window, cx| {
                let result = if submit {
                    this.host.submit()
                } else {
                    this.host.reload()
                };
                this.note(result);
                cx.notify();
            }))
    }

    fn pause_button(&self, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .id("btn-pause")
            .px_3()
            .py_2()
            .border_1()
            .border_color(rgb(ACCENT_RGB))
            .text_color(rgb(ACCENT_RGB))
            .child("Pause")
            .on_click(cx.listener(|this, _event, _window, cx| {
                let result = this.host.open_row("row pause");
                this.note(result);
                cx.notify();
            }))
    }

    fn paste_button(&self, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .id("paste")
            .px_3()
            .py_2()
            .border_1()
            .border_color(rgb(ACCENT_RGB))
            .text_color(rgb(ACCENT_RGB))
            .child("Paste")
            .on_click(cx.listener(|this, _event, _window, cx| {
                if let Some(text) = cx.read_from_clipboard().and_then(|item| item.text()) {
                    this.host.set_query(&text);
                }
                cx.notify();
            }))
    }
}

impl<G: 'static> Focusable for Shell<G> {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus.clone()
    }
}

impl<G: ExplorerGet + ExplorerPost + 'static> Render for Shell<G> {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let network_line = format!(
            "{}\n{}\n{}",
            self.switcher.summary(),
            self.switcher.prefixes().join(" "),
            self.host.root(),
        );
        let mut data_rows = div().flex().flex_col().gap_1();
        let mut tx_row_index = 0usize;
        for row in painted_rows(&self.host) {
            let row_id = tx_row_index;
            tx_row_index += 1;
            if let Some(txid) = transaction_id_in_row(&row) {
                let txid = txid.to_string();
                data_rows = data_rows.child(
                    div()
                        .flex()
                        .flex_row()
                        .id(("tx-row", row_id))
                        .child(row)
                        .on_click(cx.listener(move |this, _event, _window, cx| {
                            let result = this.host.open_transaction(&txid);
                            this.note(result);
                            cx.notify();
                        })),
                );
            } else {
                let row_text = row.clone();
                data_rows = data_rows.child(
                    div()
                        .flex()
                        .flex_row()
                        .id(("tx-row", row_id))
                        .child(row)
                        .on_click(cx.listener(move |this, _event, _window, cx| {
                            let result = this.host.open_row(&row_text);
                            this.note(result);
                            cx.notify();
                        })),
                );
            }
        }
        let mut column = div()
            .id("shell")
            .track_focus(&self.focus)
            .flex()
            .flex_col()
            .gap_3()
            .size_full()
            .bg(rgb(BACKGROUND_RGB))
            .text_color(rgb(TEXT_RGB))
            .p_4()
            .overflow_y_scroll()
            .on_mouse_down(
                MouseButton::Left,
                cx.listener(|this, _event, window, cx| {
                    window.focus(&this.focus, cx);
                }),
            )
            .on_key_down(cx.listener(|this, event: &KeyDownEvent, _window, cx| {
                let modifiers = &event.keystroke.modifiers;
                if modifiers.control || modifiers.alt || modifiers.platform || modifiers.function {
                    return;
                }
                let key = event.keystroke.key.as_str();
                if key == "backspace" {
                    this.host.backspace();
                } else if key == "enter" {
                    let result = match this.host.screen {
                        Screen::Broadcast | Screen::TestTransactions | Screen::Calculator => {
                            this.host.submit()
                        }
                        Screen::Popup => Ok(()),
                        _ => this.host.reload(),
                    };
                    this.note(result);
                } else if let Some(text) = event.keystroke.key_char.clone() {
                    this.host.push_str(&text);
                }
                cx.notify();
            }))
            .child(self.body_text())
            .child(data_rows)
            .child(network_line);
        if self.host.screen != Screen::Popup {
            column = column.child(self.screen_choices(cx));
            column = column.child(self.network_choices(cx));
            let mut actions = div()
                .flex()
                .flex_row()
                .gap_3()
                .child(self.command_button(cx, "load", "Load", false))
                .child(self.paste_button(cx));
            if matches!(
                self.host.screen,
                Screen::Broadcast | Screen::TestTransactions | Screen::Calculator
            ) {
                actions = actions.child(self.command_button(cx, "submit", "Submit", true));
            }
            if self.host.screen == Screen::Mempool {
                actions = actions.child(self.pause_button(cx));
            }
            column = column.child(actions);
        }
        column.child(
            div()
                .id("continue")
                .px_3()
                .py_2()
                .border_1()
                .border_color(rgb(ACCENT_RGB))
                .text_color(rgb(ACCENT_RGB))
                .child("Continue")
                .on_click(cx.listener(|this, _event, _window, cx| {
                    let result = this.host.activate(this.host.screen.after_continue());
                    this.note(result);
                    cx.notify();
                })),
        )
    }
}

fn open_shell<G>(
    cx: &mut App,
    npub: String,
    client: ExplorerClient<G>,
) -> anyhow::Result<gpui::WindowHandle<Shell<G>>>
where
    G: ExplorerGet + ExplorerPost + 'static,
{
    cx.open_window(WindowOptions::default(), move |_window, cx| {
        cx.new(move |cx| {
            let switcher = NetworkSwitcher::new(Network::Mainnet);
            Shell {
                focus: cx.focus_handle(),
                popup: NpubPopupModel::new(npub).expect("npub"),
                switcher,
                host: ScreenHost::new(client),
            }
        })
    })
}

fn main() {
    let npub = launch_npub().expect("npub");
    application().run(move |cx: &mut App| {
        open_shell(cx, npub, ureq_client(Network::Mainnet)).expect("open window");
        cx.activate(true);
    });
}

#[cfg(test)]
mod opened_window {
    use std::cell::RefCell;
    use std::collections::BTreeMap;
    use std::path::Path;
    use std::rc::Rc;

    use splora_native::{
        ClientError, ClientRequest, ExplorerClient, ExplorerGet, ExplorerPost, Screen,
    };

    use super::{Shell, open_shell};

    struct FixtureExplorer {
        by_path: Rc<RefCell<BTreeMap<String, String>>>,
    }

    impl FixtureExplorer {
        fn new() -> Self {
            let mut by_path = BTreeMap::new();
            let files = [
                ("/blocks", "blocks.json"),
                ("/mempool/recent", "mempool_recent.json"),
                ("/block/block-id-1", "block.json"),
                ("/block/block-id-1/txs", "block_txs.json"),
                ("/tx/tx-1", "tx.json"),
                ("/address/xyz", "address.json"),
                ("/address/xyz/txs", "address_txs.json"),
                ("/address/xyz/utxo", "address_utxos.json"),
                ("/block-height/100", "block_height.txt"),
                ("/tx", "broadcast.txt"),
                ("/txs/test", "test_tx.json"),
            ];
            for (path, name) in files {
                by_path.insert(path.to_string(), read_fixture(name));
            }
            by_path.insert(
                "/block/block-id-1/txids".to_string(),
                "[\"block-tx-1\"]".to_string(),
            );
            by_path.insert("/blocks/tip/height".to_string(), "800000\n".to_string());
            by_path.insert("/blocks/tip/hash".to_string(), "tip-hash-1\n".to_string());
            by_path.insert(
                "/fee-estimates".to_string(),
                r#"{"6":5,"1":12.5}"#.to_string(),
            );
            by_path.insert(
                "/mempool".to_string(),
                r#"{"count":2,"vsize":300,"total_fee":1500,"fee_histogram":[[10.5,200],[1,100]]}"#
                    .to_string(),
            );
            Self {
                by_path: Rc::new(RefCell::new(by_path)),
            }
        }

        fn insert(&self, path: &str, body: impl Into<String>) {
            self.by_path
                .borrow_mut()
                .insert(path.to_string(), body.into());
        }

        fn clone_handle(&self) -> Self {
            Self {
                by_path: Rc::clone(&self.by_path),
            }
        }

        fn body(&self, path: &str) -> Result<String, ClientError> {
            self.by_path
                .borrow()
                .get(path)
                .cloned()
                .ok_or_else(|| ClientError::Http(format!("missing {path}")))
        }
    }

    impl ExplorerGet for FixtureExplorer {
        fn get_text(&self, url: &str) -> Result<String, ClientError> {
            self.body(indexer_path(url))
        }

        fn get_request(&self, request: &ClientRequest) -> Result<String, ClientError> {
            if request.method != "GET" {
                return Err(ClientError::BadPath);
            }
            self.body(&request.path)
        }
    }

    impl ExplorerPost for FixtureExplorer {
        fn post_text(&self, request: &ClientRequest) -> Result<String, ClientError> {
            self.body(&request.path)
        }
    }

    fn indexer_path(url: &str) -> &str {
        const API: &str = "/api";
        if let Some(index) = url.rfind(API) {
            let rest = &url[index + API.len()..];
            if rest.starts_with('/') {
                return rest;
            }
        }
        url
    }

    fn read_fixture(name: &str) -> String {
        let path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../shared/tests/fixtures")
            .join(name);
        std::fs::read_to_string(&path).unwrap_or_else(|err| panic!("{}: {err}", path.display()))
    }

    fn assert_window(text: &str, fields: &[&str]) {
        for field in fields {
            assert!(text.contains(field), "{text}");
        }
    }

    fn load_screen(
        shell: &mut Shell<FixtureExplorer>,
        screen: Screen,
        query: Option<&str>,
        submit: bool,
    ) -> String {
        let activated = shell.host.activate(screen);
        if query.is_none() && !submit {
            let text = shell.body_text();
            activated.unwrap_or_else(|err| panic!("{err}\n{text}"));
            return text;
        }
        if let Some(query) = query {
            shell.host.set_query(query);
        }
        let result = if submit {
            shell.host.submit()
        } else {
            shell.host.reload()
        };
        let text = shell.body_text();
        let non_hex_height = screen == Screen::Block && query == Some("100");
        if !non_hex_height {
            result.unwrap_or_else(|err| panic!("{err}\n{text}"));
        }
        text
    }

    #[test]
    fn open_window_shows_shared_fixture_fields() {
        // Test-only. Shipping main() still calls application() and maps a real window.
        gpui_platform::headless().run(|cx| {
            let keys = nostr::key::Keys::generate();
            let npub = nostr::nips::nip19::ToBech32::to_bech32(&keys.public_key())
                .expect("public key encodes");
            let handle = open_shell(cx, npub, ExplorerClient::new(splora_api::Network::Mainnet, FixtureExplorer::new()))
                .expect("open window");
            let compositor = cx.compositor_name();
            assert_eq!(compositor, "headless", "shell opened on {compositor}");
            assert!(handle.read(cx).is_ok());
            handle
                .update(cx, |shell, _window, _cx| {
                    let dashboard = load_screen(shell, Screen::Dashboard, None, false);
                    assert_window(
                        &dashboard,
                        &[
                            "block hash block-id-1",
                            "height 100",
                            "tx_count 12",
                            "timestamp 1600000000",
                            "size 1.5 kB",
                            "weight 4 kWU",
                            "previous_hash prev-block-1",
                            "median_time 1599990000",
                            "recent txid recent-tx-1",
                            "fee 800",
                            "vsize 141",
                            "value 99000",
                            "tip height: 800000 hash: tip-hash-1",
                            "fee target 1 rate 12.5",
                            "fee target 6 rate 5",
                            "mempool count 2 vsize 300 total_fee 1500",
                            "fee_histogram 10.5 200",
                            "fee_histogram 1 100",
                        ],
                    );

                    let block = load_screen(shell, Screen::Block, Some("block-id-1"), false);
                    assert_window(
                        &block,
                        &[
                            "block hash block-id-1",
                            "height 100",
                            "tx_count 12",
                            "timestamp 1600000000",
                            "size 1.5 kB",
                            "weight 4 kWU",
                            "previous_hash prev-block-1",
                            "median_time 1599990000",
                            "tx txid block-tx-1 fee 3000 confirmed true",
                            "block_hash block-id-1",
                            "block_time 1600000000",
                            "size 222",
                            "weight 888",
                            "txid block-tx-1",
                        ],
                    );

                    let tx = load_screen(shell, Screen::Tx, Some("tx-1"), false);
                    assert_window(
                        &tx,
                        &[
                            "tx txid tx-1 fee 4500 confirmed true block_height 100 size 180 weight 720",
                            "block_hash block-id-1",
                            "block_time 1600000000",
                        ],
                    );

                    let address = load_screen(shell, Screen::Address, Some("xyz"), false);
                    assert_window(
                        &address,
                        &[
                            "address xyz chain_tx_count 9 funded_sum 500000 mempool_tx_count 3",
                            "mempool_funded_sum 2500",
                            "address tx txid addr-tx-1 confirmed false",
                            "fee 111",
                            "size 90",
                            "weight 360",
                            "utxo txid utxo-tx-1 vout 2 value 42000 confirmed true",
                            "block_height 100",
                            "block_hash block-id-1",
                            "block_time 1600000000",
                        ],
                    );

                    let broadcast = load_screen(shell, Screen::Broadcast, Some("deadbeef"), true);
                    assert_window(&broadcast, &["broadcast txid broadcast-txid-1"]);

                    let tested = load_screen(shell, Screen::TestTransactions, Some("deadbeef"), true);
                    assert_window(
                        &tested,
                        &[
                            "test txid test-tx-1 allowed false reject-reason min relay fee not met",
                            "fee 0.00001",
                            "effective_feerate 1",
                        ],
                    );

                    let height = load_screen(shell, Screen::Block, Some("100"), false);
                    assert!(height.contains("height-block-hash-1"), "{height}");
                })
                .expect("update");
            // calloop's run() clears a stop requested before the loop starts.
            // Quit again from a foreground task so the loop sees it and returns.
            cx.quit();
            cx.spawn(async move |cx: &mut gpui::AsyncApp| {
                cx.update(|app| app.quit());
            })
            .detach();
        });
    }

    fn observe(line: &str) {
        println!("assert {line}");
    }

    fn prefix_rows(text: &str) -> Vec<String> {
        text.lines()
            .filter_map(|line| line.trim().strip_prefix("row prefix "))
            .map(str::to_string)
            .collect()
    }

    fn recent_body(ids: &[String]) -> String {
        let rows = ids
            .iter()
            .map(|id| format!(r#"{{"txid":"{id}","fee":1,"vsize":1,"value":1}}"#))
            .collect::<Vec<_>>()
            .join(",");
        format!("[{rows}]")
    }

    fn require_plain(path: &str) {
        assert!(
            splora_frontend_shared::indexer_path_is_plain(path),
            "{path}"
        );
        observe(&format!("plain path {path}"));
    }

    /// Drives the shipped GPUI shell from the npub popup through the shared cases.
    /// Headless only: `gpui_platform::headless`, never `application()`.
    #[test]
    fn shared_suite_drives_the_shipped_gpui_screen() {
        gpui_platform::headless().run(|cx| {
            let compositor = cx.compositor_name();
            assert_eq!(compositor, "headless", "shell opened on {compositor}");
            observe("compositor headless");
            let keys = nostr::key::Keys::generate();
            let npub = nostr::nips::nip19::ToBech32::to_bech32(&keys.public_key())
                .expect("public key encodes");
            let explorer = FixtureExplorer::new();
            let mempool = explorer.body("/mempool").expect("mempool");
            let blocks = explorer.body("/blocks").expect("blocks");
            let block = explorer.body("/block/block-id-1").expect("block");
            let block_txs = explorer.body("/block/block-id-1/txs").expect("block txs");
            let recent = explorer.body("/mempool/recent").expect("recent");
            let tx_body = explorer.body("/tx/tx-1").expect("tx");
            let address_stats = read_fixture("address.json");
            let address_utxo = read_fixture("address_utxos.json");

            let prefix_body = r#"["1wizSAYSbuyXbt9d8JV8ytm5acqq2TorC","1wizOTHER000000000000000000000000","3other00000000000000000000000000"]"#;
            let bech32 = r#"["bc1q0003match000000000000000000000000"]"#;
            let bech32m = r#"["bc1pqyqsmatchone000000000000000000"]"#;
            let address_txs = r#"[{"txid":"multi-in","version":2,"locktime":0,"vin":[{"txid":"prev","vout":0,"prevout":{"scriptpubkey":"0014aa","scriptpubkey_asm":"OP_0","scriptpubkey_type":"v0_p2wpkh","scriptpubkey_address":"bc1qexample","value":1},"scriptsig":"","scriptsig_asm":"","is_coinbase":false,"sequence":1},{"txid":"prev","vout":1,"prevout":{"scriptpubkey":"0014aa","scriptpubkey_asm":"OP_0","scriptpubkey_type":"v0_p2wpkh","scriptpubkey_address":"bc1qexample","value":1},"scriptsig":"","scriptsig_asm":"","is_coinbase":false,"sequence":1}],"vout":[{"scriptpubkey":"0014bb","scriptpubkey_asm":"OP_0","scriptpubkey_type":"v0_p2wpkh","scriptpubkey_address":"bc1qother","value":1}],"size":180,"weight":720,"sigops":1,"fee":100,"status":{"confirmed":false}},{"txid":"multi-out","version":2,"locktime":0,"vin":[{"txid":"prev","vout":0,"prevout":{"scriptpubkey":"0014bb","scriptpubkey_asm":"OP_0","scriptpubkey_type":"v0_p2wpkh","scriptpubkey_address":"bc1qother","value":1},"scriptsig":"","scriptsig_asm":"","is_coinbase":false,"sequence":1}],"vout":[{"scriptpubkey":"0014aa","scriptpubkey_asm":"OP_0","scriptpubkey_type":"v0_p2wpkh","scriptpubkey_address":"bc1qexample","value":1},{"scriptpubkey":"0014aa","scriptpubkey_asm":"OP_0","scriptpubkey_type":"v0_p2wpkh","scriptpubkey_address":"bc1qexample","value":1}],"size":180,"weight":720,"sigops":1,"fee":100,"status":{"confirmed":true,"block_height":5,"block_hash":"h5","block_time":5}},{"txid":"both-sides","version":2,"locktime":0,"vin":[{"txid":"prev","vout":0,"prevout":{"scriptpubkey":"0014aa","scriptpubkey_asm":"OP_0","scriptpubkey_type":"v0_p2wpkh","scriptpubkey_address":"bc1qexample","value":1},"scriptsig":"","scriptsig_asm":"","is_coinbase":false,"sequence":1}],"vout":[{"scriptpubkey":"0014aa","scriptpubkey_asm":"OP_0","scriptpubkey_type":"v0_p2wpkh","scriptpubkey_address":"bc1qexample","value":1}],"size":180,"weight":720,"sigops":1,"fee":100,"status":{"confirmed":true,"block_height":4,"block_hash":"h4","block_time":4}},{"txid":"single-in","version":2,"locktime":0,"vin":[{"txid":"prev","vout":0,"prevout":{"scriptpubkey":"0014aa","scriptpubkey_asm":"OP_0","scriptpubkey_type":"v0_p2wpkh","scriptpubkey_address":"bc1qexample","value":1},"scriptsig":"","scriptsig_asm":"","is_coinbase":false,"sequence":1}],"vout":[{"scriptpubkey":"0014bb","scriptpubkey_asm":"OP_0","scriptpubkey_type":"v0_p2wpkh","scriptpubkey_address":"bc1qother","value":1}],"size":180,"weight":720,"sigops":1,"fee":100,"status":{"confirmed":true,"block_height":3,"block_hash":"h3","block_time":3}},{"txid":"single-out","version":2,"locktime":0,"vin":[{"txid":"prev","vout":0,"prevout":{"scriptpubkey":"0014bb","scriptpubkey_asm":"OP_0","scriptpubkey_type":"v0_p2wpkh","scriptpubkey_address":"bc1qother","value":1},"scriptsig":"","scriptsig_asm":"","is_coinbase":false,"sequence":1}],"vout":[{"scriptpubkey":"0014aa","scriptpubkey_asm":"OP_0","scriptpubkey_type":"v0_p2wpkh","scriptpubkey_address":"bc1qexample","value":1}],"size":180,"weight":720,"sigops":1,"fee":100,"status":{"confirmed":true,"block_height":2,"block_hash":"h2","block_time":2}}]"#;
            let genesis_hash =
                "0000000000000000000000000000000000000000000000000000000000000000";
            let genesis = r#"{"id":"0000000000000000000000000000000000000000000000000000000000000000","height":0,"version":1,"timestamp":0,"tx_count":1,"size":285,"weight":1140,"merkle_root":"m","mediantime":0}"#;
            let genesis_txs = r#"[{"txid":"coinbase-genesis","version":1,"locktime":0,"vin":[{"txid":"0000000000000000000000000000000000000000000000000000000000000000","vout":4294967295,"prevout":null,"scriptsig":"","scriptsig_asm":"","is_coinbase":true,"sequence":0}],"vout":[{"scriptpubkey":"00","scriptpubkey_asm":"","scriptpubkey_type":"op_return","value":0}],"size":100,"weight":400,"sigops":0,"fee":0,"status":{"confirmed":true,"block_height":0,"block_hash":"0000000000000000000000000000000000000000000000000000000000000000","block_time":0}}]"#;
            let tip_block = r#"{"id":"tip-block-id","height":800000,"version":1,"timestamp":1700000000,"tx_count":2,"size":200,"weight":800,"merkle_root":"m","previousblockhash":"prev-tip-block","mediantime":1699990000}"#;
            let next_page = block_txs.replace("block-tx-1", "block-tx-page-25");
            let multi_in = serde_json::from_str::<Vec<serde_json::Value>>(address_txs)
                .expect("address txs")
                .into_iter()
                .next()
                .expect("multi-in")
                .to_string();

            for (path, body) in [
                ("/address-prefix/1wiz", prefix_body),
                ("/address-prefix/1wizS", prefix_body),
                ("/address-prefix/1Wiz", prefix_body),
                ("/address-prefix/bc1q0003", bech32),
                ("/address-prefix/bc1pqyqs", bech32m),
                ("/block/block-id-1/status", r#"{"in_best_chain":true,"height":100,"next_best":"next-block-hash"}"#),
                ("/block-height/0", genesis_hash),
                ("/block/0000000000000000000000000000000000000000000000000000000000000000", genesis),
                ("/block/0000000000000000000000000000000000000000000000000000000000000000/txs", genesis_txs),
                ("/block/0000000000000000000000000000000000000000000000000000000000000000/txids", r#"["coinbase-genesis"]"#),
                ("/block/tip-block-id", tip_block),
                ("/block/tip-block-id/txs", "[]"),
                ("/block/tip-block-id/txids", "[]"),
                ("/block/block-id-1/txs/25", next_page.as_str()),
            ] {
                require_plain(path);
                explorer.insert(path, body);
            }
            let legacy = "1wizSAYSbuyXbt9d8JV8ytm5acqq2TorC";
            let marked = "bc1qexample";
            for address in [legacy, marked] {
                let stats = format!("/address/{address}");
                let txs = format!("/address/{address}/txs");
                let utxo = format!("/address/{address}/utxo");
                require_plain(&stats);
                require_plain(&txs);
                require_plain(&utxo);
                explorer.insert(&stats, address_stats.clone());
                explorer.insert(&txs, address_txs);
                explorer.insert(&utxo, address_utxo.clone());
            }
            require_plain("/tx/multi-in");
            explorer.insert("/tx/multi-in", multi_in.clone());

            let handle = open_shell(
                cx,
                npub,
                ExplorerClient::new(splora_api::Network::Mainnet, explorer.clone_handle()),
            )
            .expect("open window");
            handle
                .update(cx, |shell, _window, _cx| {
                    assert_eq!(shell.host.screen, Screen::Popup);
                    let popup = shell.body_text();
                    assert!(popup.starts_with("Splora\n"), "{popup}");
                    observe("start screen popup");

                    let names: Vec<&str> = splora_frontend_shared::shared_cases()
                        .iter()
                        .map(|case| case.name)
                        .collect();
                    assert_eq!(names.len(), 29);
                    observe(&format!("shared_cases {}", names.len()));
                    assert!(!splora_frontend_shared::liquid_rows_painted());
                    observe("liquid rows not painted");

                    shell
                        .host
                        .activate(Screen::Dashboard)
                        .expect("continue to dashboard");
                    let home_text = shell.body_text();
                    let home = splora_frontend_shared::home_view(&mempool, &blocks)
                        .expect("home_view");
                    assert!(home.projected_navigable);
                    assert!(
                        home_text.contains(&home.projected_link_id),
                        "{home_text}"
                    );
                    assert!(
                        home_text.contains(&home.projected_open_path),
                        "missing projected path {}\n{home_text}",
                        home.projected_open_path
                    );
                    observe(&format!(
                        "projected link {} path {}",
                        home.projected_link_id, home.projected_open_path
                    ));
                    assert!(!home.confirmed.is_empty());
                    for link in &home.confirmed {
                        assert!(home_text.contains(&link.id), "{home_text}");
                        assert!(
                            home_text.contains(&link.height.to_string()),
                            "{home_text}"
                        );
                        require_plain(&link.open_path);
                        observe(&format!(
                            "confirmed block {} height {} path {}",
                            link.id, link.height, link.open_path
                        ));
                    }
                    let first = &home.confirmed[0];
                    shell
                        .host
                        .open_row(&format!("row block height {} hash {}", first.height, first.id))
                        .expect("open confirmed block");
                    let opened = shell.body_text();
                    let opened_block = splora_frontend_shared::block_open(&block).expect("block_open");
                    assert!(opened.contains(&opened_block.hash), "{opened}");
                    assert!(opened.contains(&opened_block.heading), "{opened}");
                    observe(&format!(
                        "opened block {} heading {}",
                        opened_block.hash, opened_block.heading
                    ));

                    shell
                        .host
                        .activate(Screen::Dashboard)
                        .expect("dashboard for search");
                    let short_lookup = splora_frontend_shared::prefix_lookup("1wiz").expect("lookup");
                    require_plain(&short_lookup.path);
                    shell.host.set_query("1wiz");
                    shell.host.reload().expect("search 1wiz");
                    let short_text = shell.body_text();
                    let short_hits =
                        splora_frontend_shared::prefix_hits("1wiz", prefix_body).expect("hits");
                    assert_eq!(prefix_rows(&short_text), short_hits, "{short_text}");
                    let resolved = splora_frontend_shared::address_view_path(&short_hits[0]);
                    require_plain(&resolved);
                    observe(&format!(
                        "legacy prefix {} hits {} resolved {}",
                        short_lookup.path,
                        short_hits.len(),
                        resolved
                    ));
                    shell
                        .host
                        .open_address(&short_hits[0])
                        .expect("open resolved address");
                    let resolved_text = shell.body_text();
                    assert!(
                        resolved_text.contains(&format!("row script {}", short_hits[0])),
                        "{resolved_text}"
                    );
                    observe(&format!("resolved address {}", short_hits[0]));

                    shell
                        .host
                        .activate(Screen::Dashboard)
                        .expect("dashboard for narrow");
                    shell.host.set_query("1wizS");
                    shell.host.reload().expect("search 1wizS");
                    let long_text = shell.body_text();
                    let long_hits =
                        splora_frontend_shared::prefix_hits("1wizS", prefix_body).expect("narrow");
                    assert!(long_hits.len() < short_hits.len());
                    assert_eq!(prefix_rows(&long_text), long_hits, "{long_text}");
                    observe(&format!(
                        "search narrows {} to {}",
                        short_hits.len(),
                        long_hits.len()
                    ));

                    for (typed, body) in [("BC1Q0003", bech32), ("bC1q0003", bech32)] {
                        let lookup = splora_frontend_shared::prefix_lookup(typed).expect("bech32");
                        require_plain(&lookup.path);
                        assert!(lookup.folded);
                        shell.host.activate(Screen::Dashboard).expect("dashboard");
                        shell.host.set_query(typed);
                        shell.host.reload().expect("bech32 search");
                        let hits = splora_frontend_shared::prefix_hits(typed, body).expect("hits");
                        assert_eq!(prefix_rows(&shell.body_text()), hits);
                        observe(&format!("bech32 folded {} path {}", typed, lookup.path));
                    }
                    let upper_q = splora_frontend_shared::prefix_lookup("BC1Q0003").expect("q");
                    let mixed_q = splora_frontend_shared::prefix_lookup("bC1q0003").expect("q");
                    assert_eq!(upper_q.path, mixed_q.path);
                    observe("bech32 case insensitive same path");

                    for (typed, body) in [("BC1PQYQS", bech32m), ("bc1PqYqS", bech32m)] {
                        let lookup = splora_frontend_shared::prefix_lookup(typed).expect("bech32m");
                        require_plain(&lookup.path);
                        shell.host.activate(Screen::Dashboard).expect("dashboard");
                        shell.host.set_query(typed);
                        shell.host.reload().expect("bech32m search");
                        let hits = splora_frontend_shared::prefix_hits(typed, body).expect("hits");
                        assert_eq!(prefix_rows(&shell.body_text()), hits);
                        observe(&format!("bech32m folded {} path {}", typed, lookup.path));
                    }
                    let upper_p = splora_frontend_shared::prefix_lookup("BC1PQYQS").expect("p");
                    let mixed_p = splora_frontend_shared::prefix_lookup("bc1PqYqS").expect("p");
                    assert_eq!(upper_p.path, mixed_p.path);
                    observe("bech32m case insensitive same path");

                    let list = splora_frontend_shared::address_tx_list(marked, address_txs)
                        .expect("address_tx_list");
                    shell.host.open_address(marked).expect("address view");
                    let address_text = shell.body_text();
                    assert!(
                        address_text.contains(&format!("row script {marked}")),
                        "{address_text}"
                    );
                    let mut cursor = 0usize;
                    for txid in &list.txids {
                        let found = address_text[cursor..]
                            .find(txid.as_str())
                            .expect("txid order");
                        cursor += found + txid.len();
                    }
                    let inputs = address_text
                        .lines()
                        .filter(|line| line.trim() == format!("highlight input {marked}"))
                        .count();
                    let outputs = address_text
                        .lines()
                        .filter(|line| line.trim() == format!("highlight output {marked}"))
                        .count();
                    let expect_in: usize = list.marks.iter().map(|mark| mark.inputs).sum();
                    let expect_out: usize = list.marks.iter().map(|mark| mark.outputs).sum();
                    assert_eq!(inputs, expect_in, "{address_text}");
                    assert_eq!(outputs, expect_out, "{address_text}");
                    observe(&format!(
                        "address txs newest first {} highlight inputs {inputs} outputs {outputs}",
                        list.txids.len()
                    ));
                    observe("highlight several inputs");
                    observe("highlight several outputs");
                    observe("highlight both sides");

                    let confirmed = splora_frontend_shared::transaction_view(&tx_body)
                        .expect("transaction_view");
                    shell.host.open_transaction(&confirmed.txid).expect("tx");
                    let confirmed_text = shell.body_text();
                    assert!(confirmed_text.contains("confirmed true"), "{confirmed_text}");
                    assert_eq!(
                        confirmed_text
                            .lines()
                            .filter(|line| line.trim().starts_with("input outpoint "))
                            .count(),
                        confirmed.input_count
                    );
                    assert_eq!(
                        confirmed_text
                            .lines()
                            .filter(|line| line.trim().starts_with("output address "))
                            .count(),
                        confirmed.output_count
                    );
                    observe(&format!(
                        "transaction {} confirmed inputs {} outputs {}",
                        confirmed.txid, confirmed.input_count, confirmed.output_count
                    ));
                    let unconfirmed = splora_frontend_shared::transaction_view(&multi_in)
                        .expect("unconfirmed view");
                    assert!(!unconfirmed.confirmed);
                    shell
                        .host
                        .open_transaction(&unconfirmed.txid)
                        .expect("unconfirmed tx");
                    let unconfirmed_text = shell.body_text();
                    assert!(
                        unconfirmed_text.contains("confirmed false"),
                        "{unconfirmed_text}"
                    );
                    assert!(unconfirmed_text.contains("Unconfirmed"), "{unconfirmed_text}");
                    observe(&format!(
                        "transaction {} unconfirmed inputs {} outputs {}",
                        unconfirmed.txid, unconfirmed.input_count, unconfirmed.output_count
                    ));

                    shell.host.open_block("block-id-1").expect("block hash");
                    let by_hash = shell.body_text();
                    let by_hash_view = splora_frontend_shared::block_open(&block).expect("open");
                    assert!(by_hash.contains(&by_hash_view.hash), "{by_hash}");
                    assert!(by_hash.contains(&by_hash_view.heading), "{by_hash}");
                    let height_body = by_hash_view.hash.clone();
                    explorer.insert("/block-height/100", height_body.clone());
                    let from_height = splora_frontend_shared::block_open_from_height(
                        by_hash_view.height,
                        &height_body,
                        &block,
                    )
                    .expect("height");
                    shell
                        .host
                        .open_block(&by_hash_view.height.to_string())
                        .expect("block height");
                    let height_text = shell.body_text();
                    assert!(height_text.contains(&from_height.hash), "{height_text}");
                    observe(&format!(
                        "block by hash {} and height {}",
                        by_hash_view.hash, by_hash_view.height
                    ));

                    let genesis_view = splora_frontend_shared::block_open(genesis).expect("genesis");
                    assert_eq!(genesis_view.tx_count, 1);
                    shell.host.open_block("0").expect("genesis");
                    let genesis_text = shell.body_text();
                    assert!(genesis_text.contains("Genesis"), "{genesis_text}");
                    assert!(genesis_text.contains(&genesis_view.heading), "{genesis_text}");
                    assert!(genesis_text.contains("coinbase-genesis"), "{genesis_text}");
                    assert!(
                        !genesis_text.contains("Previous Block"),
                        "{genesis_text}"
                    );
                    assert!(
                        !genesis_text.contains("row previous_hash"),
                        "{genesis_text}"
                    );
                    let edges = splora_frontend_shared::adjacent_blocks(0, 800000);
                    assert!(!edges.previous_shown);
                    observe("genesis no previous block");
                    observe(&format!("coinbase only heading {}", genesis_view.heading));

                    shell.host.open_block("tip-block-id").expect("tip");
                    let tip_text = shell.body_text();
                    let tip_edges = splora_frontend_shared::adjacent_blocks(800000, 800000);
                    assert!(!tip_edges.next_shown);
                    assert!(tip_edges.previous_shown);
                    assert!(tip_text.contains("prev-tip-block"), "{tip_text}");
                    assert!(!tip_text.contains("row next_best"), "{tip_text}");
                    assert!(!tip_text.contains("Next Block"), "{tip_text}");
                    observe("tip no next block");

                    shell.host.open_block("block-id-1").expect("adjacent");
                    let adjacent_text = shell.body_text();
                    let adjacent = splora_frontend_shared::adjacent_blocks(100, 800000);
                    assert!(adjacent.previous_shown && adjacent.next_shown);
                    assert!(
                        adjacent_text.contains("row previous_hash prev-block-1"),
                        "{adjacent_text}"
                    );
                    assert!(
                        adjacent_text.contains("row next_best next-block-hash"),
                        "{adjacent_text}"
                    );
                    observe("block navigation previous and next");

                    let page_path = splora_frontend_shared::block_tx_page_path("block-id-1", 25);
                    let next_path = splora_frontend_shared::next_block_tx_path("block-id-1");
                    assert_eq!(page_path, next_path);
                    require_plain(&page_path);
                    assert_eq!(splora_frontend_shared::BLOCK_TX_PAGE_LEN, 25);
                    let differ = splora_frontend_shared::block_pages_differ(&block_txs, &next_page)
                        .expect("pages");
                    assert!(differ);
                    let first_page =
                        splora_frontend_shared::block_tx_page(&block_txs).expect("page");
                    let later_page =
                        splora_frontend_shared::block_tx_page(&next_page).expect("next page");
                    assert_ne!(first_page.txids, later_page.txids);
                    shell
                        .host
                        .open_row("row page 25")
                        .expect("next block page");
                    let page_text = shell.body_text();
                    for id in &later_page.txids {
                        assert!(page_text.contains(id), "{page_text}");
                    }
                    observe(&format!(
                        "next block page path {page_path} tx {}",
                        later_page.txids[0]
                    ));

                    shell.host.open_block("block-id-1").expect("details start");
                    let collapsed = shell.body_text();
                    assert!(
                        !collapsed.contains("block details\nid "),
                        "{collapsed}"
                    );
                    assert!(!splora_frontend_shared::block_details_open(true));
                    shell
                        .host
                        .open_row("row block details")
                        .expect("expand");
                    let expanded = shell.body_text();
                    assert!(splora_frontend_shared::block_details_open(false));
                    assert!(expanded.contains("block details\nid "), "{expanded}");
                    observe("block details expanded");
                    shell
                        .host
                        .open_row("row block details")
                        .expect("collapse");
                    let closed = shell.body_text();
                    assert!(!splora_frontend_shared::block_details_open(true));
                    assert!(!closed.contains("block details\nid "), "{closed}");
                    observe("block details collapsed");

                    let visible = splora_frontend_shared::recent_unconfirmed(&recent)
                        .expect("recent");
                    shell.host.activate(Screen::Mempool).expect("mempool");
                    let recent_text = shell.body_text();
                    for id in &visible {
                        assert!(recent_text.contains(id), "{recent_text}");
                    }
                    observe(&format!("recent unconfirmed {}", visible.join(" ")));
                    let arrived = "just-arrived".to_string();
                    let with_new =
                        splora_frontend_shared::recent_with_arrival(&visible, &arrived);
                    assert_eq!(with_new[0], arrived);
                    explorer.insert("/mempool/recent", recent_body(&with_new));
                    shell.host.reload().expect("arrival reload");
                    let arrived_text = shell.body_text();
                    let mut seen = 0usize;
                    for id in &with_new {
                        let found = arrived_text[seen..]
                            .find(id.as_str())
                            .expect("arrival order");
                        seen += found + id.len();
                    }
                    observe(&format!("recent arrival {arrived}"));

                    let paused =
                        splora_frontend_shared::recent_while_paused(&visible, &with_new);
                    assert_eq!(paused, visible);
                    assert!(!paused.iter().any(|id| id == &arrived));
                    // Pause captures the rows already on screen. The later body is fetched again.
                    explorer.insert("/mempool/recent", recent_body(&visible));
                    shell.host.reload().expect("rows at pause");
                    shell.host.open_row("row pause").expect("pause");
                    explorer.insert("/mempool/recent", recent_body(&with_new));
                    shell.host.reload().expect("paused reload");
                    let paused_text = shell.body_text();
                    assert!(
                        !paused_text.contains(&arrived),
                        "later arrival stayed visible\n{paused_text}"
                    );
                    for id in &paused {
                        assert!(paused_text.contains(id), "{paused_text}");
                    }
                    observe("recent paused");

                    shell.host.open_row("row pause").expect("resume");
                    let mut many: Vec<String> = (0..12).map(|n| format!("cap-tx-{n}")).collect();
                    let capped = splora_frontend_shared::recent_capped(&many, 10);
                    assert_eq!(capped.len(), 10);
                    explorer.insert("/mempool/recent", recent_body(&many));
                    shell.host.reload().expect("cap reload");
                    let cap_text = shell.body_text();
                    for id in &capped {
                        assert!(cap_text.contains(id), "{cap_text}");
                    }
                    assert!(!cap_text.contains(&many[10]), "{cap_text}");
                    assert!(!cap_text.contains(&many[11]), "{cap_text}");
                    observe("recent cap 10");
                    let _ = &mut many;

                    let status = splora_frontend_shared::status_from_mempool(&mempool)
                        .expect("status");
                    shell.host.activate(Screen::Mempool).expect("status screen");
                    let status_text = shell.body_text();
                    assert!(status_text.contains("Unconfirmed:"), "{status_text}");
                    assert!(
                        status_text.contains(&format!(
                            "mempool count {} vsize {}",
                            status.unconfirmed_count, status.vsize
                        )),
                        "{status_text}"
                    );
                    assert!(status_text.contains("Mempool size:"), "{status_text}");
                    assert!(
                        status_text.contains(&status.vsize.to_string()),
                        "{status_text}"
                    );
                    observe(&format!(
                        "unconfirmed count {} mempool vsize {}",
                        status.unconfirmed_count, status.vsize
                    ));
                    observe("deferred byte size incoming rate nTx medianFee");
                })
                .expect("update");
            cx.quit();
            cx.spawn(async move |cx: &mut gpui::AsyncApp| {
                cx.update(|app| app.quit());
            })
            .detach();
        });
    }
}
