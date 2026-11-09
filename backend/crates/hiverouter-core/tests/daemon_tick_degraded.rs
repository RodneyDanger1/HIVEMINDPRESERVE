//! A daemon tick must never claim work ran when no runner exists. It reports
//! `degraded`, stores the reason, and the `state = 'running'` guard stops it
//! from re-reporting a terminal daemon.

use std::sync::Arc;

use hiverouter_core::daemon::{DaemonStore, NewDaemon};
use hiverouter_core::db::Database;

fn fixture() -> (DaemonStore, Arc<Database>) {
    let db = Arc::new(Database::open_in_memory().expect("db"));
    db.migrate().expect("migrate");
    (DaemonStore::new(db.clone()), db)
}

#[test]
fn a_tick_without_a_runner_reports_degraded_and_never_fakes_alive() {
    let (store, db) = fixture();
    let daemon = store
        .create(NewDaemon {
            display_name: "Watchdog".to_owned(),
            owner_kind: "operator".to_owned(),
            owner_id: "operator".to_owned(),
            kind: "watchdog".to_owned(),
            job_json: r#"{"kind":"poll"}"#.to_owned(),
            interval_secs: 60,
            created_by: "operator".to_owned(),
        })
        .expect("create");
    store.set_enabled(&daemon.daemon_id, true).expect("start");
    assert_eq!(store.tick_due(10).expect("tick"), 1);
    let fresh = store
        .get(&daemon.daemon_id)
        .expect("query")
        .expect("daemon");
    assert_eq!(fresh.state, "degraded");
    assert_eq!(
        fresh.last_error_code.as_deref(),
        Some("E_DAEMON_JOB_UNIMPLEMENTED")
    );
    assert!(
        fresh.next_tick_at.is_none(),
        "a degraded daemon must not keep a tick window open"
    );
    let reports = store.reports(&daemon.daemon_id, 10).expect("reports");
    assert_eq!(reports.len(), 1);
    assert_eq!(reports[0].outcome, "degraded");
    let detail: serde_json::Value =
        serde_json::from_str(&reports[0].detail_json).expect("detail json");
    assert_eq!(detail["executed"], serde_json::json!(false));
    assert_eq!(detail["job"], serde_json::json!(r#"{"kind":"poll"}"#));
    assert!(
        detail["reason"].as_str().is_some_and(|reason| !reason.is_empty()),
        "the report must name why nothing ran: {detail}"
    );
    assert_eq!(
        store.tick_due(10).expect("second tick"),
        0,
        "the AND state='running' guard must stop a degraded daemon from ticking again"
    );
    assert_eq!(
        store.reports(&daemon.daemon_id, 10).expect("reports").len(),
        1,
        "no duplicate report after degradation"
    );
    let tasks: i64 = db
        .with_connection(|connection| {
            connection.query_row("SELECT COUNT(*) FROM tasks", [], |row| row.get(0))
        })
        .expect("task count");
    assert_eq!(tasks, 0, "a daemon must never become a task");
}

#[test]
fn a_tick_with_an_empty_job_reports_the_specific_reason() {
    let (store, _db) = fixture();
    let daemon = store
        .create(NewDaemon {
            display_name: "Empty".to_owned(),
            owner_kind: "hive".to_owned(),
            owner_id: "root".to_owned(),
            kind: "relay".to_owned(),
            job_json: "{}".to_owned(),
            interval_secs: 30,
            created_by: "hyper:root".to_owned(),
        })
        .expect("create");
    store.set_enabled(&daemon.daemon_id, true).expect("start");
    assert_eq!(store.tick_due(10).expect("tick"), 1);
    let reports = store.reports(&daemon.daemon_id, 10).expect("reports");
    let detail: serde_json::Value =
        serde_json::from_str(&reports[0].detail_json).expect("detail json");
    assert_eq!(detail["executed"], serde_json::json!(false));
    assert_eq!(
        detail["reason"], "daemon has no executable job",
        "an empty job is reported distinctly from an unimplemented runner"
    );
}
