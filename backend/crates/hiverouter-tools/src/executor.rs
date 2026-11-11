//! Read-only tool executor for the 1.2 model tool loop (W12 interface).
//!
//! `file_read`, `file_list`, `report_result` and `finish` may execute through
//! this module; the two terminal-result tools carry no side effect and return
//! their structured claim as text. Every other tool name is a named policy
//! refusal [`Refusal::ToolNotAllowed`]. Side effects stay owned by the
//! approval/loop layers. The tool loop maps an [`ExecutedTool`] to `TOOL_RESULT`
//! and must never treat a refusal as success.
//!
//! The tier allowlist and argument schema are enforced by
//! [`ToolRegistry::resolve_call`] before anything executes; a tier without the
//! grant gets `E_TOOL_NOT_ALLOWED` and nothing is read from disk.

use hiverouter_contract::Refusal;

use crate::error::ToolError;
use crate::fs_search::{self, GlobRequest, GrepRequest, ReadRangeRequest};
use crate::jail::WorkspaceJail;
use crate::tool::{
    AgentTier, FileListArgs, FileReadArgs, FileReadRangeArgs, FinishArgs, FsGlobArgs, FsGrepArgs,
    ReportResultArgs, ToolId, ToolRegistry,
};

/// Result of routing one model-originated read-only tool call.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExecutedTool {
    /// The tool executed and produced bounded text (the caller caps bytes).
    Ok {
        /// Text the loop may return as `TOOL_RESULT.text` (file content or a
        /// deterministic JSON listing).
        text: String,
    },
    /// The tool did not execute; the model receives a named refusal.
    Refused {
        /// Stable `E_*` code (never fabricated; from [`Refusal::code`]).
        code: String,
        /// Non-secret detail for the ledger / model-facing explanation.
        detail: String,
    },
}

/// Executes only `file_read` / `file_list` through the workspace jail.
///
/// # Errors
/// - [`Refusal::ToolNotFound`] / [`Refusal::ToolNotAllowed`] / [`Refusal::ToolSchema`]
///   from the registry for a read tool whose tier/arguments are invalid;
/// - [`Refusal::PathOutsideJail`], [`Refusal::PathNotFound`],
///   [`Refusal::FileTooLarge`] or [`Refusal::SchemaInvalid`] from the jail
///   (including non-UTF-8 content: `TOOL_RESULT.text` is text, never binary).
///
/// Every non-read-only tool returns `Ok(ExecutedTool::Refused)` with
/// `E_TOOL_NOT_ALLOWED` and performs no I/O.
pub fn execute_read_only(
    jail: &WorkspaceJail,
    registry: &ToolRegistry,
    tier: AgentTier,
    tool: &str,
    args: &serde_json::Value,
) -> Result<ExecutedTool, ToolError> {
    let Some(id) = ToolId::from_id(tool) else {
        return Ok(ExecutedTool::Refused {
            code: Refusal::ToolNotAllowed.code().to_owned(),
            detail: format!(
                "tool {tool:?} is not executable by the read-only executor; \
                 side-effecting tools are approval/loop-owned"
            ),
        });
    };
    let Some(known) = registry.contract(id) else {
        return Ok(ExecutedTool::Refused {
            code: Refusal::ToolNotAllowed.code().to_owned(),
            detail: format!(
                "tool {tool:?} is not executable by the read-only executor; \
                 side-effecting tools are approval/loop-owned"
            ),
        });
    };
    if !matches!(known.permission_class, crate::tool::PermissionClass::Read) {
        return Ok(ExecutedTool::Refused {
            code: Refusal::ToolNotAllowed.code().to_owned(),
            detail: format!(
                "tool {tool:?} is side-effecting or control-plane; the read-only executor \
                 performs no I/O and the approval/loop owner must execute it"
            ),
        });
    }
    // Registry first: unknown id, missing tier grant or invalid args never
    // reach an executor (fail closed before any filesystem access).
    let contract = registry.resolve_call(tier, tool, args)?;
    match contract.id {
        ToolId::FileRead => execute_file_read(jail, args),
        ToolId::FileList => execute_file_list(jail, args),
        ToolId::ReportResult => execute_report_result(args),
        ToolId::Finish => execute_finish(args),
        ToolId::FsGlob => execute_fs_glob(jail, args),
        ToolId::FsGrep => execute_fs_grep(jail, args),
        ToolId::FileReadRange => execute_file_read_range(jail, args),
        _ => Ok(ExecutedTool::Refused {
            code: Refusal::ToolNotAllowed.code().to_owned(),
            detail: format!(
                "tool {tool:?} is not executable by the read-only filesystem executor; \
                 side-effecting or process-owned tools are approval/loop-owned"
            ),
        }),
    }
}

