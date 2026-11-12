//! Background job registry (CAP-5).
//!
//! One owner for explicit, allowlisted background child processes (operator/HTTP initiated; not an approval-gated tool):
//!
//! - a job is identified by a uuid-v7 id and carries its `session_id`, program,
//!   direct argument vector, workspace-relative working directory, state, exit
//!   code, pid and a bounded output tail;
//! - **no shell interpolation**: the program is executed directly
//!   ([`tokio::process::Command`] + `args`), command-interpreter hosts
//!   (`cmd`, `powershell`, `sh`, `wsl`, ...) and path-qualified programs are
//!   refused before spawn, and the child environment is sanitised through
//!   [`hiverouter_tools::env_safety`] so provider credentials do not leak;
//! - the working directory is resolved through the workspace
//!   [`WorkspaceJail`] (parent traversal / absolute / UNC / symlink escapes are
//!   refused, never clamped);
//! - at most [`MAX_CONCURRENT_JOBS`] (4) jobs run at once; the fifth is refused
//!   with `E_SEMAPHORE_FULL` rather than queued invisibly;
//! - each job has a timeout; on timeout or kill the whole process tree is
//!   terminated through `taskkill /PID <pid> /T /F` (the same direct-argument
//!   pattern `daemon.rs` uses) plus `Child::kill`, and `kill_on_drop(true)` keeps
//!   a dropped supervisor from orphaning the direct child;
//! - [`JobRegistry::cleanup_session`] kills and removes every job owned by one
//!   session; dropping the registry best-effort kills anything still running.
//!
//! # Containment truth
//!
//! `hiverouter-core` does not depend on `win32job`, so this module cannot create
//! a Job Object itself. The kill-tree path is the existing `taskkill /T /F`
//! pattern: it reaps the direct child and its descendants *at kill time*. A
//! grandchild that detaches itself from the tree before the kill is outside that
//! guarantee, exactly as it is for `daemon.rs`. When the Lead adds `win32job` to
//! this crate, [`kill_tree`] is the single place to swap in a kill-on-close Job
//! Object for stronger containment.

use std::collections::{HashMap, VecDeque};
use std::path::Path;
use std::sync::{Arc, Mutex, MutexGuard};
use std::time::Duration;

use hiverouter_contract::{Refusal, Timestamp};
use rusqlite::{params, Connection};
use hiverouter_tools::env_safety;
use hiverouter_tools::shell::is_shell_interpreter;
use hiverouter_tools::{ToolError, WorkspaceJail};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tokio::io::{AsyncRead, AsyncReadExt};
use tokio::process::{Child, Command};
use tokio::sync::{watch, OwnedSemaphorePermit, Semaphore};
use tokio_util::sync::CancellationToken;

/// Latest-state job snapshot schema (v28). Terminal snapshots survive restart;
/// the in-memory registry remains the live authority for running jobs.
pub const SCHEMA_V28: &str = r#"
CREATE TABLE IF NOT EXISTS jobs (
    id           TEXT PRIMARY KEY,
    session_id   TEXT NOT NULL,
    program      TEXT NOT NULL,
    args_json    TEXT NOT NULL,
    cwd          TEXT NOT NULL,
    state        TEXT NOT NULL CHECK (state IN ('running','succeeded','failed','timed_out','killed')),
    pid          INTEGER,
    exit_code    INTEGER,
    started_at   TEXT NOT NULL,
    ended_at     TEXT,
    timeout_ms   INTEGER NOT NULL,
    output_tail  TEXT NOT NULL,
    output_bytes INTEGER NOT NULL,
    truncated    INTEGER NOT NULL CHECK (truncated IN (0,1))
);
CREATE INDEX IF NOT EXISTS idx_jobs_session ON jobs (session_id, started_at);
"#;

/// Down migration for v28.
pub const SCHEMA_V28_DOWN: &str = "DROP INDEX IF EXISTS idx_jobs_session; DROP TABLE IF EXISTS jobs;";

/// Migration descriptor installed by [`crate::db::MIGRATIONS`].
#[must_use]
pub const fn job_migration(version: i64) -> crate::db::Migration {
    crate::db::Migration {
        version,
        name: "jobs",
        up: SCHEMA_V28,
        down: SCHEMA_V28_DOWN,
    }
}

/// Marks any snapshot left `running` by a previous process as failed. A new
/// process cannot own the old child handle; pretending the job still runs would
/// be a fabricated success. Returns the number of reconciled rows.
pub fn reconcile_running_jobs(connection: &Connection) -> rusqlite::Result<usize> {
    // The old process may have died before or after the child exited. Keep the
    // row's recorded bytes/tail unchanged and leave ended_at NULL: the true
    // outcome and end time are unknown. `failed` means "no owner after restart",
    // not a measured exit.
    connection.execute(
        "UPDATE jobs SET state = 'failed', ended_at = NULL
         WHERE state = 'running'",
        [],
    )
}

