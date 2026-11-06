//! `hiverouter` — the HiveMind v1 localhost server binary.
//!
//! Boot sequence: parse config -> open SQLite -> migrate -> run environment
//! checks -> bind loopback -> serve. Every failure is named and printed before
//! exit; the process never pretends to have started.

use std::process::ExitCode;
use std::sync::Arc;

use hiverouter_core::config::Config;
use hiverouter_core::db::Database;
use hiverouter_core::http::{self, AppState};
use hiverouter_core::recovery::recover_on_boot;
use hiverouter_core::run_checks;
use hiverouter_core::snapshot::SnapshotStore;
use tokio::net::TcpListener;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> ExitCode {
    init_tracing();
    if let Err(code) = run().await {
        return code;
    }
    ExitCode::SUCCESS
}

async fn run() -> Result<(), ExitCode> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.iter().any(|argument| argument == "--help" || argument == "-h") {
        print_usage();
        return Ok(());
    }
    if args
        .iter()
        .any(|argument| argument == "--version" || argument == "-V")
    {
        println!(
            "hiverouter {} (contract schema {})",
            env!("CARGO_PKG_VERSION"),
            hiverouter_contract::SCHEMA_VERSION
        );
        return Ok(());
    }

    let config = match Config::from_args(args) {
        Ok(config) => config,
        Err(error) => {
            eprintln!("hiverouter: {error}");
            return Err(ExitCode::FAILURE);
        }
    };

    let database = match Database::open(config.db_file()) {
        Ok(database) => database,
        Err(error) => {
            eprintln!(
                "hiverouter: cannot open database {}: {} ({})",
                config.db_file().display(),
                error,
                error.refusal().code()
            );
            return Err(ExitCode::FAILURE);
        }
    };
    let schema_version = match database.migrate() {
        Ok(version) => version,
        Err(error) => {
            eprintln!(
                "hiverouter: database migration failed: {} ({})",
                error,
                error.refusal().code()
            );
            return Err(ExitCode::FAILURE);
        }
    };

    let database = Arc::new(database);
    let snapshots = SnapshotStore::new(&config.data_dir, database.clone());
    let graph = hiverouter_core::graph::GraphAuthority::new();
    match recover_on_boot(database.clone(), &graph, &snapshots) {
        Ok(report) => {
            if !report.is_empty() {
                tracing::warn!(
                    interrupted = report.interrupted_runs.len(),
                    restored = report.restored_runs.len(),
                    corrupt = report.corrupt_snapshots.len(),
                    "kill-restore recovery ran; active runs marked interrupted"
                );
            }
        }
        Err(refusal) => {
            eprintln!(
                "hiverouter: boot recovery failed: {} ({}); refusing to start on a corrupt workspace",
                refusal.message(),
                refusal.code()
            );
            return Err(ExitCode::FAILURE);
        }
    }

    let checks = run_checks();
    for check in &checks {
        if check.ok {
            tracing::info!(check = check.name, detail = %check.detail, "environment check passed");
        } else {
            tracing::warn!(
                check = check.name,
                detail = %check.detail,
                code = check.code.unwrap_or("E_INTERNAL_ERROR"),
                "environment check failed"
            );
        }
    }

    let state = Arc::new(AppState::new(
        database,
        checks,
        Arc::new(config.clone()),
    ));

    let listener = match TcpListener::bind(config.bind).await {
        Ok(listener) => listener,
        Err(error) => {
            eprintln!(
                "hiverouter: E_WS_DISCONNECTED: cannot bind {}: {error}",
                config.bind
            );
            return Err(ExitCode::FAILURE);
        }
    };
    let bound = match listener.local_addr() {
        Ok(address) => address,
        Err(error) => {
            eprintln!("hiverouter: cannot read the bound address: {error}");
            return Err(ExitCode::FAILURE);
        }
    };
    println!(
        "hiverouter {} listening on http://{bound} (db {}, schema v{schema_version}, contract {})",
        env!("CARGO_PKG_VERSION"),
        config.db_file().display(),
        hiverouter_contract::SCHEMA_VERSION
    );
    tracing::info!(
        address = %bound,
        db = %config.db_file().display(),
        schema_version,
        "server booted"
    );

    match http::serve(listener, state, shutdown_signal()).await {
        Ok(()) => {
            tracing::info!("server stopped cleanly");
            Ok(())
        }
        Err(error) => {
            eprintln!("hiverouter: server error: {error}");
            Err(ExitCode::FAILURE)
        }
    }
}

async fn shutdown_signal() {
    if let Err(error) = tokio::signal::ctrl_c().await {
        tracing::warn!(%error, "could not install Ctrl+C handler");
        return;
    }
    tracing::info!("shutdown signal received");
}

fn init_tracing() {
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_target(false)
        .init();
}

fn print_usage() {
    println!(
        "hiverouter {version} — HiveMind v1 backend (contract schema {schema})

USAGE:
    hiverouter [--bind <ADDR>] [--data-dir <DIR>] [--db <FILE>]

OPTIONS:
    --bind <ADDR>      loopback listen address (default {default_bind})
    --data-dir <DIR>   data directory for SQLite and snapshots (default {default_data_dir})
    --db <FILE>        SQLite database file (default <data-dir>/hiverouter.db)
    -h, --help         show this help
    -V, --version      show the version and contract schema

ENVIRONMENT:
    HIVEROUTER_BIND       same as --bind
    HIVEROUTER_DATA_DIR   same as --data-dir
    HIVEROUTER_DB         same as --db
    RUST_LOG              tracing filter (default: info)

Only loopback bind addresses are accepted in v1; the server refuses anything else
with E_ORIGIN_REJECTED.",
        version = env!("CARGO_PKG_VERSION"),
        schema = hiverouter_contract::SCHEMA_VERSION,
        default_bind = hiverouter_core::config::DEFAULT_BIND,
        default_data_dir = hiverouter_core::config::DEFAULT_DATA_DIR,
    );
}
