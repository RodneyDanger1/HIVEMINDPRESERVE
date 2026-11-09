//! Agent tool contract + registry (Phase 6 WPs 6.5/6.10, CANONICAL C18).
//!
//! One registry owns: the tool ids (enum-derived, never typed counts), the
//! per-tier allowlists, the permission class per tool, and the JSON Schema of
//! each tool's arguments. `schemars` is the schema source (Wave-A pin
//! `schemars 1.2.3`); validation uses `jsonschema 0.60.2`.
//!
//! Rules:
//! - a tool that is not registered → `E_TOOL_NOT_FOUND` (never a silent drop);
//! - a tool not on the agent tier's allowlist → `E_TOOL_NOT_ALLOWED`;
//! - arguments that violate the tool schema → `E_TOOL_SCHEMA`;
//! - Mutate / Execute / Network tools always require the approval gate;
//!   Read tools do not (read-only hints are not an approval bypass).

use std::collections::BTreeMap;

use hiverouter_contract::Refusal;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use strum::{EnumIter, IntoEnumIterator, IntoStaticStr};

use crate::error::ToolError;

/// The permission class of a tool call (approval gate key).
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, EnumIter, IntoStaticStr, Serialize,
    Deserialize,
)]
#[serde(rename_all = "snake_case")]
pub enum PermissionClass {
    /// Reads workspace state only.
    Read,
    /// Mutates workspace files.
    Mutate,
    /// Executes a child process.
    Execute,
    /// Performs a network request.
    Network,
}

impl PermissionClass {
    /// Stable label (ledger/wire).
    #[must_use]
    pub fn label(self) -> &'static str {
        match self {
            PermissionClass::Read => "read",
            PermissionClass::Mutate => "mutate",
            PermissionClass::Execute => "execute",
            PermissionClass::Network => "network",
        }
    }

    /// Whether an approval is required before execution.
    #[must_use]
    pub fn requires_approval(self) -> bool {
        !matches!(self, PermissionClass::Read)
    }

    /// Every class, in declaration order (census input).
    #[must_use]
    pub fn all() -> Vec<PermissionClass> {
        PermissionClass::iter().collect()
    }
}

/// Agent tiers (blueprint §2a / Team Report E4).
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, EnumIter, IntoStaticStr, Serialize,
    Deserialize,
)]
#[serde(rename_all = "snake_case")]
pub enum AgentTier {
    /// Orchestrator; owns the run.
    HyperAgent,
    /// Sub-hive; decomposes work.
    SuperAgent,
    /// Executes one task.
    WorkerAgent,
    /// Invokes approved tools for a task.
    ToolAgent,
}

impl AgentTier {
    /// Stable label.
    #[must_use]
    pub fn label(self) -> &'static str {
        match self {
            AgentTier::HyperAgent => "hyper_agent",
            AgentTier::SuperAgent => "super_agent",
            AgentTier::WorkerAgent => "worker_agent",
            AgentTier::ToolAgent => "tool_agent",
        }
    }

    /// Every tier, in declaration order (census input).
    #[must_use]
    pub fn all() -> Vec<AgentTier> {
        AgentTier::iter().collect()
    }
}

/// The registered tool ids (enum-derived; the census counts these variants).
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, EnumIter, IntoStaticStr, Serialize,
    Deserialize,
)]
#[serde(rename_all = "snake_case")]
pub enum ToolId {
    /// Spawn a child agent.
    SpawnAgent,
    /// Send a short message to this task's parent.
    AskParent,
    /// Send a short message to a sibling under the same parent.
    SendToSibling,
    /// Report a task result to the parent.
    ReportResult,
    /// Finish the current task.
    Finish,
    /// Read a workspace file.
    FileRead,
    /// List a workspace directory.
    FileList,
    /// Write a workspace file.
    FileWrite,
    /// Execute an allowlisted program in the workspace jail.
    ShellExec,
}

