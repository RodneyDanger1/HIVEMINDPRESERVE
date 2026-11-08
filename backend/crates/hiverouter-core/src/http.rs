//! Localhost HTTP surface: `GET /health`, `GET /api/session`, `GET /ws`, and an
//! honest 404 until the embedded web build exists.

use std::net::{IpAddr, SocketAddr};
use std::sync::Arc;
use std::time::Instant;

use axum::extract::{ConnectInfo, Path, Query, State};
use axum::http::{header, HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::{delete, get, post};
use axum::{Json, Router};
use hiverouter_contract::Refusal;
use hiverouter_provider::{ClientConfig, DeepSeekClient};
use hiverouter_tools::jail::WorkspaceJail;
use serde::Serialize;
use tokio::net::TcpListener;
use uuid::Uuid;

use crate::bus::EventBus;
use crate::checks::Check;
use crate::config::Config;
use crate::db::Database;
use crate::execution::{ProviderExecutor, RunEngine, UnconfiguredExecutor};
use crate::graph::GraphAuthority;
use crate::history::HistoryReadModel;
use crate::prompts::{load_personas_embedded, Persona, TemplateRegistry};
use crate::provider_executor::ProviderExecutorAdapter;
use crate::snapshot::SnapshotStore;
use crate::tools_bridge::ToolsBridge;
use crate::ws::{refusal_response, SessionAuth, ws_handler};

/// Shared server state.
#[derive(Debug)]
pub struct AppState {
    /// SQLite database (cache/metrics/usage/run rows).
    pub db: Arc<Database>,
    /// Environment checks ran at boot.
    pub checks: Vec<Check>,
    /// Process start time.
    pub started_at: Instant,
    /// Resolved configuration.
    pub config: Arc<Config>,
    /// Server event sequencer + fan-out.
    pub bus: Arc<EventBus>,
    /// The single authoritative graph owner.
    pub graph: Arc<GraphAuthority>,
    /// WebSocket session authentication.
    pub session: Arc<SessionAuth>,
    /// Run lifecycle engine.
    pub engine: Arc<RunEngine>,
    /// History / time-travel read model.
    pub history: Arc<HistoryReadModel>,
    /// Wired tools capabilities (settings, jailed files, approvals, diagnose).
    pub tools: Arc<ToolsBridge>,
    /// Named refusal when the embedded prompt assets failed to load.
    ///
    /// `None` means the planner template registry loaded from the binary's
    /// embedded assets. `Some(refusal)` means the boot path is serving with a
    /// required-and-failed `prompt_assets` check: `/health` reports 503 and the
    /// error code, and planner renders refuse by name â€” never a fake success.
    pub prompt_assets_error: Option<Refusal>,
}

/// Loads the shipped prompt templates embedded in the binary.
///
/// # Errors
/// Returns the named refusal (`E_TEMPLATE_INVALID`, `E_TEMPLATE_UNKNOWN`,
/// `E_TEMPLATE_LICENSE_MISSING`, `E_TEMPLATE_REF_FORBIDDEN`) when the embedded
/// manifest or a template body is missing/malformed.
pub fn load_prompt_assets() -> Result<Arc<TemplateRegistry>, Refusal> {
    TemplateRegistry::load_embedded().map(Arc::new)
}

/// Production prompt-asset load with an explicit failed boot check on error.
///
/// A missing/invalid asset must never degrade into an empty registry that
/// silently answers later with `E_TEMPLATE_UNKNOWN`: the refusal is recorded as
/// a required failed check (so `/health` is 503), logged, and kept on the state.
fn prompt_assets_for(checks: &mut Vec<Check>) -> (Arc<TemplateRegistry>, Option<Refusal>) {
    match load_prompt_assets() {
        Ok(templates) => (templates, None),
        Err(refusal) => {
            tracing::error!(
                code = refusal.code(),
                message = refusal.message(),
                "embedded prompt assets refused at boot; planner renders will refuse by name"
            );
            checks.push(Check {
                name: "prompt_assets",
                required: true,
                ok: false,
                detail: format!(
                    "embedded template assets refused: {} ({})",
                    refusal.message(),
                    refusal.code()
                ),
                code: Some(refusal.code()),
            });
            // The engine still needs a registry object; it is empty, but the
            // failure is explicit (required failed check + health code + log),
            // never silent. The render path refuses with E_TEMPLATE_UNKNOWN.
            (Arc::new(TemplateRegistry::default()), Some(refusal))
        }
    }
}

fn persona_assets_for(
    templates: &TemplateRegistry,
    checks: &mut Vec<Check>,
) -> Vec<Persona> {
    match load_personas_embedded(templates) {
        Ok(personas) => personas,
        Err(refusal) => {
            tracing::error!(
                code = refusal.code(),
                message = refusal.message(),
                "embedded persona assets refused at boot; scoped allowlists stay empty"
            );
            checks.push(Check {
                name: "persona_assets",
                required: true,
                ok: false,
                detail: format!(
                    "embedded persona assets refused: {} ({})",
                    refusal.message(),
                    refusal.code()
                ),
                code: Some(refusal.code()),
            });
            Vec::new()
        }
    }
}

fn merge_persisted_personas(
    db: &Arc<Database>,
    templates: &TemplateRegistry,
    mut personas: Vec<Persona>,
) -> Vec<Persona> {
    let store = crate::store::RunStore::new(db.clone());
    let persisted = match store.agent_profiles() {
        Ok(profiles) => profiles,
        Err(error) => {
            tracing::warn!(
                code = error.code(),
                "persisted agent profiles refused; embedded profiles only"
            );
            return personas;
        }
    };
    for (id, value, source) in persisted {
        let persona: Persona = match serde_json::from_value(value) {
            Ok(persona) => persona,
            Err(_) => {
                tracing::warn!(profile_id = %id, %source, "persisted profile is invalid JSON");
                continue;
            }
        };
        if persona.id != id {
            tracing::warn!(
                profile_id = %id,
                profile_inner_id = %persona.id,
                "persisted profile id does not match its row id"
            );
            continue;
        }
        if let Err(refusal) = persona.validate(templates) {
            tracing::warn!(
                profile_id = %id,
                code = refusal.code(),
                "persisted profile failed validation; embedded profile retained"
            );
            continue;
        }
        if let Some(existing) = personas.iter_mut().find(|existing| existing.id == id) {
            *existing = persona;
        } else {
            personas.push(persona);
        }
    }
    personas
}

fn tools_migration_check(db: &Database, checks: &mut Vec<Check>) {
    if let Err(error) = ToolsBridge::migrate(db) {
        tracing::error!(
            code = error.code(),
            "tools approval-table migration failed; required health check fails"
        );
        checks.push(Check {
            name: "tools_migration",
            required: true,
            ok: false,
            detail: format!(
                "tools approval-table migration failed: {} ({})",
                error.message(),
                error.code()
            ),
            code: Some(error.code()),
        });
    }
}

fn build_tools(db: &Arc<Database>, config: &Config) -> Arc<ToolsBridge> {
    let now_unix_secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or(0);
    match ToolsBridge::expire_pending_approvals(db, now_unix_secs) {
        Ok(0) => {}
        Ok(expired) => tracing::warn!(
            expired,
            "consumed restart-orphaned pending approvals at boot; no waiter survives restart"
        ),
        Err(error) => tracing::error!(
            code = error.code(),
            "approval boot reaper failed; restart-orphaned rows may remain pending (no in-process waiter survives)"
        ),
    }
    Arc::new(
        ToolsBridge::new(&config.workspace_root, db.clone())
            .with_side_effects_enabled(config.side_effects_enabled),
    )
}

/// Builds the production executor. Missing key or disabled provider falls back
/// to the honest `E_KEY_MISSING` executor; it never fabricates replies.
fn build_executor(config: &Config) -> Arc<dyn ProviderExecutor> {
    if !config.provider.enabled {
        tracing::info!("provider disabled by configuration; live prompts refuse E_KEY_MISSING");
        return Arc::new(UnconfiguredExecutor);
    }
    match ProviderExecutorAdapter::from_env(config.provider.clone()) {
        Ok(executor) => Arc::new(executor),
        Err(error) => {
            tracing::warn!(
                code = error.code(),
                "provider key/client unavailable; live prompts refuse by name"
            );
            Arc::new(UnconfiguredExecutor)
        }
    }
}


/// Builds the shared engine, wiring the production workspace jail when the
/// configured root is usable. A missing/invalid root leaves the jail absent so
/// read-only tool calls refuse E_FILE_TREE_UNAVAILABLE instead of touching disk.
fn build_engine(
    db: Arc<Database>,
    bus: Arc<EventBus>,
    graph: Arc<GraphAuthority>,
    config: &Config,
    templates: Arc<TemplateRegistry>,
    personas: Vec<Persona>,
    snapshots: Arc<SnapshotStore>,
    tools: Arc<ToolsBridge>,
    wire_workspace_jail: bool,
) -> Arc<RunEngine> {
    let mut engine = RunEngine::new(db, bus, graph, build_executor(config), templates)
        .with_personas(personas)
        .with_snapshot_store(snapshots)
        .with_tools(tools);
    if wire_workspace_jail {
        match WorkspaceJail::open(&config.workspace_root) {
            Ok(jail) => engine = engine.with_tool_workspace(jail),
            Err(error) => {
                tracing::warn!(
                    workspace_root = %config.workspace_root.display(),
                    code = error.code(),
                    "workspace jail unavailable; read-only model tool calls will refuse E_FILE_TREE_UNAVAILABLE"
                );
            }
        }
    }
    Arc::new(engine)
}

/// Publishes the boot-recovery settlement frames once the bus exists.
///
/// `main.rs` performs `recover_on_boot` before `AppState` is built (the bus does
/// not exist yet), so the settlement `NODE_STATE` frames are published here as a
/// catch-up from the persisted recovery outcome. It mutates nothing and is a
/// no-op on a clean boot (no interrupted run, no recovered task).
fn publish_recovery_catch_up(db: &Arc<Database>, graph: &GraphAuthority, bus: &EventBus) {
    match crate::recovery::publish_recovered_node_states(db.clone(), graph, bus) {
        Ok(_) => {}
        Err(error) => tracing::warn!(
            code = error.code(),
            "boot catch-up could not publish recovered node states"
        ),
    }
}

impl AppState {
    /// Builds state for a running server with a fresh event bus, graph and
    /// session token derived from the configuration.
    ///
    /// Loads the embedded prompt templates (production path). A load failure is
    /// recorded as a required failed `prompt_assets` check and the named refusal
    /// on the state; it never degrades silently.
    #[must_use]
    pub fn new(db: Arc<Database>, mut checks: Vec<Check>, config: Arc<Config>) -> Self {
        let session = Arc::new(SessionAuth::new(
            Uuid::now_v7().to_string(),
            config.allowed_origins.clone(),
        ));
        let bus = Arc::new(EventBus::new());
        let graph = Arc::new(GraphAuthority::new());
        let snapshots = Arc::new(SnapshotStore::new(&config.data_dir, db.clone()));
        let (templates, prompt_assets_error) = prompt_assets_for(&mut checks);
        let personas = merge_persisted_personas(
            &db,
            &templates,
            persona_assets_for(&templates, &mut checks),
        );
        tools_migration_check(&db, &mut checks);
        let tools = build_tools(&db, &config);
        let engine = build_engine(db.clone(), bus.clone(), graph.clone(), &config, templates, personas, snapshots.clone(), tools.clone(), true);
        let history = Arc::new(HistoryReadModel::new(db.clone(), (*snapshots).clone()));
        publish_recovery_catch_up(&db, &graph, &bus);
        Self {
            db,
            checks,
            started_at: Instant::now(),
            config,
            bus,
            graph,
            session,
            engine,
            history,
            tools,
            prompt_assets_error,
        }
    }

    /// Builds state with an explicitly recovered graph (boot path).
    ///
    /// This is the binary's construction path: it loads the embedded prompt
    /// templates and records a named refusal (`prompt_assets_error` + required
    /// failed check) if they are missing/invalid.
    #[must_use]
    pub fn with_graph(
        db: Arc<Database>,
        mut checks: Vec<Check>,
        config: Arc<Config>,
        graph: Arc<GraphAuthority>,
    ) -> Self {
        let session = Arc::new(SessionAuth::new(
            Uuid::now_v7().to_string(),
            config.allowed_origins.clone(),
        ));
        let bus = Arc::new(EventBus::new());
        let snapshots = Arc::new(SnapshotStore::new(&config.data_dir, db.clone()));
        let (templates, prompt_assets_error) = prompt_assets_for(&mut checks);
        let personas = merge_persisted_personas(
            &db,
            &templates,
            persona_assets_for(&templates, &mut checks),
        );
        tools_migration_check(&db, &mut checks);
        let tools = build_tools(&db, &config);
        let engine = build_engine(db.clone(), bus.clone(), graph.clone(), &config, templates, personas, snapshots.clone(), tools.clone(), true);
        let history = Arc::new(HistoryReadModel::new(db.clone(), (*snapshots).clone()));
        publish_recovery_catch_up(&db, &graph, &bus);
        Self {
            db,
            checks,
            started_at: Instant::now(),
            config,
            bus,
            graph,
            session,
            engine,
            history,
            tools,
            prompt_assets_error,
        }
    }

    /// Builds state with explicitly supplied services (integration tests).
    ///
    /// Test-only seam: it keeps the documented empty `TemplateRegistry::default()`
    /// so unit/integration tests can build an engine without assets. Production
    /// uses [`AppState::new`] / [`AppState::with_graph`], which load the embedded
    /// planner template and record a named refusal on failure.
    #[must_use]
    pub fn with_services(
        db: Arc<Database>,
        mut checks: Vec<Check>,
        config: Arc<Config>,
        bus: Arc<EventBus>,
        graph: Arc<GraphAuthority>,
        session: Arc<SessionAuth>,
    ) -> Self {
        let snapshots = Arc::new(SnapshotStore::new(&config.data_dir, db.clone()));
        tools_migration_check(&db, &mut checks);
        let tools = build_tools(&db, &config);
        let engine = build_engine(db.clone(), bus.clone(), graph.clone(), &config, Arc::new(TemplateRegistry::default()), Vec::new(), snapshots.clone(), tools.clone(), false);
        let history = Arc::new(HistoryReadModel::new(db.clone(), (*snapshots).clone()));
        Self {
            db,
            checks,
            started_at: Instant::now(),
            config,
            bus,
            graph,
            session,
            engine,
            history,
            tools,
            prompt_assets_error: None,
        }
    }

    /// Whether the production prompt templates loaded successfully.
    #[must_use]
    pub fn prompt_assets_loaded(&self) -> bool {
        self.prompt_assets_error.is_none()
    }
}

/// Health payload.
#[derive(Debug, Serialize)]
pub struct HealthResponse {
    /// `ok` (all checks passed), `degraded` (optional check failed) or
    /// `failing` (required check failed).
    pub status: &'static str,
    /// Crate version.
    pub version: &'static str,
    /// Contract schema id.
    pub schema: &'static str,
    /// Process uptime in seconds.
    pub uptime_secs: u64,
    /// Environment checks.
    pub checks: Vec<Check>,
    /// Database health.
    pub database: DatabaseHealth,
    /// Transport counters.
    pub transport: TransportHealth,
    /// Embedded prompt-asset load status (planner template registry).
    pub prompt_assets: PromptAssetsHealth,
}

/// Database portion of the health payload.
#[derive(Debug, Serialize)]
pub struct DatabaseHealth {
    /// Database file path (`:memory:` for in-memory databases).
    pub path: String,
    /// Applied schema version, when readable.
    pub schema_version: Option<i64>,
    /// Named refusal code when the database is not readable.
    pub error_code: Option<&'static str>,
}

/// Transport portion of the health payload.
#[derive(Debug, Serialize)]
pub struct TransportHealth {
    /// Current server event sequence.
    pub last_seq: u64,
    /// Graph nodes in memory.
    pub graph_nodes: Option<usize>,
    /// Origin allowlist size (the values themselves stay out of the payload).
    pub allowed_origins: usize,
}

/// Prompt-asset portion of the health payload (production boot proof).
#[derive(Debug, Serialize)]
pub struct PromptAssetsHealth {
    /// True when the embedded planner template registry loaded at boot.
    pub loaded: bool,
    /// Named refusal code when the embedded assets were missing or invalid.
    pub error_code: Option<&'static str>,
}

/// Builds the server router.
#[must_use]
pub fn build_router(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/health", get(health))
        .route("/diagnose", get(diagnose))
        .route("/api/session", get(api_session))
        .route("/api/history/runs", get(history_runs))
        .route("/api/history/runs/{run_id}/revisions", get(history_revisions))
        .route("/api/history/runs/{run_id}/state", get(history_state))
        .route("/api/history/runs/{run_id}/messages", get(history_messages))
        .route("/api/agents/runs/{run_id}/messages", get(agent_messages))
        .route("/api/agents/profiles", get(agent_profiles))
        .route("/api/agents/profiles/import", post(import_agent_profile))
        .route("/api/agents/profiles/{profile_id}", delete(delete_agent_profile))
        .route("/api/agents/memory", get(agent_memory_list))
        .route("/api/agents/memory/{agent_key}", delete(clear_agent_memory))
        .route("/api/usage/runs/{run_id}", get(run_usage))
        .route("/api/projects", get(projects))
        .route("/api/provider/models", get(provider_models))
        .route("/api/provider/balance", get(provider_balance))
        .route("/ws", get(ws_handler))
        .fallback(crate::static_ui::static_ui)
        .with_state(state)
}

/// `GET /diagnose` â€” tools-owned derived diagnostics (no secrets).
async fn diagnose(State(state): State<Arc<AppState>>) -> Response {
    match state.tools.diagnostics() {
        Ok(mut value) => {
            if let Some(object) = value.as_object_mut() {
                object.insert(
                    "side_effects_enabled".to_owned(),
                    serde_json::Value::Bool(state.config.side_effects_enabled),
                );
            }
            (StatusCode::OK, Json(value)).into_response()
        }
        Err(refusal) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({
                "code": refusal.code(),
                "message": refusal.message(),
                "how_to_fix": refusal.how_to_fix(),
                "retryable": refusal.retryable(),
            })),
        )
            .into_response(),
    }
}

