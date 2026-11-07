//! Pure run/task lifecycle state machine for the HiveMind 1.2 tool loop.
//!
//! This module is a **deterministic, side-effect-free authority** for the run and
//! task state graphs described by `plans/waves/W11-SUSPENSION-DESIGN.md` §4 and
//! required by `plans/waves/W12-INTERFACES.md`. It performs no I/O, holds no
//! locks, reads no clock and touches no database: the same `(from, event)` pair
//! always returns the same transition and the same single effect descriptor.
//! Integration code (`execution.rs`, Lead-owned) applies the descriptor exactly
//! once; this module never emits events itself.
//!
//! ## Phase 1 (logic-gatekeeper), written before the file was created
//!
//! ```json
//! {
//!   "phase1": {
//!     "architecture": "Run and task lifecycle states are the frozen contract AgentState values; the persisted run/task status columns and the GraphAuthority task nodes are the current owner, and plans/waves/W11-SUSPENSION-DESIGN.md §4 is the design authority for the 1.2 additions. This module is a new pure leaf: it owns only the legal-state sets, the legal edge table and the single-effect descriptors; it imports AgentState and Refusal from hiverouter-contract and nothing else from the crate, so execution.rs/db.rs/tool_loop.rs remain the only effect owners.",
//!     "dataFlow": "Untrusted input none. Caller supplies (current AgentState, StateEvent) from the engine after it observed a real transition trigger (submit complete fail cancel suspend resume deny timeout restart). The module validates the edge against the legal-state sets, rejects terminal-state and approval-replay applications by name, and returns a typed transition {from,event,to,effect} whose effect is the single thing the engine must persist/emit once; no effect list, no callbacks, no state mutation inside the module.",
//!     "edgeCases": [
//!       "awaiting_approval + complete is refused E_SCHEMA_INVALID (never a direct completion while consent is pending)",
//!       "awaiting_approval + fail is refused E_SCHEMA_INVALID; only Timeout/Cancel/Kill(restart) may leave a wait",
//!       "resume/deny/timeout applied when no approval is pending returns E_APPROVAL_REPLAY, so a duplicate resolution cannot produce a second effect",
//!       "submit applied to a non-creation state is refused E_SCHEMA_INVALID, never an implicit second run",
//!       "restart of a terminal run returns E_RUN_TERMINAL / E_TASK_TERMINAL and cannot resurrect a terminal run",
//!       "task kill in awaiting_approval maps to cancelled (W11 §4 kill row), not killed, and produces one cancel effect",
//!       "a UI-only AgentState (streaming, loading, replay, reconnecting) is refused E_SCHEMA_INVALID because it is not a persisted run/task lifecycle state"
//!     ],
//!     "rootCause": "The engine has no single authority for legal lifecycle edges: run status is written ad hoc at create/plan/terminal sites and the 1.2 states awaiting_approval/interrupted have no transition table, so a future tool loop could complete, fail or resume across a pending approval without one deterministic check; this module makes the edge table executable and fails closed on every illegal or replayed transition using only existing refusal codes."
//!   }
//! }
//! ```
//!
//! ## Single-effect rule
//!
//! Every successful application returns exactly one [`Effect`]. The engine maps
//! that descriptor to **one** persistence write and **one** externally visible
//! event chain (e.g. [`Effect::RequestApproval`] ⇒ approval row commit + run/
//! task status `awaiting_approval` + one `APPROVAL_REQUEST`; [`Effect::CancelRun`]
//! ⇒ status `cancelled` + one `RUN_STATUS`). No transition is a no-op: `from`
//! and `to` always differ, and the same event cannot be applied twice for one
//! wait (the second attempt is `E_APPROVAL_REPLAY`).
//!
//! ## Not owned here
//!
//! Approval nonce/TTL persistence, event publishing, database writes, counters,
//! caps and recovery are explicitly **not** modeled: this module only describes
//! the edge and the single effect the owners must perform.

use std::fmt;

use hiverouter_contract::{AgentState, Refusal};

/// Creation sentinel: the state *before* a run or task row exists.
///
/// It is deliberately **not** a persisted lifecycle state and is absent from
/// [`RUN_STATES`] / [`TASK_STATES`]; only `Submit` is accepted from it.
pub const BEFORE_RUN: AgentState = AgentState::Empty;

/// Legal persisted run lifecycle states (contract `AgentState` values).
///
/// `Killed`, `Streaming`, `Planning`-adjacent UI states etc. are intentionally
/// absent: a run is killed only through cancellation, and `streaming`/`loading`/
/// `replaying`/`reconnecting` are node/transport states, not run lifecycle
/// states (contract `state.rs`; `store.rs` active-run query).
pub const RUN_STATES: &[AgentState] = &[
    AgentState::Queued,
    AgentState::Planning,
    AgentState::Running,
    AgentState::AwaitingApproval,
    AgentState::Completed,
    AgentState::Failed,
    AgentState::Cancelled,
    AgentState::Interrupted,
];

/// Run states with no outgoing lifecycle edge (no resurrection without a new run).
pub const RUN_TERMINAL_STATES: &[AgentState] = &[
    AgentState::Completed,
    AgentState::Failed,
    AgentState::Cancelled,
    AgentState::Interrupted,
];

/// Legal persisted task lifecycle states.
///
/// A task adds `Killed` (explicit `KILL_WORKER`) and does not use `Planning`
/// (planning happens once per run, before task execution).
pub const TASK_STATES: &[AgentState] = &[
    AgentState::Queued,
    AgentState::Running,
    AgentState::AwaitingApproval,
    AgentState::Completed,
    AgentState::Failed,
    AgentState::Cancelled,
    AgentState::Killed,
    AgentState::Interrupted,
];

