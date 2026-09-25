//! GPUI shell. The first view is the npub popup. Continue opens the dashboard.
//! Screen changes call `ScreenHost`, which requests the electrs route and renders its fields.

use gpui::{
    App, Context, FocusHandle, Focusable, IntoElement, KeyDownEvent, MouseButton, Render, Window,
    WindowOptions, div, prelude::*, rgb,
};
use gpui_platform::application;
use splora_api::Network;
use splora_native::{
    ACCENT_RGB, BACKGROUND_RGB, ClientError, NetworkSwitcher, NpubPopupModel, Screen, ScreenHost,
    TEXT_RGB, UreqExplorer, launch_npub, network_label, painted_shell_text, ureq_client,
};

struct Shell {
    focus: FocusHandle,
    popup: NpubPopupModel,
    switcher: NetworkSwitcher,
    host: ScreenHost<UreqExplorer>,
}

impl Shell {
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

impl Focusable for Shell {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus.clone()
    }
}

impl Render for Shell {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let network_line = format!(
            "{}\n{}\n{}",
            self.switcher.summary(),
            self.switcher.prefixes().join(" "),
            self.host.root(),
        );
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

fn main() {
    let npub = launch_npub().expect("npub");
    application().run(move |cx: &mut App| {
        let npub = npub.clone();
        cx.open_window(WindowOptions::default(), move |_window, cx| {
            let npub = npub.clone();
            cx.new(move |cx| {
                let switcher = NetworkSwitcher::new(Network::Mainnet);
                Shell {
                    focus: cx.focus_handle(),
                    popup: NpubPopupModel::new(npub).expect("npub"),
                    switcher,
                    host: ScreenHost::new(ureq_client(Network::Mainnet)),
                }
            })
        })
        .expect("open window");
        cx.activate(true);
    });
}
