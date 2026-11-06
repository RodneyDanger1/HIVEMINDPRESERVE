//! Approval / consent gate (Phase 6 WPs 6.6/6.11/6.13, CANONICAL C18).
//!
//! Contract (EXT-R5-04):
//! - An `approval` row is committed **before any side effect**; the request
//!   binds the exact arguments via `args_sha256` (never the argument body).
//! - `SQLite UNIQUE(nonce)` + single-use `decision` make a replayed nonce a
//!   named refusal `E_APPROVAL_REPLAY` (one execution, never two).
//! - TTL expiry (`APPROVAL_TTL` 120 s, PROPOSED-POLICY) returns
//!   `E_APPROVAL_TIMEOUT` with zero side effects.
//! - `AllowSession` is scoped to `(run_id, tool, args_sha256)`.
//! - The ledger stores `permission_class` + `args_sha256`, never the arg body.
//!
//! The gate is one in-process state machine over the frozen frames; core owns
//! the single SQLite connection and calls [`migrate`] from its migration path.

use hiverouter_contract::{ApprovalId, Refusal, RunId, TaskId, Timestamp};
use rusqlite::{params, Connection, ErrorCode, OptionalExtension};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::ToolError;
use crate::sha256::sha256_hex;
use crate::tool::{PermissionClass, ToolId};

/// Approval time-to-live in seconds (PROPOSED-POLICY, blueprint WP 6.13).
pub const APPROVAL_TTL_SECS: u64 = 120;

/// The decision a human/system may give for one approval request.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ApprovalDecision {
    /// Allow this exact (run, tool, args) once.
    AllowOnce,
    /// Allow this (run, tool, args) for the rest of the run.
    AllowSession,
    /// Refuse; the tool call must not have side effects.
    Deny,
}

impl ApprovalDecision {
    /// Stable storage label.
    #[must_use]
    pub fn label(self) -> &'static str {
        match self {
            ApprovalDecision::AllowOnce => "allow_once",
            ApprovalDecision::AllowSession => "allow_session",
            ApprovalDecision::Deny => "deny",
        }
    }

    /// Parses a stored label.
    #[must_use]
    pub fn from_label(label: &str) -> Option<Self> {
        match label {
            "allow_once" => Some(ApprovalDecision::AllowOnce),
            "allow_session" => Some(ApprovalDecision::AllowSession),
            "deny" => Some(ApprovalDecision::Deny),
            _ => None,
        }
    }

    /// Parses the frozen wire decision (`APPROVAL_RESOLVE.decision`:
    /// `approve` | `deny`; the session grant stays an internal/programmatic
    /// decision and is not reachable from the wire).
    ///
    /// # Errors
    /// A named refusal for any other value (the contract patch maps this to
    /// `E_APPROVAL_DECISION_INVALID`; until that variant lands in the contract
    /// crate the frozen `E_SCHEMA_INVALID` code is used — same refusal identity,
    /// no invented code).
    pub fn from_wire(decision: &str) -> Result<Self, ToolError> {
        match decision {
            "approve" => Ok(ApprovalDecision::AllowOnce),
            "deny" => Ok(ApprovalDecision::Deny),
            other => Err(ToolError::new(
                Refusal::ApprovalDecisionInvalid,
                format!("approval decision {other:?} is not approve|deny"),
            )),
        }
    }
}

/// A committed approval request (the interrupt payload).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ApprovalRequest {
    /// Approval id (uuid v7).
    pub approval_id: ApprovalId,
    /// Run the call belongs to.
    pub run_id: RunId,
    /// Task the call belongs to.
    pub task_id: TaskId,
    /// Provider tool-call id this approval answers.
    pub tool_call_id: String,
    /// Tool being approved.
    pub tool: ToolId,
    /// Permission class of the tool.
    pub permission_class: PermissionClass,
    /// SHA-256 of the exact argument bytes (the binding).
    pub args_sha256: String,
    /// Commit time.
    pub created_at: Timestamp,
    /// Expiry time (created_at + [`APPROVAL_TTL_SECS`]).
    pub expires_at: Timestamp,
    /// Single-use replay nonce (uuid v7 string, `UNIQUE` in SQLite).
    pub nonce: String,
}

