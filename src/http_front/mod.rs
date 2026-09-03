// SPDX-License-Identifier: Unlicense
//! Public TLS front for per-network indexer HTTP unix sockets.
//!
//! Strips the network prefix and `/api` for Esplora REST so the backend sees
//! `/block`. Keeps `/api/v1/ws`. Never connects to `*.electrum.sock`.
//! `/signet/...` is 307 to `/mutinynet/...` and does not open a socket.

use bytes::{Buf, Bytes};
use clap::{Arg, Command};
use hyper::header::{HeaderValue, LOCATION};
use hyper::service::service_fn;
use hyper::{Body, HeaderMap, Method, Request, Response, StatusCode};
use hyperlocal::UnixClientExt;
use rustls::ServerConfig;
use rustls::pki_types::{CertificateDer, PrivateKeyDer};
use std::net::SocketAddr;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::Once;
use tokio::net::TcpListener;
use tokio_rustls::TlsAcceptor;

const DEFAULT_SOCKET_DIR: &str = "/run/splora";

/// Indexer HTTP unix instance names (NixOS `services.splora.instances`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NetworkId {
    Mainnet,
    Testnet3,
    Testnet4,
    Mutinynet,
    Liquid,
}

impl NetworkId {
    pub fn http_socket_file_name(self) -> &'static str {
        match self {
            NetworkId::Mainnet => "mainnet.http.sock",
            NetworkId::Testnet3 => "testnet3.http.sock",
            NetworkId::Testnet4 => "testnet4.http.sock",
            NetworkId::Mutinynet => "mutinynet.http.sock",
            NetworkId::Liquid => "liquid.http.sock",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Route {
    Redirect307 {
        location: String,
    },
    Proxy {
        network: NetworkId,
        backend_path: String,
    },
    NotFound,
}

/// Path-only routing. Query string is not part of the match.
pub fn route(path: &str) -> Route {
    if let Some(suffix) = strip_named_prefix(path, "/signet") {
        return Route::Redirect307 {
            location: format!("/mutinynet{suffix}"),
        };
    }
    if let Some(rest) = strip_named_prefix(path, "/testnet4") {
        return proxy_after_network(NetworkId::Testnet4, rest);
    }
    if let Some(rest) = strip_named_prefix(path, "/testnet") {
        return proxy_after_network(NetworkId::Testnet3, rest);
    }
    if let Some(rest) = strip_named_prefix(path, "/mutinynet") {
        return proxy_after_network(NetworkId::Mutinynet, rest);
    }
    if let Some(rest) = strip_named_prefix(path, "/liquid") {
        return proxy_after_network(NetworkId::Liquid, rest);
    }
    if path == "/api" || path.starts_with("/api/") {
        return proxy_after_network(NetworkId::Mainnet, path);
    }
    Route::NotFound
}

fn strip_named_prefix<'a>(path: &'a str, prefix: &str) -> Option<&'a str> {
    if path == prefix {
        return Some("");
    }
    let rest = path.strip_prefix(prefix)?;
    if rest.starts_with('/') {
        Some(rest)
    } else {
        None
    }
}

fn proxy_after_network(network: NetworkId, after_network: &str) -> Route {
    if after_network != "/api" && !after_network.starts_with("/api/") {
        return Route::NotFound;
    }
    Route::Proxy {
        network,
        backend_path: backend_path(after_network),
    }
}

/// `/api/v1/ws` stays `/api/v1/ws`. Other `/api/...` REST loses the `/api` prefix.
fn backend_path(path_with_api: &str) -> String {
    if path_with_api == "/api/v1/ws" || path_with_api.starts_with("/api/v1/ws/") {
        return path_with_api.to_string();
    }
    match path_with_api.strip_prefix("/api") {
        Some("") => "/".to_string(),
        Some(rest) => rest.to_string(),
        None => path_with_api.to_string(),
    }
}

pub fn http_socket_path(socket_dir: &Path, network: NetworkId) -> PathBuf {
    socket_dir.join(network.http_socket_file_name())
}

/// True when the last path component looks like an Electrum newline socket.
pub fn is_electrum_socket(path: &Path) -> bool {
    path.file_name()
        .and_then(|n| n.to_str())
        .is_some_and(|n| n.ends_with("electrum.sock"))
}