fn execute_fs_glob(
    jail: &WorkspaceJail,
    args: &serde_json::Value,
) -> Result<ExecutedTool, ToolError> {
    let parsed: FsGlobArgs = serde_json::from_value(args.clone()).map_err(|error| {
        ToolError::new(
            Refusal::SchemaInvalid,
            format!("fs_glob arguments do not deserialize after schema validation: {error}"),
        )
    })?;
    let mut request = GlobRequest::new(parsed.pattern);
    if let Some(root) = parsed.root {
        request = request.with_root(root);
    }
    if let Some(max_results) = parsed.max_results {
        request = request.with_max_results(max_results);
    }
    let result = fs_search::glob(jail, &request)?;
    let text = serde_json::to_string(&result).map_err(|error| {
        ToolError::new(
            Refusal::InternalError,
            format!("could not serialize fs_glob result: {error}"),
        )
    })?;
    Ok(ExecutedTool::Ok { text })
}

fn execute_fs_grep(
    jail: &WorkspaceJail,
    args: &serde_json::Value,
) -> Result<ExecutedTool, ToolError> {
    let parsed: FsGrepArgs = serde_json::from_value(args.clone()).map_err(|error| {
        ToolError::new(
            Refusal::SchemaInvalid,
            format!("fs_grep arguments do not deserialize after schema validation: {error}"),
        )
    })?;
    let mut request = GrepRequest::new(parsed.pattern);
    if let Some(root) = parsed.root {
        request = request.with_root(root);
    }
    if let Some(glob) = parsed.glob {
        request = request.with_glob(glob);
    }
    if let Some(case_sensitive) = parsed.case_sensitive {
        request = request.with_case_sensitive(case_sensitive);
    }
    if let Some(max_matches) = parsed.max_matches {
        request = request.with_max_matches(max_matches);
    }
    let result = fs_search::grep(jail, &request)?;
    let text = serde_json::to_string(&result).map_err(|error| {
        ToolError::new(
            Refusal::InternalError,
            format!("could not serialize fs_grep result: {error}"),
        )
    })?;
    Ok(ExecutedTool::Ok { text })
}

fn execute_file_read_range(
    jail: &WorkspaceJail,
    args: &serde_json::Value,
) -> Result<ExecutedTool, ToolError> {
    let parsed: FileReadRangeArgs = serde_json::from_value(args.clone()).map_err(|error| {
        ToolError::new(
            Refusal::SchemaInvalid,
            format!("file_read_range arguments do not deserialize after schema validation: {error}"),
        )
    })?;
    let request = ReadRangeRequest::new(parsed.path, parsed.start_line, parsed.line_count);
    let result = fs_search::read_range(jail, &request)?;
    let text = serde_json::to_string(&result).map_err(|error| {
        ToolError::new(
            Refusal::InternalError,
            format!("could not serialize file_read_range result: {error}"),
        )
    })?;
    Ok(ExecutedTool::Ok { text })
}

/// A terminal-result report from the model. It carries no side effect and is
/// returned as text so the run ledger/event and the next turn see the claim.
fn execute_report_result(args: &serde_json::Value) -> Result<ExecutedTool, ToolError> {
    let report: ReportResultArgs = serde_json::from_value(args.clone()).map_err(|error| {
        ToolError::new(
            Refusal::SchemaInvalid,
            format!("report_result arguments do not deserialize after schema validation: {error}"),
        )
    })?;
    Ok(ExecutedTool::Ok {
        text: serde_json::json!({
            "summary": report.summary,
            "success": report.success,
        })
        .to_string(),
    })
}