/// Persists one terminal (or live) snapshot. Upsert by id is idempotent.
pub fn persist_job(connection: &Connection, snapshot: &JobSnapshot) -> rusqlite::Result<()> {
    let args_json = serde_json::to_string(&snapshot.args).map_err(|error| {
        rusqlite::Error::ToSqlConversionFailure(Box::new(error))
    })?;
    connection.execute(
        "INSERT INTO jobs(id, session_id, program, args_json, cwd, state, pid, exit_code,
                          started_at, ended_at, timeout_ms, output_tail, output_bytes, truncated)
         VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14)
         ON CONFLICT(id) DO UPDATE SET
             state=excluded.state, pid=excluded.pid, exit_code=excluded.exit_code,
             ended_at=excluded.ended_at, output_tail=excluded.output_tail,
             output_bytes=excluded.output_bytes, truncated=excluded.truncated",
        params![
            snapshot.id,
            snapshot.session_id,
            snapshot.program,
            args_json,
            snapshot.cwd,
            snapshot.state.as_str(),
            snapshot.pid.map(i64::from),
            snapshot.exit_code.map(i64::from),
            snapshot.started_at.as_str(),
            snapshot.ended_at.as_ref().map(Timestamp::as_str),
            i64::try_from(snapshot.timeout_ms).unwrap_or(i64::MAX),
            snapshot.output.tail,
            i64::try_from(snapshot.output.bytes).unwrap_or(i64::MAX),
            if snapshot.output.truncated { 1 } else { 0 },
        ],
    )?;
    Ok(())
}

/// Reads persisted snapshots, newest first.
pub fn list_persisted_jobs(
    connection: &Connection,
    limit: u32,
) -> Result<Vec<JobSnapshot>, rusqlite::Error> {
    let mut statement = connection.prepare(
        "SELECT id, session_id, program, args_json, cwd, state, pid, exit_code,
                started_at, ended_at, timeout_ms, output_tail, output_bytes, truncated
         FROM jobs ORDER BY started_at DESC, id DESC LIMIT ?1",
    )?;
    let rows = statement.query_map(params![i64::from(limit)], |row| {
        let state: String = row.get(5)?;
        let started: String = row.get(8)?;
        let ended: Option<String> = row.get(9)?;
        let args_json: String = row.get(3)?;
        let args: Vec<String> = serde_json::from_str(&args_json).map_err(|error| {
            rusqlite::Error::FromSqlConversionFailure(3, rusqlite::types::Type::Text, Box::new(error))
        })?;
        Ok(JobSnapshot {
            id: row.get(0)?,
            session_id: row.get(1)?,
            program: row.get(2)?,
            args,
            cwd: row.get(4)?,
            state: parse_job_state(&state).ok_or_else(|| {
                rusqlite::Error::InvalidColumnType(5, "state".to_owned(), rusqlite::types::Type::Text)
            })?,
            pid: row.get::<_, Option<i64>>(6)?.and_then(|value| u32::try_from(value).ok()),
            exit_code: row.get::<_, Option<i64>>(7)?.and_then(|value| i32::try_from(value).ok()),
            started_at: Timestamp::parse(&started).map_err(|_| {
                rusqlite::Error::InvalidColumnType(8, "started_at".to_owned(), rusqlite::types::Type::Text)
            })?,
            ended_at: match ended {
                Some(raw) => Some(Timestamp::parse(&raw).map_err(|_| {
                    rusqlite::Error::InvalidColumnType(9, "ended_at".to_owned(), rusqlite::types::Type::Text)
                })?),
                None => None,
            },
            timeout_ms: u64::try_from(row.get::<_, i64>(10)?).unwrap_or(0),
            output: JobOutput {
                tail: row.get(11)?,
                bytes: u64::try_from(row.get::<_, i64>(12)?).unwrap_or(0),
                truncated: row.get::<_, i64>(13)? != 0,
            },
        })
    })?;
    rows.collect()
}

fn parse_job_state(raw: &str) -> Option<JobState> {
    match raw {
        "running" => Some(JobState::Running),
        "succeeded" => Some(JobState::Succeeded),
        "failed" => Some(JobState::Failed),
        "timed_out" => Some(JobState::TimedOut),
        "killed" => Some(JobState::Killed),
        _ => None,
    }
}

/// Hard concurrency cap: at most four background jobs run at once.
pub const MAX_CONCURRENT_JOBS: usize = 4;

/// Default per-job timeout (5 minutes).
pub const DEFAULT_JOB_TIMEOUT: Duration = Duration::from_secs(300);

/// Default output tail retained per job (64 KiB).
pub const DEFAULT_OUTPUT_TAIL_BYTES: usize = 64 * 1024;

/// Hard bound on the retained output tail (1 MiB).
pub const MAX_OUTPUT_TAIL_BYTES: usize = 1024 * 1024;

/// Maximum number of direct arguments accepted for one job.
pub const MAX_ARGUMENTS: usize = 256;

/// Maximum bytes in one direct argument.
pub const MAX_ARGUMENT_BYTES: usize = 8192;

/// Maximum bytes in a session id.
pub const MAX_SESSION_ID_BYTES: usize = 160;

/// How long [`JobRegistry::kill`] waits for the state transition.
const KILL_GRACE: Duration = Duration::from_secs(10);

/// How long a supervisor waits for the output pumps after the child exits.
const READER_GRACE: Duration = Duration::from_secs(2);

/// Lifecycle state of one background job.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum JobState {
    /// The child is running.
    Running,
    /// The child exited with code 0.
    Succeeded,
    /// The child exited non-zero, or its wait failed.
    Failed,
    /// The job exceeded its timeout and was terminated.
    TimedOut,
    /// The job was killed through [`JobRegistry::kill`] or a session cleanup.
    Killed,
}