/// Task states with no outgoing lifecycle edge.
pub const TASK_TERMINAL_STATES: &[AgentState] = &[
    AgentState::Completed,
    AgentState::Failed,
    AgentState::Cancelled,
    AgentState::Killed,
    AgentState::Interrupted,
];

/// Every lifecycle event this module knows, in declaration order.
pub const ALL_EVENTS: &[StateEvent] = &[
    StateEvent::Submit,
    StateEvent::Plan,
    StateEvent::Start,
    StateEvent::Complete,
    StateEvent::Fail,
    StateEvent::Cancel,
    StateEvent::Kill,
    StateEvent::Suspend,
    StateEvent::Resume,
    StateEvent::Deny,
    StateEvent::Timeout,
    StateEvent::Restart,
];

/// A lifecycle trigger observed by the engine.
///
/// The nine required 1.2 triggers are `Submit`, `Complete`, `Fail`, `Cancel`,
/// `Suspend`, `Resume`, `Deny`, `Timeout` and `Restart`; `Plan`, `Start` and
/// `Kill` are the extra edges needed by the existing engine/contract
/// (`execution.rs` sets `planning`/`running`; `KILL_WORKER` produces `killed`)
/// and are reported as such — no other trigger is invented.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum StateEvent {
    /// A run/task is submitted (creation; only legal from [`BEFORE_RUN`]).
    Submit,
    /// The planner call starts (`queued -> planning`).
    Plan,
    /// Execution starts (`queued|planning -> running`).
    Start,
    /// The execution finished successfully.
    Complete,
    /// The execution failed with a named refusal (error code is engine-owned).
    Fail,
    /// The operator/user cancelled the run/task.
    Cancel,
    /// `KILL_WORKER` was accepted for a running task.
    Kill,
    /// A side-effect tool call needs consent (`running -> awaiting_approval`).
    Suspend,
    /// A fresh approval decision allows the call once (`awaiting_approval -> running`).
    Resume,
    /// The operator denied the call; the loop continues with a denied result.
    Deny,
    /// The approval TTL expired (CAS winner); the run fails by name.
    Timeout,
    /// Crash/restart recovery interrupted an active run/task.
    Restart,
}

impl StateEvent {
    /// Stable label used in errors/tests (not a wire value).
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            StateEvent::Submit => "submit",
            StateEvent::Plan => "plan",
            StateEvent::Start => "start",
            StateEvent::Complete => "complete",
            StateEvent::Fail => "fail",
            StateEvent::Cancel => "cancel",
            StateEvent::Kill => "kill",
            StateEvent::Suspend => "suspend",
            StateEvent::Resume => "resume",
            StateEvent::Deny => "deny",
            StateEvent::Timeout => "timeout",
            StateEvent::Restart => "restart",
        }
    }
}

/// Which lifecycle graph an error/transition belongs to.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Scope {
    /// Run lifecycle.
    Run,
    /// Task lifecycle.
    Task,
}

impl Scope {
    /// Refusal used when an event targets a terminal state in this scope.
    #[must_use]
    pub fn terminal_refusal(self) -> Refusal {
        match self {
            Scope::Run => Refusal::RunTerminal,
            Scope::Task => Refusal::TaskTerminal,
        }
    }
}

/// The single effect the integration owner must perform for a transition.
///
/// Each value is deliberately one observable action (one persistence write plus
/// its one event chain); see the module docs for the mapping rules. This enum is
/// data, not a callback: the module never performs any of these.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Effect {
    /// Create the run row (`queued`) and publish `RUN_CREATED`.
    CreateRun,
    /// Persist `planning` and publish `RUN_STATUS planning`.
    MarkPlanning,
    /// Persist `running` and publish `RUN_STATUS running`.
    StartRun,
    /// Persist `completed` and publish the terminal `RUN_STATUS completed`.
    MarkCompleted,
    /// Persist `failed` (with the engine-owned error code) and publish the terminal `RUN_STATUS failed`.
    MarkFailed,
    /// Persist `cancelled` and publish the terminal `RUN_STATUS cancelled`.
    CancelRun,
    /// Commit the approval row, persist `awaiting_approval`, publish one `APPROVAL_REQUEST` (+ `TOOL_CALL`).
    RequestApproval,
    /// Persist `running`, publish `APPROVAL_RESOLVED approve` once and execute the granted call once.
    ResumeFromApproval,
    /// Persist `running`, publish `APPROVAL_RESOLVED deny` once and append one denied tool result.
    FeedDeniedToolResult,
    /// Persist `failed` with error code `E_APPROVAL_TIMEOUT` and publish the terminal `RUN_STATUS`.
    FailOnApprovalTimeout,
    /// Persist `interrupted` and publish the terminal recovery `RUN_STATUS`.
    MarkInterrupted,
    /// Create the task row (`queued`) as part of the accepted plan.
    CreateTask,
    /// Persist task `running` and publish `NODE_STATE running`.
    StartTask,
    /// Persist task `completed` and publish `NODE_STATE completed`.
    MarkTaskCompleted,
    /// Persist task `failed` and publish `NODE_STATE failed`.
    MarkTaskFailed,
    /// Persist task `cancelled` and publish `NODE_STATE cancelled`.
    MarkTaskCancelled,
    /// Persist task `killed` and publish `NODE_STATE killed`.
    MarkTaskKilled,
    /// Commit the task approval row and publish one `APPROVAL_REQUEST`.
    RequestTaskApproval,
    /// Persist task `running` and continue the approved call once.
    ResumeTaskFromApproval,
    /// Persist task `running` and append one denied tool result.
    FeedTaskDeniedToolResult,
    /// Persist task `failed` with `E_APPROVAL_TIMEOUT` and publish the node state.
    FailTaskOnApprovalTimeout,
    /// Persist task `interrupted` and publish the terminal recovery `NODE_STATE`.
    MarkTaskInterrupted,
}

