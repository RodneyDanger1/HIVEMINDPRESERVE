//! Core -> tools bridge (Phase 6 wiring for the patched frozen commands).
//!
//! `hiverouter-core` depends on `hiverouter-tools` (never the reverse). This
//! module is the single wiring point for the tools-owned capabilities used by
//! the WS command dispatch and the `/diagnose` route:
//!
//! - `SETTINGS_STATUS` -> [`hiverouter_tools::commands::settings_status_command`]
//! - `FILE_TREE_LIST` / `FILE_READ` -> the workspace jail ([`WorkspaceJail`])
//! - `APPROVAL_RESOLVE` -> the single approval gate + nonce/replay state machine,
//!   and exactly one append-only ledger row per committed decision
//!   ([`ToolsBridge::resolve_approval_ledgered`]); refusals never ledger.
//! - model-originated side effects -> one committed approval row and the frozen
//!   `APPROVAL_REQUEST` payload ([`ToolsBridge::request_model_tool_approval`]);
//!   the producer never executes the tool.
//! - `/diagnose` -> [`hiverouter_tools::health::diagnostics`]
//!
//! When a capability is genuinely unavailable (no usable workspace root), the
//! bridge returns the truthful named refusal `E_FILE_TREE_UNAVAILABLE` instead
//! of a generic internal error.

use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use hiverouter_contract::command;
use hiverouter_contract::{event, Refusal, RunId, TaskId, Timestamp};
use hiverouter_tools::approval::{ApprovalRequest, ApprovalResolution};
use hiverouter_tools::jail::WorkspaceJail;
use hiverouter_tools::ledger::{append_approval_decision, RunLedger, LEDGER_FILE_NAME};
use hiverouter_tools::ToolError;

use crate::db::Database;

/// Wired tools capabilities for the local server.
#[derive(Debug)]
pub struct ToolsBridge {
    jail: Option<WorkspaceJail>,
    jail_error: Option<String>,
    gate: hiverouter_tools::approval::ApprovalGate,
    workspace_root: PathBuf,
    db: Arc<Database>,
    /// The one open append-only ledger per directory: a single instance keeps
    /// the per-file monotonic `seq` counter and its write lock meaningful across
    /// commands (two fresh `RunLedger`s would each restart at seq 0).
    ledger: Mutex<Option<(PathBuf, RunLedger)>>,
}

impl ToolsBridge {
    /// Creates the bridge; a workspace root that cannot be opened leaves the
    /// jail unavailable (file commands then refuse `E_FILE_TREE_UNAVAILABLE`).
    #[must_use]
    pub fn new(workspace_root: &Path, db: Arc<Database>) -> Self {
        let (jail, jail_error) = if workspace_root.is_dir() {
            match WorkspaceJail::open(workspace_root) {
                Ok(jail) => (Some(jail), None),
                Err(error) => (
                    None,
                    Some(format!("{}: {}", error.code(), error.detail())),
                ),
            }
        } else {
            (
                None,
                Some(format!(
                    "workspace root {} is not an existing directory",
                    workspace_root.display()
                )),
            )
        };
        Self {
            jail,
            jail_error,
            gate: hiverouter_tools::approval::ApprovalGate::new(),
            workspace_root: workspace_root.to_path_buf(),
            db,
            ledger: Mutex::new(None),
        }
    }

    /// Creates the tools-owned approval table (idempotent). Called from the core
    /// migration/boot path so the tools layer never opens its own connection.
    ///
    /// # Errors
    /// `E_INTERNAL_ERROR` when the DDL fails.
    pub fn migrate(db: &Database) -> Result<(), Refusal> {
        db.with_connection_refusal(|conn| {
            hiverouter_tools::approval::migrate(conn).map_err(|error| error.refusal())
        })
    }

    /// Workspace root the jail was opened against.
    #[must_use]
    pub fn workspace_root(&self) -> &Path {
        &self.workspace_root
    }

    /// Whether the workspace jail is usable.
    #[must_use]
    pub fn file_access_available(&self) -> bool {
        self.jail.is_some()
    }