impl JobState {
    /// Stable wire label.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Running => "running",
            Self::Succeeded => "succeeded",
            Self::Failed => "failed",
            Self::TimedOut => "timed_out",
            Self::Killed => "killed",
        }
    }

    /// Whether the job can no longer change state.
    #[must_use]
    pub fn is_terminal(self) -> bool {
        !matches!(self, Self::Running)
    }
}

/// One job start request.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct JobRequest {
    /// Session that owns the job (used by [`JobRegistry::cleanup_session`]).
    pub session_id: String,
    /// Bare executable name (allowlisted; no path separators, no interpreter).
    pub program: String,
    /// Direct argument vector (never a shell command string).
    pub args: Vec<String>,
    /// Workspace-relative working directory (`None` = workspace root).
    pub cwd: Option<String>,
}

impl JobRequest {
    /// Builds a request.
    #[must_use]
    pub fn new(
        session_id: impl Into<String>,
        program: impl Into<String>,
        args: Vec<String>,
    ) -> Self {
        Self {
            session_id: session_id.into(),
            program: program.into(),
            args,
            cwd: None,
        }
    }

    /// Sets the workspace-relative working directory.
    #[must_use]
    pub fn with_cwd(mut self, cwd: impl Into<String>) -> Self {
        self.cwd = Some(cwd.into());
        self
    }
}

/// Program allowlist plus resource bounds for the registry.
#[derive(Clone, Debug)]
pub struct JobPolicy {
    allowed_programs: Vec<String>,
    timeout: Duration,
    output_tail_bytes: usize,
}

impl JobPolicy {
    /// Builds a policy from an allowlist of bare executable names.
    ///
    /// Names are trimmed, lowercased, sorted and de-duplicated; empty entries are
    /// dropped. Command-interpreter hosts remain denied even when listed (see
    /// [`JobPolicy::allows`]).
    #[must_use]
    pub fn new(allowed_programs: impl IntoIterator<Item = String>) -> Self {
        let mut allowed: Vec<String> = allowed_programs
            .into_iter()
            .map(|program| program.trim().to_lowercase())
            .filter(|program| !program.is_empty())
            .collect();
        allowed.sort();
        allowed.dedup();
        Self {
            allowed_programs: allowed,
            timeout: DEFAULT_JOB_TIMEOUT,
            output_tail_bytes: DEFAULT_OUTPUT_TAIL_BYTES,
        }
    }

    /// Default allowlist for v1 developer utilities.
    #[must_use]
    pub fn conservative_default() -> Self {
        Self::new(["git.exe".to_owned(), "cargo.exe".to_owned()])
    }

    /// Overrides the per-job timeout (clamped to at least 1 ms).
    #[must_use]
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout.max(Duration::from_millis(1));
        self
    }

    /// Sets the retained output-tail bound (clamped to
    /// `1..=`[`MAX_OUTPUT_TAIL_BYTES`]).
    #[must_use]
    pub fn with_output_tail_bytes(mut self, output_tail_bytes: usize) -> Self {
        self.output_tail_bytes = output_tail_bytes.clamp(1, MAX_OUTPUT_TAIL_BYTES);
        self
    }

    /// The allowlist.
    #[must_use]
    pub fn allowed_programs(&self) -> &[String] {
        &self.allowed_programs
    }

    /// Configured timeout.
    #[must_use]
    pub fn timeout(&self) -> Duration {
        self.timeout
    }

    /// Configured output-tail bound.
    #[must_use]
    pub fn output_tail_bytes(&self) -> usize {
        self.output_tail_bytes
    }

    /// Whether a program is an allowlisted bare executable.
    ///
    /// Path-qualified names (`\`, `/`, `:`), empty names and command-interpreter
    /// hosts are refused regardless of the allowlist.
    #[must_use]
    pub fn allows(&self, program: &str) -> bool {
        let program = program.trim();
        if program.is_empty()
            || program.contains('\\')
            || program.contains('/')
            || program.contains(':')
        {
            return false;
        }
        if is_shell_interpreter(program) {
            return false;
        }
        let needle = program.to_lowercase();
        self.allowed_programs
            .iter()
            .any(|allowed| allowed == &needle)
    }
}

/// Bounded output tail plus total byte accounting for one job.
#[derive(Debug)]
struct TailBuffer {
    bytes: VecDeque<u8>,
    cap: usize,
    total: u64,
    truncated: bool,
}

impl TailBuffer {
    fn new(cap: usize) -> Self {
        let cap = cap.clamp(1, MAX_OUTPUT_TAIL_BYTES);
        Self {
            bytes: VecDeque::with_capacity(cap.min(8192)),
            cap,
            total: 0,
            truncated: false,
        }
    }

    fn append(&mut self, chunk: &[u8]) {
        self.total = self.total.saturating_add(chunk.len() as u64);
        if chunk.len() >= self.cap {
            self.bytes.clear();
            let start = chunk.len() - self.cap;
            self.bytes.extend(chunk[start..].iter().copied());
            self.truncated = true;
            return;
        }
        // usize arithmetic must never underflow: the old expression subtracted
        // `cap` from bytes+chunk even when the buffer was smaller than the cap,
        // panicking (debug) or wrapping (release) on the first ordinary chunk.
        let overflow = self
            .bytes
            .len()
            .saturating_add(chunk.len())
            .saturating_sub(self.cap);
        if overflow > 0 {
            let drop_count = overflow.min(self.bytes.len());
            self.bytes.drain(..drop_count);
            self.truncated = true;
        }
        self.bytes.extend(chunk.iter().copied());
    }