fn ensure_crypto_provider() {
    static START: Once = Once::new();
    START.call_once(|| {
        let _ = rustls::crypto::aws_lc_rs::default_provider().install_default();
    });
}

/// TLS 1.3 only. Suites: AES-GCM and ChaCha20-Poly1305. No SHA-1. No MD5.
pub fn tls13_server_config(
    certs: Vec<CertificateDer<'static>>,
    key: PrivateKeyDer<'static>,
    alpn: &[&[u8]],
) -> Result<Arc<ServerConfig>, String> {
    ensure_crypto_provider();
    let mut provider = rustls::crypto::aws_lc_rs::default_provider();
    provider.cipher_suites.retain(|cs| {
        matches!(
            cs.suite(),
            rustls::CipherSuite::TLS13_AES_128_GCM_SHA256
                | rustls::CipherSuite::TLS13_AES_256_GCM_SHA384
                | rustls::CipherSuite::TLS13_CHACHA20_POLY1305_SHA256
        )
    });
    let mut config = ServerConfig::builder_with_provider(Arc::new(provider))
        .with_protocol_versions(&[&rustls::version::TLS13])
        .map_err(|e| e.to_string())?
        .with_no_client_auth()
        .with_single_cert(certs, key)
        .map_err(|e| e.to_string())?;
    config.alpn_protocols = alpn.iter().map(|p| p.to_vec()).collect();
    Ok(Arc::new(config))
}

pub fn load_tls_pem(
    cert_pem: &[u8],
    key_pem: &[u8],
) -> Result<(Vec<CertificateDer<'static>>, PrivateKeyDer<'static>), String> {
    let certs = rustls_pemfile::certs(&mut &*cert_pem)
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| format!("tls cert: {e}"))?;
    if certs.is_empty() {
        return Err("tls cert: no certificates".into());
    }
    let key = rustls_pemfile::private_key(&mut &*key_pem)
        .map_err(|e| format!("tls key: {e}"))?
        .ok_or_else(|| "tls key: no private key".to_string())?;
    Ok((certs, key))
}

pub fn cli() -> Command {
    Command::new("splora-http")
        .about("TLS path-routing front for indexer HTTP unix sockets. Not Electrum newline.")
        .color(clap::ColorChoice::Never)
        .arg(
            Arg::new("socket-dir")
                .long("socket-dir")
                .num_args(1)
                .default_value(DEFAULT_SOCKET_DIR)
                .help("Directory of <instance>.http.sock files (default /run/splora)."),
        )
        .arg(
            Arg::new("listen")
                .long("listen")
                .num_args(1)
                .required(true)
                .help("TCP listen address for TLS HTTP/1.1 and HTTP/2 (ALPN h2, http/1.1)."),
        )
        .arg(
            Arg::new("quic")
                .long("quic")
                .num_args(1)
                .help("UDP listen address for HTTP/3 (ALPN h3). Defaults to --listen."),
        )
        .arg(
            Arg::new("tls-cert")
                .long("tls-cert")
                .num_args(1)
                .required(true)
                .help("PEM certificate chain."),
        )
        .arg(
            Arg::new("tls-key")
                .long("tls-key")
                .num_args(1)
                .required(true)
                .help("PEM private key."),
        )
}

pub fn run_from_args() -> Result<(), String> {
    let m = cli().get_matches();
    let socket_dir = PathBuf::from(m.get_one::<String>("socket-dir").unwrap());
    let listen: SocketAddr = m
        .get_one::<String>("listen")
        .unwrap()
        .parse()
        .map_err(|e| format!("invalid --listen: {e}"))?;
    let quic: SocketAddr = match m.get_one::<String>("quic") {
        Some(s) => s.parse().map_err(|e| format!("invalid --quic: {e}"))?,
        None => listen,
    };
    let cert = std::fs::read(m.get_one::<String>("tls-cert").unwrap())
        .map_err(|e| format!("tls-cert: {e}"))?;
    let key = std::fs::read(m.get_one::<String>("tls-key").unwrap())
        .map_err(|e| format!("tls-key: {e}"))?;
    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .map_err(|e| e.to_string())?;
    rt.block_on(serve(socket_dir, listen, quic, &cert, &key))
}

