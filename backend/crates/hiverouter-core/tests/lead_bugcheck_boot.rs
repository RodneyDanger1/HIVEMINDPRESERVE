//! LEAD-BUGCHECK reproduction (read-only investigation target): boot restart must
//! leave no resolvable pending approvals.
//!
//! Design requirements pinned by this test:
//! - `plans/waves/W11-SUSPENSION-DESIGN.md` §4/§9: "crash/restart ... pending
//!   approvals expired"; "boot: runs non-terminal -> interrupted ...; pending
//!   approvals -> expired".
//! - `plans/foresight/APPROVAL-SUSPENSION-PLAN.md` E4: restart: pending approvals
//!   expired; late resolve gets `E_APPROVAL_TIMEOUT`/`E_APPROVAL_NOT_FOUND`.
//! - `plans/foresight/SIDE-EFFECT-EXECUTOR-PLAN.md` test 12: run persisted
//!   `awaiting_approval` + restart => interrupted with `E_APPROVAL_TIMEOUT`;
//!   pending row expired; nothing executed.
//!
//! The fixture deliberately uses the real production boot order
//! (`main.rs`: recovery, then `AppState::with_graph` -> `build_tools` ->
//! `ToolsBridge::expire_pending_approvals`) with a FUTURE `expires_at`, which is
//! exactly the window a crash/restart hits in practice (TTL 120 s).

use std::sync::Arc;

use hiverouter_contract::{command, AgentState, Refusal};
use hiverouter_core::db::Database;
use hiverouter_core::graph::GraphAuthority;
use hiverouter_core::recovery::recover_on_boot;
use hiverouter_core::snapshot::SnapshotStore;
use hiverouter_core::store::RunStore;
use hiverouter_core::tools_bridge::ToolsBridge;

#[test]
fn boot_restart_expires_orphaned_pending_approval_and_late_resolve_times_out() {
    let temp = tempfile::tempdir().expect("temp dir");
    let db = Arc::new(Database::open(&temp.path().join("hiverouter.db")).expect("open db"));
    db.migrate().expect("core migrations");
    ToolsBridge::migrate(&db).expect("approval table migration");

    let store = RunStore::new(db.clone());
    let run = store
        .create_run(
            "lead-bugcheck",
            "restart while awaiting approval",
            AgentState::Running,
        )
        .expect("create run");
    store
        .set_run_status(run.id, AgentState::AwaitingApproval, None)
        .expect("run is awaiting approval");

    let bridge = ToolsBridge::new(temp.path(), db.clone());
    let request = bridge
        .request_model_tool_approval(
            run.id,
            hiverouter_contract::TaskId::new(),
            "call-boot-restart-1",
            "file_write",
            &serde_json::json!({"path": "notes.txt", "content": "hello"}),
        )
        .expect("commit approval request");

    let now = hiverouter_contract::Timestamp::now()
        .unix_secs()
        .expect("representable now");
    let pending = bridge.pending_approvals(run.id).expect("read pending");
    assert_eq!(
        pending.len(),
        1,
        "fixture must have exactly one pending approval"
    );
    assert!(
        !pending[0].is_expired(now),
        "fixture must be future-TTL (the crash window); otherwise this test only re-tests TTL expiry"
    );

    // Production boot order: recovery first, then the tools build reaper.
    let graph = GraphAuthority::new();
    let snapshots = SnapshotStore::new(temp.path(), db.clone());
    let report = recover_on_boot(db.clone(), &graph, &snapshots).expect("boot recovery");
    assert!(
        report.interrupted_runs.contains(&run.id),
        "the awaiting-approval run must be interrupted at boot: {report:?}"
    );
    let recovered = store.get_run(run.id).expect("read recovered run");
    assert_eq!(recovered.status, AgentState::Interrupted);
    assert_eq!(
        recovered.error_code.as_deref(),
        Some(Refusal::ApprovalTimeout.code()),
        "interrupted awaiting-approval runs must carry E_APPROVAL_TIMEOUT"
    );

    // This is the second half of main.rs boot: AppState build -> build_tools ->
    // ToolsBridge::expire_pending_approvals. No in-process waiter survives restart,
    // so every still-pending row is restart-orphaned and must be consumed.
    let now_after_boot = hiverouter_contract::Timestamp::now()
        .unix_secs()
        .expect("representable now");
    let expired = ToolsBridge::expire_pending_approvals(&db, now_after_boot)
        .expect("boot approval reaper must succeed");
    assert_eq!(
        expired, 1,
        "the restart-orphaned pending row must be consumed"
    );
    assert_eq!(
        bridge
            .pending_approvals(run.id)
            .expect("read pending after boot"),
        Vec::new(),
        "boot restart must not leave a resolvable stale approval (no waiter survives)"
    );

    // A late operator decision must not be accepted as a fresh resolution; the
    // design allows E_APPROVAL_TIMEOUT (restart expiry) or E_APPROVAL_NOT_FOUND.
    let late = bridge.resolve_approval(&command::ApprovalResolve {
        approval_id: pending[0].approval_id,
        nonce: pending[0].nonce.clone(),
        decision: "approve".to_owned(),
        reason: None,
    });
    let error = late.expect_err("late decision after a restart must be refused");
    assert!(
        matches!(error, Refusal::ApprovalTimeout | Refusal::ApprovalNotFound),
        "late decision after boot expiry must be E_APPROVAL_TIMEOUT/E_APPROVAL_NOT_FOUND, got {error:?}"
    );
    assert!(
        request.nonce.is_some(),
        "the committed request must have carried a nonce"
    );
}
