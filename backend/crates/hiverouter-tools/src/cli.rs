//! Agent-native `hiverouter` CLI (Phase 12 WPs 12.1/12.3, agent-native design).
//!
//! Contract:
//! - **stdout is for machines**: every outcome (success *and* failure) is a
//!   `hiverouter.cli.v1` JSON envelope when stdout is not a TTY or when
//!   `--format json` / `--json` is passed (the default for agents);
//! - **stderr is for humans**: human mode (`--format human` or a TTY) prints
//!   readable lines there and leaves stdout empty;
//! - **exit codes are stable**: `0` ok, `2` usage/parse error, `3` named
//!   refusal (`E_*` with `how_to_fix`), `4` internal error;
//! - **self-describing**: top-level `--help` lists resources, resource
//!   `--help` lists actions, `hiverouter schema [command]` returns the typed
//!   schema, generation from one command table (no drift);
//! - **dry-run** previews the HTTP request for `health`/`diagnose` without
//!   sending; `preflight` never sends;
//! - **safety tiers**: every command declares `tier` (0 read-only local, 1 read
//!   local HTTP, 2 mutating) and `mutating`; v1 ships no mutating CLI command,
//!   so `--yes` is intentionally absent rather than a no-op.
//!
//! The CLI reads `DEEPSEEK_API_KEY` only to report a boolean presence; it never
//! prints, logs or returns the key.

use std::collections::BTreeMap;
use std::io::IsTerminal;
// (process probes live in cli_support; no direct Command import needed)
use std::time::Duration;

use hiverouter_contract::{Refusal, Timestamp, Usage};
use hiverouter_provider::{
    run_cost_cap_usd, ApiKey, Model, PricingTable, RunAccounting, CALL_CAP, DEFAULT_BASE_URL,
    MAX_ATTEMPTS, TOKEN_BUDGET_COMBINED,
};
use serde::{Deserialize, Serialize};

use crate::census::{standard_census, PROMPT_CLAIMED_TOOL_IDS};
use crate::cli_support::{
    fetch_json, node_floor, parse_version_triple, rust_msrv_floor, version_probe,
};
use crate::tool::{AgentTier, ToolId, ToolRegistry};

/// CLI envelope schema id.
pub const CLI_SCHEMA: &str = "hiverouter.cli.v1";

/// Minimum Node version (FACT F3 / blueprint WP 1.10).
pub const NODE_FLOOR: &str = "22.12.0";

/// Minimum Rust version (FACT F9).
pub const MSRV: &str = "1.89.0";

/// Exit code: success.
pub const EXIT_OK: i32 = 0;
/// Exit code: usage / command-line error.
pub const EXIT_USAGE: i32 = 2;
/// Exit code: named refusal.
pub const EXIT_REFUSAL: i32 = 3;
/// Exit code: internal error.
pub const EXIT_INTERNAL: i32 = 4;

/// One CLI command declaration (single source of truth for help + schema).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct CommandSpec {
    /// Dotted command name (`resource.action`).
    pub name: &'static str,
    /// Resource group.
    pub resource: &'static str,
    /// Action name.
    pub action: &'static str,
    /// One-line summary.
    pub summary: &'static str,
    /// Safety tier: 0 read-only local, 1 read-only local HTTP, 2 mutating.
    pub tier: u8,
    /// Whether the command mutates state (none in v1).
    pub mutating: bool,
    /// Required positional arguments.
    pub positionals: &'static [&'static str],
    /// Accepted flags (besides globals).
    pub flags: &'static [&'static str],
}

