//! Frozen command handlers for the patched contract surface (Phase 6/10 wiring).
//!
//! These functions map the `hiverouter-contract` command payloads to the tool
//! layer with the patched refusal identity:
//! - `APPROVAL_RESOLVE` â†’ nonce/replay/TTL state machine ([`crate::approval`]);
//! - `SETTINGS_STATUS` â†’ key presence boolean, never the key;
//! - `FILE_TREE_LIST` / `FILE_READ` â†’ the workspace jail only, with
//!   `E_PATH_OUTSIDE_JAIL` / `E_PATH_NOT_FOUND` / `E_FILE_TOO_LARGE`.
//!
//! Core depends on this crate and calls these functions from its command
//! dispatch; the crate never depends on core.

use hiverouter_contract::{command, event, Refusal};
use rusqlite::Connection;

use crate::approval::{ApprovalGate, ApprovalResolution};
use crate::error::ToolError;
use crate::jail::WorkspaceJail;
use crate::settings::settings_status;
use crate::sha256::sha256_hex;

/// `SETTINGS_STATUS` â†’ `SETTINGS_STATUS_RESULT`.
#[must_use]
pub fn settings_status_command() -> event::SettingsStatusResult {
    settings_status()
}

/// `FILE_TREE_LIST` â†’ `FILE_TREE_RESULT` (one level of the workspace root).
///
/// # Errors
/// [`Refusal::PathNotFound`] when the root summary cannot be listed and
/// [`Refusal::PathOutsideJail`] for any containment failure (the jail resolves
/// the path; there is no ambient `std::fs` path here).
pub fn file_tree_list(
    jail: &WorkspaceJail,
    requested: Option<&str>,
) -> Result<event::FileTreeResult, ToolError> {
    let prefix = requested
        .map(|path| path.trim_matches('/'))
        .filter(|path| !path.is_empty());
    let entries = jail
        .list(prefix)?
        .into_iter()
        .map(|entry| {
            let path = match prefix {
                Some(prefix) => format!("{prefix}/{}", entry.name),
                None => entry.name.clone(),
            };
            event::FileTreeEntry {
                path,
                name: entry.name,
                is_dir: entry.is_dir,
                kind: if entry.is_dir { "dir" } else { "file" }.to_owned(),
                size_bytes: if entry.is_dir { None } else { Some(entry.size) },
            }
        })
        .collect();
    Ok(event::FileTreeResult {
        root: jail.root().to_string_lossy().into_owned(),
        entries,
    })
}

/// `FILE_READ` â†’ `FILE_CONTENT` (jailed, 1 MiB cap, UTF-8 required by the event).
///
/// # Errors
/// [`Refusal::PathOutsideJail`], [`Refusal::PathNotFound`],
/// [`Refusal::FileTooLarge`], or [`Refusal::SchemaInvalid`] when the bytes are
/// not UTF-8 (the frozen `FILE_CONTENT` payload carries text, not binary).
pub fn file_read(
    jail: &WorkspaceJail,
    payload: &command::FileRead,
) -> Result<event::FileContent, ToolError> {
    let bytes = jail.read(&payload.path)?;
    let text = String::from_utf8(bytes.clone()).map_err(|_| {
        ToolError::new(
            Refusal::SchemaInvalid,
            "file is not valid UTF-8; FILE_CONTENT carries text and refuses binary content",
        )
    })?;
    Ok(event::FileContent {
        path: payload.path.clone(),
        sha256: sha256_hex(&bytes),
        bytes: u64::try_from(bytes.len()).unwrap_or(u64::MAX),
        text,
    })
}