impl Effect {
    /// Refusal code the integration must attach to this effect, if any.
    ///
    /// Only the approval-timeout effects carry a contract error code; every
    /// other effect inherits the code from the engine's own failure path.
    #[must_use]
    pub fn refusal(self) -> Option<Refusal> {
        match self {
            Effect::FailOnApprovalTimeout | Effect::FailTaskOnApprovalTimeout => {
                Some(Refusal::ApprovalTimeout)
            }
            _ => None,
        }
    }
}

/// A validated run transition with exactly one effect.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RunTransition {
    /// State the run was in when the event was applied.
    pub from: AgentState,
    /// Event that caused the transition.
    pub event: StateEvent,
    /// State the run must be persisted in.
    pub to: AgentState,
    /// The single effect the owner must perform.
    pub effect: Effect,
}

/// A validated task transition with exactly one effect.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TaskTransition {
    /// State the task was in when the event was applied.
    pub from: AgentState,
    /// Event that caused the transition.
    pub event: StateEvent,
    /// State the task must be persisted in.
    pub to: AgentState,
    /// The single effect the owner must perform.
    pub effect: Effect,
}

/// Typed refusal for an illegal application.
///
/// Every variant maps to a **frozen contract refusal code** already present in
/// `hiverouter_contract::Refusal`; this module adds no new code:
/// terminal-state events → `E_RUN_TERMINAL` / `E_TASK_TERMINAL`, illegal edges →
/// `E_SCHEMA_INVALID`, approval decisions with no pending wait → `E_APPROVAL_REPLAY`,
/// and internal invariant violations → `E_INTERNAL_ERROR`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RunStateError {
    /// The event targets a terminal state (no edge exists, ever).
    Terminal {
        /// Which graph the state belongs to.
        scope: Scope,
        /// Current terminal state.
        state: AgentState,
        /// Event that was refused.
        event: StateEvent,
    },
    /// The event is not a legal edge from this non-terminal state.
    Illegal {
        /// Current state.
        from: AgentState,
        /// Event that was refused.
        event: StateEvent,
    },
    /// An approval decision/wait event was applied while no wait is pending
    /// (duplicate resume/deny/timeout, or a decision after the wait ended).
    ApprovalReplay {
        /// Current state.
        state: AgentState,
        /// Event that was refused.
        event: StateEvent,
    },
    /// Internal guard: an edge would not change state or would target a state
    /// outside the legal set (a table bug, never caller input).
    Invariant {
        /// State the edge started from.
        from: AgentState,
        /// State the edge would produce.
        to: AgentState,
    },
}

impl RunStateError {
    /// Frozen contract refusal this error maps to.
    #[must_use]
    pub fn refusal(self) -> Refusal {
        match self {
            RunStateError::Terminal { scope, .. } => scope.terminal_refusal(),
            RunStateError::Illegal { .. } => Refusal::SchemaInvalid,
            RunStateError::ApprovalReplay { .. } => Refusal::ApprovalReplay,
            RunStateError::Invariant { .. } => Refusal::InternalError,
        }
    }

    /// Stable refusal code (`E_*`, frozen contract vocabulary).
    #[must_use]
    pub fn code(self) -> &'static str {
        self.refusal().code()
    }
}

impl fmt::Display for RunStateError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RunStateError::Terminal {
                scope,
                state,
                event,
            } => write!(
                f,
                "{} ({:?}): {:?} is not legal from terminal state {}",
                self.code(),
                scope,
                event,
                state
            ),
            RunStateError::Illegal { from, event } => {
                write!(
                    f,
                    "{}: {:?} is not a legal edge from {}",
                    self.code(),
                    event,
                    from
                )
            }
            RunStateError::ApprovalReplay { state, event } => write!(
                f,
                "{}: {:?} applied while no approval wait is pending (state {})",
                self.code(),
                event,
                state
            ),
            RunStateError::Invariant { from, to } => write!(
                f,
                "{}: transition table invariant violated ({} -> {})",
                self.code(),
                from,
                to
            ),
        }
    }
}

impl std::error::Error for RunStateError {}

fn run_error_terminal(state: AgentState, event: StateEvent) -> RunStateError {
    RunStateError::Terminal {
        scope: Scope::Run,
        state,
        event,
    }
}

fn task_error_terminal(state: AgentState, event: StateEvent) -> RunStateError {
    RunStateError::Terminal {
        scope: Scope::Task,
        state,
        event,
    }
}

fn is_one_of(list: &[AgentState], state: AgentState) -> bool {
    list.iter().any(|candidate| *candidate == state)
}

/// True when `state` is a persisted run lifecycle state (not [`BEFORE_RUN`]).
#[must_use]
pub fn is_run_state(state: AgentState) -> bool {
    is_one_of(RUN_STATES, state)
}

/// True when `state` is a persisted task lifecycle state (not [`BEFORE_RUN`]).
#[must_use]
pub fn is_task_state(state: AgentState) -> bool {
    is_one_of(TASK_STATES, state)
}

/// True when a run in `state` has no outgoing lifecycle edge.
#[must_use]
pub fn is_run_terminal(state: AgentState) -> bool {
    is_one_of(RUN_TERMINAL_STATES, state)
}