    fn snapshot(&self) -> JobOutput {
        let bytes: Vec<u8> = self.bytes.iter().copied().collect();
        JobOutput {
            tail: String::from_utf8_lossy(&bytes).into_owned(),
            bytes: self.total,
            truncated: self.truncated,
        }
    }
}

/// Bounded capture of a job's combined stdout/stderr.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct JobOutput {
    /// Last `cap` bytes of combined output (lossy UTF-8, never the whole stream).
    pub tail: String,
    /// Total bytes observed on stdout + stderr (before truncation).
    pub bytes: u64,
    /// The tail dropped earlier bytes.
    pub truncated: bool,
}

/// Point-in-time view of one job.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct JobSnapshot {
    /// Job id (uuid-v7).
    pub id: String,
    /// Owning session.
    pub session_id: String,
    /// Program executed.
    pub program: String,
    /// Direct arguments passed.
    pub args: Vec<String>,
    /// Workspace-relative working directory (`"."` for the root).
    pub cwd: String,
    /// Current lifecycle state.
    pub state: JobState,
    /// OS process id while known.
    pub pid: Option<u32>,
    /// Exit code once the child exited (`None` when it never started or was
    /// terminated before an exit code was available).
    pub exit_code: Option<i32>,
    /// Start timestamp.
    pub started_at: Timestamp,
    /// Terminal timestamp (`None` while running).
    pub ended_at: Option<Timestamp>,
    /// Per-job timeout in milliseconds.
    pub timeout_ms: u64,
    /// Bounded output capture.
    pub output: JobOutput,
}

/// Job registry failure with its stable refusal identity.
#[derive(Debug, Error)]
pub enum JobError {
    /// A validated request was refused by the registry (program/jail/schema).
    #[error("{refusal}: {detail}")]
    Refused {
        /// Stable refusal identity.
        refusal: Refusal,
        /// Non-secret detail.
        detail: String,
    },
    /// The concurrency cap is full; the job was not started.
    #[error(
        "E_SEMAPHORE_FULL: the job registry already runs {cap} jobs; \
         the new job was refused (it is never silently queued)"
    )]
    ConcurrencyLimit {
        /// The cap (`MAX_CONCURRENT_JOBS`).
        cap: usize,
    },
    /// No job with this id is registered.
    #[error("job `{0}` is not registered")]
    UnknownJob(String),
    /// A killed job did not reach a terminal state within the grace window.
    #[error("E_TOOL_TIMEOUT: job `{0}` did not stop within the kill grace window")]
    KillTimeout(String),
}

impl JobError {
    /// Maps the error to the stable wire refusal.
    #[must_use]
    pub fn refusal(&self) -> Refusal {
        match self {
            Self::Refused { refusal, .. } => *refusal,
            Self::ConcurrencyLimit { .. } => Refusal::SemaphoreFull,
            Self::UnknownJob(_) => Refusal::JobNotFound,
            Self::KillTimeout(_) => Refusal::ToolTimeout,
        }
    }

    /// Stable wire code for the error.
    #[must_use]
    pub fn code(&self) -> &'static str {
        self.refusal().code()
    }
}

/// One live job entry.
#[derive(Debug)]
struct JobEntry {
    id: String,
    session_id: String,
    program: String,
    args: Vec<String>,
    cwd: String,
    state: JobState,
    pid: Option<u32>,
    exit_code: Option<i32>,
    started_at: Timestamp,
    ended_at: Option<Timestamp>,
    timeout: Duration,
    cancel: CancellationToken,
    state_tx: watch::Sender<JobState>,
    output: Arc<Mutex<TailBuffer>>,
}

/// Shared registry state (also held by each running supervisor).
struct RegistryInner {
    jail: WorkspaceJail,
    policy: JobPolicy,
    slots: Arc<Semaphore>,
    jobs: Mutex<HashMap<String, JobEntry>>,
    /// Optional durable sink: terminal snapshots are written here by the
    /// supervisor so a naturally-completed job is not mistaken for a crash
    /// orphan after restart.
    persistence: Mutex<Option<Arc<crate::db::Database>>>,
}

impl std::fmt::Debug for RegistryInner {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("RegistryInner")
            .field("workspace_root", &self.jail.root())
            .field("policy", &self.policy)
            .finish_non_exhaustive()
    }
}

impl RegistryInner {
    fn jobs(&self) -> MutexGuard<'_, HashMap<String, JobEntry>> {
        self.jobs
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }
}

impl Drop for RegistryInner {
    fn drop(&mut self) {
        // Best effort: a supervisor normally terminates its own child, but a
        // runtime shutdown can drop the supervisor future. Kill any process that
        // is still marked running so the registry never leaks a child.
        let jobs = self
            .jobs
            .get_mut()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        for entry in jobs.values() {
            if entry.state != JobState::Running {
                continue;
            }
            if let Some(pid) = entry.pid {
                kill_pid_best_effort(pid);
            }
        }
    }
}