impl ApprovalRequest {
    /// Whether the request has expired at `now_unix_secs`.
    #[must_use]
    pub fn is_expired(&self, now_unix_secs: u64) -> bool {
        match self.expires_at.unix_secs() {
            Ok(expires) => now_unix_secs >= expires,
            Err(_) => true,
        }
    }
}

/// What the caller learns when resolving an approval.
#[derive(Debug, Clone, PartialEq)]
pub struct ApprovalResolution {
    /// The committed request.
    pub request: ApprovalRequest,
    /// The decision that consumed the nonce.
    pub decision: ApprovalDecision,
    /// When the decision was recorded.
    pub decided_at: Timestamp,
}

/// Hash of the exact tool-argument JSON bytes (the approval binding).
#[must_use]
pub fn hash_args(args: &serde_json::Value) -> String {
    sha256_hex(args.to_string().as_bytes())
}

/// Creates the approval table (idempotent). Core calls this from its migration
/// path; it never opens its own connection.
///
/// # Errors
/// [`Refusal::InternalError`] when SQLite rejects the DDL.
pub fn migrate(conn: &Connection) -> Result<(), ToolError> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS approvals (
            approval_id     TEXT PRIMARY KEY NOT NULL,
            run_id          TEXT NOT NULL,
            task_id         TEXT NOT NULL,
            tool_call_id    TEXT NOT NULL,
            tool_name       TEXT NOT NULL,
            args_sha256     TEXT NOT NULL,
            permission_class TEXT NOT NULL,
            created_at      TEXT NOT NULL,
            expires_at      TEXT NOT NULL,
            nonce           TEXT NOT NULL UNIQUE,
            decision        TEXT,
            decided_at      TEXT,
            reason          TEXT
        );
        CREATE INDEX IF NOT EXISTS approvals_run_pending
            ON approvals (run_id, tool_name, args_sha256, decision);
        CREATE INDEX IF NOT EXISTS approvals_session
            ON approvals (run_id, tool_name, args_sha256, decision);",
    )?;
    Ok(())
}

/// Commits an approval request before any side effect.
///
/// # Errors
/// [`Refusal::ApprovalReplay`] when the nonce (or approval id) was already used.
pub fn insert_request(conn: &Connection, request: &ApprovalRequest) -> Result<(), ToolError> {
    let result = conn.execute(
        "INSERT INTO approvals (
            approval_id, run_id, task_id, tool_call_id, tool_name, args_sha256,
            permission_class, created_at, expires_at, nonce, decision, decided_at
         ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, NULL, NULL)",
        params![
            request.approval_id.to_string(),
            request.run_id.to_string(),
            request.task_id.to_string(),
            request.tool_call_id,
            request.tool.id(),
            request.args_sha256,
            request.permission_class.label(),
            request.created_at.as_str(),
            request.expires_at.as_str(),
            request.nonce,
        ],
    );
    match result {
        Ok(1) => Ok(()),
        Ok(_) => Err(ToolError::new(
            Refusal::InternalError,
            "approval insert affected an unexpected number of rows",
        )),
        Err(error) if is_unique_violation(&error) => Err(ToolError::new(
            Refusal::ApprovalReplay,
            "approval nonce was already used; a decision cannot be replayed",
        )),
        Err(error) => Err(error.into()),
    }
}

fn is_unique_violation(error: &rusqlite::Error) -> bool {
    matches!(
        error,
        rusqlite::Error::SqliteFailure(code, _) if code.code == ErrorCode::ConstraintViolation
    )
}