/// Every CLI command, in progressive-disclosure order.
pub const COMMANDS: &[CommandSpec] = &[
    CommandSpec {
        name: "version",
        resource: "version",
        action: "",
        summary: "Print CLI/binary version and schema ids.",
        tier: 0,
        mutating: false,
        positionals: &[],
        flags: &[],
    },
    CommandSpec {
        name: "preflight",
        resource: "preflight",
        action: "",
        summary: "First-run dependency check: Node >=22.12.0, Rust MSRV 1.89.0, provider key presence.",
        tier: 0,
        mutating: false,
        positionals: &[],
        flags: &[],
    },
    CommandSpec {
        name: "schema",
        resource: "schema",
        action: "",
        summary: "List command schemas, or print one command schema.",
        tier: 0,
        mutating: false,
        positionals: &["command?"],
        flags: &[],
    },
    CommandSpec {
        name: "tools.list",
        resource: "tools",
        action: "list",
        summary: "List registered tools, permission classes and approval requirements.",
        tier: 0,
        mutating: false,
        positionals: &[],
        flags: &[],
    },
    CommandSpec {
        name: "tools.describe",
        resource: "tools",
        action: "describe",
        summary: "Print one tool contract (JSON Schema, tier allowlists).",
        tier: 0,
        mutating: false,
        positionals: &["tool_id"],
        flags: &[],
    },
    CommandSpec {
        name: "capabilities.census",
        resource: "capabilities",
        action: "census",
        summary: "Derived capability census (tool/refusal/tier counts from enums).",
        tier: 0,
        mutating: false,
        positionals: &[],
        flags: &[],
    },
    CommandSpec {
        name: "provider.cost",
        resource: "provider",
        action: "cost",
        summary: "Decimal cost for one usage sample at a UTC instant (F1 price rows).",
        tier: 0,
        mutating: false,
        positionals: &[],
        flags: &["--model", "--cache-hit", "--cache-miss", "--output", "--at"],
    },
    CommandSpec {
        name: "health",
        resource: "health",
        action: "",
        summary: "GET the local server /health endpoint (no mutation).",
        tier: 1,
        mutating: false,
        positionals: &[],
        flags: &["--url", "--timeout"],
    },
    CommandSpec {
        name: "diagnose",
        resource: "diagnose",
        action: "",
        summary: "GET the local server /diagnose endpoint (no secrets).",
        tier: 1,
        mutating: false,
        positionals: &[],
        flags: &["--url", "--timeout"],
    },
];

/// A named CLI failure.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CliFailure {
    /// Stable `E_*` code.
    pub code: &'static str,
    /// Human cause.
    pub message: String,
    /// How to fix it.
    pub how_to_fix: String,
    /// Whether a retry may succeed.
    pub retryable: bool,
}

/// One structured CLI outcome.
#[derive(Debug, Clone)]
pub struct CliOutcome {
    /// Process exit code.
    pub exit_code: i32,
    /// Machine payload (`{"schema":..,"ok":..,"command":..,...}`).
    pub payload: serde_json::Value,
    /// Human lines (printed to stderr only).
    pub human_lines: Vec<String>,
}

impl CliOutcome {
    fn success(command: &str, data: serde_json::Value, human: Vec<String>) -> Self {
        Self {
            exit_code: EXIT_OK,
            payload: serde_json::json!({
                "schema": CLI_SCHEMA,
                "ok": true,
                "command": command,
                "data": data,
                "meta": meta(command),
            }),
            human_lines: human,
        }
    }

    fn failure(command: &str, exit_code: i32, failure: CliFailure) -> Self {
        Self {
            exit_code,
            payload: serde_json::json!({
                "schema": CLI_SCHEMA,
                "ok": false,
                "command": command,
                "error": failure,
                "meta": meta(command),
            }),
            human_lines: vec![
                format!(
                    "{}: {} ({})",
                    failure.code, failure.message, failure.how_to_fix
                ),
                format!("retryable: {}", failure.retryable),
            ],
        }
    }

    /// Serialised JSON envelope.
    #[must_use]
    pub fn to_json(&self) -> String {
        serde_json::to_string(&self.payload)
            .unwrap_or_else(|_| format!("{{\"schema\":\"{CLI_SCHEMA}\",\"ok\":false}}"))
    }
}

fn meta(command: &str) -> serde_json::Value {
    serde_json::json!({
        "cli_schema": CLI_SCHEMA,
        "version": env!("CARGO_PKG_VERSION"),
        "command": command,
        "safety": safety_for(command),
    })
}

fn safety_for(command: &str) -> serde_json::Value {
    match COMMANDS.iter().find(|spec| spec.name == command) {
        Some(spec) => serde_json::json!({"tier": spec.tier, "mutating": spec.mutating}),
        None => serde_json::json!({"tier": 0, "mutating": false}),
    }
}

fn failure_from_refusal(refusal: Refusal) -> CliFailure {
    CliFailure {
        code: refusal.code(),
        message: refusal.message().to_owned(),
        how_to_fix: refusal.how_to_fix().to_owned(),
        retryable: refusal.retryable(),
    }
}

fn argv_error(command: &str, message: impl Into<String>) -> CliOutcome {
    CliOutcome::failure(
        command,
        EXIT_USAGE,
        CliFailure {
            code: Refusal::SchemaInvalid.code(),
            message: message.into(),
            how_to_fix: "Run `hiverouter --help` or `hiverouter schema <command>`.".to_owned(),
            retryable: false,
        },
    )
}

fn named_refusal(command: &str, refusal: Refusal, detail: impl Into<String>) -> CliOutcome {
    let mut failure = failure_from_refusal(refusal);
    let detail = detail.into();
    if !detail.is_empty() {
        failure.message = format!("{} ({detail})", failure.message);
    }
    CliOutcome::failure(command, EXIT_REFUSAL, failure)
}

