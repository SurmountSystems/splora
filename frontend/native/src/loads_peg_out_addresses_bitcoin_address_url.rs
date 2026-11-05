//! Peg-out address Bitcoin address URL.
//!
//! The URL comes from the shared host function.
//! This module does not invent that URL.
//! The window is opened headless and closed before this function returns.

use gpui::{Context, IntoElement, Render, Window, WindowOptions, div, prelude::*};
use gpui_platform::headless;

struct PegOutAddressesBitcoinAddressUrlView;

impl Render for PegOutAddressesBitcoinAddressUrlView {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div().id("peg-out-addresses-bitcoin-address-url-host")
    }
}

/// Paint the peg-out address Bitcoin address URL.
///
/// Opens one headless window with `show: false` and calls `cx.quit()`
/// before returning, including when the window fails to open.
/// The URL stays in the shared host function.
pub fn paint_loads_peg_out_addresses_bitcoin_address_url(address: &str) -> String {
    headless().run(|cx| {
        let options = WindowOptions {
            show: false,
            focus: false,
            ..Default::default()
        };
        let opened = cx.open_window(options, |_window, cx| {
            cx.new(|_| PegOutAddressesBitcoinAddressUrlView)
        });
        cx.quit();
        cx.spawn(async move |cx: &mut gpui::AsyncApp| {
            cx.update(|app| app.quit());
        })
        .detach();
        opened.expect("headless peg out addresses bitcoin address url window");
    });
    splora_frontend_shared::loads_peg_out_addresses_bitcoin_address_url(address)
}
