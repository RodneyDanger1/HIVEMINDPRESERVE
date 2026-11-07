//! Shared fixtures for the contract acceptance tests.
//!
//! These are test-only constructors; production code never fabricates frames.

#![allow(dead_code)]

use hiverouter_contract::command::{
    ApprovalResolve, CancelRun, ChatPin, Command, CommandName, FileRead, FileTreeList,
    HistoryRevisionSet, KillWorker, RewireTask, SettingsStatus, SpawnSubHive, SubmitPrompt,
    TemplateInsert, ViewportRecenter, ViewportSet,
};
use hiverouter_contract::event::{
    Ack, ApprovalRequest, ApprovalResolved, EmptyPayload, ErrorPayload, Event, FileContent,
    FileTreeEntry, FileTreeResult, HistorySnapshot, NodeEvent, ReplyDelta, ReplyDone,
    ResyncRequiredPayload, RunCreated, RunMetrics, RunStatusPayload, ServerHello,
    SettingsStatusResult, ToolCallPayload, ToolOutcome, ToolResultPayload,
};
use hiverouter_contract::ids::{ApprovalId, RunId, TaskId};
use hiverouter_contract::state::AgentState;
use hiverouter_contract::usage::Usage;
use hiverouter_contract::{Refusal, Timestamp};

/// One representative value for every frozen command (the decision-gated names
/// are covered by their refusal samples instead).
pub fn command_samples() -> Vec<Command> {
    vec![
        Command::SubmitPrompt(SubmitPrompt {
            text: "Build a honeycomb graph for the run".to_owned(),
            project_id: "project-1".to_owned(),
        }),
        Command::SpawnSubHive(SpawnSubHive {
            parent_task_id: TaskId::new(),
            role: "worker".to_owned(),
        }),
        Command::RewireTask(RewireTask {
            task_id: TaskId::new(),
            new_parent_id: TaskId::new(),
            expected_rev: 3,
        }),
        Command::CancelRun(CancelRun {
            run_id: RunId::new(),
        }),
        Command::KillWorker(KillWorker {
            task_id: TaskId::new(),
        }),
        Command::ChatPin(ChatPin { pinned: true }),
        Command::TemplateInsert(TemplateInsert {
            template_id: "template-1".to_owned(),
            project_id: "project-1".to_owned(),
        }),
        Command::HistoryRevisionSet(HistoryRevisionSet {
            run_id: RunId::new(),
            revision: 12,
        }),
        Command::ViewportSet(ViewportSet {
            x: 120.5,
            y: -40.0,
            zoom: 1.25,
        }),
        Command::ViewportRecenter(ViewportRecenter {}),
        Command::ApprovalResolve(ApprovalResolve {
            approval_id: ApprovalId::new(),
            nonce: "nonce-1".to_owned(),
            decision: "approve".to_owned(),
            reason: Some("operator approved".to_owned()),
        }),
        Command::SettingsStatus(SettingsStatus {}),
        Command::FileTreeList(FileTreeList {}),
        Command::FileRead(FileRead {
            path: "src/main.rs".to_owned(),
        }),
    ]
}

