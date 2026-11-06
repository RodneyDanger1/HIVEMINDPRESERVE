//! HiveMind v1 backend core: localhost axum server, environment checks, SQLite
//! bootstrap. Phase 1 surface only; later phases add transport, graph and stores
//! without duplicating these owners.

#![forbid(unsafe_code)]
#![warn(missing_debug_implementations)]

pub mod bus;
pub mod caps;
pub mod checks;
pub mod config;
pub mod context;
pub mod db;
pub mod entities;
pub mod execution;
pub mod graph;
pub mod handshake;
pub mod history;
pub mod http;
pub mod planner;
pub mod prompts;
pub mod provider_executor;
pub mod recovery;
pub mod snapshot;
pub mod static_ui;
pub mod store;
pub mod tools_bridge;
pub mod ws;

pub use bus::{EventBus, Replay, SequencedEvent};
pub use caps::Caps;
pub use checks::{run_checks, Check};
pub use config::{Config, ConfigError};
pub use context::{ContextStore, ScopedContext};
pub use db::{Database, DbError, Migration, MIGRATIONS};
pub use entities::{ChatMessage, ChatRole, Run, TaskRecord};
pub use execution::{DeltaSink, ProviderExecutor, RunEngine, RunRegistry, StreamedReply};
pub use graph::{GraphAuthority, GraphSnapshot, TaskNode};
pub use handshake::{HandshakeFrame, HandshakeLog, HandshakeOutcome};
pub use history::{HistoryReadModel, TimeTravel};
pub use http::{build_router, serve, AppState, HealthResponse};
pub use planner::{Plan, PlanNode};
pub use prompts::{Persona, TemplateRegistry};
pub use provider_executor::ProviderExecutorAdapter;
pub use recovery::{recover_on_boot, RecoveryReport};
pub use snapshot::{SnapshotEnvelope, SnapshotStore};
pub use store::RunStore;
pub use tools_bridge::ToolsBridge;
pub use ws::SessionAuth;
