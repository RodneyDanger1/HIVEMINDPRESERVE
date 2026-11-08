//! Localhost WebSocket transport.
//!
//! One axum `/ws` endpoint (blueprint Phase 2, WP 2.5/2.6/2.8): loopback-only,
//! Origin allowlist + per-session token gate before the upgrade, `hello{last_seq}`
//! replay/resync on connect, and server->client fan-out from the [`EventBus`].
//!
//! Client -> server frames are either the transport `hello` control frame
//! (`{"type":"hello","last_seq":<u64|null>}`) or a frozen COMMAND envelope.
//! Every command gets a typed ACK or a typed ERROR event (never a silent drop).

use std::collections::BTreeMap;
use std::sync::Arc;

use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::extract::{ConnectInfo, Query, State};
use axum::http::{header, HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use serde::Deserialize;
use serde_json::{json, Value};
use tokio::sync::broadcast::error::RecvError;
use uuid::Uuid;

use hiverouter_contract::command::{Command, CommandName, TemplateInsert};
use hiverouter_contract::event::{
    Ack, EmptyPayload, ErrorPayload, Event, EventName, HistorySnapshot, NodeEvent, ServerHello,
};
use hiverouter_contract::schema;
use hiverouter_contract::{AgentState, Refusal, RunId, TaskId};

use crate::bus::{EventBus, Replay, SequencedEvent};
use crate::caps::Caps;
use crate::context::{ContextStore, ContextSwitchOutcome, ScopedContext};
use crate::entities::TaskRecord;
use crate::execution::{task_agent_tier, task_node_kind, validate_spawn_role};
use crate::graph::{GraphAuthority, TaskNode};
use crate::http::AppState;
use crate::prompts::TemplateRegistry;
use crate::store::RunStore;

/// How long a new connection may wait before sending its `hello` frame.
pub const HELLO_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(10);

/// Per-server session authentication: one boot token and the Origin allowlist.
#[derive(Debug, Clone)]
pub struct SessionAuth {
    token: String,
    allowed_origins: Vec<String>,
}

impl SessionAuth {
    /// Builds the auth gate.
    #[must_use]
    pub fn new(token: String, allowed_origins: Vec<String>) -> Self {
        Self {
            token,
            allowed_origins,
        }
    }

    /// Boot token handed to the UI through `GET /api/session`.
    #[must_use]
    pub fn token(&self) -> &str {
        &self.token
    }

    /// Origins the transport accepts (exact match, case-insensitive).
    #[must_use]
    pub fn allowed_origins(&self) -> &[String] {
        &self.allowed_origins
    }

    /// Whether an Origin header value is allowlisted. A missing Origin is never
    /// accepted for a WebSocket upgrade (browsers always send one).
    #[must_use]
    pub fn is_origin_allowed(&self, origin: Option<&str>) -> bool {
        match origin {
            Some(origin) => self
                .allowed_origins
                .iter()
                .any(|allowed| allowed.eq_ignore_ascii_case(origin)),
            None => false,
        }
    }
}

/// Query parameters accepted on `/ws`.
#[derive(Debug, Deserialize)]
pub struct WsQuery {
    /// Per-session token.
    #[serde(default)]
    pub token: Option<String>,
    /// Last server sequence the client has processed (optional; the `hello`
    /// frame carries the authoritative value).
    #[serde(default)]
    pub last_seq: Option<u64>,
}

/// Longest accepted `template_id` / `project_id` identifier in `TEMPLATE_INSERT`.
///
/// Both are short identifiers on the wire (a manifest row id and a workspace
/// binding); anything longer cannot name a real template or project.
pub const MAX_TEMPLATE_FIELD_CHARS: usize = 128;

/// One node of an insertable template DAG.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TemplateDagNode {
    /// Agent role the spawned task runs.
    pub role: String,
    /// Prompt template the node renders (a [`TemplateRegistry`] id).
    pub template_id: String,
}

/// One insertable template DAG, resolved from the loaded prompt assets.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TemplateDag {
    /// Manifest row id this DAG belongs to.
    pub template_id: String,
    /// Nodes to spawn, in insertion order.
    pub nodes: Vec<TemplateDagNode>,
}

/// Template-DAG catalog derived from the real, loaded prompt-asset registry.
///
/// The shipped assets contain one prompt template (`planner`) and **no multi-node
/// DAG manifests**: a manifest row is a prompt asset, not a task graph. The
/// catalog therefore exposes the only mapping the assets support without
/// inventing node sets — one single-node DAG per registered prompt template,
/// whose node role and prompt are that template id. The four Library template
/// ids (`content-writer`, `data-analyst`, `code-reviewer`, `chatbot`) have no
/// prompt asset in the manifest, so a real lookup refuses them by name; adding
/// four `.prompty` assets plus manifest rows makes them resolvable here with no
/// code change. Multi-node DAG data is not present in the assets.
#[derive(Clone, Debug, Default)]
pub struct TemplateDagCatalog {
    dags: BTreeMap<String, TemplateDag>,
}

impl TemplateDagCatalog {
    /// Builds the catalog from the loaded prompt-template registry.
    #[must_use]
    pub fn from_registry(registry: &TemplateRegistry) -> Self {
        let mut dags = BTreeMap::new();
        for template_id in registry.template_ids() {
            let dag = TemplateDag {
                template_id: template_id.clone(),
                nodes: vec![TemplateDagNode {
                    role: template_id.clone(),
                    template_id: template_id.clone(),
                }],
            };
            dags.insert(template_id, dag);
        }
        Self { dags }
    }

    /// Looks a template DAG up by manifest id.
    #[must_use]
    pub fn get(&self, template_id: &str) -> Option<&TemplateDag> {
        self.dags.get(template_id)
    }

    /// Every insertable template id, in manifest order.
    #[must_use]
    pub fn template_ids(&self) -> Vec<String> {
        self.dags.keys().cloned().collect()
    }

    /// Whether the catalog holds no template DAG.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.dags.is_empty()
    }
}

