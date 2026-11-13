//! Truth tests for the self-diagnosis surfaces:
//! `GET /api/logs`, `/diagnose` audit metadata, and the derived tool census.
//!
//! These tests use a real loopback listener and raw HTTP so they exercise the
//! same Host/Origin/launcher-token gates as the browser. Test files run in
//! separate processes; the one process-wide env mutation is serialized by
//! `ENV_LOCK` inside this file.

use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use hiverouter_core::config::Config;
use hiverouter_core::db::Database;
use hiverouter_core::http::{self, AppState};
use hiverouter_core::run_checks;
use serde_json::Value;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::oneshot;

/// Serializes tests that share the process-wide `HIVEROUTER_LAUNCH_TOKEN`.
static ENV_LOCK: Mutex<()> = Mutex::new(());

/// Removes the launch-token env var even when an assertion panics.
struct LaunchTokenGuard;

impl LaunchTokenGuard {
    fn set(value: &str) -> Self {
        std::env::set_var("HIVEROUTER_LAUNCH_TOKEN", value);
        Self
    }
}

impl Drop for LaunchTokenGuard {
    fn drop(&mut self) {
        std::env::remove_var("HIVEROUTER_LAUNCH_TOKEN");
    }
}

fn has_no_store(response: &RawResponse) -> bool {
    response
        .headers
        .to_ascii_lowercase()
        .contains("cache-control: no-store")
}

struct TestServer {
    addr: SocketAddr,
    shutdown: Option<oneshot::Sender<()>>,
    handle: tokio::task::JoinHandle<()>,
}

