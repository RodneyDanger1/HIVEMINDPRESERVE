//! Single deterministic supervisor lane for remote leases and daemons.
//!
//! It is not a scheduler and never invokes a model. Each tick expires stale
//! remote leases, advances due daemons, and records one durable supervisor
//! state row. The lane is single-flight; a crash leaves the last tick visible
//! and the next boot reconciles actions without replay.

use std::sync::Arc;
use std::time::Duration;

use hiverouter_contract::{Refusal, Timestamp};
use rusqlite::params;
use tokio_util::sync::CancellationToken;

use crate::daemon::DaemonStore;
use crate::db::Database;
use crate::remote::RemoteStore;

/// Durable supervisor state exposed to the UI.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SupervisorState {
    pub lane_started_at: Option<String>,
    pub last_tick_at: Option<String>,
    pub next_due_at: Option<String>,
    pub backoff_ms: i64,
    pub last_error_code: Option<String>,
}

/// Deterministic remote/daemon supervisor.
#[derive(Debug)]
pub struct RemoteSupervisor {
    db: Arc<Database>,
    remote: Arc<RemoteStore>,
    daemons: Arc<DaemonStore>,
    interval: Duration,
    shutdown: CancellationToken,
}

impl RemoteSupervisor {
    #[must_use]
    pub fn new(
        db: Arc<Database>,
        remote: Arc<RemoteStore>,
        daemons: Arc<DaemonStore>,
        interval: Duration,
        shutdown: CancellationToken,
    ) -> Self {
        Self {
            db,
            remote,
            daemons,
            interval,
            shutdown,
        }
    }

    /// Runs until shutdown. Failures are recorded and retried with bounded
    /// exponential backoff; they never create a task and never hot-loop.
    pub async fn run(self) {
        let started = Timestamp::now().as_str().to_owned();
        let _ = self.record_state(None, 0, None, Some(&started));
        let mut backoff_ms: i64 = 0;
        loop {
            if self.shutdown.is_cancelled() {
                break;
            }
            let delay = if backoff_ms == 0 {
                self.interval
            } else {
                Duration::from_millis(u64::try_from(backoff_ms).unwrap_or(30_000))
            };
            tokio::select! {
                _ = self.shutdown.cancelled() => break,
                _ = tokio::time::sleep(delay) => {}
            }
            let now = Timestamp::now().as_str().to_owned();
            match self.tick_once() {
                Ok(()) => {
                    backoff_ms = 0;
                    let _ = self.record_state(Some(&now), 0, None, None);
                }
                Err(refusal) => {
                    backoff_ms = next_backoff(backoff_ms);
                    let _ = self.record_state(
                        Some(&now),
                        backoff_ms,
                        Some(refusal.code()),
                        None,
                    );
                }
            }
        }
    }

    /// One deterministic tick used by tests and the run loop.
    pub fn tick_once(&self) -> Result<(), Refusal> {
        self.remote.expire_leases()?;
        self.daemons.tick_due(64)?;
        Ok(())
    }

    /// Reads the single durable supervisor state row.
    pub fn state(&self) -> Result<SupervisorState, Refusal> {
        read_state(&self.db)
    }

    fn record_state(
        &self,
        last_tick: Option<&str>,
        backoff_ms: i64,
        error_code: Option<&str>,
        lane_started: Option<&str>,
    ) -> Result<(), Refusal> {
        let now = Timestamp::now().as_str().to_owned();
        let next = Timestamp::from_unix_secs(
            Timestamp::now()
                .unix_secs()
                .map_err(|_| Refusal::InternalError)?
                .saturating_add(u64::try_from(
                    (backoff_ms.max(0) / 1000).max(2),
                )
                .unwrap_or(2)),
        )
        .as_str()
        .to_owned();
        self.db
            .with_connection(|connection| {
                connection.execute(
                    "UPDATE remote_supervisor_state SET
                        lane_started_at = COALESCE(?1, lane_started_at),
                        last_tick_at = COALESCE(?2, last_tick_at),
                        next_due_at = ?3, backoff_ms = ?4,
                        last_error_code = ?5, updated_at = ?6
                     WHERE id = 1",
                    params![lane_started, last_tick, next, backoff_ms, error_code, now],
                )?;
                Ok(())
            })
            .map_err(|error: crate::db::DbError| error.refusal())
    }
}

/// Reads the singleton supervisor row without starting the lane (HTTP read).
pub fn read_state(db: &Database) -> Result<SupervisorState, Refusal> {
    db.with_connection(|connection| {
        connection
            .query_row(
                "SELECT lane_started_at, last_tick_at, next_due_at, backoff_ms,
                        last_error_code
                 FROM remote_supervisor_state WHERE id = 1",
                [],
                |row| {
                    Ok(SupervisorState {
                        lane_started_at: row.get(0)?,
                        last_tick_at: row.get(1)?,
                        next_due_at: row.get(2)?,
                        backoff_ms: row.get(3)?,
                        last_error_code: row.get(4)?,
                    })
                },
            )
            .map_err(Into::into)
    })
    .map_err(|error: crate::db::DbError| error.refusal())
}

fn next_backoff(current_ms: i64) -> i64 {    let next = if current_ms == 0 {
        2_000
    } else {
        current_ms.saturating_mul(2)
    };
    next.min(30_000)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture() -> RemoteSupervisor {
        let db = Arc::new(Database::open_in_memory().expect("db"));
        db.migrate().expect("migrate");
        RemoteSupervisor::new(
            db.clone(),
            Arc::new(RemoteStore::new(db.clone())),
            Arc::new(DaemonStore::new(db)),
            Duration::from_secs(2),
            CancellationToken::new(),
        )
    }

    #[test]
    fn tick_expires_leases_and_is_idempotent() {
        let supervisor = fixture();
        supervisor.tick_once().expect("first tick");
        supervisor.tick_once().expect("second tick");
        let state = supervisor.state().expect("state");
        assert_eq!(state.backoff_ms, 0);
    }

    #[test]
    fn tick_demotes_an_expired_online_remote() {
        let supervisor = fixture();
        let remote = supervisor
            .remote
            .create_remote(crate::remote::NewRemote {
                display_name: "Lease test".to_owned(),
                owner_kind: "operator".to_owned(),
                owner_id: "operator".to_owned(),
                transport_kind: "in_process".to_owned(),
                adapter_kind: "fixture".to_owned(),
                endpoint_ref: "fixture".to_owned(),
                credential_ref: None,
                capabilities_json: "[]".to_owned(),
                created_by: "operator".to_owned(),
            })
            .expect("remote");
        supervisor
            .remote
            .heartbeat(&remote.remote_id)
            .expect("heartbeat");
        supervisor
            .db
            .with_connection(|connection| {
                connection.execute(
                    "UPDATE remote_agents SET lease_expires_at = '1970-01-01T00:00:00Z'
                     WHERE remote_id = ?1",
                    params![remote.remote_id],
                )
            })
            .expect("expire lease");
        supervisor.tick_once().expect("tick");
        let stored = supervisor
            .remote
            .get_remote(&remote.remote_id)
            .expect("remote")
            .expect("row");
        assert_eq!(stored.state, "offline");
    }

    #[test]
    fn backoff_is_bounded() {
        assert_eq!(next_backoff(0), 2_000);
        assert_eq!(next_backoff(2_000), 4_000);
        assert_eq!(next_backoff(30_000), 30_000);
    }
}