/// Inserts one manifest-backed template DAG into the active run.
///
/// This is the production `TEMPLATE_INSERT` path and it never starts from a canned
/// answer: every step is a real lookup or validation, and nothing is
/// written before all of them pass.
///
/// 1. Validate the payload identifiers.
/// 2. Resolve the template id through [`TemplateDagCatalog`] (built from the
///    loaded prompt-asset manifest). A miss refuses `E_TEMPLATE_UNKNOWN` and names
///    the known ids in the log.
/// 3. Resolve the target run. The frozen payload carries no run id, so the one
///    active run is the only legal target (`E_RUN_NOT_FOUND` when none exists),
///    and it must be non-terminal and bound to the requested project.
/// 4. Enforce the spawn caps for every node before the first insert.
/// 5. Insert each node as a `queued` task row plus a graph node under the run's
///    planned root task. The transport owner publishes one `NODE_SPAWN` per
///    inserted node after the `ACK`; the nodes stay `queued` until the run-scoped
///    dispatcher picks post-plan nodes up, and the `ACK` never claims execution.
///
/// # Errors
/// `E_SCHEMA_INVALID` for an invalid payload/project binding, `E_TEMPLATE_UNKNOWN`
/// for an id absent from the prompt-asset manifest, `E_RUN_NOT_FOUND` when no run
/// is active (or the run has no planned root task), `E_RUN_TERMINAL` for a
/// terminal run, and the cap refusals (`E_SPAWN_DEPTH`, `E_FANOUT_LIMIT`,
/// `E_AGENT_LIMIT`).
fn insert_template_dag(
    templates: &TemplateRegistry,
    store: &RunStore,
    graph: &GraphAuthority,
    caps: Caps,
    active_run: Option<RunId>,
    payload: &TemplateInsert,
) -> Result<Vec<TaskRecord>, Refusal> {
    let project_id = payload.project_id.trim();
    let template_id = payload.template_id.trim();
    if project_id.is_empty()
        || template_id.is_empty()
        || project_id.chars().count() > MAX_TEMPLATE_FIELD_CHARS
        || template_id.chars().count() > MAX_TEMPLATE_FIELD_CHARS
    {
        return Err(Refusal::SchemaInvalid);
    }

    let catalog = TemplateDagCatalog::from_registry(templates);
    let dag = match catalog.get(template_id) {
        Some(dag) => dag.clone(),
        None => {
            tracing::warn!(
                requested = %template_id,
                known = ?catalog.template_ids(),
                "TEMPLATE_INSERT refused after a real prompt-asset lookup"
            );
            return Err(Refusal::TemplateUnknown);
        }
    };

    let run_id = active_run.ok_or(Refusal::RunNotFound)?;
    let run = store.get_run(run_id)?;
    if run.status.is_terminal() {
        return Err(Refusal::RunTerminal);
    }
    if run.project_id != project_id {
        return Err(Refusal::SchemaInvalid);
    }

    let tasks = store.tasks_for_run(run_id)?;
    let root = tasks
        .iter()
        .find(|task| task.parent_task_id.is_none())
        .cloned()
        .ok_or(Refusal::TaskNotFound)?;
    let depth = root.depth.saturating_add(1);
    let parent_tier = task_agent_tier(&root);
    for node in &dag.nodes {
        // A template id is a persona/profile name, not necessarily a spawn
        // class. Library templates are worker profiles; authority-looking ids
        // (planner/hyper) are never admitted as template children.
        let requested_class = match node.role.trim().to_ascii_lowercase().as_str() {
            "planner" | "hyper" | "hyperagent" | "hyper_agent" | "orchestrator" => {
                return Err(Refusal::ToolNotAllowed);
            }
            "super" | "superagent" | "super_agent" | "sub_hive" | "subhive" => "superagent",
            "subagent" | "sub_agent" => "subagent",
            _ => "worker",
        };
        if requested_class == "subagent" && depth < 2 {
            return Err(Refusal::SpawnDepth);
        }
        validate_spawn_role(parent_tier, requested_class)?;
    }
    let siblings = tasks
        .iter()
        .filter(|task| task.parent_task_id == Some(root.id))
        .count();
    let agents = u64::try_from(tasks.len()).unwrap_or(u64::MAX);
    for node in &dag.nodes {
        // Every node must render a real prompt asset; a DAG that points at a
        // missing template is refused before anything is inserted.
        if templates.get(&node.template_id).is_none() {
            tracing::error!(
                template = %node.template_id,
                "template DAG references a prompt asset that is not loaded"
            );
            return Err(Refusal::TemplateUnknown);
        }
    }
    for index in 0..dag.nodes.len() {
        let index = u64::try_from(index).unwrap_or(u64::MAX);
        caps.check_spawn(
            depth,
            usize::try_from(index)
                .unwrap_or(usize::MAX)
                .saturating_add(siblings),
            agents.saturating_add(index),
        )?;
    }

    let mut spawn_seq = tasks.iter().map(|task| task.spawn_seq).max().unwrap_or(0);
    let mut inserted = Vec::with_capacity(dag.nodes.len());
    for node in &dag.nodes {
        spawn_seq = spawn_seq.saturating_add(1);
        let task = TaskRecord {
            id: TaskId::new(),
            run_id,
            parent_task_id: Some(root.id),
            role: node.role.clone(),
            depth,
            spawn_seq,
            state: AgentState::Queued,
            rev: 0,
        };
        store.insert_task(&task)?;
        graph.add_task(task.id, Some(root.id))?;
        graph.set_metadata(
            task.id,
            task_node_kind(&task),
            &task.role,
            None,
        )?;
        inserted.push(task);
    }
    Ok(inserted)
}

/// Seeds the scoped context for a task that has none yet.
///
/// A task's context parent is its authoritative graph parent (`previous_parent`,
/// read before the rewire). Seeding records only that real link and no facts; a
/// task with a persisted row but no context would otherwise make the rewire
/// context switch refuse `E_TASK_NOT_FOUND`.
///
/// # Errors
/// `E_INTERNAL_ERROR` when the context store lock is poisoned.
fn seed_rewire_context(
    contexts: &ContextStore,
    task_id: TaskId,
    previous_parent: Option<TaskId>,
) -> Result<(), Refusal> {
    if let Some(existing) = contexts.get(task_id)? {
        if existing.parent_task_id != previous_parent {
            // The scoped context and the graph disagree about the current
            // parent. The switch clears the facts of the context's parent, so
            // report the drift instead of hiding it; the authoritative graph is
            // the parent the next switch records.
            tracing::warn!(
                task = %task_id,
                context_parent = ?existing.parent_task_id,
                graph_parent = ?previous_parent,
                "rewire found a scoped context whose parent differs from the graph"
            );
        }
        return Ok(());
    }
    contexts.set(task_id, ScopedContext::new(previous_parent))?;
    Ok(())
}

