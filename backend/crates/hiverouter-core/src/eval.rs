//! Executed evaluation receipts (CAP-9 foundation).
//!
//! An eval run executes one allowlisted command through the existing
//! [`crate::jobs`] registry, compares its real terminal state/exit code to an
//! expected exit code, and stores an immutable receipt. This is evidence, not
//! learning: a lesson may cite a receipt, but a receipt never activates code.
//!
//! Deliberately omitted from the receipt: command output tails (they can carry
//! secrets), model output, and any synthetic "score". Only exit state, exit
//! code, output byte count, and output SHA-256 are stored.

use hiverouter_contract::Timestamp;
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};

/// Append-only eval receipt schema (v29).
pub const SCHEMA_V29: &str = r#"
CREATE TABLE IF NOT EXISTS eval_runs (
    id             TEXT PRIMARY KEY NOT NULL,
    suite_id       TEXT NOT NULL,
    case_id        TEXT NOT NULL,
    program        TEXT NOT NULL,
    args_json      TEXT NOT NULL,
    expected_exit  INTEGER NOT NULL,
    actual_exit    INTEGER,
    state          TEXT NOT NULL,
    passed         INTEGER NOT NULL CHECK (passed IN (0,1)),
    job_id         TEXT,
    output_sha256  TEXT NOT NULL,
    output_bytes   INTEGER NOT NULL,
    created_at     TEXT NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_eval_runs_suite ON eval_runs (suite_id, created_at);
"#;

/// Down migration for v29.
pub const SCHEMA_V29_DOWN: &str = "DROP INDEX IF EXISTS idx_eval_runs_suite; DROP TABLE IF EXISTS eval_runs;";

/// Migration descriptor installed by [`crate::db::MIGRATIONS`].
#[must_use]
pub const fn eval_migration(version: i64) -> crate::db::Migration {
    crate::db::Migration {
        version,
        name: "executed_eval",
        up: SCHEMA_V29,
        down: SCHEMA_V29_DOWN,
    }
}

/// One immutable executed-eval receipt.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct EvalReceipt {
    /// Receipt id (uuid-v7 string).
    pub id: String,
    /// Operator-defined suite id.
    pub suite_id: String,
    /// Operator-defined case id.
    pub case_id: String,
    /// Bare executable name.
    pub program: String,
    /// Direct arguments used for the run.
    pub args: Vec<String>,
    /// Expected process exit code.
    pub expected_exit: i32,
    /// Actual exit code when the child exited; `None` when unknown/terminated.
    pub actual_exit: Option<i32>,
    /// Terminal job state label (`succeeded`, `failed`, `timed_out`, `killed`).
    pub state: String,
    /// True only when the observed state/exit match the expectation.
    pub passed: bool,
    /// Background job id that produced the evidence.
    pub job_id: Option<String>,
    /// SHA-256 of the bounded output tail.
    pub output_sha256: String,
    /// Observed stdout+stderr byte count.
    pub output_bytes: u64,
    /// Receipt creation time.
    pub created_at: String,
}

/// Persists one receipt. Idempotent by id; existing receipts are never updated.
///
/// # Errors
/// SQLite errors; a duplicate id returns a constraint error rather than
/// silently overwriting the evidence.
pub fn record_eval(connection: &Connection, receipt: &EvalReceipt) -> rusqlite::Result<()> {
    let args_json = serde_json::to_string(&receipt.args)
        .map_err(|error| rusqlite::Error::ToSqlConversionFailure(Box::new(error)))?;
    connection.execute(
        "INSERT INTO eval_runs
         (id, suite_id, case_id, program, args_json, expected_exit, actual_exit,
          state, passed, job_id, output_sha256, output_bytes, created_at)
         VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13)",
        params![
            receipt.id,
            receipt.suite_id,
            receipt.case_id,
            receipt.program,
            args_json,
            i64::from(receipt.expected_exit),
            receipt.actual_exit.map(i64::from),
            receipt.state,
            if receipt.passed { 1 } else { 0 },
            receipt.job_id,
            receipt.output_sha256,
            i64::try_from(receipt.output_bytes).unwrap_or(i64::MAX),
            receipt.created_at,
        ],
    )?;
    Ok(())
}