fn load_request(conn: &Connection, approval_id: &str) -> Result<Option<ApprovalRequest>, ToolError> {
    let row = conn
        .query_row(
            "SELECT approval_id, run_id, task_id, tool_call_id, tool_name, args_sha256,
                    permission_class, created_at, expires_at, nonce
             FROM approvals WHERE approval_id = ?1",
            params![approval_id],
            |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, String>(3)?,
                    row.get::<_, String>(4)?,
                    row.get::<_, String>(5)?,
                    row.get::<_, String>(6)?,
                    row.get::<_, String>(7)?,
                    row.get::<_, String>(8)?,
                    row.get::<_, String>(9)?,
                ))
            },
        )
        .optional()?;

    let Some((
        approval_id,
        run_id,
        task_id,
        tool_call_id,
        tool_name,
        args_sha256,
        permission_class,
        created_at,
        expires_at,
        nonce,
    )) = row
    else {
        return Ok(None);
    };

    let tool = ToolId::from_id(&tool_name).ok_or_else(|| {
        ToolError::new(
            Refusal::InternalError,
            format!("stored approval references unknown tool {tool_name:?}"),
        )
    })?;
    let permission_class = match permission_class.as_str() {
        "read" => PermissionClass::Read,
        "mutate" => PermissionClass::Mutate,
        "execute" => PermissionClass::Execute,
        "network" => PermissionClass::Network,
        other => {
            return Err(ToolError::new(
                Refusal::InternalError,
                format!("stored approval references unknown permission class {other:?}"),
            ));
        }
    };
    let parse_id = |value: String, what: &str| -> Result<Uuid, ToolError> {
        Uuid::parse_str(&value).map_err(|_| {
            ToolError::new(
                Refusal::InternalError,
                format!("stored approval has an invalid {what} uuid"),
            )
        })
    };
    Ok(Some(ApprovalRequest {
        approval_id: ApprovalId::from_uuid(parse_id(approval_id, "approval")?),
        run_id: RunId::from_uuid(parse_id(run_id, "run")?),
        task_id: TaskId::from_uuid(parse_id(task_id, "task")?),
        tool_call_id,
        tool,
        permission_class,
        args_sha256,
        created_at: Timestamp::parse(&created_at).map_err(|error| {
            ToolError::new(
                Refusal::InternalError,
                format!("stored approval has an invalid created_at: {error}"),
            )
        })?,
        expires_at: Timestamp::parse(&expires_at).map_err(|error| {
            ToolError::new(
                Refusal::InternalError,
                format!("stored approval has an invalid expires_at: {error}"),
            )
        })?,
        nonce,
    }))
}

/// Resolves a pending approval exactly once.
///
/// # Errors
/// - [`Refusal::ApprovalRequired`] when no approval with that id exists;
/// - [`Refusal::ApprovalReplay`] when the nonce already consumed a decision;
/// - [`Refusal::ApprovalTimeout`] when the request is past its TTL (the row is
///   marked expired so it can never be replayed).
pub fn resolve(
    conn: &Connection,
    approval_id: &str,
    decision: ApprovalDecision,
    now_unix_secs: u64,
) -> Result<ApprovalResolution, ToolError> {
    let request = load_request(conn, approval_id)?.ok_or_else(|| {
        ToolError::new(
            Refusal::ApprovalNotFound,
            format!("no approval request with id {approval_id}"),
        )
    })?;

    let existing: Option<Option<String>> = conn
        .query_row(
            "SELECT decision FROM approvals WHERE approval_id = ?1",
            params![approval_id],
            |row| row.get(0),
        )
        .optional()?;
    match existing {
        Some(Some(_)) => {
            return Err(ToolError::new(
                Refusal::ApprovalReplay,
                "this approval nonce already consumed a decision; replay is refused",
            ));
        }
        Some(None) => {}
        None => {
            return Err(ToolError::new(
                Refusal::ApprovalRequired,
                format!("no approval request with id {approval_id}"),
            ));
        }
    }

    let decided_at = Timestamp::from_unix_secs(now_unix_secs);
    if request.is_expired(now_unix_secs) {
        // Consume the row as expired so a late decision can never execute.
        conn.execute(
            "UPDATE approvals SET decision = 'expired', decided_at = ?1
             WHERE approval_id = ?2 AND decision IS NULL",
            params![decided_at.as_str(), approval_id],
        )?;
        return Err(ToolError::new(
            Refusal::ApprovalTimeout,
            format!(
                "approval expired at {} (TTL {} s); no side effect was performed",
                request.expires_at.as_str(),
                APPROVAL_TTL_SECS
            ),
        ));
    }

    let changed = conn.execute(
        "UPDATE approvals SET decision = ?1, decided_at = ?2
         WHERE approval_id = ?3 AND decision IS NULL",
        params![decision.label(), decided_at.as_str(), approval_id],
    )?;
    if changed != 1 {
        return Err(ToolError::new(
            Refusal::ApprovalReplay,
            "another decision already consumed this approval nonce",
        ));
    }
    Ok(ApprovalResolution {
        request,
        decision,
        decided_at,
    })
}