async fn history_runs(State(state): State<Arc<AppState>>) -> Response {
    match state.history.runs(200) {
        Ok(runs) => (
            StatusCode::OK,
            Json(serde_json::json!({
                "runs": runs.iter().map(|run| serde_json::json!({
                    "run_id": run.id.to_string(),
                    "project_id": run.project_id,
                    "status": run.status.as_str(),
                    "error_code": run.error_code,
                    "created_at": run.created_at.as_str(),
                })).collect::<Vec<_>>(),
            })),
        )
            .into_response(),
        Err(refusal) => refusal_response(StatusCode::SERVICE_UNAVAILABLE, &refusal),
    }
}

async fn history_revisions(
    State(state): State<Arc<AppState>>,
    Path(run_id): Path<String>,
) -> Response {
    let Ok(run_id) = run_id.parse::<hiverouter_contract::RunId>() else {
        return refusal_response(StatusCode::NOT_FOUND, &hiverouter_contract::Refusal::RunNotFound);
    };
    match state.history.time_travel(run_id) {
        Ok(view) => (
            StatusCode::OK,
            Json(serde_json::json!({
                "run_id": view.run_id.to_string(),
                "latest_revision": view.latest_revision,
                "revision_count": view.revision_count(),
                "revisions": view.revisions.iter().map(|meta| serde_json::json!({
                    "revision": meta.revision,
                    "created_at": meta.created_at.as_str(),
                    "run_status": meta.run_status.map(|state| state.as_str()),
                })).collect::<Vec<_>>(),
            })),
        )
            .into_response(),
        Err(refusal) => refusal_response(StatusCode::NOT_FOUND, &refusal),
    }
}

