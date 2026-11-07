//! SQLite store for runs, tasks, chat messages and command receipts
//! (blueprint Phase 4, WP 4.6; schema created by migration v2).

use std::sync::{Arc, OnceLock};

use hiverouter_contract::{AgentState, Refusal, RunId, TaskId, Timestamp};
use rusqlite::{params, OptionalExtension};
use uuid::Uuid;

use crate::db::{Database, DbError};
use crate::entities::{ChatMessage, ChatRole, Run, TaskRecord};
use crate::run_state::{is_terminal_task_state, TASK_TERMINAL_STATES};

fn map(error: DbError) -> Refusal {
    error.refusal()
}

/// One durable agent-to-agent message row.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AgentMessageRecord {
    /// Message id.
    pub id: String,
    /// Sender task id.
    pub from_task_id: String,
    /// Recipient task id.
    pub to_task_id: String,
    /// Message kind (`ask_parent` / `send_to_sibling`).
    pub kind: String,
    /// Bounded text.
    pub text: String,
    /// RFC3339 creation time.
    pub created_at: String,
}

/// SQL list of terminal task states, derived from the single state-machine set.
///
/// The guard must never drift from [`TASK_TERMINAL_STATES`]: a state added to the
/// lifecycle machine is therefore also un-resurrectable in SQLite. Values are
/// this crate's own `AgentState::as_str()` constants (never caller input).
fn terminal_task_state_sql() -> &'static str {
    static LIST: OnceLock<String> = OnceLock::new();
    LIST.get_or_init(|| {
        TASK_TERMINAL_STATES
            .iter()
            .map(|state| format!("'{}'", state.as_str()))
            .collect::<Vec<_>>()
            .join(", ")
    })
}

/// Outcome of one guarded task-state write (inside the connection lock).
enum TaskWrite {
    /// The row was non-terminal and is now `state` at this revision.
    Applied(u64),
    /// No task row has this id.
    Missing,
    /// The row is terminal; `current` is the state that was already persisted.
    Terminal {
        /// Terminal state already stored.
        current: AgentState,
        /// Revision already stored (unchanged by the refused write).
        rev: u64,
    },
}


/// Typed access to the run/task/message tables.
#[derive(Debug, Clone)]
pub struct RunStore {
    db: Arc<Database>,
}

impl RunStore {
    /// Creates a store over the shared database.
    #[must_use]
    pub fn new(db: Arc<Database>) -> Self {
        Self { db }
    }

    /// Inserts a new run.
    ///
    /// # Errors
    /// `E_INTERNAL_ERROR` / `E_DB_BUSY` on database errors.
    pub fn create_run(
        &self,
        project_id: &str,
        prompt: &str,
        status: AgentState,
    ) -> Result<Run, Refusal> {
        let now = Timestamp::now();
        let run = Run {
            id: RunId::new(),
            project_id: project_id.to_owned(),
            prompt: prompt.to_owned(),
            status,
            error_code: None,
            created_at: now.clone(),
            updated_at: now,
        };
        self.db
            .with_connection(|connection| {
                connection.execute(
                    "INSERT INTO runs(id, project_id, prompt, status, error_code, created_at, updated_at)
                     VALUES (?1, ?2, ?3, ?4, NULL, ?5, ?6)",
                    params![
                        run.id.to_string(),
                        run.project_id,
                        run.prompt,
                        run.status.as_str(),
                        run.created_at.as_str(),
                        run.updated_at.as_str()
                    ],
                )?;
                Ok(())
            })
            .map_err(map)?;
        Ok(run)
    }

    /// Reads a run by id.
    ///
    /// # Errors
    /// `E_RUN_NOT_FOUND` when the row does not exist.
    pub fn get_run(&self, run_id: RunId) -> Result<Run, Refusal> {
        let maybe = self
            .db
            .with_connection(|connection| {
                connection
                    .query_row(
                        "SELECT id, project_id, prompt, status, error_code, created_at, updated_at
                         FROM runs WHERE id = ?1",
                        params![run_id.to_string()],
                        row_to_run,
                    )
                    .optional()
            })
            .map_err(map)?;
        maybe.ok_or(Refusal::RunNotFound)
    }

