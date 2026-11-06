//! SQLite store for runs, tasks, chat messages and command receipts
//! (blueprint Phase 4, WP 4.6; schema created by migration v2).

use std::sync::Arc;

use hiverouter_contract::{AgentState, Refusal, RunId, TaskId, Timestamp};
use rusqlite::{params, OptionalExtension};
use uuid::Uuid;

use crate::db::{Database, DbError};
use crate::entities::{ChatMessage, ChatRole, Run, TaskRecord};

fn map(error: DbError) -> Refusal {
    error.refusal()
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
    /// # Errors
    /// `E_TASK_NOT_FOUND` when the row does not exist.
    pub fn update_task_state(&self, task_id: TaskId, state: AgentState) -> Result<u64, Refusal> {
        let updated = self
            .db
            .with_connection(|connection| {
                connection.execute(
                    "UPDATE tasks SET state = ?2, rev = rev + 1 WHERE id = ?1",
                    params![task_id.to_string(), state.as_str()],
                )
            })
            .map_err(map)?;
        if updated == 0 {
            return Err(Refusal::TaskNotFound);
        }
        Ok(self.get_task(task_id)?.rev)
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

    /// Records a mutating command receipt (idempotency by correlation id).
    ///
    /// # Errors
    /// Database refusal.
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
