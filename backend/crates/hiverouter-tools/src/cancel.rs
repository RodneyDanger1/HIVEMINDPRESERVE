//! Run/task cancellation and kill enforcement (Phase 6 WP 6.7).
//!
//! One [`ProcessRegistry`] owns the per-run `CancellationToken` and the
//! terminal-state transitions. Core's Phase 4 `RunRegistry` owns run
//! bookkeeping and calls into this trait surface for the actual process kill
//! (see `docs/contracts/ADAPTERS.md` adapter note): the tools layer owns the
//! process/Job-Object teardown; core owns the run row.
//!
//! Rules:
//! - cancel is idempotent (cancelling twice is not a failure);
//! - a terminal run refuses further kill commands with `E_RUN_TERMINAL`;
//! - a task that was never registered cannot be killed (`E_TASK_NOT_FOUND`);
//! - the [`CancellationToken`] is the single broadcast mechanism children
//!   select on; no `JoinHandle::abort` is used for graceful cancellation.

use std::collections::HashMap;
use std::sync::Mutex;

use hiverouter_contract::{Refusal, RunId, TaskId, Timestamp};
use serde::Serialize;
use tokio_util::sync::CancellationToken;

use crate::error::ToolError;

/// Terminal state broadcast to the event fan-out.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TerminalState {
    /// The run was cancelled (planned teardown).
    Cancelled,
    /// One worker/task was killed on request.
    Killed,
}

impl TerminalState {
    /// Stable label for the ledger.
    #[must_use]
    pub fn label(self) -> &'static str {
        match self {
            TerminalState::Cancelled => "cancelled",
            TerminalState::Killed => "killed",
        }
    }
}

/// Outcome of a cancel/kill request.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TerminalOutcome {
    /// Terminal state applied.
    pub state: TerminalState,
    /// Whether this call was the one that moved the run to terminal.
    pub newly_terminal: bool,
    /// When the terminal state was recorded.
    pub at: Timestamp,
}

#[derive(Debug)]
struct RunEntry {
    token: CancellationToken,
    terminal: Option<TerminalOutcome>,
    tasks: HashMap<TaskId, CancellationToken>,
}

/// Per-run cancellation registry (thread-safe).
#[derive(Debug, Default)]
pub struct ProcessRegistry {
    runs: Mutex<HashMap<RunId, RunEntry>>,
}

impl ProcessRegistry {
    /// Creates an empty registry.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Registers (or returns the existing) cancellation token for a run.
    pub fn register_run(&self, run_id: RunId) -> CancellationToken {
        let mut runs = self.lock();
        runs.entry(run_id)
            .or_insert_with(|| RunEntry {
                token: CancellationToken::new(),
                terminal: None,
                tasks: HashMap::new(),
            })
            .token
            .clone()
    }

    /// The run's token, when the run is registered.
    #[must_use]
    pub fn token_for(&self, run_id: RunId) -> Option<CancellationToken> {
        self.lock().get(&run_id).map(|entry| entry.token.clone())
    }

    /// Registers a task under a run (returns its child token).
    ///
    /// # Errors
    /// [`Refusal::RunNotFound`] when the run is not registered.
    pub fn register_task(&self, run_id: RunId, task_id: TaskId) -> Result<CancellationToken, ToolError> {
        let mut runs = self.lock();
        let entry = runs.get_mut(&run_id).ok_or_else(|| {
            ToolError::new(Refusal::RunNotFound, "run is not registered for cancellation")
        })?;
        if let Some(terminal) = &entry.terminal {
            return Err(ToolError::new(
                Refusal::RunTerminal,
                format!("run is already {} ", terminal.state.label()),
            ));
        }
        let token = entry.token.child_token();
        entry.tasks.insert(task_id, token.clone());
        Ok(token)
    }

    /// Cancels a run and all its tasks.
    ///
    /// # Errors
    /// [`Refusal::RunNotFound`] when the run is not registered;
    /// [`Refusal::RunTerminal`] when it already reached a terminal state
    /// (cancel is idempotent only for the call that records the terminal).
    pub fn cancel_run(&self, run_id: RunId) -> Result<TerminalOutcome, ToolError> {
        let mut runs = self.lock();
        let entry = runs.get_mut(&run_id).ok_or_else(|| {
            ToolError::new(Refusal::RunNotFound, "run is not registered for cancellation")
        })?;
        if entry.terminal.is_some() {
            return Err(ToolError::new(Refusal::RunTerminal, "run is already terminal"));
        }
        entry.token.cancel();
        for token in entry.tasks.values() {
            token.cancel();
        }
        let outcome = TerminalOutcome {
            state: TerminalState::Cancelled,
            newly_terminal: true,
            at: Timestamp::now(),
        };
        entry.terminal = Some(outcome.clone());
        Ok(outcome)
    }