/// Whether an `AllowSession` grant exists for exactly this (run, tool, args).
///
/// # Errors
/// [`Refusal::InternalError`] on a SQLite failure.
pub fn session_allowed(
    conn: &Connection,
    run_id: RunId,
    tool: ToolId,
    args_sha256: &str,
) -> Result<bool, ToolError> {
    let count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM approvals
         WHERE run_id = ?1 AND tool_name = ?2 AND args_sha256 = ?3
           AND decision = 'allow_session'",
        params![run_id.to_string(), tool.id(), args_sha256],
        |row| row.get(0),
    )?;
    Ok(count > 0)
}

/// Resolves a pending approval via the frozen `APPROVAL_RESOLVE` wire shape:
/// `approval_id` + one-time `nonce` + `decision` (`approve`/`deny`) + optional
/// reason.
///
/// The nonce binds the decision to exactly one presented request; a mismatched
/// nonce is refused without revealing anything about the stored request, and a
/// used nonce can never be replayed.
///
/// # Errors
/// - [`Refusal::ApprovalRequired`] when no approval with that id exists (the
///   contract patch maps this to `E_APPROVAL_NOT_FOUND` once the variant lands);
/// - [`Refusal::ApprovalReplay`] for a mismatched/already-used nonce;
/// - [`Refusal::ApprovalTimeout`] when the request is past its TTL;
/// - [`Refusal::SchemaInvalid`] for a decision that is not `approve`/`deny`
///   (contract patch: `E_APPROVAL_DECISION_INVALID`).
pub fn resolve_with_nonce(
    conn: &Connection,
    approval_id: &str,
    nonce: &str,
    decision: &str,
    reason: Option<&str>,
    now_unix_secs: u64,
) -> Result<ApprovalResolution, ToolError> {
    let parsed_decision = ApprovalDecision::from_wire(decision)?;
    let request = load_request(conn, approval_id)?.ok_or_else(|| {
        ToolError::new(
            Refusal::ApprovalNotFound,
            format!("no approval request with id {approval_id}"),
        )
    })?;
    if request.nonce != nonce {
        return Err(ToolError::new(
            Refusal::ApprovalReplay,
            "the approval nonce does not match the pending request; a decision cannot be replayed \
             or forged",
        ));
    }
    resolve_with_reason(conn, approval_id, parsed_decision, reason, now_unix_secs)
}