pub async fn serve(
    socket_dir: PathBuf,
    listen: SocketAddr,
    quic: SocketAddr,
    cert_pem: &[u8],
    key_pem: &[u8],
) -> Result<(), String> {
    let (certs_tcp, key_tcp) = load_tls_pem(cert_pem, key_pem)?;
    let (certs_quic, key_quic) = load_tls_pem(cert_pem, key_pem)?;
    let tcp_cfg = tls13_server_config(certs_tcp, key_tcp, &[b"h2", b"http/1.1"])?;
    let quic_cfg = tls13_server_config(certs_quic, key_quic, &[b"h3"])?;
    let socket_dir = Arc::new(socket_dir);
    let tcp = serve_tcp_tls(listen, tcp_cfg, Arc::clone(&socket_dir));
    let udp = serve_quic(quic, quic_cfg, socket_dir);
    tokio::select! {
        r = tcp => r,
        r = udp => r,
    }
}

async fn serve_tcp_tls(
    addr: SocketAddr,
    tls: Arc<ServerConfig>,
    socket_dir: Arc<PathBuf>,
) -> Result<(), String> {
    let listener = TcpListener::bind(addr)
        .await
        .map_err(|e| format!("tcp bind {addr}: {e}"))?;
    let acceptor = TlsAcceptor::from(tls);
    loop {
        let (stream, _) = listener
            .accept()
            .await
            .map_err(|e| format!("tcp accept: {e}"))?;
        let acceptor = acceptor.clone();
        let socket_dir = Arc::clone(&socket_dir);
        tokio::spawn(async move {
            let tls_stream = match acceptor.accept(stream).await {
                Ok(s) => s,
                Err(_) => return,
            };
            let alpn = tls_stream.get_ref().1.alpn_protocol().map(|p| p.to_vec());
            let dir = Arc::clone(&socket_dir);
            let svc = service_fn(move |req: Request<Body>| {
                let dir = Arc::clone(&dir);
                async move { Ok::<_, hyper::Error>(handle_front(req, dir.as_path()).await) }
            });
            let mut http = hyper::server::conn::Http::new();
            match alpn.as_deref() {
                Some(b"h2") => {
                    http.http2_only(true);
                }
                _ => {
                    http.http1_only(true);
                }
            }
            let _ = http.serve_connection(tls_stream, svc).with_upgrades().await;
        });
    }
}

async fn serve_quic(
    addr: SocketAddr,
    tls: Arc<ServerConfig>,
    socket_dir: Arc<PathBuf>,
) -> Result<(), String> {
    let quic_crypto = quinn::crypto::rustls::QuicServerConfig::try_from(tls)
        .map_err(|e| format!("quic tls: {e}"))?;
    let server_config = quinn::ServerConfig::with_crypto(Arc::new(quic_crypto));
    let endpoint = quinn::Endpoint::server(server_config, addr)
        .map_err(|e| format!("quic bind {addr}: {e}"))?;
    loop {
        let incoming = match endpoint.accept().await {
            Some(i) => i,
            None => return Ok(()),
        };
        let socket_dir = Arc::clone(&socket_dir);
        tokio::spawn(async move {
            let conn = match incoming.await {
                Ok(c) => c,
                Err(_) => return,
            };
            if let Err(e) = handle_h3_connection(conn, socket_dir).await {
                log::debug!("h3 connection: {e}");
            }
        });
    }
}

async fn handle_h3_connection(
    conn: quinn::Connection,
    socket_dir: Arc<PathBuf>,
) -> Result<(), String> {
    // Bytes must be named: h3_quinn::Connection implements Connection<B> for every Buf.
    let mut h3_conn =
        h3::server::Connection::<h3_quinn::Connection, Bytes>::new(h3_quinn::Connection::new(conn))
            .await
            .map_err(|e| e.to_string())?;
    loop {
        match h3_conn.accept().await {
            Ok(Some(resolver)) => {
                let socket_dir = Arc::clone(&socket_dir);
                tokio::spawn(handle_h3_accepted(resolver, socket_dir));
            }
            Ok(None) => break,
            Err(e) => return Err(e.to_string()),
        }
    }
    Ok(())
}

