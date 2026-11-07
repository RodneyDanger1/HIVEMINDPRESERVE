from pathlib import Path

root = Path(r"C:\Users\Riley\Desktop\hiverouter-v1")
p = root / "backend/crates/hiverouter-tools/src/approval.rs"
t = p.read_text(encoding="utf-8")

needle = """        CREATE INDEX IF NOT EXISTS approvals_session
            ON approvals (run_id, tool_name, args_sha256, decision);\","""
rep = """        CREATE INDEX IF NOT EXISTS approvals_session
            ON approvals (run_id, tool_name, args_sha256, decision);
        CREATE UNIQUE INDEX IF NOT EXISTS approvals_one_pending_per_call
            ON approvals (run_id, tool_call_id) WHERE decision IS NULL;\","""
if needle not in t:
    raise SystemExit("migrate needle missing")
t = t.replace(needle, rep)

marker = "/// Consumes every still-pending approval at boot."
idx = t.find(marker)
if idx < 0:
    raise SystemExit("orphan marker missing")

func = '''/// Marks one pending approval expired (live-wait timeout path).
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

'''
t = t[:idx] + func + t[idx:]
p.write_text(t, encoding="utf-8", newline="\n")
print("approval primitives patched")
