//! Named refusal table — stable `E_*` codes shared by backend and UI.
//!
//! Codes come from `docs/contracts/UI-COMMANDS-v1.md` §5, the Phase 1 work
//! package tables (blueprint WP 1.6) and the budget invariants (blueprint §2a).
//! A code is a stable identifier: messages may change, codes must not. Adding a
//! variant is a contract change and needs a `CHANGE-LEDGER-IMPL.md` row plus a
//! re-run of the command matrix test.

use serde::{Deserialize, Deserializer, Serialize, Serializer};

macro_rules! refusal_table {
    ($( $variant:ident => ($code:literal, $retryable:literal, $message:literal, $fix:literal) ),+ $(,)?) => {
        /// A named refusal. Every error path in the backend and every refusal in
        /// the UI is one of these variants; there is no free-form error string
        /// on the wire.
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, thiserror::Error)]
        pub enum Refusal {
            $(
                #[error($message)]
                $variant,
            )+
        }

        impl Refusal {
            /// Every refusal, in declaration order. Used by the unique-code test
            /// and by the UI refusal table.
            pub const ALL: &'static [Refusal] = &[$(Refusal::$variant),+];

            /// Stable wire code (`E_*`).
            #[must_use]
            pub fn code(&self) -> &'static str {
                match self {
                    $(Refusal::$variant => $code),+
                }
            }

            /// Human-readable cause (may change; the code is the stable identity).
            #[must_use]
            pub fn message(&self) -> &'static str {
                match self {
                    $(Refusal::$variant => $message),+
                }
            }

            /// What the user or caller can do about it.
            #[must_use]
            pub fn how_to_fix(&self) -> &'static str {
                match self {
                    $(Refusal::$variant => $fix),+
                }
            }

            /// Whether retrying the same request may succeed without a change.
            #[must_use]
            pub fn retryable(&self) -> bool {
                match self {
                    $(Refusal::$variant => $retryable),+
                }
            }

            /// Looks a refusal up by its stable code.
            #[must_use]
            pub fn from_code(code: &str) -> Option<Refusal> {
                match code {
                    $($code => Some(Refusal::$variant),)+
                    _ => None,
                }
            }
        }
    };
}