    /// Details of why the jail is unavailable (diagnostics only, no secrets).
    #[must_use]
    pub fn jail_error(&self) -> Option<&str> {
        self.jail_error.as_deref()
    }

    /// The single approval gate.
    #[must_use]
    pub fn approval_gate(&self) -> &hiverouter_tools::approval::ApprovalGate {
        &self.gate
    }

    fn require_jail(&self) -> Result<&WorkspaceJail, Refusal> {
        self.jail.as_ref().ok_or(Refusal::FileTreeUnavailable)
    }

    fn map_tool_error(error: ToolError) -> Refusal {
        error.refusal()
    }

    /// `SETTINGS_STATUS` -> `SETTINGS_STATUS_RESULT` (never the key).
    #[must_use]
    pub fn settings_status(&self) -> event::SettingsStatusResult {
        hiverouter_tools::commands::settings_status_command()
    }

    /// `FILE_TREE_LIST` -> `FILE_TREE_RESULT` through the jail.
    ///
    /// # Errors
    /// `E_FILE_TREE_UNAVAILABLE`, `E_PATH_OUTSIDE_JAIL`, `E_PATH_NOT_FOUND`.
    pub fn file_tree_list(&self) -> Result<event::FileTreeResult, Refusal> {
        let jail = self.require_jail()?;
        hiverouter_tools::commands::file_tree_list(jail).map_err(Self::map_tool_error)
    }

    /// `FILE_READ` -> `FILE_CONTENT` through the jail.
    ///
    /// # Errors
    /// `E_FILE_TREE_UNAVAILABLE`, `E_PATH_OUTSIDE_JAIL`, `E_PATH_NOT_FOUND`,
    /// `E_FILE_TOO_LARGE`, `E_SCHEMA_INVALID` (non-UTF-8 file).
    pub fn file_read(&self, payload: &command::FileRead) -> Result<event::FileContent, Refusal> {
        let jail = self.require_jail()?;
        hiverouter_tools::commands::file_read(jail, payload).map_err(Self::map_tool_error)
    }

    /// `APPROVAL_RESOLVE` -> resolution through the nonce/replay gate, without
    /// touching the run ledger (read-only/preview callers only; the command
    /// wiring uses [`Self::resolve_approval_ledgered`]).
    ///
    /// # Errors
    /// `E_APPROVAL_NOT_FOUND`, `E_APPROVAL_REPLAY`, `E_APPROVAL_TIMEOUT`,
    /// `E_APPROVAL_DECISION_INVALID`, `E_DB_BUSY`.
    pub fn resolve_approval(
        &self,
        payload: &command::ApprovalResolve,
    ) -> Result<ApprovalResolution, Refusal> {
        let now_unix_secs = Timestamp::now().unix_secs().unwrap_or(0);
        self.db.with_connection_refusal(|conn| {
            self.gate
                .resolve_command(
                    conn,
                    &payload.approval_id.to_string(),
                    &payload.nonce,
                    &payload.decision,
                    payload.reason.as_deref(),
                    now_unix_secs,
                )
                .map_err(|error| error.refusal())
        })
    }

    /// Ledger directory for approval-decision rows under the server data dir.
    ///
    /// The ledger is append-only JSONL (`run-ledger.jsonl`); it is runtime state,
    /// never a tracked artifact.
    #[must_use]
    pub fn approval_ledger_dir(data_dir: &Path) -> PathBuf {
        data_dir.join("ledger")
    }