/// Parsed global + command args.
#[derive(Debug, Default)]
struct ParsedArgs {
    positionals: Vec<String>,
    flags: BTreeMap<String, String>,
    format: Option<Format>,
    dry_run: bool,
    help: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Format {
    Json,
    Human,
}

fn parse_args(argv: &[String]) -> Result<ParsedArgs, String> {
    let mut parsed = ParsedArgs::default();
    let mut index = 0;
    while index < argv.len() {
        let arg = &argv[index];
        match arg.as_str() {
            "--help" | "-h" => parsed.help = true,
            "--dry-run" => parsed.dry_run = true,
            "--json" => parsed.format = Some(Format::Json),
            "--version" | "-V" => {
                parsed.positionals.push("version".to_owned());
            }
            other if other.starts_with("--format=") => {
                parsed.format = Some(match &other["--format=".len()..] {
                    "json" => Format::Json,
                    "human" => Format::Human,
                    other => return Err(format!("unknown --format value {other:?}")),
                });
            }
            other if other.starts_with("--") => {
                let (name, value) = match other.split_once('=') {
                    Some((name, value)) => (name.to_owned(), value.to_owned()),
                    None => {
                        let next = argv.get(index + 1).ok_or_else(|| {
                            format!("flag {other} needs a value (use {other}=<value>)")
                        })?;
                        index += 1;
                        (other.to_owned(), next.clone())
                    }
                };
                parsed.flags.insert(name, value);
            }
            other => parsed.positionals.push(other.to_owned()),
        }
        index += 1;
    }
    Ok(parsed)
}

fn command_name(parsed: &ParsedArgs) -> String {
    if parsed.positionals.is_empty() {
        return "help".to_owned();
    }
    let resource = &parsed.positionals[0];
    let action = parsed.positionals.get(1);
    match action {
        Some(action) => format!("{resource}.{action}"),
        None => resource.clone(),
    }
}

/// Dispatches one CLI invocation without touching stdout/stderr.
#[must_use]
pub fn dispatch(argv: &[String]) -> CliOutcome {
    let parsed = match parse_args(argv) {
        Ok(parsed) => parsed,
        Err(message) => return argv_error("help", message),
    };
    let command = command_name(&parsed);
    if parsed.help {
        return help_outcome(&command);
    }
    match command.as_str() {
        "help" => help_outcome("help"),
        "version" => version_outcome(),
        "preflight" => preflight_outcome(),
        "schema" => schema_outcome(&parsed),
        _ if parsed.positionals.first().map(String::as_str) == Some("schema") => schema_outcome(&parsed),
        "tools.list" => tools_list_outcome(),
        "tools.describe" => tools_describe_outcome(&parsed),
        "capabilities.census" => census_outcome(),
        "provider.cost" => provider_cost_outcome(&parsed),
        "health" => http_outcome(&parsed, "health", "/health"),
        "diagnose" => http_outcome(&parsed, "diagnose", "/diagnose"),
        _ => CliOutcome::failure(
            &command,
            EXIT_USAGE,
            failure_from_refusal(Refusal::CommandUnknown),
        ),
    }
}

/// Runs the CLI, printing machine JSON to stdout and human lines to stderr.
/// Returns the process exit code.
#[must_use]
pub fn run_main(argv: Vec<String>) -> i32 {
    let parsed = parse_args(&argv).ok();
    let format = parsed
        .as_ref()
        .and_then(|parsed| parsed.format)
        .unwrap_or(if std::io::stdout().is_terminal() {
            Format::Human
        } else {
            Format::Json
        });
    let outcome = dispatch(&argv);
    match format {
        Format::Json => println!("{}", outcome.to_json()),
        Format::Human => {
            for line in &outcome.human_lines {
                eprintln!("{line}");
            }
        }
    }
    outcome.exit_code
}

fn help_outcome(command: &str) -> CliOutcome {
    if command == "help" {
        let resources: Vec<&'static str> = {
            let mut seen = Vec::new();
            for spec in COMMANDS {
                if !seen.contains(&spec.resource) {
                    seen.push(spec.resource);
                }
            }
            seen
        };
        let human = vec![
            "hiverouter — HiveMind v1 local control CLI (agent-native)".to_owned(),
            "Usage: hiverouter <resource> <action> [flags]   |   hiverouter --help".to_owned(),
            format!("Resources: {}", resources.join(", ")),
            "Discovery: `hiverouter schema` lists every command; `hiverouter schema <command>` returns its schema.".to_owned(),
            "Output: JSON on stdout by default (non-TTY); --format json|human; human output goes to stderr.".to_owned(),
            "Exit codes: 0 ok, 2 usage, 3 named refusal, 4 internal.".to_owned(),
        ];
        return CliOutcome::success(
            "help",
            serde_json::json!({
                "resources": resources,
                "commands": COMMANDS,
                "global_flags": ["--format=json|human", "--json", "--dry-run", "--help", "--timeout=<secs>", "--url=<url>"],
                "exit_codes": {"ok": EXIT_OK, "usage": EXIT_USAGE, "refusal": EXIT_REFUSAL, "internal": EXIT_INTERNAL},
            }),
            human,
        );
    }
    let resource = command.split('.').next().unwrap_or(command);
    let actions: Vec<&CommandSpec> = COMMANDS
        .iter()
        .filter(|spec| spec.resource == resource)
        .collect();
    if actions.is_empty() {
        return CliOutcome::failure(
            command,
            EXIT_USAGE,
            failure_from_refusal(Refusal::CommandUnknown),
        );
    }
    let human = actions
        .iter()
        .map(|spec| format!("hiverouter {} — {} (tier {}, mutating: {})", spec.name, spec.summary, spec.tier, spec.mutating))
        .collect();
    CliOutcome::success(
        command,
        serde_json::json!({"resource": resource, "actions": actions}),
        human,
    )
}