    /// Whether a non-terminal run exists.
    ///
    /// # Errors
    /// Database refusal.
    pub fn latest_active_run(&self) -> Result<Option<Run>, Refusal> {
        self.db
            .with_connection(|connection| {
                connection
                    .query_row(
                        "SELECT id, project_id, prompt, status, error_code, created_at, updated_at
                         FROM runs WHERE status IN ('queued','planning','thinking','running','streaming','awaiting_approval')
                         ORDER BY created_at DESC LIMIT 1",
                        [],
                        row_to_run,
                    )
                    .optional()
            })
            .map_err(map)
    }

    /// Lists the newest runs.
    ///
    /// # Errors
    /// Database refusal.
    pub fn list_runs(&self, limit: u32) -> Result<Vec<Run>, Refusal> {
        self.db
            .with_connection(|connection| {
                let mut statement = connection.prepare(
                    "SELECT id, project_id, prompt, status, error_code, created_at, updated_at
                     FROM runs ORDER BY created_at DESC LIMIT ?1",
                )?;
                let rows = statement.query_map(params![i64::from(limit)], row_to_run)?;
                let mut runs = Vec::new();
                for row in rows {
                    runs.push(row?);
                }
                Ok(runs)
            })
            .map_err(map)
    }

    /// Updates a run's status and error code.
    ///
    /// # Errors
    /// `E_RUN_NOT_FOUND` when the run does not exist.
    pub fn set_run_status(
        &self,
        run_id: RunId,
        status: AgentState,
        error_code: Option<&str>,
    ) -> Result<(), Refusal> {
        let updated = self
            .db
            .with_connection(|connection| {
                connection.execute(
                    "UPDATE runs SET status = ?2, error_code = ?3, updated_at = ?4 WHERE id = ?1",
                    params![
                        run_id.to_string(),
                        status.as_str(),
                        error_code,
                        Timestamp::now().as_str()
                    ],
                )
            })
            .map_err(map)?;
        if updated == 0 {
            return Err(Refusal::RunNotFound);
        }
        Ok(())
    }

    /// Inserts a task row.
    ///
    /// # Errors
    /// Database refusal.
    pub fn insert_task(&self, task: &TaskRecord) -> Result<(), Refusal> {
        self.db
            .with_connection(|connection| {
                connection.execute(
                    "INSERT INTO tasks(id, run_id, parent_task_id, role, depth, spawn_seq, state, rev)
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
                    params![
                        task.id.to_string(),
                        task.run_id.to_string(),
                        task.parent_task_id.map(|parent| parent.to_string()),
                        task.role,
                        i64::from(task.depth),
                        i64::try_from(task.spawn_seq).unwrap_or(i64::MAX),
                        task.state.as_str(),
                        i64::try_from(task.rev).unwrap_or(i64::MAX),
                    ],
                )?;
                Ok(())
            })
            .map_err(map)
    }

    /// Reads a task by id.
    ///
    /// # Errors
    /// `E_TASK_NOT_FOUND` when the row does not exist.
    pub fn get_task(&self, task_id: TaskId) -> Result<TaskRecord, Refusal> {
        let maybe = self
            .db
            .with_connection(|connection| {
                connection
                    .query_row(
                        "SELECT id, run_id, parent_task_id, role, depth, spawn_seq, state, rev
                         FROM tasks WHERE id = ?1",
                        params![task_id.to_string()],
                        row_to_task,
                    )
                    .optional()
            })
            .map_err(map)?;
        maybe.ok_or(Refusal::TaskNotFound)
    }