/// Background job registry: bounded concurrency, timeout, output tail, kill and
/// per-session cleanup.
#[derive(Clone, Debug)]
pub struct JobRegistry {
    inner: Arc<RegistryInner>,
}

impl JobRegistry {
    /// Opens (creating if needed) a workspace root and builds a registry.
    ///
    /// # Errors
    /// [`JobError::Refused`] when the workspace root cannot be opened as a jail.
    pub fn open(workspace_root: impl AsRef<Path>, policy: JobPolicy) -> Result<Self, JobError> {
        let jail = WorkspaceJail::open(workspace_root).map_err(map_tool_error)?;
        Ok(Self::with_jail(jail, policy))
    }

    /// Builds a registry over an existing workspace jail.
    #[must_use]
    pub fn with_jail(jail: WorkspaceJail, policy: JobPolicy) -> Self {
        Self {
            inner: Arc::new(RegistryInner {
                jail,
                policy,
                slots: Arc::new(Semaphore::new(MAX_CONCURRENT_JOBS)),
                jobs: Mutex::new(HashMap::new()),
                persistence: Mutex::new(None),
            }),
        }
    }

    /// Attaches the durable job store. Terminal snapshots are persisted by the
    /// supervisor; starting/killing also persist at the HTTP layer.
    #[must_use]
    pub fn with_persistence(self, db: Arc<crate::db::Database>) -> Self {
        if let Ok(mut slot) = self.inner.persistence.lock() {
            *slot = Some(db);
        }
        self
    }

    /// The workspace jail used for every job working directory.
    #[must_use]
    pub fn jail(&self) -> &WorkspaceJail {
        &self.inner.jail
    }

    /// The policy in force.
    #[must_use]
    pub fn policy(&self) -> &JobPolicy {
        &self.inner.policy
    }

    /// Starts one background job.
    ///
    /// # Errors
    /// [`JobError::Refused`] for a non-allowlisted/interpreter/path-qualified
    /// program, an escaping or non-directory working directory, or an invalid
    /// request; [`JobError::ConcurrencyLimit`] when four jobs are already
    /// running; [`JobError::Refused`] with `E_INTERNAL_ERROR` when the child
    /// cannot be spawned.
    pub async fn start(&self, request: JobRequest) -> Result<JobSnapshot, JobError> {
        self.validate_request(&request)?;
        let root = self.inner.jail.root().to_path_buf();
        let cwd_path = match request.cwd.as_deref() {
            Some(raw) => {
                let resolved = self.inner.jail.resolve(raw).map_err(map_tool_error)?;
                if !resolved.is_dir() {
                    return Err(refused(
                        Refusal::ToolSchema,
                        format!("job cwd {raw:?} is not a directory inside the workspace"),
                    ));
                }
                resolved
            }
            None => root.clone(),
        };
        let cwd_label = relative_label(&root, &cwd_path);

        let permit = self.inner.slots.clone().try_acquire_owned().map_err(|_| {
            JobError::ConcurrencyLimit {
                cap: MAX_CONCURRENT_JOBS,
            }
        })?;

        let timeout = self.inner.policy.timeout();
        let tail_bytes = self.inner.policy.output_tail_bytes();
        let mut command = Command::new(&request.program);
        command
            .args(&request.args)
            .current_dir(&cwd_path)
            .stdin(std::process::Stdio::null())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .kill_on_drop(true);
        env_safety::sanitize_tokio(&mut command);
        let mut child = command.spawn().map_err(|error| {
            refused(
                Refusal::InternalError,
                format!("failed to spawn job program {:?}: {error}", request.program),
            )
        })?;
        let pid = child.id();
        let stdout = match child.stdout.take() {
            Some(stdout) => stdout,
            None => {
                drop(permit);
                let _ = child.kill().await;
                return Err(refused(
                    Refusal::InternalError,
                    "job stdout pipe is unavailable",
                ));
            }
        };
        let stderr = match child.stderr.take() {
            Some(stderr) => stderr,
            None => {
                drop(permit);
                let _ = child.kill().await;
                return Err(refused(
                    Refusal::InternalError,
                    "job stderr pipe is unavailable",
                ));
            }
        };

        let id = uuid::Uuid::now_v7().to_string();
        let output = Arc::new(Mutex::new(TailBuffer::new(tail_bytes)));
        let cancel = CancellationToken::new();
        let (state_tx, _state_rx) = watch::channel(JobState::Running);
        let entry = JobEntry {
            id: id.clone(),
            session_id: request.session_id,
            program: request.program,
            args: request.args,
            cwd: cwd_label,
            state: JobState::Running,
            pid,
            exit_code: None,
            started_at: Timestamp::now(),
            ended_at: None,
            timeout,
            cancel: cancel.clone(),
            state_tx: state_tx.clone(),
            output: Arc::clone(&output),
        };
        let snapshot = snapshot_of(&entry);
        self.inner.jobs().insert(id.clone(), entry);
        let _supervisor = tokio::spawn(supervise(
            Arc::clone(&self.inner),
            child,
            stdout,
            stderr,
            id,
            cancel,
            state_tx,
            output,
            timeout,
            permit,
        ));
        Ok(snapshot)
    }