async fn history_state(
    State(state): State<Arc<AppState>>,
    Path(run_id): Path<String>,
    Query(query): Query<std::collections::HashMap<String, String>>,
) -> Response {
    let Ok(run_id) = run_id.parse::<hiverouter_contract::RunId>() else {
        return refusal_response(StatusCode::NOT_FOUND, &hiverouter_contract::Refusal::RunNotFound);
    };
    let Some(revision) = query.get("revision").and_then(|value| value.parse::<u64>().ok()) else {
        return refusal_response(StatusCode::BAD_REQUEST, &hiverouter_contract::Refusal::SchemaInvalid);
    };
    match state.history.state_at(run_id, revision) {
        Ok(snapshot) => (
            StatusCode::OK,
            Json(serde_json::json!({
                "run_id": snapshot.run_id.to_string(),
                "revision": snapshot.revision,
                "schema_version": snapshot.schema_version,
                "run_status": snapshot.run_status.map(|state| state.as_str()),
                "graph": snapshot.graph,
            })),
        )
            .into_response(),
        Err(refusal) if refusal == hiverouter_contract::Refusal::HistoryGap => {
            refusal_response(StatusCode::NOT_FOUND, &refusal)
        }
        Err(refusal) => refusal_response(StatusCode::CONFLICT, &refusal),
    }
}

