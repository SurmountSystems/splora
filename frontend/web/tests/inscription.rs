//! Inscription on a non-liquid transaction input from `GET /tx/:txid`.
//!
//! The transaction page paints a button whose visible text is Inscription
//! when `vin.prevout.scriptpubkey_type` is `v1_p2tr`, the body is not from
//! Liquid, and the taproot script-path script includes `0063036f7264`.
//! That script is the witness item before the control block. A last item
//! that starts with `50` is an annex, and the script is the item before
//! the control block. A key-path spend has no script. The load is
//! `GET /tx/:txid` with no query and no fragment.
//!
//! A Liquid indexer body includes `is_pegin` on the input, including false.
//! A Bitcoin body omits that key. The row helper the page already calls
//! does not take a separate network argument. `Runestone` stays on an
//! `op_return` output whose scriptpubkey starts with `6a5d`. The script,
//! the witness, and the script hex stay off the page. There is no tooltip.

use splora_web::{transaction_requests, transaction_row_markup};

const TXID: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
const PREV: &str = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
const MARK: &str = "0063036f7264";
const SCRIPT: &str = "aa0063036f7264bb";
const PLAIN: &str = "51acplainscriptsentinel";
const STACK: &str = "cc0063036f7264dd";
const CONTROL: &str = "c0controlblocksentinel";
const CONTROL_MARKED: &str = "c00063036f7264ee";
const ANNEX: &str = "50annexsentinel";
const ANNEX_MARKED: &str = "500063036f7264ff";
const WIT_ITEM: &str = "wititemsentinel";
const SIG: &str = "sighexxsentinel";
const SIG_ASM: &str = "sigasmsentinel";
const PREV_SCRIPT: &str = "5120prevscriptsentinel";
const PREV_ASM: &str = "prevasmsentinel";
const STONE: &str = "6a5dstonexsentinel";
const STONE_ASM: &str = "stoneasmsentinel";
const WORD: &str = "Inscription";
const BUTTON: &str =
    "<button type=\"button\" class=\"btn btn-sm badge badge-ord\">Inscription</button>";
const RUNESTONE: &str =
    "<button type=\"button\" class=\"btn btn-sm badge badge-ord\">Runestone</button>";
const BESIDE: &str = "<span class=\"op-return\">OP_RETURN</span><button type=\"button\" class=\"btn btn-sm badge badge-ord\">Runestone</button>";
const P2TR: &str = "<span class=\"p2tr-tapscript-label\">P2TR tapscript</span>";
const PREVOUT_V1: &str = "<span class=\"previous-output-type\">V1_P2TR</span>";

fn witness(items: &[&str]) -> String {
    let body = items
        .iter()
        .map(|item| format!("\"{item}\""))
        .collect::<Vec<_>>()
        .join(",");
    format!("[{body}]")
}