/// `APPROVAL_RESOLVE` â†’ resolution for the pending gate.
///
/// # Errors
/// [`Refusal::ApprovalNotFound`], [`Refusal::ApprovalReplay`],
/// [`Refusal::ApprovalTimeout`], [`Refusal::ApprovalDecisionInvalid`] (see
/// [`ApprovalGate::resolve_command`]).
pub fn approval_resolve(
    gate: &ApprovalGate,
    conn: &Connection,
    payload: &command::ApprovalResolve,
    now_unix_secs: u64,
) -> Result<ApprovalResolution, ToolError> {
    gate.resolve_command(
        conn,
        &payload.approval_id.to_string(),
        &payload.nonce,
        &payload.decision,
        payload.reason.as_deref(),
        now_unix_secs,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use hiverouter_contract::{ApprovalId, RunId, TaskId};

    #[test]
    fn file_read_and_tree_go_through_the_jail() {
        let temp = tempfile::tempdir().expect("tempdir");
        std::fs::write(temp.path().join("note.txt"), b"hello jail").expect("write");
        let jail = WorkspaceJail::open(temp.path()).expect("jail");

        let content = file_read(
            &jail,
            &command::FileRead {
                path: "note.txt".to_owned(),
            },
        )
        .expect("read");
        assert_eq!(content.text, "hello jail");
        assert_eq!(content.bytes, 10);
        assert_eq!(content.sha256.len(), 64);
        assert_eq!(content.path, "note.txt");

        let tree = file_tree_list(&jail, None).expect("tree");
        assert_eq!(tree.entries.len(), 1);
        assert_eq!(tree.entries[0].name, "note.txt");
        assert!(!tree.entries[0].is_dir);
    }

    #[test]
    fn file_read_refusals_are_the_patched_codes() {
        let temp = tempfile::tempdir().expect("tempdir");
        let jail = WorkspaceJail::open(temp.path()).expect("jail");

        let missing = file_read(
            &jail,
            &command::FileRead {
                path: "nope.txt".to_owned(),
            },
        )
        .expect_err("missing file");
        assert_eq!(missing.code(), "E_PATH_NOT_FOUND");

        let outside = file_read(
            &jail,
            &command::FileRead {
                path: "../outside.txt".to_owned(),
            },
        )
        .expect_err("escape");
        assert_eq!(outside.code(), "E_PATH_OUTSIDE_JAIL");

        let big = vec![b'x'; usize::try_from(crate::jail::MAX_READ_BYTES).expect("fits") + 1];
        std::fs::write(temp.path().join("big.bin"), &big).expect("write");
        let too_large = file_read(
            &jail,
            &command::FileRead {
                path: "big.bin".to_owned(),
            },
        )
        .expect_err("too large");
        assert_eq!(too_large.code(), "E_FILE_TOO_LARGE");

        let binary = vec![0xff, 0xfe, 0x00];
        std::fs::write(temp.path().join("binary.dat"), &binary).expect("write");
        let non_utf8 = file_read(
            &jail,
            &command::FileRead {
                path: "binary.dat".to_owned(),
            },
        )
        .expect_err("binary");
        assert_eq!(non_utf8.code(), "E_SCHEMA_INVALID");
    }

    #[test]
    fn approval_resolve_command_uses_nonce() {
        let conn = Connection::open_in_memory().expect("db");
        crate::approval::migrate(&conn).expect("migrate");
        let gate = ApprovalGate::default();
        let request = gate
            .request(
                &conn,
                RunId::new(),
                TaskId::new(),
                "call-1",
                crate::tool::ToolId::FileWrite,
                crate::tool::PermissionClass::Mutate,
                &serde_json::json!({"path": "a.txt", "content": "x"}),
                0,
            )
            .expect("committed");
        let payload = command::ApprovalResolve {
            approval_id: ApprovalId::from_uuid(request.approval_id.as_uuid()),
            nonce: request.nonce.clone(),
            decision: "approve".to_owned(),
            reason: Some("test".to_owned()),
        };
        let resolution = approval_resolve(&gate, &conn, &payload, 1).expect("resolved");
        assert_eq!(
            resolution.decision,
            crate::approval::ApprovalDecision::AllowOnce
        );
        let replay = approval_resolve(&gate, &conn, &payload, 1).expect_err("replay");
        assert_eq!(replay.code(), "E_APPROVAL_REPLAY");
    }

    #[test]
    fn settings_payload_is_secret_free() {
        let payload = settings_status_command();
        let rendered = serde_json::to_string(&payload).expect("serialises");
        assert!(!rendered.contains("sk-"));
        assert_eq!(payload.provider, "deepseek");
    }
}