/// One representative value for every event name.
pub fn event_samples() -> Vec<Event> {
    let run_id = RunId::new();
    let task_id = TaskId::new();
    let approval_id = ApprovalId::new();
    vec![
        Event::Ack(Ack {
            command: CommandName::SubmitPrompt,
            correlation_id: uuid::Uuid::now_v7(),
        }),
        Event::Error(ErrorPayload::from_refusal(&Refusal::Cycle)),
        Event::ServerHello(ServerHello {
            last_seq: Some(41),
            schema: hiverouter_contract::SCHEMA_VERSION.to_owned(),
        }),
        Event::ResyncRequired(ResyncRequiredPayload {
            expected_seq: 10,
            last_seq: 3,
        }),
        Event::RunCreated(RunCreated {
            run_id,
            status: AgentState::Queued,
        }),
        Event::RunStatus(RunStatusPayload {
            run_id,
            status: AgentState::Failed,
            error_code: Some(Refusal::Cycle.code().to_owned()),
        }),
        Event::ReplyDelta(ReplyDelta {
            run_id,
            text_delta: "partial reply".to_owned(),
        }),
        Event::ReplyDone(ReplyDone {
            run_id,
            usage: Usage {
                prompt_tokens: Some(10),
                completion_tokens: Some(20),
                total_tokens: Some(30),
                prompt_cache_hit_tokens: None,
                prompt_cache_miss_tokens: Some(10),
                reasoning_tokens: Some(5),
            },
        }),
        Event::NodeSpawn(NodeEvent {
            task_id,
            state: AgentState::Queued,
            rev: None,
            kind: None,
            parent_task_id: None,
        }),
        Event::NodeDrag(NodeEvent {
            task_id,
            state: AgentState::Running,
            rev: None,
            kind: None,
            parent_task_id: None,
        }),
        Event::NodeRewire(NodeEvent {
            task_id,
            state: AgentState::Running,
            rev: None,
            kind: None,
            parent_task_id: None,
        }),
        Event::NodeState(NodeEvent {
            task_id,
            state: AgentState::Completed,
            rev: None,
            kind: None,
            parent_task_id: None,
        }),
        Event::RunMetrics(RunMetrics {
            superagents: 2,
            workers: 8,
            green_pct: 62.5,
            delta: -1.5,
        }),
        Event::ApprovalRequest(ApprovalRequest {
            approval_id,
            tool: "file_write".to_owned(),
            decision: None,
            // 1.2 additive fields: a model-originated approval carries the
            // nonce/args binding that APPROVAL_RESOLVE requires.
            nonce: Some("nonce-1".to_owned()),
            run_id: Some(run_id),
            task_id: Some(task_id),
            tool_call_id: Some("call_01".to_owned()),
            args_sha256: Some(
                "9f86d081884c7d659a2feaa0c55ad015a3bf4f1b2b0b822cd15d6c15b0f00a08".to_owned(),
            ),
            args_preview: None,
            expires_at: Some(Timestamp::parse("2026-11-06T09:20:00Z").expect("valid expiry")),
        }),
        Event::ApprovalResolved(ApprovalResolved {
            approval_id,
            tool: "shell.run".to_owned(),
            decision: "approved".to_owned(),
        }),
        Event::SettingsStatusResult(SettingsStatusResult {
            provider: "deepseek".to_owned(),
            model: "deepseek-flash".to_owned(),
            key_present: false,
            thinking: None,
            reasoning_effort: None,
        }),
        Event::FileTreeResult(FileTreeResult {
            root: ".".to_owned(),
            entries: vec![
                FileTreeEntry {
                    path: "src".to_owned(),
                    name: "src".to_owned(),
                    is_dir: true,
                },
                FileTreeEntry {
                    path: "README.md".to_owned(),
                    name: "README.md".to_owned(),
                    is_dir: false,
                },
            ],
        }),
        Event::FileContent(FileContent {
            path: "README.md".to_owned(),
            sha256: "9f86d081884c7d659a2feaa0c55ad015a3bf4f1b2b0b822cd15d6c15b0f00a08".to_owned(),
            bytes: 12,
            text: "hello hive\n".to_owned(),
        }),
        Event::HistorySnapshot(HistorySnapshot {
            run_id,
            revision: 12,
            graph: serde_json::json!({"nodes": [], "edges": []}),
        }),
        Event::Ping(EmptyPayload {}),
        Event::Pong(EmptyPayload {}),
        Event::ToolCall(ToolCallPayload {
            run_id,
            task_id,
            tool_call_id: "call_01".to_owned(),
            tool: "file_read".to_owned(),
            args_sha256: "9f86d081884c7d659a2feaa0c55ad015a3bf4f1b2b0b822cd15d6c15b0f00a08".to_owned(),
            turn: 1,
        }),
        Event::ToolResult(ToolResultPayload {
            run_id,
            task_id,
            tool_call_id: "call_01".to_owned(),
            outcome: ToolOutcome::Refused,
            code: Some(Refusal::ToolNotAllowed.code().to_owned()),
            bytes: Some(0),
            sha256: None,
        }),
    ]
}

/// A valid command envelope as a JSON value (fixed ids, fixed timestamp), used
/// to prove the decoders accept the frozen wire shape.
pub fn frozen_command_envelope() -> serde_json::Value {
    serde_json::json!({
        "schema": "hiverouter.v1",
        "type": "COMMAND",
        "name": "SUBMIT_PROMPT",
        "id": "019a0d5c-1111-7abc-8def-0123456789ab",
        "correlation_id": "019a0d5c-1111-7abc-8def-0123456789ac",
        "run_id": null,
        "task_id": null,
        "seq": null,
        "ts": "2026-11-06T09:15:41Z",
        "payload": {"text": "hello hive", "project_id": "project-1"}
    })
}

fn command_envelope(name: &str, payload: serde_json::Value) -> serde_json::Value {
    serde_json::json!({
        "schema": "hiverouter.v1",
        "type": "COMMAND",
        "name": name,
        "id": "019a0d5c-2222-7abc-8def-0123456789ab",
        "correlation_id": "019a0d5c-2222-7abc-8def-0123456789ab",
        "run_id": null,
        "task_id": null,
        "seq": null,
        "ts": "2026-11-06T09:15:41Z",
        "payload": payload,
    })
}

