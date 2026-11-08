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
    state: Arc<AppState>,
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
        let served_state = state.clone();
        let handle = tokio::spawn(async move {
            let _ = http::serve(listener, served_state, async move {
                let _ = shutdown_rx.await;
            })
            .await;
        });
        Self {
            addr,
            state,
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

async fn http_json(
    addr: SocketAddr,
    method: &str,
    path: &str,
    body: Option<&str>,
) -> (u16, String) {
    let mut stream = TcpStream::connect(addr).await.expect("connect");
    let body = body.unwrap_or_default();
    let request = format!(
        "{method} {path} HTTP/1.1\r\nHost: {addr}\r\nConnection: close\r\nAccept: application/json\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n{body}",
        body.len()
    );
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
async fn agent_profiles_import_validates_persists_and_deletes() {
    let temp = tempfile::tempdir().expect("temp dir");
    let server = TestServer::start(&temp).await;
    let profile = serde_json::json!({
        "id": "custom-writer",
        "display_name": "Custom Writer",
        "tier": "worker",
        "template_id": "content-writer",
        "allowed_tool_ids": ["file_read", "ask_parent", "finish"],
        "output_schema_id": null,
        "refusal_policy": "strict",
        "max_children": 2,
        "max_depth": 2
    });
    let (status, body) = http_json(
        server.addr,
        "POST",
        "/api/agents/profiles/import",
        Some(&serde_json::json!({ "profile": profile }).to_string()),
    )
    .await;
    assert_eq!(status, 200, "profile import: {body}");
    let (_, list) = http_get(server.addr, "/api/agents/profiles").await;
    let value: Value = serde_json::from_str(&list).expect("profiles JSON");
    let profiles = value["profiles"].as_array().expect("profiles");
    assert_eq!(profiles.len(), 8, "seven embedded + one imported");
    assert!(profiles.iter().any(|profile| profile["id"] == "custom-writer"));
    let custom = profiles
        .iter()
        .find(|profile| profile["id"] == "custom-writer")
        .expect("imported profile");
    assert_eq!(custom["max_children"], 2);
    assert_eq!(custom["max_depth"], 2);

    // The imported profile is persisted, not just installed in this AppState.
    let boot_config = Config::resolve(
        "127.0.0.1:0",
        temp.path().to_str().expect("utf8 path"),
        None,
    )
    .expect("boot config");
    let boot_db = Database::open(boot_config.db_file()).expect("boot database");
    boot_db.migrate().expect("boot migrations");
    let boot_state = AppState::new(Arc::new(boot_db), run_checks(), Arc::new(boot_config));
    assert!(
        boot_state
            .engine
            .personas()
            .iter()
            .any(|persona| persona.id == "custom-writer"),
        "imported profile must survive a fresh boot"
    );

    let (status, body) = http_json(
        server.addr,
        "DELETE",
        "/api/agents/profiles/custom-writer",
        None,
    )
    .await;
    assert_eq!(status, 200, "profile delete: {body}");
    let (_, list) = http_get(server.addr, "/api/agents/profiles").await;
    let value: Value = serde_json::from_str(&list).expect("profiles JSON");
    assert_eq!(
        value["profiles"].as_array().expect("profiles").len(),
        7,
        "delete removes the imported profile"
    );
}

#[tokio::test]
async fn agent_profiles_import_refuses_unknown_template() {
    let temp = tempfile::tempdir().expect("temp dir");
    let server = TestServer::start(&temp).await;
    let profile = serde_json::json!({
        "id": "bad-writer",
        "display_name": "Bad Writer",
        "tier": "worker",
        "template_id": "not-a-template",
        "allowed_tool_ids": [],
        "output_schema_id": null,
        "refusal_policy": "strict"
    });
    let (status, body) = http_json(
        server.addr,
        "POST",
        "/api/agents/profiles/import",
        Some(&serde_json::json!({ "profile": profile }).to_string()),
    )
    .await;
    assert_eq!(status, 400, "unknown template must refuse: {body}");
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
    assert!(health["pid"].as_u64().is_some_and(|pid| pid > 0));
    assert!(!health["boot_id"].as_str().unwrap_or_default().is_empty());
    assert!(health["data_dir"].as_str().is_some_and(|value| !value.is_empty()));
    assert!(health["workspace_root"].as_str().is_some_and(|value| !value.is_empty()));
    assert_eq!(health["database"]["schema_version"], 11);

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
async fn agent_read_surfaces_are_mounted_and_truthful() {
    let temp = tempfile::tempdir().expect("temp dir");
    let server = TestServer::start(&temp).await;

    let (status, body) = http_get(server.addr, "/api/approvals").await;
    assert_eq!(status, 200, "approvals route: {body}");
    let approvals: Value = serde_json::from_str(&body).expect("approvals JSON");
    assert_eq!(approvals["count"], 0);
    assert!(approvals["approvals"].as_array().expect("approvals array").is_empty());

    let (status, unknown) = http_get(
        server.addr,
        "/api/approvals?run_id=00000000-0000-0000-0000-000000000000",
    )
    .await;
    assert_eq!(status, 404);
    let unknown: Value = serde_json::from_str(&unknown).expect("unknown approval JSON");
    assert_eq!(unknown["code"], "E_RUN_NOT_FOUND");

    let (status, body) = http_get(server.addr, "/api/runtime").await;
    assert_eq!(status, 200, "runtime route: {body}");
    let runtime: Value = serde_json::from_str(&body).expect("runtime JSON");
    assert_eq!(runtime["status"], "idle");
    assert_eq!(runtime["caps"]["max_agents"], 64);
    assert_eq!(runtime["caps"]["max_calls"], 40);

    let (status, body) = http_get(server.addr, "/api/templates").await;
    assert_eq!(status, 200, "templates route: {body}");
    let templates: Value = serde_json::from_str(&body).expect("templates JSON");
    assert!(templates["templates"].is_array());

    let (status, body) = http_get(server.addr, "/api/workspace").await;
    assert_eq!(status, 200, "workspace route: {body}");
    let workspace: Value = serde_json::from_str(&body).expect("workspace JSON");
    assert!(workspace["workspace_root"].as_str().is_some_and(|value| !value.is_empty()));

    let (status, body) = http_get(server.addr, "/api/runs/active").await;
    assert_eq!(status, 200, "active-run route: {body}");
    let active: Value = serde_json::from_str(&body).expect("active-run JSON");
    assert!(active["active_run_id"].is_null());
    assert!(active["latest_active_run"].is_null());
}

#[tokio::test]
async fn run_scoped_memory_route_exposes_branch_rows() {
    let temp = tempfile::tempdir().expect("temp dir");
    let server = TestServer::start(&temp).await;
    let run = server
        .state
        .engine
        .store
        .create_run("p", "memory", hiverouter_contract::AgentState::Running)
        .expect("run");
    server
        .state
        .engine
        .store
        .record_agent_memory_scoped(
            "worker",
            "task:a",
            "branch-a",
            run.id,
            hiverouter_contract::TaskId::new(),
            "hive-a",
        )
        .expect("memory a");
    server
        .state
        .engine
        .store
        .record_agent_memory_scoped(
            "worker",
            "task:b",
            "branch-b",
            run.id,
            hiverouter_contract::TaskId::new(),
            "hive-b",
        )
        .expect("memory b");
    let (status, body) = http_get(
        server.addr,
        &format!("/api/agents/runs/{}/memory", run.id),
    )
    .await;
    assert_eq!(status, 200, "memory route: {body}");
    let value: Value = serde_json::from_str(&body).expect("memory JSON");
    assert_eq!(value["count"], 2);
    let scopes = value["memories"]
        .as_array()
        .expect("memories array")
        .iter()
        .filter_map(|row| row["scope_id"].as_str())
        .collect::<Vec<_>>();
    assert!(scopes.contains(&"hive-a"));
    assert!(scopes.contains(&"hive-b"));

    let (status, body) = http_get(
        server.addr,
        &format!("/api/agents/runs/{}/memory?scope=hive-a", run.id),
    )
    .await;
    assert_eq!(status, 200, "scoped memory route: {body}");
    let scoped: Value = serde_json::from_str(&body).expect("scoped memory JSON");
    assert_eq!(scoped["count"], 1);
    assert_eq!(scoped["scope"], "hive-a");

    let (status, _) = http_get(
        server.addr,
        &format!(
            "/api/agents/runs/{}/messages?task_id=00000000-0000-0000-0000-000000000001",
            run.id
        ),
    )
    .await;
    assert_eq!(status, 404, "unknown recipient task must not look like an empty inbox");

    let (status, _) = http_get(
        server.addr,
        &format!(
            "/api/agents/runs/{}/messages?from_task_id=00000000-0000-0000-0000-000000000002",
            run.id
        ),
    )
    .await;
    assert_eq!(status, 404, "unknown sender must not look like an empty bus");

    let (status, _) = http_get(
        server.addr,
        &format!("/api/agents/runs/{}/messages?kind=bogus", run.id),
    )
    .await;
    assert_eq!(status, 400, "unknown message kind must be rejected");
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
async fn agent_profiles_route_returns_embedded_registry() {
    let temp = tempfile::tempdir().expect("temp dir");
    let server = TestServer::start(&temp).await;
    let (status, body) = http_get(server.addr, "/api/agents/profiles").await;
    assert_eq!(status, 200, "profiles route: {body}");
    let value: Value = serde_json::from_str(&body).expect("profiles JSON");
    let profiles = value["profiles"].as_array().expect("profiles array");
    assert_eq!(profiles.len(), 7, "embedded persona registry");
    let ids: Vec<&str> = profiles
        .iter()
        .filter_map(|profile| profile["id"].as_str())
        .collect();
    assert!(ids.contains(&"planner"));
    assert!(ids.contains(&"content-writer"));
    assert!(ids.contains(&"code-reviewer"));
}

#[tokio::test]
async fn agent_memory_route_lists_and_clears_role_memory() {
    let temp = tempfile::tempdir().expect("temp dir");
    let server = TestServer::start(&temp).await;
    let run = server
        .state
        .engine
        .store
        .create_run("p", "memory", hiverouter_contract::AgentState::Completed)
        .expect("run");
    server
        .state
        .engine
        .store
        .record_agent_memory(
            "content-writer",
            "task:1",
            "remembered",
            run.id,
            hiverouter_contract::TaskId::new(),
        )
        .expect("memory");
    let (status, body) = http_get(server.addr, "/api/agents/memory").await;
    assert_eq!(status, 200, "memory list: {body}");
    let value: Value = serde_json::from_str(&body).expect("memory JSON");
    let profiles = value["profiles"].as_array().expect("profiles array");
    let writer = profiles
        .iter()
        .find(|profile| profile["id"] == "content-writer")
        .expect("content-writer profile");
    assert_eq!(writer["memories"][0]["key"], "task:1");

    let mut stream = TcpStream::connect(server.addr).await.expect("connect");
    let request = format!(
        "DELETE /api/agents/memory/content-writer HTTP/1.1\r\nHost: {}\r\nConnection: close\r\n\r\n",
        server.addr
    );
    stream.write_all(request.as_bytes()).await.expect("delete");
    let mut raw = Vec::new();
    stream.read_to_end(&mut raw).await.expect("read delete");
    let text = String::from_utf8_lossy(&raw);
    assert!(text.starts_with("HTTP/1.1 200"), "delete response: {text}");
    assert!(
        server
            .state
            .engine
            .store
            .agent_memory("content-writer", 8)
            .expect("memory")
            .is_empty()
    );
}

#[tokio::test]
async fn projects_route_lists_created_projects() {
    let temp = tempfile::tempdir().expect("temp dir");
    let server = TestServer::start(&temp).await;
    server
        .state
        .engine
        .store
        .create_run("project-a", "run", hiverouter_contract::AgentState::Completed)
        .expect("run");
    let (status, body) = http_get(server.addr, "/api/projects").await;
    assert_eq!(status, 200, "projects route: {body}");
    let value: Value = serde_json::from_str(&body).expect("projects JSON");
    let projects = value["projects"].as_array().expect("projects array");
    assert_eq!(projects.len(), 1);
    assert_eq!(projects[0]["id"], "project-a");
}

#[tokio::test]
async fn run_usage_route_returns_persisted_tokens() {
    let temp = tempfile::tempdir().expect("temp dir");
    let server = TestServer::start(&temp).await;
    let run = server
        .state
        .engine
        .store
        .create_run("p", "usage", hiverouter_contract::AgentState::Completed)
        .expect("run");
    server
        .state
        .engine
        .store
        .record_run_usage(
            run.id,
            &hiverouter_contract::Usage {
                prompt_tokens: Some(7),
                completion_tokens: Some(3),
                total_tokens: Some(10),
                ..hiverouter_contract::Usage::default()
            },
        )
        .expect("usage");
    let (status, body) = http_get(server.addr, &format!("/api/usage/runs/{}", run.id)).await;
    assert_eq!(status, 200, "usage route: {body}");
    let value: Value = serde_json::from_str(&body).expect("usage JSON");
    assert_eq!(value["usage"]["prompt_tokens"], 7);
    assert_eq!(value["usage"]["completion_tokens"], 3);
    assert_eq!(value["usage"]["total_tokens"], 10);
}

#[tokio::test]
async fn agent_messages_route_returns_durable_mailbox() {
    let temp = tempfile::tempdir().expect("temp dir");
    let server = TestServer::start(&temp).await;
    let run = server
        .state
        .engine
        .store
        .create_run("p", "messages", hiverouter_contract::AgentState::Running)
        .expect("run");
    let from = hiverouter_contract::TaskId::new();
    let to = hiverouter_contract::TaskId::new();
    server
        .state
        .engine
        .store
        .record_agent_message(run.id, from, to, "ask_parent", "need input")
        .expect("message");
    let (status, body) = http_get(
        server.addr,
        &format!("/api/agents/runs/{}/messages", run.id),
    )
    .await;
    assert_eq!(status, 200, "agent messages route: {body}");
    let value: Value = serde_json::from_str(&body).expect("messages JSON");
    let messages = value["messages"].as_array().expect("messages array");
    assert_eq!(messages.len(), 1);
    assert_eq!(messages[0]["from_task_id"], from.to_string());
    assert_eq!(messages[0]["to_task_id"], to.to_string());
    assert_eq!(messages[0]["text"], "need input");
}

#[tokio::test]
async fn history_messages_route_returns_persisted_transcript() {
    let temp = tempfile::tempdir().expect("temp dir");
    let server = TestServer::start(&temp).await;
    let run = server
        .state
        .engine
        .store
        .create_run("p", "hello", hiverouter_contract::AgentState::Queued)
        .expect("run");
    server
        .state
        .engine
        .store
        .append_message(run.id, hiverouter_core::entities::ChatRole::User, "hi")
        .expect("user message");
    server
        .state
        .engine
        .store
        .append_message(
            run.id,
            hiverouter_core::entities::ChatRole::Assistant,
            "hello back",
        )
        .expect("assistant message");
    let (status, body) = http_get(
        server.addr,
        &format!("/api/history/runs/{}/messages", run.id),
    )
    .await;
    assert_eq!(status, 200, "GET messages must be mounted: {body}");
    let value: Value = serde_json::from_str(&body).expect("messages body is JSON");
    assert_eq!(value["run_id"], run.id.to_string());
    assert_eq!(value["truncated"], false);
    let messages = value["messages"].as_array().expect("messages array");
    assert_eq!(messages.len(), 2);
    assert_eq!(messages[0]["role"], "user");
    assert_eq!(messages[1]["text"], "hello back");

    let (missing_status, _) = http_get(
        server.addr,
        "/api/history/runs/00000000-0000-0000-0000-000000000000/messages",
    )
    .await;
    assert_eq!(missing_status, 404);
}

#[tokio::test]
async fn history_messages_keeps_the_newest_window() {
    let temp = tempfile::tempdir().expect("temp dir");
    let server = TestServer::start(&temp).await;
    let run = server
        .state
        .engine
        .store
        .create_run("p", "long", hiverouter_contract::AgentState::Queued)
        .expect("run");
    for index in 0..=1_000 {
        server
            .state
            .engine
            .store
            .append_message(
                run.id,
                hiverouter_core::entities::ChatRole::User,
                &format!("message-{index}"),
            )
            .expect("message");
    }
    let (status, body) = http_get(
        server.addr,
        &format!("/api/history/runs/{}/messages", run.id),
    )
    .await;
    assert_eq!(status, 200);
    let value: Value = serde_json::from_str(&body).expect("messages body is JSON");
    assert_eq!(value["truncated"], true);
    let messages = value["messages"].as_array().expect("messages array");
    assert_eq!(messages.len(), 1_000);
    assert_eq!(messages[0]["text"], "message-1", "oldest message was dropped");
    assert_eq!(messages[999]["text"], "message-1000", "newest message survived");
}