    /// Reads the current snapshot of one job.
    ///
    /// # Errors
    /// [`JobError::UnknownJob`] when the id is not registered.
    pub fn get(&self, id: &str) -> Result<JobSnapshot, JobError> {
        let jobs = self.inner.jobs();
        let entry = jobs
            .get(id)
            .ok_or_else(|| JobError::UnknownJob(id.to_owned()))?;
        Ok(snapshot_of(entry))
    }

    /// Waits for one job to reach a terminal state and returns its snapshot.
    ///
    /// # Errors
    /// [`JobError::UnknownJob`] when the id is not registered (or its state
    /// channel closed).
    pub async fn wait(&self, id: &str) -> Result<JobSnapshot, JobError> {
        let mut receiver = {
            let jobs = self.inner.jobs();
            let entry = jobs
                .get(id)
                .ok_or_else(|| JobError::UnknownJob(id.to_owned()))?;
            if entry.state.is_terminal() {
                return Ok(snapshot_of(entry));
            }
            entry.state_tx.subscribe()
        };
        receiver
            .wait_for(|state| state.is_terminal())
            .await
            .map_err(|_| JobError::UnknownJob(id.to_owned()))?;
        self.get(id)
    }

    /// Kills one job (idempotent for an already terminal job).
    ///
    /// Cancelling is cooperative with the supervisor, which terminates the
    /// process tree and records [`JobState::Killed`]. If the job completed before
    /// the cancel took effect, its real terminal state is returned.
    ///
    /// # Errors
    /// [`JobError::UnknownJob`] for an unknown id,
    /// [`JobError::KillTimeout`] when the job does not stop within the grace
    /// window.
    pub async fn kill(&self, id: &str) -> Result<JobSnapshot, JobError> {
        let (cancel, mut receiver) = {
            let jobs = self.inner.jobs();
            let entry = jobs
                .get(id)
                .ok_or_else(|| JobError::UnknownJob(id.to_owned()))?;
            if entry.state.is_terminal() {
                return Ok(snapshot_of(entry));
            }
            (entry.cancel.clone(), entry.state_tx.subscribe())
        };
        cancel.cancel();
        let stopped =
            tokio::time::timeout(KILL_GRACE, receiver.wait_for(|state| state.is_terminal())).await;
        match stopped {
            Ok(Ok(_)) => self.get(id),
            Ok(Err(_)) => Err(JobError::UnknownJob(id.to_owned())),
            Err(_) => Err(JobError::KillTimeout(id.to_owned())),
        }
    }

    /// Every job owned by one session, ordered by start time then id.
    #[must_use]
    pub fn list_session(&self, session_id: &str) -> Vec<JobSnapshot> {
        let jobs = self.inner.jobs();
        let mut snapshots: Vec<JobSnapshot> = jobs
            .values()
            .filter(|entry| entry.session_id == session_id)
            .map(snapshot_of)
            .collect();
        snapshots.sort_by(|left, right| {
            left.started_at
                .cmp(&right.started_at)
                .then_with(|| left.id.cmp(&right.id))
        });
        snapshots
    }

    /// Every registered job, ordered by start time then id.
    #[must_use]
    pub fn list_all(&self) -> Vec<JobSnapshot> {
        let jobs = self.inner.jobs();
        let mut snapshots: Vec<JobSnapshot> = jobs.values().map(snapshot_of).collect();
        snapshots.sort_by(|left, right| {
            left.started_at
                .cmp(&right.started_at)
                .then_with(|| left.id.cmp(&right.id))
        });
        snapshots
    }

    /// Number of jobs currently running.
    #[must_use]
    pub fn running_count(&self) -> usize {
        let jobs = self.inner.jobs();
        jobs.values()
            .filter(|entry| entry.state == JobState::Running)
            .count()
    }

    /// Kills and removes every job owned by one session.
    ///
    /// Jobs started for the same session *after* this call are not affected.
    /// Returns the number of removed entries.
    ///
    /// # Errors
    /// [`JobError::KillTimeout`] when a canceled job does not reach a terminal
    /// state within the grace window.
    pub async fn cleanup_session(&self, session_id: &str) -> Result<usize, JobError> {
        let mut pending: Vec<(String, CancellationToken, watch::Receiver<JobState>)> = Vec::new();
        {
            let jobs = self.inner.jobs();
            for entry in jobs.values() {
                if entry.session_id == session_id {
                    pending.push((
                        entry.id.clone(),
                        entry.cancel.clone(),
                        entry.state_tx.subscribe(),
                    ));
                }
            }
        }
        for (_, cancel, _) in &pending {
            cancel.cancel();
        }
        for (id, _, mut receiver) in pending {
            match tokio::time::timeout(KILL_GRACE, receiver.wait_for(|state| state.is_terminal()))
                .await
            {
                Ok(Ok(_)) | Ok(Err(_)) => {}
                Err(_) => return Err(JobError::KillTimeout(id)),
            }
        }
        let leftover: Vec<u32> = {
            let jobs = self.inner.jobs();
            jobs.values()
                .filter(|entry| entry.session_id == session_id)
                .filter(|entry| entry.state == JobState::Running)
                .filter_map(|entry| entry.pid)
                .collect()
        };
        for pid in leftover {
            // A supervisor that panicked before updating the entry leaves the
            // process running; never remove a registry entry while its child may
            // still be alive.
            kill_pid_best_effort(pid);
        }
        let mut jobs = self.inner.jobs();
        let before = jobs.len();
        jobs.retain(|_, entry| entry.session_id != session_id);
        Ok(before - jobs.len())
    }

