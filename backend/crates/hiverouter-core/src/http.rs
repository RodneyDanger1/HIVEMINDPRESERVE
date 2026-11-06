//! Localhost HTTP surface: `GET /health`, `GET /api/session`, `GET /ws`, and an
//! honest 404 until the embedded web build exists.

use std::net::{IpAddr, SocketAddr};
use std::sync::Arc;
use std::time::Instant;

use axum::extract::{ConnectInfo, State};
use axum::http::{header, HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use axum::{Json, Router};
use serde::Serialize;
use tokio::net::TcpListener;
use uuid::Uuid;

use crate::bus::EventBus;
use crate::checks::Check;
use crate::config::Config;
use crate::db::Database;
use crate::graph::GraphAuthority;
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
}

impl AppState {
    /// Builds state for a running server with a fresh event bus, graph and
    /// session token derived from the configuration.
    #[must_use]
    pub fn new(db: Arc<Database>, checks: Vec<Check>, config: Arc<Config>) -> Self {
        let session = Arc::new(SessionAuth::new(
            Uuid::now_v7().to_string(),
            config.allowed_origins.clone(),
        ));
        Self {
            db,
            checks,
            started_at: Instant::now(),
            config,
            bus: Arc::new(EventBus::new()),
            graph: Arc::new(GraphAuthority::new()),
            session,
        }
    }

    /// Builds state with explicitly supplied services (integration tests).
    #[must_use]
    pub fn with_services(
        db: Arc<Database>,
        checks: Vec<Check>,
        config: Arc<Config>,
        bus: Arc<EventBus>,
        graph: Arc<GraphAuthority>,
        session: Arc<SessionAuth>,
    ) -> Self {
        Self {
            db,
            checks,
            started_at: Instant::now(),
            config,
            bus,
            graph,
            session,
        }
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

/// Builds the server router.
#[must_use]
pub fn build_router(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/health", get(health))
        .route("/api/session", get(api_session))
        .route("/ws", get(ws_handler))
        .fallback(not_found)
        .with_state(state)
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

async fn not_found() -> impl IntoResponse {
    (
        StatusCode::NOT_FOUND,
        Json(serde_json::json!({
            "error": "not found",
            "schema": hiverouter_contract::SCHEMA_VERSION,
        })),
    )
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