fn version_outcome() -> CliOutcome {
    let data = serde_json::json!({
        "version": env!("CARGO_PKG_VERSION"),
        "cli_schema": CLI_SCHEMA,
        "health_schema": crate::health::HEALTH_SCHEMA,
        "diagnose_schema": crate::health::DIAGNOSE_SCHEMA,
        "contract_schema": hiverouter_contract::SCHEMA_VERSION,
        "provider_version": hiverouter_provider::PROVIDER_VERSION,
        "target": format!("{}-{}", std::env::consts::ARCH, std::env::consts::OS),
    });
    CliOutcome::success("version", data, vec![format!("hiverouter {}", env!("CARGO_PKG_VERSION"))])
}

fn preflight_outcome() -> CliOutcome {
    let node = version_probe("node");
    let rustc = version_probe("rustc");
    let key_present = ApiKey::present_in_env();

    let node_info = node.as_ref().ok().map(|(program, version)| (program.clone(), version.clone()));
    let mut checks = Vec::new();
    let node_ok = match &node_info {
        Some((_, version)) => match parse_version_triple(version.trim_start_matches('v')) {
            Some(triple) => triple >= node_floor(),
            None => false,
        },
        None => false,
    };
    checks.push(serde_json::json!({
        "name": "node",
        "ok": node_ok,
        "required": format!(">={NODE_FLOOR}"),
        "observed": node_info.as_ref().map(|(_, version)| version.clone()),
        "code": if node_ok { serde_json::Value::Null } else { serde_json::Value::String(Refusal::NodeMissing.code().to_owned()) },
    }));

    let rust_info = rustc
        .as_ref()
        .ok()
        .map(|(program, version)| (program.clone(), version.clone()));
    let rust_ok = match &rust_info {
        Some((_, version)) => version
            .split_whitespace()
            .nth(1)
            .and_then(parse_version_triple)
            .is_some_and(|triple| triple >= rust_msrv_floor()),
        None => false,
    };
    checks.push(serde_json::json!({
        "name": "rustc",
        "ok": rust_ok,
        "required": format!(">={MSRV}"),
        "observed": rust_info.as_ref().map(|(_, version)| version.clone()),
        "code": if rust_ok { serde_json::Value::Null } else { serde_json::Value::String(Refusal::ToolchainMsrv.code().to_owned()) },
    }));
    checks.push(serde_json::json!({
        "name": "deepseek_api_key",
        "ok": true,
        "present": key_present,
        "required": "present for live provider calls",
        "observed": if key_present { "present" } else { "absent" },
        "code": if key_present { serde_json::Value::Null } else { serde_json::Value::String(Refusal::KeyMissing.code().to_owned()) },
        "blocking": false,
    }));

    let ready = node_ok && rust_ok;
    let data = serde_json::json!({
        "ready": ready,
        "checks": checks,
        "warnings": if key_present { Vec::<String>::new() } else { vec![Refusal::KeyMissing.code().to_owned()] },
        "versions": {
            "cli": env!("CARGO_PKG_VERSION"),
            "rustc": rust_info.as_ref().map(|(_, version)| version.clone()),
            "node": node_info.as_ref().map(|(_, version)| version.clone()),
        },
    });
    let human = vec![
        format!("node: {} ({})", if node_ok { "ok" } else { "MISSING/TOO OLD" }, NODE_FLOOR),
        format!("rustc: {} ({})", if rust_ok { "ok" } else { "MISSING/TOO OLD" }, MSRV),
        format!("DEEPSEEK_API_KEY present: {key_present}"),
    ];
    if ready {
        CliOutcome::success("preflight", data, human)
    } else {
        let refusal = if node_ok {
            Refusal::ToolchainMsrv
        } else {
            Refusal::NodeMissing
        };
        let mut outcome = CliOutcome::failure("preflight", EXIT_REFUSAL, failure_from_refusal(refusal));
        outcome.payload["data"] = data;
        outcome
    }
}