/// True when a task in `state` has no outgoing lifecycle edge.
#[must_use]
pub fn is_task_terminal(state: AgentState) -> bool {
    is_one_of(TASK_TERMINAL_STATES, state)
}

/// Terminal-task predicate under the name the engine/store guards use.
///
/// This is the exact symbol the W1 remediation manifest names for the
/// terminal-state guard: [`crate::store::RunStore::update_task_state`] refuses to
/// move a `completed`/`failed`/`cancelled`/`killed`/`interrupted` task to a
/// different state, and the execution owner must apply the same predicate before
/// it publishes a completion (a late `REPLY_DONE` may not resurrect a
/// `KILL_WORKER`'d task). Defined once here so both guards share one set.
#[must_use]
pub fn is_terminal_task_state(state: AgentState) -> bool {
    is_task_terminal(state)
}

fn guard_run(from: AgentState, event: StateEvent) -> Result<(), RunStateError> {
    if from == BEFORE_RUN {
        // Creation sentinel: only Submit is allowed; handled by the caller.
        return Err(RunStateError::Illegal { from, event });
    }
    if is_run_terminal(from) {
        return Err(run_error_terminal(from, event));
    }
    if !is_run_state(from) {
        return Err(RunStateError::Illegal { from, event });
    }
    Ok(())
}

/// Applies one lifecycle event to a run and returns the single effect to perform.
///
/// # Errors
/// [`RunStateError`] with a frozen refusal code: terminal targets
/// (`E_RUN_TERMINAL`), illegal edges (`E_SCHEMA_INVALID`), approval decisions
/// with no pending wait (`E_APPROVAL_REPLAY`) and table invariants
/// (`E_INTERNAL_ERROR`).
pub fn apply_run(from: AgentState, event: StateEvent) -> Result<RunTransition, RunStateError> {
    if from == BEFORE_RUN {
        return match event {
            StateEvent::Submit => Ok(RunTransition {
                from,
                event,
                to: AgentState::Queued,
                effect: Effect::CreateRun,
            }),
            _ => Err(RunStateError::Illegal { from, event }),
        };
    }
    guard_run(from, event)?;

    let (to, effect) = match (from, event) {
        (AgentState::Queued, StateEvent::Plan) => (AgentState::Planning, Effect::MarkPlanning),
        (AgentState::Queued | AgentState::Planning, StateEvent::Start) => {
            (AgentState::Running, Effect::StartRun)
        }
        (AgentState::Running, StateEvent::Complete) => {
            (AgentState::Completed, Effect::MarkCompleted)
        }
        (AgentState::Queued | AgentState::Planning | AgentState::Running, StateEvent::Fail) => {
            (AgentState::Failed, Effect::MarkFailed)
        }
        (
            AgentState::Queued
            | AgentState::Planning
            | AgentState::Running
            | AgentState::AwaitingApproval,
            StateEvent::Cancel,
        ) => (AgentState::Cancelled, Effect::CancelRun),
        (AgentState::Running, StateEvent::Suspend) => {
            (AgentState::AwaitingApproval, Effect::RequestApproval)
        }
        (AgentState::AwaitingApproval, StateEvent::Resume) => {
            (AgentState::Running, Effect::ResumeFromApproval)
        }
        (AgentState::AwaitingApproval, StateEvent::Deny) => {
            (AgentState::Running, Effect::FeedDeniedToolResult)
        }
        (AgentState::AwaitingApproval, StateEvent::Timeout) => {
            (AgentState::Failed, Effect::FailOnApprovalTimeout)
        }
        (
            AgentState::Queued
            | AgentState::Planning
            | AgentState::Running
            | AgentState::AwaitingApproval,
            StateEvent::Restart,
        ) => (AgentState::Interrupted, Effect::MarkInterrupted),
        _ => {
            if matches!(
                event,
                StateEvent::Resume | StateEvent::Deny | StateEvent::Timeout
            ) {
                return Err(RunStateError::ApprovalReplay { state: from, event });
            }
            return Err(RunStateError::Illegal { from, event });
        }
    };
    let transition = RunTransition {
        from,
        event,
        to,
        effect,
    };
    if transition.from == transition.to || !is_run_state(transition.to) {
        return Err(RunStateError::Invariant { from, to });
    }
    // Single-effect rule: every accepted edge carries exactly one effect by type
    // construction; nothing else to validate here.
    Ok(transition)
}