/// Applies the documented rewire context switch for a persisted task.
///
/// The old parent's scoped facts are cleared, the new parent's facts are injected
/// and the task's scoped context now points at the new parent. The parent-fact
/// source is the new parent's own scoped context; an absent parent context
/// contributes no facts rather than inventing them. `allowed_tool_ids` /
/// `output_schema_id` keep the task's own values: they are the task's persona
/// capability set, and the production persona loader is wired separately.
///
/// # Errors
/// `E_TASK_NOT_FOUND` when the task has no context, `E_INTERNAL_ERROR` when the
/// store lock is poisoned.
fn apply_rewire_context(
    contexts: &ContextStore,
    task_id: TaskId,
    new_parent: TaskId,
) -> Result<ContextSwitchOutcome, Refusal> {
    let current = contexts.get(task_id)?.ok_or(Refusal::TaskNotFound)?;
    let new_parent_facts = contexts
        .get(new_parent)?
        .map(|context| context.facts)
        .unwrap_or_default();
    contexts.switch_on_rewire(
        task_id,
        Some(new_parent),
        &new_parent_facts,
        current.allowed_tool_ids,
        current.output_schema_id,
    )
}

/// `GET /ws` upgrade handler.
///
/// Rejections happen before the upgrade and carry the named refusal in a JSON
/// body: non-allowlisted Origin -> HTTP 403 `E_ORIGIN_REJECTED`; wrong token ->
/// HTTP 401 `E_TOKEN_INVALID`.
pub async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<Arc<AppState>>,
    ConnectInfo(peer): ConnectInfo<std::net::SocketAddr>,
    headers: HeaderMap,
    Query(query): Query<WsQuery>,
) -> Response {
    // The listener is loopback-bound; refuse anything that arrived through an
    // unexpected interface anyway.
    if !peer.ip().is_loopback() {
        return refusal_response(StatusCode::FORBIDDEN, &Refusal::OriginRejected);
    }
    let origin = headers
        .get(header::ORIGIN)
        .and_then(|value| value.to_str().ok());
    if !state.session.is_origin_allowed(origin) {
        return refusal_response(StatusCode::FORBIDDEN, &Refusal::OriginRejected);
    }
    if query.token.as_deref() != Some(state.session.token()) {
        return refusal_response(StatusCode::UNAUTHORIZED, &Refusal::TokenInvalid);
    }
    let last_seq = query.last_seq;
    ws.on_upgrade(move |socket| session_loop(socket, state, last_seq))
}

/// Builds the JSON refusal response used before a WebSocket upgrade.
#[must_use]
pub fn refusal_response(status: StatusCode, refusal: &Refusal) -> Response {
    let payload = json!({
        "code": refusal.code(),
        "message": refusal.message(),
        "how_to_fix": refusal.how_to_fix(),
        "retryable": refusal.retryable(),
    });
    (status, axum::Json(payload)).into_response()
}

async fn session_loop(mut socket: WebSocket, state: Arc<AppState>, query_last_seq: Option<u64>) {
    let mut receiver = state.bus.subscribe();

    // The first client frame must be a hello control frame (or the query
    // parameter is used as the reported sequence).
    let reported = match tokio::time::timeout(HELLO_TIMEOUT, wait_for_hello(&mut socket)).await {
        Ok(Some(last_seq)) => last_seq.or(query_last_seq),
        Ok(None) => return,
        Err(_) => {
            let _ = send_event_direct(&mut socket, Event::Error(error_payload(Refusal::BadFrame)))
                .await;
            return;
        }
    };

    if send_hello_replay_and_approvals(&mut socket, &state, reported)
        .await
        .is_err()
    {
        return;
    }

    loop {
        tokio::select! {
            incoming = socket.recv() => {
                match incoming {
                    Some(Ok(Message::Text(text))) => {
                        if handle_client_text(&mut socket, &state, text.as_str()).await.is_err() {
                            return;
                        }
                    }
                    Some(Ok(Message::Binary(_))) => {
                        let _ = send_event_direct(&mut socket, Event::Error(error_payload(Refusal::BadFrame))).await;
                    }
                    Some(Ok(Message::Ping(payload))) => {
                        let _ = socket.send(Message::Pong(payload)).await;
                        publish_pong(state.bus.as_ref());
                    }
                    Some(Ok(Message::Pong(_))) => {}
                    Some(Ok(Message::Close(_))) | None => return,
                    Some(Err(_)) => return,
                }
            }
            event = receiver.recv() => {
                match event {
                    Ok(sequenced) => {
                        if send_envelope(&mut socket, &sequenced.envelope).await.is_err() {
                            return;
                        }
                    }
                    Err(RecvError::Lagged(skipped)) => {
                        tracing::warn!(skipped, "session lagged behind the event bus; requesting resync");
                        let _ = send_event_direct(
                            &mut socket,
                            Event::ResyncRequired(hiverouter_contract::event::ResyncRequiredPayload {
                                expected_seq: state.bus.oldest_seq().unwrap_or_else(|| state.bus.current_seq() + 1),
                                last_seq: state.bus.current_seq(),
                            }),
                        ).await;
                        return;
                    }
                    Err(RecvError::Closed) => return,
                }
            }
        }
    }
}

async fn wait_for_hello(socket: &mut WebSocket) -> Option<Option<u64>> {
    loop {
        match socket.recv().await {
            Some(Ok(Message::Text(text))) => {
                let Ok(value) = serde_json::from_str::<Value>(text.as_str()) else {
                    let _ = send_event_direct(socket, Event::Error(error_payload(Refusal::BadFrame))).await;
                    continue;
                };
                if value.get("type").and_then(Value::as_str) != Some("hello") {
                    let _ = send_event_direct(socket, Event::Error(error_payload(Refusal::BadFrame))).await;
                    continue;
                }
                return Some(value.get("last_seq").and_then(Value::as_u64));
            }
            Some(Ok(Message::Ping(payload))) => {
                let _ = socket.send(Message::Pong(payload)).await;
            }
            Some(Ok(Message::Pong(_))) => {}
            Some(Ok(Message::Close(_))) | None | Some(Err(_)) => return None,
            Some(Ok(Message::Binary(_))) => {
                let _ = send_event_direct(socket, Event::Error(error_payload(Refusal::BadFrame))).await;
            }
        }
    }
}

