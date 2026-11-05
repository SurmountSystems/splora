//! Peg-in address Bitcoin transaction URL.
//!
//! The URL comes from the shared host function.
//! This module does not invent that URL.
//! The window is opened headless and closed before this function returns.

use gpui::{Context, IntoElement, Render, Window, WindowOptions, div, prelude::*};
use gpui_platform::headless;

struct PegInAddressesBitcoinTxUrlView;

impl Render for PegInAddressesBitcoinTxUrlView {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div().id("peg-in-addresses-bitcoin-tx-url-host")
    }
}

/// Paint the peg-in address Bitcoin transaction URL.
///
/// Opens one headless window with `show: false` and calls `cx.quit()`
/// before returning, including when the window fails to open.
/// The URL stays in the shared host function.
pub fn paint_loads_peg_in_addresses_bitcoin_tx_url(txid: &str, vout: u32) -> String {
    headless().run(|cx| {
        let options = WindowOptions {
            show: false,
            focus: false,
            ..Default::default()
        };
        let opened = cx.open_window(options, |_window, cx| {
            cx.new(|_| PegInAddressesBitcoinTxUrlView)
        });
        cx.quit();
        cx.spawn(async move |cx: &mut gpui::AsyncApp| {
            cx.update(|app| app.quit());
        })
        .detach();
        opened.expect("headless peg in addresses bitcoin tx url window");
    });
    splora_frontend_shared::loads_peg_in_addresses_bitcoin_tx_url(txid, vout)
}