impl ToolId {
    /// Stable wire id.
    #[must_use]
    pub fn id(self) -> &'static str {
        match self {
            ToolId::SpawnAgent => "spawn_agent",
            ToolId::AskParent => "ask_parent",
            ToolId::SendToSibling => "send_to_sibling",
            ToolId::ReportResult => "report_result",
            ToolId::Finish => "finish",
            ToolId::FileRead => "file_read",
            ToolId::FileList => "file_list",
            ToolId::FileWrite => "file_write",
            ToolId::ShellExec => "shell_exec",
        }
    }

    /// Resolves a wire id.
    #[must_use]
    pub fn from_id(id: &str) -> Option<ToolId> {
        ToolId::iter().find(|tool| tool.id() == id)
    }

    /// Every tool id, in declaration order (census input).
    #[must_use]
    pub fn all() -> Vec<ToolId> {
        ToolId::iter().collect()
    }
}

/// A tool contract: identity, permission class and argument JSON Schema.
#[derive(Debug, Clone, PartialEq)]
pub struct ToolContract {
    /// Enum identity.
    pub id: ToolId,
    /// Stable wire id.
    pub display_name: &'static str,
    /// Human description (also offered to the model).
    pub description: &'static str,
    /// Permission class.
    pub permission_class: PermissionClass,
    /// Argument JSON Schema (schemars-derived; `additionalProperties: false`).
    pub schema: serde_json::Value,
}

impl ToolContract {
    /// Whether the approval gate must run before execution.
    #[must_use]
    pub fn requires_approval(&self) -> bool {
        self.permission_class.requires_approval()
    }

    /// Validates tool arguments against this contract's JSON Schema.
    ///
    /// # Errors
    /// [`Refusal::ToolSchema`] with the first validation error path.
    pub fn validate_args(&self, args: &serde_json::Value) -> Result<(), ToolError> {
        jsonschema::validate(&self.schema, args).map_err(|error| {
            ToolError::new(
                Refusal::ToolSchema,
                format!("arguments do not match the {} schema: {error}", self.id.id()),
            )
        })
    }
}

/// Schema helper: derives a schemars schema and forbids unknown properties.
macro_rules! schema_of {
    ($ty:ty) => {{
        let mut schema = serde_json::to_value(schemars::schema_for!($ty))
            .expect("schemars schemas are JSON-serialisable");
        if let Some(object) = schema.as_object_mut() {
            object.insert(
                "additionalProperties".to_owned(),
                serde_json::Value::Bool(false),
            );
        }
        schema
    }};
}

/// Arguments for `spawn_agent`.
#[derive(Debug, Clone, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct SpawnAgentArgs {
    /// Role the child should take.
    pub role: String,
    /// Instruction for the child.
    pub instruction: String,
    /// Parent task id (the run row is authoritative; this is a claim only).
    #[serde(default)]
    pub parent_task_id: Option<String>,
}

/// Arguments for `ask_parent`.
#[derive(Debug, Clone, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct AskParentArgs {
    /// Bounded message text.
    pub text: String,
}

/// Arguments for `send_to_sibling`.
#[derive(Debug, Clone, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct SendToSiblingArgs {
    /// Sibling task id; must share this task's parent and run.
    pub target_task_id: String,
    /// Bounded message text.
    pub text: String,
}

/// Arguments for `report_result`.
#[derive(Debug, Clone, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ReportResultArgs {
    /// Summary of the task result.
    pub summary: String,
    /// Whether the task succeeded.
    pub success: bool,
}

/// Arguments for `finish`.
#[derive(Debug, Clone, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct FinishArgs {
    /// Final verdict text.
    pub verdict: String,
}

/// Arguments for `file_read`.
#[derive(Debug, Clone, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct FileReadArgs {
    /// Workspace-relative path.
    pub path: String,
}

/// Arguments for `file_list`.
#[derive(Debug, Clone, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct FileListArgs {
    /// Workspace-relative directory path (defaults to the root).
    #[serde(default)]
    pub path: Option<String>,
}

