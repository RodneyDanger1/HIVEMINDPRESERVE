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
use tokio_util::sync::CancellationToken;

use hiverouter_core::daemon::DaemonStore;
use hiverouter_core::remote::RemoteStore;
use hiverouter_core::remote_supervisor::RemoteSupervisor;

#[tokio::main]
async fn main() -> ExitCode {
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

    let _audit_guard = match hiverouter_tools::init_json_audit(config.data_dir.join("audit")) {
        Ok(guard) => guard,
        Err(error) => {
            eprintln!("hiverouter: cannot initialise the JSON audit sink: {error}");
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
    let remote_boot = RemoteStore::new(database.clone());
    if let Err(refusal) = remote_boot.reconcile_on_boot() {
        // Reconciliation is the only thing that stops a crash-stale `executing`
        // action or an abandoned approval from locking a device forever. A
        // silent warning would start the server on an unreconciled control
        // plane; fail the boot with the named code instead.
        eprintln!(
            "hiverouter: remote boot reconciliation failed: {} ({}); refusing to start on an unreconciled control plane",
            refusal.message(),
            refusal.code()
        );
        return Err(ExitCode::FAILURE);
    }
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

    let mut checks = run_checks();
    if let Some(key_check) = hiverouter_core::checks::provider_key_check_for(
        config.provider.enabled,
        config.provider.key_present(),
        config.provider.key_env_name(),
    ) {
        checks.push(key_check);
    }
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

    let graph = Arc::new(graph);
    let state = Arc::new(AppState::with_graph(
        database,
        checks,
        Arc::new(config.clone()),
        graph,
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

    let shutdown = CancellationToken::new();
    let supervisor = RemoteSupervisor::new(
        state.db.clone(),
        Arc::new(RemoteStore::new(state.db.clone())),
        Arc::new(DaemonStore::new(state.db.clone()).with_script_root(config.workspace_root.clone())),
        std::time::Duration::from_secs(2),
        shutdown.clone(),
    );
    let supervisor_handle = tokio::spawn(async move { supervisor.run().await });

    let shutdown_for_serve = shutdown.clone();
    let server_result = http::serve(listener, state, async move {
        tokio::select! {
            _ = shutdown_signal() => {}
            _ = shutdown_for_serve.cancelled() => {}
        }
    })
    .await;
    shutdown.cancel();
    let _ = supervisor_handle.await;
    match server_result {
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


fn print_usage() {
    println!(
        "hiverouter {version} — HiveMind v1 backend (contract schema {schema})

USAGE:
    hiverouter [--bind <ADDR>] [--data-dir <DIR>] [--db <FILE>] [--workspace-root <DIR>]

OPTIONS:
    --bind <ADDR>           loopback listen address (default {default_bind})
    --data-dir <DIR>        data directory for SQLite and snapshots (default {default_data_dir})
    --db <FILE>             SQLite database file (default <data-dir>/hiverouter.db)
    --workspace-root <DIR>  workspace jail for file tools (default: current directory)
    -h, --help              show this help
    -V, --version           show the version and contract schema

ENVIRONMENT:
    HIVEROUTER_BIND           same as --bind
    HIVEROUTER_DATA_DIR       same as --data-dir
    HIVEROUTER_DB             same as --db
    HIVEROUTER_WORKSPACE_ROOT same as --workspace-root
    RUST_LOG                  tracing filter (default: info)

Only loopback bind addresses are accepted in v1; the server refuses anything else
with E_ORIGIN_REJECTED.",
        version = env!("CARGO_PKG_VERSION"),
        schema = hiverouter_contract::SCHEMA_VERSION,
        default_bind = hiverouter_core::config::DEFAULT_BIND,
        default_data_dir = hiverouter_core::config::DEFAULT_DATA_DIR,
    );
}