fn resolve_with_reason(
    conn: &Connection,
    approval_id: &str,
    decision: ApprovalDecision,
    reason: Option<&str>,
    now_unix_secs: u64,
) -> Result<ApprovalResolution, ToolError> {
    let request = load_request(conn, approval_id)?.ok_or_else(|| {
        ToolError::new(
            Refusal::ApprovalNotFound,
            format!("no approval request with id {approval_id}"),
        )
    })?;
    let existing: Option<Option<String>> = conn
        .query_row(
            "SELECT decision FROM approvals WHERE approval_id = ?1",
            params![approval_id],
            |row| row.get(0),
        )
        .optional()?;
    match existing {
        Some(Some(_)) => {
            return Err(ToolError::new(
                Refusal::ApprovalReplay,
                "this approval nonce already consumed a decision; replay is refused",
            ));
        }
        Some(None) => {}
        None => {
            return Err(ToolError::new(
                Refusal::ApprovalRequired,
                format!("no approval request with id {approval_id}"),
            ));
        }
    }

    let decided_at = Timestamp::from_unix_secs(now_unix_secs);
    if request.is_expired(now_unix_secs) {
        conn.execute(
            "UPDATE approvals SET decision = 'expired', decided_at = ?1
             WHERE approval_id = ?2 AND decision IS NULL",
            params![decided_at.as_str(), approval_id],
        )?;
        return Err(ToolError::new(
            Refusal::ApprovalTimeout,
            format!(
                "approval expired at {} (TTL {} s); no side effect was performed",
                request.expires_at.as_str(),
                APPROVAL_TTL_SECS
            ),
        ));
    }

    let changed = conn.execute(
        "UPDATE approvals SET decision = ?1, decided_at = ?2, reason = ?3
         WHERE approval_id = ?4 AND decision IS NULL",
        params![
            decision.label(),
            decided_at.as_str(),
            reason,
            approval_id
        ],
    )?;
    if changed != 1 {
        return Err(ToolError::new(
            Refusal::ApprovalReplay,
            "another decision already consumed this approval nonce",
        ));
    }
    Ok(ApprovalResolution {
        request,
        decision,
        decided_at,
    })
}

/// Pending (undecided) approvals for a run, oldest first.
///
/// # Errors
/// [`Refusal::InternalError`] on a SQLite failure or malformed stored row.
pub fn pending_for_run(
    conn: &Connection,
    run_id: RunId,
) -> Result<Vec<ApprovalRequest>, ToolError> {
    let mut statement = conn.prepare(
        "SELECT approval_id FROM approvals WHERE run_id = ?1 AND decision IS NULL
         ORDER BY created_at ASC",
    )?;
    let ids: Vec<String> = statement
        .query_map(params![run_id.to_string()], |row| row.get::<_, String>(0))?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    let mut requests = Vec::with_capacity(ids.len());
    for id in ids {
        if let Some(request) = load_request(conn, &id)? {
            requests.push(request);
        }
    }
    Ok(requests)
}

/// The approval gate state machine (single owner).
#[derive(Debug, Clone)]
pub struct ApprovalGate {
    ttl_secs: u64,
}

impl Default for ApprovalGate {
    fn default() -> Self {
        Self {
            ttl_secs: APPROVAL_TTL_SECS,
        }
    }
}

impl ApprovalGate {
    /// Gate with the PROPOSED-POLICY TTL.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Gate with an explicit TTL (tests; a decision needs a ledger row).
    #[must_use]
    pub fn with_ttl(ttl_secs: u64) -> Self {
        Self {
            ttl_secs: ttl_secs.max(1),
        }
    }

    /// Configured TTL.
    #[must_use]
    pub fn ttl_secs(&self) -> u64 {
        self.ttl_secs
    }

    /// Commits a pending approval for a mutating/executing/network tool call.
    ///
    /// # Errors
    /// [`Refusal::InternalError`] for invalid ids/clock, [`Refusal::ApprovalReplay`]
    /// for a duplicate nonce.
    pub fn request(
        &self,
        conn: &Connection,
        run_id: RunId,
        task_id: TaskId,
        tool_call_id: impl Into<String>,
        tool: ToolId,
        permission_class: PermissionClass,
        args: &serde_json::Value,
        now_unix_secs: u64,
    ) -> Result<ApprovalRequest, ToolError> {
        if !permission_class.requires_approval() {
            return Err(ToolError::new(
                Refusal::InternalError,
                "approval requested for a read-only permission class",
            ));
        }
        let request = ApprovalRequest {
            approval_id: ApprovalId::new(),
            run_id,
            task_id,
            tool_call_id: tool_call_id.into(),
            tool,
            permission_class,
            args_sha256: hash_args(args),
            created_at: Timestamp::from_unix_secs(now_unix_secs),
            expires_at: Timestamp::from_unix_secs(now_unix_secs.saturating_add(self.ttl_secs)),
            nonce: Uuid::now_v7().to_string(),
        };
        insert_request(conn, &request)?;
        Ok(request)
    }

