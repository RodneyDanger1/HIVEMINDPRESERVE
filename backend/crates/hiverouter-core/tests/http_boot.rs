//! Boot + HTTP acceptance tests for Phase 1: the binary boots on loopback, serves
//! `GET /health`, returns an honest 404, and refuses a non-loopback bind by name.

use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;

use hiverouter_core::config::Config;
use hiverouter_core::db::Database;
use hiverouter_core::http::{self, AppState};
use hiverouter_core::run_checks;
use serde_json::Value;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::oneshot;

struct TestServer {
    addr: SocketAddr,
    shutdown: Option<oneshot::Sender<()>>,
    handle: tokio::task::JoinHandle<()>,
}

impl TestServer {
    async fn start(temp: &tempfile::TempDir) -> Self {
        let config = Config::resolve(
            "127.0.0.1:0",
            temp.path().to_str().expect("utf8 path"),
            None,
        )
        .expect("test config");
        let database = Database::open(config.db_file()).expect("database");
        database.migrate().expect("migrations");
        let state = Arc::new(AppState::new(
            Arc::new(database),
            run_checks(),
            Arc::new(config),
        ));
        let listener = TcpListener::bind("127.0.0.1:0").await.expect("bind");
        let addr = listener.local_addr().expect("local addr");
        let (shutdown, shutdown_rx) = oneshot::channel();
        let handle = tokio::spawn(async move {
            let _ = http::serve(listener, state, async move {
                let _ = shutdown_rx.await;
            })
            .await;
        });
        Self {
            addr,
            shutdown: Some(shutdown),
            handle,
        }
    }
}

impl Drop for TestServer {
    fn drop(&mut self) {
        if let Some(shutdown) = self.shutdown.take() {
            let _ = shutdown.send(());
        }
    }
}

async fn http_get(addr: SocketAddr, path: &str) -> (u16, String) {
    let mut stream = TcpStream::connect(addr).await.expect("connect");
    let request =
        format!("GET {path} HTTP/1.1\r\nHost: {addr}\r\nConnection: close\r\nAccept: application/json\r\n\r\n");
    stream.write_all(request.as_bytes()).await.expect("write request");
    let mut raw = Vec::new();
    stream.read_to_end(&mut raw).await.expect("read response");
    let text = String::from_utf8_lossy(&raw).into_owned();
    let status = text
        .lines()
        .next()
        .and_then(|line| line.split_whitespace().nth(1))
        .and_then(|code| code.parse::<u16>().ok())
        .expect("http status line");
    let body = text
        .split("\r\n\r\n")
        .nth(1)
        .unwrap_or_default()
        .to_owned();
    (status, body)
}

#[tokio::test]
async fn health_endpoint_reports_boot_checks_and_database() {
    let temp = tempfile::tempdir().expect("temp dir");
    let server = TestServer::start(&temp).await;

    let (status, body) = http_get(server.addr, "/health").await;
    assert_eq!(status, 200, "health must answer 200: {body}");
    let health: Value = serde_json::from_str(&body).expect("health body is JSON");
    assert_eq!(health["schema"], "hiverouter.v1");
    assert_eq!(health["version"], env!("CARGO_PKG_VERSION"));
    assert!(
        health["status"] == "ok" || health["status"] == "degraded",
        "status must be ok/degraded, got {}",
        health["status"]
    );
    assert!(health["uptime_secs"].is_u64());
    assert_eq!(health["database"]["schema_version"], 3);

    let checks = health["checks"].as_array().expect("checks array");
    assert!(checks.iter().any(|check| check["name"] == "rustc" && check["ok"] == true));
    assert!(checks.iter().any(|check| check["name"] == "node"));
    let rustc = checks
        .iter()
        .find(|check| check["name"] == "rustc")
        .expect("rustc check");
    let detail = rustc["detail"].as_str().expect("rustc detail");
    assert!(detail.contains("MSRV 1.89.0"), "detail: {detail}");
}

