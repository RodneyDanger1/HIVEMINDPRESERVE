//! Tool registry/allowlist census truth.
//!
//! These assertions cover the properties a self-diagnosis tool depends on:
//! the registry, prompt claims and tier allowlists agree (derived census), and
//! the HyperAgent/SuperAgent observation surfaces are read-only tools that need
//! no operator approval.

use hiverouter_tools::tool::{AgentTier, PermissionClass, ToolId, ToolRegistry};
use hiverouter_tools::{standard_census, PROMPT_CLAIMED_TOOL_IDS};

#[test]
fn census_agrees_with_registry_allowlists_and_prompt_claims() {
    let registry = ToolRegistry::standard();
    let census = standard_census().expect("capability census has no drift");
    assert_eq!(census.tool_count, ToolId::all().len());
    assert_eq!(census.tool_count, registry.contract_count());
    assert_eq!(
        census.registry_tool_ids.len(),
        PROMPT_CLAIMED_TOOL_IDS.len(),
        "registry and prompt claims are the same set"
    );
    for tier in AgentTier::all() {
        for id in registry.allowlist(tier) {
            assert!(
                registry.contract(*id).is_some(),
                "{} grant {id:?} has no contract",
                tier.label()
            );
        }
    }
}

#[test]
fn hyper_and_super_agents_get_read_only_observation_tools() {
    let registry = ToolRegistry::standard();
    for tier in [AgentTier::HyperAgent, AgentTier::SuperAgent] {
        for tool in [
            "fs_glob",
            "fs_grep",
            "file_read_range",
            "git_status",
            "git_diff",
            "git_log",
        ] {
            let id = ToolId::from_id(tool).expect("registered tool");
            assert!(
                registry.is_allowed(tier, id),
                "{} must be able to observe via {tool}",
                tier.label()
            );
            let contract = registry.contract(id).expect("contract");
            assert_eq!(
                contract.permission_class,
                PermissionClass::Read,
                "{tool} must stay read-only for {}",
                tier.label()
            );
            assert!(
                !contract.requires_approval(),
                "{tool} must not prompt for an approval"
            );
        }
    }

    // The diagnostic tier grant must never be confused with mutation authority.
    assert!(!registry.is_allowed(AgentTier::HyperAgent, ToolId::ShellExec));
    assert!(!registry.is_allowed(AgentTier::HyperAgent, ToolId::FileWrite));
    assert!(registry.is_allowed(AgentTier::SuperAgent, ToolId::FileWrite));
    assert_eq!(
        registry
            .contract(ToolId::FileWrite)
            .expect("file_write contract")
            .permission_class,
        PermissionClass::Mutate
    );
}
