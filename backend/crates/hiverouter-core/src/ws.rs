//! Localhost WebSocket transport.
//!
//! One axum `/ws` endpoint (blueprint Phase 2, WP 2.5/2.6/2.8): loopback-only,
//! Origin allowlist + per-session token gate before the upgrade, `hello{last_seq}`
//! replay/resync on connect, and server->client fan-out from the [`EventBus`].
//!
//! Client -> server frames are either the transport `hello` control frame
//! (`{"type":"hello","last_seq":<u64|null>}`) or a frozen COMMAND envelope.
//! Every command gets a typed ACK or a typed ERROR event (never a silent drop).

use std::sync::Arc;

use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::extract::{ConnectInfo, Query, State};
use axum::http::{header, HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use serde::Deserialize;
use serde_json::{json, Value};
use tokio::sync::broadcast::error::RecvError;
use uuid::Uuid;

use hiverouter_contract::command::{Command, CommandName};
use hiverouter_contract::event::{
    Ack, EmptyPayload, ErrorPayload, Event, EventName, NodeEvent, ServerHello,
};
use hiverouter_contract::schema;
use hiverouter_contract::Refusal;

use crate::bus::{EventBus, Replay, SequencedEvent};
use crate::http::AppState;

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

    if !send_hello_and_replay(&mut socket, &state.bus, reported)
        .await
        .is_ok()
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
        send_hello_and_replay(socket, &state.bus, last_seq).await?;
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
            match state
                .graph
                .rewire(payload.task_id, payload.new_parent_id, payload.expected_rev)
            {
                Ok(()) => {
                    publish_ack(state, CommandName::RewireTask, correlation);
                    let state_value = AgentStateForEvent::running();
                    publish_node_event(
                        state,
                        EventName::NodeRewire,
                        payload.task_id,
                        state_value,
                        Some(correlation),
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
            let result = state.tools.settings_status();
            publish_ack(state, CommandName::SettingsStatus, correlation);
            publish_with_correlation(state, Event::SettingsStatusResult(result), correlation);
            Ok(())
        }
        Command::FileTreeList(_) => {
            match state.tools.file_tree_list() {
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
            match state.tools.resolve_approval(&payload) {
                Ok(resolution) => {
                    let resolved =
                        crate::tools_bridge::ToolsBridge::approval_resolved_event(&resolution);
                    publish_ack(state, CommandName::ApprovalResolve, correlation);
                    publish_with_correlation(state, Event::ApprovalResolved(resolved), correlation);
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

#[derive(Clone, Copy)]
struct AgentStateForEvent;

impl AgentStateForEvent {
    fn running() -> hiverouter_contract::AgentState {
        hiverouter_contract::AgentState::Running
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

fn publish_node_event(
    state: &Arc<AppState>,
    name: EventName,
    task_id: hiverouter_contract::TaskId,
    agent_state: hiverouter_contract::AgentState,
    correlation: Option<Uuid>,
) {
    let node = NodeEvent {
        task_id,
        state: agent_state,
    };
    let event = match name {
        EventName::NodeSpawn => Event::NodeSpawn(node),
        EventName::NodeDrag => Event::NodeDrag(node),
        EventName::NodeRewire => Event::NodeRewire(node),
        _ => Event::NodeState(node),
    };
    if let Err(error) = state.bus.publish_with_correlation(event, correlation) {
        tracing::error!(code = error.code(), "failed to publish NODE event");
    }
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