async fn handle_h3_accepted(
    resolver: h3::server::RequestResolver<h3_quinn::Connection, Bytes>,
    socket_dir: Arc<PathBuf>,
) {
    match resolver.resolve_request().await {
        Ok((req, stream)) => {
            if let Err(e) = handle_h3_resolved(req, stream, socket_dir).await {
                log::debug!("h3 request: {e}");
            }
        }
        Err(e) => log::debug!("h3 resolve: {e}"),
    }
}

async fn handle_h3_resolved(
    req: http::Request<()>,
    mut stream: h3::server::RequestStream<h3_quinn::BidiStream<Bytes>, Bytes>,
    socket_dir: Arc<PathBuf>,
) -> Result<(), String> {
    let mut body = Vec::new();
    while let Some(mut chunk) = stream.recv_data().await.map_err(|e| e.to_string())? {
        let n = chunk.remaining();
        body.extend_from_slice(&chunk.copy_to_bytes(n));
    }
    let method = Method::from_bytes(req.method().as_str().as_bytes()).unwrap_or(Method::GET);
    let path = req.uri().path().to_string();
    let query = req.uri().query().map(str::to_string);
    let mut headers = HeaderMap::new();
    for (k, v) in req.headers().iter() {
        if let Ok(name) = hyper::header::HeaderName::from_bytes(k.as_str().as_bytes())
            && let Ok(val) = HeaderValue::from_bytes(v.as_bytes())
        {
            headers.append(name, val);
        }
    }
    let resp = dispatch(
        method,
        &path,
        query.as_deref(),
        headers,
        Body::from(body),
        None,
        socket_dir.as_path(),
    )
    .await;
    let status = resp.status().as_u16();
    let mut builder = http::Response::builder().status(status);
    for (k, v) in resp.headers().iter() {
        builder = builder.header(k.as_str(), v.as_bytes());
    }
    let out = hyper::body::to_bytes(resp.into_body())
        .await
        .map_err(|e| e.to_string())?;
    let h3_resp = builder.body(()).map_err(|e| e.to_string())?;
    stream
        .send_response(h3_resp)
        .await
        .map_err(|e| e.to_string())?;
    if !out.is_empty() {
        stream.send_data(out).await.map_err(|e| e.to_string())?;
    }
    stream.finish().await.map_err(|e| e.to_string())?;
    Ok(())
}

/// Route plus unix HTTP/1.1 proxy. Signet returns 307 without connecting.
pub async fn handle_front(mut req: Request<Body>, socket_dir: &Path) -> Response<Body> {
    let path = req.uri().path().to_string();
    let query = req.uri().query().map(str::to_string);
    let method = req.method().clone();
    // hyper 0.14 CanUpgrade is Request/Response only, not http::request::Parts.
    let upgrade = if req.headers().get("upgrade").is_some() {
        Some(hyper::upgrade::on(&mut req))
    } else {
        None
    };
    let (parts, body) = req.into_parts();
    dispatch(
        method,
        &path,
        query.as_deref(),
        parts.headers,
        body,
        upgrade,
        socket_dir,
    )
    .await
}

async fn dispatch(
    method: Method,
    path: &str,
    query: Option<&str>,
    headers: HeaderMap,
    body: Body,
    upgrade: Option<hyper::upgrade::OnUpgrade>,
    socket_dir: &Path,
) -> Response<Body> {
    match route(path) {
        Route::Redirect307 { location } => {
            let location = match query {
                Some(q) => format!("{location}?{q}"),
                None => location,
            };
            Response::builder()
                .status(StatusCode::TEMPORARY_REDIRECT)
                .header(LOCATION, location)
                .body(Body::empty())
                .unwrap_or_else(|_| Response::new(Body::empty()))
        }
        Route::NotFound => Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body(Body::from("not found"))
            .unwrap_or_else(|_| Response::new(Body::from("not found"))),
        Route::Proxy {
            network,
            backend_path,
        } => {
            let socket = http_socket_path(socket_dir, network);
            if is_electrum_socket(&socket) {
                return Response::builder()
                    .status(StatusCode::INTERNAL_SERVER_ERROR)
                    .body(Body::from("electrum socket refused"))
                    .unwrap_or_else(|_| Response::new(Body::empty()));
            }
            proxy_unix(
                method,
                &socket,
                &backend_path,
                query,
                headers,
                body,
                upgrade,
            )
            .await
        }
    }
}