fn schema_outcome(parsed: &ParsedArgs) -> CliOutcome {
    match parsed.positionals.get(1) {
        None => {
            let commands: Vec<serde_json::Value> = COMMANDS
                .iter()
                .map(|spec| serde_json::to_value(spec).expect("command spec serialises"))
                .collect();
            CliOutcome::success(
                "schema",
                serde_json::json!({"schema": CLI_SCHEMA, "commands": commands}),
                vec![format!("{} commands; use `hiverouter schema <command>`", COMMANDS.len())],
            )
        }
        Some(name) => match COMMANDS.iter().find(|spec| spec.name == name) {
            Some(spec) => CliOutcome::success(
                "schema",
                serde_json::json!({
                    "schema": CLI_SCHEMA,
                    "command": serde_json::to_value(spec).expect("spec"),
                    "request": {"argv": ["hiverouter", spec.name], "flags": spec.flags},
                    "response": {"envelope": {"schema": CLI_SCHEMA, "ok": "bool", "command": "string", "data": "object", "meta": "object"}},
                    "error": {"envelope": {"ok": false, "error": {"code": Refusal::SchemaInvalid.code(), "message": "string", "how_to_fix": "string", "retryable": "bool"}}},
                    "exit_codes": {"ok": EXIT_OK, "usage": EXIT_USAGE, "refusal": EXIT_REFUSAL, "internal": EXIT_INTERNAL},
                }),
                vec![format!("hiverouter {} — {}", spec.name, spec.summary)],
            ),
            None => CliOutcome::failure(
                "schema",
                EXIT_USAGE,
                failure_from_refusal(Refusal::CommandUnknown),
            ),
        },
    }
}

fn tools_registry() -> ToolRegistry {
    ToolRegistry::standard()
}

fn tools_list_outcome() -> CliOutcome {
    let registry = tools_registry();
    let tools: Vec<serde_json::Value> = ToolId::all()
        .into_iter()
        .filter_map(|id| registry.contract(id))
        .map(|contract| {
            serde_json::json!({
                "id": contract.id.id(),
                "description": contract.description,
                "permission_class": contract.permission_class.label(),
                "requires_approval": contract.requires_approval(),
            })
        })
        .collect();
    CliOutcome::success(
        "tools.list",
        serde_json::json!({"count": tools.len(), "tools": tools}),
        vec![format!("{} registered tools", tools.len())],
    )
}

fn tools_describe_outcome(parsed: &ParsedArgs) -> CliOutcome {
    let Some(tool_name) = parsed.positionals.get(2) else {
        return argv_error("tools.describe", "missing <tool_id>");
    };
    let Some(id) = ToolId::from_id(tool_name) else {
        return named_refusal(
            "tools.describe",
            Refusal::ToolNotFound,
            format!("no tool {tool_name:?} in the frozen vocabulary"),
        );
    };
    let registry = tools_registry();
    let Some(contract) = registry.contract(id) else {
        return named_refusal("tools.describe", Refusal::ToolNotFound, "no contract");
    };
    let allowlists: serde_json::Value = AgentTier::all()
        .into_iter()
        .map(|tier| (tier.label().to_owned(), registry.is_allowed(tier, id)))
        .collect();
    let data = serde_json::json!({
        "id": contract.id.id(),
        "description": contract.description,
        "permission_class": contract.permission_class.label(),
        "requires_approval": contract.requires_approval(),
        "input_schema": contract.schema,
        "allowed_for_tiers": allowlists,
    });
    CliOutcome::success(
        "tools.describe",
        data,
        vec![format!("{} ({} permission)", id.id(), contract.permission_class.label())],
    )
}

