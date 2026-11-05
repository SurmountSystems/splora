//! Desktop transaction replace-by-fee paint.
//!
//! Cypress `shows RBF transactions properly (desktop)` compares the css
//! width of `.container-xl > :nth-child(3)` with `.alert`, and checks that
//! `.btn-warning` and `.alert` do not overlap. The alert is shown when a
//! replacement txid is present and the transaction is not confirmed.

use serde::Deserialize;
use serde_json::Value;

/// Laid-out rectangle from `getBoundingClientRect`.
#[derive(Debug, Clone, Deserialize)]
pub struct DesktopRbfRect {
    pub left: f64,
    pub right: f64,
    pub top: f64,
    pub bottom: f64,
}

/// Widths and rectangles read from the shipped desktop replace-by-fee paint.
#[derive(Debug, Clone)]
pub struct DesktopRbfMeasurement {
    pub alert_css_width: String,
    pub third_child_css_width: String,
    pub confirmations_and_alert_overlap: bool,
    pub alert_rect: DesktopRbfRect,
    pub button_rect: DesktopRbfRect,
    pub viewport: String,
}

/// HTML the transaction page returns for an unconfirmed replacement.
///
/// `replacement_body` is the websocket object Cypress sends for
/// `rbfTransaction` (`txReplaced` or `rbfTransaction`). A confirmed
/// transaction, or a body with no txid, returns an empty string.
pub fn desktop_rbf_transaction_markup(tx_body: &str, replacement_body: &str) -> String {
    if confirmed(tx_body) {
        return String::new();
    }
    let Some(txid) = replacement_txid(replacement_body) else {
        return String::new();
    };
    let txid = escape_html(&txid);
    let details = crate::rows::transaction_row_markup(tx_body);
    format!(
        r#"<style>
.container-xl {{
  padding-right: 15px;
  padding-left: 15px;
  margin-bottom: 40px;
  box-sizing: border-box;
}}
@media (min-width: 1400px) {{
  .container-xl {{ max-width: 1140px; }}
}}
.title-block {{
  display: flex;
  flex-direction: row;
  flex-wrap: wrap;
  justify-content: space-between;
  align-items: baseline;
  padding-top: 20px;
  padding-bottom: 10px;
  box-sizing: border-box;
}}
.title-block h1 {{
  margin: 0;
  margin-right: 15px;
  line-height: 1;
}}
.alert {{
  margin-bottom: 1rem;
}}
.alert-mempool {{
  color: #fff;
  width: 100%;
  display: inline-flex;
  flex-direction: row;
  box-sizing: border-box;
}}
</style>
<div class="container-xl">
  <div class="title-block">
    <div class="alert alert-mempool" role="alert">
      <span>This transaction has been replaced by:</span>
      <a href="/tx/{txid}">{txid}</a>
    </div>
    <h1>Transaction</h1>
    <div class="container-buttons">
      <button type="button" class="btn btn-sm btn-warning">Replaced</button>
    </div>
  </div>
  <div class="clearfix"></div>
  <div class="transaction-details">{details}</div>
</div>
"#
    )
}

/// Measure the desktop replace-by-fee paint in WebKit.
///
/// `viewport` is the Cypress viewport name. The vendored tree does not
/// give `macbook-16` a pixel size, so this does not invent one. The window
/// width requested from the driver is the vendored `.container-xl`
/// breakpoint, 1400. Returned widths are `getComputedStyle` from that
/// layout, not that breakpoint.
pub fn measure_desktop_rbf_transaction_paint(
    tx_body: &str,
    replacement_body: &str,
    viewport: &str,
) -> DesktopRbfMeasurement {
    #[cfg(target_arch = "wasm32")]
    {
        let _ = (tx_body, replacement_body);
        return DesktopRbfMeasurement {
            alert_css_width: String::new(),
            third_child_css_width: String::new(),
            confirmations_and_alert_overlap: true,
            alert_rect: DesktopRbfRect {
                left: 0.0,
                right: 0.0,
                top: 0.0,
                bottom: 0.0,
            },
            button_rect: DesktopRbfRect {
                left: 0.0,
                right: 0.0,
                top: 0.0,
                bottom: 0.0,
            },
            viewport: viewport.to_string(),
        };
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        measure_with_webkit(tx_body, replacement_body, viewport)
    }
}

fn confirmed(body: &str) -> bool {
    serde_json::from_str::<Value>(body)
        .ok()
        .and_then(|value| value.get("status")?.get("confirmed")?.as_bool())
        .unwrap_or(false)
}