    /// Updates a task's state and bumps its revision.
    ///
    /// Terminal states are sticky: a task that is already `completed`, `failed`,
    /// `cancelled`, `killed` or `interrupted` can never be moved to a *different*
    /// state (a late completion may not resurrect a killed task). Replaying the
    /// same terminal state is idempotent and returns the stored revision without
    /// bumping it.
    ///
    /// # Errors
    /// `E_TASK_NOT_FOUND` when the row does not exist; `E_TASK_TERMINAL` when the
    /// row is terminal and a different state was requested. The read/compare and
    /// the write happen inside one connection lock, so the decision cannot race a
    /// concurrent writer.
    pub fn update_task_state(&self, task_id: TaskId, state: AgentState) -> Result<u64, Refusal> {
        let id = task_id.to_string();
        let sql = format!(
            "UPDATE tasks SET state = ?2, rev = rev + 1 WHERE id = ?1 AND state NOT IN ({})",
            terminal_task_state_sql()
        );
        let outcome = self
            .db
            .with_connection(|connection| {
                let updated = connection.execute(&sql, params![id, state.as_str()])?;
                if updated > 0 {
                    let rev: i64 = connection.query_row(
                        "SELECT rev FROM tasks WHERE id = ?1",
                        params![id],
                        |row| row.get(0),
                    )?;
                    return Ok(TaskWrite::Applied(u64::try_from(rev).unwrap_or(0)));
                }
                let existing = connection
                    .query_row(
                        "SELECT state, rev FROM tasks WHERE id = ?1",
                        params![id],
                        |row| {
                            let current: String = row.get(0)?;
                            let rev: i64 = row.get(1)?;
                            Ok((current, rev))
                        },
                    )
                    .optional()?;
                Ok(match existing {
                    None => TaskWrite::Missing,
                    Some((current, rev)) => TaskWrite::Terminal {
                        current: parse_state(&current),
                        rev: u64::try_from(rev).unwrap_or(0),
                    },
                })
            })
            .map_err(map)?;
        match outcome {
            TaskWrite::Applied(rev) => Ok(rev),
            TaskWrite::Missing => Err(Refusal::TaskNotFound),
            TaskWrite::Terminal { current, rev } => {
                if current == state && is_terminal_task_state(current) {
                    // Idempotent replay: the task is already in this terminal
                    // state. No resurrection and no revision churn.
                    Ok(rev)
                } else {
                    tracing::warn!(
                        task = %task_id,
                        from = current.as_str(),
                        to = state.as_str(),
                        "refused to move a terminal task to a different state"
                    );
                    Err(Refusal::TaskTerminal)
                }
            }
        }
    }

    /// Lists a run's tasks in spawn order.
    ///
    /// # Errors
    /// Database refusal.
    pub fn tasks_for_run(&self, run_id: RunId) -> Result<Vec<TaskRecord>, Refusal> {
        self.db
            .with_connection(|connection| {
                let mut statement = connection.prepare(
                    "SELECT id, run_id, parent_task_id, role, depth, spawn_seq, state, rev
                     FROM tasks WHERE run_id = ?1 ORDER BY spawn_seq ASC",
                )?;
                let rows = statement.query_map(params![run_id.to_string()], row_to_task)?;
                let mut tasks = Vec::new();
                for row in rows {
                    tasks.push(row?);
                }
                Ok(tasks)
            })
            .map_err(map)
    }

    /// Number of tasks in a run.
    ///
    /// # Errors
    /// Database refusal.
    pub fn task_count(&self, run_id: RunId) -> Result<u64, Refusal> {
        let count: i64 = self
            .db
            .with_connection(|connection| {
                connection.query_row(
                    "SELECT COUNT(*) FROM tasks WHERE run_id = ?1",
                    params![run_id.to_string()],
                    |row| row.get(0),
                )
            })
            .map_err(map)?;
        Ok(u64::try_from(count).unwrap_or(0))
    }

    /// Appends a chat message.
    ///
    /// # Errors
    /// Database refusal.
    pub fn append_message(
        &self,
        run_id: RunId,
        role: ChatRole,
        text: &str,
    ) -> Result<ChatMessage, Refusal> {
        let message = ChatMessage {
            id: Uuid::now_v7(),
            run_id,
            role,
            text: text.to_owned(),
            created_at: Timestamp::now(),
        };
        self.db
            .with_connection(|connection| {
                connection.execute(
                    "INSERT INTO chat_messages(id, run_id, role, text, created_at)
                     VALUES (?1, ?2, ?3, ?4, ?5)",
                    params![
                        message.id.to_string(),
                        message.run_id.to_string(),
                        message.role.as_str(),
                        message.text,
                        message.created_at.as_str()
                    ],
                )?;
                Ok(())
            })
            .map_err(map)?;
        Ok(message)
    }