/// Lists receipts, newest first, bounded.
pub fn list_eval(connection: &Connection, limit: u32) -> Result<Vec<EvalReceipt>, rusqlite::Error> {
    let mut statement = connection.prepare(
        "SELECT id, suite_id, case_id, program, args_json, expected_exit, actual_exit,
                state, passed, job_id, output_sha256, output_bytes, created_at
         FROM eval_runs ORDER BY created_at DESC, id DESC LIMIT ?1",
    )?;
    let rows = statement.query_map(params![i64::from(limit)], |row| {
        let args_json: String = row.get(4)?;
        let args = serde_json::from_str(&args_json).map_err(|error| {
            rusqlite::Error::FromSqlConversionFailure(4, rusqlite::types::Type::Text, Box::new(error))
        })?;
        Ok(EvalReceipt {
            id: row.get(0)?,
            suite_id: row.get(1)?,
            case_id: row.get(2)?,
            program: row.get(3)?,
            args,
            expected_exit: i32::try_from(row.get::<_, i64>(5)?).unwrap_or(0),
            actual_exit: row.get::<_, Option<i64>>(6)?.and_then(|value| i32::try_from(value).ok()),
            state: row.get(7)?,
            passed: row.get::<_, i64>(8)? != 0,
            job_id: row.get(9)?,
            output_sha256: row.get(10)?,
            output_bytes: u64::try_from(row.get::<_, i64>(11)?).unwrap_or(0),
            created_at: row.get(12)?,
        })
    })?;
    rows.collect()
}

/// Eval suite/case schema (v31).
pub const SCHEMA_V31: &str = r#"
CREATE TABLE IF NOT EXISTS eval_suites (
    id          TEXT PRIMARY KEY NOT NULL,
    name        TEXT NOT NULL,
    description TEXT NOT NULL,
    created_at  TEXT NOT NULL
);
CREATE TABLE IF NOT EXISTS eval_cases (
    id            TEXT PRIMARY KEY NOT NULL,
    suite_id      TEXT NOT NULL REFERENCES eval_suites(id) ON DELETE CASCADE,
    case_id       TEXT NOT NULL,
    program       TEXT NOT NULL,
    args_json     TEXT NOT NULL,
    expected_exit INTEGER NOT NULL,
    created_at    TEXT NOT NULL,
    UNIQUE (suite_id, case_id)
);
CREATE INDEX IF NOT EXISTS idx_eval_cases_suite ON eval_cases (suite_id, case_id);
"#;

/// Down migration for v31.
pub const SCHEMA_V31_DOWN: &str =
    "DROP INDEX IF EXISTS idx_eval_cases_suite; DROP TABLE IF EXISTS eval_cases; DROP TABLE IF EXISTS eval_suites;";

/// Migration descriptor for v31.
#[must_use]
pub const fn eval_suite_migration(version: i64) -> crate::db::Migration {
    crate::db::Migration {
        version,
        name: "eval_suites",
        up: SCHEMA_V31,
        down: SCHEMA_V31_DOWN,
    }
}

/// One eval suite.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct EvalSuite {
    /// Suite id.
    pub id: String,
    /// Display name.
    pub name: String,
    /// Description.
    pub description: String,
    /// Cases in declaration order.
    pub cases: Vec<EvalCase>,
    /// Creation time.
    pub created_at: String,
}

/// One eval case definition.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct EvalCase {
    /// Stable case id within the suite.
    pub case_id: String,
    /// Bare executable name.
    pub program: String,
    /// Direct arguments.
    pub args: Vec<String>,
    /// Expected exit code.
    pub expected_exit: i32,
}

/// Aggregate metrics for one suite.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct EvalMetrics {
    /// Suite id.
    pub suite_id: String,
    /// Number of receipts.
    pub total: u64,
    /// Passing receipts.
    pub passed: u64,
    /// Failing receipts.
    pub failed: u64,
}

