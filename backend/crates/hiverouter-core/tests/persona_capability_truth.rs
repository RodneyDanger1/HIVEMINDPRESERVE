//! Persona-vs-registry capability truth.
//!
//! The census compares the registry, tier allowlists and a hardcoded prompt
//! claim list. It cannot see the shipped persona files, so a registry tool can
//! be fully wired but advertised to no agent. This test closes that gap: the
//! union of every shipped persona allowlist must equal the registered tool set.

use std::collections::BTreeSet;

use hiverouter_core::prompts::{load_personas_embedded, TemplateRegistry};
use hiverouter_tools::census::PROMPT_CLAIMED_TOOL_IDS;
use hiverouter_tools::tool::{ToolId, ToolRegistry};

#[test]
fn every_registered_tool_is_advertised_by_at_least_one_shipped_persona() {
    let templates = TemplateRegistry::load_embedded().expect("embedded templates");
    let personas = load_personas_embedded(&templates).expect("embedded personas");

    let registered: BTreeSet<&str> = ToolId::all()
        .iter()
        .map(|tool| tool.id())
        .collect();
    let advertised: BTreeSet<&str> = personas
        .iter()
        .flat_map(|persona| persona.allowed_tool_ids.iter().map(String::as_str))
        .collect();

    let missing: Vec<&&str> = registered.difference(&advertised).collect();
    assert!(
        missing.is_empty(),
        "registered tools are advertised to no shipped persona: {missing:?}"
    );
    let unknown: Vec<&&str> = advertised.difference(&registered).collect();
    assert!(
        unknown.is_empty(),
        "shipped personas advertise unknown tools: {unknown:?}"
    );

    // The hardcoded prompt claim list must equal the actual registry, so this
    // test also fails when one side is edited alone.
    let claimed: BTreeSet<&str> = PROMPT_CLAIMED_TOOL_IDS.iter().copied().collect();
    assert_eq!(
        claimed, registered,
        "PROMPT_CLAIMED_TOOL_IDS drifted from the registry"
    );
}

#[test]
fn a_worker_persona_can_reach_the_full_coding_surface() {
    let templates = TemplateRegistry::load_embedded().expect("embedded templates");
    let personas = load_personas_embedded(&templates).expect("embedded personas");
    let registry = ToolRegistry::standard();
    let agent = personas
        .iter()
        .find(|persona| persona.id == "agent")
        .expect("agent persona");
    for tool in [
        "fs_glob",
        "fs_grep",
        "file_read_range",
        "apply_patch",
        "run_tests",
        "git_stage",
        "git_commit",
        "git_status",
        "git_diff",
        "git_log",
    ] {
        let id = ToolId::from_id(tool).expect("tool id");
        assert!(
            agent.allowed_tool_ids.iter().any(|allowed| allowed == tool),
            "agent persona lacks {tool}"
        );
        assert!(
            registry.is_allowed(hiverouter_tools::tool::AgentTier::WorkerAgent, id),
            "worker tier lacks {tool}"
        );
    }
}
