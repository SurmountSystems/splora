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
    painted_rows, painted_shell_text, ureq_client,
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
            Err(err) => err.to_string(),
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
        for row in painted_rows(&self.host) {
            data_rows = data_rows.child(div().flex().flex_row().child(row));
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
                        Screen::Broadcast | Screen::TestTransactions => this.host.submit(),
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
                Screen::Broadcast | Screen::TestTransactions
            ) {
                actions = actions.child(self.command_button(cx, "submit", "Submit", true));
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
    use std::collections::BTreeMap;
    use std::path::Path;

    use splora_native::{
        ClientError, ClientRequest, ExplorerClient, ExplorerGet, ExplorerPost, Screen,
    };

    use super::{Shell, open_shell};

    struct FixtureExplorer {
        by_path: BTreeMap<String, String>,
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
            Self { by_path }
        }

        fn body(&self, path: &str) -> Result<String, ClientError> {
            self.by_path
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
        gpui_platform::application().run(|cx| {
            let keys = nostr::key::Keys::generate();
            let npub = nostr::nips::nip19::ToBech32::to_bech32(&keys.public_key())
                .expect("public key encodes");
            let handle = open_shell(cx, npub, ExplorerClient::new(splora_api::Network::Mainnet, FixtureExplorer::new()))
                .expect("open window");
            let compositor = gpui::guess_compositor();
            assert!(
                compositor == "X11" || compositor == "Wayland",
                "shell opened on {compositor}"
            );
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
                            "size 1500",
                            "weight 4000",
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
                            "size 1500",
                            "weight 4000",
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
}