async fn history_messages(
    State(state): State<Arc<AppState>>,
    Path(run_id): Path<String>,
) -> Response {
    let Ok(run_id) = run_id.parse::<hiverouter_contract::RunId>() else {
        return refusal_response(StatusCode::NOT_FOUND, &hiverouter_contract::Refusal::RunNotFound);
    };
    if let Err(refusal) = state.engine.store.get_run(run_id) {
        return refusal_response(StatusCode::NOT_FOUND, &refusal);
    }
    const MAX_MESSAGES: usize = 1_000;
    match state.engine.store.messages(run_id) {
        Ok(messages) => {
            let truncated = messages.len() > MAX_MESSAGES;
            let visible = messages.into_iter().take(MAX_MESSAGES);
            let rows: Vec<serde_json::Value> = visible
                .map(|message| {
                    serde_json::json!({
                        "id": message.id.to_string(),
                        "run_id": message.run_id.to_string(),
                        "role": message.role.as_str(),
                        "text": message.text,
                        "created_at": message.created_at.as_str(),
                    })
                })
                .collect();
            (
                StatusCode::OK,
                Json(serde_json::json!({
                    "run_id": run_id.to_string(),
                    "messages": rows,
                    "truncated": truncated,
                })),
            )
                .into_response()
        }
        Err(refusal) => refusal_response(StatusCode::SERVICE_UNAVAILABLE, &refusal),
    }
}