    /// Lists a run's chat messages in insertion order.
    ///
    /// # Errors
    /// Database refusal.
    pub fn messages(&self, run_id: RunId) -> Result<Vec<ChatMessage>, Refusal> {
        self.db
            .with_connection(|connection| {
                let mut statement = connection.prepare(
                    "SELECT id, run_id, role, text, created_at FROM chat_messages
                     WHERE run_id = ?1 ORDER BY created_at ASC, rowid ASC",
                )?;
                let rows = statement.query_map(params![run_id.to_string()], |row| {
                    let role: String = row.get(2)?;
                    Ok(ChatMessage {
                        id: {
                            let raw: String = row.get(0)?;
                            raw.parse().unwrap_or_else(|_| Uuid::nil())
                        },
                        run_id: {
                            let raw: String = row.get(1)?;
                            raw.parse().unwrap_or_else(|_| RunId::from_uuid(Uuid::nil()))
                        },
                        role: ChatRole::from_str(&role).unwrap_or(ChatRole::System),
                        text: row.get(3)?,
                        created_at: {
                            let raw: String = row.get(4)?;
                            Timestamp::parse(&raw).unwrap_or_else(|_| Timestamp::from_unix_secs(0))
                        },
                    })
                })?;
                let mut messages = Vec::new();
                for row in rows {
                    messages.push(row?);
                }
                Ok(messages)
            })
            .map_err(map)
    }

    /// Records the bounded final text of a completed task.
    ///
    /// One row per task; a retry replaces the same task's row rather than
    /// duplicating it. Parent aggregators read this table, never the run-wide
    /// chat transcript, so a child result cannot be attributed to the wrong task.
    ///
    /// # Errors
    /// Database refusal.
    pub fn record_task_result(
        &self,
        run_id: RunId,
        task_id: TaskId,
        text: &str,
    ) -> Result<(), Refusal> {
        self.db
            .with_connection(|connection| {
                connection.execute(
                    "INSERT INTO task_results(task_id, run_id, text, created_at)
                     VALUES (?1, ?2, ?3, ?4)
                     ON CONFLICT(task_id) DO UPDATE SET text = excluded.text,
                                                        created_at = excluded.created_at",
                    params![
                        task_id.to_string(),
                        run_id.to_string(),
                        text,
                        Timestamp::now().as_str()
                    ],
                )?;
                Ok(())
            })
            .map_err(map)?;
        Ok(())
    }

    /// Reads a task's recorded final text.
    ///
    /// # Errors
    /// Database refusal.
    pub fn task_result(&self, task_id: TaskId) -> Result<Option<String>, Refusal> {
        self.db
            .with_connection(|connection| {
                connection
                    .query_row(
                        "SELECT text FROM task_results WHERE task_id = ?1",
                        params![task_id.to_string()],
                        |row| row.get(0),
                    )
                    .optional()
            })
            .map_err(map)
    }

    /// Records one durable agent-to-agent message.
    ///
    /// # Errors
    /// Database refusal.
    pub fn record_agent_message(
        &self,
        run_id: RunId,
        from_task_id: TaskId,
        to_task_id: TaskId,
        kind: &str,
        text: &str,
    ) -> Result<Uuid, Refusal> {
        let id = Uuid::now_v7();
        self.db
            .with_connection(|connection| {
                connection.execute(
                    "INSERT INTO agent_messages(id, run_id, from_task_id, to_task_id, kind, text, created_at)
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                    params![
                        id.to_string(),
                        run_id.to_string(),
                        from_task_id.to_string(),
                        to_task_id.to_string(),
                        kind,
                        text,
                        Timestamp::now().as_str()
                    ],
                )?;
                Ok(())
            })
            .map_err(map)?;
        Ok(id)
    }