impl TestServer {
    async fn start(temp: &tempfile::TempDir) -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").await.expect("bind");
        let addr = listener.local_addr().expect("local addr");
        // Bind first, then resolve the config with the real port so the Origin
        // allowlist contains the authority the browser would send.
        let config = Config::resolve_with_workspace(
            &addr.to_string(),
            temp.path().to_str().expect("utf8 path"),
            None,
            Some(temp.path()),
        )
        .expect("test config");
        let database = Database::open(config.db_file()).expect("database");
        database.migrate().expect("migrations");
        let state = Arc::new(AppState::new(
            Arc::new(database),
            run_checks(),
            Arc::new(config),
        ));
        let (shutdown, shutdown_rx) = oneshot::channel();
        let served_state = state.clone();
        let handle = tokio::spawn(async move {
            let _ = http::serve(listener, served_state, async move {
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
        self.handle.abort();
    }
}

struct RawResponse {
    status: u16,
    headers: String,
    body: String,
}

async fn request(addr: SocketAddr, path: &str, extra_headers: &[(&str, &str)]) -> RawResponse {
    let mut stream = TcpStream::connect(addr).await.expect("connect");
    let mut request = format!(
        "GET {path} HTTP/1.1\r\nHost: {addr}\r\nConnection: close\r\nAccept: application/json\r\n"
    );
    for (name, value) in extra_headers {
        request.push_str(name);
        request.push_str(": ");
        request.push_str(value);
        request.push_str("\r\n");
    }
    request.push_str("\r\n");
    stream
        .write_all(request.as_bytes())
        .await
        .expect("write request");
    let mut raw = Vec::new();
    stream.read_to_end(&mut raw).await.expect("read response");
    let text = String::from_utf8_lossy(&raw).into_owned();
    let (head, body) = text.split_once("\r\n\r\n").unwrap_or((text.as_str(), ""));
    let status = head
        .lines()
        .next()
        .and_then(|line| line.split_whitespace().nth(1))
        .and_then(|code| code.parse::<u16>().ok())
        .expect("http status line");
    RawResponse {
        status,
        headers: head.to_owned(),
        body: body.to_owned(),
    }
}

fn write_log(dir: &std::path::Path, name: &str, body: &str) {
    std::fs::write(dir.join(name), body.as_bytes()).expect("write log");
}

#[tokio::test]
async fn logs_route_requires_launcher_token_and_allowlisted_origin() {
    let _guard = ENV_LOCK.lock().expect("env lock");
    let temp = tempfile::tempdir().expect("temp dir");
    let server = TestServer::start(&temp).await;
    write_log(temp.path(), "server.stdout.log", "boot line\ntail line\n");
    let expected_token = "launch-token-for-test-0123456789";
    let _token_guard = LaunchTokenGuard::set(expected_token);

    let path = "/api/logs?source=stdout&tail=5";
    let no_token = request(server.addr, path, &[]).await;
    assert_eq!(no_token.status, 403, "body: {}", no_token.body);
    let no_token_json: Value = serde_json::from_str(&no_token.body).expect("refusal JSON");
    assert_eq!(no_token_json["code"], "E_ORIGIN_REJECTED");
    assert!(
        has_no_store(&no_token),
        "refusals must not be cached: {}",
        no_token.headers
    );

    let wrong_token = request(server.addr, path, &[("x-hivemind-launch", "not-the-token")]).await;
    assert_eq!(wrong_token.status, 403, "body: {}", wrong_token.body);

    let allowed = request(server.addr, path, &[("x-hivemind-launch", expected_token)]).await;
    assert_eq!(allowed.status, 200, "body: {}", allowed.body);
    assert!(has_no_store(&allowed));
    let value: Value = serde_json::from_str(&allowed.body).expect("logs JSON");
    assert_eq!(value["schema"], "hiverouter.logs.v1");
    assert_eq!(value["source"], "stdout");
    assert!(value["content"]
        .as_str()
        .expect("content")
        .contains("tail line"));

    let allowed_origin = format!("http://{}", server.addr);
    let same_origin = request(
        server.addr,
        path,
        &[
            ("x-hivemind-launch", expected_token),
            ("Origin", &allowed_origin),
        ],
    )
    .await;
    assert_eq!(same_origin.status, 200, "body: {}", same_origin.body);

    let hostile_origin = request(
        server.addr,
        path,
        &[
            ("x-hivemind-launch", expected_token),
            ("Origin", "http://evil.example"),
        ],
    )
    .await;
    assert_eq!(hostile_origin.status, 403, "body: {}", hostile_origin.body);
    let hostile_json: Value = serde_json::from_str(&hostile_origin.body).expect("refusal JSON");
    assert_eq!(hostile_json["code"], "E_ORIGIN_REJECTED");
    assert!(hostile_json["detail"]
        .as_str()
        .unwrap_or_default()
        .contains("Origin"));
}

#[tokio::test]
async fn logs_route_redacts_secrets_and_bounds_the_body() {
    let _guard = ENV_LOCK.lock().expect("env lock");
    let temp = tempfile::tempdir().expect("temp dir");
    let server = TestServer::start(&temp).await;
    let body = concat!(
        "Authorization: Bearer abcdef1234567890SECRET\n",
        "{\"api_key\":\"sk-live-abcdef1234567890\"}\n",
        "DEEPSEEK_API_KEY=env-secret-value-12345\n",
        "https://127.0.0.1:8099/?launch=deadbeefdeadbeef\n",
        "final line\n"
    );
    write_log(temp.path(), "server.stdout.log", body);

    let response = request(server.addr, "/api/logs?source=stdout&tail=5", &[]).await;
    assert_eq!(response.status, 200, "body: {}", response.body);
    assert!(has_no_store(&response), "{}", response.headers);
    let value: Value = serde_json::from_str(&response.body).expect("logs JSON");
    let content = value["content"].as_str().expect("content");
    assert!(content.contains("final line"), "{content}");
    assert!(content.contains("[REDACTED]"), "{content}");
    assert!(!content.contains("abcdef1234567890SECRET"), "{content}");
    assert!(!content.contains("sk-live-abcdef1234567890"), "{content}");
    assert!(!content.contains("env-secret-value-12345"), "{content}");
    assert!(!content.contains("deadbeefdeadbeef"), "{content}");
    assert!(value["returned_bytes"].as_u64().expect("bytes") <= 65_536);

    // A body larger than the cap is read from the end, never returned whole.
    let mut oversized = String::from("HEAD-MARKER\n");
    while oversized.len() < 160 * 1024 {
        oversized.push_str("0123456789abcdef0123456789abcdef\n");
    }
    oversized.push_str("TAIL-MARKER\n");
    write_log(temp.path(), "launcher.log", &oversized);
    let response = request(server.addr, "/api/logs?source=launcher&tail=5000", &[]).await;
    assert_eq!(response.status, 200, "body: {}", response.body);
    let value: Value = serde_json::from_str(&response.body).expect("logs JSON");
    let content = value["content"].as_str().expect("content");
    assert_eq!(value["truncated"], true);
    assert!(!content.contains("HEAD-MARKER"), "cap must cut the head");
    assert!(content.contains("TAIL-MARKER"), "cap must keep the tail");
    assert!(value["returned_bytes"].as_u64().expect("bytes") <= 65_536);
}

#[tokio::test]
async fn logs_route_refuses_traversal_unknown_sources_and_bad_tail() {
    let _guard = ENV_LOCK.lock().expect("env lock");
    let temp = tempfile::tempdir().expect("temp dir");
    let server = TestServer::start(&temp).await;
    std::fs::write(temp.path().join("outside-secret.txt"), b"outside-secret")
        .expect("outside file");
    write_log(temp.path(), "server.stdout.log", "inside line\n");

    for path in [
        "/api/logs?source=../../outside-secret.txt&tail=5",
        "/api/logs?source=..%2F..%2Foutside-secret.txt&tail=5",
        "/api/logs?source=server.stdout.log&tail=5",
        "/api/logs?source=stdout&path=../../outside-secret.txt",
        "/api/logs?source=stdout&tail=0",
        "/api/logs?source=stdout&tail=not-a-number",
        "/api/logs?tail=5",
    ] {
        let response = request(server.addr, path, &[]).await;
        assert_eq!(
            response.status, 400,
            "{path} must be refused, body: {}",
            response.body
        );
        assert!(!response.body.contains("outside-secret"), "{path}");
        assert!(has_no_store(&response), "{path}");
    }

    let missing = request(server.addr, "/api/logs?source=stderr&tail=5", &[]).await;
    assert_eq!(missing.status, 404, "body: {}", missing.body);
    let value: Value = serde_json::from_str(&missing.body).expect("refusal JSON");
    assert_eq!(value["code"], "E_PATH_NOT_FOUND");
    assert!(has_no_store(&missing));

    let no_audit = request(server.addr, "/api/logs?source=audit&tail=5", &[]).await;
    assert_eq!(no_audit.status, 404, "body: {}", no_audit.body);
    let value: Value = serde_json::from_str(&no_audit.body).expect("refusal JSON");
    assert_eq!(value["code"], "E_PATH_NOT_FOUND");

    // The audit directory must be named even before the sink creates its first
    // file: "null" was the bug, "does not exist yet" is the honest state.
    let diagnosis = request(server.addr, "/diagnose", &[]).await;
    assert_eq!(diagnosis.status, 200, "body: {}", diagnosis.body);
    let value: Value = serde_json::from_str(&diagnosis.body).expect("diagnose JSON");
    assert_ne!(value["audit_log_dir"], Value::Null);
    assert_eq!(
        PathBuf::from(value["audit_log_dir"].as_str().expect("audit dir")),
        temp.path().join("audit")
    );
    let audit = value["log_files"]
        .as_array()
        .expect("log_files")
        .iter()
        .find(|file| file["source"] == "audit")
        .expect("audit metadata");
    assert_eq!(audit["exists"], false);
    assert_eq!(
        value["recent_refusals"].as_array().expect("refusals").len(),
        0
    );
}

#[tokio::test]
async fn audit_log_source_and_diagnose_metadata_are_real() {
    let _guard = ENV_LOCK.lock().expect("env lock");
    let temp = tempfile::tempdir().expect("temp dir");
    let audit_dir = temp.path().join("audit");
    std::fs::create_dir_all(&audit_dir).expect("audit dir");
    std::fs::write(
        audit_dir.join("audit.jsonl.2026-11-13"),
        concat!(
            "{\"timestamp\":\"2026-11-13T00:00:00Z\",\"level\":\"WARN\",\"message\":\"refusal\",\"code\":\"E_APPROVAL_TIMEOUT\",\"run_id\":\"run-1\",\"tool\":\"shell_exec\",\"permission_class\":\"execute\"}\n",
            "{\"timestamp\":\"2026-11-13T00:01:00Z\",\"level\":\"INFO\",\"message\":\"tool_call\",\"code\":\"\"}\n"
        ),
    )
    .expect("audit file");
    let server = TestServer::start(&temp).await;

    let logs = request(server.addr, "/api/logs?source=audit&tail=10", &[]).await;
    assert_eq!(logs.status, 200, "body: {}", logs.body);
    let value: Value = serde_json::from_str(&logs.body).expect("logs JSON");
    assert_eq!(value["source"], "audit");
    assert_eq!(value["file"], "audit/audit.jsonl.2026-11-13");
    assert!(value["content"]
        .as_str()
        .expect("content")
        .contains("E_APPROVAL_TIMEOUT"));

    let diagnosis = request(server.addr, "/diagnose", &[]).await;
    assert_eq!(diagnosis.status, 200, "body: {}", diagnosis.body);
    let value: Value = serde_json::from_str(&diagnosis.body).expect("diagnose JSON");
    let expected_dir = PathBuf::from(
        value["audit_log_dir"]
            .as_str()
            .expect("audit_log_dir is a real path, not null"),
    );
    assert_eq!(expected_dir, temp.path().join("audit"));
    let files = value["log_files"].as_array().expect("log_files array");
    let sources: Vec<&str> = files
        .iter()
        .filter_map(|file| file["source"].as_str())
        .collect();
    assert_eq!(sources, ["launcher", "stdout", "stderr", "audit"]);
    let audit = files
        .iter()
        .find(|file| file["source"] == "audit")
        .expect("audit metadata");
    assert_eq!(audit["exists"], true);
    assert_eq!(audit["file"], "audit/audit.jsonl.2026-11-13");
    assert_eq!(value["audit_scan"]["file"], "audit.jsonl.2026-11-13");
    let refusals = value["recent_refusals"]
        .as_array()
        .expect("recent_refusals array");
    assert_eq!(refusals.len(), 1, "{refusals:?}");
    assert_eq!(refusals[0]["code"], "E_APPROVAL_TIMEOUT");
    assert_eq!(refusals[0]["tool"], "shell_exec");
    assert!(!diagnosis.body.contains("sk-"));
    assert!(!diagnosis.body.contains("Bearer "));
}