async fn agent_messages(
    State(state): State<Arc<AppState>>,
    Path(run_id): Path<String>,
) -> Response {
    let Ok(run_id) = run_id.parse::<hiverouter_contract::RunId>() else {
        return refusal_response(StatusCode::NOT_FOUND, &hiverouter_contract::Refusal::RunNotFound);
    };
    if let Err(refusal) = state.engine.store.get_run(run_id) {
        return refusal_response(StatusCode::NOT_FOUND, &refusal);
    }
    match state.engine.store.agent_messages_for_run(run_id) {
        Ok(messages) => (
            StatusCode::OK,
            Json(serde_json::json!({
                "run_id": run_id.to_string(),
                "messages": messages.into_iter().map(|message| serde_json::json!({
                    "id": message.id,
                    "from_task_id": message.from_task_id,
                    "to_task_id": message.to_task_id,
                    "kind": message.kind,
                    "text": message.text,
                    "created_at": message.created_at,
                })).collect::<Vec<_>>(),
            })),
        )
            .into_response(),
        Err(refusal) => refusal_response(StatusCode::SERVICE_UNAVAILABLE, &refusal),
    }
}

async fn import_agent_profile(
    State(state): State<Arc<AppState>>,
    Json(body): Json<serde_json::Value>,
) -> Response {
    let profile_value = body.get("profile").cloned().unwrap_or(body);
    let persona: Persona = match serde_json::from_value(profile_value.clone()) {
        Ok(persona) => persona,
        Err(_) => {
            return refusal_response(StatusCode::BAD_REQUEST, &Refusal::SchemaInvalid);
        }
    };
    if let Err(refusal) = persona.validate(&state.engine.templates) {
        return refusal_response(StatusCode::BAD_REQUEST, &refusal);
    }
    if let Err(refusal) = state
        .engine
        .store
        .upsert_agent_profile(&persona.id, &profile_value, "api")
    {
        return refusal_response(StatusCode::SERVICE_UNAVAILABLE, &refusal);
    }
    state.engine.upsert_persona(persona.clone());
    (
        StatusCode::OK,
        Json(serde_json::json!({
            "profile": {
                "id": persona.id,
                "display_name": persona.display_name,
                "tier": persona.tier,
                "template_id": persona.template_id,
                "allowed_tool_ids": persona.allowed_tool_ids,
                "output_schema_id": persona.output_schema_id,
                "refusal_policy": persona.refusal_policy,
                "max_children": persona.max_children,
                "max_depth": persona.max_depth,
            },
            "source": "api",
        })),
    )
        .into_response()
}

