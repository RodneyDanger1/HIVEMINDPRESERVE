//! Executed-eval lesson candidates (CAP-9).
//!
//! A candidate lesson is activated only when it cites two stored eval receipts:
//! a baseline and a candidate, both of which must have passed. Activation
//! appends an `applied_lesson` line to `evolution/LESSONS.jsonl`; the existing
//! boot/prompt loader then admits it as bounded data. Rollback appends an
//! unreferenced `rolled_back` line and removes it from the next prompt reload.
//! Nothing here changes code, tools, caps, or approvals.

use hiverouter_contract::Timestamp;
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};

/// Lesson-candidate schema (v30).
pub const SCHEMA_V30: &str = r#"
CREATE TABLE IF NOT EXISTS lesson_candidates (
    id                TEXT PRIMARY KEY NOT NULL,
    candidate_key     TEXT NOT NULL,
    title             TEXT NOT NULL,
    body              TEXT NOT NULL,
    baseline_eval_id  TEXT NOT NULL REFERENCES eval_runs(id),
    candidate_eval_id TEXT NOT NULL REFERENCES eval_runs(id),
    status            TEXT NOT NULL CHECK (status IN ('pending','active','rolled_back')),
    created_at        TEXT NOT NULL,
    activated_at      TEXT,
    rolled_back_at    TEXT
);
CREATE INDEX IF NOT EXISTS idx_lesson_candidates_status ON lesson_candidates (status, created_at);
"#;

/// Down migration for v30.
pub const SCHEMA_V30_DOWN: &str = "DROP INDEX IF EXISTS idx_lesson_candidates_status; DROP TABLE IF EXISTS lesson_candidates;";

/// Migration descriptor.
#[must_use]
pub const fn learning_migration(version: i64) -> crate::db::Migration {
    crate::db::Migration {
        version,
        name: "lesson_candidates",
        up: SCHEMA_V30,
        down: SCHEMA_V30_DOWN,
    }
}

/// One candidate lesson.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct LessonCandidate {
    /// Candidate id.
    pub id: String,
    /// Short area/key.
    pub candidate_key: String,
    /// Human title/problem.
    pub title: String,
    /// Bounded proposal text injected as data.
    pub body: String,
    /// Baseline eval receipt id.
    pub baseline_eval_id: String,
    /// Candidate eval receipt id.
    pub candidate_eval_id: String,
    /// `pending` | `active` | `rolled_back`.
    pub status: String,
    /// Creation time.
    pub created_at: String,
    /// Activation time.
    pub activated_at: Option<String>,
    /// Rollback time.
    pub rolled_back_at: Option<String>,
}

fn map_candidate(row: &rusqlite::Row<'_>) -> rusqlite::Result<LessonCandidate> {
    Ok(LessonCandidate {
        id: row.get(0)?,
        candidate_key: row.get(1)?,
        title: row.get(2)?,
        body: row.get(3)?,
        baseline_eval_id: row.get(4)?,
        candidate_eval_id: row.get(5)?,
        status: row.get(6)?,
        created_at: row.get(7)?,
        activated_at: row.get(8)?,
        rolled_back_at: row.get(9)?,
    })
}

/// Inserts a pending candidate. Both eval ids must exist and pass.
pub fn create_candidate(
    connection: &Connection,
    candidate: &LessonCandidate,
) -> rusqlite::Result<()> {
    let mut compared: Vec<(String, String, i64)> = Vec::new();
    for eval_id in [&candidate.baseline_eval_id, &candidate.candidate_eval_id] {
        let row: (String, String, i64) = connection.query_row(
            "SELECT suite_id, case_id, passed FROM eval_runs WHERE id = ?1",
            params![eval_id],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        )?;
        if row.2 != 1 {
            return Err(rusqlite::Error::InvalidParameterName(
                "lesson candidate requires two passing eval receipts".to_owned(),
            ));
        }
        compared.push(row);
    }
    if compared[0].0 != compared[1].0 || compared[0].1 != compared[1].1 {
        return Err(rusqlite::Error::InvalidParameterName(
            "baseline and candidate receipts must cover the same suite/case".to_owned(),
        ));
    }
    connection.execute(
        "INSERT INTO lesson_candidates
         (id, candidate_key, title, body, baseline_eval_id, candidate_eval_id,
          status, created_at, activated_at, rolled_back_at)
         VALUES (?1,?2,?3,?4,?5,?6,'pending',?7,NULL,NULL)",
        params![
            candidate.id,
            candidate.candidate_key,
            candidate.title,
            candidate.body,
            candidate.baseline_eval_id,
            candidate.candidate_eval_id,
            candidate.created_at,
        ],
    )?;
    Ok(())
}