    fn validate_request(&self, request: &JobRequest) -> Result<(), JobError> {
        let session = request.session_id.as_str();
        if session.is_empty() {
            return Err(refused(
                Refusal::ToolSchema,
                "job session_id must not be empty",
            ));
        }
        if session.len() > MAX_SESSION_ID_BYTES {
            return Err(refused(
                Refusal::ToolSchema,
                format!(
                    "job session_id is {} bytes; the cap is {MAX_SESSION_ID_BYTES}",
                    session.len()
                ),
            ));
        }
        if session.trim() != session || session.chars().any(char::is_control) {
            return Err(refused(
                Refusal::ToolSchema,
                "job session_id must not have surrounding whitespace or control characters",
            ));
        }
        if request.program.is_empty() || request.program.trim() != request.program {
            return Err(refused(
                Refusal::ToolSchema,
                "job program must be a non-empty bare executable name without surrounding whitespace",
            ));
        }
        if !self.inner.policy.allows(&request.program) {
            return Err(refused(
                Refusal::ToolNotAllowed,
                format!(
                    "program {:?} is not an allowlisted bare executable ({}); \
                     command-interpreter hosts and path-qualified programs are refused",
                    request.program,
                    self.inner.policy.allowed_programs().join(", ")
                ),
            ));
        }
        if request.args.len() > MAX_ARGUMENTS {
            return Err(refused(
                Refusal::ToolSchema,
                format!(
                    "job has {} arguments; the cap is {MAX_ARGUMENTS}",
                    request.args.len()
                ),
            ));
        }
        for argument in &request.args {
            if argument.contains('\0') {
                return Err(refused(
                    Refusal::ToolSchema,
                    "job argument contains a NUL byte",
                ));
            }
            if argument.len() > MAX_ARGUMENT_BYTES {
                return Err(refused(
                    Refusal::ToolSchema,
                    format!(
                        "job argument is {} bytes; the cap is {MAX_ARGUMENT_BYTES}",
                        argument.len()
                    ),
                ));
            }
        }
        Ok(())
    }
}

/// Supervisor: waits for one child, enforces timeout/cancel and records the
/// terminal state.
#[allow(clippy::too_many_arguments)]
async fn supervise(
    inner: Arc<RegistryInner>,
    mut child: Child,
    stdout: tokio::process::ChildStdout,
    stderr: tokio::process::ChildStderr,
    id: String,
    cancel: CancellationToken,
    state_tx: watch::Sender<JobState>,
    output: Arc<Mutex<TailBuffer>>,
    timeout: Duration,
    permit: OwnedSemaphorePermit,
) {
    let stop = CancellationToken::new();
    let out_task = tokio::spawn(pump(stdout, Arc::clone(&output), stop.clone()));
    let err_task = tokio::spawn(pump(stderr, Arc::clone(&output), stop.clone()));
    let out_abort = out_task.abort_handle();
    let err_abort = err_task.abort_handle();

    enum WaitOutcome {
        Cancelled,
        TimedOut,
        Exited(std::io::Result<std::process::ExitStatus>),
    }
    let outcome = tokio::select! {
        biased;
        () = cancel.cancelled() => WaitOutcome::Cancelled,
        () = tokio::time::sleep(timeout) => WaitOutcome::TimedOut,
        status = child.wait() => WaitOutcome::Exited(status),
    };
    let (state, exit_code) = match outcome {
        WaitOutcome::Exited(Ok(status)) => {
            let exit_code = status.code();
            let state = if status.success() {
                JobState::Succeeded
            } else {
                JobState::Failed
            };
            (state, exit_code)
        }
        WaitOutcome::Exited(Err(_)) => (JobState::Failed, None),
        WaitOutcome::Cancelled => {
            kill_tree(&mut child).await;
            let status = child.wait().await.ok();
            (JobState::Killed, status.and_then(|status| status.code()))
        }
        WaitOutcome::TimedOut => {
            kill_tree(&mut child).await;
            let status = child.wait().await.ok();
            (JobState::TimedOut, status.and_then(|status| status.code()))
        }
    };

    // Give the pumps a moment to drain EOF; if a detached grandchild still holds
    // the pipe, stop and abort them instead of leaking a task.
    let drain_stop = stop.clone();
    let drained = tokio::time::timeout(READER_GRACE, async move {
        let _ = out_task.await;
        let _ = err_task.await;
    })
    .await;
    if drained.is_err() {
        drain_stop.cancel();
        out_abort.abort();
        err_abort.abort();
    }

    let terminal_snapshot = {
        let mut jobs = inner.jobs();
        if let Some(entry) = jobs.get_mut(&id) {
            entry.state = state;
            entry.exit_code = exit_code;
            entry.ended_at = Some(Timestamp::now());
        }
        jobs.get(&id).map(snapshot_of)
    };
    if let Some(snapshot) = terminal_snapshot {
        let database = inner
            .persistence
            .lock()
            .ok()
            .and_then(|slot| slot.as_ref().cloned());
        if let Some(database) = database {
            if let Err(error) =
                database.with_connection(|connection| persist_job(connection, &snapshot))
            {
                tracing::warn!(
                    job_id = %snapshot.id,
                    "terminal job snapshot could not be persisted: {error}"
                );
            }
        }
    }
    // `send_replace` updates the stored value even when no waiter is subscribed
    // yet (a plain `send` fails with zero receivers and would lose the terminal
    // state for a `wait` that subscribes between the map update and this call).
    let _ = state_tx.send_replace(state);
    drop(permit);
}