/// Arguments for `file_write`.
#[derive(Debug, Clone, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct FileWriteArgs {
    /// Workspace-relative path.
    pub path: String,
    /// Full new file content.
    pub content: String,
    /// Optional content hash the caller believes the file has (stale-write guard).
    #[serde(default)]
    pub expected_sha256: Option<String>,
}

/// Arguments for `shell_exec`.
#[derive(Debug, Clone, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ShellExecArgs {
    /// Program name or path (must be on the shell allowlist).
    pub program: String,
    /// Arguments passed directly (no shell interpolation).
    pub args: Vec<String>,
    /// Workspace-relative working directory.
    #[serde(default)]
    pub cwd: Option<String>,
}

/// The one tool registry (contracts + per-tier allowlists).
#[derive(Debug, Clone)]
pub struct ToolRegistry {
    contracts: BTreeMap<ToolId, ToolContract>,
    allowlists: BTreeMap<AgentTier, Vec<ToolId>>,
}

impl Default for ToolRegistry {
    fn default() -> Self {
        Self::standard()
    }
}

impl ToolRegistry {
    /// Builds the v1 registry: every [`ToolId`] contract plus the frozen tier
    /// allowlists.
    #[must_use]
    pub fn standard() -> Self {
        let contract = |id: ToolId,
                        description: &'static str,
                        permission_class: PermissionClass,
                        schema: serde_json::Value| ToolContract {
            id,
            display_name: id.id(),
            description,
            permission_class,
            schema,
        };

        let mut contracts = BTreeMap::new();
        for contract in [
            contract(
                ToolId::SpawnAgent,
                "Spawn a child agent (depth/fan-out/agent caps enforced before any spawn).",
                PermissionClass::Execute,
                schema_of!(SpawnAgentArgs),
            ),
            contract(
                ToolId::AskParent,
                "Send a bounded message to this task's parent.",
                PermissionClass::Read,
                schema_of!(AskParentArgs),
            ),
            contract(
                ToolId::SendToSibling,
                "Send a bounded message to a sibling; cross-branch targets require operator approval.",
                PermissionClass::Mutate,
                schema_of!(SendToSiblingArgs),
            ),
            contract(
                ToolId::ReportResult,
                "Report a task result to the parent task.",
                PermissionClass::Read,
                schema_of!(ReportResultArgs),
            ),
            contract(
                ToolId::Finish,
                "Finish the current task with a final verdict.",
                PermissionClass::Read,
                schema_of!(FinishArgs),
            ),
            contract(
                ToolId::FileRead,
                "Read a file inside the workspace jail.",
                PermissionClass::Read,
                schema_of!(FileReadArgs),
            ),
            contract(
                ToolId::FileList,
                "List a directory inside the workspace jail.",
                PermissionClass::Read,
                schema_of!(FileListArgs),
            ),
            contract(
                ToolId::FileWrite,
                "Write a file inside the workspace jail (requires approval).",
                PermissionClass::Mutate,
                schema_of!(FileWriteArgs),
            ),
            contract(
                ToolId::ShellExec,
                "Execute an allowlisted program inside the workspace jail (requires approval).",
                PermissionClass::Execute,
                schema_of!(ShellExecArgs),
            ),
        ] {
            contracts.insert(contract.id, contract);
        }

        let mut allowlists = BTreeMap::new();
        allowlists.insert(
            AgentTier::HyperAgent,
            vec![ToolId::SpawnAgent, ToolId::ReportResult, ToolId::Finish],
        );
        allowlists.insert(
            AgentTier::SuperAgent,
            vec![
                ToolId::SpawnAgent,
                ToolId::AskParent,
                ToolId::SendToSibling,
                ToolId::ReportResult,
                ToolId::Finish,
                ToolId::FileRead,
                ToolId::FileList,
                ToolId::FileWrite,
                ToolId::ShellExec,
            ],
        );
        allowlists.insert(
            AgentTier::WorkerAgent,
            vec![
                ToolId::SpawnAgent,
                ToolId::AskParent,
                ToolId::SendToSibling,
                ToolId::ReportResult,
                ToolId::Finish,
                ToolId::FileRead,
                ToolId::FileList,
                ToolId::FileWrite,
                ToolId::ShellExec,
            ],
        );
        allowlists.insert(
            AgentTier::ToolAgent,
            vec![
                ToolId::FileRead,
                ToolId::FileList,
                ToolId::FileWrite,
                ToolId::ShellExec,
            ],
        );

        Self {
            contracts,
            allowlists,
        }
    }