/// Returns true only when both cited eval receipts exist and passed.
pub fn evals_passed(
    connection: &Connection,
    baseline_eval_id: &str,
    candidate_eval_id: &str,
) -> rusqlite::Result<bool> {
    let mut passed = 0_i64;
    for eval_id in [baseline_eval_id, candidate_eval_id] {
        passed += connection.query_row(
            "SELECT passed FROM eval_runs WHERE id = ?1",
            params![eval_id],
            |row| row.get::<_, i64>(0),
        )?;
    }
    Ok(passed == 2)
}

/// Lists candidates, newest first.
pub fn list_candidates(
    connection: &Connection,
    limit: u32,
) -> Result<Vec<LessonCandidate>, rusqlite::Error> {
    let mut statement = connection.prepare(
        "SELECT id, candidate_key, title, body, baseline_eval_id, candidate_eval_id,
                status, created_at, activated_at, rolled_back_at
         FROM lesson_candidates ORDER BY created_at DESC, id DESC LIMIT ?1",
    )?;
    let rows = statement.query_map(params![i64::from(limit)], map_candidate)?;
    rows.collect()
}

/// Reads one candidate.
pub fn get_candidate(
    connection: &Connection,
    id: &str,
) -> Result<Option<LessonCandidate>, rusqlite::Error> {
    let mut statement = connection.prepare(
        "SELECT id, candidate_key, title, body, baseline_eval_id, candidate_eval_id,
                status, created_at, activated_at, rolled_back_at
         FROM lesson_candidates WHERE id = ?1",
    )?;
    let mut rows = statement.query_map(params![id], map_candidate)?;
    rows.next().transpose()
}

/// Marks one candidate active; only a pending candidate can activate.
pub fn activate_candidate(
    connection: &Connection,
    id: &str,
) -> Result<Option<LessonCandidate>, rusqlite::Error> {
    let changed = connection.execute(
        "UPDATE lesson_candidates SET status = 'active', activated_at = ?2
         WHERE id = ?1 AND status = 'pending'",
        params![id, Timestamp::now().as_str()],
    )?;
    if changed == 0 {
        return Ok(None);
    }
    get_candidate(connection, id)
}

/// Compensates a failed activation (DB active, ledger append failed).
pub fn reset_activation(connection: &Connection, id: &str) -> rusqlite::Result<()> {
    connection.execute(
        "UPDATE lesson_candidates SET status = 'pending', activated_at = NULL
         WHERE id = ?1 AND status = 'active'",
        params![id],
    )?;
    Ok(())
}

/// Marks one active candidate rolled back.
pub fn rollback_candidate(
    connection: &Connection,
    id: &str,
) -> Result<Option<LessonCandidate>, rusqlite::Error> {
    let changed = connection.execute(
        "UPDATE lesson_candidates SET status = 'rolled_back', rolled_back_at = ?2
         WHERE id = ?1 AND status = 'active'",
        params![id, Timestamp::now().as_str()],
    )?;
    if changed == 0 {
        return Ok(None);
    }
    get_candidate(connection, id)
}

/// Builds the `applied_lesson` JSONL line the prompt loader accepts.
#[must_use]
pub fn applied_lesson_line(candidate: &LessonCandidate, receipt_sha256: &str) -> String {
    serde_json::json!({
        "state": "applied_lesson",
        "proposal_id": candidate.id,
        "area": candidate.candidate_key,
        "problem": candidate.title,
        "proposal": candidate.body,
        "receipt_sha256": receipt_sha256,
        "activated_at": candidate.activated_at,
    })
    .to_string()
}

/// Builds the rollback audit line (not admitted by the prompt loader).
#[must_use]
pub fn rolled_back_line(candidate: &LessonCandidate) -> String {
    serde_json::json!({
        "state": "rolled_back_lesson",
        "proposal_id": candidate.id,
        "rolled_back_at": candidate
            .rolled_back_at
            .clone()
            .unwrap_or_else(|| Timestamp::now().as_str().to_owned()),
    })
    .to_string()
}