/// A finish verdict from the model. It is a read-class terminal result; the
/// engine still owns the actual terminal state transition.
fn execute_finish(args: &serde_json::Value) -> Result<ExecutedTool, ToolError> {
    let finish: FinishArgs = serde_json::from_value(args.clone()).map_err(|error| {
        ToolError::new(
            Refusal::SchemaInvalid,
            format!("finish arguments do not deserialize after schema validation: {error}"),
        )
    })?;
    Ok(ExecutedTool::Ok {
        text: serde_json::json!({"verdict": finish.verdict}).to_string(),
    })
}

fn execute_file_read(
    jail: &WorkspaceJail,
    args: &serde_json::Value,
) -> Result<ExecutedTool, ToolError> {
    let read: FileReadArgs = serde_json::from_value(args.clone()).map_err(|error| {
        ToolError::new(
            Refusal::SchemaInvalid,
            format!("file_read arguments do not deserialize after schema validation: {error}"),
        )
    })?;
    let bytes = jail.read(&read.path)?;
    let text = String::from_utf8(bytes).map_err(|_| {
        ToolError::new(
            Refusal::SchemaInvalid,
            "file is not valid UTF-8; the tool result text carries text and refuses binary content",
        )
    })?;
    Ok(ExecutedTool::Ok { text })
}