    /// `APPROVAL_RESOLVE` command wiring: consume the nonce once, append exactly
    /// one ledger row for the committed decision, and return the resolution.
    ///
    /// Ordering is load-bearing: the gate is the only consumer of the nonce, so
    /// a replay/not-found/expired request returns `Err` **before** any ledger
    /// write (never ledger on refusal). Only a committed resolution is appended
    /// (`allowed` / `denied` + named code). If the ledger cannot be written after
    /// the decision was committed, this returns `E_INTERNAL_ERROR` so the caller
    /// publishes ERROR instead of ACK/`APPROVAL_RESOLVED` — a decision whose
    /// evidence could not be recorded is never claimed as a success (fail closed;
    /// the consumed nonce stays single-use).
    ///
    /// The ledger instance is cached per directory behind a mutex: concurrent
    /// resolutions therefore share one monotonic sequence and one append/flush
    /// lock instead of interleaving lines from fresh instances.
    ///
    /// # Errors
    /// See [`Self::resolve_approval`]; additionally `E_INTERNAL_ERROR` when the
    /// ledger cannot be opened or appended after a committed decision.
    pub fn resolve_approval_ledgered(
        &self,
        payload: &command::ApprovalResolve,
        ledger_dir: &Path,
    ) -> Result<ApprovalResolution, Refusal> {
        let resolution = self.resolve_approval(payload)?;
        let mut guard = self.ledger.lock().map_err(|_| Refusal::InternalError)?;
        let reopen = match guard.as_ref() {
            Some((dir, _)) => dir.as_path() != ledger_dir,
            None => true,
        };
        if reopen {
            let opened = RunLedger::open_in(ledger_dir).map_err(|error| error.refusal())?;
            *guard = Some((ledger_dir.to_path_buf(), opened));
        }
        let ledger = match guard.as_ref() {
            Some((_, ledger)) => ledger,
            None => return Err(Refusal::InternalError),
        };
        append_approval_decision(ledger, &resolution).map_err(|error| error.refusal())?;
        drop(guard);
        Ok(resolution)
    }

    /// Pending (undecided) approval requests for a run, oldest first.
    ///
    /// Read-only: it never consumes a nonce, resolves, executes or ledgers.
    ///
    /// # Errors
    /// `E_INTERNAL_ERROR` on a SQLite failure or malformed stored row.
    pub fn pending_approvals(&self, run_id: RunId) -> Result<Vec<ApprovalRequest>, Refusal> {
        self.db.with_connection_refusal(|conn| {
            hiverouter_tools::approval::pending_for_run(conn, run_id)
                .map_err(|error| error.refusal())
        })
    }

    /// Ledger file path for the approval-decision evidence (tests/diagnostics).
    #[must_use]
    pub fn approval_ledger_file(data_dir: &Path) -> PathBuf {
        Self::approval_ledger_dir(data_dir).join(LEDGER_FILE_NAME)
    }

    /// Creates (or returns) the committed approval request for one
    /// model-originated non-read tool call and returns the frozen wire
    /// `APPROVAL_REQUEST` payload — **without executing the tool**.
    ///
    /// The gate commits the row first (single-use nonce, `args_sha256` binding,
    /// `expires_at = now + APPROVAL_TTL_SECS`), so the returned event carries
    /// everything the UI needs to resolve it later. Calling this twice for the
    /// same `(run_id, tool_call_id)` with the same arguments while a pending
    /// request is still unexpired returns that same request (same approval
    /// id/nonce) instead of inserting a duplicate. The same tool-call id with
    /// **different arguments** is refused by name (`E_APPROVAL_REPLAY`) rather
    /// than silently reusing a binding for the wrong arguments; an
    /// expired/decided request is never resurrected.
    ///
    /// The engine owns the rest of the suspension seam: publish the returned
    /// event on the bus with the run filter and persist `awaiting_approval`
    /// (run_state `Effect::RequestApproval`). This method performs no side
    /// effect, no execution, and no ledger write.
    ///
    /// # Errors
    /// - `E_TOOL_NOT_FOUND` for a name outside the frozen tool vocabulary;
    /// - `E_SCHEMA_INVALID` for an empty `tool_call_id`;
    /// - `E_APPROVAL_REPLAY` when the same `(run_id, tool_call_id)` already has a
    ///   pending request bound to different arguments;
    /// - `E_INTERNAL_ERROR` for a read-class tool (no approval is needed or
    ///   permitted) or when the row cannot be committed;
    /// - `E_DB_BUSY` / `E_INTERNAL_ERROR` on database failure.
    pub fn request_model_tool_approval(
        &self,
        run_id: RunId,
        task_id: TaskId,
        tool_call_id: &str,
        tool: &str,
        args: &serde_json::Value,
    ) -> Result<event::ApprovalRequest, Refusal> {
        if tool_call_id.trim().is_empty() {
            return Err(Refusal::SchemaInvalid);
        }
        let digest = hiverouter_tools::approval::hash_args(args);
        let now_unix_secs = Timestamp::now().unix_secs().unwrap_or(0);
        self.db.with_connection_refusal(|conn| {
            let pending = hiverouter_tools::approval::pending_for_run(conn, run_id)
                .map_err(|error| error.refusal())?;
            if let Some(existing) = pending.into_iter().find(|request| {
                request.tool_call_id == tool_call_id && !request.is_expired(now_unix_secs)
            }) {
                if existing.args_sha256 != digest {
                    // Never reuse an approval bound to other arguments: the UI
                    // would present the wrong request and a later grant check
                    // would refuse execution after a decision was consumed.
                    return Err(Refusal::ApprovalReplay);
                }
                return Ok(Self::approval_request_event(&existing));
            }
            let request = hiverouter_tools::approval::request_tool_approval(
                &self.gate,
                conn,
                run_id,
                task_id,
                tool_call_id,
                tool,
                args,
                now_unix_secs,
            )
            .map_err(|error| error.refusal())?;
            Ok(Self::approval_request_event(&request))
        })
    }