/// Every command name (except the decision-gated names) paired with an envelope
/// that must be refused by name.
pub fn refusal_samples() -> Vec<(CommandName, &'static str, serde_json::Value, Refusal)> {
    vec![
        (
            CommandName::SubmitPrompt,
            "SUBMIT_PROMPT empty text",
            command_envelope("SUBMIT_PROMPT", serde_json::json!({"text": "", "project_id": "p"})),
            Refusal::PromptEmpty,
        ),
        (
            CommandName::SubmitPrompt,
            "SUBMIT_PROMPT too long",
            command_envelope(
                "SUBMIT_PROMPT",
                serde_json::json!({"text": "x".repeat(8001), "project_id": "p"}),
            ),
            Refusal::PromptTooLong,
        ),
        (
            CommandName::SubmitPrompt,
            "SUBMIT_PROMPT missing field",
            command_envelope("SUBMIT_PROMPT", serde_json::json!({"text": "hi"})),
            Refusal::SchemaInvalid,
        ),
        (
            CommandName::SpawnSubHive,
            "SPAWN_SUB_HIVE bad uuid",
            command_envelope(
                "SPAWN_SUB_HIVE",
                serde_json::json!({"parent_task_id": "not-a-uuid", "role": "worker"}),
            ),
            Refusal::SchemaInvalid,
        ),
        (
            CommandName::RewireTask,
            "REWIRE_TASK missing expected_rev",
            command_envelope(
                "REWIRE_TASK",
                serde_json::json!({
                    "task_id": "019a0d5c-3333-7abc-8def-0123456789ab",
                    "new_parent_id": "019a0d5c-3333-7abc-8def-0123456789ac"
                }),
            ),
            Refusal::SchemaInvalid,
        ),
        (
            CommandName::CancelRun,
            "CANCEL_RUN bad run id",
            command_envelope("CANCEL_RUN", serde_json::json!({"run_id": "nope"})),
            Refusal::SchemaInvalid,
        ),
        (
            CommandName::KillWorker,
            "KILL_WORKER wrong type",
            command_envelope("KILL_WORKER", serde_json::json!({"task_id": 7})),
            Refusal::SchemaInvalid,
        ),
        (
            CommandName::ChatPin,
            "CHAT_PIN wrong type",
            command_envelope("CHAT_PIN", serde_json::json!({"pinned": "yes"})),
            Refusal::SchemaInvalid,
        ),
        (
            CommandName::TemplateInsert,
            "TEMPLATE_INSERT missing template",
            command_envelope("TEMPLATE_INSERT", serde_json::json!({"project_id": "p"})),
            Refusal::SchemaInvalid,
        ),
        (
            CommandName::HistoryRevisionSet,
            "HISTORY_REVISION_SET revision string",
            command_envelope(
                "HISTORY_REVISION_SET",
                serde_json::json!({
                    "run_id": "019a0d5c-4444-7abc-8def-0123456789ab",
                    "revision": "twelve"
                }),
            ),
            Refusal::SchemaInvalid,
        ),
        (
            CommandName::ViewportSet,
            "VIEWPORT_SET zoom out of range",
            command_envelope("VIEWPORT_SET", serde_json::json!({"x": 0.0, "y": 0.0, "zoom": 5.0})),
            Refusal::ZoomRange,
        ),
        (
            CommandName::ViewportRecenter,
            "VIEWPORT_RECENTER bad payload",
            command_envelope("VIEWPORT_RECENTER", serde_json::json!("recenter")),
            Refusal::SchemaInvalid,
        ),
        (
            CommandName::ApprovalResolve,
            "APPROVAL_RESOLVE invalid decision",
            command_envelope(
                "APPROVAL_RESOLVE",
                serde_json::json!({
                    "approval_id": "019a0d5c-6666-7abc-8def-0123456789ab",
                    "nonce": "nonce-1",
                    "decision": "maybe"
                }),
            ),
            Refusal::ApprovalDecisionInvalid,
        ),
        (
            CommandName::ApprovalResolve,
            "APPROVAL_RESOLVE empty nonce",
            command_envelope(
                "APPROVAL_RESOLVE",
                serde_json::json!({
                    "approval_id": "019a0d5c-6666-7abc-8def-0123456789ab",
                    "nonce": "",
                    "decision": "approve"
                }),
            ),
            Refusal::SchemaInvalid,
        ),
        (
            CommandName::SettingsStatus,
            "SETTINGS_STATUS bad payload",
            command_envelope("SETTINGS_STATUS", serde_json::json!("status")),
            Refusal::SchemaInvalid,
        ),
        (
            CommandName::FileTreeList,
            "FILE_TREE_LIST bad payload",
            command_envelope("FILE_TREE_LIST", serde_json::json!([1, 2, 3])),
            Refusal::SchemaInvalid,
        ),
        (
            CommandName::FileRead,
            "FILE_READ missing path",
            command_envelope("FILE_READ", serde_json::json!({})),
            Refusal::SchemaInvalid,
        ),
        (
            CommandName::DiffPropose,
            "DIFF_PROPOSE is decision-gated",
            command_envelope("DIFF_PROPOSE", serde_json::json!({"decision": "approve"})),
            Refusal::DecisionRequired,
        ),
        (
            CommandName::DiffApply,
            "DIFF_APPLY is decision-gated",
            command_envelope("DIFF_APPLY", serde_json::json!({"decision": "approve"})),
            Refusal::DecisionRequired,
        ),
        (
            CommandName::DiffReject,
            "DIFF_REJECT is decision-gated",
            command_envelope("DIFF_REJECT", serde_json::json!({"decision": "reject"})),
            Refusal::DecisionRequired,
        ),
    ]
}

/// Unknown command name sample.
pub fn unknown_command_envelope() -> serde_json::Value {
    command_envelope("NOT_A_REAL_COMMAND", serde_json::json!({}))
}