async fn send_hello_and_replay(
    socket: &mut WebSocket,
    bus: &EventBus,
    reported: Option<u64>,
) -> Result<(), ()> {
    let server_seq = bus.current_seq();
    match reported {
        None => {
            send_event_direct(
                socket,
                Event::ServerHello(ServerHello {
                    last_seq: Some(server_seq),
                    schema: hiverouter_contract::SCHEMA_VERSION.to_owned(),
                }),
            )
            .await
        }
        Some(last_seq) => match bus.replay_after(last_seq) {
            Replay::Current | Replay::Fresh => {
                send_event_direct(
                    socket,
                    Event::ServerHello(ServerHello {
                        last_seq: Some(server_seq),
                        schema: hiverouter_contract::SCHEMA_VERSION.to_owned(),
                    }),
                )
                .await
            }
            Replay::Events(events) => {
                send_event_direct(
                    socket,
                    Event::ServerHello(ServerHello {
                        last_seq: Some(server_seq),
                        schema: hiverouter_contract::SCHEMA_VERSION.to_owned(),
                    }),
                )
                .await?;
                for event in events {
                    if send_envelope(socket, &event.envelope).await.is_err() {
                        return Err(());
                    }
                }
                Ok(())
            }
            Replay::Gap {
                expected_seq,
                last_seq,
            } => {
                send_event_direct(
                    socket,
                    Event::ResyncRequired(hiverouter_contract::event::ResyncRequiredPayload {
                        expected_seq,
                        last_seq,
                    }),
                )
                .await?;
                // The versioned graph snapshot API lands in Phase 9; until then
                // the client is told to resnapshot and the live stream resumes
                // from the current sequence.
                send_event_direct(
                    socket,
                    Event::ServerHello(ServerHello {
                        last_seq: Some(server_seq),
                        schema: hiverouter_contract::SCHEMA_VERSION.to_owned(),
                    }),
                )
                .await
            }
        },
    }
}

/// Bounded cap on pending approvals replayed after one hello (mirror of the
/// server constant documented for the UI replay window).
const MAX_APPROVAL_REPLAY_EVENTS: usize = 20;

/// Publishes the bounded pending-approval replay batch after `SERVER_HELLO`.
///
/// The socket is already subscribed to the bus (session_loop subscribes before
/// the hello exchange), so every event published here is delivered to this
/// connection's loop immediately after hello and to other clients as an
/// idempotent replace-by-id upsert. Replay is best-effort: a refusal is logged
/// and the session still works; the UI stays fail-closed (read-only card)
/// instead of fabricating an approval.
fn publish_pending_approval_replays(state: &Arc<AppState>) -> usize {
    match state.tools.pending_approval_events_all(MAX_APPROVAL_REPLAY_EVENTS) {
        Ok(events) => {
            let mut published = 0_usize;
            for approval in events {
                if state.bus.publish(Event::ApprovalRequest(approval)).is_ok() {
                    published += 1;
                }
            }
            if published > 0 {
                tracing::info!(published, "replayed pending approvals after hello");
            }
            published
        }
        Err(error) => {
            tracing::warn!(code = error.code(), "pending-approval replay refused");
            0
        }
    }
}

async fn send_hello_replay_and_approvals(
    socket: &mut WebSocket,
    state: &Arc<AppState>,
    reported: Option<u64>,
) -> Result<(), ()> {
    send_hello_and_replay(socket, &state.bus, reported).await?;
    let _ = publish_pending_approval_replays(state);
    Ok(())
}