    /// Wire `APPROVAL_REQUEST` payload for one committed tools approval request.
    ///
    /// The event is a faithful projection: `decision` is absent while the
    /// request is open, and the single-use `nonce`, run/task/tool-call ids, the
    /// `args_sha256` binding and the TTL `expires_at` are all carried so a
    /// pre-1.2 UI can still resolve the command.
    #[must_use]
    pub fn approval_request_event(request: &ApprovalRequest) -> event::ApprovalRequest {
        event::ApprovalRequest {
            approval_id: request.approval_id,
            tool: request.tool.id().to_owned(),
            decision: None,
            nonce: Some(request.nonce.clone()),
            run_id: Some(request.run_id),
            task_id: Some(request.task_id),
            tool_call_id: Some(request.tool_call_id.clone()),
            args_sha256: Some(request.args_sha256.clone()),
            expires_at: Some(request.expires_at.clone()),
        }
    }

    /// Wire `APPROVAL_REQUEST` payloads for every open request of a run,
    /// oldest first.
    ///
    /// Read-only projection over [`Self::pending_approvals`]: it never consumes
    /// a nonce, resolves, executes or ledgers, so it is safe for UI replay.
    ///
    /// # Errors
    /// See [`Self::pending_approvals`].
    pub fn pending_approval_events(
        &self,
        run_id: RunId,
    ) -> Result<Vec<event::ApprovalRequest>, Refusal> {
        let pending = self.pending_approvals(run_id)?;
        Ok(pending
            .iter()
            .map(Self::approval_request_event)
            .collect())
    }

    /// `/diagnose` payload from the tools health/derived census layer.
    ///
    /// # Errors
    /// `E_INTERNAL_ERROR` when the capability census is inconsistent.
    pub fn diagnostics(&self) -> Result<serde_json::Value, Refusal> {
        let db_ok = self.db.schema_version().is_ok();
        let key_present = self.settings_status().key_present;
        hiverouter_tools::health::diagnostics(db_ok, key_present)
            .map_err(|error| error.refusal())
    }