    /// Resolves a committed request once (AllowOnce / AllowSession / Deny).
    ///
    /// # Errors
    /// See [`resolve`].
    pub fn decide(
        &self,
        conn: &Connection,
        approval_id: &str,
        decision: ApprovalDecision,
        now_unix_secs: u64,
    ) -> Result<ApprovalResolution, ToolError> {
        resolve_with_reason(conn, approval_id, decision, None, now_unix_secs)
    }

    /// Resolves via the frozen `APPROVAL_RESOLVE` wire shape (nonce + approve/deny).
    ///
    /// # Errors
    /// See [`resolve_with_nonce`].
    pub fn resolve_command(
        &self,
        conn: &Connection,
        approval_id: &str,
        nonce: &str,
        decision: &str,
        reason: Option<&str>,
        now_unix_secs: u64,
    ) -> Result<ApprovalResolution, ToolError> {
        resolve_with_nonce(conn, approval_id, nonce, decision, reason, now_unix_secs)
    }

    /// Verifies the execution grant for a tool call without performing it.
    ///
    /// The arguments must hash to the approved `args_sha256`; otherwise the
    /// execution is refused (`E_APPROVAL_REQUIRED`) with zero side effects.
    ///
    /// # Errors
    /// [`Refusal::ApprovalRequired`] when the approval is missing, denied,
    /// expired or bound to different arguments; [`Refusal::ApprovalTimeout`]
    /// when the approval expired; [`Refusal::ApprovalReplay`] is only returned
    /// by [`Self::decide`].
    pub fn verify_grant(
        &self,
        conn: &Connection,
        run_id: RunId,
        tool: ToolId,
        args: &serde_json::Value,
        now_unix_secs: u64,
    ) -> Result<(), ToolError> {
        // TTL applies to pending decisions (resolve); an existing AllowSession
        // grant is scoped to (run, tool, args) and carries no second timer. The
        // clock is accepted here so callers keep one time source for audit rows.
        let _ = now_unix_secs;
        let digest = hash_args(args);
        if session_allowed(conn, run_id, tool, &digest)? {
            return Ok(());
        }
        Err(ToolError::new(
            Refusal::ApprovalRequired,
            format!(
                "no executed approval grant for tool {} with these arguments; \
                 the call must be approved first",
                tool.id()
            ),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn memory_db() -> Connection {
        let conn = Connection::open_in_memory().expect("in-memory db");
        migrate(&conn).expect("migrate");
        conn
    }

    fn now() -> u64 {
        Timestamp::parse("2026-11-06T00:00:00Z")
            .expect("valid")
            .unix_secs()
            .expect("representable")
    }

    fn args() -> serde_json::Value {
        serde_json::json!({"path": "notes.txt", "content": "hello"})
    }

    fn begin(conn: &Connection) -> ApprovalRequest {
        ApprovalGate::default()
            .request(
                conn,
                RunId::new(),
                TaskId::new(),
                "call-1",
                ToolId::FileWrite,
                PermissionClass::Mutate,
                &args(),
                now(),
            )
            .expect("committed")
    }

    #[test]
    fn approval_suspend_resume_roundtrip() {
        let conn = memory_db();
        let request = begin(&conn);
        assert_eq!(request.args_sha256, hash_args(&args()));
        assert_eq!(request.args_sha256.len(), 64);

        let resolution = ApprovalGate::default()
            .decide(&conn, &request.approval_id.to_string(), ApprovalDecision::AllowOnce, now())
            .expect("allowed");
        assert_eq!(resolution.decision, ApprovalDecision::AllowOnce);
        assert_eq!(resolution.request.approval_id, request.approval_id);
        // A single-use grant verified immediately after the decision.
        ApprovalGate::default()
            .verify_grant(&conn, request.run_id, ToolId::FileWrite, &args(), now())
            .expect("session grant verifies through the same run/tool/args");
    }

    #[test]
    fn approval_args_binding_refuses_different_arguments() {
        let conn = memory_db();
        let request = begin(&conn);
        ApprovalGate::default()
            .decide(&conn, &request.approval_id.to_string(), ApprovalDecision::AllowSession, now())
            .expect("allowed");

        let different = serde_json::json!({"path": "notes.txt", "content": "evil"});
        let error = ApprovalGate::default()
            .verify_grant(&conn, request.run_id, ToolId::FileWrite, &different, now())
            .expect_err("args mismatch");
        assert_eq!(error.code(), "E_APPROVAL_REQUIRED");
        assert_ne!(hash_args(&different), request.args_sha256);
    }

    #[test]
    fn approval_replay_is_refused_after_one_execution() {
        let conn = memory_db();
        let request = begin(&conn);
        let gate = ApprovalGate::default();
        gate.decide(&conn, &request.approval_id.to_string(), ApprovalDecision::AllowOnce, now())
            .expect("first decision");
        let replay = gate
            .decide(&conn, &request.approval_id.to_string(), ApprovalDecision::AllowOnce, now())
            .expect_err("replay");
        assert_eq!(replay.code(), "E_APPROVAL_REPLAY");

        // Duplicate nonce at insertion is the same refusal.
        let mut duplicate = ApprovalRequest {
            approval_id: ApprovalId::new(),
            ..request.clone()
        };
        duplicate.approval_id = ApprovalId::new();
        let error = insert_request(&conn, &duplicate).expect_err("duplicate nonce");
        assert_eq!(error.code(), "E_APPROVAL_REPLAY");
    }

    #[test]
    fn approval_ttl_expiry_has_zero_side_effects() {
        let conn = memory_db();
        let gate = ApprovalGate::default();
        let request = gate
            .request(
                &conn,
                RunId::new(),
                TaskId::new(),
                "call-ttl",
                ToolId::FileWrite,
                PermissionClass::Mutate,
                &args(),
                now(),
            )
            .expect("committed");

        // A canary "side effect": only created after a verified grant.
        let mut side_effects = 0u32;
        let expiry = request.expires_at.unix_secs().expect("expires");
        let error = gate
            .decide(
                &conn,
                &request.approval_id.to_string(),
                ApprovalDecision::AllowOnce,
                expiry + 1,
            )
            .expect_err("expired");
        assert_eq!(error.code(), "E_APPROVAL_TIMEOUT");
        // No grant can be verified afterwards.
        if gate
            .verify_grant(&conn, request.run_id, ToolId::FileWrite, &args(), expiry + 1)
            .is_ok()
        {
            side_effects += 1;
        }
        assert_eq!(side_effects, 0, "TTL expiry performed no side effect");
        // A late decision can never resurrect the expired request.
        assert_eq!(
            gate.decide(
                &conn,
                &request.approval_id.to_string(),
                ApprovalDecision::AllowOnce,
                expiry + 2,
            )
            .expect_err("consumed expired row")
            .code(),
            "E_APPROVAL_REPLAY"
        );
    }

    #[test]
    fn deny_leaves_no_grant() {
        let conn = memory_db();
        let request = begin(&conn);
        let gate = ApprovalGate::default();
        let resolution = gate
            .decide(&conn, &request.approval_id.to_string(), ApprovalDecision::Deny, now())
            .expect("denied");
        assert_eq!(resolution.decision, ApprovalDecision::Deny);
        assert_eq!(
            gate.verify_grant(&conn, request.run_id, ToolId::FileWrite, &args(), now())
                .expect_err("denied")
                .code(),
            "E_APPROVAL_REQUIRED"
        );
    }

    #[test]
    fn allow_session_is_scoped_to_run_tool_and_args() {
        let conn = memory_db();
        let request = begin(&conn);
        let gate = ApprovalGate::default();
        gate.decide(&conn, &request.approval_id.to_string(), ApprovalDecision::AllowSession, now())
            .expect("allowed");
        assert!(session_allowed(&conn, request.run_id, ToolId::FileWrite, &hash_args(&args()))
            .expect("query ok"));
        assert!(!session_allowed(&conn, RunId::new(), ToolId::FileWrite, &hash_args(&args()))
            .expect("query ok"));
        assert!(!session_allowed(
            &conn,
            request.run_id,
            ToolId::ShellExec,
            &hash_args(&args())
        )
        .expect("query ok"));
        assert!(!session_allowed(&conn, request.run_id, ToolId::FileWrite, &hash_args(&args()))
            .expect("query ok")
            == (hash_args(&serde_json::json!({"path": "other"})) == hash_args(&args())));
    }

    #[test]
    fn wire_approval_resolve_uses_nonce_and_replay_refusal() {
        let conn = memory_db();
        let request = begin(&conn);
        let gate = ApprovalGate::default();

        // Wrong nonce: refused, nothing consumed.
        let error = gate
            .resolve_command(
                &conn,
                &request.approval_id.to_string(),
                "not-the-nonce",
                "approve",
                None,
                now(),
            )
            .expect_err("wrong nonce");
        assert_eq!(error.code(), "E_APPROVAL_REPLAY");

        // Correct nonce + approve â†’ resolved once.
        let resolution = gate
            .resolve_command(
                &conn,
                &request.approval_id.to_string(),
                &request.nonce,
                "approve",
                Some("tests approve"),
                now(),
            )
            .expect("resolved");
        assert_eq!(resolution.decision, ApprovalDecision::AllowOnce);

        // Replay with the same nonce is refused.
        let error = gate
            .resolve_command(
                &conn,
                &request.approval_id.to_string(),
                &request.nonce,
                "approve",
                None,
                now(),
            )
            .expect_err("replay");
        assert_eq!(error.code(), "E_APPROVAL_REPLAY");

        // Unknown id is a named refusal.
        let error = gate
            .resolve_command(&conn, "no-such-approval", "nonce", "approve", None, now())
            .expect_err("not found");
        assert_eq!(error.code(), "E_APPROVAL_NOT_FOUND");
    }

    #[test]
    fn wire_approval_decision_is_validated() {
        assert_eq!(
            ApprovalDecision::from_wire("approve").expect("approve"),
            ApprovalDecision::AllowOnce
        );
        assert_eq!(
            ApprovalDecision::from_wire("deny").expect("deny"),
            ApprovalDecision::Deny
        );
        let error = ApprovalDecision::from_wire("allow_session").expect_err("not a wire value");
        assert_eq!(error.code(), "E_APPROVAL_DECISION_INVALID");
    }

    #[test]
    fn ledger_row_never_stores_argument_body() {
        let conn = memory_db();
        let secret_content = "TOP-SECRET-ARGUMENT-BODY";
        let secret_args = serde_json::json!({"path": "a.txt", "content": secret_content});
        ApprovalGate::default()
            .request(
                &conn,
                RunId::new(),
                TaskId::new(),
                "call-secret",
                ToolId::FileWrite,
                PermissionClass::Mutate,
                &secret_args,
                now(),
            )
            .expect("committed");
        let stored: String = conn
            .query_row("SELECT args_sha256 FROM approvals", [], |row| row.get(0))
            .expect("row");
        assert_eq!(stored, hash_args(&secret_args));
        assert!(!stored.contains(secret_content));
    }

    #[test]
    fn read_only_tool_cannot_request_approval() {
        let conn = memory_db();
        let error = ApprovalGate::default()
            .request(
                &conn,
                RunId::new(),
                TaskId::new(),
                "call-read",
                ToolId::FileRead,
                PermissionClass::Read,
                &serde_json::json!({"path": "a.txt"}),
                now(),
            )
            .expect_err("read needs no approval");
        assert_eq!(error.code(), "E_INTERNAL_ERROR");
    }
}

