//! Phase 3 acceptance (W5): the production server loads the shipped planner
//! prompt template from embedded assets, and missing/invalid assets refuse by
//! name instead of silently running with an empty registry.
//!
//! The production path is exercised through `AppState::new` / `AppState::with_graph`
//! (the binary boot constructor) plus the embedded loader; the injectable
//! `load_from_manifest_text` seam proves the refusal paths without touching the
//! embedded files.

use std::net::SocketAddr;
use std::sync::Arc;

use hiverouter_contract::Refusal;
use hiverouter_core::config::Config;
use hiverouter_core::db::Database;
use hiverouter_core::graph::GraphAuthority;
use hiverouter_core::http::{self, AppState};
use hiverouter_core::prompts::{TemplateRegistry, EMBEDDED_MANIFEST, EMBEDDED_TEMPLATE_FILES};
use hiverouter_core::run_checks;
use serde_json::Value;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::oneshot;

const BODY: &str =
    "---\nname: t1\nmodel:\n  id: deepseek-flash\n---\nsystem:\nHello planner\nuser:\nDo {{ task }}\n";

fn manifest_row(file: &str, spdx: &str) -> String {
    format!(
        r#"[{{"template_id":"t1","file":"{file}","spdx":"{spdx}","source_url":"authored in test","fetched_at":"2026-11-06"}}]"#
    )
}

fn embedded_planner() -> &'static str {
    EMBEDDED_TEMPLATE_FILES
        .iter()
        .find(|(name, _)| *name == "planner.prompty")
        .map(|(_, body)| *body)
        .expect("planner.prompty is embedded at compile time")
}

#[test]
fn embedded_assets_contain_the_shipped_planner_template() {
    assert!(!EMBEDDED_MANIFEST.trim().is_empty(), "manifest embedded");
    assert!(
        EMBEDDED_MANIFEST.contains("\"template_id\": \"planner\"")
            || EMBEDDED_MANIFEST.contains("\"template_id\":\"planner\""),
        "manifest row for planner is embedded"
    );
    assert!(
        embedded_planner().contains("HiveMind planner"),
        "planner system prompt body is embedded"
    );
    assert!(
        embedded_planner().contains("{{ task }}"),
        "planner input placeholder is embedded"
    );
}

#[test]
fn embedded_registry_loads_a_renderable_planner_template() {
    let registry = TemplateRegistry::load_embedded().expect("embedded planner loads");
    assert_eq!(
        registry.template_ids(),
        vec![
            "chatbot".to_owned(),
            "code-reviewer".to_owned(),
            "content-writer".to_owned(),
            "data-analyst".to_owned(),
            "planner".to_owned(),
            "subagent".to_owned(),
            "superagent".to_owned(),
        ]
    );
    let template = registry.get("planner").expect("planner template");
    assert_eq!(template.name, "planner");
    assert!(template.manifest.spdx == "MIT");
    assert!(template.inputs.iter().any(|input| input == "task"));

    let mut inputs = std::collections::BTreeMap::new();
    inputs.insert("task".to_owned(), "draw the hive".to_owned());
    let messages = registry.render("planner", &inputs).expect("render planner");
    assert_eq!(messages.len(), 2);
    assert!(messages[0].content.contains("HiveMind planner"));
    assert!(messages[1].content.contains("draw the hive"));
    assert!(!messages[1].content.contains("{{ task }}"));
}

fn state_parts(temp: &tempfile::TempDir) -> (Arc<Database>, Arc<Config>) {
    let config = Config::resolve("127.0.0.1:0", temp.path().to_str().expect("utf8"), None)
        .expect("test config");
    let database = Database::open(config.db_file()).expect("database");
    database.migrate().expect("migrations");
    (Arc::new(database), Arc::new(config))
}

#[test]
fn production_constructor_loads_the_planner_template() {
    let temp = tempfile::tempdir().expect("temp");
    let (db, config) = state_parts(&temp);
    let state = AppState::new(db, run_checks(), config);
    assert!(state.prompt_assets_loaded(), "no prompt-asset error");
    assert!(state.prompt_assets_error.is_none());
    assert_eq!(
        state.engine.templates.template_ids(),
        vec![
            "chatbot".to_owned(),
            "code-reviewer".to_owned(),
            "content-writer".to_owned(),
            "data-analyst".to_owned(),
            "planner".to_owned(),
            "subagent".to_owned(),
            "superagent".to_owned(),
        ],
        "engine got the loaded planner registry (not an empty default)"
    );
    assert!(
        state
            .checks
            .iter()
            .all(|check| check.name != "prompt_assets"),
        "no failed prompt_assets check on a good embedded load"
    );
}