    /// The contract for a tool id (registered in the standard registry).
    #[must_use]
    pub fn contract(&self, id: ToolId) -> Option<&ToolContract> {
        self.contracts.get(&id)
    }

    /// Whether `tier` may call `id`.
    #[must_use]
    pub fn is_allowed(&self, tier: AgentTier, id: ToolId) -> bool {
        self.allowlists
            .get(&tier)
            .map_or(false, |allowed| allowed.contains(&id))
    }

    /// The allowlist for a tier.
    #[must_use]
    pub fn allowlist(&self, tier: AgentTier) -> &[ToolId] {
        self.allowlists
            .get(&tier)
            .map(Vec::as_slice)
            .unwrap_or(&[])
    }

    /// Resolves a tool call to its contract, enforcing the tier allowlist.
    ///
    /// # Errors
    /// [`Refusal::ToolNotFound`] for an unregistered id and
    /// [`Refusal::ToolNotAllowed`] when the tier has no grant.
    pub fn resolve(&self, tier: AgentTier, id: ToolId) -> Result<&ToolContract, ToolError> {
        let contract = self.contracts.get(&id).ok_or_else(|| {
            ToolError::new(
                Refusal::ToolNotFound,
                format!("tool id {id:?} is not registered"),
            )
        })?;
        if !self.is_allowed(tier, id) {
            return Err(ToolError::new(
                Refusal::ToolNotAllowed,
                format!("tool {} is not on the {} allowlist", id.id(), tier.label()),
            ));
        }
        Ok(contract)
    }

    /// Resolves a wire tool name ("spawn_agent", ...) and validates its args.
    ///
    /// # Errors
    /// [`Refusal::ToolNotFound`], [`Refusal::ToolNotAllowed`] or
    /// [`Refusal::ToolSchema`].
    pub fn resolve_call(
        &self,
        tier: AgentTier,
        tool_name: &str,
        args: &serde_json::Value,
    ) -> Result<&ToolContract, ToolError> {
        let id = ToolId::from_id(tool_name).ok_or_else(|| {
            ToolError::new(
                Refusal::ToolNotFound,
                format!("tool {tool_name:?} is not in the frozen tool vocabulary"),
            )
        })?;
        let contract = self.resolve(tier, id)?;
        contract.validate_args(args)?;
        Ok(contract)
    }

    /// Registered ids, derived from the map (never a typed count).
    pub fn registered_ids(&self) -> impl Iterator<Item = &ToolId> {
        self.contracts.keys()
    }

    /// Number of registered contracts.
    #[must_use]
    pub fn contract_count(&self) -> usize {
        self.contracts.len()
    }