refusal_table! {
    // --- transport / UI -----------------------------------------------------
    OriginRejected => ("E_ORIGIN_REJECTED", false,
        "The WebSocket Origin header is not on the localhost allowlist.",
        "Connect from http://127.0.0.1:<port> or http://localhost:<port>; do not proxy the UI from another origin."),
    TokenInvalid => ("E_TOKEN_INVALID", false,
        "The per-session WebSocket token is missing or does not match.",
        "Reload the UI so it fetches a fresh session token from the local server."),
    ResyncRequired => ("E_RESYNC_REQUIRED", true,
        "The client is too far behind the server event stream to receive a diff.",
        "Request a fresh snapshot (HISTORY_REVISION_SET) before applying further events."),
    ZoomRange => ("E_ZOOM_RANGE", false,
        "Viewport zoom is outside the allowed 0.1..3.0 range.",
        "Clamp zoom to 0.1..3.0 before sending VIEWPORT_SET."),
    HistoryGap => ("E_HISTORY_GAP", false,
        "The requested run revision is not available in the local snapshot history.",
        "Request the nearest available revision from the history API."),
    DbBusy => ("E_DB_BUSY", true,
        "The local SQLite database is locked by another operation.",
        "Retry after the current write completes."),
    SchemaNewer => ("E_DB_SCHEMA_NEWER", false,
        "The local database schema is newer than this HiveMind build supports.",
        "Upgrade HiveMind or restore a backup made by this build; do not delete the database."),
    WsDisconnected => ("E_WS_DISCONNECTED", true,
        "The local WebSocket connection is not available.",
        "Wait for reconnect; the server resynchronises on hello{last_seq}."),
    BadFrame => ("E_BAD_FRAME", false,
        "The frame is not valid JSON or does not match the envelope schema.",
        "Send a JSON object matching the hiverouter.v1 envelope."),

    // --- command / schema ---------------------------------------------------
    CommandUnknown => ("E_COMMAND_UNKNOWN", false,
        "The command name is not part of the frozen UI-COMMANDS-v1 vocabulary.",
        "Use a command name listed in docs/contracts/UI-COMMANDS-v1.md section 2."),
    SchemaInvalid => ("E_SCHEMA_INVALID", false,
        "The frame or payload failed schema validation.",
        "Fix the payload fields and types; see the command's payload schema."),
    PromptEmpty => ("E_PROMPT_EMPTY", false,
        "The submitted prompt text is empty.",
        "Type a prompt of 1..8000 characters."),
    PromptTooLong => ("E_PROMPT_TOO_LONG", false,
        "The submitted prompt text exceeds 8000 characters.",
        "Shorten the prompt to 8000 characters or fewer."),
    RunActive => ("E_RUN_ACTIVE", false,
        "A run is already active in this session.",
        "Cancel the active run before submitting another prompt."),
    KeyMissing => ("E_KEY_MISSING", false,
        "No DeepSeek API key is configured.",
        "Set the DEEPSEEK_API_KEY environment variable and restart the server."),
    SpawnDepth => ("E_SPAWN_DEPTH", false,
        "The spawn would exceed the maximum depth of 3 (Hyper -> Super -> Worker).",
        "Spawn from a shallower task, or let the existing worker finish."),
    FanoutLimit => ("E_FANOUT_LIMIT", false,
        "The spawn would exceed the fan-out limit of 8 children per task.",
        "Reuse an existing child task or wait for one to finish."),
    AgentLimit => ("E_AGENT_LIMIT", false,
        "The run would exceed the 64-agent total cap.",
        "Reduce the planned agent count or finish the run."),
    SemaphoreFull => ("E_SEMAPHORE_FULL", true,
        "The local in-flight provider semaphore is full.",
        "Wait for in-flight calls to finish; the scheduler retries queued work."),
    Cycle => ("E_CYCLE", false,
        "The rewire would introduce a cycle in the task graph.",
        "Choose a new parent that does not descend from the task."),
    RewireInFlight => ("E_REWIRE_IN_FLIGHT", true,
        "Another rewire is already in flight for this task.",
        "Retry after the current rewire is acknowledged."),
    RunTerminal => ("E_RUN_TERMINAL", false,
        "The run has already finished (completed, failed or cancelled).",
        "Start a new run with SUBMIT_PROMPT."),
    RunNotFound => ("E_RUN_NOT_FOUND", false,
        "The referenced run does not exist in this workspace.",
        "Check the run id returned by RUN_CREATED."),
    TaskNotFound => ("E_TASK_NOT_FOUND", false,
        "The referenced task does not exist in this run.",
        "Check the task id returned by NODE_SPAWN."),
    TaskTerminal => ("E_TASK_TERMINAL", false,
        "The task has already reached a terminal state.",
        "Spawn a new task instead of killing a finished one."),
    KillDenied => ("E_KILL_DENIED", false,
        "The task cannot be killed from its current state.",
        "Only running or queued tasks can be killed."),
    BudgetExceeded => ("E_BUDGET_EXCEEDED", false,
        "The run exceeded the $0.50 cost cap.",
        "Start a new run or raise the cap through the run policy."),
    TokenBudgetExceeded => ("E_TOKEN_BUDGET_EXCEEDED", false,
        "The run exceeded the 400,000 combined token budget.",
        "Start a new run or reduce the planned output."),
    CallLimit => ("E_CALL_LIMIT", false,
        "The run exceeded the 40-call cap or the 40-super-step plan ceiling.",
        "Split the work into a new run."),
    UsageIncomplete => ("E_USAGE_INCOMPLETE", false,
        "Provider usage counters are missing, so cost cannot be settled.",
        "Inspect the provider response; incomplete usage marks the run INCOMPLETE and never settles a cost."),
    PriceUnsourced => ("E_PRICE_UNSOURCED", false,
        "A price row has no source_url/fetched_at provenance.",
        "Add the missing provenance to the price table."),
    PeakUndefined => ("E_PEAK_UNDEFINED", false,
        "The pricing peak/off-peak window is not defined for this instant.",
        "Configure a priced window that covers the current UTC time."),
    CostMathInvalid => ("E_COST_MATH_INVALID", false,
        "Cost arithmetic reached a non-decimal path.",
        "Keep money arithmetic in rust_decimal; never use f64 on cost paths."),
    MultikeyForbidden => ("E_MULTIKEY_FORBIDDEN", false,
        "Multiple API keys are not supported.",
        "Configure exactly one DeepSeek account key."),
    LlmRateLimit => ("E_LLM_RATE_LIMIT", true,
        "The provider rate-limited the request (HTTP 429) after the retry budget.",
        "Wait for the Retry-After window, then retry."),
    ApprovalRequired => ("E_APPROVAL_REQUIRED", false,
        "The tool call requires explicit approval before execution.",
        "Approve or reject the pending APPROVAL_REQUEST."),
    ApprovalReplay => ("E_APPROVAL_REPLAY", false,
        "The approval nonce was already used.",
        "Resolve the current pending approval; do not replay an old decision."),
    ApprovalTimeout => ("E_APPROVAL_TIMEOUT", true,
        "The approval request timed out before a decision arrived.",
        "Resolve the request when prompted; the tool call is refused, never faked."),
    ApprovalNotFound => ("E_APPROVAL_NOT_FOUND", false,
        "The approval id does not exist or is no longer pending.",
        "Refresh the pending approvals and resolve an approval that still exists."),
    ApprovalDecisionInvalid => ("E_APPROVAL_DECISION_INVALID", false,
        "The approval decision is not one of the frozen values.",
        "Send decision \"approve\" or \"deny\" with the pending approval nonce."),
    DiffDecisionRequired => ("E_DIFF_DECISION_REQUIRED", false,
        "A code diff requires an explicit decision.",
        "Approve or reject the diff decision; DIFF_* is decision-gated in v1."),
    DecisionRequired => ("E_DECISION_REQUIRED", false,
        "This action is decision-gated and is not implemented in v1.",
        "Use the frozen command set; DIFF_* stays decision-gated."),
    TemplateUnknown => ("E_TEMPLATE_UNKNOWN", false,
        "The template id is not present in the template manifest.",
        "Add the template manifest row before inserting it."),
    TemplateLicenseMissing => ("E_TEMPLATE_LICENSE_MISSING", false,
        "The template licence is missing or not on the allowlist.",
        "Record {SPDX, source_url, fetched_at}; allowed: MIT, Apache-2.0, BSD, ISC, CC0-1.0."),
    TemplateRefForbidden => ("E_TEMPLATE_REF_FORBIDDEN", false,
        "The template uses a forbidden environment or file reference.",
        "Inline literal values into the template; env/file refs are refused for user templates."),
    TemplateInvalid => ("E_TEMPLATE_INVALID", false,
        "The template asset failed schema validation.",
        "Fix the .prompty front matter and role markers."),
    PersonaUnknown => ("E_PERSONA_UNKNOWN", false,
        "The persona id is not present in the persona registry.",
        "Register the persona before spawning with it."),
    SnapshotCorrupt => ("E_SNAPSHOT_CORRUPT", false,
        "The graph snapshot JSON is corrupt or fails its versioned schema.",
        "Restore an earlier snapshot revision; the workspace is never silently reset."),

    // --- files / workspace jail --------------------------------------------
    PathOutsideJail => ("E_PATH_OUTSIDE_JAIL", false,
        "The requested path resolves outside the configured workspace root.",
        "Use a workspace-relative path that stays inside the workspace root."),
    PathNotFound => ("E_PATH_NOT_FOUND", false,
        "The requested file or directory does not exist inside the workspace.",
        "Refresh the file tree and pick an existing workspace-relative path."),
    FileTooLarge => ("E_FILE_TOO_LARGE", false,
        "The requested file exceeds the 1 MiB read cap.",
        "Read a smaller file or add the path to a workspace exclusion list."),
    FileTreeUnavailable => ("E_FILE_TREE_UNAVAILABLE", false,
        "Workspace file access is unavailable because no usable workspace root is configured.",
        "Start the server with --workspace-root pointing at an existing directory."),

    // --- tool / safety ------------------------------------------------------
    ToolNotFound => ("E_TOOL_NOT_FOUND", false,
        "The requested tool is not registered.",
        "Register the tool in the tool registry before calling it."),
    ToolSchema => ("E_TOOL_SCHEMA", false,
        "The tool arguments failed the tool's input schema.",
        "Fix the arguments to match the tool schema."),
    JailEscape => ("E_JAIL_ESCAPE", false,
        "The path or process escaped the workspace jail.",
        "Use a path inside the workspace root; escapes are refused, never clamped."),
    ToolTimeout => ("E_TOOL_TIMEOUT", true,
        "The tool exceeded its execution timeout.",
        "Retry with a smaller scope or a longer per-tool timeout."),
    ToolCancelled => ("E_TOOL_CANCELLED", true,
        "The tool was cancelled by run cancellation.",
        "Retry after the run is restarted."),
    ToolNotAllowed => ("E_TOOL_NOT_ALLOWED", false,
        "The tool is not on this agent tier's allowlist.",
        "Use a tool from the tier's capability census."),

    // --- provider -----------------------------------------------------------
    ProviderAuth => ("E_PROVIDER_AUTH", false,
        "The DeepSeek API rejected the credentials.",
        "Check DEEPSEEK_API_KEY; HTTP 401 is never retried."),
    ProviderBalance => ("E_PROVIDER_BALANCE", false,
        "The DeepSeek API account has insufficient balance.",
        "Top up the DeepSeek account or use a funded key; HTTP 402 is never retried."),
    ProviderRateLimit => ("E_PROVIDER_RATE_LIMIT", true,
        "DeepSeek returned HTTP 429 (account concurrency or rate limit).",
        "Back off and retry within the retry budget; reduce in-flight calls."),
    ProviderTimeout => ("E_PROVIDER_TIMEOUT", true,
        "The provider call did not complete within its timeout.",
        "Retry within the retry budget."),
    ProviderIncomplete => ("E_PROVIDER_INCOMPLETE", false,
        "The provider stream ended before the reply was complete.",
        "Retry the call; the partial reply is not reported as success."),
    ProviderInvalidResponse => ("E_PROVIDER_INVALID_RESPONSE", false,
        "The provider response did not match the documented response shape.",
        "Inspect the response body; do not parse undocumented fields."),

    // --- build / toolchain --------------------------------------------------
    ToolchainMsrv => ("E_TOOLCHAIN_MSRV", false,
        "The Rust toolchain is older than the required MSRV 1.89.0.",
        "Install Rust 1.89.0 or newer and rebuild."),
    NodeMissing => ("E_NODE_MISSING", false,
        "Node.js is missing or older than the required 22.12.0.",
        "Install Node.js 22.12.0 or newer; the web build and typecheck need it."),

    // --- remote control plane ------------------------------------------------
    RemoteUnknown => ("E_REMOTE_UNKNOWN", false,
        "The remote principal does not exist or is retired.",
        "Refresh the remote roster and choose an enrolled remote."),
    RemoteOffline => ("E_REMOTE_OFFLINE", true,
        "The remote is not currently online; no action was dispatched.",
        "Wait for a heartbeat or run the supervisor probe before retrying."),
    RemoteCapability => ("E_REMOTE_CAPABILITY", false,
        "No live grant covers this remote, device, capability and action.",
        "Grant the exact capability/action to this remote before actuating."),
    DeviceUnknown => ("E_DEVICE_UNKNOWN", false,
        "The device is not registered to this remote.",
        "Refresh the remote device inventory and choose a registered device."),
    DeviceStateStale => ("E_DEVICE_STATE_STALE", true,
        "The device state is stale or unknown; a safe action cannot be planned.",
        "Refresh the device state and retry only after a fresh observation."),
    DeviceActionInFlight => ("E_DEVICE_ACTION_IN_FLIGHT", true,
        "Another action for this device is already in flight.",
        "Wait for the current action result; a device is single-flight."),
    DeviceApprovalRequired => ("E_DEVICE_APPROVAL_REQUIRED", false,
        "This device action requires an explicit operator approval.",
        "Resolve the pending device approval with its nonce; do not auto-actuate."),
    DeviceApprovalInvalid => ("E_DEVICE_APPROVAL_INVALID", false,
        "The device approval is missing, expired, replayed or does not match the request.",
        "Refresh the pending approval and resolve the current nonce once."),
    RemoteTransport => ("E_REMOTE_TRANSPORT", false,
        "The remote transport is not supported or is unavailable in this build.",
        "Use the in-process fixture adapter or enable a supported transport."),
    DetachedConflict => ("E_DETACHED_CONFLICT", false,
        "A detached remote or daemon with that display name already exists in this hive.",
        "Choose a different display name or retire the existing principal first."),
    DetachedCap => ("E_DETACHED_CAP", false,
        "The run's lower-level detached-principal admission cap is reached.",
        "Retire an existing detached principal or use the HyperAgent root-control path."),

    // --- internal -----------------------------------------------------------
    InternalError => ("E_INTERNAL_ERROR", true,
        "An internal invariant failed; the request was refused rather than faked.",
        "Check the server log for the refusal context and retry after fixing the invariant."),
}