/// Reads one pipe into the bounded tail until EOF or cancellation.
async fn pump<R: AsyncRead + Unpin>(
    mut reader: R,
    output: Arc<Mutex<TailBuffer>>,
    stop: CancellationToken,
) {
    let mut chunk = [0_u8; 8192];
    loop {
        tokio::select! {
            biased;
            () = stop.cancelled() => break,
            result = reader.read(&mut chunk) => match result {
                Ok(0) => break,
                Ok(read) => {
                    let mut buffer = output
                        .lock()
                        .unwrap_or_else(std::sync::PoisonError::into_inner);
                    buffer.append(&chunk[..read]);
                }
                Err(_) => break,
            },
        }
    }
}

/// Synchronous best-effort tree kill, used where awaiting is impossible
/// (`Drop`) or as a last resort after a supervisor failed to update its entry.
fn kill_pid_best_effort(pid: u32) {
    #[cfg(windows)]
    {
        let _ = std::process::Command::new("taskkill")
            .args(["/PID", &pid.to_string(), "/T", "/F"])
            .stdin(std::process::Stdio::null())
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status();
    }
    #[cfg(not(windows))]
    {
        let _ = pid;
    }
}

/// Terminates the child and its descendants.
///
/// Windows uses `taskkill /PID <pid> /T /F` with direct arguments (never a shell
/// string); other platforms fall back to `Child::kill` (no process-group
/// support is claimed there).
async fn kill_tree(child: &mut Child) {
    if let Some(pid) = child.id() {
        #[cfg(windows)]
        {
            let mut command = Command::new("taskkill");
            command
                .args(["/PID", &pid.to_string(), "/T", "/F"])
                .stdin(std::process::Stdio::null())
                .stdout(std::process::Stdio::null())
                .stderr(std::process::Stdio::null());
            // CREATE_NO_WINDOW: no console flash from a background server.
            command.creation_flags(0x0800_0000);
            let _ = command.status().await;
        }
        #[cfg(not(windows))]
        {
            let _ = pid;
        }
    }
    let _ = child.kill().await;
}

fn snapshot_of(entry: &JobEntry) -> JobSnapshot {
    let output = entry
        .output
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .snapshot();
    JobSnapshot {
        id: entry.id.clone(),
        session_id: entry.session_id.clone(),
        program: entry.program.clone(),
        args: entry.args.clone(),
        cwd: entry.cwd.clone(),
        state: entry.state,
        pid: entry.pid,
        exit_code: entry.exit_code,
        started_at: entry.started_at.clone(),
        ended_at: entry.ended_at.clone(),
        timeout_ms: u64::try_from(entry.timeout.as_millis()).unwrap_or(u64::MAX),
        output,
    }
}

/// Workspace-relative label for a resolved jail path.
fn relative_label(root: &Path, path: &Path) -> String {
    match path.strip_prefix(root) {
        Ok(relative) if relative.as_os_str().is_empty() => ".".to_owned(),
        Ok(relative) => relative
            .components()
            .map(|component| component.as_os_str().to_string_lossy().into_owned())
            .collect::<Vec<String>>()
            .join("/"),
        Err(_) => ".".to_owned(),
    }
}

fn map_tool_error(error: ToolError) -> JobError {
    JobError::Refused {
        refusal: error.refusal(),
        detail: error.detail().to_owned(),
    }
}

fn refused(refusal: Refusal, detail: impl Into<String>) -> JobError {
    JobError::Refused {
        refusal,
        detail: detail.into(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tail_buffer_keeps_last_cap_bytes_and_never_underflows() {
        let mut buffer = TailBuffer::new(16);
        buffer.append(b"abcd");
        buffer.append(b"efghijkl");
        assert!(!buffer.truncated);
        buffer.append(b"0123456789abcdefghij");
        let snapshot = buffer.snapshot();
        assert_eq!(snapshot.tail, "456789abcdefghij");
        assert!(snapshot.truncated);
        assert_eq!(snapshot.bytes, 4 + 8 + 20);
    }

    #[test]
    fn default_cap_accepts_an_ordinary_read_chunk_without_panicking() {
        let mut buffer = TailBuffer::new(DEFAULT_OUTPUT_TAIL_BYTES);
        buffer.append(&vec![b'x'; 8192]);
        let snapshot = buffer.snapshot();
        assert_eq!(snapshot.tail.len(), 8192);
        assert!(!snapshot.truncated);
        assert_eq!(snapshot.bytes, 8192);
    }

    #[test]
    fn growing_past_the_default_cap_keeps_only_the_last_cap_bytes() {
        let mut buffer = TailBuffer::new(DEFAULT_OUTPUT_TAIL_BYTES);
        buffer.append(&vec![b'a'; DEFAULT_OUTPUT_TAIL_BYTES]);
        buffer.append(b"TAIL");
        let snapshot = buffer.snapshot();
        assert_eq!(snapshot.tail.len(), DEFAULT_OUTPUT_TAIL_BYTES);
        assert!(snapshot.tail.ends_with("TAIL"));
        assert!(snapshot.truncated);
    }
}
