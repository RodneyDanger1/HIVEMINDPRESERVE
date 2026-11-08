//! SQLite store for runs, tasks, chat messages and command receipts
//! (blueprint Phase 4, WP 4.6; schema created by migration v2).

use std::sync::{Arc, OnceLock};

use hiverouter_contract::{AgentState, Refusal, RunId, TaskId, Timestamp, Usage};
use rusqlite::{params, OptionalExtension};
use uuid::Uuid;

use crate::db::{Database, DbError};
use crate::entities::{ChatMessage, ChatRole, Run, TaskRecord};
use crate::run_state::{is_terminal_task_state, TASK_TERMINAL_STATES};

fn map(error: DbError) -> Refusal {
    error.refusal()
}

/// One durable memory row, including its hive/branch scope.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AgentMemoryRecord {
    /// Owning run.
    pub run_id: String,
    /// Hive/branch scope (parent task id or `root`).
    pub scope_id: String,
    /// Agent/role key.
    pub agent_key: String,
    /// Memory key.
    pub key: String,
    /// Memory value.
    pub value: String,
    /// Task that wrote the row.
    pub task_id: String,
    /// RFC3339 creation time.
    pub created_at: String,
}

/// Persisted task identity derived at admission by the v13 task trigger.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TaskIdentity {
    /// `hyper`, `super`, `agent`, or `subagent`.
    pub class: String,
    /// Rank 1..4.
    pub rank: u8,
    /// Persona/profile id.
    pub profile_id: String,
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
    /// Message lifecycle status (`open`, `answered`, `expired`).
    pub status: String,
    /// Optional deadline (RFC3339).
    pub deadline_at: Option<String>,
    /// Optional answer time (RFC3339).
    pub answered_at: Option<String>,
    /// Optional operator/parent answer text.
    pub answer_text: Option<String>,
    /// Actor that answered (`operator`, task id, etc.).
    pub answer_actor: Option<String>,
    /// Optional message this row replies to.
    pub in_reply_to: Option<String>,
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
                    "INSERT OR IGNORE INTO projects(id, name, created_at) VALUES (?1, ?1, ?2)",
                    params![run.project_id, run.created_at.as_str()],
                )?;
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

    /// Lists durable projects in creation order.
    ///
    /// # Errors
    /// Database refusal.
    pub fn projects(&self) -> Result<Vec<(String, String, String)>, Refusal> {
        self.db
            .with_connection(|connection| {
                let mut statement =
                    connection.prepare("SELECT id, name, created_at FROM projects ORDER BY created_at ASC, id ASC")?;
                let rows = statement.query_map([], |row| {
                    Ok((row.get(0)?, row.get(1)?, row.get(2)?))
                })?;
                let mut projects = Vec::new();
                for row in rows {
                    projects.push(row?);
                }
                Ok(projects)
            })
            .map_err(map)
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

    /// Moves a persisted task row to a new parent and pins its row revision to
    /// the graph node revision produced by the same rewire.
    ///
    /// The graph is the live authority, but roster/fan-out/A2A readers use the
    /// task rows. Without this write those readers keep the pre-rewire parent
    /// forever, including across a restart.
    ///
    /// # Errors
    /// `E_TASK_NOT_FOUND` when the row does not exist; database refusal otherwise.
    pub fn update_task_parent(
        &self,
        task_id: TaskId,
        parent_task_id: Option<TaskId>,
        rev: u64,
    ) -> Result<(), Refusal> {
        let id = task_id.to_string();
        let parent = parent_task_id.map(|parent| parent.to_string());
        let revision = i64::try_from(rev).map_err(|_| Refusal::InternalError)?;
        let updated = self
            .db
            .with_connection(|connection| {
                connection.execute(
                    "UPDATE tasks SET parent_task_id = ?2, rev = ?3 WHERE id = ?1",
                    params![id, parent, revision],
                )
            })
            .map_err(map)?;
        if updated == 0 {
            return Err(Refusal::TaskNotFound);
        }
        Ok(())
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

    /// Records one durable agent-to-agent message under an atomic run budget.
    ///
    /// The insert and the count check are one SQL statement, so concurrent
    /// senders cannot both pass a check and exceed the budget.
    ///
    /// # Errors
    /// `E_CALL_LIMIT` when the run already holds `max_messages`; database refusal otherwise.
    pub fn record_agent_message_limited(
        &self,
        run_id: RunId,
        from_task_id: TaskId,
        to_task_id: TaskId,
        kind: &str,
        text: &str,
        max_messages: u64,
    ) -> Result<Uuid, Refusal> {
        let id = Uuid::now_v7();
        let changed = self
            .db
            .with_connection(|connection| {
                connection.execute(
                    "INSERT INTO agent_messages(id, run_id, from_task_id, to_task_id, kind, text, created_at)
                     SELECT ?1, ?2, ?3, ?4, ?5, ?6, ?7
                     WHERE (SELECT COUNT(*) FROM agent_messages WHERE run_id = ?2) < ?8",
                    params![
                        id.to_string(),
                        run_id.to_string(),
                        from_task_id.to_string(),
                        to_task_id.to_string(),
                        kind,
                        text,
                        Timestamp::now().as_str(),
                        i64::try_from(max_messages).unwrap_or(i64::MAX)
                    ],
                )
            })
            .map_err(map)?;
        if changed == 0 {
            return Err(Refusal::CallLimit);
        }
        Ok(id)
    }

    /// Answers one open A2A message with a compare-and-set transition.
    ///
    /// # Errors
    /// `E_TASK_NOT_FOUND` when the message is missing; `E_TASK_TERMINAL` when it
    /// is no longer open; database refusal otherwise.
    pub fn answer_agent_message(
        &self,
        run_id: RunId,
        message_id: Uuid,
        actor: &str,
        answer: &str,
    ) -> Result<(), Refusal> {
        let changed = self
            .db
            .with_connection(|connection| {
                connection.execute(
                    "UPDATE agent_messages
                     SET status = 'answered', answered_at = ?3, answer_text = ?4,
                         answer_actor = ?5
                     WHERE run_id = ?1 AND id = ?2 AND status = 'open'",
                    params![
                        run_id.to_string(),
                        message_id.to_string(),
                        Timestamp::now().as_str(),
                        answer,
                        actor
                    ],
                )
            })
            .map_err(map)?;
        if changed == 1 {
            return Ok(());
        }
        let exists: i64 = self
            .db
            .with_connection(|connection| {
                connection.query_row(
                    "SELECT COUNT(*) FROM agent_messages WHERE run_id = ?1 AND id = ?2",
                    params![run_id.to_string(), message_id.to_string()],
                    |row| row.get(0),
                )
            })
            .map_err(map)?;
        if exists == 0 {
            Err(Refusal::TaskNotFound)
        } else {
            Err(Refusal::TaskTerminal)
        }
    }

    /// Sets the deadline for one open message.
    ///
    /// # Errors
    /// Database refusal.
    pub fn set_agent_message_deadline(
        &self,
        run_id: RunId,
        message_id: Uuid,
        deadline: &str,
    ) -> Result<(), Refusal> {
        self.db
            .with_connection(|connection| {
                connection.execute(
                    "UPDATE agent_messages SET deadline_at = ?3
                     WHERE run_id = ?1 AND id = ?2 AND status = 'open'",
                    params![run_id.to_string(), message_id.to_string(), deadline],
                )
            })
            .map_err(map)?;
        Ok(())
    }

    /// Reads one message's current status, if it exists.
    ///
    /// # Errors
    /// Database refusal.
    pub fn agent_message_status(
        &self,
        run_id: RunId,
        message_id: Uuid,
    ) -> Result<Option<String>, Refusal> {
        self.db
            .with_connection(|connection| {
                connection
                    .query_row(
                        "SELECT status FROM agent_messages WHERE run_id = ?1 AND id = ?2",
                        params![run_id.to_string(), message_id.to_string()],
                        |row| row.get(0),
                    )
                    .optional()
            })
            .map_err(map)
    }

    /// Marks an open message expired.
    ///
    /// # Errors
    /// Database refusal.
    pub fn expire_agent_message(&self, run_id: RunId, message_id: Uuid) -> Result<(), Refusal> {
        self.db
            .with_connection(|connection| {
                connection.execute(
                    "UPDATE agent_messages SET status = 'expired'
                     WHERE run_id = ?1 AND id = ?2 AND status = 'open'",
                    params![run_id.to_string(), message_id.to_string()],
                )
            })
            .map_err(map)?;
        Ok(())
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

    /// Reads a task's persisted class/rank/profile, when the v13 trigger
    /// resolved it. An unknown role stays `None`; it is never guessed.
    ///
    /// # Errors
    /// Database refusal.
    pub fn task_identity(&self, task_id: TaskId) -> Result<Option<TaskIdentity>, Refusal> {
        self.db
            .with_connection(|connection| {
                let row: Option<(Option<String>, Option<i64>, Option<String>)> = connection
                    .query_row(
                        "SELECT class, rank, profile_id FROM tasks WHERE id = ?1",
                        params![task_id.to_string()],
                        |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
                    )
                    .optional()?;
                Ok(row.and_then(|(class, rank, profile)| {
                    Some(TaskIdentity {
                        class: class?,
                        rank: u8::try_from(rank?).ok()?,
                        profile_id: profile?,
                    })
                }))
            })
            .map_err(map)
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
                    "SELECT id, from_task_id, to_task_id, kind, text, created_at,
                            status, deadline_at, answered_at, answer_text, answer_actor, in_reply_to
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
                        status: row.get(6)?,
                        deadline_at: row.get(7)?,
                        answered_at: row.get(8)?,
                        answer_text: row.get(9)?,
                        answer_actor: row.get(10)?,
                        in_reply_to: row.get(11)?,
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

    /// Lists durable messages addressed to one task inside one run, newest
    /// first and bounded by `limit`. This is the recipient's A2A inbox.
    ///
    /// # Errors
    /// Database refusal.
    pub fn agent_messages_for_task(
        &self,
        run_id: RunId,
        task_id: TaskId,
        limit: usize,
    ) -> Result<Vec<AgentMessageRecord>, Refusal> {
        self.db
            .with_connection(|connection| {
                let mut statement = connection.prepare(
                    "SELECT id, from_task_id, to_task_id, kind, text, created_at,
                            status, deadline_at, answered_at, answer_text, answer_actor, in_reply_to
                     FROM agent_messages
                     WHERE run_id = ?1 AND to_task_id = ?2
                     ORDER BY created_at DESC, rowid DESC LIMIT ?3",
                )?;
                let rows = statement.query_map(
                    params![
                        run_id.to_string(),
                        task_id.to_string(),
                        i64::try_from(limit).unwrap_or(i64::MAX)
                    ],
                    |row| {
                        Ok(AgentMessageRecord {
                            id: row.get(0)?,
                            from_task_id: row.get(1)?,
                            to_task_id: row.get(2)?,
                            kind: row.get(3)?,
                            text: row.get(4)?,
                            created_at: row.get(5)?,
                            status: row.get(6)?,
                            deadline_at: row.get(7)?,
                            answered_at: row.get(8)?,
                            answer_text: row.get(9)?,
                            answer_actor: row.get(10)?,
                            in_reply_to: row.get(11)?,
                        })
                    },
                )?;
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
        self.record_agent_memory_scoped(agent_key, key, value, run_id, task_id, "run")
    }

    /// Writes one memory entry inside a run **and branch scope**. `scope_id` is
    /// the owning hive/branch key (the parent task id, or `root`).
    ///
    /// # Errors
    /// Database refusal.
    pub fn record_agent_memory_scoped(
        &self,
        agent_key: &str,
        key: &str,
        value: &str,
        run_id: RunId,
        task_id: TaskId,
        scope_id: &str,
    ) -> Result<(), Refusal> {
        self.db
            .with_connection(|connection| {
                connection.execute(
                    "INSERT INTO agent_memory(run_id, scope_id, agent_key, key, value, task_id, created_at)
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
                     ON CONFLICT(run_id, scope_id, agent_key, key) DO UPDATE SET
                        value = excluded.value,
                        task_id = excluded.task_id,
                        created_at = excluded.created_at",
                    params![
                        run_id.to_string(),
                        scope_id,
                        agent_key,
                        key,
                        value,
                        task_id.to_string(),
                        Timestamp::now().as_str()
                    ],
                )?;
                Ok(())
            })
            .map_err(map)?;
        Ok(())
    }

    /// Returns the latest durable memories for a role **inside one run**,
    /// newest first. This is the recall path used by a running task: facts from
    /// another run or another hive must not be injected.
    ///
    /// # Errors
    /// Database refusal.
    pub fn agent_memory_for_run(
        &self,
        run_id: RunId,
        agent_key: &str,
        limit: usize,
    ) -> Result<Vec<(String, String)>, Refusal> {
        self.db
            .with_connection(|connection| {
                let mut statement = connection.prepare(
                    "SELECT key, value FROM agent_memory WHERE run_id = ?1 AND agent_key = ?2
                     ORDER BY created_at DESC, rowid DESC LIMIT ?3",
                )?;
                let rows = statement.query_map(
                    params![
                        run_id.to_string(),
                        agent_key,
                        i64::try_from(limit).unwrap_or(i64::MAX)
                    ],
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

    /// Returns the latest durable memories for one run/scope across every role,
    /// newest first. This is the branch/run bank recall path: sibling isolation
    /// comes from selecting the correct scope, not from a role-name filter.
    ///
    /// # Errors
    /// Database refusal.
    pub fn agent_memory_for_scope_any_role(
        &self,
        run_id: RunId,
        scope_id: &str,
        limit: usize,
    ) -> Result<Vec<(String, String)>, Refusal> {
        self.db
            .with_connection(|connection| {
                let mut statement = connection.prepare(
                    "SELECT key, value FROM agent_memory
                     WHERE run_id = ?1 AND scope_id = ?2
                     ORDER BY created_at DESC, rowid DESC LIMIT ?3",
                )?;
                let rows = statement.query_map(
                    params![
                        run_id.to_string(),
                        scope_id,
                        i64::try_from(limit).unwrap_or(i64::MAX)
                    ],
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

    /// Returns the latest durable memories for a role inside one run and one
    /// hive/branch scope, newest first.
    ///
    /// # Errors
    /// Database refusal.
    pub fn agent_memory_for_scope(
        &self,
        run_id: RunId,
        scope_id: &str,
        agent_key: &str,
        limit: usize,
    ) -> Result<Vec<(String, String)>, Refusal> {
        self.db
            .with_connection(|connection| {
                let mut statement = connection.prepare(
                    "SELECT key, value FROM agent_memory
                     WHERE run_id = ?1 AND scope_id = ?2 AND agent_key = ?3
                     ORDER BY created_at DESC, rowid DESC LIMIT ?4",
                )?;
                let rows = statement.query_map(
                    params![
                        run_id.to_string(),
                        scope_id,
                        agent_key,
                        i64::try_from(limit).unwrap_or(i64::MAX)
                    ],
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

    /// Lists every durable memory row for one run with its branch scope.
    ///
    /// This is the read-only operator surface; it never feeds a running task.
    ///
    /// # Errors
    /// Database refusal.
    pub fn agent_memory_entries_for_run(
        &self,
        run_id: RunId,
        limit: usize,
    ) -> Result<Vec<AgentMemoryRecord>, Refusal> {
        self.agent_memory_entries_scoped(run_id, None, limit)
    }

    /// Lists run memory rows, optionally limited to one hive/branch scope.
    ///
    /// # Errors
    /// Database refusal.
    pub fn agent_memory_entries_for_scope(
        &self,
        run_id: RunId,
        scope_id: &str,
        limit: usize,
    ) -> Result<Vec<AgentMemoryRecord>, Refusal> {
        self.agent_memory_entries_scoped(run_id, Some(scope_id), limit)
    }

    fn agent_memory_entries_scoped(
        &self,
        run_id: RunId,
        scope_id: Option<&str>,
        limit: usize,
    ) -> Result<Vec<AgentMemoryRecord>, Refusal> {
        self.db
            .with_connection(|connection| {
                let limit = i64::try_from(limit).unwrap_or(i64::MAX);
                let mut statement = connection.prepare(
                    "SELECT run_id, scope_id, agent_key, key, value, task_id, created_at
                     FROM agent_memory
                     WHERE run_id = ?1 AND (?2 IS NULL OR scope_id = ?2)
                     ORDER BY created_at DESC, rowid DESC LIMIT ?3",
                )?;
                let rows = statement.query_map(
                    params![run_id.to_string(), scope_id, limit],
                    |row| {
                        Ok(AgentMemoryRecord {
                            run_id: row.get(0)?,
                            scope_id: row.get(1)?,
                            agent_key: row.get(2)?,
                            key: row.get(3)?,
                            value: row.get(4)?,
                            task_id: row.get(5)?,
                            created_at: row.get(6)?,
                        })
                    },
                )?;
                let mut memories = Vec::new();
                for row in rows {
                    memories.push(row?);
                }
                Ok(memories)
            })
            .map_err(map)
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


    /// Persists one run's measured token totals (NULL stays NULL).
    ///
    /// # Errors
    /// Database refusal.
    pub fn record_run_usage(&self, run_id: RunId, usage: &Usage) -> Result<(), Refusal> {
        let as_sql = |value: Option<u64>| value.map(|value| i64::try_from(value).unwrap_or(i64::MAX));
        self.db
            .with_connection(|connection| {
                connection.execute(
                    "INSERT INTO run_usage(run_id, prompt_tokens, completion_tokens, total_tokens,
                        prompt_cache_hit_tokens, prompt_cache_miss_tokens, reasoning_tokens, created_at)
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
                     ON CONFLICT(run_id) DO UPDATE SET
                        prompt_tokens = excluded.prompt_tokens,
                        completion_tokens = excluded.completion_tokens,
                        total_tokens = excluded.total_tokens,
                        prompt_cache_hit_tokens = excluded.prompt_cache_hit_tokens,
                        prompt_cache_miss_tokens = excluded.prompt_cache_miss_tokens,
                        reasoning_tokens = excluded.reasoning_tokens,
                        created_at = excluded.created_at",
                    params![
                        run_id.to_string(),
                        as_sql(usage.prompt_tokens),
                        as_sql(usage.completion_tokens),
                        as_sql(usage.total_tokens),
                        as_sql(usage.prompt_cache_hit_tokens),
                        as_sql(usage.prompt_cache_miss_tokens),
                        as_sql(usage.reasoning_tokens),
                        Timestamp::now().as_str()
                    ],
                )?;
                Ok(())
            })
            .map_err(map)?;
        Ok(())
    }

    /// Reads a run's persisted usage, if any.
    ///
    /// # Errors
    /// Database refusal.
    pub fn run_usage(&self, run_id: RunId) -> Result<Option<Usage>, Refusal> {
        self.db
            .with_connection(|connection| {
                connection
                    .query_row(
                        "SELECT prompt_tokens, completion_tokens, total_tokens,
                                prompt_cache_hit_tokens, prompt_cache_miss_tokens, reasoning_tokens
                         FROM run_usage WHERE run_id = ?1",
                        params![run_id.to_string()],
                        |row| {
                            let read = |index: usize| -> rusqlite::Result<Option<u64>> {
                                Ok(row
                                    .get::<_, Option<i64>>(index)?
                                    .and_then(|value| u64::try_from(value).ok()))
                            };
                            Ok(Usage {
                                prompt_tokens: read(0)?,
                                completion_tokens: read(1)?,
                                total_tokens: read(2)?,
                                prompt_cache_hit_tokens: read(3)?,
                                prompt_cache_miss_tokens: read(4)?,
                                reasoning_tokens: read(5)?,
                            })
                        },
                    )
                    .optional()
            })
            .map_err(map)
    }

    /// Lists persisted imported profiles as `(id, profile JSON, source)`.
    ///
    /// # Errors
    /// Database or stored-JSON refusal.
    pub fn agent_profiles(
        &self,
    ) -> Result<Vec<(String, serde_json::Value, String)>, Refusal> {
        let rows = self
            .db
            .with_connection(|connection| {
                let mut statement =
                    connection.prepare("SELECT id, profile_json, source FROM agent_profiles")?;
                let rows = statement.query_map([], |row| {
                    Ok((
                        row.get::<_, String>(0)?,
                        row.get::<_, String>(1)?,
                        row.get::<_, String>(2)?,
                    ))
                })?;
                let mut profiles = Vec::new();
                for row in rows {
                    profiles.push(row?);
                }
                Ok(profiles)
            })
            .map_err(map)?;
        let mut decoded = Vec::new();
        for (id, json, source) in rows {
            let value = serde_json::from_str(&json).map_err(|_| Refusal::SchemaInvalid)?;
            decoded.push((id, value, source));
        }
        Ok(decoded)
    }

    /// Upserts one imported profile as validated JSON.
    ///
    /// # Errors
    /// Database refusal.
    pub fn upsert_agent_profile(
        &self,
        id: &str,
        profile: &serde_json::Value,
        source: &str,
    ) -> Result<(), Refusal> {
        let json = serde_json::to_string(profile).map_err(|_| Refusal::SchemaInvalid)?;
        self.db
            .with_connection(|connection| {
                connection.execute(
                    "INSERT INTO agent_profiles(id, profile_json, source, imported_at)
                     VALUES (?1, ?2, ?3, ?4)
                     ON CONFLICT(id) DO UPDATE SET
                        profile_json = excluded.profile_json,
                        source = excluded.source,
                        imported_at = excluded.imported_at",
                    params![id, json, source, Timestamp::now().as_str()],
                )?;
                Ok(())
            })
            .map_err(map)?;
        Ok(())
    }

    /// Deletes one imported profile; returns whether a row was removed.
    ///
    /// # Errors
    /// Database refusal.
    pub fn delete_agent_profile(&self, id: &str) -> Result<bool, Refusal> {
        let removed = self
            .db
            .with_connection(|connection| {
                connection.execute("DELETE FROM agent_profiles WHERE id = ?1", params![id])
            })
            .map_err(map)?;
        Ok(removed > 0)
    }

    /// Clears one role's durable memories.
    ///
    /// # Errors
    /// Database refusal.
    pub fn clear_agent_memory(&self, agent_key: &str) -> Result<u64, Refusal> {
        let removed = self
            .db
            .with_connection(|connection| {
                connection.execute(
                    "DELETE FROM agent_memory WHERE agent_key = ?1",
                    params![agent_key],
                )
            })
            .map_err(map)?;
        Ok(u64::try_from(removed).unwrap_or(0))
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
    fn task_parent_row_follows_a_rewire() {
        let store = store();
        let task = seed_task(&store, AgentState::Running);
        let new_parent = TaskId::new();
        store
            .update_task_parent(task.id, Some(new_parent), 9)
            .expect("parent write");
        let stored = store.get_task(task.id).expect("task");
        assert_eq!(stored.parent_task_id, Some(new_parent));
        assert_eq!(stored.rev, 9, "row rev is pinned to the graph rev");
        assert_eq!(
            store.update_task_parent(TaskId::new(), None, 1),
            Err(Refusal::TaskNotFound)
        );
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
    fn agent_message_budget_is_atomic_and_named() {
        let store = store();
        let run = store
            .create_run("p", "message budget", AgentState::Running)
            .expect("run");
        let from = TaskId::new();
        let to = TaskId::new();
        store
            .record_agent_message_limited(run.id, from, to, "ask_parent", "one", 1)
            .expect("first message");
        assert_eq!(
            store.record_agent_message_limited(run.id, from, to, "ask_parent", "two", 1),
            Err(Refusal::CallLimit)
        );
        assert_eq!(store.agent_message_count(run.id).expect("count"), 1);
    }

    #[test]
    fn a2a_answer_is_cas_and_duplicate_is_refused() {
        let store = store();
        let run = store
            .create_run("p", "answer", AgentState::Running)
            .expect("run");
        let from = TaskId::new();
        let to = TaskId::new();
        let message_id = store
            .record_agent_message(run.id, from, to, "ask_parent", "question")
            .expect("message");
        store
            .answer_agent_message(run.id, message_id, "operator", "answer")
            .expect("first answer");
        assert_eq!(
            store.answer_agent_message(run.id, message_id, "operator", "again"),
            Err(Refusal::TaskTerminal)
        );
        let messages = store.agent_messages_for_run(run.id).expect("messages");
        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0].status, "answered");
        assert_eq!(messages[0].answer_text.as_deref(), Some("answer"));
        assert_eq!(messages[0].answer_actor.as_deref(), Some("operator"));
        assert_eq!(
            store.answer_agent_message(run.id, Uuid::now_v7(), "operator", "missing"),
            Err(Refusal::TaskNotFound)
        );
    }

    #[test]
    fn task_identity_is_persisted_by_the_v13_trigger() {
        let store = store();
        let run = store
            .create_run("p", "identity", AgentState::Running)
            .expect("run");
        let subagent = TaskRecord {
            id: TaskId::new(),
            run_id: run.id,
            parent_task_id: None,
            role: "sub-agent".to_owned(),
            depth: 2,
            spawn_seq: 1,
            state: AgentState::Queued,
            rev: 0,
        };
        let unknown = TaskRecord {
            id: TaskId::new(),
            run_id: run.id,
            parent_task_id: None,
            role: "researcher".to_owned(),
            depth: 1,
            spawn_seq: 2,
            state: AgentState::Queued,
            rev: 0,
        };
        store.insert_task(&subagent).expect("subagent");
        store.insert_task(&unknown).expect("unknown");
        let identity = store
            .task_identity(subagent.id)
            .expect("identity")
            .expect("resolved");
        assert_eq!(identity.class, "subagent");
        assert_eq!(identity.rank, 4);
        assert_eq!(identity.profile_id, "subagent");
        assert!(store.task_identity(unknown.id).expect("unknown").is_none());
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

    #[test]
    fn agent_memory_is_run_scoped() {
        let store = store();
        let first = store
            .create_run("p", "first", AgentState::Running)
            .expect("first run");
        let second = store
            .create_run("p", "second", AgentState::Running)
            .expect("second run");
        store
            .record_agent_memory("worker", "task:1", "from-first", first.id, TaskId::new())
            .expect("first memory");
        store
            .record_agent_memory("worker", "task:1", "from-second", second.id, TaskId::new())
            .expect("second memory");
        assert_eq!(
            store
                .agent_memory_for_run(first.id, "worker", 8)
                .expect("first recall"),
            vec![("task:1".to_owned(), "from-first".to_owned())]
        );
        assert_eq!(
            store
                .agent_memory_for_run(second.id, "worker", 8)
                .expect("second recall"),
            vec![("task:1".to_owned(), "from-second".to_owned())]
        );
    }

    #[test]
    fn agent_memory_is_branch_scoped() {
        let store = store();
        let run = store
            .create_run("p", "branch", AgentState::Running)
            .expect("run");
        store
            .record_agent_memory_scoped("worker", "task:1", "hive-a", run.id, TaskId::new(), "hive-a")
            .expect("hive a");
        store
            .record_agent_memory_scoped("worker", "task:1", "hive-b", run.id, TaskId::new(), "hive-b")
            .expect("hive b");
        assert_eq!(
            store
                .agent_memory_for_scope(run.id, "hive-a", "worker", 8)
                .expect("hive a recall"),
            vec![("task:1".to_owned(), "hive-a".to_owned())]
        );
        assert_eq!(
            store
                .agent_memory_for_scope(run.id, "hive-b", "worker", 8)
                .expect("hive b recall"),
            vec![("task:1".to_owned(), "hive-b".to_owned())]
        );
    }

    #[test]
    fn create_run_materializes_its_project_once() {
        let store = store();
        store
            .create_run("project-a", "first", AgentState::Running)
            .expect("first run");
        store
            .create_run("project-a", "second", AgentState::Running)
            .expect("second run");
        store
            .create_run("project-b", "third", AgentState::Running)
            .expect("third run");
        let projects = store.projects().expect("projects");
        assert_eq!(projects.len(), 2);
        assert!(projects.iter().any(|(id, _, _)| id == "project-a"));
        assert!(projects.iter().any(|(id, _, _)| id == "project-b"));
    }

    #[test]
    fn run_usage_persists_nulls_not_zero() {
        let store = store();
        let run = store
            .create_run("p", "usage", AgentState::Running)
            .expect("run");
        let usage = Usage {
            prompt_tokens: Some(10),
            completion_tokens: None,
            total_tokens: Some(10),
            ..Usage::default()
        };
        store.record_run_usage(run.id, &usage).expect("usage");
        let reread = store.run_usage(run.id).expect("read").expect("row");
        assert_eq!(reread.prompt_tokens, Some(10));
        assert_eq!(reread.completion_tokens, None);
        assert_eq!(reread.total_tokens, Some(10));
    }
}