    /// Number of durable messages recorded in a run.
    ///
    /// # Errors
    /// Database refusal.
    pub fn agent_message_count(&self, run_id: RunId) -> Result<u64, Refusal> {
        let count: i64 = self
            .db
            .with_connection(|connection| {
                connection.query_row(
                    "SELECT COUNT(*) FROM agent_messages WHERE run_id = ?1",
                    params![run_id.to_string()],
                    |row| row.get(0),
                )
            })
            .map_err(map)?;
        Ok(u64::try_from(count).unwrap_or(0))
    }

    /// Lists a run's durable agent messages in insertion order.
    ///
    /// # Errors
    /// Database refusal.
    pub fn agent_messages_for_run(
        &self,
        run_id: RunId,
    ) -> Result<Vec<AgentMessageRecord>, Refusal> {
        self.db
            .with_connection(|connection| {
                let mut statement = connection.prepare(
                    "SELECT id, from_task_id, to_task_id, kind, text, created_at
                     FROM agent_messages WHERE run_id = ?1 ORDER BY created_at ASC, rowid ASC",
                )?;
                let rows = statement.query_map(params![run_id.to_string()], |row| {
                    Ok(AgentMessageRecord {
                        id: row.get(0)?,
                        from_task_id: row.get(1)?,
                        to_task_id: row.get(2)?,
                        kind: row.get(3)?,
                        text: row.get(4)?,
                        created_at: row.get(5)?,
                    })
                })?;
                let mut messages = Vec::new();
                for row in rows {
                    messages.push(row?);
                }
                Ok(messages)
            })
            .map_err(map)
    }


    /// Upserts one durable role-scoped memory entry.
    ///
    /// # Errors
    /// Database refusal.
    pub fn record_agent_memory(
        &self,
        agent_key: &str,
        key: &str,
        value: &str,
        run_id: RunId,
        task_id: TaskId,
    ) -> Result<(), Refusal> {
        self.db
            .with_connection(|connection| {
                connection.execute(
                    "INSERT INTO agent_memory(agent_key, key, value, run_id, task_id, created_at)
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6)
                     ON CONFLICT(agent_key, key) DO UPDATE SET
                        value = excluded.value,
                        run_id = excluded.run_id,
                        task_id = excluded.task_id,
                        created_at = excluded.created_at",
                    params![
                        agent_key,
                        key,
                        value,
                        run_id.to_string(),
                        task_id.to_string(),
                        Timestamp::now().as_str()
                    ],
                )?;
                Ok(())
            })
            .map_err(map)?;
        Ok(())
    }

    /// Returns the latest durable memories for a role, newest first.
    ///
    /// # Errors
    /// Database refusal.
    pub fn agent_memory(
        &self,
        agent_key: &str,
        limit: usize,
    ) -> Result<Vec<(String, String)>, Refusal> {
        self.db
            .with_connection(|connection| {
                let mut statement = connection.prepare(
                    "SELECT key, value FROM agent_memory WHERE agent_key = ?1
                     ORDER BY created_at DESC, rowid DESC LIMIT ?2",
                )?;
                let rows = statement.query_map(
                    params![agent_key, i64::try_from(limit).unwrap_or(i64::MAX)],
                    |row| Ok((row.get(0)?, row.get(1)?)),
                )?;
                let mut memories = Vec::new();
                for row in rows {
                    memories.push(row?);
                }
                Ok(memories)
            })
            .map_err(map)
    }


    pub fn record_receipt(&self, correlation_id: Uuid, run_id: RunId) -> Result<(), Refusal> {
        self.db
            .with_connection(|connection| {
                connection.execute(
                    "INSERT OR REPLACE INTO command_receipts(correlation_id, run_id, created_at)
                     VALUES (?1, ?2, ?3)",
                    params![correlation_id.to_string(), run_id.to_string(), Timestamp::now().as_str()],
                )?;
                Ok(())
            })
            .map_err(map)?;
        Ok(())
    }

    /// Looks up the run a mutating command already created (24 h window).
    ///
    /// # Errors
    /// Database refusal.
    pub fn receipt_run(&self, correlation_id: Uuid) -> Result<Option<RunId>, Refusal> {
        let maybe: Option<(String, String)> = self
            .db
            .with_connection(|connection| {
                connection
                    .query_row(
                        "SELECT run_id, created_at FROM command_receipts WHERE correlation_id = ?1",
                        params![correlation_id.to_string()],
                        |row| Ok((row.get(0)?, row.get(1)?)),
                    )
                    .optional()
            })
            .map_err(map)?;
        let Some((run_id, created_at)) = maybe else {
            return Ok(None);
        };
        let Ok(created) = Timestamp::parse(&created_at) else {
            return Ok(None);
        };
        let Ok(created_secs) = created.unix_secs() else {
            return Ok(None);
        };
        let now = Timestamp::now().unix_secs().unwrap_or(created_secs);
        if now.saturating_sub(created_secs) > 24 * 3_600 {
            return Ok(None);
        }
        Ok(run_id.parse().ok())
    }
}