    /// Number of allowlist grants across all tiers (census input).
    #[must_use]
    pub fn grant_count(&self) -> usize {
        self.allowlists.values().map(Vec::len).sum()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_tool_id_has_exactly_one_contract() {
        let registry = ToolRegistry::standard();
        let enum_ids: Vec<ToolId> = ToolId::all();
        assert_eq!(registry.contract_count(), enum_ids.len(), "census drift: enum vs registry");
        for id in enum_ids {
            assert!(registry.contract(id).is_some(), "missing contract for {id:?}");
        }
        for id in registry.registered_ids() {
            assert!(ToolId::from_id(id.id()).is_some());
        }
    }

    #[test]
    fn schemas_are_valid_and_forbid_unknown_fields() {
        let registry = ToolRegistry::standard();
        for id in ToolId::all() {
            let contract = registry.contract(id).expect("contract");
            let schema = &contract.schema;
            assert!(schema.is_object(), "{id:?} schema is an object");
            assert_eq!(
                schema.get("additionalProperties"),
                Some(&serde_json::Value::Bool(false)),
                "{id:?} schema forbids unknown fields"
            );
            // A deliberately empty object fails the required-field check for
            // every tool except `file_list` (all of its fields are optional).
            let empty = serde_json::json!({});
            if id == ToolId::FileList {
                assert!(contract.validate_args(&empty).is_ok());
            } else {
                assert_eq!(
                    contract.validate_args(&empty).expect_err("required").code(),
                    "E_TOOL_SCHEMA"
                );
            }
            jsonschema::validator_for(schema).expect("schema compiles");
        }
    }

    #[test]
    fn tier_allowlist_grants_and_refusals_are_by_name() {
        let registry = ToolRegistry::standard();
        assert!(registry.is_allowed(AgentTier::HyperAgent, ToolId::SpawnAgent));
        assert!(!registry.is_allowed(AgentTier::HyperAgent, ToolId::ShellExec));
        assert!(!registry.is_allowed(AgentTier::HyperAgent, ToolId::FileWrite));
        assert!(registry.is_allowed(AgentTier::WorkerAgent, ToolId::ShellExec));
        assert!(!registry.is_allowed(AgentTier::ToolAgent, ToolId::SpawnAgent));

        let error = registry
            .resolve(AgentTier::HyperAgent, ToolId::ShellExec)
            .expect_err("not allowed");
        assert_eq!(error.code(), "E_TOOL_NOT_ALLOWED");
        assert!(registry.resolve(AgentTier::WorkerAgent, ToolId::FileRead).is_ok());
    }

    #[test]
    fn resolve_call_validates_args_by_name() {
        let registry = ToolRegistry::standard();
        let ok = registry
            .resolve_call(
                AgentTier::WorkerAgent,
                "file_read",
                &serde_json::json!({"path": "docs/a.txt"}),
            )
            .expect("valid");
        assert_eq!(ok.permission_class, PermissionClass::Read);
        assert!(!ok.requires_approval());

        let unknown = registry
            .resolve_call(AgentTier::WorkerAgent, "rm_rf", &serde_json::json!({}))
            .expect_err("unknown tool");
        assert_eq!(unknown.code(), "E_TOOL_NOT_FOUND");

        let bad_args = registry
            .resolve_call(AgentTier::WorkerAgent, "file_read", &serde_json::json!({"wrong": 1}))
            .expect_err("bad args");
        assert_eq!(bad_args.code(), "E_TOOL_SCHEMA");

        let extra = registry
            .resolve_call(
                AgentTier::WorkerAgent,
                "file_read",
                &serde_json::json!({"path": "a", "extra": true}),
            )
            .expect_err("unknown field");
        assert_eq!(extra.code(), "E_TOOL_SCHEMA");

        let not_allowed = registry
            .resolve_call(AgentTier::HyperAgent, "file_read", &serde_json::json!({"path": "a"}))
            .expect_err("tier");
        assert_eq!(not_allowed.code(), "E_TOOL_NOT_ALLOWED");
    }

    #[test]
    fn mutating_and_executing_tools_require_approval() {
        let registry = ToolRegistry::standard();
        for id in ToolId::all() {
            let contract = registry.contract(id).expect("contract");
            let expected = match contract.permission_class {
                PermissionClass::Read => false,
                PermissionClass::Mutate | PermissionClass::Execute | PermissionClass::Network => true,
            };
            assert_eq!(contract.requires_approval(), expected, "{id:?}");
        }
        assert!(registry
            .contract(ToolId::FileWrite)
            .expect("contract")
            .requires_approval());
        assert!(registry
            .contract(ToolId::ShellExec)
            .expect("contract")
            .requires_approval());
    }

    #[test]
    fn allowlist_membership_is_always_a_registered_tool() {
        let registry = ToolRegistry::standard();
        for tier in AgentTier::all() {
            for id in registry.allowlist(tier) {
                assert!(ToolId::from_id(id.id()).is_some());
            }
        }
    }
}