#[tokio::test]
async fn static_ui_serves_index_assets_and_spa_fallback() {
    let temp = tempfile::tempdir().expect("temp dir");
    let server = TestServer::start(&temp).await;

    // GET / serves the embedded SPA shell.
    let (status, body) = http_get(server.addr, "/").await;
    assert_eq!(status, 200, "GET / must serve the embedded UI");
    assert!(
        body.contains("<div id=\"root\">"),
        "index.html must contain the app root: {body}"
    );

    // A real built asset is served from memory.
    let dist = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../web/dist");
    let asset = std::fs::read_dir(dist.join("assets"))
        .expect("web/dist/assets exists")
        .next()
        .expect("at least one built asset")
        .expect("asset entry")
        .file_name()
        .into_string()
        .expect("utf8 asset name");
    let (status, asset_body) = http_get(server.addr, &format!("/assets/{asset}")).await;
    assert_eq!(status, 200, "asset /assets/{asset} must be embedded");
    assert!(!asset_body.is_empty());

    // SPA client route (no file extension) falls back to index.html.
    let (status, spa_body) = http_get(server.addr, "/runs/example-revision").await;
    assert_eq!(status, 200);
    assert!(spa_body.contains("<div id=\"root\">"));

    // Missing asset is an honest 404.
    let (status, missing) = http_get(server.addr, "/assets/does-not-exist.js").await;
    assert_eq!(status, 404);
    let value: Value = serde_json::from_str(&missing).expect("404 body is JSON");
    assert_eq!(value["error"], "not found");

    // Unknown API path stays a JSON 404, never the SPA shell.
    let (status, api_missing) = http_get(server.addr, "/api/nope").await;
    assert_eq!(status, 404);
    let value: Value = serde_json::from_str(&api_missing).expect("api 404 is JSON");
    assert_eq!(value["error"], "not found");

    // Path traversal cannot escape web/dist: the request is refused, never served.
    let (traversal_status, traversal_body) = http_get(server.addr, "/../Cargo.toml").await;
    assert!(traversal_status == 400 || traversal_status == 404);
    assert!(!traversal_body.contains("[workspace]"), "traversal leaked a file");
}

#[tokio::test]
async fn graceful_shutdown_lets_the_server_exit() {
    let temp = tempfile::tempdir().expect("temp dir");
    let mut server = TestServer::start(&temp).await;
    let (status, _) = http_get(server.addr, "/health").await;
    assert_eq!(status, 200);
    if let Some(shutdown) = server.shutdown.take() {
        let _ = shutdown.send(());
    }
    let finished = tokio::time::timeout(Duration::from_secs(5), &mut server.handle).await;
    assert!(finished.is_ok(), "server did not stop after the shutdown signal");
}

#[tokio::test]
async fn binary_refuses_a_non_loopback_bind_by_name() {
    let temp = tempfile::tempdir().expect("temp dir");
    let output = tokio::process::Command::new(env!("CARGO_BIN_EXE_hiverouter"))
        .args([
            "--bind",
            "0.0.0.0:0",
            "--data-dir",
            temp.path().to_str().expect("utf8 path"),
        ])
        .output()
        .await
        .expect("run binary");
    assert!(!output.status.success(), "non-loopback bind must fail");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("E_ORIGIN_REJECTED"),
        "stderr must name the refusal, got: {stderr}"
    );
}

#[tokio::test]
async fn binary_help_and_version_are_successful() {
    let help = tokio::process::Command::new(env!("CARGO_BIN_EXE_hiverouter"))
        .arg("--help")
        .output()
        .await
        .expect("run --help");
    assert!(help.status.success());
    let help_text = String::from_utf8_lossy(&help.stdout);
    assert!(help_text.contains("hiverouter"), "help output: {help_text}");
    assert!(help_text.contains("--bind"));

    let version = tokio::process::Command::new(env!("CARGO_BIN_EXE_hiverouter"))
        .arg("--version")
        .output()
        .await
        .expect("run --version");
    assert!(version.status.success());
    let version_text = String::from_utf8_lossy(&version.stdout);
    assert!(version_text.contains("hiverouter.v1"), "version output: {version_text}");
}

#[tokio::test]
async fn diagnose_route_returns_tools_json() {
    let temp = tempfile::tempdir().expect("temp dir");
    let server = TestServer::start(&temp).await;
    let (status, body) = http_get(server.addr, "/diagnose").await;
    assert_eq!(status, 200, "GET /diagnose must be mounted: {body}");
    let value: Value = serde_json::from_str(&body).expect("diagnose body is JSON");
    assert_eq!(value["schema"], "hiverouter.diagnose.v1");
    // Derived census fields are present; the payload never carries the key.
    assert!(value.get("census").is_some() || value.get("tools").is_some());
    assert!(!body.contains("DEEPSEEK_API_KEY"));
    assert!(!body.contains("sk-"));
}