fn row_to_run(row: &rusqlite::Row<'_>) -> rusqlite::Result<Run> {
    let id: String = row.get(0)?;
    let status: String = row.get(3)?;
    let error_code: Option<String> = row.get(4)?;
    let created_at: String = row.get(5)?;
    let updated_at: String = row.get(6)?;
    Ok(Run {
        id: id.parse().unwrap_or_else(|_| RunId::from_uuid(Uuid::nil())),
        project_id: row.get(1)?,
        prompt: row.get(2)?,
        status: parse_state(&status),
        error_code,
        created_at: Timestamp::parse(&created_at).unwrap_or_else(|_| Timestamp::from_unix_secs(0)),
        updated_at: Timestamp::parse(&updated_at).unwrap_or_else(|_| Timestamp::from_unix_secs(0)),
    })
}

fn row_to_task(row: &rusqlite::Row<'_>) -> rusqlite::Result<TaskRecord> {
    let id: String = row.get(0)?;
    let run_id: String = row.get(1)?;
    let parent: Option<String> = row.get(2)?;
    let state: String = row.get(6)?;
    let depth: i64 = row.get(4)?;
    let spawn_seq: i64 = row.get(5)?;
    let rev: i64 = row.get(7)?;
    Ok(TaskRecord {
        id: id.parse().unwrap_or_else(|_| TaskId::from_uuid(Uuid::nil())),
        run_id: run_id
            .parse()
            .unwrap_or_else(|_| RunId::from_uuid(Uuid::nil())),
        parent_task_id: parent.and_then(|raw| raw.parse().ok()),
        role: row.get(3)?,
        depth: u8::try_from(depth).unwrap_or(u8::MAX),
        spawn_seq: u64::try_from(spawn_seq).unwrap_or(0),
        state: parse_state(&state),
        rev: u64::try_from(rev).unwrap_or(0),
    })
}