async fn delete_agent_profile(
    State(state): State<Arc<AppState>>,
    Path(profile_id): Path<String>,
) -> Response {
    let id = profile_id.trim();
    if id.is_empty() || id.chars().count() > 128 {
        return refusal_response(StatusCode::BAD_REQUEST, &Refusal::SchemaInvalid);
    }
    let removed = match state.engine.store.delete_agent_profile(id) {
        Ok(removed) => removed,
        Err(refusal) => return refusal_response(StatusCode::SERVICE_UNAVAILABLE, &refusal),
    };
    if !removed {
        return (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({
                "code": "E_PROFILE_NOT_FOUND",
                "message": "no imported profile with that id",
            })),
        )
            .into_response();
    }
    state.engine.remove_persona(id);
    // Deleting an override restores the embedded profile without a restart.
    if let Ok(builtins) = load_personas_embedded(&state.engine.templates) {
        if let Some(builtin) = builtins.into_iter().find(|persona| persona.id == id) {
            state.engine.upsert_persona(builtin);
        }
    }
    (
        StatusCode::OK,
        Json(serde_json::json!({ "id": id, "removed": true })),
    )
        .into_response()
}

async fn agent_profiles(State(state): State<Arc<AppState>>) -> Response {
    let profiles: Vec<serde_json::Value> = state
        .engine
        .personas()
        .into_iter()
        .map(|persona| {
            serde_json::json!({
                "id": persona.id,
                "display_name": persona.display_name,
                "tier": persona.tier,
                "template_id": persona.template_id,
                "allowed_tool_ids": persona.allowed_tool_ids,
                "output_schema_id": persona.output_schema_id,
                "refusal_policy": persona.refusal_policy,
                "max_children": persona.max_children,
                "max_depth": persona.max_depth,
            })
        })
        .collect();
    (StatusCode::OK, Json(serde_json::json!({ "profiles": profiles }))).into_response()
}

async fn agent_memory_list(State(state): State<Arc<AppState>>) -> Response {
    let mut profiles: Vec<serde_json::Value> = Vec::new();
    for persona in state.engine.personas() {
        match state.engine.store.agent_memory(&persona.id, 8) {
            Ok(memories) => profiles.push(serde_json::json!({
                "id": persona.id,
                "display_name": persona.display_name,
                "memories": memories.into_iter().map(|(key, value)| serde_json::json!({
                    "key": key,
                    "value": value,
                })).collect::<Vec<_>>(),
            })),
            Err(refusal) => {
                return refusal_response(StatusCode::SERVICE_UNAVAILABLE, &refusal);
            }
        }
    }
    (StatusCode::OK, Json(serde_json::json!({ "profiles": profiles }))).into_response()
}

async fn clear_agent_memory(
    State(state): State<Arc<AppState>>,
    Path(agent_key): Path<String>,
) -> Response {
    let key = agent_key.trim();
    if key.is_empty() || key.chars().count() > 128 {
        return refusal_response(StatusCode::BAD_REQUEST, &hiverouter_contract::Refusal::SchemaInvalid);
    }
    match state.engine.store.clear_agent_memory(key) {
        Ok(removed) => (
            StatusCode::OK,
            Json(serde_json::json!({ "agent_key": key, "removed": removed })),
        )
            .into_response(),
        Err(refusal) => refusal_response(StatusCode::SERVICE_UNAVAILABLE, &refusal),
    }
}

