//! Memory bank isolation and validated-write gates.
//!
//! Pins the HIGH audit findings on the store side:
//! - a task with no persisted identity must fail closed to its **own** branch,
//!   never to the run bank every task can read;
//! - a bank read is run-scoped, so a foreign run's bank cannot be read through
//!   the scoped API;
//! - the model-facing validated write routes into the caller's own bank and
//!   only accepts `kind`/`trust` values the read filter actually accepts, so no
//!   row can be stored silently unreadable.

use std::sync::Arc;

use hiverouter_contract::{AgentState, RunId, TaskId};
use hiverouter_core::db::Database;
use hiverouter_core::entities::TaskRecord;
use hiverouter_core::store::RunStore;

fn store() -> RunStore {
    let db = Arc::new(Database::open_in_memory().expect("in-memory database"));
    db.migrate().expect("migrations");
    RunStore::new(db)
}

fn task(run_id: RunId, parent: Option<TaskId>, role: &str, seq: u64, state: AgentState) -> TaskRecord {
    TaskRecord {
        id: TaskId::new(),
        run_id,
        parent_task_id: parent,
        role: role.to_owned(),
        depth: u8::try_from(seq).unwrap_or(1),
        spawn_seq: seq,
        state,
        rev: 0,
    }
}

#[test]
fn missing_identity_write_fails_closed_to_own_branch() {
    let store = store();
    let run = store
        .create_run("p", "memory identity", AgentState::Running)
        .expect("run");
    let reader = task(run.id, None, "worker", 0, AgentState::Running);
    store.insert_task(&reader).expect("reader task");

    // No task row for this id: identity resolution returns None.
    let unknown = TaskId::new();
    store
        .record_agent_memory_scoped("worker", "leak", "must-not-reach-run", run.id, unknown, "run")
        .expect("write");

    let run_bank = format!("bank:run:{}", run.id);
    assert!(
        store
            .memory_entries_for_bank(&run_bank, 10)
            .expect("run bank")
            .is_empty(),
        "an unidentified writer must never land in the run bank"
    );
    let own_branch = format!("bank:branch:{unknown}");
    let branch = store.memory_entries_for_bank(&own_branch, 10).expect("branch");
    assert_eq!(branch.len(), 1, "the write must land in the writer's own branch");
    assert_eq!(branch[0].value, "must-not-reach-run");

    // The unrelated task's read-set sees the run bank but not the secret.
    let read_set = store
        .memory_read_set(run.id, reader.id, None, 16)
        .expect("read set");
    assert!(
        read_set.iter().all(|entry| entry.value != "must-not-reach-run"),
        "a missing-identity write leaked into another task's read-set: {read_set:?}"
    );
}

#[test]
fn scoped_bank_read_refuses_a_foreign_run_bank() {
    let store = store();
    let run_a = store
        .create_run("p", "run-a", AgentState::Running)
        .expect("run a");
    let run_b = store
        .create_run("p", "run-b", AgentState::Running)
        .expect("run b");
    let task_a = task(run_a.id, None, "worker", 0, AgentState::Running);
    store.insert_task(&task_a).expect("task a");
    let bank = format!("bank:branch:{}", task_a.id);
    store
        .record_agent_memory_scoped("worker", "k", "run-a-secret", run_a.id, task_a.id, "run")
        .expect("write");

    assert_eq!(
        store
            .memory_entries_for_bank_scoped(run_a.id, &bank, 10)
            .expect("in-run read")
            .len(),
        1
    );
    assert!(
        store
            .memory_entries_for_bank_scoped(run_b.id, &bank, 10)
            .expect("cross-run read")
            .is_empty(),
        "run B must not read run A's bank through the scoped API"
    );
    // The unscoped operator surface is intentionally still row-addressable;
    // HTTP owns the caller scoping and must use the scoped API.
    assert_eq!(
        store
            .memory_entries_for_bank(&bank, 10)
            .expect("operator read")
            .len(),
        1,
        "the unscoped operator read documents the remaining HTTP-side scoping gap"
    );
}