/// Applies one lifecycle event to a task and returns the single effect to perform.
///
/// Task-specific rules (W11 §4): `kill` on a waiting task maps to `cancelled`
/// (the run-level `ToolCancelled` mapping), a task never enters `planning`, and
/// terminal tasks never transition.
///
/// # Errors
/// [`RunStateError`] with a frozen refusal code (`E_TASK_TERMINAL`,
/// `E_SCHEMA_INVALID`, `E_APPROVAL_REPLAY`, `E_INTERNAL_ERROR`).
pub fn apply_task(from: AgentState, event: StateEvent) -> Result<TaskTransition, RunStateError> {
    if from == BEFORE_RUN {
        return match event {
            StateEvent::Submit => Ok(TaskTransition {
                from,
                event,
                to: AgentState::Queued,
                effect: Effect::CreateTask,
            }),
            _ => Err(RunStateError::Illegal { from, event }),
        };
    }
    if is_task_terminal(from) {
        return Err(task_error_terminal(from, event));
    }
    if !is_task_state(from) {
        return Err(RunStateError::Illegal { from, event });
    }

    let (to, effect) = match (from, event) {
        (AgentState::Queued, StateEvent::Start) => (AgentState::Running, Effect::StartTask),
        (AgentState::Running, StateEvent::Complete) => {
            (AgentState::Completed, Effect::MarkTaskCompleted)
        }
        (AgentState::Queued | AgentState::Running, StateEvent::Fail) => {
            (AgentState::Failed, Effect::MarkTaskFailed)
        }
        (
            AgentState::Queued | AgentState::Running | AgentState::AwaitingApproval,
            StateEvent::Cancel,
        ) => (AgentState::Cancelled, Effect::MarkTaskCancelled),
        (AgentState::Running, StateEvent::Kill) => (AgentState::Killed, Effect::MarkTaskKilled),
        (AgentState::AwaitingApproval, StateEvent::Kill) => {
            (AgentState::Cancelled, Effect::MarkTaskCancelled)
        }
        (AgentState::Running, StateEvent::Suspend) => {
            (AgentState::AwaitingApproval, Effect::RequestTaskApproval)
        }
        (AgentState::AwaitingApproval, StateEvent::Resume) => {
            (AgentState::Running, Effect::ResumeTaskFromApproval)
        }
        (AgentState::AwaitingApproval, StateEvent::Deny) => {
            (AgentState::Running, Effect::FeedTaskDeniedToolResult)
        }
        (AgentState::AwaitingApproval, StateEvent::Timeout) => {
            (AgentState::Failed, Effect::FailTaskOnApprovalTimeout)
        }
        (
            AgentState::Queued | AgentState::Running | AgentState::AwaitingApproval,
            StateEvent::Restart,
        ) => (AgentState::Interrupted, Effect::MarkTaskInterrupted),
        _ => {
            if matches!(
                event,
                StateEvent::Resume | StateEvent::Deny | StateEvent::Timeout
            ) {
                return Err(RunStateError::ApprovalReplay { state: from, event });
            }
            return Err(RunStateError::Illegal { from, event });
        }
    };
    let transition = TaskTransition {
        from,
        event,
        to,
        effect,
    };
    if transition.from == transition.to || !is_task_state(transition.to) {
        return Err(RunStateError::Invariant { from, to });
    }
    Ok(transition)
}

/// Whether [`apply_run`] would accept `(from, event)`.
#[must_use]
pub fn can_apply_run(from: AgentState, event: StateEvent) -> bool {
    apply_run(from, event).is_ok()
}

/// Whether [`apply_task`] would accept `(from, event)`.
#[must_use]
pub fn can_apply_task(from: AgentState, event: StateEvent) -> bool {
    apply_task(from, event).is_ok()
}

/// Every run event legal from `state`, in [`ALL_EVENTS`] declaration order.
#[must_use]
pub fn legal_events_run(state: AgentState) -> Vec<StateEvent> {
    ALL_EVENTS
        .iter()
        .copied()
        .filter(|event| can_apply_run(state, *event))
        .collect()
}