fn census_outcome() -> CliOutcome {
    match standard_census() {
        Ok(census) => {
            let refusals = serde_json::json!({
                "count": Refusal::ALL.len(),
                "codes": Refusal::ALL.iter().map(|refusal| refusal.code()).collect::<Vec<_>>(),
            });
            let data = serde_json::json!({
                "census": census,
                "refusals": refusals,
                "prompt_claimed": PROMPT_CLAIMED_TOOL_IDS,
            });
            CliOutcome::success(
                "capabilities.census",
                data,
                vec![format!(
                    "tools={} refusals={} tiers={} classes={} grants={}",
                    census.tool_count,
                    census.refusal_count,
                    census.tier_count,
                    census.permission_class_count,
                    census.grant_count()
                )],
            )
        }
        Err(error) => named_refusal("capabilities.census", Refusal::InternalError, error.detail()),
    }
}

fn int_flag(parsed: &ParsedArgs, name: &str) -> Result<u64, CliOutcome> {
    let raw = parsed
        .flags
        .get(name)
        .ok_or_else(|| argv_error("provider.cost", format!("missing {name}")))?;
    raw.parse::<u64>().map_err(|_| {
        argv_error(
            "provider.cost",
            format!("{name} must be a non-negative integer, got {raw:?}"),
        )
    })
}

fn provider_cost_outcome(parsed: &ParsedArgs) -> CliOutcome {
    let model_id = parsed
        .flags
        .get("--model")
        .map(String::as_str)
        .unwrap_or("flash");
    let Some(model) = Model::from_id(match model_id {
        "flash" => "deepseek-flash",
        "pro" => "deepseek-v4-pro",
        other => other,
    }) else {
        return argv_error(
            "provider.cost",
            format!("unknown model {model_id:?}; expected flash or pro"),
        );
    };
    let hit = match int_flag(parsed, "--cache-hit") {
        Ok(value) => value,
        Err(outcome) => return outcome,
    };
    let miss = match int_flag(parsed, "--cache-miss") {
        Ok(value) => value,
        Err(outcome) => return outcome,
    };
    let output = match int_flag(parsed, "--output") {
        Ok(value) => value,
        Err(outcome) => return outcome,
    };
    let at = parsed
        .flags
        .get("--at")
        .map(String::as_str)
        .unwrap_or("2026-11-06T00:00:00Z");
    let at_unix = match Timestamp::parse(at) {
        Ok(timestamp) => match timestamp.unix_secs() {
            Ok(secs) => secs,
            Err(error) => return argv_error("provider.cost", format!("invalid --at: {error}")),
        },
        Err(error) => return argv_error("provider.cost", format!("invalid --at: {error}")),
    };
    let usage = Usage {
        prompt_tokens: Some(hit.saturating_add(miss)),
        completion_tokens: Some(output),
        total_tokens: Some(hit.saturating_add(miss).saturating_add(output)),
        prompt_cache_hit_tokens: Some(hit),
        prompt_cache_miss_tokens: Some(miss),
        reasoning_tokens: None,
    };
    let mut accounting = RunAccounting::new(PricingTable::documented());
    match accounting.record(model, Some(&usage), at_unix) {
        Ok(call) => {
            let data = serde_json::json!({
                "model": model.id(),
                "usage": usage,
                "at": at,
                "peak": call.peak.label(),
                "status": accounting.report().status,
                "settled_usd": accounting.report().settled_usd,
                "report": accounting.report(),
                "provenance": {
                    "usage": call.usage_provenance.label(),
                    "amount": call.amount_provenance.label(),
                    "price_source_url": hiverouter_provider::pricing::PRICE_SOURCE_URL,
                    "price_fetched_at": hiverouter_provider::pricing::PRICE_FETCHED_AT,
                },
                "dry_run": parsed.dry_run,
            });
            CliOutcome::success(
                "provider.cost",
                data,
                vec![format!(
                    "{} {} at {}: ${} ({})",
                    model.id(),
                    call.peak.label(),
                    at,
                    accounting.report().settled_usd.unwrap_or_else(|| "null".to_owned()),
                    accounting.report().status
                )],
            )
        }
        Err(error) => named_refusal("provider.cost", error.refusal(), error.detail()),
    }
}

fn http_outcome(parsed: &ParsedArgs, command: &str, path: &str) -> CliOutcome {
    let base_url = parsed
        .flags
        .get("--url")
        .cloned()
        .unwrap_or_else(|| "http://127.0.0.1:8080".to_owned());
    let timeout_secs: u64 = parsed
        .flags
        .get("--timeout")
        .and_then(|value| value.parse().ok())
        .unwrap_or(10);
    let url = format!("{}{}", base_url.trim_end_matches('/'), path);
    if parsed.dry_run {
        return CliOutcome::success(
            command,
            serde_json::json!({
                "dry_run": true,
                "request": {"method": "GET", "url": url, "timeout_secs": timeout_secs},
            }),
            vec![format!("dry-run: would GET {url}")],
        );
    }
    match fetch_json(&url, Duration::from_secs(timeout_secs)) {
        Ok(value) => {
            let status = value
                .get("status")
                .and_then(|status| status.as_str())
                .unwrap_or("unknown")
                .to_owned();
            CliOutcome::success(
                command,
                serde_json::json!({"url": url, "response": value}),
                vec![format!("{command}: {status} ({url})")],
            )
        }
        Err(detail) => named_refusal(command, Refusal::WsDisconnected, detail),
    }
}