#[test]
fn validated_write_routes_to_the_callers_own_bank_and_stays_readable() {
    let store = store();
    let run = store
        .create_run("p", "validated write", AgentState::Running)
        .expect("run");
    let super_agent = task(run.id, None, "superagent", 0, AgentState::Running);
    let agent = task(run.id, Some(super_agent.id), "worker", 1, AgentState::Running);
    let sibling = task(run.id, Some(super_agent.id), "worker", 2, AgentState::Running);
    for task in [&super_agent, &agent, &sibling] {
        store.insert_task(task).expect("task");
    }

    let entry_id = store
        .record_memory_entry(
            run.id,
            super_agent.id,
            "super",
            "hive-key",
            "hive-value",
            "fact",
            "unverified",
        )
        .expect("validated super write");
    assert!(entry_id.starts_with("entry:"));

    let banks = store.memory_banks_for_run(run.id).expect("banks");
    let hive = banks
        .iter()
        .find(|bank| bank.kind == "hive")
        .expect("hive bank");
    assert_eq!(hive.owner_task_id.as_deref(), Some(super_agent.id.to_string().as_str()));
    assert_eq!(
        store
            .memory_entries_for_bank_scoped(run.id, &hive.id, 10)
            .expect("hive entries")
            .len(),
        1,
        "the write must land in the caller's hive bank"
    );

    // The agent under the hive sees the hive fact through its bounded read-set.
    let read_set = store
        .memory_read_set(run.id, agent.id, Some(super_agent.id), 16)
        .expect("read set");
    assert!(
        read_set.iter().any(|entry| entry.value == "hive-value"),
        "the validated write must be readable (not silently filtered): {read_set:?}"
    );

    // An agent write stays in its own branch; a sibling never sees it.
    store
        .record_memory_entry(
            run.id,
            agent.id,
            "worker",
            "branch-key",
            "branch-value",
            "note",
            "unverified",
        )
        .expect("validated agent write");
    let sibling_set = store
        .memory_read_set(run.id, sibling.id, Some(super_agent.id), 16)
        .expect("sibling read set");
    assert!(
        sibling_set.iter().all(|entry| entry.key != "branch-key"),
        "a sibling branch must not read another branch's write: {sibling_set:?}"
    );
}

#[test]
fn validated_write_rejects_values_the_read_filter_would_hide() {
    let store = store();
    let run = store
        .create_run("p", "validated refusals", AgentState::Running)
        .expect("run");
    let agent = task(run.id, None, "worker", 0, AgentState::Running);
    store.insert_task(&agent).expect("task");

    // `quarantined` rows are hidden by the read filter: refuse at the boundary.
    assert_eq!(
        store.record_memory_entry(
            run.id,
            agent.id,
            "worker",
            "k",
            "v",
            "fact",
            "quarantined"
        ),
        Err(hiverouter_contract::Refusal::SchemaInvalid)
    );
    assert_eq!(
        store.record_memory_entry(
            run.id,
            agent.id,
            "worker",
            "k",
            "v",
            "rumour",
            "unverified"
        ),
        Err(hiverouter_contract::Refusal::SchemaInvalid)
    );
    assert_eq!(
        store.record_memory_entry(run.id, agent.id, "worker", "   ", "v", "fact", "unverified"),
        Err(hiverouter_contract::Refusal::SchemaInvalid)
    );
    // Unknown caller identity is refused, never defaulted to a wider bank.
    assert_eq!(
        store.record_memory_entry(
            run.id,
            TaskId::new(),
            "worker",
            "k",
            "v",
            "fact",
            "unverified"
        ),
        Err(hiverouter_contract::Refusal::TaskNotFound)
    );
    assert!(
        store
            .memory_entries_for_bank_scoped(run.id, &format!("bank:branch:{}", agent.id), 10)
            .expect("entries")
            .is_empty(),
        "a refused write must not leave a row"
    );
}