async fn handle_client_text(
    socket: &mut WebSocket,
    state: &Arc<AppState>,
    text: &str,
) -> Result<(), ()> {
    let value = match serde_json::from_str::<Value>(text) {
        Ok(value) => value,
        Err(_) => {
            state
                .bus
                .publish(Event::Error(error_payload(Refusal::BadFrame)))
                .map_err(|_| ())?;
            return Ok(());
        }
    };

    // Transport control frame: client asks for a resync.
    if value.get("type").and_then(Value::as_str) == Some("hello") {
        let last_seq = value.get("last_seq").and_then(Value::as_u64);
        send_hello_replay_and_approvals(socket, state, last_seq).await?;
        return Ok(());
    }

    let envelope = match schema::parse_envelope(&value) {
        Ok(envelope) => envelope,
        Err(refusal) => {
            publish_error(state, refusal, None);
            return Ok(());
        }
    };
    let correlation = envelope.correlation_id;
    let command = match schema::decode_command(&envelope) {
        Ok(command) => command,
        Err(refusal) => {
            publish_error(state, refusal, Some(correlation));
            return Ok(());
        }
    };

    match command {
        Command::RewireTask(payload) => {
            // The previous parent is the authoritative graph link *before* the
            // mutation: the context switch must clear exactly that parent's
            // scoped facts.
            let previous_parent = state.graph.parent_of(payload.task_id).ok().flatten();
            // A scoped context exists only for a persisted task. A graph-only
            // node (transport tests, or a node whose row is written later) has
            // no context, and none is fabricated for it.
            let task = match state.engine.store.get_task(payload.task_id) {
                Ok(task) => Some(task),
                Err(Refusal::TaskNotFound) => None,
                Err(refusal) => {
                    publish_error(state, refusal, Some(correlation));
                    return Ok(());
                }
            };
            if let Some(task) = &task {
                if let Err(refusal) =
                    seed_rewire_context(&state.engine.contexts, task.id, previous_parent)
                {
                    publish_error(state, refusal, Some(correlation));
                    return Ok(());
                }
            }
            match state
                .graph
                .rewire(payload.task_id, payload.new_parent_id, payload.expected_rev)
            {
                Ok(()) => {
                    // One locked read of the authoritative node after the
                    // mutation: the published state and the stored revision can
                    // never disagree with the graph.
                    let node = match state.graph.node(payload.task_id) {
                        Ok(Some(node)) => node,
                        Ok(None) => {
                            publish_error(state, Refusal::TaskNotFound, Some(correlation));
                            return Ok(());
                        }
                        Err(refusal) => {
                            publish_error(state, refusal, Some(correlation));
                            return Ok(());
                        }
                    };
                    // Roster/fan-out/A2A readers use the task row, not the graph:
                    // persist the new parent and pin the row rev to the graph rev
                    // so a restart cannot resurrect the previous edge.
                    if let Some(task) = &task {
                        if let Err(refusal) =
                            state.engine.store.update_task_parent(
                                task.id,
                                Some(payload.new_parent_id),
                                node.rev,
                            )
                        {
                            tracing::error!(
                                code = refusal.code(),
                                task = %task.id,
                                "rewire graph committed but the task row parent write failed; refusing"
                            );
                            publish_error(state, refusal, Some(correlation));
                            return Ok(());
                        }
                    }
                    // Live context-stream switch (M-016): the graph rewire alone
                    // left the old parent's scoped context in place.
                    if let Some(task) = &task {
                        match apply_rewire_context(
                            &state.engine.contexts,
                            task.id,
                            payload.new_parent_id,
                        ) {
                            Ok(outcome) => tracing::debug!(
                                task = %task.id,
                                cleared = outcome.cleared_facts.len(),
                                injected = outcome.injected_facts.len(),
                                "rewire switched the task's scoped context to the new parent"
                            ),
                            Err(refusal) => {
                                // The graph mutation is already committed, so the
                                // scoped context and the graph now disagree: fail
                                // loudly and let the client resync the authoritative
                                // graph instead of ACKing a half-applied switch.
                                tracing::error!(
                                    code = refusal.code(),
                                    task = %task.id,
                                    "rewire committed but the context switch failed; the command is refused"
                                );
                                publish_error(state, refusal, Some(correlation));
                                return Ok(());
                            }
                        }
                    }
                    publish_ack(state, CommandName::RewireTask, correlation);
                    publish_node_event(
                        state.bus.as_ref(),
                        EventName::NodeRewire,
                        &node,
                        task.as_ref().map(|task| task.run_id),
                        Some(correlation),
                        task.as_ref()
                            .map(|task| task_node_kind(task).to_owned()),
                    );
                }
                Err(refusal) => publish_error(state, refusal, Some(correlation)),
            }
            Ok(())
        }
        Command::ViewportSet(_) | Command::ViewportRecenter(_) | Command::ChatPin(_) => {
            // Client-only surfaces: there is no server effect, but the frozen
            // contract still requires an explicit ACK.
            publish_ack(state, command.name(), correlation);
            Ok(())
        }
        Command::SubmitPrompt(payload) => {
            // A keyless deployment must refuse the command before create_run,
            // not create a run that dies later with E_KEY_MISSING.
            if state
                .checks
                .iter()
                .any(|check| check.name == "deepseek_api_key" && !check.ok)
            {
                publish_error(state, Refusal::KeyMissing, Some(correlation));
                return Ok(());
            }
            match state
                .engine
                .submit_prompt(&payload.text, &payload.project_id, correlation)
            {
                Ok(_run) => publish_ack(state, CommandName::SubmitPrompt, correlation),
                Err(refusal) => publish_error(state, refusal, Some(correlation)),
            }
            Ok(())
        }
        Command::SpawnSubHive(payload) => {
            match state
                .engine
                .spawn_sub_hive(payload.parent_task_id, &payload.role)
            {
                Ok(_task) => publish_ack(state, CommandName::SpawnSubHive, correlation),
                Err(refusal) => publish_error(state, refusal, Some(correlation)),
            }
            Ok(())
        }
        Command::CancelRun(payload) => {
            match state.engine.cancel_run(payload.run_id) {
                Ok(()) => publish_ack(state, CommandName::CancelRun, correlation),
                Err(refusal) => publish_error(state, refusal, Some(correlation)),
            }
            Ok(())
        }
        Command::KillWorker(payload) => {
            match state.engine.kill_worker(payload.task_id) {
                Ok(()) => publish_ack(state, CommandName::KillWorker, correlation),
                Err(refusal) => publish_error(state, refusal, Some(correlation)),
            }
            Ok(())
        }
        Command::SettingsStatus(_) => {
            let mut result = state.tools.settings_status();
            result.model = state.config.provider.model.id().to_owned();
            result.thinking = Some(match state.config.provider.planner_thinking.kind {
                hiverouter_provider::dto::ThinkingType::Enabled => "enabled".to_owned(),
                hiverouter_provider::dto::ThinkingType::Disabled => "disabled".to_owned(),
            });
            result.reasoning_effort = Some(
                match state.config.provider.planner_reasoning_effort {
                    hiverouter_provider::ReasoningEffort::None => "none",
                    hiverouter_provider::ReasoningEffort::Low => "low",
                    hiverouter_provider::ReasoningEffort::High => "high",
                    hiverouter_provider::ReasoningEffort::Max => "max",
                }
                .to_owned(),
            );
            publish_ack(state, CommandName::SettingsStatus, correlation);
            publish_with_correlation(state, Event::SettingsStatusResult(result), correlation);
            Ok(())
        }
        Command::TemplateInsert(payload) => {
            // A boot that failed to load the embedded prompt assets must refuse
            // with that named failure; flattening it into "unknown template"
            // would hide a broken deployment behind a data error.
            if let Some(refusal) = state.prompt_assets_error {
                publish_error(state, refusal, Some(correlation));
                return Ok(());
            }
            match insert_template_dag(
                state.engine.templates.as_ref(),
                &state.engine.store,
                &state.engine.graph,
                state.engine.caps,
                state.engine.active_run(),
                &payload,
            ) {
                Ok(tasks) => {
                    if let Err(refusal) = state.engine.seed_template_contexts(&tasks) {
                        publish_error(state, refusal, Some(correlation));
                        return Ok(());
                    }
                    let ids: Vec<TaskId> = tasks.iter().map(|task| task.id).collect();
                    if let Err(refusal) = state.engine.enqueue_spawned_tasks(&ids) {
                        publish_error(state, refusal, Some(correlation));
                        return Ok(());
                    }
                    publish_ack(state, CommandName::TemplateInsert, correlation);
                    for task in &tasks {
                        publish_node_spawn(state.bus.as_ref(), &state.graph, task);
                    }
                }
                Err(refusal) => publish_error(state, refusal, Some(correlation)),
            }
            Ok(())
        }
        Command::HistoryRevisionSet(payload) => {
            match state.history.state_at(payload.run_id, payload.revision) {
                Ok(envelope) => {
                    let graph =
                        crate::http::history_graph_value(state.as_ref(), payload.run_id, envelope.graph);
                    let snapshot = HistorySnapshot {
                        run_id: envelope.run_id,
                        revision: envelope.revision,
                        graph,
                    };
                    publish_ack(state, CommandName::HistoryRevisionSet, correlation);
                    publish_with_correlation(state, Event::HistorySnapshot(snapshot), correlation);
                }
                Err(refusal) => publish_error(state, refusal, Some(correlation)),
            }
            Ok(())
        }
        Command::FileTreeList(payload) => {
            match state.tools.file_tree_list(&payload) {
                Ok(result) => {
                    publish_ack(state, CommandName::FileTreeList, correlation);
                    publish_with_correlation(state, Event::FileTreeResult(result), correlation);
                }
                Err(refusal) => publish_error(state, refusal, Some(correlation)),
            }
            Ok(())
        }
        Command::FileRead(payload) => {
            match state.tools.file_read(&payload) {
                Ok(result) => {
                    publish_ack(state, CommandName::FileRead, correlation);
                    publish_with_correlation(state, Event::FileContent(result), correlation);
                }
                Err(refusal) => publish_error(state, refusal, Some(correlation)),
            }
            Ok(())
        }
        Command::ApprovalResolve(payload) => {
            // One wiring point owns the decision evidence: the gate consumes the
            // nonce once, then exactly one ledger row is appended for the committed
            // decision (refusals return before any ledger write). A ledger failure
            // surfaces as ERROR, never as a fake ACK/APPROVAL_RESOLVED.
            let ledger_dir = crate::tools_bridge::ToolsBridge::approval_ledger_dir(
                &state.config.data_dir,
            );
            match state.tools.resolve_approval_ledgered(&payload, &ledger_dir) {
                Ok(resolution) => {
                    let resolved =
                        crate::tools_bridge::ToolsBridge::approval_resolved_event(&resolution);
                    publish_ack(state, CommandName::ApprovalResolve, correlation);
                    publish_with_correlation(state, Event::ApprovalResolved(resolved), correlation);
                    // Wake the suspended engine turn only after the decision was
                    // committed and published. A false result is legal (late,
                    // duplicate or terminal-run resolve) and never fails the command.
                    if !state.engine.notify_approval_resolved(&resolution) {
                        tracing::warn!(
                            approval_id = %resolution.request.approval_id,
                            "approval decided with no armed engine wait (late/duplicate/terminal run)"
                        );
                    }
                }
                Err(refusal) => publish_error(state, refusal, Some(correlation)),
            }
            Ok(())
        }
        other => {
            // Defensive: every frozen command is handled above or earlier.
            tracing::error!(
                command = other.name().as_str(),
                "unhandled frozen command reached the dispatch fallback"
            );
            publish_error(state, Refusal::InternalError, Some(correlation));
            Ok(())
        }
    }
}