async fn proxy_unix(
    method: Method,
    socket: &Path,
    backend_path: &str,
    query: Option<&str>,
    mut headers: HeaderMap,
    body: Body,
    incoming_upgrade: Option<hyper::upgrade::OnUpgrade>,
) -> Response<Body> {
    if !headers.contains_key("x-forwarded-proto") {
        headers.insert("x-forwarded-proto", HeaderValue::from_static("https"));
    }
    let uri_path = match query {
        Some(q) => format!("{backend_path}?{q}"),
        None => backend_path.to_string(),
    };
    let uri: hyper::Uri = hyperlocal::Uri::new(socket, &uri_path).into();
    let mut builder = Request::builder().method(method).uri(uri);
    for (k, v) in headers.iter() {
        builder = builder.header(k, v);
    }
    let backend_req = match builder.body(body) {
        Ok(r) => r,
        Err(_) => {
            return Response::builder()
                .status(StatusCode::BAD_REQUEST)
                .body(Body::from("bad request"))
                .unwrap_or_else(|_| Response::new(Body::empty()));
        }
    };
    let client = hyper::Client::unix();
    let mut resp = match client.request(backend_req).await {
        Ok(r) => r,
        Err(_) => {
            return Response::builder()
                .status(StatusCode::BAD_GATEWAY)
                .body(Body::from("backend unavailable"))
                .unwrap_or_else(|_| Response::new(Body::empty()));
        }
    };
    if resp.status() == StatusCode::SWITCHING_PROTOCOLS
        && let Some(client_up) = incoming_upgrade
    {
        let server_up = hyper::upgrade::on(&mut resp);
        tokio::spawn(async move {
            if let (Ok(mut c), Ok(mut s)) = (client_up.await, server_up.await) {
                let _ = tokio::io::copy_bidirectional(&mut c, &mut s).await;
            }
        });
    }
    resp
}

#[cfg(test)]
mod tests {
    use super::*;
    use hyper::service::{make_service_fn, service_fn};
    use hyperlocal::UnixServerExt;
    use rustls::ClientConfig;
    use rustls::pki_types::ServerName;
    use std::sync::Mutex;
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::time::Duration;
    use tokio::net::TcpStream;
    use tokio_rustls::TlsConnector;

    const TEST_CERT_PEM: &[u8] = b"-----BEGIN CERTIFICATE-----
MIIBfTCCASOgAwIBAgIUYn9RUGf5PQ7jUBggFDmoubk0RF8wCgYIKoZIzj0EAwIw
FDESMBAGA1UEAwwJbG9jYWxob3N0MB4XDTI2MDkwMzAzNDI0OFoXDTM2MDgzMTAz
NDI0OFowFDESMBAGA1UEAwwJbG9jYWxob3N0MFkwEwYHKoZIzj0CAQYIKoZIzj0D
AQcDQgAEBlWuLnC3FsEPsDn+9Na2fMOEdGGTXDbeJSB2ut1sHGAf3iL4rhGRLX8N
sqIONFak8j1nGOUrBxTKpTfwaFchMKNTMFEwHQYDVR0OBBYEFEeNch/pbgFWeEKY
/+s9ueZifITKMB8GA1UdIwQYMBaAFEeNch/pbgFWeEKY/+s9ueZifITKMA8GA1Ud
EwEB/wQFMAMBAf8wCgYIKoZIzj0EAwIDSAAwRQIgFakrQ3eDEi73rggFaTr9dUtM
jWTQj6nDVKBC9vHUCjICIQCg3gCTXoV72gTzXOKcwaCH47A7KxYcWUwaPyhNptqo
3g==
-----END CERTIFICATE-----
";

    const TEST_KEY_PEM: &[u8] = b"-----BEGIN PRIVATE KEY-----
MIGHAgEAMBMGByqGSM49AgEGCCqGSM49AwEHBG0wawIBAQQg7sLa3kwvsw81mxjr
VVljnqHYjWZeoHbN8M+6GN2MtHqhRANCAAQGVa4ucLcWwQ+wOf701rZ8w4R0YZNc
Nt4lIHa63WwcYB/eIviuEZEtfw2yog40VqTyPWcY5SsHFMqlN/BoVyEw
-----END PRIVATE KEY-----
";