#[test]
fn boot_constructor_loads_the_planner_template() {
    let temp = tempfile::tempdir().expect("temp");
    let (db, config) = state_parts(&temp);
    let state = AppState::with_graph(db, run_checks(), config, Arc::new(GraphAuthority::new()));
    assert!(state.prompt_assets_loaded());
    assert_eq!(
        state.engine.templates.template_ids(),
        vec![
            "chatbot".to_owned(),
            "code-reviewer".to_owned(),
            "content-writer".to_owned(),
            "data-analyst".to_owned(),
            "planner".to_owned(),
            "subagent".to_owned(),
            "superagent".to_owned(),
        ],
        "binary boot path (with_graph) loads the planner template"
    );
}

#[test]
fn test_constructor_keeps_the_documented_default_seam() {
    // with_services is the documented test-only empty-registry seam; it must not
    // be mistaken for the production path (which is covered above).
    let temp = tempfile::tempdir().expect("temp");
    let (db, config) = state_parts(&temp);
    let state = AppState::with_services(
        db,
        run_checks(),
        config,
        Arc::new(hiverouter_core::bus::EventBus::new()),
        Arc::new(GraphAuthority::new()),
        Arc::new(hiverouter_core::ws::SessionAuth::new(
            "test-token".to_owned(),
            vec!["http://127.0.0.1:9999".to_owned()],
        )),
    );
    assert!(state.engine.templates.template_ids().is_empty());
    assert!(state.prompt_assets_loaded());
}

#[test]
fn invalid_manifest_json_is_a_named_refusal() {
    assert_eq!(
        TemplateRegistry::load_from_manifest_text("{ not a manifest", &[])
            .expect_err("malformed manifest"),
        Refusal::TemplateInvalid
    );
}

#[test]
fn manifest_naming_a_missing_file_refuses_unknown() {
    let manifest = manifest_row("absent.prompty", "MIT");
    assert_eq!(
        TemplateRegistry::load_from_manifest_text(&manifest, &[])
            .expect_err("missing template file"),
        Refusal::TemplateUnknown
    );
}

#[test]
fn bad_licence_refuses_by_name() {
    let manifest = manifest_row("t1.prompty", "GPL-3.0");
    assert_eq!(
        TemplateRegistry::load_from_manifest_text(&manifest, &[("t1.prompty", BODY)])
            .expect_err("licence not on allowlist"),
        Refusal::TemplateLicenseMissing
    );
}

#[test]
fn env_reference_refuses_by_name() {
    let manifest = manifest_row("t1.prompty", "MIT");
    let body = "---\nname: t1\nmodel:\n  id: deepseek-flash\n---\nsystem:\nKey ${env:API_KEY}\n";
    assert_eq!(
        TemplateRegistry::load_from_manifest_text(&manifest, &[("t1.prompty", body)])
            .expect_err("env reference forbidden"),
        Refusal::TemplateRefForbidden
    );
}

#[test]
fn empty_instruction_body_refuses_invalid() {
    let manifest = manifest_row("t1.prompty", "MIT");
    let body = "---\nname: t1\nmodel:\n  id: deepseek-flash\n---\n";
    assert_eq!(
        TemplateRegistry::load_from_manifest_text(&manifest, &[("t1.prompty", body)])
            .expect_err("no role body"),
        Refusal::TemplateInvalid
    );
}

async fn http_get(addr: SocketAddr, path: &str) -> (u16, String) {
    let mut stream = TcpStream::connect(addr).await.expect("connect");
    stream
        .write_all(
            format!("GET {path} HTTP/1.1\r\nHost: 127.0.0.1\r\nConnection: close\r\n\r\n")
                .as_bytes(),
        )
        .await
        .expect("write");
    let mut body = Vec::new();
    stream.read_to_end(&mut body).await.expect("read");
    let response = String::from_utf8_lossy(&body).to_owned();
    let status = response
        .lines()
        .next()
        .and_then(|line| line.split_whitespace().nth(1))
        .and_then(|code| code.parse::<u16>().ok())
        .expect("status line");
    let payload = response
        .split("\r\n\r\n")
        .nth(1)
        .unwrap_or_default()
        .to_owned();
    (status, payload)
}

#[tokio::test]
async fn health_reports_prompt_assets_loaded_on_the_production_path() {
    let temp = tempfile::tempdir().expect("temp");
    let (db, config) = state_parts(&temp);
    let state = Arc::new(AppState::new(db, run_checks(), config));
    let listener = TcpListener::bind("127.0.0.1:0").await.expect("bind");
    let addr = listener.local_addr().expect("addr");
    let (shutdown, shutdown_rx) = oneshot::channel();
    let serve_state = state.clone();
    let handle = tokio::spawn(async move {
        let _ = http::serve(listener, serve_state, async move {
            let _ = shutdown_rx.await;
        })
        .await;
    });

    let (status, body) = http_get(addr, "/health").await;
    assert_eq!(status, 200, "health 200 when required checks pass");
    let payload: Value = serde_json::from_str(&body).expect("health JSON");
    assert_eq!(payload["prompt_assets"]["loaded"], true);
    assert!(payload["prompt_assets"]["error_code"].is_null());
    assert_ne!(payload["status"], "failing");

    let _ = shutdown.send(());
    handle.abort();
}