async fn run_usage(
    State(state): State<Arc<AppState>>,
    Path(run_id): Path<String>,
) -> Response {
    let Ok(run_id) = run_id.parse::<hiverouter_contract::RunId>() else {
        return refusal_response(StatusCode::NOT_FOUND, &hiverouter_contract::Refusal::RunNotFound);
    };
    if let Err(refusal) = state.engine.store.get_run(run_id) {
        return refusal_response(StatusCode::NOT_FOUND, &refusal);
    }
    match state.engine.store.run_usage(run_id) {
        Ok(Some(usage)) => (
            StatusCode::OK,
            Json(serde_json::json!({
                "run_id": run_id.to_string(),
                "usage": {
                    "prompt_tokens": usage.prompt_tokens,
                    "completion_tokens": usage.completion_tokens,
                    "total_tokens": usage.total_tokens,
                    "prompt_cache_hit_tokens": usage.prompt_cache_hit_tokens,
                    "prompt_cache_miss_tokens": usage.prompt_cache_miss_tokens,
                    "reasoning_tokens": usage.reasoning_tokens,
                },
            })),
        )
            .into_response(),
        Ok(None) => (
            StatusCode::OK,
            Json(serde_json::json!({ "run_id": run_id.to_string(), "usage": null })),
        )
            .into_response(),
        Err(refusal) => refusal_response(StatusCode::SERVICE_UNAVAILABLE, &refusal),
    }
}

/// `GET /api/provider/models` — documented DeepSeek model catalogue proxy.
///
/// The API key never reaches the browser: the local server reads
/// `DEEPSEEK_API_KEY`, calls the provider, and returns only public metadata.
async fn provider_models(State(state): State<Arc<AppState>>, headers: HeaderMap) -> Response {
    if !provider_origin_allowed(&state, &headers) {
        return refusal_response(StatusCode::FORBIDDEN, &Refusal::OriginRejected);
    }
    let client = match DeepSeekClient::from_env(ClientConfig::default()) {
        Ok(client) => client,
        Err(error) => return provider_error_response(&error),
    };
    match client.list_models().await {
        Ok(models) => (
            StatusCode::OK,
            [(header::CACHE_CONTROL, "no-store")],
            Json(serde_json::json!({
                "object": models.object,
                "models": models.data,
            })),
        )
            .into_response(),
        Err(error) => provider_error_response(&error),
    }
}

/// `GET /api/provider/balance` — documented DeepSeek account balance proxy.
async fn provider_balance(State(state): State<Arc<AppState>>, headers: HeaderMap) -> Response {
    if !provider_origin_allowed(&state, &headers) {
        return refusal_response(StatusCode::FORBIDDEN, &Refusal::OriginRejected);
    }
    let client = match DeepSeekClient::from_env(ClientConfig::default()) {
        Ok(client) => client,
        Err(error) => return provider_error_response(&error),
    };
    match client.balance().await {
        Ok(balance) => (
            StatusCode::OK,
            [(header::CACHE_CONTROL, "no-store")],
            Json(serde_json::json!({
                "is_available": balance.is_available,
                "balances": balance.balance_infos,
            })),
        )
            .into_response(),
        Err(error) => provider_error_response(&error),
    }
}

fn provider_origin_allowed(state: &AppState, headers: &HeaderMap) -> bool {
    match headers.get(header::ORIGIN).and_then(|value| value.to_str().ok()) {
        None => true,
        Some(origin) => state
            .config
            .allowed_origins
            .iter()
            .any(|allowed| allowed.eq_ignore_ascii_case(origin)),
    }
}

fn provider_error_response(error: &hiverouter_provider::ProviderError) -> Response {
    let status = match error.code() {
        "E_KEY_MISSING" => StatusCode::SERVICE_UNAVAILABLE,
        "E_PROVIDER_AUTH" => StatusCode::UNAUTHORIZED,
        "E_PROVIDER_RATE_LIMIT" => StatusCode::TOO_MANY_REQUESTS,
        _ => StatusCode::BAD_GATEWAY,
    };
    refusal_response(status, &error.refusal())
}

async fn projects(State(state): State<Arc<AppState>>) -> Response {
    match state.engine.store.projects() {
        Ok(projects) => (
            StatusCode::OK,
            Json(serde_json::json!({
                "projects": projects.into_iter().map(|(id, name, created_at)| serde_json::json!({
                    "id": id,
                    "name": name,
                    "created_at": created_at,
                })).collect::<Vec<_>>(),
            })),
        )
            .into_response(),
        Err(refusal) => refusal_response(StatusCode::SERVICE_UNAVAILABLE, &refusal),
    }
}