/// Every task event legal from `state`, in [`ALL_EVENTS`] declaration order.
#[must_use]
pub fn legal_events_task(state: AgentState) -> Vec<StateEvent> {
    ALL_EVENTS
        .iter()
        .copied()
        .filter(|event| can_apply_task(state, *event))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run_apply(from: AgentState, event: StateEvent) -> Result<RunTransition, RunStateError> {
        apply_run(from, event)
    }

    fn task_apply(from: AgentState, event: StateEvent) -> Result<TaskTransition, RunStateError> {
        apply_task(from, event)
    }

    #[test]
    fn state_sets_are_contract_states() {
        for state in RUN_STATES.iter().chain(TASK_STATES.iter()) {
            assert!(
                AgentState::ALL.contains(state),
                "{state} must be a contract AgentState"
            );
        }
        assert!(RUN_STATES.contains(&AgentState::AwaitingApproval));
        assert!(TASK_STATES.contains(&AgentState::AwaitingApproval));
        assert!(!RUN_STATES.contains(&BEFORE_RUN));
        assert!(!TASK_STATES.contains(&BEFORE_RUN));
        for state in RUN_TERMINAL_STATES {
            assert!(RUN_STATES.contains(state));
        }
        for state in TASK_TERMINAL_STATES {
            assert!(TASK_STATES.contains(state));
        }
        assert!(!RUN_TERMINAL_STATES.contains(&AgentState::AwaitingApproval));
        assert!(!TASK_TERMINAL_STATES.contains(&AgentState::AwaitingApproval));
        // Contract-owned terminality agrees for every terminal state.
        assert!(AgentState::Killed.is_terminal());
        assert!(AgentState::Interrupted.is_terminal());
    }

    #[test]
    fn terminal_task_predicate_matches_the_terminal_set_only() {
        for state in TASK_STATES {
            assert_eq!(
                is_terminal_task_state(*state),
                TASK_TERMINAL_STATES.contains(state),
                "is_terminal_task_state disagrees with TASK_TERMINAL_STATES for {state}"
            );
            assert_eq!(is_terminal_task_state(*state), is_task_terminal(*state));
        }
        for state in [
            AgentState::Queued,
            AgentState::Running,
            AgentState::AwaitingApproval,
            BEFORE_RUN,
        ] {
            assert!(
                !is_terminal_task_state(state),
                "{state} must stay settlable by the engine"
            );
        }
    }

    #[test]
    fn submit_creates_once_and_only_from_creation_sentinel() {
        let created = run_apply(BEFORE_RUN, StateEvent::Submit).expect("submit");
        assert_eq!(created.to, AgentState::Queued);
        assert_eq!(created.effect, Effect::CreateRun);
        assert_eq!(
            run_apply(AgentState::Queued, StateEvent::Submit).expect_err("double submit"),
            RunStateError::Illegal {
                from: AgentState::Queued,
                event: StateEvent::Submit
            }
        );
        assert_eq!(
            run_apply(BEFORE_RUN, StateEvent::Start).expect_err("start before submit"),
            RunStateError::Illegal {
                from: BEFORE_RUN,
                event: StateEvent::Start
            }
        );
    }

    #[test]
    fn planning_and_start_edges() {
        assert_eq!(
            run_apply(AgentState::Queued, StateEvent::Plan).expect("plan"),
            RunTransition {
                from: AgentState::Queued,
                event: StateEvent::Plan,
                to: AgentState::Planning,
                effect: Effect::MarkPlanning
            }
        );
        assert_eq!(
            run_apply(AgentState::Planning, StateEvent::Start).expect("start"),
            RunTransition {
                from: AgentState::Planning,
                event: StateEvent::Start,
                to: AgentState::Running,
                effect: Effect::StartRun
            }
        );
        assert_eq!(
            run_apply(AgentState::Queued, StateEvent::Start).expect("direct start"),
            RunTransition {
                from: AgentState::Queued,
                event: StateEvent::Start,
                to: AgentState::Running,
                effect: Effect::StartRun
            }
        );
        // A second Start is not legal.
        assert_eq!(
            run_apply(AgentState::Running, StateEvent::Start).expect_err("double start"),
            RunStateError::Illegal {
                from: AgentState::Running,
                event: StateEvent::Start
            }
        );
    }

    #[test]
    fn complete_only_from_running() {
        assert_eq!(
            run_apply(AgentState::Running, StateEvent::Complete).expect("complete"),
            RunTransition {
                from: AgentState::Running,
                event: StateEvent::Complete,
                to: AgentState::Completed,
                effect: Effect::MarkCompleted
            }
        );
        assert_eq!(
            run_apply(AgentState::Planning, StateEvent::Complete).expect_err("complete planning"),
            RunStateError::Illegal {
                from: AgentState::Planning,
                event: StateEvent::Complete
            }
        );
        assert_eq!(
            run_apply(AgentState::AwaitingApproval, StateEvent::Complete)
                .expect_err("complete pending approval"),
            RunStateError::Illegal {
                from: AgentState::AwaitingApproval,
                event: StateEvent::Complete
            }
        );
    }

    #[test]
    fn fail_only_from_active_execution() {
        assert_eq!(
            run_apply(AgentState::Running, StateEvent::Fail)
                .expect("fail")
                .to,
            AgentState::Failed
        );
        assert_eq!(
            run_apply(AgentState::Planning, StateEvent::Fail)
                .expect("plan fail")
                .to,
            AgentState::Failed
        );
        assert_eq!(
            run_apply(AgentState::Queued, StateEvent::Fail)
                .expect("queued fail")
                .to,
            AgentState::Failed
        );
        assert_eq!(
            run_apply(AgentState::AwaitingApproval, StateEvent::Fail)
                .expect_err("fail during wait is timeout/cancel only"),
            RunStateError::Illegal {
                from: AgentState::AwaitingApproval,
                event: StateEvent::Fail
            }
        );
    }

    #[test]
    fn suspend_is_the_only_edge_into_awaiting_approval() {
        assert_eq!(
            run_apply(AgentState::Running, StateEvent::Suspend).expect("suspend"),
            RunTransition {
                from: AgentState::Running,
                event: StateEvent::Suspend,
                to: AgentState::AwaitingApproval,
                effect: Effect::RequestApproval
            }
        );
        for from in [
            AgentState::Queued,
            AgentState::Planning,
            AgentState::AwaitingApproval,
        ] {
            assert_eq!(
                run_apply(from, StateEvent::Suspend).expect_err("illegal suspend"),
                RunStateError::Illegal {
                    from,
                    event: StateEvent::Suspend
                }
            );
        }
    }

    #[test]
    fn resume_and_deny_leave_the_wait_with_distinct_effects() {
        let resumed = run_apply(AgentState::AwaitingApproval, StateEvent::Resume).expect("resume");
        assert_eq!(resumed.to, AgentState::Running);
        assert_eq!(resumed.effect, Effect::ResumeFromApproval);
        let denied = run_apply(AgentState::AwaitingApproval, StateEvent::Deny).expect("deny");
        assert_eq!(denied.to, AgentState::Running);
        assert_eq!(denied.effect, Effect::FeedDeniedToolResult);
        assert_ne!(resumed.effect, denied.effect);
        // Single-use: the same wait cannot be resolved twice.
        assert_eq!(
            run_apply(resumed.to, StateEvent::Resume).expect_err("double resume"),
            RunStateError::ApprovalReplay {
                state: AgentState::Running,
                event: StateEvent::Resume
            }
        );
        assert_eq!(
            run_apply(denied.to, StateEvent::Deny).expect_err("double deny"),
            RunStateError::ApprovalReplay {
                state: AgentState::Running,
                event: StateEvent::Deny
            }
        );
    }

    #[test]
    fn timeout_fails_with_the_frozen_approval_timeout_code() {
        let timed_out =
            run_apply(AgentState::AwaitingApproval, StateEvent::Timeout).expect("timeout");
        assert_eq!(timed_out.to, AgentState::Failed);
        assert_eq!(timed_out.effect, Effect::FailOnApprovalTimeout);
        assert_eq!(
            Effect::FailOnApprovalTimeout.refusal(),
            Some(Refusal::ApprovalTimeout)
        );
        assert_eq!(
            run_apply(AgentState::Running, StateEvent::Timeout).expect_err("late timeout"),
            RunStateError::ApprovalReplay {
                state: AgentState::Running,
                event: StateEvent::Timeout
            }
        );
    }

    #[test]
    fn cancel_is_legal_from_every_active_state_and_terminal_otherwise() {
        for from in [
            AgentState::Queued,
            AgentState::Planning,
            AgentState::Running,
            AgentState::AwaitingApproval,
        ] {
            let cancelled = run_apply(from, StateEvent::Cancel).expect("cancel");
            assert_eq!(cancelled.to, AgentState::Cancelled);
            assert_eq!(cancelled.effect, Effect::CancelRun);
        }
        for from in RUN_TERMINAL_STATES {
            assert_eq!(
                run_apply(*from, StateEvent::Cancel).expect_err("terminal cancel"),
                RunStateError::Terminal {
                    scope: Scope::Run,
                    state: *from,
                    event: StateEvent::Cancel
                }
            );
        }
    }

    #[test]
    fn restart_marks_active_states_interrupted_and_never_resurrects() {
        for from in [
            AgentState::Queued,
            AgentState::Planning,
            AgentState::Running,
            AgentState::AwaitingApproval,
        ] {
            let restarted = run_apply(from, StateEvent::Restart).expect("restart");
            assert_eq!(restarted.to, AgentState::Interrupted);
            assert_eq!(restarted.effect, Effect::MarkInterrupted);
        }
        for from in RUN_TERMINAL_STATES {
            assert_eq!(
                run_apply(*from, StateEvent::Restart).expect_err("no resurrection"),
                RunStateError::Terminal {
                    scope: Scope::Run,
                    state: *from,
                    event: StateEvent::Restart
                }
            );
        }
    }

    #[test]
    fn terminal_run_states_have_no_outgoing_edges() {
        for from in RUN_TERMINAL_STATES {
            for event in ALL_EVENTS {
                let error = run_apply(*from, *event).expect_err("terminal event");
                assert_eq!(
                    error,
                    RunStateError::Terminal {
                        scope: Scope::Run,
                        state: *from,
                        event: *event
                    }
                );
                assert_eq!(error.code(), "E_RUN_TERMINAL");
            }
        }
    }

    #[test]
    fn task_machine_covers_lifecycle_and_kill_rules() {
        assert_eq!(
            task_apply(BEFORE_RUN, StateEvent::Submit)
                .expect("create task")
                .effect,
            Effect::CreateTask
        );
        assert_eq!(
            task_apply(AgentState::Queued, StateEvent::Start)
                .expect("start")
                .effect,
            Effect::StartTask
        );
        assert_eq!(
            task_apply(AgentState::Running, StateEvent::Complete)
                .expect("complete")
                .to,
            AgentState::Completed
        );
        assert_eq!(
            task_apply(AgentState::Running, StateEvent::Fail)
                .expect("fail")
                .to,
            AgentState::Failed
        );
        assert_eq!(
            task_apply(AgentState::Running, StateEvent::Kill)
                .expect("kill")
                .to,
            AgentState::Killed
        );
        // W11 §4 kill row: killing the waiting task cancels instead of killing.
        let waiting_kill =
            task_apply(AgentState::AwaitingApproval, StateEvent::Kill).expect("waiting kill");
        assert_eq!(waiting_kill.to, AgentState::Cancelled);
        assert_eq!(waiting_kill.effect, Effect::MarkTaskCancelled);
        assert_eq!(
            task_apply(AgentState::Running, StateEvent::Suspend)
                .expect("suspend")
                .to,
            AgentState::AwaitingApproval
        );
        assert_eq!(
            task_apply(AgentState::AwaitingApproval, StateEvent::Resume)
                .expect("resume")
                .to,
            AgentState::Running
        );
        assert_eq!(
            task_apply(AgentState::AwaitingApproval, StateEvent::Deny)
                .expect("deny")
                .effect,
            Effect::FeedTaskDeniedToolResult
        );
        let timeout =
            task_apply(AgentState::AwaitingApproval, StateEvent::Timeout).expect("timeout");
        assert_eq!(timeout.to, AgentState::Failed);
        assert_eq!(timeout.effect.refusal(), Some(Refusal::ApprovalTimeout));
        assert_eq!(
            task_apply(AgentState::Queued, StateEvent::Cancel)
                .expect("cancel")
                .to,
            AgentState::Cancelled
        );
        assert_eq!(
            task_apply(AgentState::Running, StateEvent::Restart)
                .expect("restart")
                .to,
            AgentState::Interrupted
        );
    }

    #[test]
    fn task_rejects_planning_only_and_run_rejects_kill() {
        assert_eq!(
            task_apply(AgentState::Queued, StateEvent::Plan).expect_err("task plan"),
            RunStateError::Illegal {
                from: AgentState::Queued,
                event: StateEvent::Plan
            }
        );
        assert_eq!(
            run_apply(AgentState::Running, StateEvent::Kill).expect_err("run kill"),
            RunStateError::Illegal {
                from: AgentState::Running,
                event: StateEvent::Kill
            }
        );
        assert_eq!(
            task_apply(AgentState::AwaitingApproval, StateEvent::Fail)
                .expect_err("fail while waiting is timeout/cancel/kill only"),
            RunStateError::Illegal {
                from: AgentState::AwaitingApproval,
                event: StateEvent::Fail
            }
        );
    }

    #[test]
    fn ui_only_states_are_not_lifecycle_states() {
        for state in [
            AgentState::Streaming,
            AgentState::Thinking,
            AgentState::Replay,
            AgentState::Reconnecting,
            AgentState::Loading,
            AgentState::Healthy,
        ] {
            assert!(!is_run_state(state));
            assert!(!is_task_state(state));
            assert_eq!(
                run_apply(state, StateEvent::Start).expect_err("ui state as run"),
                RunStateError::Illegal {
                    from: state,
                    event: StateEvent::Start
                }
            );
        }
    }

    #[test]
    fn every_error_maps_to_an_existing_frozen_refusal_code() {
        let run_terminal = RunStateError::Terminal {
            scope: Scope::Run,
            state: AgentState::Completed,
            event: StateEvent::Cancel,
        };
        let task_terminal = RunStateError::Terminal {
            scope: Scope::Task,
            state: AgentState::Killed,
            event: StateEvent::Cancel,
        };
        assert_eq!(run_terminal.refusal(), Refusal::RunTerminal);
        assert_eq!(task_terminal.refusal(), Refusal::TaskTerminal);
        assert_eq!(
            RunStateError::Illegal {
                from: AgentState::Running,
                event: StateEvent::Suspend
            }
            .refusal(),
            Refusal::SchemaInvalid
        );
        assert_eq!(
            RunStateError::ApprovalReplay {
                state: AgentState::Running,
                event: StateEvent::Resume
            }
            .refusal(),
            Refusal::ApprovalReplay
        );
        assert_eq!(
            RunStateError::Invariant {
                from: AgentState::Running,
                to: AgentState::Running
            }
            .refusal(),
            Refusal::InternalError
        );
        // The frozen contract exposes every code this module can return.
        assert_eq!(Refusal::RunTerminal.code(), "E_RUN_TERMINAL");
        assert_eq!(Refusal::TaskTerminal.code(), "E_TASK_TERMINAL");
        assert_eq!(Refusal::SchemaInvalid.code(), "E_SCHEMA_INVALID");
        assert_eq!(Refusal::ApprovalReplay.code(), "E_APPROVAL_REPLAY");
        assert_eq!(Refusal::ApprovalTimeout.code(), "E_APPROVAL_TIMEOUT");
        assert_eq!(Refusal::InternalError.code(), "E_INTERNAL_ERROR");
    }

    #[test]
    fn every_accepted_edge_is_single_effect_and_changes_state() {
        for from in RUN_STATES {
            for event in ALL_EVENTS {
                if let Ok(transition) = apply_run(*from, *event) {
                    assert_ne!(transition.from, transition.to, "no-op edge refused");
                    assert!(is_run_state(transition.to));
                    // The transition type carries exactly one effect by
                    // construction (one `Effect` field, no list); the owner must
                    // perform it exactly once. Timeout effects must name the
                    // frozen timeout code so the engine cannot invent one.
                    if transition.to == AgentState::Failed && matches!(event, StateEvent::Timeout) {
                        assert_eq!(transition.effect.refusal(), Some(Refusal::ApprovalTimeout));
                    }
                }
            }
        }
        for from in TASK_STATES {
            for event in ALL_EVENTS {
                if let Ok(transition) = apply_task(*from, *event) {
                    assert_ne!(transition.from, transition.to, "no-op edge refused");
                    assert!(is_task_state(transition.to));
                }
            }
        }
    }

    #[test]
    fn can_apply_and_legal_events_match_apply() {
        for from in RUN_STATES.iter().chain(std::iter::once(&BEFORE_RUN)) {
            for event in ALL_EVENTS {
                assert_eq!(
                    can_apply_run(*from, *event),
                    apply_run(*from, *event).is_ok(),
                    "can_apply_run disagrees with apply_run for {from}/{event:?}"
                );
            }
        }
        for from in TASK_STATES.iter().chain(std::iter::once(&BEFORE_RUN)) {
            for event in ALL_EVENTS {
                assert_eq!(
                    can_apply_task(*from, *event),
                    apply_task(*from, *event).is_ok(),
                    "can_apply_task disagrees with apply_task for {from}/{event:?}"
                );
            }
        }
        assert_eq!(
            legal_events_run(AgentState::AwaitingApproval),
            vec![
                StateEvent::Cancel,
                StateEvent::Resume,
                StateEvent::Deny,
                StateEvent::Timeout,
                StateEvent::Restart
            ]
        );
        assert_eq!(
            legal_events_run(AgentState::Completed),
            Vec::<StateEvent>::new()
        );
        assert_eq!(
            legal_events_task(AgentState::AwaitingApproval),
            vec![
                StateEvent::Cancel,
                StateEvent::Kill,
                StateEvent::Resume,
                StateEvent::Deny,
                StateEvent::Timeout,
                StateEvent::Restart
            ]
        );
    }

    #[test]
    fn transitions_are_deterministic() {
        for from in RUN_STATES {
            for event in ALL_EVENTS {
                let first = apply_run(*from, *event);
                for _ in 0..16 {
                    assert_eq!(first, apply_run(*from, *event));
                }
            }
        }
        for from in TASK_STATES {
            for event in ALL_EVENTS {
                let first = apply_task(*from, *event);
                for _ in 0..16 {
                    assert_eq!(first, apply_task(*from, *event));
                }
            }
        }
    }

    #[test]
    fn display_never_panics_and_names_the_code() {
        let errors = [
            RunStateError::Terminal {
                scope: Scope::Task,
                state: AgentState::Killed,
                event: StateEvent::Cancel,
            },
            RunStateError::Illegal {
                from: AgentState::Queued,
                event: StateEvent::Deny,
            },
            RunStateError::ApprovalReplay {
                state: AgentState::Running,
                event: StateEvent::Deny,
            },
            RunStateError::Invariant {
                from: AgentState::Running,
                to: AgentState::Running,
            },
        ];
        for error in errors {
            let rendered = error.to_string();
            assert!(
                rendered.starts_with(error.code()),
                "display includes the frozen code"
            );
        }
    }
}