fn vin(kind: &str, witness_json: &str, liquid: bool) -> String {
    let pegin = if liquid { r#","is_pegin":false"# } else { "" };
    format!(
        r#"{{"txid":"{PREV}","vout":0,"prevout":{{"scriptpubkey":"{PREV_SCRIPT}","scriptpubkey_asm":"{PREV_ASM}","scriptpubkey_type":"{kind}","value":5}},"scriptsig":"{SIG}","scriptsig_asm":"{SIG_ASM}","witness":{witness_json},"is_coinbase":false,"sequence":4294967295{pegin}}}"#
    )
}

fn tx(kind: &str, witness_json: &str, liquid: bool) -> String {
    format!(
        r#"{{"txid":"{TXID}","version":1,"locktime":0,"vin":[{}],"vout":[{{"scriptpubkey":"{STONE}","scriptpubkey_asm":"{STONE_ASM}","scriptpubkey_type":"op_return","value":0}}],"size":10,"weight":40,"sigops":0,"fee":1}}"#,
        vin(kind, witness_json, liquid)
    )
}

fn paint(kind: &str, witness_json: &str, liquid: bool) -> String {
    let requests = transaction_requests(TXID);
    assert_eq!(requests.len(), 1, "{requests:?}");
    assert_eq!(requests[0].method, "GET");
    assert_eq!(requests[0].path, format!("/tx/{TXID}"));
    assert!(!requests[0].path.contains('?'));
    assert!(!requests[0].path.contains('#'));
    let markup = transaction_row_markup(&tx(kind, witness_json, liquid));
    assert!(
        markup.contains("<h2>Inputs & Outputs</h2>"),
        "the transaction page was not rendered: {markup}"
    );
    markup
}

fn assert_exact_inscription(markup: &str) {
    assert!(markup.contains(BUTTON), "Inscription is absent: {markup}");
    assert_eq!(
        markup.matches(BUTTON).count(),
        1,
        "Inscription was not painted once: {markup}"
    );
    assert_eq!(
        markup.matches(WORD).count(),
        1,
        "Inscription was not the visible text once: {markup}"
    );
    let stripped = markup.replacen(BUTTON, "", 1);
    assert!(
        !stripped.contains(WORD),
        "Inscription was painted inside a longer sentence: {stripped}"
    );
}

fn assert_no_inscription(markup: &str) {
    assert!(
        !markup.contains(WORD),
        "the page painted Inscription: {markup}"
    );
    assert!(
        !markup.contains(BUTTON),
        "the page painted the Inscription button: {markup}"
    );
}

fn assert_runestone_stays(markup: &str) {
    assert_eq!(
        markup.matches(RUNESTONE).count(),
        1,
        "Runestone was replaced or dropped: {markup}"
    );
    assert_eq!(
        markup.matches(BESIDE).count(),
        1,
        "Runestone left the OP_RETURN output: {markup}"
    );
    assert_eq!(
        markup.matches(WORD).count(),
        markup.matches(BUTTON).count(),
        "Inscription replaced Runestone or leaked beside the button: {markup}"
    );
}

fn assert_scripts_stay_off(markup: &str) {
    for needle in [
        MARK,
        SCRIPT,
        PLAIN,
        STACK,
        CONTROL,
        CONTROL_MARKED,
        ANNEX,
        ANNEX_MARKED,
        WIT_ITEM,
        SIG,
        SIG_ASM,
        PREV_SCRIPT,
        PREV_ASM,
        STONE,
        STONE_ASM,
    ] {
        assert!(!markup.contains(needle), "{needle} was painted: {markup}");
    }
}

fn assert_no_tooltip(markup: &str) {
    assert!(
        !markup.contains("tooltip"),
        "a tooltip was painted: {markup}"
    );
    assert!(
        !markup.contains("Inscription data"),
        "a tooltip was painted: {markup}"
    );
    assert!(
        !BUTTON.contains("title="),
        "the Inscription button has a tooltip"
    );
}

fn assert_on_the_input(markup: &str) {
    let input_at = markup
        .find(PREVOUT_V1)
        .expect("previous output type left the input");
    let label_at = markup.find(P2TR).expect("P2TR tapscript left the input");
    let button_at = markup.find(BUTTON).expect("Inscription left the input");
    let stone_at = markup.find(RUNESTONE).expect("Runestone left the output");
    assert!(
        input_at < label_at && label_at < button_at && button_at < stone_at,
        "Inscription was not on the transaction input beside P2TR tapscript: {markup}"
    );
}

#[test]
fn v1_p2tr_script_path_paints_inscription_on_the_no_query_tx_page() {
    let marked = witness(&[WIT_ITEM, SCRIPT, CONTROL]);
    let markup = paint("v1_p2tr", &marked, false);
    assert_exact_inscription(&markup);
    assert_on_the_input(&markup);
    assert_eq!(
        markup.matches(P2TR).count(),
        1,
        "P2TR tapscript was dropped or repeated: {markup}"
    );
    assert_runestone_stays(&markup);
    assert_scripts_stay_off(&markup);
    assert_no_tooltip(&markup);
}

#[test]
fn annexed_script_path_paints_inscription_once() {
    let annex = witness(&[WIT_ITEM, SCRIPT, CONTROL, ANNEX]);
    let markup = paint("v1_p2tr", &annex, false);
    assert_exact_inscription(&markup);
    assert_on_the_input(&markup);
    assert_runestone_stays(&markup);
    assert_scripts_stay_off(&markup);
    assert_no_tooltip(&markup);
}

#[test]
fn taproot_without_the_bytes_omits_inscription() {
    let plain = witness(&[WIT_ITEM, PLAIN, CONTROL]);
    let markup = paint("v1_p2tr", &plain, false);
    assert_no_inscription(&markup);
    assert_eq!(
        markup.matches(P2TR).count(),
        1,
        "P2TR tapscript was dropped: {markup}"
    );
    assert!(
        markup.contains(PREVOUT_V1),
        "previous output type left the input: {markup}"
    );
    assert_runestone_stays(&markup);
    assert_scripts_stay_off(&markup);
    assert_no_tooltip(&markup);
}

#[test]
fn marker_outside_the_script_path_omits_inscription() {
    let cases = [
        witness(&[STACK, PLAIN, CONTROL]),
        witness(&[WIT_ITEM, PLAIN, CONTROL_MARKED]),
        witness(&[WIT_ITEM, PLAIN, CONTROL, ANNEX_MARKED]),
        witness(&[SCRIPT]),
    ];
    for witness_json in cases {
        let markup = paint("v1_p2tr", &witness_json, false);
        assert_no_inscription(&markup);
        assert_eq!(
            markup.matches(P2TR).count(),
            1,
            "P2TR tapscript was dropped: {markup}"
        );
        assert_runestone_stays(&markup);
        assert_scripts_stay_off(&markup);
        assert_no_tooltip(&markup);
    }
}

#[test]
fn non_v1_p2tr_omits_inscription() {
    let marked = witness(&[WIT_ITEM, SCRIPT, CONTROL]);
    let markup = paint("v0_p2wpkh", &marked, false);
    assert_no_inscription(&markup);
    assert!(
        !markup.contains(P2TR),
        "a v0_p2wpkh input painted P2TR tapscript: {markup}"
    );
    assert!(
        markup.contains("<span class=\"previous-output-type\">V0_P2WPKH</span>"),
        "previous output type left the input: {markup}"
    );
    assert_runestone_stays(&markup);
    assert_scripts_stay_off(&markup);
    assert_no_tooltip(&markup);
}

#[test]
fn liquid_omits_inscription() {
    let marked = witness(&[WIT_ITEM, SCRIPT, CONTROL]);
    let markup = paint("v1_p2tr", &marked, true);
    assert_no_inscription(&markup);
    assert_eq!(
        markup.matches(P2TR).count(),
        1,
        "P2TR tapscript was dropped on liquid: {markup}"
    );
    assert!(
        markup.contains(PREVOUT_V1),
        "previous output type left the liquid input: {markup}"
    );
    assert_runestone_stays(&markup);
    assert_scripts_stay_off(&markup);
    assert_no_tooltip(&markup);
}