/// Inserts a suite and its cases in one transaction.
pub fn create_suite(connection: &Connection, suite: &EvalSuite) -> rusqlite::Result<()> {
    let transaction = connection.unchecked_transaction()?;
    transaction.execute(
        "INSERT INTO eval_suites(id, name, description, created_at) VALUES (?1,?2,?3,?4)",
        params![suite.id, suite.name, suite.description, suite.created_at],
    )?;
    for case in &suite.cases {
        let args_json = serde_json::to_string(&case.args)
            .map_err(|error| rusqlite::Error::ToSqlConversionFailure(Box::new(error)))?;
        transaction.execute(
            "INSERT INTO eval_cases(id, suite_id, case_id, program, args_json, expected_exit, created_at)
             VALUES (?1,?2,?3,?4,?5,?6,?7)",
            params![
                uuid::Uuid::now_v7().to_string(),
                suite.id,
                case.case_id,
                case.program,
                args_json,
                i64::from(case.expected_exit),
                suite.created_at,
            ],
        )?;
    }
    transaction.commit()?;
    Ok(())
}

/// Lists suites with their cases, newest first.
pub fn list_suites(connection: &Connection, limit: u32) -> Result<Vec<EvalSuite>, rusqlite::Error> {
    let mut statement = connection.prepare(
        "SELECT id, name, description, created_at FROM eval_suites
         ORDER BY created_at DESC, id DESC LIMIT ?1",
    )?;
    let rows = statement.query_map(params![i64::from(limit)], |row| {
        Ok((
            row.get::<_, String>(0)?,
            row.get::<_, String>(1)?,
            row.get::<_, String>(2)?,
            row.get::<_, String>(3)?,
        ))
    })?;
    let mut suites = Vec::new();
    for row in rows {
        let (id, name, description, created_at) = row?;
        let mut case_statement = connection.prepare(
            "SELECT case_id, program, args_json, expected_exit FROM eval_cases
             WHERE suite_id = ?1 ORDER BY case_id ASC",
        )?;
        let cases = case_statement
            .query_map(params![id], |row| {
                let args_json: String = row.get(2)?;
                let args = serde_json::from_str(&args_json).map_err(|error| {
                    rusqlite::Error::FromSqlConversionFailure(
                        2,
                        rusqlite::types::Type::Text,
                        Box::new(error),
                    )
                })?;
                Ok(EvalCase {
                    case_id: row.get(0)?,
                    program: row.get(1)?,
                    args,
                    expected_exit: i32::try_from(row.get::<_, i64>(3)?).unwrap_or(0),
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;
        suites.push(EvalSuite {
            id,
            name,
            description,
            cases,
            created_at,
        });
    }
    Ok(suites)
}

/// Reads one suite with cases.
pub fn get_suite(connection: &Connection, suite_id: &str) -> Result<Option<EvalSuite>, rusqlite::Error> {
    let suites = list_suites(connection, 1_000)?;
    Ok(suites.into_iter().find(|suite| suite.id == suite_id))
}

/// Aggregate pass/fail metrics for one suite.
pub fn suite_metrics(
    connection: &Connection,
    suite_id: &str,
) -> Result<EvalMetrics, rusqlite::Error> {
    let (total, passed): (i64, i64) = connection.query_row(
        "SELECT COUNT(*), COALESCE(SUM(passed), 0) FROM eval_runs WHERE suite_id = ?1",
        params![suite_id],
        |row| Ok((row.get(0)?, row.get(1)?)),
    )?;
    let total = u64::try_from(total).unwrap_or(0);
    let passed = u64::try_from(passed).unwrap_or(0);
    Ok(EvalMetrics {
        suite_id: suite_id.to_owned(),
        total,
        passed,
        failed: total.saturating_sub(passed),
    })
}

/// Creates a receipt from an observed job terminal state.
#[must_use]
pub fn receipt_from_observation(
    suite_id: &str,
    case_id: &str,
    program: &str,
    args: &[String],
    expected_exit: i32,
    job_id: &str,
    state: &str,
    actual_exit: Option<i32>,
    output_sha256: &str,
    output_bytes: u64,
) -> EvalReceipt {
    EvalReceipt {
        id: uuid::Uuid::now_v7().to_string(),
        suite_id: suite_id.to_owned(),
        case_id: case_id.to_owned(),
        program: program.to_owned(),
        args: args.to_vec(),
        expected_exit,
        actual_exit,
        state: state.to_owned(),
        passed: state == "succeeded" && actual_exit == Some(expected_exit),
        job_id: Some(job_id.to_owned()),
        output_sha256: output_sha256.to_owned(),
        output_bytes,
        created_at: Timestamp::now().as_str().to_owned(),
    }
}
