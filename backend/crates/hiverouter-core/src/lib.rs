//! HiveMind v1 backend core: localhost axum server, environment checks, SQLite
//! bootstrap. Phase 1 surface only; later phases add transport, graph and stores
//! without duplicating these owners.

#![forbid(unsafe_code)]
#![warn(missing_debug_implementations)]

pub mod checks;
pub mod config;
pub mod db;
pub mod http;

pub use checks::{run_checks, Check};
pub use config::{Config, ConfigError};
pub use db::{Database, DbError, Migration, MIGRATIONS};
pub use http::{build_router, serve, AppState, HealthResponse};