fn replacement_txid(body: &str) -> Option<String> {
    let value: Value = serde_json::from_str(body).ok()?;
    for key in ["txReplaced", "rbfTransaction"] {
        let Some(txid) = value
            .get(key)
            .and_then(|entry| entry.get("txid"))
            .and_then(Value::as_str)
        else {
            continue;
        };
        if !txid.is_empty() {
            return Some(txid.to_string());
        }
    }
    None
}

fn escape_html(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

#[cfg(not(target_arch = "wasm32"))]
fn measure_with_webkit(
    tx_body: &str,
    replacement_body: &str,
    viewport: &str,
) -> DesktopRbfMeasurement {
    use std::io::{Read, Write};
    use std::net::{Ipv4Addr, TcpListener, TcpStream};
    use std::process::{Child, Command, Stdio};
    use std::thread;
    use std::time::{Duration, Instant};

    const LAYOUT_SCRIPT: &str = r#"
const alertEl = document.querySelector(".alert");
const third = document.querySelector(".container-xl > :nth-child(3)");
const button = document.querySelector(".btn-warning");
function rect(el) {
  const r = el.getBoundingClientRect();
  return {left: r.left, right: r.right, top: r.top, bottom: r.bottom};
}
function overlap(a, b) {
  if (a.right < b.left || b.right < a.left) return false;
  if (a.bottom < b.top || b.bottom < a.top) return false;
  return true;
}
if (!alertEl || !third || !button) {
  return JSON.stringify({missing: true, href: location.href, body: document.body ? document.body.innerHTML.slice(0, 400) : ""});
}
const alertRect = rect(alertEl);
const buttonRect = rect(button);
return JSON.stringify({
  alertWidth: getComputedStyle(alertEl).width,
  thirdWidth: getComputedStyle(third).width,
  overlap: overlap(alertRect, buttonRect),
  alertRect: alertRect,
  buttonRect: buttonRect
});
"#;

    #[derive(Deserialize)]
    struct BrowserMeasure {
        #[serde(rename = "alertWidth")]
        alert_width: String,
        #[serde(rename = "thirdWidth")]
        third_width: String,
        overlap: bool,
        #[serde(rename = "alertRect")]
        alert_rect: DesktopRbfRect,
        #[serde(rename = "buttonRect")]
        button_rect: DesktopRbfRect,
    }

    struct Driver {
        child: Child,
        port: u16,
        session: Option<String>,
        browser_pid: Option<i64>,
    }

    impl Drop for Driver {
        fn drop(&mut self) {
            if let Some(session) = self.session.take() {
                let _ = http_exchange(self.port, "DELETE", &format!("/session/{session}"), None);
            }
            let _ = self.child.kill();
            let _ = self.child.wait();
            if let Some(pid) = self.browser_pid.take() {
                let _ = Command::new("kill").arg(pid.to_string()).status();
            }
        }
    }

    fn http_exchange(port: u16, method: &str, path: &str, body: Option<&str>) -> String {
        let body = body.unwrap_or("");
        let request = format!(
            "{method} {path} HTTP/1.1\r\nHost: 127.0.0.1:{port}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
            body.len()
        );
        let mut stream = TcpStream::connect((Ipv4Addr::LOCALHOST, port))
            .unwrap_or_else(|error| panic!("webkit driver connect: {error}"));
        stream
            .set_read_timeout(Some(Duration::from_secs(30)))
            .expect("read timeout");
        stream
            .set_write_timeout(Some(Duration::from_secs(10)))
            .expect("write timeout");
        stream
            .write_all(request.as_bytes())
            .unwrap_or_else(|error| panic!("webkit driver write: {error}"));
        let mut buf = Vec::new();
        let mut chunk = [0u8; 8192];
        loop {
            match stream.read(&mut chunk) {
                Ok(0) => break,
                Ok(count) => buf.extend_from_slice(&chunk[..count]),
                Err(error)
                    if error.kind() == std::io::ErrorKind::WouldBlock
                        || error.kind() == std::io::ErrorKind::TimedOut =>
                {
                    break;
                }
                Err(error) => panic!("webkit driver read: {error}"),
            }
        }
        let text = String::from_utf8_lossy(&buf).into_owned();
        let split = text
            .find("\r\n\r\n")
            .unwrap_or_else(|| panic!("webkit driver response has no body: {text}"));
        text[split + 4..].to_string()
    }

    fn driver_value(body: &str) -> Value {
        let value: Value = serde_json::from_str(body)
            .unwrap_or_else(|error| panic!("webkit driver json ({error}): {body}"));
        if value
            .get("value")
            .and_then(|item| item.get("error"))
            .is_some()
        {
            panic!("webkit driver error: {body}");
        }
        value
    }

    let listener = TcpListener::bind((Ipv4Addr::LOCALHOST, 0)).expect("free port");
    let port = listener.local_addr().expect("port").port();
    drop(listener);

    let mut child = Command::new("WebKitWebDriver")
        .args([format!("--port={port}"), "--host=127.0.0.1".to_string()])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn WebKitWebDriver");
    let stderr = child.stderr.take();
    let mut driver = Driver {
        child,
        port,
        session: None,
        browser_pid: None,
    };

    let started = Instant::now();
    loop {
        if TcpStream::connect((Ipv4Addr::LOCALHOST, port)).is_ok() {
            break;
        }
        if started.elapsed() > Duration::from_secs(15) {
            let mut note = String::new();
            if let Some(mut err) = stderr {
                let _ = err.read_to_string(&mut note);
            }
            panic!("WebKitWebDriver did not listen on {port}: {note}");
        }
        thread::sleep(Duration::from_millis(50));
    }

    let created = http_exchange(
        port,
        "POST",
        "/session",
        Some(r#"{"capabilities":{"alwaysMatch":{"browserName":"MiniBrowser"}}}"#),
    );
    let created = driver_value(&created);
    let session = created["value"]["sessionId"]
        .as_str()
        .unwrap_or_else(|| panic!("webkit session id missing: {created}"))
        .to_string();
    driver.browser_pid = created["value"]["capabilities"]["webkit:processID"].as_i64();
    driver.session = Some(session.clone());

    // Vendored styles.scss: .container-xl max-width applies at min-width 1400px.
    // Not a Cypress macbook-16 pixel size.
    let rect_body = r#"{"x":0,"y":0,"width":1400,"height":900}"#;
    let _rect = http_exchange(
        port,
        "POST",
        &format!("/session/{session}/window/rect"),
        Some(rect_body),
    );

    let markup = desktop_rbf_transaction_markup(tx_body, replacement_body);
    let page = format!(
        "<!DOCTYPE html><html><head><meta charset=\"utf-8\"></head><body>{markup}</body></html>"
    );
    let html_path = std::env::temp_dir().join(format!(
        "splora-desktop-rbf-{}-{port}.html",
        std::process::id()
    ));
    std::fs::write(&html_path, page).expect("write desktop rbf html");
    let url = format!("file://{}", html_path.display());
    let navigate = serde_json::json!({ "url": url }).to_string();
    let navigated = http_exchange(
        port,
        "POST",
        &format!("/session/{session}/url"),
        Some(&navigate),
    );
    driver_value(&navigated);

    let execute = serde_json::json!({ "script": LAYOUT_SCRIPT, "args": [] }).to_string();
    let executed = http_exchange(
        port,
        "POST",
        &format!("/session/{session}/execute/sync"),
        Some(&execute),
    );
    let executed = driver_value(&executed);
    let reported = executed["value"]
        .as_str()
        .unwrap_or_else(|| panic!("webkit execute did not return a string: {executed}"));
    if reported.contains("\"missing\":true") {
        let _ = std::fs::remove_file(&html_path);
        panic!("desktop replace-by-fee regions were not in the laid-out paint: {reported}");
    }
    let parsed: BrowserMeasure = serde_json::from_str(reported)
        .unwrap_or_else(|error| panic!("layout json ({error}): {reported}"));
    let _ = std::fs::remove_file(&html_path);
    if !laid_out_width(&parsed.alert_width) || !laid_out_width(&parsed.third_width) {
        panic!(
            "css width was not a laid-out box: alert={} third={}",
            parsed.alert_width, parsed.third_width
        );
    }

    DesktopRbfMeasurement {
        alert_css_width: parsed.alert_width,
        third_child_css_width: parsed.third_width,
        confirmations_and_alert_overlap: parsed.overlap,
        alert_rect: parsed.alert_rect,
        button_rect: parsed.button_rect,
        viewport: viewport.to_string(),
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn laid_out_width(width: &str) -> bool {
    let Some(number) = width.strip_suffix("px") else {
        return false;
    };
    number
        .parse::<f64>()
        .is_ok_and(|value| value.is_finite() && value > 0.0)
}
