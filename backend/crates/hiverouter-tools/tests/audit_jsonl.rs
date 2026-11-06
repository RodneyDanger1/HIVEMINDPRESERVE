//! End-to-end JSONL audit sink test (Phase 11).
//!
//! This is an integration test binary (its own process) because installing the
//! tracing subscriber is a once-per-process operation — exactly the single-owner
//! rule the sink documents. It exercises the real `tracing-subscriber` JSON layer
//! and the real `tracing-appender` rolling writer, then reads the file back and
//! runs the secret probe.

use hiverouter_contract::{Refusal, RunId};
use hiverouter_tools::{
    init_json_audit, probe_audit_dir, probe_audit_file, read_audit_rows, record_refusal,
    record_tool_call, PermissionClass, ToolId,
};

#[test]
fn audit_jsonl_roundtrip_and_secret_probe() {
    let temp = tempfile::tempdir().expect("tempdir");
    let guard = init_json_audit(temp.path()).expect("audit sink initialises once");

    let run = RunId::new();
    record_tool_call(
        run,
        ToolId::FileRead,
        PermissionClass::Read,
        "executed",
        None,
    );
    record_refusal(
        run,
        Refusal::Cycle,
        Some(ToolId::FileWrite),
        Some(PermissionClass::Mutate),
    );
    // Dropping the guard stops the non-blocking writer and flushes the lines.
    drop(guard);

    let files = hiverouter_tools::audit::list_audit_files(temp.path()).expect("list");
    assert_eq!(files.len(), 1, "one rolling daily file expected: {files:?}");

    let rows = read_audit_rows(temp.path()).expect("every line parses as JSON");
    assert!(rows.len() >= 2, "both audit events were flushed: {rows:?}");
    assert!(
        rows.iter()
            .any(|row| row.code.as_deref() == Some("E_CYCLE")),
        "the refusal row carries its stable code: {rows:?}"
    );
    assert!(
        rows.iter()
            .any(|row| row.run_id.as_deref() == Some(run.to_string().as_str())),
        "the row carries the run id: {rows:?}"
    );
    assert!(!temp.path().join("audit.jsonl").exists() || files.len() == 1);

    // The audit file must never contain key/token material.
    probe_audit_dir(temp.path()).expect("audit dir is secret-free");
    for file in &files {
        probe_audit_file(file).expect("each file is secret-free");
    }
}
