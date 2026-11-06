//! Core -> tools bridge (Phase 6 wiring for the patched frozen commands).
//!
//! `hiverouter-core` depends on `hiverouter-tools` (never the reverse). This
//! module is the single wiring point for the tools-owned capabilities used by
//! the WS command dispatch and the `/diagnose` route:
//!
//! - `SETTINGS_STATUS` -> [`hiverouter_tools::commands::settings_status_command`]
//! - `FILE_TREE_LIST` / `FILE_READ` -> the workspace jail ([`WorkspaceJail`])
//! - `APPROVAL_RESOLVE` -> the single approval gate + nonce/replay state machine
//! - `/diagnose` -> [`hiverouter_tools::health::diagnostics`]
//!
//! When a capability is genuinely unavailable (no usable workspace root), the
//! bridge returns the truthful named refusal `E_FILE_TREE_UNAVAILABLE` instead
//! of a generic internal error.

use std::path::{Path, PathBuf};
use std::sync::Arc;

use hiverouter_contract::command;
use hiverouter_contract::{event, Refusal, Timestamp};
use hiverouter_tools::approval::ApprovalResolution;
use hiverouter_tools::jail::WorkspaceJail;
use hiverouter_tools::ToolError;

use crate::db::Database;

/// Wired tools capabilities for the local server.
#[derive(Debug)]
pub struct ToolsBridge {
    jail: Option<WorkspaceJail>,
    jail_error: Option<String>,
    gate: hiverouter_tools::approval::ApprovalGate,
    workspace_root: PathBuf,
    db: Arc<Database>,
}

impl ToolsBridge {
    /// Creates the bridge; a workspace root that cannot be opened leaves the
    /// jail unavailable (file commands then refuse `E_FILE_TREE_UNAVAILABLE`).
    #[must_use]
    pub fn new(workspace_root: &Path, db: Arc<Database>) -> Self {
        let (jail, jail_error) = if workspace_root.is_dir() {
            match WorkspaceJail::open(workspace_root) {
                Ok(jail) => (Some(jail), None),
                Err(error) => (
                    None,
                    Some(format!("{}: {}", error.code(), error.detail())),
                ),
            }
        } else {
            (
                None,
                Some(format!(
                    "workspace root {} is not an existing directory",
                    workspace_root.display()
                )),
            )
        };
        Self {
            jail,
            jail_error,
            gate: hiverouter_tools::approval::ApprovalGate::new(),
            workspace_root: workspace_root.to_path_buf(),
            db,
        }
    }

    /// Creates the tools-owned approval table (idempotent). Called from the core
    /// migration/boot path so the tools layer never opens its own connection.
    ///
    /// # Errors
    /// `E_INTERNAL_ERROR` when the DDL fails.
    pub fn migrate(db: &Database) -> Result<(), Refusal> {
        db.with_connection_refusal(|conn| {
            hiverouter_tools::approval::migrate(conn).map_err(|error| error.refusal())
        })
    }

    /// Workspace root the jail was opened against.
    #[must_use]
    pub fn workspace_root(&self) -> &Path {
        &self.workspace_root
    }

    /// Whether the workspace jail is usable.
    #[must_use]
    pub fn file_access_available(&self) -> bool {
        self.jail.is_some()
    }

    /// Details of why the jail is unavailable (diagnostics only, no secrets).
    #[must_use]
    pub fn jail_error(&self) -> Option<&str> {
        self.jail_error.as_deref()
    }

    /// The single approval gate.
    #[must_use]
    pub fn approval_gate(&self) -> &hiverouter_tools::approval::ApprovalGate {
        &self.gate
    }

    fn require_jail(&self) -> Result<&WorkspaceJail, Refusal> {
        self.jail.as_ref().ok_or(Refusal::FileTreeUnavailable)
    }

    fn map_tool_error(error: ToolError) -> Refusal {
        error.refusal()
    }

    /// `SETTINGS_STATUS` -> `SETTINGS_STATUS_RESULT` (never the key).
    #[must_use]
    pub fn settings_status(&self) -> event::SettingsStatusResult {
        hiverouter_tools::commands::settings_status_command()
    }

    /// `FILE_TREE_LIST` -> `FILE_TREE_RESULT` through the jail.
    ///
    /// # Errors
    /// `E_FILE_TREE_UNAVAILABLE`, `E_PATH_OUTSIDE_JAIL`, `E_PATH_NOT_FOUND`.
    pub fn file_tree_list(&self) -> Result<event::FileTreeResult, Refusal> {
        let jail = self.require_jail()?;
        hiverouter_tools::commands::file_tree_list(jail).map_err(Self::map_tool_error)
    }

    /// `FILE_READ` -> `FILE_CONTENT` through the jail.
    ///
    /// # Errors
    /// `E_FILE_TREE_UNAVAILABLE`, `E_PATH_OUTSIDE_JAIL`, `E_PATH_NOT_FOUND`,
    /// `E_FILE_TOO_LARGE`, `E_SCHEMA_INVALID` (non-UTF-8 file).
    pub fn file_read(&self, payload: &command::FileRead) -> Result<event::FileContent, Refusal> {
        let jail = self.require_jail()?;
        hiverouter_tools::commands::file_read(jail, payload).map_err(Self::map_tool_error)
    }

    /// `APPROVAL_RESOLVE` -> resolution through the nonce/replay gate.
    ///
    /// # Errors
    /// `E_APPROVAL_NOT_FOUND`, `E_APPROVAL_REPLAY`, `E_APPROVAL_TIMEOUT`,
    /// `E_APPROVAL_DECISION_INVALID`, `E_DB_BUSY`.
    pub fn resolve_approval(
        &self,
        payload: &command::ApprovalResolve,
    ) -> Result<ApprovalResolution, Refusal> {
        let now_unix_secs = Timestamp::now().unix_secs().unwrap_or(0);
        self.db.with_connection_refusal(|conn| {
            self.gate
                .resolve_command(
                    conn,
                    &payload.approval_id.to_string(),
                    &payload.nonce,
                    &payload.decision,
                    payload.reason.as_deref(),
                    now_unix_secs,
                )
                .map_err(|error| error.refusal())
        })
    }

    /// `/diagnose` payload from the tools health/derived census layer.
    ///
    /// # Errors
    /// `E_INTERNAL_ERROR` when the capability census is inconsistent.
    pub fn diagnostics(&self) -> Result<serde_json::Value, Refusal> {
        let db_ok = self.db.schema_version().is_ok();
        let key_present = self.settings_status().key_present;
        hiverouter_tools::health::diagnostics(db_ok, key_present)
            .map_err(|error| error.refusal())
    }

    /// Contract event for an approval resolution (`approve`/`deny` echo).
    #[must_use]
    pub fn approval_resolved_event(resolution: &ApprovalResolution) -> event::ApprovalResolved {
        let decision = match resolution.decision {
            hiverouter_tools::approval::ApprovalDecision::AllowOnce
            | hiverouter_tools::approval::ApprovalDecision::AllowSession => "approve",
            hiverouter_tools::approval::ApprovalDecision::Deny => "deny",
        };
        event::ApprovalResolved {
            approval_id: resolution.request.approval_id,
            tool: resolution.request.tool.id().to_owned(),
            decision: decision.to_owned(),
        }
    }
}