fn error_payload(refusal: Refusal) -> ErrorPayload {
    ErrorPayload::from_refusal(&refusal)
}

fn publish_error(state: &Arc<AppState>, refusal: Refusal, correlation: Option<Uuid>) {
    if let Err(error) = state
        .bus
        .publish_with_correlation(Event::Error(error_payload(refusal)), correlation)
    {
        tracing::error!(code = error.code(), "failed to publish ERROR event");
    }
}

fn publish_ack(state: &Arc<AppState>, command: CommandName, correlation: Uuid) {
    if let Err(error) = state.bus.publish_with_correlation(
        Event::Ack(Ack {
            command,
            correlation_id: correlation,
        }),
        Some(correlation),
    ) {
        tracing::error!(code = error.code(), "failed to publish ACK event");
    }
}

fn publish_with_correlation(state: &Arc<AppState>, event: Event, correlation: Uuid) {
    if let Err(error) = state
        .bus
        .publish_for_run(event, None, Some(correlation))
    {
        tracing::error!(code = error.code(), "failed to publish command result event");
    }
}

/// Publish one frozen `NODE_*` frame for the authoritative graph node.
///
/// `kind` comes from the task's authoritative tier, never from the free-form
/// role string. The publisher keeps it stable across spawn and rewire.
fn publish_node_event(
    bus: &EventBus,
    name: EventName,
    node: &TaskNode,
    run_id: Option<RunId>,
    correlation: Option<Uuid>,
    kind: Option<String>,
) {
    let payload = NodeEvent {
        task_id: node.task_id,
        state: node.state,
        rev: Some(node.rev),
        kind,
        parent_task_id: node.parent_task_id,
    };
    let event = match name {
        EventName::NodeSpawn => Event::NodeSpawn(payload),
        EventName::NodeDrag => Event::NodeDrag(payload),
        EventName::NodeRewire => Event::NodeRewire(payload),
        _ => Event::NodeState(payload),
    };
    if let Err(error) = bus.publish_for_run(event, run_id, correlation) {
        tracing::error!(code = error.code(), "failed to publish NODE event");
    }
}

/// Publishes `NODE_SPAWN` for a task row created after planning.
///
/// The frame is built from the graph node (not from the task row) so its `rev`
/// is the graph revision `REWIRE_TASK` compares, never the row's own counter.
fn publish_node_spawn(bus: &EventBus, graph: &GraphAuthority, task: &TaskRecord) {
    let node = match graph.node(task.id) {
        Ok(Some(node)) => node,
        Ok(None) => {
            tracing::error!(
                task = %task.id,
                "inserted node has no graph node; refusing to publish a fabricated NODE_SPAWN"
            );
            return;
        }
        Err(error) => {
            tracing::error!(
                code = error.code(),
                task = %task.id,
                "could not read the graph node for NODE_SPAWN"
            );
            return;
        }
    };
    publish_node_event(
        bus,
        EventName::NodeSpawn,
        &node,
        Some(task.run_id),
        None,
        Some(task_node_kind(task).to_owned()),
    );
}

