//! Approval / consent gate (Phase 6 WPs 6.6/6.11/6.13, CANONICAL C18).
//!
//! Contract (EXT-R5-04):
//! - An `approval` row is committed **before any side effect**; the request
//!   binds the exact arguments via `args_sha256` (never the argument body).
//! - `SQLite UNIQUE(nonce)` + single-use `decision` make a replayed nonce a
//!   named refusal `E_APPROVAL_REPLAY` (one execution, never two).
//! - TTL expiry (`APPROVAL_TTL` 120 s, PROPOSED-POLICY) returns
//!   `E_APPROVAL_TIMEOUT` with zero side effects; an `allow_once` decision made
//!   inside the TTL is executable only before `expires_at` (it is consumed as
//!   `expired` instead of executing late).
//! - `AllowSession` is scoped to `(run_id, tool, args_sha256)` and is
//!   intentionally non-expiring.
//! - Bound verification additionally matches `approval_id` and
//!   `tool_call_id`, so a same-args call cannot consume another call's grant.
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
use crate::tool::{PermissionClass, ToolId, ToolRegistry};

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
/// The one-pending-per-call unique index is created only when the legacy table
/// contains no duplicate pending `(run_id, tool_call_id)` groups. Duplicates are
/// never deleted (audit truth); migration instead refuses with the existing
/// unique-violation refusal so the boot log names the condition and a later boot
/// (after the boot orphan reaper has consumed pending rows) can create the index.
///
/// # Errors
/// [`Refusal::InternalError`] when SQLite rejects the base DDL; the existing
/// [`Refusal::ApprovalReplay`] refusal when duplicate pending rows block the
/// unique partial index.
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
            reason          TEXT,
            executed_at     TEXT
        );
        CREATE INDEX IF NOT EXISTS approvals_run_pending
            ON approvals (run_id, tool_name, args_sha256, decision);
        CREATE INDEX IF NOT EXISTS approvals_session
            ON approvals (run_id, tool_name, args_sha256, decision);",
    )?;
    let duplicate_groups: i64 = conn.query_row(
        "SELECT COUNT(*) FROM (
             SELECT 1 FROM approvals WHERE decision IS NULL
             GROUP BY run_id, tool_call_id HAVING COUNT(*) > 1
         )",
        [],
        |row| row.get(0),
    )?;
    if duplicate_groups > 0 {
        return Err(ToolError::new(
            Refusal::ApprovalReplay,
            format!(
                "{duplicate_groups} duplicate pending approval group(s) for one \
                 (run_id, tool_call_id) block the one-pending-per-call index; audit/expire \
                 them and retry (no rows were deleted)"
            ),
        ));
    }
    match conn.execute(
        "CREATE UNIQUE INDEX IF NOT EXISTS approvals_one_pending_per_call
            ON approvals (run_id, tool_call_id) WHERE decision IS NULL",
        [],
    ) {
        Ok(_) => Ok(()),
        Err(error) if is_unique_violation(&error) => Err(ToolError::new(
            Refusal::ApprovalReplay,
            format!(
                "duplicate pending approval rows block the one-pending-per-call index; \
                 no rows were deleted ({error})"
            ),
        )),
        Err(error) => Err(error.into()),
    }
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