    /// Kills one task.
    ///
    /// # Errors
    /// [`Refusal::RunNotFound`], [`Refusal::TaskNotFound`], or
    /// [`Refusal::TaskTerminal`] when the task was already killed or the run is
    /// terminal.
    pub fn kill_task(&self, run_id: RunId, task_id: TaskId) -> Result<TerminalOutcome, ToolError> {
        let mut runs = self.lock();
        let entry = runs.get_mut(&run_id).ok_or_else(|| {
            ToolError::new(Refusal::RunNotFound, "run is not registered")
        })?;
        if let Some(terminal) = &entry.terminal {
            return Err(ToolError::new(
                Refusal::RunTerminal,
                format!("run is already {}", terminal.state.label()),
            ));
        }
        let token = entry.tasks.get(&task_id).ok_or_else(|| {
            ToolError::new(Refusal::TaskNotFound, "task is not registered in this run")
        })?;
        if token.is_cancelled() {
            return Err(ToolError::new(
                Refusal::TaskTerminal,
                "task is already cancelled/killed",
            ));
        }
        token.cancel();
        Ok(TerminalOutcome {
            state: TerminalState::Killed,
            newly_terminal: false,
            at: Timestamp::now(),
        })
    }

    /// Whether the run's token has been cancelled.
    #[must_use]
    pub fn is_cancelled(&self, run_id: RunId) -> bool {
        self.lock()
            .get(&run_id)
            .map(|entry| entry.token.is_cancelled())
            .unwrap_or(false)
    }

    /// The terminal outcome for a run, if any.
    #[must_use]
    pub fn terminal(&self, run_id: RunId) -> Option<TerminalOutcome> {
        self.lock()
            .get(&run_id)
            .and_then(|entry| entry.terminal.clone())
    }

    /// Removes a finished run from the registry (inverse of `register_run`).
    pub fn unregister(&self, run_id: RunId) {
        self.lock().remove(&run_id);
    }

    /// Number of registered runs (diagnostics).
    #[must_use]
    pub fn active_runs(&self) -> usize {
        self.lock().len()
    }

    fn lock(&self) -> std::sync::MutexGuard<'_, HashMap<RunId, RunEntry>> {
        self.runs.lock().unwrap_or_else(|poisoned| poisoned.into_inner())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cancel_broadcasts_to_all_tasks_and_records_terminal() {
        let registry = ProcessRegistry::new();
        let run = RunId::new();
        registry.register_run(run);
        let task_a = registry.register_task(run, TaskId::new()).expect("task a");
        let task_b = registry.register_task(run, TaskId::new()).expect("task b");
        let outcome = registry.cancel_run(run).expect("cancelled");
        assert_eq!(outcome.state, TerminalState::Cancelled);
        assert!(outcome.newly_terminal);
        assert!(task_a.is_cancelled());
        assert!(task_b.is_cancelled());
        assert!(registry.is_cancelled(run));
        // A second cancel is refused by name, not silently repeated.
        assert_eq!(
            registry.cancel_run(run).expect_err("terminal").code(),
            "E_RUN_TERMINAL"
        );
    }

    #[test]
    fn kill_one_task_only_cancels_that_task() {
        let registry = ProcessRegistry::new();
        let run = RunId::new();
        registry.register_run(run);
        let task_a = TaskId::new();
        let task_b = TaskId::new();
        let token_a = registry.register_task(run, task_a).expect("task a");
        let token_b = registry.register_task(run, task_b).expect("task b");
        let outcome = registry.kill_task(run, task_a).expect("killed");
        assert_eq!(outcome.state, TerminalState::Killed);
        assert!(token_a.is_cancelled());
        assert!(!token_b.is_cancelled());
        // Killing again is terminal by name.
        assert_eq!(
            registry.kill_task(run, task_a).expect_err("terminal").code(),
            "E_TASK_TERMINAL"
        );
        assert_eq!(
            registry
                .kill_task(run, TaskId::new())
                .expect_err("not found")
                .code(),
            "E_TASK_NOT_FOUND"
        );
    }

    #[test]
    fn unregistered_run_and_task_refusals_are_named() {
        let registry = ProcessRegistry::new();
        let run = RunId::new();
        assert_eq!(
            registry.cancel_run(run).expect_err("run").code(),
            "E_RUN_NOT_FOUND"
        );
        assert_eq!(
            registry
                .register_task(run, TaskId::new())
                .expect_err("run")
                .code(),
            "E_RUN_NOT_FOUND"
        );
        registry.register_run(run);
        assert_eq!(
            registry
                .kill_task(run, TaskId::new())
                .expect_err("task")
                .code(),
            "E_TASK_NOT_FOUND"
        );
        assert_eq!(registry.active_runs(), 1);
        registry.unregister(run);
        assert_eq!(registry.active_runs(), 0);
        assert!(registry.token_for(run).is_none());
    }

    #[tokio::test]
    async fn tasks_can_select_on_the_token() {
        let registry = ProcessRegistry::new();
        let run = RunId::new();
        registry.register_run(run);
        let task = TaskId::new();
        let token = registry.register_task(run, task).expect("task");
        let waiter = token.clone();
        let handle = tokio::spawn(async move {
            waiter.cancelled().await;
            true
        });
        registry.cancel_run(run).expect("cancelled");
        assert!(handle.await.expect("join"), "child observed cancellation");
    }
}