    async fn echo_path_server(
        socket: PathBuf,
        hits: Arc<AtomicU64>,
        last_path: Arc<Mutex<String>>,
    ) {
        if socket.exists() {
            let _ = std::fs::remove_file(&socket);
        }
        let make = make_service_fn(move |_| {
            let hits = Arc::clone(&hits);
            let last_path = Arc::clone(&last_path);
            async move {
                Ok::<_, hyper::Error>(service_fn(move |req: Request<Body>| {
                    let hits = Arc::clone(&hits);
                    let last_path = Arc::clone(&last_path);
                    async move {
                        hits.fetch_add(1, Ordering::SeqCst);
                        *last_path.lock().unwrap() = req.uri().path().to_string();
                        Ok::<_, hyper::Error>(Response::new(Body::from(
                            req.uri().path().to_string(),
                        )))
                    }
                }))
            }
        });
        hyper::Server::bind_unix(&socket)
            .expect("bind unix echo")
            .serve(make)
            .await
            .ok();
    }

    async fn wait_for_socket(path: &Path) {
        let deadline = std::time::Instant::now() + Duration::from_secs(5);
        while std::time::Instant::now() < deadline {
            if path.exists() {
                return;
            }
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
        panic!("unix socket did not appear: {:?}", path);
    }

    fn get(path: &str) -> Request<Body> {
        Request::get(path).body(Body::empty()).unwrap()
    }

    /// Named contract: `/signet/api/tx/x` returns 307 Location `/mutinynet/api/tx/x`
    /// and does not connect a backend.
    #[tokio::test]
    async fn signet_api_tx_returns_307_to_mutinynet_and_does_not_connect_backend() {
        let dir = tempfile::tempdir().unwrap();
        let sock = dir.path().join("mutinynet.http.sock");
        let hits = Arc::new(AtomicU64::new(0));
        let last = Arc::new(Mutex::new(String::new()));
        tokio::spawn(echo_path_server(sock.clone(), Arc::clone(&hits), last));
        wait_for_socket(&sock).await;

        let resp = handle_front(get("/signet/api/tx/x"), dir.path()).await;
        assert_eq!(resp.status(), StatusCode::TEMPORARY_REDIRECT);
        assert_eq!(resp.headers().get(LOCATION).unwrap(), "/mutinynet/api/tx/x");
        tokio::time::sleep(Duration::from_millis(50)).await;
        assert_eq!(
            hits.load(Ordering::SeqCst),
            0,
            "signet must not open the mutinynet socket"
        );
    }

    /// Named contract: `/liquid/api/tx/x` hits liquid UDS with path `/tx/x`.
    #[tokio::test]
    async fn liquid_api_tx_hits_liquid_uds_with_path_tx() {
        let dir = tempfile::tempdir().unwrap();
        let sock = http_socket_path(dir.path(), NetworkId::Liquid);
        assert_eq!(sock.file_name().unwrap(), "liquid.http.sock");
        let hits = Arc::new(AtomicU64::new(0));
        let last = Arc::new(Mutex::new(String::new()));
        tokio::spawn(echo_path_server(
            sock.clone(),
            Arc::clone(&hits),
            Arc::clone(&last),
        ));
        wait_for_socket(&sock).await;

        let resp = handle_front(get("/liquid/api/tx/x"), dir.path()).await;
        assert_eq!(resp.status(), StatusCode::OK);
        let body = hyper::body::to_bytes(resp.into_body()).await.unwrap();
        assert_eq!(body.as_ref(), b"/tx/x");
        assert_eq!(last.lock().unwrap().as_str(), "/tx/x");
        assert_eq!(hits.load(Ordering::SeqCst), 1);
        assert!(!is_electrum_socket(&sock));
    }

    /// Named contract: `/api/v1/ws` on mainnet stays `/api/v1/ws` on mainnet.http.sock.
    #[tokio::test]
    async fn mainnet_api_v1_ws_stays_on_mainnet_http_sock() {
        let dir = tempfile::tempdir().unwrap();
        let sock = http_socket_path(dir.path(), NetworkId::Mainnet);
        assert_eq!(sock.file_name().unwrap(), "mainnet.http.sock");
        let hits = Arc::new(AtomicU64::new(0));
        let last = Arc::new(Mutex::new(String::new()));
        tokio::spawn(echo_path_server(
            sock.clone(),
            Arc::clone(&hits),
            Arc::clone(&last),
        ));
        wait_for_socket(&sock).await;

        let resp = handle_front(get("/api/v1/ws"), dir.path()).await;
        assert_eq!(resp.status(), StatusCode::OK);
        let body = hyper::body::to_bytes(resp.into_body()).await.unwrap();
        assert_eq!(body.as_ref(), b"/api/v1/ws");
        assert_eq!(last.lock().unwrap().as_str(), "/api/v1/ws");
        match route("/api/v1/ws") {
            Route::Proxy {
                network,
                backend_path,
            } => {
                assert_eq!(network, NetworkId::Mainnet);
                assert_eq!(backend_path, "/api/v1/ws");
                let p = http_socket_path(dir.path(), network);
                assert_eq!(p.file_name().unwrap(), "mainnet.http.sock");
            }
            other => panic!("expected mainnet proxy, got {:?}", other),
        }
    }

    /// Named contract: electrum.sock is never a target.
    #[tokio::test]
    async fn electrum_sock_is_never_a_proxy_target() {
        let dir = tempfile::tempdir().unwrap();
        let electrum = dir.path().join("mainnet.electrum.sock");
        let hits = Arc::new(AtomicU64::new(0));
        let last = Arc::new(Mutex::new(String::new()));
        tokio::spawn(echo_path_server(electrum.clone(), Arc::clone(&hits), last));
        wait_for_socket(&electrum).await;

        let paths = [
            "/api/tx/x",
            "/api/v1/ws",
            "/api/electrum",
            "/testnet/api/tx/x",
            "/testnet4/api/tx/x",
            "/mutinynet/api/tx/x",
            "/liquid/api/tx/x",
        ];
        for path in paths {
            match route(path) {
                Route::Proxy { network, .. } => {
                    let sock = http_socket_path(dir.path(), network);
                    assert!(
                        !is_electrum_socket(&sock),
                        "route {path} targeted {:?}",
                        sock
                    );
                    assert!(
                        sock.file_name()
                            .unwrap()
                            .to_str()
                            .unwrap()
                            .ends_with(".http.sock"),
                        "route {path} socket {:?}",
                        sock
                    );
                }
                other => panic!("expected proxy for {path}, got {:?}", other),
            }
        }

        let resp = handle_front(get("/api/tx/x"), dir.path()).await;
        tokio::time::sleep(Duration::from_millis(50)).await;
        assert_eq!(
            hits.load(Ordering::SeqCst),
            0,
            "must not connect to mainnet.electrum.sock"
        );
        assert_eq!(resp.status(), StatusCode::BAD_GATEWAY);
        assert!(is_electrum_socket(&electrum));
    }

    /// Named contract: TLS 1.2 handshake refused when this process terminates TLS.
    #[tokio::test]
    async fn tls_1_2_handshake_is_refused() {
        let (certs, key) = load_tls_pem(TEST_CERT_PEM, TEST_KEY_PEM).unwrap();
        let server_cfg = tls13_server_config(certs.clone(), key, &[b"h2", b"http/1.1"]).unwrap();
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        let acceptor = TlsAcceptor::from(server_cfg);
        let server = tokio::spawn(async move {
            let (stream, _) = listener.accept().await.unwrap();
            match acceptor.accept(stream).await {
                Ok(_) => panic!("TLS 1.2 handshake must not succeed"),
                Err(_) => {}
            }
        });

        ensure_crypto_provider();
        let mut roots = rustls::RootCertStore::empty();
        roots.add(certs[0].clone()).unwrap();
        let client_cfg = ClientConfig::builder_with_provider(Arc::new(
            rustls::crypto::aws_lc_rs::default_provider(),
        ))
        .with_protocol_versions(&[&rustls::version::TLS12])
        .expect("tls1.2 client")
        .with_root_certificates(roots)
        .with_no_client_auth();
        let connector = TlsConnector::from(Arc::new(client_cfg));
        let stream = TcpStream::connect(addr).await.unwrap();
        let name = ServerName::try_from("localhost").unwrap();
        let result = connector.connect(name, stream).await;
        assert!(
            result.is_err(),
            "TLS 1.2 client must be refused by a TLS 1.3-only server"
        );
        server.await.unwrap();
    }
}