/// The cost/price constants exposed by the CLI schema (documented, not typed).
#[must_use]
pub fn provider_limits() -> serde_json::Value {
    serde_json::json!({
        "base_url": DEFAULT_BASE_URL,
        "run_cost_cap_usd": run_cost_cap_usd().to_string(),
        "token_budget_combined": TOKEN_BUDGET_COMBINED,
        "call_cap": CALL_CAP,
        "max_attempts": MAX_ATTEMPTS,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn argv(parts: &[&str]) -> Vec<String> {
        parts.iter().map(|part| (*part).to_owned()).collect()
    }

    fn dispatch_json(parts: &[&str]) -> (i32, serde_json::Value) {
        let outcome = dispatch(&argv(parts));
        (outcome.exit_code, outcome.payload)
    }

    #[test]
    fn version_is_ok_json() {
        let (code, payload) = dispatch_json(&["version", "--format=json"]);
        assert_eq!(code, EXIT_OK);
        assert_eq!(payload["schema"], CLI_SCHEMA);
        assert_eq!(payload["ok"], true);
        assert_eq!(payload["command"], "version");
        assert_eq!(payload["data"]["version"], env!("CARGO_PKG_VERSION"));
        assert_eq!(payload["data"]["contract_schema"], "hiverouter.v1");
        assert_eq!(payload["meta"]["safety"]["mutating"], false);
    }

    #[test]
    fn unknown_command_is_usage_error_with_named_code() {
        let (code, payload) = dispatch_json(&["frobnicate", "--format=json"]);
        assert_eq!(code, EXIT_USAGE);
        assert_eq!(payload["ok"], false);
        assert_eq!(payload["error"]["code"], "E_COMMAND_UNKNOWN");
        assert!(!payload["error"]["how_to_fix"].as_str().expect("fix").is_empty());
    }

    #[test]
    fn schema_is_generated_from_the_single_command_table() {
        let (code, payload) = dispatch_json(&["schema", "--format=json"]);
        assert_eq!(code, EXIT_OK);
        let commands = payload["data"]["commands"].as_array().expect("commands");
        assert_eq!(commands.len(), COMMANDS.len());
        for spec in COMMANDS {
            assert!(commands.iter().any(|row| row["name"] == spec.name));
            let (code, one) = dispatch_json(&["schema", spec.name, "--format=json"]);
            assert_eq!(code, EXIT_OK);
            assert_eq!(one["data"]["command"]["name"], spec.name);
            assert_eq!(one["data"]["command"]["tier"], spec.tier);
        }
    }

    #[test]
    fn help_is_progressive() {
        let (code, payload) = dispatch_json(&["--help"]);
        assert_eq!(code, EXIT_OK);
        assert!(payload["data"]["resources"].as_array().expect("resources").len() >= 7);
        let (code, tools_help) = dispatch_json(&["tools", "--help"]);
        assert_eq!(code, EXIT_OK);
        assert_eq!(tools_help["data"]["resource"], "tools");
        assert!(tools_help["data"]["actions"].as_array().expect("actions").len() >= 2);
    }

    #[test]
    fn preflight_reports_checks_and_never_returns_the_key() {
        let (code, payload) = dispatch_json(&["preflight", "--format=json"]);
        assert!(code == EXIT_OK || code == EXIT_REFUSAL);
        let checks = payload["data"]["checks"].as_array().expect("checks");
        assert!(checks.len() >= 3);
        assert!(checks.iter().any(|check| check["name"] == "node"));
        assert!(checks.iter().any(|check| check["name"] == "rustc"));
        let all = payload.to_string();
        assert!(!all.contains("sk-"), "preflight never prints key material");
        assert!(payload["data"]["versions"]["cli"].is_string());
    }

    #[test]
    fn tools_list_count_is_derived_from_the_enum() {
        let (code, payload) = dispatch_json(&["tools", "list", "--format=json"]);
        assert_eq!(code, EXIT_OK);
        assert_eq!(
            payload["data"]["count"].as_u64().expect("count"),
            u64::try_from(ToolId::all().len()).expect("fits")
        );
        let tools = payload["data"]["tools"].as_array().expect("tools");
        assert!(tools.iter().any(|tool| tool["id"] == "shell_exec"
            && tool["requires_approval"] == true));
        assert!(tools.iter().any(|tool| tool["id"] == "file_read"
            && tool["permission_class"] == "read"));
    }

    #[test]
    fn tools_describe_returns_schema_and_unknown_tool_refuses_by_name() {
        let (code, payload) = dispatch_json(&["tools", "describe", "file_write", "--format=json"]);
        assert_eq!(code, EXIT_OK);
        assert_eq!(payload["data"]["input_schema"]["type"], "object");
        assert_eq!(payload["data"]["requires_approval"], true);
        assert_eq!(payload["data"]["allowed_for_tiers"]["hyper_agent"], false);

        let (code, payload) = dispatch_json(&["tools", "describe", "rm_rf", "--format=json"]);
        assert_eq!(code, EXIT_REFUSAL);
        assert_eq!(payload["error"]["code"], "E_TOOL_NOT_FOUND");
    }

    #[test]
    fn census_command_matches_derived_counts() {
        let (code, payload) = dispatch_json(&["capabilities", "census", "--format=json"]);
        assert_eq!(code, EXIT_OK);
        assert_eq!(
            payload["data"]["census"]["tool_count"].as_u64().expect("tools"),
            u64::try_from(ToolId::all().len()).expect("fits")
        );
        assert_eq!(
            payload["data"]["refusals"]["count"].as_u64().expect("refusals"),
            u64::try_from(Refusal::ALL.len()).expect("fits")
        );
    }

    #[test]
    fn provider_cost_is_peak_aware_and_decimal() {
        let off_peak = dispatch_json(&[
            "provider",
            "cost",
            "--model=flash",
            "--cache-hit=1",
            "--cache-miss=1",
            "--output=1",
            "--at=2026-11-06T00:00:00Z",
            "--format=json",
        ]);
        assert_eq!(off_peak.0, EXIT_OK);
        assert_eq!(off_peak.1["data"]["peak"], "off_peak");
        let peak = dispatch_json(&[
            "provider",
            "cost",
            "--model=flash",
            "--cache-hit=1000000",
            "--cache-miss=1000000",
            "--output=1000000",
            "--at=2026-11-06T01:00:00Z",
            "--format=json",
        ]);
        assert_eq!(peak.0, EXIT_OK);
        assert_eq!(peak.1["data"]["peak"], "peak");
        assert_eq!(peak.1["data"]["settled_usd"], "1.506");
        assert_eq!(peak.1["data"]["status"], "COMPLETE");
        assert_eq!(peak.1["data"]["provenance"]["amount"], "derived");

        let missing_flag = dispatch_json(&["provider", "cost", "--format=json"]);
        assert_eq!(missing_flag.0, EXIT_USAGE);
        assert_eq!(missing_flag.1["error"]["code"], "E_SCHEMA_INVALID");
    }

    #[test]
    fn health_and_diagnose_support_dry_run_without_network() {
        let (code, payload) = dispatch_json(&[
            "health",
            "--url=http://127.0.0.1:1",
            "--timeout=1",
            "--dry-run",
            "--format=json",
        ]);
        assert_eq!(code, EXIT_OK);
        assert_eq!(payload["data"]["dry_run"], true);
        assert_eq!(payload["data"]["request"]["method"], "GET");
        assert_eq!(payload["data"]["request"]["url"], "http://127.0.0.1:1/health");
    }

    #[test]
    fn unreachable_local_server_is_a_named_refusal() {
        let (code, payload) = dispatch_json(&[
            "diagnose",
            "--url=http://127.0.0.1:1",
            "--timeout=1",
            "--format=json",
        ]);
        assert_eq!(code, EXIT_REFUSAL);
        assert_eq!(payload["error"]["code"], "E_WS_DISCONNECTED");
        assert!(payload["error"]["retryable"].as_bool().expect("retryable"));
    }

    #[test]
    fn bad_global_flag_is_usage_error() {
        let (code, payload) = dispatch_json(&["version", "--format=yaml"]);
        assert_eq!(code, EXIT_USAGE);
        assert_eq!(payload["error"]["code"], "E_SCHEMA_INVALID");
    }

    #[test]
    fn json_envelope_is_always_parseable() {
        let outcome = dispatch(&argv(&["nope"]));
        let parsed: serde_json::Value = serde_json::from_str(&outcome.to_json()).expect("json");
        assert_eq!(parsed["schema"], CLI_SCHEMA);
        assert_eq!(parsed["ok"], false);
    }
}