/// Loads one committed approval request by id.
///
/// Public for read-only replay projections (core `pending_approval_events_all`):
/// it never consumes a nonce, resolves, executes or ledgers.
///
/// # Errors
/// [`Refusal::InternalError`] for an unreadable/malformed stored row.
pub fn load_request(conn: &Connection, approval_id: &str) -> Result<Option<ApprovalRequest>, ToolError> {
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
        "control" => PermissionClass::Control,
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
/// - [`Refusal::ApprovalReplay`] when the stored decision is a non-expired
///   decision (the nonce already consumed a decision);
/// - [`Refusal::ApprovalTimeout`] when the request is past its TTL, or when the
///   stored decision is `expired` (a TTL or boot reaper already consumed it).
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
        Some(Some(stored)) => {
            if stored == "expired" {
                return Err(expired_decision_refusal());
            }
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

/// Compares the presented one-time nonce with the stored one without an early exit.
///
/// # Threat model
/// The nonce is a uuid v7 string (fixed 36-byte hyphenated form) that authorizes a
/// side effect, and it arrives over the session-authenticated WebSocket. A byte-wise
/// `==` returns on the first differing byte, which gives a remote timing oracle on a
/// capability value. This comparison touches every byte of both inputs and folds any
/// length mismatch into the same accumulator before returning, so only the input
/// length class is observable (the expected length is fixed and public). Length is
/// not secret: a wrong-length nonce can never match. This is deliberately not a
/// general-purpose constant-time primitive for variable-length secrets.
///
/// The result is used only to decide whether to return the replay refusal; the value
/// is never logged or embedded in the refusal message.
fn nonce_matches(stored: &str, presented: &str) -> bool {
    let stored = stored.as_bytes();
    let presented = presented.as_bytes();
    let mut diff = (stored.len() ^ presented.len()) as u32;
    let max_len = stored.len().max(presented.len());
    for index in 0..max_len {
        let left = stored.get(index).copied().unwrap_or(0);
        let right = presented.get(index).copied().unwrap_or(0);
        diff |= u32::from(left ^ right);
    }
    diff == 0
}

/// Resolves a pending approval via the frozen `APPROVAL_RESOLVE` wire shape:
/// `approval_id` + one-time `nonce` + `decision` (`approve`/`deny`) + optional
/// reason.
///
/// The nonce binds the decision to exactly one presented request; a mismatched
/// nonce is refused without revealing anything about the stored request, and a
/// used nonce can never be replayed. The comparison is performed by
/// [`nonce_matches`] rather than `==` (the nonce authorizes a side effect, so it is
/// treated as a capability value, not an ordinary string).
///
/// # Errors
/// - [`Refusal::ApprovalRequired`] when no approval with that id exists (the
///   contract patch maps this to `E_APPROVAL_NOT_FOUND` once the variant lands);
/// - [`Refusal::ApprovalReplay`] for a mismatched/already-used nonce whose
///   stored decision is not `expired`;
/// - [`Refusal::ApprovalTimeout`] when the request is past its TTL, or when the
///   stored decision is `expired` (a TTL or boot reaper already consumed it);
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
    if !nonce_matches(&request.nonce, nonce) {
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
        Some(Some(stored)) => {
            if stored == "expired" {
                return Err(expired_decision_refusal());
            }
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

/// The refusal for a row already consumed as `expired`: a TTL or boot reaper
/// ended it, so the honest answer is a timeout, not a replay/forgery signal.
fn expired_decision_refusal() -> ToolError {
    ToolError::new(
        Refusal::ApprovalTimeout,
        "this approval was already consumed as expired (TTL or boot restart); no side effect \
         was performed and a late decision cannot resurrect it",
    )
}

/// Marks one pending approval expired (live-wait timeout path).
///
/// Returns `true` only when this call won the `decision IS NULL` CAS.
///
/// # Errors
/// Database failure, reported as `E_INTERNAL_ERROR`.
pub fn expire(conn: &Connection, approval_id: &str, now_unix_secs: u64) -> Result<bool, ToolError> {
    let decided_at = Timestamp::from_unix_secs(now_unix_secs);
    let changed = conn.execute(
        "UPDATE approvals SET decision = 'expired', decided_at = ?1
         WHERE approval_id = ?2 AND decision IS NULL",
        params![decided_at.as_str(), approval_id],
    )?;
    Ok(changed == 1)
}

/// Marks one pending approval cancelled (run/task cancellation path).
///
/// Returns `true` only when this call won the `decision IS NULL` CAS.
///
/// # Errors
/// Database failure, reported as `E_INTERNAL_ERROR`.
pub fn cancel_pending(
    conn: &Connection,
    approval_id: &str,
    now_unix_secs: u64,
) -> Result<bool, ToolError> {
    let decided_at = Timestamp::from_unix_secs(now_unix_secs);
    let changed = conn.execute(
        "UPDATE approvals SET decision = 'cancelled', decided_at = ?1
         WHERE approval_id = ?2 AND decision IS NULL",
        params![decided_at.as_str(), approval_id],
    )?;
    Ok(changed == 1)
}

/// Reads the stored decision label for one approval (`None` when undecided/unknown).
///
/// # Errors
/// Database failure, reported as `E_INTERNAL_ERROR`.
pub fn decision_label(conn: &Connection, approval_id: &str) -> Result<Option<String>, ToolError> {
    conn.query_row(
        "SELECT decision FROM approvals WHERE approval_id = ?1",
        params![approval_id],
        |row| row.get::<_, Option<String>>(0),
    )
    .optional()
    .map(|row| row.flatten())
    .map_err(ToolError::from)
}

/// Number of approval rows ever created for a run (all decisions).
///
/// # Errors
/// Database failure, reported as `E_INTERNAL_ERROR`.
pub fn count_for_run(conn: &Connection, run_id: RunId) -> Result<u64, ToolError> {
    let count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM approvals WHERE run_id = ?1",
        params![run_id.to_string()],
        |row| row.get(0),
    )?;
    u64::try_from(count).map_err(|_| {
        ToolError::new(
            Refusal::InternalError,
            "approval row count exceeded u64 range",
        )
    })
}

/// Consumes every still-pending approval at boot.
///
/// No in-process waiter survives a restart, so every pending row is
/// restart-orphaned: the TTL is deliberately not consulted (the deadline may
/// still be in the future when the process dies). Marking the rows `expired`
/// (never approved) keeps a late decision from being accepted as a resolution;
/// the resolve path reports its named timeout refusal instead.
///
/// This is distinct from [`expire_pending`] (the TTL reaper used for a live
/// process): boot orphan expiry is unconditional.
///
/// # Errors
/// Database failure, reported as `E_INTERNAL_ERROR`.
pub fn expire_orphaned_pending(conn: &Connection, now_unix_secs: u64) -> Result<u64, ToolError> {
    let decided_at = Timestamp::from_unix_secs(now_unix_secs);
    let changed = conn.execute(
        "UPDATE approvals SET decision = 'expired', decided_at = ?1
         WHERE decision IS NULL",
        params![decided_at.as_str()],
    )?;
    u64::try_from(changed).map_err(|_| {
        ToolError::new(
            Refusal::InternalError,
            "approval row count exceeded u64 range",
        )
    })
}

/// Consumes every pending approval whose TTL has elapsed.
///
/// This is the live-process TTL reaper; the boot path uses
/// [`expire_orphaned_pending`] instead because no waiter survives a restart.
/// An expired request is marked `expired` so a late UI decision can never
/// execute it. Returns the number of rows consumed.
///
/// # Errors
/// Database or stored-timestamp failures, reported as `E_INTERNAL_ERROR`.
pub fn expire_pending(conn: &Connection, now_unix_secs: u64) -> Result<u64, ToolError> {
    let mut statement =
        conn.prepare("SELECT approval_id, expires_at FROM approvals WHERE decision IS NULL")?;
    let rows = statement
        .query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        })?
        .collect::<Result<Vec<_>, _>>()?;
    let mut expired = 0_u64;
    for (approval_id, raw_expires) in rows {
        let expires = Timestamp::parse(&raw_expires).map_err(|error| {
            ToolError::new(
                Refusal::InternalError,
                format!("stored approval has an invalid expires_at: {error}"),
            )
        })?;
        let is_expired = expires
            .unix_secs()
            .map(|at| now_unix_secs >= at)
            .unwrap_or(true);
        if !is_expired {
            continue;
        }
        // Reuse the single per-row CAS primitive so the TTL/boot paths share one
        // `decision IS NULL` predicate (WP-01 merge requirement).
        if expire(conn, &approval_id, now_unix_secs)? {
            expired = expired.saturating_add(1);
        }
    }
    Ok(expired)
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

/// Approval ids for every committed, undecided approval whose run is still
/// non-terminal, oldest first, bounded by `limit`.
///
/// Read-only replay projection (WP-05): never consumes a nonce, resolves,
/// executes or ledgers. Callers load full requests with [`load_request`] and
/// skip expired rows before publishing.
///
/// # Errors
/// [`Refusal::InternalError`] on a SQLite failure or a nonsensical limit.
pub fn pending_ids_for_active_runs(
    conn: &Connection,
    limit: usize,
) -> Result<Vec<String>, ToolError> {
    if limit == 0 {
        return Ok(Vec::new());
    }
    let capped = i64::try_from(limit).map_err(|_| {
        ToolError::new(
            Refusal::InternalError,
            "replay limit does not fit into a SQLite LIMIT parameter",
        )
    })?;
    let mut statement = conn.prepare(
        "SELECT a.approval_id
           FROM approvals a
           JOIN runs r ON r.id = a.run_id
          WHERE a.decision IS NULL
            AND r.status NOT IN ('completed','failed','cancelled','killed','interrupted')
          ORDER BY a.created_at ASC
          LIMIT ?1",
    )?;
    let ids = statement
        .query_map(params![capped], |row| row.get::<_, String>(0))?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    Ok(ids)
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
    /// `AllowSession` stays intentionally non-expiring and is scoped by
    /// `(run, tool, args_sha256)`. A single-use `AllowOnce` grant is valid only
    /// while its `expires_at` is still in the future: a decision made inside the
    /// TTL cannot be executed arbitrarily later. An expired unexecuted
    /// `allow_once` row is consumed as `expired` (post-decision state) so the
    /// live-wait timeout path sees the terminal label instead of re-applying an
    /// executable grant.
    ///
    /// This unbound lookup is retained for existing callers. New execution
    /// paths should use [`Self::verify_grant_bound`], which additionally binds
    /// the approval id and provider tool-call id.
    ///
    /// # Errors
    /// [`Refusal::ApprovalRequired`] when the approval is missing, denied or
    /// bound to different arguments; [`Refusal::ApprovalTimeout`] when a
    /// single-use grant expired; [`Refusal::ApprovalReplay`] when it was already
    /// executed.
    pub fn verify_grant(
        &self,
        conn: &Connection,
        run_id: RunId,
        tool: ToolId,
        args: &serde_json::Value,
        now_unix_secs: u64,
    ) -> Result<(), ToolError> {
        let digest = hash_args(args);
        // AllowSession is scoped by (run, tool, args) and carries no second
        // timer: it stays valid for the rest of the run.
        if session_allowed(conn, run_id, tool, &digest)? {
            return Ok(());
        }

        // Single-use AllowOnce grant: exactly one execution may consume it, and
        // only before the approval TTL elapsed.
        let allowance: Option<(String, Option<String>, String)> = conn
            .query_row(
                "SELECT approval_id, executed_at, expires_at FROM approvals
                 WHERE run_id = ?1 AND tool_name = ?2 AND args_sha256 = ?3
                   AND decision = 'allow_once'
                 ORDER BY decided_at ASC LIMIT 1",
                params![run_id.to_string(), tool.id(), digest],
                |row| {
                    Ok((
                        row.get::<_, String>(0)?,
                        row.get::<_, Option<String>>(1)?,
                        row.get::<_, String>(2)?,
                    ))
                },
            )
            .optional()?;
        match allowance {
            Some((approval_id, None, expires_at)) => {
                Self::consume_allow_once(conn, &approval_id, &expires_at, now_unix_secs)
            }
            Some((_, Some(_), _)) => Err(executed_grant_refusal()),
            None => Err(ToolError::new(
                Refusal::ApprovalRequired,
                format!(
                    "no executed approval grant for tool {} with these arguments; \
                     the call must be approved first",
                    tool.id()
                ),
            )),
        }
    }

    /// Verifies and consumes the grant bound to one exact approval request and
    /// provider tool call.
    ///
    /// Unlike [`Self::verify_grant`], a same-arguments call cannot consume a
    /// different call's single-use grant: `approval_id`, `tool_call_id`,
    /// `run_id`, tool and `args_sha256` must all match the committed request.
    /// `AllowSession` remains non-expiring for the exact bound
    /// `(run, tool, tool_call_id, args)` request; `AllowOnce` is single-use and
    /// only valid before its `expires_at`.
    ///
    /// # Errors
    /// `E_APPROVAL_NOT_FOUND` for an unknown approval id;
    /// [`Refusal::ApprovalRequired`] for a binding mismatch, no decision,
    /// denial or cancellation; [`Refusal::ApprovalTimeout`] for an expired
    /// single-use grant; [`Refusal::ApprovalReplay`] for a consumed grant.
    pub fn verify_grant_bound(
        &self,
        conn: &Connection,
        approval_id: &str,
        tool_call_id: &str,
        run_id: RunId,
        tool: ToolId,
        args: &serde_json::Value,
        now_unix_secs: u64,
    ) -> Result<(), ToolError> {
        let request = load_request(conn, approval_id)?.ok_or_else(|| {
            ToolError::new(
                Refusal::ApprovalNotFound,
                format!("no approval request with id {approval_id}"),
            )
        })?;
        if request.run_id != run_id
            || request.tool != tool
            || request.tool_call_id != tool_call_id
            || request.args_sha256 != hash_args(args)
        {
            return Err(ToolError::new(
                Refusal::ApprovalRequired,
                "the execution grant is bound to a different (run, tool, tool_call_id, args); \
                 a grant cannot be consumed by another call",
            ));
        }

        let row: Option<(Option<String>, Option<String>, String)> = conn
            .query_row(
                "SELECT decision, executed_at, expires_at FROM approvals
                 WHERE approval_id = ?1",
                params![approval_id],
                |row| {
                    Ok((
                        row.get::<_, Option<String>>(0)?,
                        row.get::<_, Option<String>>(1)?,
                        row.get::<_, String>(2)?,
                    ))
                },
            )
            .optional()?;
        let Some((decision, executed_at, expires_at)) = row else {
            return Err(ToolError::new(
                Refusal::ApprovalNotFound,
                format!("no approval request with id {approval_id}"),
            ));
        };
        match decision.as_deref() {
            Some("allow_session") => Ok(()),
            Some("allow_once") => {
                if executed_at.is_some() {
                    return Err(executed_grant_refusal());
                }
                Self::consume_allow_once(conn, approval_id, &expires_at, now_unix_secs)
            }
            Some("expired") => Err(expired_decision_refusal()),
            Some("cancelled") => Err(ToolError::new(
                Refusal::ApprovalRequired,
                "this approval was cancelled; no side effect may execute",
            )),
            Some("deny") => Err(ToolError::new(
                Refusal::ApprovalRequired,
                "this approval was denied; no side effect may execute",
            )),
            Some(other) => Err(ToolError::new(
                Refusal::ApprovalRequired,
                format!("stored approval decision {other:?} is not an executable grant"),
            )),
            None => Err(ToolError::new(
                Refusal::ApprovalRequired,
                "this approval has no decision yet; the call cannot execute",
            )),
        }
    }

    /// Consumes one unexecuted `allow_once` grant if it is still valid.
    fn consume_allow_once(
        conn: &Connection,
        approval_id: &str,
        expires_at: &str,
        now_unix_secs: u64,
    ) -> Result<(), ToolError> {
        let expires = Timestamp::parse(expires_at).map_err(|error| {
            ToolError::new(
                Refusal::InternalError,
                format!("stored approval has an invalid expires_at: {error}"),
            )
        })?;
        let expired = expires
            .unix_secs()
            .map(|at| now_unix_secs >= at)
            .unwrap_or(true);
        if expired {
            let _ = expire_decided_allow_once(conn, approval_id, now_unix_secs)?;
            return Err(ToolError::new(
                Refusal::ApprovalTimeout,
                format!(
                    "the single-use approval expired at {expires_at}; it cannot execute after \
                     the TTL"
                ),
            ));
        }
        let executed_at = Timestamp::from_unix_secs(now_unix_secs);
        let changed = conn.execute(
            "UPDATE approvals SET executed_at = ?1
             WHERE approval_id = ?2 AND decision = 'allow_once' AND executed_at IS NULL",
            params![executed_at.as_str(), approval_id],
        )?;
        if changed == 1 {
            Ok(())
        } else {
            Err(executed_grant_refusal())
        }
    }
}

/// Refusal for a single-use grant that was already executed.
fn executed_grant_refusal() -> ToolError {
    ToolError::new(
        Refusal::ApprovalReplay,
        "this single-use approval was already executed; replay is refused",
    )
}

/// Post-decision terminal state for an `allow_once` grant whose TTL elapsed
/// before execution.
///
/// The live-wait timeout path reads the stored decision label; flipping an
/// approved-but-unexecuted grant to `expired` means that path finalizes a
/// timeout instead of re-applying an executable grant after the deadline.
fn expire_decided_allow_once(
    conn: &Connection,
    approval_id: &str,
    now_unix_secs: u64,
) -> Result<bool, ToolError> {
    let decided_at = Timestamp::from_unix_secs(now_unix_secs);
    let changed = conn.execute(
        "UPDATE approvals SET decision = 'expired', decided_at = ?1
         WHERE approval_id = ?2 AND decision = 'allow_once' AND executed_at IS NULL",
        params![decided_at.as_str(), approval_id],
    )?;
    Ok(changed == 1)
}

/// Commits a model-originated side-effect approval request (W12 tool loop).
///
/// Maps the wire tool name to its [`ToolId`] and registered permission class,
/// then delegates to [`ApprovalGate::request`], which fills the single-use
/// `nonce`, the `args_sha256` binding via [`hash_args`], the creation time and
/// the `expires_at` value from the gate TTL, and inserts the row **before any
/// side effect**. Only `mutate` / `execute` / `network` tools can request
/// approval; read-only tools are refused by the gate itself.
///
/// # Errors
/// - [`Refusal::ToolNotFound`] for a name outside the frozen tool vocabulary;
/// - [`Refusal::SchemaInvalid`] for an empty `tool_call_id` (the event contract
///   requires the provider call id);
/// - [`Refusal::InternalError`] when the tool has no registered permission class
///   or is read-only (no approval is needed or permitted);
/// - [`Refusal::ApprovalReplay`] when the generated nonce/approval id collides.
pub fn request_tool_approval(
    gate: &ApprovalGate,
    conn: &Connection,
    run_id: RunId,
    task_id: TaskId,
    tool_call_id: &str,
    tool: &str,
    args: &serde_json::Value,
    now_unix_secs: u64,
) -> Result<ApprovalRequest, ToolError> {
    if tool_call_id.trim().is_empty() {
        return Err(ToolError::new(
            Refusal::SchemaInvalid,
            "an approval request must carry the provider tool_call_id (empty id refused)",
        ));
    }
    let id = ToolId::from_id(tool).ok_or_else(|| {
        ToolError::new(
            Refusal::ToolNotFound,
            format!("tool {tool:?} is not in the frozen tool vocabulary"),
        )
    })?;
    let permission_class = ToolRegistry::standard()
        .contract(id)
        .map(|contract| contract.permission_class)
        .ok_or_else(|| {
            ToolError::new(
                Refusal::InternalError,
                format!("tool {} has no registered permission class", id.id()),
            )
        })?;
    gate.request(
        conn,
        run_id,
        task_id,
        tool_call_id.to_owned(),
        id,
        permission_class,
        args,
        now_unix_secs,
    )
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
    fn expire_pending_consumes_only_elapsed_requests() {
        let conn = memory_db();
        let request = begin(&conn);
        let expiry = request.expires_at.unix_secs().expect("expires");
        assert_eq!(
            expire_pending(&conn, expiry.saturating_sub(1)).expect("before expiry"),
            0,
            "a live approval is not consumed"
        );
        assert_eq!(
            pending_for_run(&conn, request.run_id).expect("pending").len(),
            1
        );
        assert_eq!(
            expire_pending(&conn, expiry).expect("at expiry"),
            1,
            "the elapsed approval is consumed once"
        );
        assert!(
            pending_for_run(&conn, request.run_id)
                .expect("pending after expiry")
                .is_empty(),
            "expired rows are no longer pending"
        );
        // A late decision cannot resurrect the consumed request; because the
        // row was consumed as `expired` (not replay/forgery), the named refusal
        // is the honest timeout.
        let error = ApprovalGate::default()
            .decide(
                &conn,
                &request.approval_id.to_string(),
                ApprovalDecision::AllowOnce,
                expiry.saturating_add(1),
            )
            .expect_err("expired approval cannot be decided");
        assert_eq!(error.code(), "E_APPROVAL_TIMEOUT");
    }

    #[test]
    fn expire_orphaned_pending_consumes_live_requests_at_boot() {
        let conn = memory_db();
        let request = begin(&conn);
        let expiry = request.expires_at.unix_secs().expect("expires");

        // Boot runs before the TTL elapsed: every pending row is orphaned because
        // no in-process waiter survives a restart, so the boot reaper must consume
        // it even though the wall-clock TTL is still in the future.
        assert_eq!(
            expire_orphaned_pending(&conn, expiry.saturating_sub(1)).expect("boot reaper"),
            1
        );
        assert!(
            pending_for_run(&conn, request.run_id)
                .expect("pending after boot")
                .is_empty(),
            "no resolvable row may survive boot"
        );
        let error = ApprovalGate::default()
            .decide(
                &conn,
                &request.approval_id.to_string(),
                ApprovalDecision::AllowOnce,
                expiry.saturating_sub(1),
            )
            .expect_err("late decision after boot expiry");
        assert_eq!(error.code(), "E_APPROVAL_TIMEOUT");
    }

    #[test]
    fn expire_cancel_and_decision_labels_are_cas_guarded() {
        let conn = memory_db();
        let gate = ApprovalGate::default();
        let run_id = RunId::new();
        let task_id = TaskId::new();
        let request = gate
            .request(
                &conn,
                run_id,
                task_id,
                "call-label-1",
                ToolId::FileWrite,
                PermissionClass::Mutate,
                &args(),
                now(),
            )
            .expect("request");
        assert_eq!(
            decision_label(&conn, &request.approval_id.to_string()).expect("label"),
            None
        );
        assert_eq!(
            count_for_run(&conn, run_id).expect("count"),
            1,
            "one approval row for the run"
        );
        assert!(expire(&conn, &request.approval_id.to_string(), now()).expect("expire"));
        assert!(
            !expire(&conn, &request.approval_id.to_string(), now()).expect("second expire"),
            "the CAS wins exactly once"
        );
        assert_eq!(
            decision_label(&conn, &request.approval_id.to_string()).expect("label"),
            Some("expired".to_owned())
        );

        let second = gate
            .request(
                &conn,
                run_id,
                task_id,
                "call-label-2",
                ToolId::FileWrite,
                PermissionClass::Mutate,
                &args(),
                now(),
            )
            .expect("second request");
        assert!(cancel_pending(&conn, &second.approval_id.to_string(), now()).expect("cancel"));
        assert_eq!(
            decision_label(&conn, &second.approval_id.to_string()).expect("label"),
            Some("cancelled".to_owned())
        );
        assert_eq!(count_for_run(&conn, run_id).expect("count"), 2);
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
        // A single-use grant is consumable exactly once.
        ApprovalGate::default()
            .verify_grant(&conn, request.run_id, ToolId::FileWrite, &args(), now())
            .expect("single-use grant verifies once");
        let replay = ApprovalGate::default()
            .verify_grant(&conn, request.run_id, ToolId::FileWrite, &args(), now())
            .expect_err("second execution of the same grant is refused");
        assert_eq!(replay.code(), "E_APPROVAL_REPLAY");
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
        // A late decision can never resurrect the expired request; the row was
        // consumed as `expired`, so the named refusal is a timeout.
        assert_eq!(
            gate.decide(
                &conn,
                &request.approval_id.to_string(),
                ApprovalDecision::AllowOnce,
                expiry + 2,
            )
            .expect_err("consumed expired row")
            .code(),
            "E_APPROVAL_TIMEOUT"
        );
    }

    #[test]
    fn allow_once_grant_is_not_executable_after_ttl() {
        let conn = memory_db();
        let request = begin(&conn);
        let gate = ApprovalGate::default();
        let expiry = request.expires_at.unix_secs().expect("expires");
        gate.decide(
            &conn,
            &request.approval_id.to_string(),
            ApprovalDecision::AllowOnce,
            now(),
        )
        .expect("approved before expiry");

        // The decision was committed inside the TTL; execution after the
        // deadline must refuse, and the post-decision state must become
        // terminal `expired` so the timeout path cannot re-apply it.
        let error = gate
            .verify_grant(&conn, request.run_id, ToolId::FileWrite, &args(), expiry)
            .expect_err("expired allow_once cannot execute");
        assert_eq!(error.code(), "E_APPROVAL_TIMEOUT");
        assert_eq!(
            decision_label(&conn, &request.approval_id.to_string()).expect("label"),
            Some("expired".to_owned())
        );
        let error = gate
            .verify_grant(
                &conn,
                request.run_id,
                ToolId::FileWrite,
                &args(),
                expiry.saturating_add(1),
            )
            .expect_err("expired grant is never resurrected");
        assert_eq!(error.code(), "E_APPROVAL_REQUIRED");
    }

    #[test]
    fn allow_session_grant_is_intentionally_non_expiring() {
        let conn = memory_db();
        let request = begin(&conn);
        let gate = ApprovalGate::default();
        let expiry = request.expires_at.unix_secs().expect("expires");
        gate.decide(
            &conn,
            &request.approval_id.to_string(),
            ApprovalDecision::AllowSession,
            now(),
        )
        .expect("session approved");

        for at in [expiry, expiry.saturating_add(86_400)] {
            gate.verify_grant(&conn, request.run_id, ToolId::FileWrite, &args(), at)
                .unwrap_or_else(|error| panic!("allow_session must stay valid at {at}: {error}"));
        }
    }

    #[test]
    fn bound_grant_refuses_other_tool_call_and_unknown_approval() {
        let conn = memory_db();
        let request = begin(&conn);
        let gate = ApprovalGate::default();
        gate.decide(
            &conn,
            &request.approval_id.to_string(),
            ApprovalDecision::AllowOnce,
            now(),
        )
        .expect("approved");

        let unknown = gate
            .verify_grant_bound(
                &conn,
                "00000000-0000-7000-8000-000000000000",
                &request.tool_call_id,
                request.run_id,
                ToolId::FileWrite,
                &args(),
                now(),
            )
            .expect_err("unknown approval");
        assert_eq!(unknown.code(), "E_APPROVAL_NOT_FOUND");

        let wrong_call = gate
            .verify_grant_bound(
                &conn,
                &request.approval_id.to_string(),
                "call-other",
                request.run_id,
                ToolId::FileWrite,
                &args(),
                now(),
            )
            .expect_err("tool_call_id binding");
        assert_eq!(wrong_call.code(), "E_APPROVAL_REQUIRED");

        gate.verify_grant_bound(
            &conn,
            &request.approval_id.to_string(),
            "call-1",
            request.run_id,
            ToolId::FileWrite,
            &args(),
            now(),
        )
        .expect("the exact binding verifies once");
        let replay = gate
            .verify_grant_bound(
                &conn,
                &request.approval_id.to_string(),
                "call-1",
                request.run_id,
                ToolId::FileWrite,
                &args(),
                now(),
            )
            .expect_err("single-use replay");
        assert_eq!(replay.code(), "E_APPROVAL_REPLAY");
    }

    #[test]
    fn bound_grants_are_per_approval_not_only_per_args() {
        let conn = memory_db();
        let gate = ApprovalGate::default();
        let run = RunId::new();
        let mut requests = Vec::new();
        for (tool_call_id, decided_at) in [("call-a", 1_u64), ("call-b", 2_u64)] {
            let request = gate
                .request(
                    &conn,
                    run,
                    TaskId::new(),
                    tool_call_id,
                    ToolId::FileWrite,
                    PermissionClass::Mutate,
                    &args(),
                    now(),
                )
                .expect("request");
            gate.decide(
                &conn,
                &request.approval_id.to_string(),
                ApprovalDecision::AllowOnce,
                now().saturating_add(decided_at),
            )
            .expect("approved");
            requests.push(request);
        }

        // Consume the later-decided `call-b` grant first. The legacy unbound
        // lookup would select `call-a`; the bound path consumes exactly the
        // grant named by the caller.
        gate.verify_grant_bound(
            &conn,
            &requests[1].approval_id.to_string(),
            "call-b",
            run,
            ToolId::FileWrite,
            &args(),
            now(),
        )
        .expect("call-b grant");
        gate.verify_grant_bound(
            &conn,
            &requests[0].approval_id.to_string(),
            "call-a",
            run,
            ToolId::FileWrite,
            &args(),
            now(),
        )
        .expect("call-a grant remains independently valid");
        let replay = gate
            .verify_grant_bound(
                &conn,
                &requests[1].approval_id.to_string(),
                "call-b",
                run,
                ToolId::FileWrite,
                &args(),
                now(),
            )
            .expect_err("call-b is single-use");
        assert_eq!(replay.code(), "E_APPROVAL_REPLAY");
    }

    #[test]
    fn bound_grant_refuses_expired_allow_once_and_allows_session_past_ttl() {
        let conn = memory_db();
        let gate = ApprovalGate::default();
        let request = begin(&conn);
        let expiry = request.expires_at.unix_secs().expect("expires");
        gate.decide(
            &conn,
            &request.approval_id.to_string(),
            ApprovalDecision::AllowOnce,
            now(),
        )
        .expect("approved");
        let error = gate
            .verify_grant_bound(
                &conn,
                &request.approval_id.to_string(),
                "call-1",
                request.run_id,
                ToolId::FileWrite,
                &args(),
                expiry,
            )
            .expect_err("expired bound grant");
        assert_eq!(error.code(), "E_APPROVAL_TIMEOUT");

        let session = gate
            .request(
                &conn,
                RunId::new(),
                TaskId::new(),
                "call-session",
                ToolId::FileWrite,
                PermissionClass::Mutate,
                &args(),
                now(),
            )
            .expect("session request");
        let session_expiry = session.expires_at.unix_secs().expect("expires");
        gate.decide(
            &conn,
            &session.approval_id.to_string(),
            ApprovalDecision::AllowSession,
            now(),
        )
        .expect("session approved");
        gate.verify_grant_bound(
            &conn,
            &session.approval_id.to_string(),
            "call-session",
            session.run_id,
            ToolId::FileWrite,
            &args(),
            session_expiry.saturating_add(86_400),
        )
        .expect("allow_session stays valid past its original TTL");
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

        // Correct nonce + approve Ã¢â€ â€™ resolved once.
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

    #[test]
    fn request_tool_approval_fills_all_six_fields_and_commits_the_row() {
        let conn = memory_db();
        let gate = ApprovalGate::default();
        let run = RunId::new();
        let task = TaskId::new();
        let args = serde_json::json!({"path": "notes.txt", "content": "hello", "expected_sha256": null});

        let request = request_tool_approval(
            &gate, &conn, run, task, "call-approval-1", "file_write", &args, now(),
        )
        .expect("approval request committed");

        assert_eq!(request.run_id, run);
        assert_eq!(request.task_id, task);
        assert_eq!(request.tool_call_id, "call-approval-1");
        assert_eq!(request.tool, ToolId::FileWrite);
        assert_eq!(request.permission_class, PermissionClass::Mutate);
        assert_eq!(request.args_sha256, hash_args(&args));
        assert_eq!(request.args_sha256.len(), 64);
        assert!(!request.args_sha256.is_empty());
        assert_eq!(
            request.created_at.unix_secs().expect("created"),
            now(),
            "created_at uses the supplied clock"
        );
        assert_eq!(
            request.expires_at.unix_secs().expect("expires"),
            now() + APPROVAL_TTL_SECS,
            "expires_at = created_at + TTL"
        );
        assert_eq!(request.nonce.len(), 36, "uuid v7 nonce");
        assert_ne!(request.nonce, request.approval_id.to_string());

        // The row is committed before any side effect; pending until resolved.
        let pending = pending_for_run(&conn, run).expect("pending");
        assert_eq!(pending.len(), 1);
        assert_eq!(pending[0], request);
        let decision: Option<String> = conn
            .query_row("SELECT decision FROM approvals WHERE approval_id = ?1", [request.approval_id.to_string()], |row| row.get(0))
            .expect("row");
        assert!(decision.is_none(), "new request is undecided");

        // A second request for the same call is a different nonce (single-use rows).
        let second = request_tool_approval(
            &gate, &conn, run, task, "call-approval-2", "shell_exec",
            &serde_json::json!({"program": "git.exe", "args": ["--version"]}), now(),
        )
        .expect("second approval");
        assert_eq!(second.permission_class, PermissionClass::Execute);
        assert_ne!(second.nonce, request.nonce);
        assert_ne!(second.approval_id, request.approval_id);
    }

    #[test]
    fn request_tool_approval_refuses_unknown_read_only_and_empty_ids() {
        let conn = memory_db();
        let gate = ApprovalGate::default();
        let run = RunId::new();
        let task = TaskId::new();

        let unknown = request_tool_approval(
            &gate, &conn, run, task, "call-x", "rm_rf", &serde_json::json!({}), now(),
        )
        .expect_err("unknown tool");
        assert_eq!(unknown.code(), "E_TOOL_NOT_FOUND");

        let read_only = request_tool_approval(
            &gate, &conn, run, task, "call-y", "file_read",
            &serde_json::json!({"path": "note.txt"}), now(),
        )
        .expect_err("read-only tools never request approval");
        assert_eq!(read_only.code(), "E_INTERNAL_ERROR");

        let empty = request_tool_approval(
            &gate, &conn, run, task, "   ", "file_write",
            &serde_json::json!({"path": "a.txt", "content": "x"}), now(),
        )
        .expect_err("empty tool_call_id");
        assert_eq!(empty.code(), "E_SCHEMA_INVALID");

        // Nothing was inserted by the refused calls.
        let pending = pending_for_run(&conn, run).expect("pending");
        assert!(pending.is_empty());
    }

    #[test]
    fn request_tool_approval_binds_arguments_without_storing_them() {
        let conn = memory_db();
        let secret = "TOP-SECRET-APPROVAL-BODY";
        let args = serde_json::json!({"path": "a.txt", "content": secret});
        let request = request_tool_approval(
            &ApprovalGate::default(), &conn, RunId::new(), TaskId::new(),
            "call-secret", "file_write", &args, now(),
        )
        .expect("committed");
        assert_eq!(request.args_sha256, hash_args(&args));
        let stored: String = conn
            .query_row("SELECT args_sha256 FROM approvals WHERE approval_id = ?1", [request.approval_id.to_string()], |row| row.get(0))
            .expect("row");
        assert_eq!(stored, request.args_sha256);
        assert!(!stored.contains(secret));
    }

    #[test]
    fn nonce_compare_accepts_only_an_exact_match() {
        let nonce = "01234567-89ab-7cde-9f01-23456789abcd";
        assert!(nonce_matches(nonce, nonce));
        assert!(!nonce_matches(
            nonce,
            "01234567-89ab-7cde-9f01-23456789abce"
        ));
        // Prefix/suffix/length differences must never be accepted.
        assert!(!nonce_matches(nonce, "01234567-89ab-7cde-9f01-23456789abc"));
        assert!(!nonce_matches(nonce, "x01234567-89ab-7cde-9f01-23456789abcd"));
        assert!(!nonce_matches(nonce, ""));
        assert!(!nonce_matches("", nonce));
    }

    #[test]
    fn nonce_mismatch_refusal_never_echoes_either_nonce() {
        let conn = memory_db();
        let request = request_tool_approval(
            &ApprovalGate::default(),
            &conn,
            RunId::new(),
            TaskId::new(),
            "call-nonce-leak",
            "file_write",
            &serde_json::json!({"path": "a.txt", "content": "x"}),
            now(),
        )
        .expect("request approval");
        let presented = "wrong-nonce-value";
        let error = resolve_with_nonce(
            &conn,
            &request.approval_id.to_string(),
            presented,
            "approve",
            None,
            now(),
        )
        .expect_err("mismatched nonce");
        assert_eq!(error.code(), "E_APPROVAL_REPLAY");
        assert!(!error.detail().contains(presented));
        assert!(!error.detail().contains(&request.nonce));
    }
}


