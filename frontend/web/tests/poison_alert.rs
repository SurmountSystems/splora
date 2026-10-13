//! Poison alert text from `splora_frontend_shared::poison_alerts`.
//!
//! The split lives in the shared crate. This page only paints the spans.

use splora_web::poison_alert_markup;

const WARNING: &str = "Warning! This transaction involves deceptively similar addresses. It may be an address poisoning attack.";

fn output(kind: &str, address: &str) -> String {
    format!(
        r#"{{"scriptpubkey":"00","scriptpubkey_asm":"OP_0","scriptpubkey_type":"{kind}","scriptpubkey_address":"{address}","value":1}}"#
    )
}

fn input(prevout: &str) -> String {
    format!(
        r#"{{"txid":"prev","vout":0,"prevout":{prevout},"scriptsig":"","scriptsig_asm":"","is_coinbase":false,"sequence":4294967295}}"#
    )
}

fn transaction(txid: &str, vin: &str, vout: &str) -> String {
    format!(
        r#"{{"txid":"{txid}","version":2,"locktime":0,"vin":[{vin}],"vout":[{vout}],"size":180,"weight":720,"sigops":1,"fee":4500}}"#
    )
}

fn span_texts(markup: &str, class: &str) -> Vec<String> {
    let needle = format!("<span class=\"{class}\">");
    let mut rest = markup;
    let mut out = Vec::new();
    while let Some(start) = rest.find(&needle) {
        let after = &rest[start + needle.len()..];
        let end = after.find("</span>").expect("span is not closed");
        out.push(after[..end].to_string());
        rest = &after[end..];
    }
    out
}

fn transaction_page() -> &'static str {
    let src = include_str!("../src/views.rs");
    let marker = "fn TransactionPage(";
    let start = src
        .find(marker)
        .expect("TransactionPage is missing from views.rs");
    let after_name = &src[start..];
    let brace_rel = after_name.find('{').expect("TransactionPage has no body");
    let body_start = start + brace_rel + 1;
    let mut depth = 1i32;
    for (offset, ch) in src[body_start..].char_indices() {
        match ch {
            '{' => depth += 1,
            '}' => {
                depth -= 1;
                if depth == 0 {
                    return &src[body_start..body_start + offset];
                }
            }
            _ => {}
        }
    }
    panic!("TransactionPage body is not closed");
}

fn donate_tx() -> String {
    let outputs = [
        output("p2pkh", "1DonatePLeaseSenderAddressXVXCmAY"),
        output("p2pkh", "1FeexV6bAHb8ybZjqQMjJrcCrHGW9sb6uF"),
        output("p2pkh", "1DonatePLease5btcToSenderXXWBoKhB"),
        output(
            "v0_p2wsh",
            "bc1qjasf9z3h7w3jspkhtgatgpyvvzgpa2wwd2lr0eh5tx44reyn2k7sfc27a4",
        ),
        output("v0_p2wpkh", "bc1q8yj0herd4r4yxszw3nkfvt53433thk0f5qst4g"),
        output("p2pkh", "1Ay8vMC7R1UbyCCZRVULMV7iQpHSAbguJP"),
        output(
            "v0_p2wsh",
            "bc1qd4ysezhmypwty5dnw7c8nqy5h5nxg0xqsvaefd0qn5kq32vwnwqqgv4rzr",
        ),
        output(
            "v1_p2tr",
            "bc1pahzjr2vnfqnqwl0h8jps2aa4zqe29wqxn2nagvd5a2z982k7ttdq8ry370",
        ),
    ];
    transaction(
        "152a5dea805f95d6f83e50a9fd082630f542a52a076ebabdb295723eaf53fa30",
        &input(&output(
            "v1_p2tr",
            "bc1pahzjr2vnfqnqwl0h8jps2aa4zqe29wqxn2nagvd5a2z982k7ttdq8ry370",
        )),
        &outputs.join(","),
    )
}

#[test]
fn donate_outputs_show_prefix_1donateplease_and_two_alerts() {
    let markup = poison_alert_markup(&donate_tx(), "mainnet");
    assert!(markup.contains("class=\"alert alert-mempool\""), "{markup}");
    assert!(markup.contains(WARNING), "{markup}");
    assert_eq!(span_texts(&markup, "poison-alert").len(), 2, "{markup}");
    assert_eq!(
        span_texts(&markup, "prefix"),
        vec!["1DonatePLease".to_string(), "1DonatePLease".to_string()]
    );
    assert_eq!(
        span_texts(&markup, "infix"),
        vec![
            "SenderAddressXVXCmAY".to_string(),
            "5btcToSenderXXWBoKhB".to_string(),
        ]
    );
    assert!(
        !span_texts(&markup, "prefix")
            .iter()
            .any(|text| text.contains("1Feex")),
        "{markup}"
    );
    assert!(!markup.contains("Fiat price last updated"), "{markup}");
}

#[test]
fn bech32_input_and_output_show_postfix_6p6e3r() {
    let incoming = "bc1qge86gqjnk8aqs3nvv7ejrvcd4zq6qur36p6e3r";
    let outgoing = "bc1qge8xyxjex6zzzx5g8hh65vsel4e548p26p6e3r";
    let body = transaction(
        "44544516084ea916ff1eb69c675c693e252addbbaf77102ffff86e3979ac6132",
        &input(&output("v0_p2wpkh", incoming)),
        &output("v0_p2wpkh", outgoing),
    );
    let markup = poison_alert_markup(&body, "mainnet");
    assert_eq!(span_texts(&markup, "poison-alert").len(), 2, "{markup}");
    assert_eq!(
        span_texts(&markup, "prefix"),
        vec!["bc1qge8".to_string(), "bc1qge8".to_string()]
    );
    assert_eq!(
        span_texts(&markup, "infix"),
        vec![
            "6gqjnk8aqs3nvv7ejrvcd4zq6qur3".to_string(),
            "xyxjex6zzzx5g8hh65vsel4e548p2".to_string(),
        ]
    );
    assert_eq!(
        span_texts(&markup, "postfix"),
        vec!["6p6e3r".to_string(), "6p6e3r".to_string()]
    );
}

#[test]
fn dissimilar_addresses_do_not_invent_an_alert() {
    let body = transaction(
        "aa",
        &input(&output("p2pkh", "1FeexV6bAHb8ybZjqQMjJrcCrHGW9sb6uF")),
        &output("p2pkh", "1Ay8vMC7R1UbyCCZRVULMV7iQpHSAbguJP"),
    );
    let markup = poison_alert_markup(&body, "mainnet");
    assert!(markup.is_empty(), "{markup}");
}

#[test]
fn transaction_page_paints_the_shared_poison_alert() {
    let page = transaction_page();
    assert!(
        page.contains("poison_alert_markup("),
        "TransactionPage does not paint the shared poison alert"
    );
    assert!(
        page.contains("\"mainnet\""),
        "TransactionPage does not name mainnet for poison_alerts"
    );
    assert!(!page.contains("Fiat price last updated"), "{page}");
}