fn parse_state(value: &str) -> AgentState {
    match value {
        "healthy" => AgentState::Healthy,
        "running" => AgentState::Running,
        "queued" => AgentState::Queued,
        "failed" => AgentState::Failed,
        "completed" => AgentState::Completed,
        "cancelled" => AgentState::Cancelled,
        "killed" => AgentState::Killed,
        "planning" => AgentState::Planning,
        "thinking" => AgentState::Thinking,
        "awaiting_approval" => AgentState::AwaitingApproval,
        "streaming" => AgentState::Streaming,
        "reconnecting" => AgentState::Reconnecting,
        "replay" => AgentState::Replay,
        "empty" => AgentState::Empty,
        "loading" => AgentState::Loading,
        "interrupted" => AgentState::Interrupted,
        _ => AgentState::Failed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn store() -> RunStore {
        let db = Database::open_in_memory().expect("in-memory database");
        db.migrate().expect("migrate");
        RunStore::new(Arc::new(db))
    }

    fn seed_task(store: &RunStore, state: AgentState) -> TaskRecord {
        let run = store
            .create_run("project-1", "terminal guard", AgentState::Running)
            .expect("run");
        let task = TaskRecord {
            id: TaskId::new(),
            run_id: run.id,
            parent_task_id: None,
            role: "worker".to_owned(),
            depth: 1,
            spawn_seq: 1,
            state,
            rev: 0,
        };
        store.insert_task(&task).expect("task row");
        task
    }

    #[test]
    fn terminal_task_cannot_be_resurrected() {
        let store = store();
        let task = seed_task(&store, AgentState::Queued);

        // Non-terminal writes stay legal and bump the revision.
        let running = store
            .update_task_state(task.id, AgentState::Running)
            .expect("start");
        assert!(running > 0, "a real transition bumps the revision");
        let completed = store
            .update_task_state(task.id, AgentState::Completed)
            .expect("complete");
        assert!(completed > running);

        // Every resurrection attempt is refused by name and changes nothing.
        for attempted in [
            AgentState::Running,
            AgentState::Killed,
            AgentState::Failed,
            AgentState::Interrupted,
        ] {
            assert_eq!(
                store.update_task_state(task.id, attempted),
                Err(Refusal::TaskTerminal),
                "{attempted} must not resurrect a completed task"
            );
            let stored = store.get_task(task.id).expect("task");
            assert_eq!(stored.state, AgentState::Completed);
            assert_eq!(stored.rev, completed, "a refused write must not bump rev");
        }

        // Replaying the same terminal state is idempotent, not an error.
        assert_eq!(
            store.update_task_state(task.id, AgentState::Completed),
            Ok(completed)
        );
        assert_eq!(store.get_task(task.id).expect("task").rev, completed);
    }

    #[test]
    fn killed_task_stays_killed_after_a_late_completion() {
        let store = store();
        let task = seed_task(&store, AgentState::Running);
        store
            .update_task_state(task.id, AgentState::Killed)
            .expect("kill");
        assert_eq!(
            store.update_task_state(task.id, AgentState::Completed),
            Err(Refusal::TaskTerminal),
            "a late completion may not overwrite an acknowledged kill"
        );
        assert_eq!(
            store.get_task(task.id).expect("task").state,
            AgentState::Killed
        );
    }

    #[test]
    fn terminal_sql_list_matches_the_state_machine() {
        let list = terminal_task_state_sql();
        for terminal in TASK_TERMINAL_STATES {
            assert!(
                list.contains(&format!("'{}'", terminal.as_str())),
                "terminal set {list} is missing {}",
                terminal.as_str()
            );
        }
        for active in [
            AgentState::Queued,
            AgentState::Running,
            AgentState::AwaitingApproval,
        ] {
            assert!(
                !list.contains(&format!("'{}'", active.as_str())),
                "active state {active} must stay settlable"
            );
        }
    }

    #[test]
    fn unknown_task_update_is_refused_by_name() {
        let store = store();
        assert_eq!(
            store.update_task_state(TaskId::new(), AgentState::Running),
            Err(Refusal::TaskNotFound)
        );
    }

    #[test]
    fn agent_messages_are_durable_and_counted_per_run() {
        let store = store();
        let run = store
            .create_run("p", "message", AgentState::Running)
            .expect("run");
        let from = TaskId::new();
        let to = TaskId::new();
        let message_id = store
            .record_agent_message(run.id, from, to, "ask_parent", "hello")
            .expect("message");
        assert_ne!(message_id, Uuid::nil());
        assert_eq!(store.agent_message_count(run.id).expect("count"), 1);
        assert_eq!(store.agent_message_count(RunId::new()).expect("other"), 0);
    }

    #[test]
    fn agent_memory_is_durable_and_role_scoped() {
        let store = store();
        let run = store
            .create_run("p", "memory", AgentState::Running)
            .expect("run");
        let task = TaskId::new();
        store
            .record_agent_memory("worker", "task:1", "remembered", run.id, task)
            .expect("memory");
        store
            .record_agent_memory("worker", "task:1", "updated", run.id, task)
            .expect("upsert");
        let memories = store.agent_memory("worker", 8).expect("recall");
        assert_eq!(memories, vec![("task:1".to_owned(), "updated".to_owned())]);
        assert!(store.agent_memory("other", 8).expect("other").is_empty());
    }
}