impl Serialize for Refusal {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(self.code())
    }
}

impl<'de> Deserialize<'de> for Refusal {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let code = String::deserialize(deserializer)?;
        Refusal::from_code(&code)
            .ok_or_else(|| serde::de::Error::custom(format!("unknown refusal code: {code}")))
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use super::*;

    #[test]
    fn codes_are_unique_and_well_formed() {
        let mut seen = HashSet::new();
        for refusal in Refusal::ALL {
            let code = refusal.code();
            assert!(seen.insert(code), "duplicate refusal code: {code}");
            assert!(
                code.starts_with("E_") && code[2..].bytes().all(|b| b.is_ascii_uppercase() || b == b'_'),
                "malformed refusal code: {code}"
            );
        }
        assert!(Refusal::ALL.len() >= 40, "refusal table unexpectedly small");
    }

    #[test]
    fn from_code_roundtrips_every_variant() {
        for refusal in Refusal::ALL {
            assert_eq!(Refusal::from_code(refusal.code()), Some(*refusal));
            assert!(!refusal.message().is_empty());
            assert!(!refusal.how_to_fix().is_empty());
        }
        assert_eq!(Refusal::from_code("E_NOT_A_REAL_CODE"), None);
    }

    #[test]
    fn wire_serialization_uses_codes() {
        let json = serde_json::to_string(&Refusal::Cycle).expect("serializes");
        assert_eq!(json, "\"E_CYCLE\"");
        let parsed: Refusal = serde_json::from_str("\"E_CYCLE\"").expect("deserializes");
        assert_eq!(parsed, Refusal::Cycle);
        assert!(serde_json::from_str::<Refusal>("\"E_BOGUS\"").is_err());
    }
}