fn execute_file_list(
    jail: &WorkspaceJail,
    args: &serde_json::Value,
) -> Result<ExecutedTool, ToolError> {
    let list: FileListArgs = serde_json::from_value(args.clone()).map_err(|error| {
        ToolError::new(
            Refusal::SchemaInvalid,
            format!("file_list arguments do not deserialize after schema validation: {error}"),
        )
    })?;
    let entries = jail.list(list.path.as_deref())?;
    let listing = entries
        .into_iter()
        .map(|entry| {
            serde_json::json!({
                "name": entry.name,
                "is_dir": entry.is_dir,
                "size": entry.size,
            })
        })
        .collect::<Vec<_>>();
    let text = serde_json::to_string(&listing).map_err(|error| {
        ToolError::new(
            Refusal::InternalError,
            format!("cannot serialise the file_list result: {error}"),
        )
    })?;
    Ok(ExecutedTool::Ok { text })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tool::{PermissionClass, ToolId};

    fn jail() -> (tempfile::TempDir, WorkspaceJail) {
        let temp = tempfile::tempdir().expect("tempdir");
        std::fs::write(temp.path().join("note.txt"), b"hello tool loop").expect("write note");
        std::fs::create_dir(temp.path().join("sub")).expect("mkdir");
        std::fs::write(temp.path().join("sub").join("inner.txt"), b"inner").expect("write inner");
        let jail = WorkspaceJail::open(temp.path()).expect("jail");
        (temp, jail)
    }

    fn registry() -> ToolRegistry {
        ToolRegistry::standard()
    }

    #[test]
    fn file_read_executes_through_the_jail() {
        let (_temp, jail) = jail();
        let outcome = execute_read_only(
            &jail,
            &registry(),
            AgentTier::WorkerAgent,
            "file_read",
            &serde_json::json!({"path": "note.txt"}),
        )
        .expect("read executes");
        assert_eq!(
            outcome,
            ExecutedTool::Ok {
                text: "hello tool loop".to_owned()
            }
        );
    }

    #[test]
    fn file_list_returns_deterministic_json() {
        let (_temp, jail) = jail();
        let outcome = execute_read_only(
            &jail,
            &registry(),
            AgentTier::WorkerAgent,
            "file_list",
            &serde_json::json!({}),
        )
        .expect("list executes");
        let ExecutedTool::Ok { text } = outcome else {
            panic!("root listing must execute");
        };
        let parsed: serde_json::Value = serde_json::from_str(&text).expect("valid JSON listing");
        let entries = parsed.as_array().expect("array");
        assert_eq!(entries.len(), 2, "note.txt + sub");
        assert_eq!(entries[0]["name"].as_str(), Some("note.txt"));
        assert_eq!(entries[0]["is_dir"].as_bool(), Some(false));
        assert_eq!(entries[0]["size"].as_u64(), Some(15));
        assert_eq!(entries[1]["name"].as_str(), Some("sub"));
        assert_eq!(entries[1]["is_dir"].as_bool(), Some(true));
        assert!(entries[1]["size"].is_number(), "directory size is a number");

        let nested = execute_read_only(
            &jail,
            &registry(),
            AgentTier::WorkerAgent,
            "file_list",
            &serde_json::json!({"path": "sub"}),
        )
        .expect("nested list executes");
        let ExecutedTool::Ok { text } = nested else {
            panic!("nested listing must execute");
        };
        let parsed: serde_json::Value = serde_json::from_str(&text).expect("valid JSON");
        assert_eq!(parsed.as_array().expect("array").len(), 1);
        assert_eq!(parsed[0]["name"].as_str(), Some("inner.txt"));
    }

    #[test]
    fn fs_glob_grep_and_read_range_execute_without_side_effects() {
        let (_temp, jail) = jail();
        let registry = registry();
        let glob = execute_read_only(
            &jail,
            &registry,
            AgentTier::WorkerAgent,
            "fs_glob",
            &serde_json::json!({"pattern": "*.txt"}),
        )
        .expect("glob executes");
        let ExecutedTool::Ok { text } = glob else {
            panic!("fs_glob must execute, got {glob:?}");
        };
        assert!(text.contains("note.txt"), "{text}");
        let range = execute_read_only(
            &jail,
            &registry,
            AgentTier::WorkerAgent,
            "file_read_range",
            &serde_json::json!({"path": "note.txt", "start_line": 1, "line_count": 1}),
        )
        .expect("range executes");
        let ExecutedTool::Ok { text } = range else {
            panic!("file_read_range must execute, got {range:?}");
        };
        assert!(text.contains("hello tool loop"), "{text}");
        let grep = execute_read_only(
            &jail,
            &registry,
            AgentTier::WorkerAgent,
            "fs_grep",
            &serde_json::json!({"pattern": "hello"}),
        )
        .expect("grep executes");
        let ExecutedTool::Ok { text } = grep else {
            panic!("fs_grep must execute, got {grep:?}");
        };
        assert!(text.contains("note.txt"), "{text}");
    }

    #[test]
    fn every_side_effect_tool_is_refused_by_name_without_side_effects() {
        let (_temp, jail) = jail();
        let registry = registry();
        for tool in ["file_write", "shell_exec", "spawn_agent"] {
            let outcome = execute_read_only(
                &jail,
                &registry,
                AgentTier::WorkerAgent,
                tool,
                &serde_json::json!({"path": "note.txt", "content": "x", "program": "cmd.exe"}),
            )
            .expect("policy refusal is a result, not an error");
            match outcome {
                ExecutedTool::Refused { code, detail } => {
                    assert_eq!(code, "E_TOOL_NOT_ALLOWED", "{tool}");
                    assert!(detail.contains(tool), "{detail}");
                }
                other => panic!("{tool} must be refused, got {other:?}"),
            }
        }
    }

    #[test]
    fn report_result_and_finish_execute_as_read_class_terminal_results() {
        let (_temp, jail) = jail();
        let registry = registry();

        let report = execute_read_only(
            &jail,
            &registry,
            AgentTier::WorkerAgent,
            "report_result",
            &serde_json::json!({"summary": "finished the leaf", "success": true}),
        )
        .expect("report executes");
        let ExecutedTool::Ok { text } = report else {
            panic!("report_result must execute, got {report:?}");
        };
        let parsed: serde_json::Value = serde_json::from_str(&text).expect("report json");
        assert_eq!(parsed["summary"], "finished the leaf");
        assert_eq!(parsed["success"], true);

        let finish = execute_read_only(
            &jail,
            &registry,
            AgentTier::WorkerAgent,
            "finish",
            &serde_json::json!({"verdict": "done"}),
        )
        .expect("finish executes");
        let ExecutedTool::Ok { text } = finish else {
            panic!("finish must execute, got {finish:?}");
        };
        let parsed: serde_json::Value = serde_json::from_str(&text).expect("finish json");
        assert_eq!(parsed["verdict"], "done");
    }

    #[test]
    fn unknown_tool_name_is_refused_not_executed() {
        let (_temp, jail) = jail();
        let outcome = execute_read_only(
            &jail,
            &registry(),
            AgentTier::ToolAgent,
            "rm_rf",
            &serde_json::json!({"path": "/"}),
        )
        .expect("refusal result");
        assert_eq!(
            outcome,
            ExecutedTool::Refused {
                code: "E_TOOL_NOT_ALLOWED".to_owned(),
                detail: "tool \"rm_rf\" is not executable by the read-only executor; \
                         side-effecting tools are approval/loop-owned"
                    .to_owned(),
            }
        );
    }

    #[test]
    fn tier_without_read_grant_is_refused_but_reads_stay_read_only() {
        let (_temp, jail) = jail();
        let error = execute_read_only(
            &jail,
            &registry(),
            AgentTier::HyperAgent,
            "file_read",
            &serde_json::json!({"path": "note.txt"}),
        )
        .expect_err("hyper agent has no file_read grant");
        assert_eq!(error.code(), "E_TOOL_NOT_ALLOWED");

        let error = execute_read_only(
            &jail,
            &registry(),
            AgentTier::WorkerAgent,
            "file_read",
            &serde_json::json!({"wrong": 1}),
        )
        .expect_err("schema");
        assert_eq!(error.code(), "E_TOOL_SCHEMA");
    }

    #[test]
    fn jail_escape_and_missing_and_binary_are_named_refusals() {
        let (_temp, jail) = jail();
        let escape = execute_read_only(
            &jail,
            &registry(),
            AgentTier::WorkerAgent,
            "file_read",
            &serde_json::json!({"path": "../outside.txt"}),
        )
        .expect_err("escape");
        assert_eq!(escape.code(), "E_PATH_OUTSIDE_JAIL");

        let missing = execute_read_only(
            &jail,
            &registry(),
            AgentTier::WorkerAgent,
            "file_read",
            &serde_json::json!({"path": "nope.txt"}),
        )
        .expect_err("missing");
        assert_eq!(missing.code(), "E_PATH_NOT_FOUND");

        std::fs::write(_temp.path().join("binary.dat"), [0xff_u8, 0xfe, 0x00]).expect("write");
        let binary = execute_read_only(
            &jail,
            &registry(),
            AgentTier::WorkerAgent,
            "file_read",
            &serde_json::json!({"path": "binary.dat"}),
        )
        .expect_err("binary");
        assert_eq!(binary.code(), "E_SCHEMA_INVALID");
    }

    #[test]
    fn read_only_executor_never_uses_write_capability() {
        // The executor's only jail operations are read/list; a write can only be
        // observed if the jail file changes, so assert content is untouched and
        // no new file appears after routing every tool name.
        let (_temp, jail) = jail();
        let registry = registry();
        for tool in [
            "file_read",
            "file_list",
            "file_write",
            "shell_exec",
            "spawn_agent",
            "report_result",
            "finish",
        ] {
            let _ = execute_read_only(
                &jail,
                &registry,
                AgentTier::WorkerAgent,
                tool,
                &serde_json::json!({"path": "note.txt", "content": "SHOULD-NOT-WRITE", "program": "cmd.exe", "role": "r", "instruction": "i", "summary": "s", "success": true, "verdict": "v"}),
            );
        }
        assert_eq!(
            std::fs::read_to_string(_temp.path().join("note.txt")).expect("read"),
            "hello tool loop"
        );
        assert!(!_temp.path().join("SHOULD-NOT-WRITE").exists());
        assert_eq!(
            std::fs::read_dir(_temp.path()).expect("dir").count(),
            2,
            "no file was created by a refused tool"
        );
    }

    #[test]
    fn permission_classes_are_never_escalated() {
        let registry = registry();
        assert_eq!(
            registry.contract(ToolId::FileRead).expect("contract").permission_class,
            PermissionClass::Read
        );
        assert_eq!(
            registry.contract(ToolId::FileList).expect("contract").permission_class,
            PermissionClass::Read
        );
    }
}