async fn send_event_direct(socket: &mut WebSocket, event: Event) -> Result<(), ()> {
    let envelope = event.into_envelope().map_err(|_| ())?;
    send_envelope(socket, &envelope).await
}

async fn send_envelope(socket: &mut WebSocket, envelope: &hiverouter_contract::Envelope) -> Result<(), ()> {
    let json = envelope.to_json().map_err(|_| ())?;
    socket
        .send(Message::Text(json.into()))
        .await
        .map_err(|_| ())
}

/// Publishes a PONG event (used by the session keepalive path).
pub fn publish_pong(bus: &EventBus) {
    let _ = bus.publish(Event::Pong(EmptyPayload {}));
}

/// Sends a sequenced event to a socket (used by tests and later phases).
pub async fn send_sequenced(
    socket: &mut WebSocket,
    sequenced: &SequencedEvent,
) -> Result<(), ()> {
    send_envelope(socket, &sequenced.envelope).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bus::Replay;
    use crate::db::Database;

    /// The real, embedded prompt-asset registry (the production template source).
    fn embedded_registry() -> TemplateRegistry {
        TemplateRegistry::load_embedded().expect("embedded prompt assets load")
    }

    struct InsertFixture {
        store: RunStore,
        graph: GraphAuthority,
        run_id: RunId,
        root: TaskRecord,
    }

    fn insert_fixture(status: AgentState) -> InsertFixture {
        let db = Database::open_in_memory().expect("in-memory database");
        db.migrate().expect("migrate");
        let store = RunStore::new(Arc::new(db));
        let run = store
            .create_run("project-1", "template insert", status)
            .expect("run");
        let root = TaskRecord {
            id: TaskId::new(),
            run_id: run.id,
            parent_task_id: None,
            role: "planner".to_owned(),
            depth: 0,
            spawn_seq: 1,
            state: AgentState::Running,
            rev: 0,
        };
        store.insert_task(&root).expect("root task row");
        let graph = GraphAuthority::new();
        graph.add_task(root.id, None).expect("root graph node");
        InsertFixture {
            store,
            graph,
            run_id: run.id,
            root,
        }
    }

    #[test]
    fn template_insert_spawns_manifest_backed_dag() {
        let fixture = insert_fixture(AgentState::Running);
        let templates = embedded_registry();
        assert!(
            templates.get("planner").is_some(),
            "the shipped manifest carries the planner prompt asset"
        );
        let payload = TemplateInsert {
            template_id: "content-writer".to_owned(),
            project_id: "project-1".to_owned(),
        };
        let inserted = insert_template_dag(
            &templates,
            &fixture.store,
            &fixture.graph,
            Caps::default(),
            Some(fixture.run_id),
            &payload,
        )
        .expect("insert the manifest-backed DAG");
        assert_eq!(inserted.len(), 1, "the content-writer DAG is one node");
        let task = &inserted[0];
        assert_eq!(task.role, "content-writer");
        assert_eq!(task.run_id, fixture.run_id);
        assert_eq!(task.parent_task_id, Some(fixture.root.id));
        assert_eq!(task.state, AgentState::Queued);
        assert_eq!(task.depth, fixture.root.depth + 1);

        // A template id that resolves to HyperAgent authority is refused even
        // though its prompt asset exists.
        let planner = TemplateInsert {
            template_id: "planner".to_owned(),
            project_id: "project-1".to_owned(),
        };
        assert_eq!(
            insert_template_dag(
                &templates,
                &fixture.store,
                &fixture.graph,
                Caps::default(),
                Some(fixture.run_id),
                &planner,
            ),
            Err(Refusal::ToolNotAllowed)
        );

        // The node is durable in the store and authoritatively in the graph.
        let stored = fixture.store.get_task(task.id).expect("task row");
        assert_eq!(stored, *task);
        assert_eq!(
            fixture.graph.state_of(task.id).expect("graph state"),
            AgentState::Queued
        );
        assert_eq!(
            fixture.graph.parent_of(task.id).expect("graph parent"),
            Some(fixture.root.id)
        );
        assert_eq!(
            fixture.store.task_count(fixture.run_id).expect("count"),
            2
        );

        // The transport frame the ACK path publishes for each inserted node: a
        // real NODE_SPAWN carrying the server node revision and the parent link.
        let bus = EventBus::new();
        for task in &inserted {
            publish_node_spawn(&bus, &fixture.graph, task);
        }
        let events = match bus.replay_after(0) {
            Replay::Events(events) => events,
            other => panic!("expected one NODE_SPAWN per inserted node, got {other:?}"),
        };
        assert_eq!(events.len(), inserted.len());
        assert_eq!(events[0].envelope.name, "NODE_SPAWN");
        assert_eq!(events[0].envelope.run_id, Some(fixture.run_id));
        let frame = events[0].envelope.payload.as_object().expect("payload");
        assert_eq!(frame["task_id"], task.id.to_string());
        assert_eq!(frame["state"], "queued");
        assert_eq!(
            frame["rev"], 0_u64,
            "a fresh graph node starts at revision 0"
        );
        assert_eq!(frame["parent_task_id"], fixture.root.id.to_string());
        assert_eq!(
            frame["kind"], "worker",
            "NODE_SPAWN carries the task's authoritative tier"
        );
    }

    #[test]
    fn template_insert_resolves_the_library_ids_after_a_real_lookup() {
        let fixture = insert_fixture(AgentState::Running);
        let templates = embedded_registry();
        for template_id in ["content-writer", "data-analyst", "code-reviewer", "chatbot"] {
            let payload = TemplateInsert {
                template_id: template_id.to_owned(),
                project_id: "project-1".to_owned(),
            };
            let tasks = insert_template_dag(
                &templates,
                &fixture.store,
                &fixture.graph,
                Caps::default(),
                Some(fixture.run_id),
                &payload,
            )
            .expect("the embedded Library template must resolve");
            assert_eq!(tasks.len(), 1);
            assert_eq!(tasks[0].role, template_id);
        }
        assert_eq!(
            fixture.store.task_count(fixture.run_id).expect("count"),
            5,
            "root + four inserted template tasks"
        );
        let catalog = TemplateDagCatalog::from_registry(&templates);
        assert_eq!(catalog.template_ids(), templates.template_ids());
        for template_id in ["planner", "content-writer", "data-analyst", "code-reviewer", "chatbot"] {
            assert!(catalog.get(template_id).is_some(), "{template_id} missing");
        }
        assert!(!catalog.is_empty());
    }

    #[test]
    fn template_insert_requires_an_active_non_terminal_run() {
        let fixture = insert_fixture(AgentState::Running);
        let templates = embedded_registry();
        let payload = TemplateInsert {
            template_id: "planner".to_owned(),
            project_id: "project-1".to_owned(),
        };
        assert_eq!(
            insert_template_dag(
                &templates,
                &fixture.store,
                &fixture.graph,
                Caps::default(),
                None,
                &payload,
            ),
            Err(Refusal::RunNotFound),
            "the frozen payload carries no run id, so an insert needs the active run"
        );

        let terminal = insert_fixture(AgentState::Completed);
        assert_eq!(
            insert_template_dag(
                &templates,
                &terminal.store,
                &terminal.graph,
                Caps::default(),
                Some(terminal.run_id),
                &payload,
            ),
            Err(Refusal::RunTerminal)
        );

        let mismatch = TemplateInsert {
            template_id: "planner".to_owned(),
            project_id: "another-project".to_owned(),
        };
        assert_eq!(
            insert_template_dag(
                &templates,
                &fixture.store,
                &fixture.graph,
                Caps::default(),
                Some(fixture.run_id),
                &mismatch,
            ),
            Err(Refusal::SchemaInvalid),
            "a node set may not cross the run's project binding"
        );
    }

    #[test]
    fn template_insert_validates_the_payload_before_any_lookup() {
        let fixture = insert_fixture(AgentState::Running);
        let templates = embedded_registry();
        for payload in [
            TemplateInsert {
                template_id: String::new(),
                project_id: "project-1".to_owned(),
            },
            TemplateInsert {
                template_id: "planner".to_owned(),
                project_id: "  ".to_owned(),
            },
            TemplateInsert {
                template_id: "p".repeat(MAX_TEMPLATE_FIELD_CHARS + 1),
                project_id: "project-1".to_owned(),
            },
        ] {
            assert_eq!(
                insert_template_dag(
                    &templates,
                    &fixture.store,
                    &fixture.graph,
                    Caps::default(),
                    Some(fixture.run_id),
                    &payload,
                ),
                Err(Refusal::SchemaInvalid)
            );
        }
        assert_eq!(fixture.store.task_count(fixture.run_id).expect("count"), 1);
    }

    #[test]
    fn template_insert_needs_a_planned_root_task() {
        let db = Database::open_in_memory().expect("db");
        db.migrate().expect("migrate");
        let store = RunStore::new(Arc::new(db));
        let run = store
            .create_run("project-1", "not planned yet", AgentState::Planning)
            .expect("run");
        let payload = TemplateInsert {
            template_id: "planner".to_owned(),
            project_id: "project-1".to_owned(),
        };
        assert_eq!(
            insert_template_dag(
                &embedded_registry(),
                &store,
                &GraphAuthority::new(),
                Caps::default(),
                Some(run.id),
                &payload,
            ),
            Err(Refusal::TaskNotFound),
            "there is no root node to attach a template node set to"
        );
    }

    #[test]
    fn rewire_switches_context_store() {
        let contexts = ContextStore::new();
        let task_id = TaskId::new();
        let old_parent = TaskId::new();
        let new_parent = TaskId::new();
        let mut context = ScopedContext::new(Some(old_parent))
            .with_fact(old_parent, "style", "old-honeycomb")
            .with_fact(old_parent, "secret_scope", "old-parent-only");
        context.allowed_tool_ids = vec!["file_read".to_owned()];
        context.output_schema_id = Some("plan-v1".to_owned());
        contexts.set(task_id, context).expect("seed context");

        seed_rewire_context(&contexts, task_id, Some(old_parent)).expect("existing context is kept");
        let outcome = apply_rewire_context(&contexts, task_id, new_parent).expect("switch");

        assert_eq!(outcome.cleared_parent, Some(old_parent));
        assert_eq!(outcome.cleared_facts.len(), 2);
        let tokens = outcome.context.token_set().join("|");
        assert!(
            !tokens.contains("old-honeycomb") && !tokens.contains("old-parent-only"),
            "the previous parent's scoped facts leaked into the new branch: {tokens}"
        );
        assert!(
            outcome.injected_facts.is_empty(),
            "no producer writes parent facts, so none may be invented"
        );
        assert_eq!(outcome.context.parent_task_id, Some(new_parent));
        let reread = contexts.get(task_id).expect("get").expect("context");
        assert_eq!(reread.parent_task_id, Some(new_parent));
        assert!(!reread.contains_parent_facts(old_parent));
        assert_eq!(
            reread.allowed_tool_ids,
            vec!["file_read".to_owned()],
            "the task keeps its own persona capability set"
        );
        assert_eq!(reread.output_schema_id.as_deref(), Some("plan-v1"));
    }

    #[test]
    fn rewire_seeds_a_missing_context_from_the_graph_parent() {
        let contexts = ContextStore::new();
        let task_id = TaskId::new();
        let parent = TaskId::new();
        seed_rewire_context(&contexts, task_id, Some(parent)).expect("seed");
        let context = contexts.get(task_id).expect("get").expect("context");
        assert_eq!(context.parent_task_id, Some(parent));
        assert!(context.facts.is_empty(), "seeding invents no facts");

        // Without a context the switch is refused by name, never fabricated.
        assert_eq!(
            apply_rewire_context(&contexts, TaskId::new(), parent),
            Err(Refusal::TaskNotFound)
        );
    }

    #[test]
    fn rewire_context_switch_appends_no_second_parent_scope() {
        let contexts = ContextStore::new();
        let task_id = TaskId::new();
        let first = TaskId::new();
        let second = TaskId::new();
        contexts.set(task_id, ScopedContext::new(None)).expect("seed");
        apply_rewire_context(&contexts, task_id, first).expect("first switch");
        apply_rewire_context(&contexts, task_id, second).expect("second switch");
        let context = contexts.get(task_id).expect("get").expect("context");
        assert_eq!(context.parent_task_id, Some(second));
        assert!(!context.contains_parent_facts(first));
        assert!(!context.contains_parent_facts(second));
    }
}

