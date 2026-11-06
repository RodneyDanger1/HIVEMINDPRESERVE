//! HiveMind v1 tools crate — Phase 6 (WPs 6.1–6.14) + Phase 11 support modules.
//!
//! One owner for the tool capability surface (CANONICAL C18):
//! - [`tool`] — tool ids, per-tier allowlists, permission classes, schemars
//!   JSON schemas and jsonschema validation;
//! - [`approval`] — the single approval/consent gate (SQLite `UNIQUE(nonce)`,
//!   single-use decisions, TTL expiry, args binding, session scope);
//! - [`jail`] — the single Windows workspace jail (six escape vectors refused);
//! - [`shell`] — allowlisted child execution with stdout caps, timeout and a
//!   kill-on-close Job Object;
//! - [`cancel`] — per-run cancellation registry and kill enforcement;
//! - [`ledger`] — append-only JSONL run ledger (args hash, never arg body);
//! - [`sha256`] — the argument-binding digest.
//!
//! Phase 11/12 modules (`audit`, `health`, `census`, `watcher`) and the
//! `hiverouter` CLI binary are added by their phases in this same crate.
//!
//! Dependency direction: `hiverouter-core` depends on this crate; this crate
//! never depends on core, so core wires [`approval::migrate`], the registry and
//! the ledger into its server.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod approval;
pub mod audit;
pub mod cancel;
pub mod census;
pub mod cli;
pub mod cli_support;
pub mod error;
pub mod health;
pub mod jail;
pub mod ledger;
pub mod sha256;
pub mod shell;
pub mod tool;
pub mod watcher;

pub use approval::{
    hash_args, migrate as migrate_approvals, session_allowed, ApprovalDecision, ApprovalGate,
    ApprovalRequest, ApprovalResolution, APPROVAL_TTL_SECS,
};
pub use audit::{
    init_json_audit, probe_audit_dir, probe_audit_file, read_audit_rows, record_refusal,
    record_tool_call, AuditGuard, AuditRow,
};
pub use cancel::{ProcessRegistry, TerminalOutcome, TerminalState};
pub use census::{
    check_census_drift, derive_census, standard_census, CapabilityCensus, PROMPT_CLAIMED_TOOL_IDS,
};
pub use cli::{CliFailure, CliOutcome, CommandSpec, COMMANDS};
pub use error::ToolError;
pub use health::{ready_state, router as health_router, DiagnoseReport, HealthReport, ServiceState};
pub use jail::{DirEntryInfo, WorkspaceJail, MAX_READ_BYTES};
pub use ledger::{LedgerOutcome, LedgerRecord, LedgerRow, RunLedger, LEDGER_FILE_NAME};
pub use sha256::sha256_hex;
pub use shell::{
    execute as execute_shell, ShellOutput, ShellPolicy, ShellRequest, DEFAULT_MAX_STDERR_BYTES,
    DEFAULT_MAX_STDOUT_BYTES, DEFAULT_TIMEOUT,
};
pub use tool::{
    AgentTier, FinishArgs, FileListArgs, FileReadArgs, FileWriteArgs, PermissionClass,
    ReportResultArgs, ShellExecArgs, SpawnAgentArgs, ToolContract, ToolId, ToolRegistry,
};
pub use watcher::{FsChanged, WorkspaceWatcher, WATCH_DEBOUNCE};