    /// Contract event for an approval resolution (`approve`/`deny` echo).
    #[must_use]
    pub fn approval_resolved_event(resolution: &ApprovalResolution) -> event::ApprovalResolved {
        let decision = match resolution.decision {
            hiverouter_tools::approval::ApprovalDecision::AllowOnce
            | hiverouter_tools::approval::ApprovalDecision::AllowSession => "approve",
            hiverouter_tools::approval::ApprovalDecision::Deny => "deny",
        };
        event::ApprovalResolved {
            approval_id: resolution.request.approval_id,
            tool: resolution.request.tool.id().to_owned(),
            decision: decision.to_owned(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use hiverouter_contract::command::ApprovalResolve;
    use hiverouter_contract::{ApprovalId, TaskId};
    use hiverouter_tools::tool::{PermissionClass, ToolId};

    fn args() -> serde_json::Value {
        serde_json::json!({"path": "note.txt", "content": "x"})
    }

    fn seeded_bridge() -> (tempfile::TempDir, Arc<Database>, ToolsBridge) {
        let temp = tempfile::tempdir().expect("temp dir");
        let db = Arc::new(Database::open(&temp.path().join("hive.db")).expect("database"));
        ToolsBridge::migrate(&db).expect("approvals migration");
        let bridge = ToolsBridge::new(temp.path(), db.clone());
        (temp, db, bridge)
    }

    fn seed(bridge: &ToolsBridge, db: &Database) -> (ApprovalId, String, RunId) {
        let now = Timestamp::now().unix_secs().unwrap_or(0);
        let run: RunId = RunId::new();
        let request = db
            .with_connection_refusal(|conn| {
                bridge
                    .approval_gate()
                    .request(
                        conn,
                        run,
                        TaskId::new(),
                        "call-1",
                        ToolId::FileWrite,
                        PermissionClass::Mutate,
                        &args(),
                        now,
                    )
                    .map_err(|error| error.refusal())
            })
            .expect("seed pending approval");
        (request.approval_id, request.nonce, run)
    }

    fn approve_payload(approval_id: ApprovalId, nonce: &str) -> ApprovalResolve {
        ApprovalResolve {
            approval_id,
            nonce: nonce.to_owned(),
            decision: "approve".to_owned(),
            reason: Some("operator approved".to_owned()),
        }
    }

    fn deny_payload(approval_id: ApprovalId, nonce: &str) -> ApprovalResolve {
        ApprovalResolve {
            approval_id,
            nonce: nonce.to_owned(),
            decision: "deny".to_owned(),
            reason: None,
        }
    }

    fn ledger_rows(temp: &tempfile::TempDir) -> Vec<serde_json::Value> {
        let file = ToolsBridge::approval_ledger_file(temp.path());
        if !file.exists() {
            return Vec::new();
        }
        std::fs::read_to_string(&file)
            .expect("ledger readable")
            .lines()
            .filter(|line| !line.trim().is_empty())
            .map(|line| serde_json::from_str(line).expect("valid ledger json line"))
            .collect()
    }

    #[test]
    fn approve_resolution_appends_exactly_one_ledger_row() {
        let (temp, db, bridge) = seeded_bridge();
        let (approval_id, nonce, run) = seed(&bridge, &db);
        let ledger_dir = ToolsBridge::approval_ledger_dir(temp.path());
        assert_eq!(ledger_dir, temp.path().join("ledger"));

        let resolution = bridge
            .resolve_approval_ledgered(&approve_payload(approval_id, &nonce), &ledger_dir)
            .expect("approved once");
        assert_eq!(resolution.request.run_id, run);

        let rows = ledger_rows(&temp);
        assert_eq!(rows.len(), 1, "one committed decision = one ledger row");
        assert_eq!(rows[0]["outcome"].as_str(), Some("allowed"));
        assert_eq!(rows[0]["tool"].as_str(), Some("file_write"));
        assert_eq!(rows[0]["permission_class"].as_str(), Some("mutate"));
        assert_eq!(rows[0]["tool_call_id"].as_str(), Some("call-1"));
        assert_eq!(
            rows[0]["run_id"].as_str(),
            Some(run.to_string().as_str()),
            "ledger row binds the real run"
        );
        assert_eq!(
            rows[0]["args_sha256"].as_str(),
            Some(hiverouter_tools::approval::hash_args(&args()).as_str()),
            "ledger row binds the approved argument digest"
        );
        assert!(rows[0]["refusal_code"].is_null());
        assert!(!rows[0].to_string().contains("\"content\""), "argument body is never ledgered");

        // The event published for this decision is truthful.
        let resolved = ToolsBridge::approval_resolved_event(&resolution);
        assert_eq!(resolved.approval_id, approval_id);
        assert_eq!(resolved.tool, "file_write");
        assert_eq!(resolved.decision, "approve");

        // Replaying the same nonce is refused and never appends a second row.
        let replay = bridge
            .resolve_approval_ledgered(&approve_payload(approval_id, &nonce), &ledger_dir)
            .expect_err("replay is refused");
        assert_eq!(replay, Refusal::ApprovalReplay);
        assert_eq!(ledger_rows(&temp).len(), 1, "refusal never ledgers");
    }

    #[test]
    fn deny_resolution_ledgers_one_named_refusal_row() {
        let (temp, db, bridge) = seeded_bridge();
        let (approval_id, nonce, _run) = seed(&bridge, &db);
        let ledger_dir = ToolsBridge::approval_ledger_dir(temp.path());

        let resolution = bridge
            .resolve_approval_ledgered(&deny_payload(approval_id, &nonce), &ledger_dir)
            .expect("denied once");
        let resolved = ToolsBridge::approval_resolved_event(&resolution);
        assert_eq!(resolved.decision, "deny");

        let rows = ledger_rows(&temp);
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0]["outcome"].as_str(), Some("denied"));
        assert_eq!(rows[0]["refusal_code"].as_str(), Some("E_APPROVAL_REQUIRED"));
        assert_eq!(rows[0]["args_sha256"].as_str().map(str::len), Some(64));
    }

    #[test]
    fn refused_resolution_never_writes_a_ledger_row() {
        let (temp, _db, bridge) = seeded_bridge();
        let ledger_dir = ToolsBridge::approval_ledger_dir(temp.path());

        let unknown = bridge
            .resolve_approval_ledgered(
                &approve_payload(ApprovalId::new(), "not-a-request-nonce"),
                &ledger_dir,
            )
            .expect_err("unknown approval id is refused");
        assert_eq!(unknown, Refusal::ApprovalNotFound);
        assert!(ledger_rows(&temp).is_empty(), "refusal must not be ledgered");
        assert!(!ToolsBridge::approval_ledger_file(temp.path()).exists());
    }

    #[test]
    fn pending_approvals_is_read_only_and_clears_after_resolution() {
        let (temp, db, bridge) = seeded_bridge();
        let (approval_id, nonce, run) = seed(&bridge, &db);

        let pending = bridge.pending_approvals(run).expect("read pending");
        assert_eq!(pending.len(), 1);
        assert_eq!(pending[0].approval_id, approval_id);
        assert!(ledger_rows(&temp).is_empty(), "read-only helper never ledgers");

        bridge
            .resolve_approval_ledgered(&approve_payload(approval_id, &nonce), &ToolsBridge::approval_ledger_dir(temp.path()))
            .expect("resolved");
        assert!(bridge.pending_approvals(run).expect("read pending").is_empty());
        assert_eq!(ledger_rows(&temp).len(), 1);
    }

    #[test]
    fn ledger_location_is_under_the_data_directory() {
        let data_dir = Path::new("data");
        assert_eq!(ToolsBridge::approval_ledger_dir(data_dir), PathBuf::from("data/ledger"));
        assert_eq!(
            ToolsBridge::approval_ledger_file(data_dir),
            PathBuf::from("data/ledger/run-ledger.jsonl")
        );
    }

    #[test]
    fn model_request_builds_wire_event_without_side_effects() {
        let (temp, _db, bridge) = seeded_bridge();
        let run = RunId::new();
        let task = TaskId::new();
        let args = serde_json::json!({"path": "note.txt", "content": "SHOULD-NOT-WRITE"});

        let event = bridge
            .request_model_tool_approval(run, task, "call-model-1", "file_write", &args)
            .expect("approval request committed");

        assert_eq!(event.tool, "file_write");
        assert!(event.decision.is_none(), "the request is open while undecided");
        assert_eq!(event.run_id, Some(run));
        assert_eq!(event.task_id, Some(task));
        assert_eq!(event.tool_call_id.as_deref(), Some("call-model-1"));
        assert_eq!(
            event.args_sha256.as_deref(),
            Some(hiverouter_tools::approval::hash_args(&args).as_str()),
            "the nonce is bound to the exact argument digest"
        );
        let nonce = event.nonce.as_deref().expect("wire event carries the nonce");
        assert_eq!(nonce.len(), 36, "uuid v7 nonce");
        let expires = event
            .expires_at
            .clone()
            .expect("wire event carries the TTL expiry")
            .unix_secs()
            .expect("representable expiry");
        let now = Timestamp::now().unix_secs().unwrap_or(0);
        assert!(expires > now, "expiry is in the future");
        assert!(expires <= now + hiverouter_tools::approval::APPROVAL_TTL_SECS + 1);

        // The row is committed (pending) and the producer executed nothing:
        // no workspace file, no execution, no ledger row.
        let pending = bridge.pending_approvals(run).expect("pending");
        assert_eq!(pending.len(), 1);
        assert_eq!(pending[0].approval_id, event.approval_id);
        assert!(!temp.path().join("note.txt").exists(), "producer never executes file_write");
        assert!(ledger_rows(&temp).is_empty(), "request is not a decision; no ledger row yet");
    }

    #[test]
    fn repeated_request_for_the_same_pending_tool_call_is_idempotent() {
        let (_temp, _db, bridge) = seeded_bridge();
        let run = RunId::new();
        let task = TaskId::new();
        let args = serde_json::json!({"path": "a.txt", "content": "x"});

        let first = bridge
            .request_model_tool_approval(run, task, "call-dup", "file_write", &args)
            .expect("first request");
        let second = bridge
            .request_model_tool_approval(run, task, "call-dup", "file_write", &args)
            .expect("second request returns the same pending approval");
        assert_eq!(first.approval_id, second.approval_id);
        assert_eq!(first.nonce, second.nonce);
        assert_eq!(bridge.pending_approvals(run).expect("pending").len(), 1);
    }

    #[test]
    fn same_tool_call_id_with_different_arguments_is_refused() {
        let (temp, _db, bridge) = seeded_bridge();
        let run = RunId::new();
        let task = TaskId::new();
        let approved_args = serde_json::json!({"path": "a.txt", "content": "approved"});

        let first = bridge
            .request_model_tool_approval(run, task, "call-args", "file_write", &approved_args)
            .expect("first request");
        let different = serde_json::json!({"path": "a.txt", "content": "DIFFERENT"});

        // Never reuse a pending approval bound to other arguments: silently
        // returning it would present the wrong request to the operator and make a
        // later grant check refuse execution after the decision was consumed.
        let refused = bridge
            .request_model_tool_approval(run, task, "call-args", "file_write", &different)
            .expect_err("different arguments under the same tool_call_id are refused");
        assert_eq!(refused, Refusal::ApprovalReplay);
        assert_eq!(bridge.pending_approvals(run).expect("pending").len(), 1);
        assert!(ledger_rows(&temp).is_empty());

        // The original pending request is still the one the operator resolves,
        // and it still resolves exactly once through the ledgered path.
        let payload = ApprovalResolve {
            approval_id: first.approval_id,
            nonce: first.nonce.clone().expect("nonce"),
            decision: "approve".to_owned(),
            reason: None,
        };
        bridge
            .resolve_approval_ledgered(&payload, &ToolsBridge::approval_ledger_dir(temp.path()))
            .expect("original request resolves");
        assert_eq!(ledger_rows(&temp).len(), 1);
    }

    #[test]
    fn model_request_refuses_read_only_unknown_and_empty_ids() {
        let (temp, _db, bridge) = seeded_bridge();
        let run = RunId::new();
        let task = TaskId::new();

        let read_only = bridge
            .request_model_tool_approval(run, task, "call-read", "file_read", &serde_json::json!({"path": "a"}))
            .expect_err("read tools need no approval");
        assert_eq!(read_only, Refusal::InternalError);

        let unknown = bridge
            .request_model_tool_approval(run, task, "call-unknown", "rm_rf", &serde_json::json!({}))
            .expect_err("unknown vocabulary");
        assert_eq!(unknown, Refusal::ToolNotFound);

        let empty = bridge
            .request_model_tool_approval(run, task, "  ", "file_write", &serde_json::json!({"path": "a", "content": "x"}))
            .expect_err("empty tool_call_id");
        assert_eq!(empty, Refusal::SchemaInvalid);

        assert!(bridge.pending_approvals(run).expect("pending").is_empty());
        assert!(ledger_rows(&temp).is_empty());
    }

    #[test]
    fn pending_approval_events_is_a_read_only_wire_projection() {
        let (temp, _db, bridge) = seeded_bridge();
        let run = RunId::new();
        let task = TaskId::new();

        bridge
            .request_model_tool_approval(run, task, "call-a", "file_write", &serde_json::json!({"path": "a", "content": "x"}))
            .expect("first request");
        bridge
            .request_model_tool_approval(run, task, "call-b", "shell_exec", &serde_json::json!({"program": "cmd.exe", "args": []}))
            .expect("second request");

        let events = bridge.pending_approval_events(run).expect("wire projection");
        assert_eq!(events.len(), 2);
        let mut ids = events
            .iter()
            .map(|event| event.tool_call_id.clone().unwrap_or_default())
            .collect::<Vec<_>>();
        ids.sort();
        assert_eq!(ids, vec!["call-a".to_owned(), "call-b".to_owned()]);
        for event in &events {
            assert!(event.nonce.as_deref().map(str::len) == Some(36));
            assert!(event.expires_at.is_some());
            assert_eq!(event.run_id, Some(run));
            assert_eq!(event.task_id, Some(task));
            assert!(event.args_sha256.as_deref().map(str::len) == Some(64));
        }
        assert!(ledger_rows(&temp).is_empty(), "projection must not ledger");
    }

    #[test]
    fn produced_request_resolves_through_the_single_ledger_path() {
        let (temp, _db, bridge) = seeded_bridge();
        let run = RunId::new();
        let task = TaskId::new();
        let event = bridge
            .request_model_tool_approval(run, task, "call-resolve", "file_write", &serde_json::json!({"path": "a", "content": "x"}))
            .expect("request");

        let payload = ApprovalResolve {
            approval_id: event.approval_id,
            nonce: event.nonce.clone().expect("nonce"),
            decision: "approve".to_owned(),
            reason: None,
        };
        let resolution = bridge
            .resolve_approval_ledgered(&payload, &ToolsBridge::approval_ledger_dir(temp.path()))
            .expect("approved once");
        assert_eq!(resolution.request.run_id, run);
        assert_eq!(ledger_rows(&temp).len(), 1);

        let replay = bridge
            .resolve_approval_ledgered(&payload, &ToolsBridge::approval_ledger_dir(temp.path()))
            .expect_err("replay refused");
        assert_eq!(replay, Refusal::ApprovalReplay);
        assert_eq!(ledger_rows(&temp).len(), 1, "existing path stays exactly once");
    }

    #[test]
    fn consecutive_decisions_share_one_ledger_and_monotonic_sequence() {
        let (temp, db, bridge) = seeded_bridge();
        let ledger_dir = ToolsBridge::approval_ledger_dir(temp.path());
        let (first_id, first_nonce, _first_run) = seed(&bridge, &db);
        let (second_id, second_nonce, _second_run) = seed(&bridge, &db);

        bridge
            .resolve_approval_ledgered(&approve_payload(first_id, &first_nonce), &ledger_dir)
            .expect("first decision");
        bridge
            .resolve_approval_ledgered(&deny_payload(second_id, &second_nonce), &ledger_dir)
            .expect("second decision");

        let rows = ledger_rows(&temp);
        assert_eq!(rows.len(), 2, "two decisions, two rows");
        assert_eq!(rows[0]["seq"].as_u64(), Some(0));
        assert_eq!(
            rows[1]["seq"].as_u64(),
            Some(1),
            "one cached ledger keeps the per-file sequence monotonic"
        );
        assert_eq!(rows[0]["outcome"].as_str(), Some("allowed"));
        assert_eq!(rows[1]["outcome"].as_str(), Some("denied"));
    }
}