async fn health(State(state): State<Arc<AppState>>) -> Response {
    let version = state.db.schema_version();
    let database = match &version {
        Ok(schema_version) => DatabaseHealth {
            path: state.db.path().display().to_string(),
            schema_version: Some(*schema_version),
            error_code: None,
        },
        Err(error) => DatabaseHealth {
            path: state.db.path().display().to_string(),
            schema_version: None,
            error_code: Some(error.refusal().code()),
        },
    };
    let required_ok = state
        .checks
        .iter()
        .filter(|check| check.required)
        .all(|check| check.ok)
        && version.is_ok();
    let all_ok = state.checks.iter().all(|check| check.ok) && version.is_ok();
    let status = if required_ok && all_ok {
        "ok"
    } else if required_ok {
        "degraded"
    } else {
        "failing"
    };
    let payload = HealthResponse {
        status,
        version: env!("CARGO_PKG_VERSION"),
        schema: hiverouter_contract::SCHEMA_VERSION,
        uptime_secs: state.started_at.elapsed().as_secs(),
        checks: state.checks.clone(),
        database,
        transport: TransportHealth {
            last_seq: state.bus.current_seq(),
            graph_nodes: state.graph.node_count().ok(),
            allowed_origins: state.session.allowed_origins().len(),
        },
        prompt_assets: PromptAssetsHealth {
            loaded: state.prompt_assets_loaded(),
            error_code: state.prompt_assets_error.map(|refusal| refusal.code()),
        },
    };
    let status_code = if required_ok {
        StatusCode::OK
    } else {
        StatusCode::SERVICE_UNAVAILABLE
    };
    (status_code, Json(payload)).into_response()
}

async fn api_session(
    State(state): State<Arc<AppState>>,
    ConnectInfo(peer): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
) -> Response {
    let origin = headers
        .get(header::ORIGIN)
        .and_then(|value| value.to_str().ok());
    let allowed = match origin {
        Some(origin) => state.session.is_origin_allowed(Some(origin)),
        // Same-origin fetches normally carry no Origin header; only a loopback
        // peer may use that path.
        None => peer.ip().is_loopback(),
    };
    if !allowed {
        return refusal_response(StatusCode::FORBIDDEN, &hiverouter_contract::Refusal::OriginRejected);
    }
    (
        StatusCode::OK,
        Json(serde_json::json!({
            "token": state.session.token(),
            "schema": hiverouter_contract::SCHEMA_VERSION,
            "last_seq": state.bus.current_seq(),
            "ws": "/ws",
        })),
    )
        .into_response()
}

/// Serves the router on an already-bound listener with graceful shutdown.
///
/// # Errors
/// Returns the listener error when `axum::serve` fails.
pub async fn serve(
    listener: TcpListener,
    state: Arc<AppState>,
    shutdown: impl std::future::Future<Output = ()> + Send + 'static,
) -> std::io::Result<()> {
    axum::serve(
        listener,
        build_router(state).into_make_service_with_connect_info::<SocketAddr>(),
    )
    .with_graceful_shutdown(shutdown)
    .await
}

/// Best-effort loopback check used by handlers that receive a peer address.
#[must_use]
pub fn is_loopback(ip: IpAddr) -> bool {
    ip.is_loopback()
}

#[cfg(test)]
mod tests {
    use super::*;
    use hiverouter_contract::{AgentState, TaskId};

    use crate::bus::Replay;
    use crate::entities::TaskRecord;

    #[test]
    fn boot_state_publishes_the_recovered_node_states() {
        let temp = tempfile::tempdir().expect("temp dir");
        let config = Arc::new(
            Config::resolve("127.0.0.1:0", temp.path().to_str().expect("utf8"), None)
                .expect("config"),
        );
        let db = Arc::new(Database::open(config.db_file()).expect("database"));
        db.migrate().expect("migrations");
        let store = crate::store::RunStore::new(db.clone());
        let run = store
            .create_run("project-1", "crashed while waiting", AgentState::Interrupted)
            .expect("run");
        let task = TaskRecord {
            id: TaskId::new(),
            run_id: run.id,
            parent_task_id: None,
            role: "worker".to_owned(),
            depth: 1,
            spawn_seq: 1,
            state: AgentState::Interrupted,
            rev: 0,
        };
        store.insert_task(&task).expect("task");

        // The production boot path: recovery ran first (main.rs), then the state
        // is built; the catch-up must publish the persisted recovery outcome.
        let state = AppState::new(db, crate::checks::run_checks(), config);
        let events = match state.bus.replay_after(0) {
            Replay::Events(events) => events,
            other => panic!("expected one boot catch-up frame, got {other:?}"),
        };
        assert_eq!(events.len(), 1, "one frame for the one recovered task");
        assert_eq!(events[0].envelope.name, "NODE_STATE");
        assert_eq!(events[0].envelope.payload["task_id"], task.id.to_string());
        assert_eq!(events[0].envelope.payload["state"], "interrupted");
        assert_eq!(events[0].envelope.run_id, Some(run.id));
    }
}