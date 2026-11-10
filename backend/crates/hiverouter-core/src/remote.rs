//! Remote control plane R1: tree-free principals, devices, grants, actions,
//! audit. This is deliberately not a task/DAG model: a remote has no run,
//! parent or depth and consumes no agent/call cap.
//!
//! The only shipped adapter is `in_process`, which updates a normalized fixture
//! state for a registered light/switch/cover. No network call is made.

use std::sync::Arc;

use hiverouter_contract::{Refusal, Timestamp};
use rusqlite::{params, OptionalExtension};
use serde_json::{json, Value};
use uuid::Uuid;

use crate::db::{Database, DbError};

fn map(error: DbError) -> Refusal {
    error.refusal()
}

/// A tree-free remote principal.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RemoteAgent {
    pub remote_id: String,
    pub display_name: String,
    pub owner_kind: String,
    pub owner_id: String,
    pub transport_kind: String,
    pub adapter_kind: String,
    pub endpoint_ref: String,
    pub credential_ref: Option<String>,
    pub capabilities_json: String,
    pub state: String,
    pub generation: i64,
    pub rev: i64,
    pub last_seen_at: Option<String>,
    pub lease_expires_at: Option<String>,
    pub quarantined_at: Option<String>,
    pub revoked_at: Option<String>,
    pub created_by: String,
    pub created_at: String,
}

/// A device owned by one remote.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Device {
    pub device_id: String,
    pub remote_id: String,
    pub class: String,
    pub name: String,
    pub room: Option<String>,
    pub adapter_ref: String,
    pub capabilities_json: String,
    pub state_json: String,
    pub state_version: i64,
    pub last_seen_at: Option<String>,
    pub safety_class: String,
    pub revoked_at: Option<String>,
}

/// A capability grant.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RemoteGrant {
    pub grant_id: String,
    pub remote_id: String,
    pub device_id: String,
    pub capability_id: String,
    pub action: String,
    pub constraints_json: String,
    pub granted_by: String,
    pub granted_at: String,
    pub expires_at: Option<String>,
    pub revoked_at: Option<String>,
}

/// One action request/result.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DeviceAction {
    pub action_id: String,
    pub remote_id: String,
    pub device_id: String,
    pub capability_id: String,
    pub action: String,
    pub params_json: String,
    pub expected_json: Option<String>,
    pub idempotency_key: String,
    pub requested_by_kind: String,
    pub requested_by_id: String,
    pub approval_nonce: Option<String>,
    pub approval_expires_at: Option<String>,
    pub status: String,
    pub error_code: Option<String>,
    pub created_at: String,
    pub started_at: Option<String>,
    pub ended_at: Option<String>,
}

/// Redacted remote audit row.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RemoteAudit {
    pub at: String,
    pub actor_kind: String,
    pub actor_id: String,
    pub remote_id: Option<String>,
    pub device_id: Option<String>,
    pub action: String,
    pub request_id: Option<String>,
    pub idempotency_key: Option<String>,
    pub outcome: String,
    pub refusal_code: Option<String>,
    pub detail_json: String,
}

/// Input for creating a remote principal.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NewRemote {
    pub display_name: String,
    pub owner_kind: String,
    pub owner_id: String,
    pub transport_kind: String,
    pub adapter_kind: String,
    pub endpoint_ref: String,
    pub credential_ref: Option<String>,
    pub capabilities_json: String,
    pub created_by: String,
    /// Optional idempotency key for a caller retry.
    pub request_id: Option<String>,
    /// Canonical fingerprint of the request payload; binds a request id to one payload.
    pub request_fingerprint: Option<String>,
}

/// Input for registering a device.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NewDevice {
    pub remote_id: String,
    pub class: String,
    pub name: String,
    pub room: Option<String>,
    pub adapter_ref: String,
    pub capabilities_json: String,
    pub safety_class: String,
}

/// Input for a grant.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NewGrant {
    pub remote_id: String,
    pub device_id: String,
    pub capability_id: String,
    pub action: String,
    pub constraints_json: String,
    pub expires_at: Option<String>,
}

/// Read/write facade for remote records.
#[derive(Debug)]
pub struct RemoteStore {
    db: Arc<Database>,
    /// Serializes local action requests so single-flight/idempotency cannot race
    /// before the durable row exists. The DB remains the durable authority.
    action_lock: std::sync::Mutex<()>,
}

const REMOTE_SELECT: &str = "SELECT remote_id, display_name, owner_kind, owner_id,
    transport_kind, adapter_kind, endpoint_ref, credential_ref, capabilities_json,
    state, generation, rev, last_seen_at, lease_expires_at, quarantined_at,
    revoked_at, created_by, created_at FROM remote_agents";

const DEVICE_SELECT: &str = "SELECT device_id, remote_id, class, name, room,
    adapter_ref, capabilities_json, state_json, state_version, last_seen_at,
    safety_class, revoked_at FROM devices";

const GRANT_SELECT: &str = "SELECT grant_id, remote_id, device_id, capability_id,
    action, constraints_json, granted_by, granted_at, expires_at, revoked_at
    FROM remote_grants";

const ACTION_SELECT: &str = "SELECT action_id, remote_id, device_id, capability_id,
    action, params_json, expected_json, idempotency_key, requested_by_kind,
    requested_by_id, approval_nonce, approval_expires_at, status, error_code,
    created_at, started_at, ended_at FROM device_actions";

fn remote_from_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<RemoteAgent> {
    Ok(RemoteAgent {
        remote_id: row.get(0)?,
        display_name: row.get(1)?,
        owner_kind: row.get(2)?,
        owner_id: row.get(3)?,
        transport_kind: row.get(4)?,
        adapter_kind: row.get(5)?,
        endpoint_ref: row.get(6)?,
        credential_ref: row.get(7)?,
        capabilities_json: row.get(8)?,
        state: row.get(9)?,
        generation: row.get(10)?,
        rev: row.get(11)?,
        last_seen_at: row.get(12)?,
        lease_expires_at: row.get(13)?,
        quarantined_at: row.get(14)?,
        revoked_at: row.get(15)?,
        created_by: row.get(16)?,
        created_at: row.get(17)?,
    })
}

fn device_from_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<Device> {
    Ok(Device {
        device_id: row.get(0)?,
        remote_id: row.get(1)?,
        class: row.get(2)?,
        name: row.get(3)?,
        room: row.get(4)?,
        adapter_ref: row.get(5)?,
        capabilities_json: row.get(6)?,
        state_json: row.get(7)?,
        state_version: row.get(8)?,
        last_seen_at: row.get(9)?,
        safety_class: row.get(10)?,
        revoked_at: row.get(11)?,
    })
}

fn grant_from_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<RemoteGrant> {
    Ok(RemoteGrant {
        grant_id: row.get(0)?,
        remote_id: row.get(1)?,
        device_id: row.get(2)?,
        capability_id: row.get(3)?,
        action: row.get(4)?,
        constraints_json: row.get(5)?,
        granted_by: row.get(6)?,
        granted_at: row.get(7)?,
        expires_at: row.get(8)?,
        revoked_at: row.get(9)?,
    })
}

fn action_from_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<DeviceAction> {
    Ok(DeviceAction {
        action_id: row.get(0)?,
        remote_id: row.get(1)?,
        device_id: row.get(2)?,
        capability_id: row.get(3)?,
        action: row.get(4)?,
        params_json: row.get(5)?,
        expected_json: row.get(6)?,
        idempotency_key: row.get(7)?,
        requested_by_kind: row.get(8)?,
        requested_by_id: row.get(9)?,
        approval_nonce: row.get(10)?,
        approval_expires_at: row.get(11)?,
        status: row.get(12)?,
        error_code: row.get(13)?,
        created_at: row.get(14)?,
        started_at: row.get(15)?,
        ended_at: row.get(16)?,
    })
}

fn hash(value: &str) -> String {
    hiverouter_tools::sha256::sha256_hex(value.as_bytes())
}

fn audit(
    conn: &rusqlite::Connection,
    actor_kind: &str,
    actor_id: &str,
    remote_id: Option<&str>,
    device_id: Option<&str>,
    action: &str,
    request_id: Option<&str>,
    idempotency_key: Option<&str>,
    params_sha256: Option<&str>,
    outcome: &str,
    refusal_code: Option<&str>,
    detail_json: &str,
) -> rusqlite::Result<()> {
    conn.execute(
        "INSERT INTO remote_audit
         (audit_id, at, actor_kind, actor_id, remote_id, device_id, action,
          request_id, idempotency_key, params_sha256, outcome, refusal_code,
          detail_json, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?2)",
        params![
            format!("audit:{}", Uuid::now_v7()),
            Timestamp::now().as_str(),
            actor_kind,
            actor_id,
            remote_id,
            device_id,
            action,
            request_id,
            idempotency_key,
            params_sha256,
            outcome,
            refusal_code,
            detail_json,
        ],
    )?;
    Ok(())
}

impl RemoteStore {
    /// Builds the store over the shared database.
    #[must_use]
    pub fn new(db: Arc<Database>) -> Self {
        Self {
            db,
            action_lock: std::sync::Mutex::new(()),
        }
    }

    /// Creates a fixture remote. Only `in_process` is supported in R1.
    ///
    /// # Errors
    /// Named refusal or database refusal.
    pub fn create_remote(&self, input: NewRemote) -> Result<RemoteAgent, Refusal> {
        if input.transport_kind != "in_process" {
            return Err(Refusal::RemoteTransport);
        }
        if !matches!(input.owner_kind.as_str(), "operator" | "hive" | "office") {
            return Err(Refusal::SchemaInvalid);
        }
        if input.display_name.trim().is_empty()
            || input.display_name.chars().count() > 120
            || input.owner_id.trim().is_empty()
        {
            return Err(Refusal::SchemaInvalid);
        }
        let remote_id = format!("remote:{}", Uuid::now_v7());
        let now = Timestamp::now().as_str().to_owned();
        self
            .db
            .with_connection(|connection| {
                connection.execute(
                    "INSERT INTO remote_agents
                     (remote_id, display_name, owner_kind, owner_id, transport_kind,
                      adapter_kind, endpoint_ref, credential_ref, capabilities_json,
                      state, generation, rev, last_seen_at, lease_expires_at,
                      created_by, created_at)
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9,
                             'enrolled', 1, 0, NULL, NULL, ?11, ?10)",
                    params![
                        remote_id,
                        input.display_name.trim(),
                        input.owner_kind,
                        input.owner_id,
                        input.transport_kind,
                        input.adapter_kind,
                        input.endpoint_ref,
                        input.credential_ref,
                        input.capabilities_json,
                        now,
                        input.created_by,
                    ],
                )?;
                audit(
                    connection,
                    if input.created_by.starts_with("hyper:")
                        || input.created_by.starts_with("agent:")
                    {
                        "system"
                    } else {
                        "operator"
                    },
                    &input.created_by,
                    Some(&remote_id),
                    None,
                    "remote_created",
                    input.request_id.as_deref(),
                    None,
                    input.request_fingerprint.as_deref(),
                    "ok",
                    None,
                    "{}",
                )?;
                Ok(())
            })
            .map_err(map)?;
        self.get_remote(&remote_id)?.ok_or(Refusal::RemoteUnknown)
    }

    /// Lists all non-retired remotes.
    ///
    /// # Errors
    /// Database refusal.
    pub fn list_remotes(&self) -> Result<Vec<RemoteAgent>, Refusal> {
        self.db
            .with_connection(|connection| {
                let mut statement = connection.prepare(&format!(
                    "{REMOTE_SELECT} WHERE state <> 'retired' ORDER BY created_at ASC"
                ))?;
                let rows = statement.query_map([], remote_from_row)?;
                let mut remotes = Vec::new();
                for row in rows {
                    remotes.push(row?);
                }
                Ok(remotes)
            })
            .map_err(map)
    }

    /// Gets one remote.
    ///
    /// # Errors
    /// Database refusal.
    pub fn get_remote(&self, remote_id: &str) -> Result<Option<RemoteAgent>, Refusal> {
        self.db
            .with_connection(|connection| {
                connection
                    .query_row(
                        &format!("{REMOTE_SELECT} WHERE remote_id = ?1"),
                        params![remote_id],
                        remote_from_row,
                    )
                    .optional()
            })
            .map_err(map)
    }

    /// Finds the remote created by one idempotent request key, if any.
    ///
    /// # Errors
    /// Database refusal.
    pub fn find_by_request_id(
        &self,
        request_id: &str,
    ) -> Result<Option<(RemoteAgent, Option<String>)>, Refusal> {
        let row: Option<(String, Option<String>)> = self
            .db
            .with_connection(|connection| {
                connection
                    .query_row(
                        "SELECT remote_id, params_sha256 FROM remote_audit
                         WHERE request_id = ?1 AND action = 'remote_created' AND outcome = 'ok'
                         ORDER BY at DESC LIMIT 1",
                        params![request_id],
                        |row| Ok((row.get(0)?, row.get(1)?)),
                    )
                    .optional()
            })
            .map_err(map)?;
        match row {
            Some((remote_id, fingerprint)) => Ok(self
                .get_remote(&remote_id)?
                .map(|remote| (remote, fingerprint))),
            None => Ok(None),
        }
    }

    /// Revokes a remote; its grants/actions become unusable immediately.
    ///
    /// # Errors
    /// Database refusal.
    pub fn revoke_remote(&self, remote_id: &str) -> Result<(), Refusal> {
        let changed = self
            .db
            .with_connection(|connection| {
                let changed = connection.execute(
                    "UPDATE remote_agents SET state = 'revoked', revoked_at = ?2, rev = rev + 1
                     WHERE remote_id = ?1 AND state <> 'retired'",
                    params![remote_id, Timestamp::now().as_str()],
                )?;
                if changed == 1 {
                    audit(
                        connection,
                        "operator",
                        "operator",
                        Some(remote_id),
                        None,
                        "remote_revoked",
                        None,
                        None,
                        None,
                        "ok",
                        None,
                        "{}",
                    )?;
                }
                Ok(changed)
            })
            .map_err(map)?;
        if changed == 0 {
            return Err(Refusal::RemoteUnknown);
        }
        Ok(())
    }

    /// Registers a device under one remote.
    ///
    /// # Errors
    /// Named or database refusal.
    pub fn create_device(&self, input: NewDevice) -> Result<Device, Refusal> {
        let remote = self.get_remote(&input.remote_id)?.ok_or(Refusal::RemoteUnknown)?;
        if remote.revoked_at.is_some() || remote.state == "retired" {
            return Err(Refusal::RemoteUnknown);
        }
        if !matches!(
            input.class.as_str(),
            "light" | "switch" | "camera" | "cover" | "lock" | "sensor" | "other"
        ) || !matches!(
            input.safety_class.as_str(),
            "read_only" | "reversible" | "critical"
        ) || input.name.trim().is_empty()
        {
            return Err(Refusal::SchemaInvalid);
        }
        let device_id = format!("device:{}", Uuid::now_v7());
        let now = Timestamp::now().as_str().to_owned();
        self.db
            .with_connection(|connection| {
                connection.execute(
                    "INSERT INTO devices
                     (device_id, remote_id, class, name, room, adapter_ref,
                      capabilities_json, state_json, state_version, last_seen_at,
                      safety_class, revoked_at, created_at)
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, '{}', 0, ?8, ?9, NULL, ?8)",
                    params![
                        device_id,
                        input.remote_id,
                        input.class,
                        input.name.trim(),
                        input.room,
                        input.adapter_ref,
                        input.capabilities_json,
                        now,
                        input.safety_class,
                    ],
                )?;
                let actor_kind = if remote.created_by.starts_with("hyper:")
                    || remote.created_by.starts_with("agent:")
                {
                    "system"
                } else {
                    "operator"
                };
                audit(
                    connection,
                    actor_kind,
                    &remote.created_by,
                    Some(&input.remote_id),
                    Some(&device_id),
                    "device_created",
                    None,
                    None,
                    None,
                    "ok",
                    None,
                    "{}",
                )?;
                Ok(())
            })
            .map_err(map)?;
        self.get_device(&device_id)?
            .ok_or(Refusal::DeviceUnknown)
    }

    /// Lists devices for one remote.
    ///
    /// # Errors
    /// Database refusal.
    pub fn list_devices(&self, remote_id: &str) -> Result<Vec<Device>, Refusal> {
        self.db
            .with_connection(|connection| {
                let mut statement = connection.prepare(&format!(
                    "{DEVICE_SELECT} WHERE remote_id = ?1 AND revoked_at IS NULL ORDER BY created_at ASC"
                ))?;
                let rows = statement.query_map(params![remote_id], device_from_row)?;
                let mut devices = Vec::new();
                for row in rows {
                    devices.push(row?);
                }
                Ok(devices)
            })
            .map_err(map)
    }

    /// Gets one device.
    ///
    /// # Errors
    /// Database refusal.
    pub fn get_device(&self, device_id: &str) -> Result<Option<Device>, Refusal> {
        self.db
            .with_connection(|connection| {
                connection
                    .query_row(
                        &format!("{DEVICE_SELECT} WHERE device_id = ?1"),
                        params![device_id],
                        device_from_row,
                    )
                    .optional()
            })
            .map_err(map)
    }

    /// Creates an explicit capability grant.
    ///
    /// # Errors
    /// Named/database refusal.
    pub fn create_grant(&self, input: NewGrant) -> Result<RemoteGrant, Refusal> {
        let remote = self.get_remote(&input.remote_id)?.ok_or(Refusal::RemoteUnknown)?;
        if remote.revoked_at.is_some() {
            return Err(Refusal::RemoteUnknown);
        }
        let device = self.get_device(&input.device_id)?.ok_or(Refusal::DeviceUnknown)?;
        if device.remote_id != input.remote_id || device.revoked_at.is_some() {
            return Err(Refusal::DeviceUnknown);
        }
        if input.capability_id.trim().is_empty() || input.action.trim().is_empty() {
            return Err(Refusal::SchemaInvalid);
        }
        let grant_id = format!("grant:{}", Uuid::now_v7());
        let now = Timestamp::now().as_str().to_owned();
        self.db
            .with_connection(|connection| {
                let actor_kind = if remote.created_by.starts_with("hyper:")
                    || remote.created_by.starts_with("agent:")
                {
                    "system"
                } else {
                    "operator"
                };
                connection.execute(
                    "INSERT INTO remote_grants
                     (grant_id, remote_id, device_id, capability_id, action,
                      constraints_json, granted_by, granted_at, expires_at, revoked_at, rev)
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, NULL, 0)",
                    params![
                        grant_id,
                        input.remote_id,
                        input.device_id,
                        input.capability_id,
                        input.action,
                        input.constraints_json,
                        remote.created_by,
                        now,
                        input.expires_at,
                    ],
                )?;
                audit(
                    connection,
                    actor_kind,
                    &remote.created_by,
                    Some(&input.remote_id),
                    Some(&input.device_id),
                    "grant_created",
                    None,
                    None,
                    None,
                    "ok",
                    None,
                    "{}",
                )?;
                Ok(())
            })
            .map_err(map)?;
        self.list_grants(&input.remote_id)?
            .into_iter()
            .find(|grant| grant.grant_id == grant_id)
            .ok_or(Refusal::RemoteUnknown)
    }

    /// Lists grants for one remote (including revoked for audit).
    ///
    /// # Errors
    /// Database refusal.
    pub fn list_grants(&self, remote_id: &str) -> Result<Vec<RemoteGrant>, Refusal> {
        self.db
            .with_connection(|connection| {
                let mut statement = connection.prepare(&format!(
                    "{GRANT_SELECT} WHERE remote_id = ?1 ORDER BY granted_at ASC"
                ))?;
                let rows = statement.query_map(params![remote_id], grant_from_row)?;
                let mut grants = Vec::new();
                for row in rows {
                    grants.push(row?);
                }
                Ok(grants)
            })
            .map_err(map)
    }

    /// Records one real fixture heartbeat and promotes the remote to `online`.
    /// A remote is never created online; this is the only transition that
    /// supplies a lease.
    ///
    /// # Errors
    /// `E_REMOTE_UNKNOWN` for a missing/revoked/retired remote; database refusal.
    pub fn heartbeat(&self, remote_id: &str) -> Result<RemoteAgent, Refusal> {
        let now_secs = Timestamp::now()
            .unix_secs()
            .map_err(|_| Refusal::InternalError)?;
        let last_seen = Timestamp::from_unix_secs(now_secs).as_str().to_owned();
        let lease = Timestamp::from_unix_secs(now_secs.saturating_add(120))
            .as_str()
            .to_owned();
        let changed = self
            .db
            .with_connection(|connection| {
                let changed = connection.execute(
                    "UPDATE remote_agents
                     SET state = 'online', last_seen_at = ?2, lease_expires_at = ?3,
                         generation = generation + 1, rev = rev + 1
                     WHERE remote_id = ?1 AND revoked_at IS NULL AND state <> 'retired'",
                    params![remote_id, last_seen, lease],
                )?;
                if changed == 1 {
                    audit(
                        connection,
                        "operator",
                        "operator",
                        Some(remote_id),
                        None,
                        "remote_heartbeat",
                        None,
                        None,
                        None,
                        "ok",
                        None,
                        "{}",
                    )?;
                }
                Ok(changed)
            })
            .map_err(map)?;
        if changed == 0 {
            return Err(Refusal::RemoteUnknown);
        }
        self.get_remote(remote_id)?.ok_or(Refusal::RemoteUnknown)
    }

    /// Expires abandoned critical approvals so their in-flight row cannot lock
    /// a device forever. Called by the supervisor each tick.
    pub fn expire_approvals(&self) -> Result<usize, Refusal> {
        let now = Timestamp::now().as_str().to_owned();
        self.db
            .with_connection(|connection| {
                let tx = connection.transaction()?;
                let mut expired: Vec<(String, String, String)> = Vec::new();
                {
                    let mut statement = tx.prepare(
                        "SELECT action_id, remote_id, device_id FROM device_actions
                         WHERE status = 'awaiting_approval'
                           AND approval_expires_at IS NOT NULL
                           AND approval_expires_at <= ?1",
                    )?;
                    let rows = statement.query_map(params![now], |row| {
                        Ok((row.get(0)?, row.get(1)?, row.get(2)?))
                    })?;
                    for row in rows {
                        expired.push(row?);
                    }
                }
                let changed = tx.execute(
                    "UPDATE device_actions SET status = 'timeout', ended_at = ?1
                     WHERE status = 'awaiting_approval'
                       AND approval_expires_at IS NOT NULL
                       AND approval_expires_at <= ?1",
                    params![now],
                )?;
                for (action_id, remote_id, device_id) in &expired {
                    audit(
                        &tx,
                        "system",
                        "supervisor",
                        Some(remote_id),
                        Some(device_id),
                        "device_action_approval_expired",
                        Some(action_id),
                        None,
                        None,
                        "timeout",
                        None,
                        &json!({"reason": "approval_expired", "at": now}).to_string(),
                    )?;
                }
                tx.commit()?;
                Ok(changed)
            })
            .map_err(map)
    }

    /// Demotes online remotes whose lease has expired. This is the supervisor's
    /// truthful health transition; it never extends a lease or invents a state.
    pub fn expire_leases(&self) -> Result<usize, Refusal> {
        let now = Timestamp::now().as_str().to_owned();
        self.db
            .with_connection(|connection| {
                let tx = connection.transaction()?;
                let mut expired: Vec<String> = Vec::new();
                {
                    let mut statement = tx.prepare(
                        "SELECT remote_id FROM remote_agents
                         WHERE state = 'online' AND lease_expires_at IS NOT NULL
                           AND lease_expires_at <= ?1",
                    )?;
                    let rows = statement.query_map(params![now], |row| row.get(0))?;
                    for row in rows {
                        expired.push(row?);
                    }
                }
                let changed = tx.execute(
                    "UPDATE remote_agents SET state = 'offline', rev = rev + 1
                     WHERE state = 'online' AND lease_expires_at IS NOT NULL
                       AND lease_expires_at <= ?1",
                    params![now],
                )?;
                for remote_id in &expired {
                    audit(
                        &tx,
                        "system",
                        "supervisor",
                        Some(remote_id),
                        None,
                        "remote_lease_expired",
                        None,
                        None,
                        None,
                        "offline",
                        None,
                        &json!({"reason": "lease_expired", "at": now}).to_string(),
                    )?;
                }
                tx.commit()?;
                Ok(changed)
            })
            .map_err(map)
    }

    /// Boot reconciliation for remote actions: an executing action becomes
    /// interrupted; an expired approval becomes timeout. No action is replayed.
    pub fn reconcile_on_boot(&self) -> Result<usize, Refusal> {
        let now = Timestamp::now().as_str().to_owned();
        self.db
            .with_connection(|connection| {
                let tx = connection.transaction()?;
                let mut interrupted: Vec<(String, String, String)> = Vec::new();
                {
                    let mut statement = tx.prepare(
                        "SELECT action_id, remote_id, device_id FROM device_actions
                         WHERE status = 'executing'",
                    )?;
                    let rows = statement.query_map([], |row| {
                        Ok((row.get(0)?, row.get(1)?, row.get(2)?))
                    })?;
                    for row in rows {
                        interrupted.push(row?);
                    }
                }
                let mut timed_out: Vec<(String, String, String)> = Vec::new();
                {
                    let mut statement = tx.prepare(
                        "SELECT action_id, remote_id, device_id FROM device_actions
                         WHERE status IN ('requested','awaiting_approval')
                           AND approval_expires_at IS NOT NULL
                           AND approval_expires_at <= ?1",
                    )?;
                    let rows = statement.query_map(params![now], |row| {
                        Ok((row.get(0)?, row.get(1)?, row.get(2)?))
                    })?;
                    for row in rows {
                        timed_out.push(row?);
                    }
                }
                let mut offline: Vec<String> = Vec::new();
                {
                    let mut statement = tx.prepare(
                        "SELECT remote_id FROM remote_agents
                         WHERE state = 'online' AND lease_expires_at IS NOT NULL
                           AND lease_expires_at <= ?1",
                    )?;
                    let rows = statement.query_map(params![now], |row| row.get(0))?;
                    for row in rows {
                        offline.push(row?);
                    }
                }
                let executing = tx.execute(
                    "UPDATE device_actions SET status = 'interrupted', ended_at = ?1
                     WHERE status = 'executing'",
                    params![now],
                )?;
                let expired = tx.execute(
                    "UPDATE device_actions SET status = 'timeout', ended_at = ?1
                     WHERE status IN ('requested','awaiting_approval')
                       AND approval_expires_at IS NOT NULL AND approval_expires_at <= ?1",
                    params![now],
                )?;
                let leases = tx.execute(
                    "UPDATE remote_agents SET state = 'offline', rev = rev + 1
                     WHERE state = 'online' AND lease_expires_at IS NOT NULL
                       AND lease_expires_at <= ?1",
                    params![now],
                )?;
                for (action_id, remote_id, device_id) in &interrupted {
                    audit(
                        &tx,
                        "system",
                        "supervisor",
                        Some(remote_id),
                        Some(device_id),
                        "device_action_boot_reconciled",
                        Some(action_id),
                        None,
                        None,
                        "interrupted",
                        None,
                        &json!({"reason": "boot_reconciliation", "prior": "executing", "at": now})
                            .to_string(),
                    )?;
                }
                for (action_id, remote_id, device_id) in &timed_out {
                    audit(
                        &tx,
                        "system",
                        "supervisor",
                        Some(remote_id),
                        Some(device_id),
                        "device_action_boot_reconciled",
                        Some(action_id),
                        None,
                        None,
                        "timeout",
                        None,
                        &json!({"reason": "approval_expired", "at": now}).to_string(),
                    )?;
                }
                for remote_id in &offline {
                    audit(
                        &tx,
                        "system",
                        "supervisor",
                        Some(remote_id),
                        None,
                        "remote_lease_expired",
                        None,
                        None,
                        None,
                        "offline",
                        None,
                        &json!({"reason": "boot_reconciliation", "at": now}).to_string(),
                    )?;
                }
                tx.commit()?;
                Ok(executing + expired + leases)
            })
            .map_err(map)
    }

    /// Requests a device action. A reversible fixture action executes now; a
    /// critical action waits for an explicit nonce-bound approval.
    ///
    /// # Errors
    /// Named/database refusal.
    pub fn request_action(
        &self,
        device_id: &str,
        capability_id: &str,
        action: &str,
        params_value: &Value,
        expected_value: Option<&Value>,
        idempotency_key: &str,
        requested_by_kind: &str,
        requested_by_id: &str,
    ) -> Result<DeviceAction, Refusal> {
        let device = self.get_device(device_id)?.ok_or(Refusal::DeviceUnknown)?;
        if device.revoked_at.is_some() {
            return Err(Refusal::DeviceUnknown);
        }
        let remote = self
            .get_remote(&device.remote_id)?
            .ok_or(Refusal::RemoteUnknown)?;
        if remote.revoked_at.is_some() {
            return Err(Refusal::RemoteUnknown);
        }
        if remote.state != "online" {
            return Err(Refusal::RemoteOffline);
        }
        if !matches!(
            requested_by_kind,
            "operator" | "task" | "remote"
        ) {
            return Err(Refusal::SchemaInvalid);
        }
        // Idempotent replay returns the recorded result and never re-actuates.
        let _action_guard = self
            .action_lock
            .lock()
            .map_err(|_| Refusal::InternalError)?;
        if let Some(existing) = self.find_action(device_id, idempotency_key)? {
            return Ok(existing);
        }
        let params_json =
            serde_json::to_string(params_value).map_err(|_| Refusal::SchemaInvalid)?;
        let params_sha256 = hash(&params_json);
        let expected_json = expected_value
            .map(serde_json::to_string)
            .transpose()
            .map_err(|_| Refusal::SchemaInvalid)?;
        let action_id = format!("action:{}", Uuid::now_v7());
        let now = Timestamp::now().as_str().to_owned();
        let refused = self
            .db
            .with_connection(|connection| {
                let tx = connection.transaction()?;
                // Re-check the control-plane authority inside the same
                // transaction that writes the action. The pre-check above can
                // race a revoke or lease expiry; this second read makes the
                // device mutation atomic with the decision.
                let now_text = Timestamp::now().as_str().to_owned();
                let remote_authority: Option<(String, Option<String>, Option<String>)> = tx
                    .query_row(
                        "SELECT state, lease_expires_at, revoked_at FROM remote_agents
                         WHERE remote_id = ?1",
                        params![device.remote_id],
                        |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
                    )
                    .optional()?;
                let Some((remote_state, lease_expires_at, remote_revoked)) = remote_authority
                else {
                    return Ok(Err(Refusal::RemoteUnknown));
                };
                if remote_revoked.is_some() {
                    return Ok(Err(Refusal::RemoteUnknown));
                }
                if remote_state != "online" {
                    return Ok(Err(Refusal::RemoteOffline));
                }
                if lease_expires_at
                    .as_deref()
                    .is_some_and(|lease| lease <= now_text.as_str())
                {
                    return Ok(Err(Refusal::RemoteOffline));
                }
                let device_row: Option<(String, String, String, Option<String>)> = tx
                    .query_row(
                        "SELECT class, state_json, safety_class, revoked_at FROM devices
                         WHERE device_id = ?1",
                        params![device_id],
                        |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
                    )
                    .optional()?;
                let Some((device_class, device_state_json, device_safety_class, device_revoked)) =
                    device_row
                else {
                    return Ok(Err(Refusal::DeviceUnknown));
                };
                if device_revoked.is_some() {
                    return Ok(Err(Refusal::DeviceUnknown));
                }
                let grant: Option<i64> = tx
                    .query_row(
                        "SELECT COUNT(*) FROM remote_grants
                         WHERE remote_id = ?1 AND device_id = ?2 AND capability_id = ?3
                           AND action = ?4 AND revoked_at IS NULL
                           AND (expires_at IS NULL OR expires_at > ?5)",
                        params![
                            device.remote_id,
                            device_id,
                            capability_id,
                            action,
                            Timestamp::now().as_str()
                        ],
                        |row| row.get(0),
                    )
                    .optional()?;
                let in_flight: i64 = tx.query_row(
                    "SELECT COUNT(*) FROM device_actions
                     WHERE device_id = ?1 AND status IN ('requested','awaiting_approval','executing')",
                    params![device_id],
                    |row| row.get(0),
                )?;
                if grant.unwrap_or(0) == 0 {
                    return Ok(Err(Refusal::RemoteCapability));
                }
                if in_flight > 0 {
                    return Ok(Err(Refusal::DeviceActionInFlight));
                }
                let status = if device_safety_class == "critical" {
                    "awaiting_approval"
                } else {
                    "confirmed"
                };
                let nonce = if status == "awaiting_approval" {
                    Some(format!("nonce:{}", Uuid::now_v7()))
                } else {
                    None
                };
                let expires = if status == "awaiting_approval" {
                    let Ok(now_secs) = Timestamp::now().unix_secs() else {
                        return Ok(Err(Refusal::InternalError));
                    };
                    Some(Timestamp::from_unix_secs(now_secs.saturating_add(120)).as_str().to_owned())
                } else {
                    None
                };
                let state_before = hash(&device_state_json);
                let state_after = if status == "confirmed" {
                    let Ok(after) =
                        fixture_apply(&device_class, action, &device_state_json)
                    else {
                        return Ok(Err(Refusal::RemoteCapability));
                    };
                    Some(after)
                } else {
                    None
                };
                tx.execute(
                    "INSERT INTO device_actions
                     (action_id, remote_id, device_id, capability_id, action, params_json,
                      params_sha256, expected_json, request_id, idempotency_key,
                      requested_by_kind, requested_by_id, approval_nonce,
                      approval_expires_at, status, state_before_hash, state_after_hash,
                      error_code, created_at, started_at, ended_at)
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12,
                             ?13, ?14, ?15, ?16, ?17, NULL, ?18, ?18, ?18)",
                    params![
                        action_id,
                        device.remote_id,
                        device_id,
                        capability_id,
                        action,
                        params_json,
                        params_sha256,
                        expected_json,
                        action_id,
                        idempotency_key,
                        requested_by_kind,
                        requested_by_id,
                        nonce,
                        expires,
                        status,
                        state_before,
                        state_after.as_ref().map(|state| hash(state)),
                        now,
                    ],
                )?;
                if let Some(after) = state_after {
                    tx.execute(
                        "UPDATE devices SET state_json = ?2, state_version = state_version + 1,
                         last_seen_at = ?3 WHERE device_id = ?1",
                        params![device_id, after, Timestamp::now().as_str()],
                    )?;
                }
                audit(
                    &tx,
                    requested_by_kind,
                    requested_by_id,
                    Some(&device.remote_id),
                    Some(device_id),
                    "device_action",
                    Some(&action_id),
                    Some(idempotency_key),
                    Some(&params_sha256),
                    status,
                    None,
                    "{}",
                )?;
                tx.commit()?;
                Ok(Ok(status))
            })
            .map_err(map)?;
        refused?;
        let action = self
            .get_action(&action_id)?
            .ok_or(Refusal::DeviceUnknown)?;
        Ok(action)
    }

    fn find_action(
        &self,
        device_id: &str,
        idempotency_key: &str,
    ) -> Result<Option<DeviceAction>, Refusal> {
        self.db
            .with_connection(|connection| {
                connection
                    .query_row(
                        &format!(
                            "{ACTION_SELECT} WHERE device_id = ?1 AND idempotency_key = ?2"
                        ),
                        params![device_id, idempotency_key],
                        action_from_row,
                    )
                    .optional()
            })
            .map_err(map)
    }

    /// Resolves a critical action exactly once.
    ///
    /// # Errors
    /// Named/database refusal.
    pub fn resolve_action(
        &self,
        action_id: &str,
        nonce: &str,
        approve: bool,
        actor: &str,
    ) -> Result<DeviceAction, Refusal> {
        let action = self
            .get_action(action_id)?
            .ok_or(Refusal::DeviceApprovalInvalid)?;
        if action.status != "awaiting_approval"
            || action.approval_nonce.as_deref() != Some(nonce)
        {
            return Err(Refusal::DeviceApprovalInvalid);
        }
        if let Some(expires) = &action.approval_expires_at {
            if expires <= &Timestamp::now().as_str().to_owned() {
                return Err(Refusal::DeviceApprovalInvalid);
            }
        }
        let _action_guard = self.action_lock.lock().map_err(|_| Refusal::InternalError)?;
        let refused = self.db
            .with_connection(|connection| {
                let tx = connection.transaction()?;
                let now = Timestamp::now().as_str().to_owned();
                // Revalidate the whole authority chain inside the transaction
                // that commits the resolution. The pre-check above can race a
                // revoke, a lease expiry or a sweep; this read makes the
                // decision atomic with the device mutation.
                let action_row: Option<(String, Option<String>, Option<String>)> = tx
                    .query_row(
                        "SELECT status, approval_nonce, approval_expires_at
                         FROM device_actions WHERE action_id = ?1",
                        params![action_id],
                        |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
                    )
                    .optional()?;
                let Some((status_now, nonce_now, expires_now)) = action_row else {
                    return Ok(Err(Refusal::DeviceApprovalInvalid));
                };
                if status_now != "awaiting_approval" || nonce_now.as_deref() != Some(nonce) {
                    return Ok(Err(Refusal::DeviceApprovalInvalid));
                }
                if expires_now
                    .as_deref()
                    .is_some_and(|expires| expires <= now.as_str())
                {
                    return Ok(Err(Refusal::DeviceApprovalInvalid));
                }
                let remote_row: Option<(String, Option<String>, Option<String>)> = tx
                    .query_row(
                        "SELECT state, lease_expires_at, revoked_at FROM remote_agents
                         WHERE remote_id = ?1",
                        params![action.remote_id],
                        |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
                    )
                    .optional()?;
                let Some((remote_state, lease_expires_at, remote_revoked)) = remote_row else {
                    return Ok(Err(Refusal::RemoteUnknown));
                };
                if remote_revoked.is_some() {
                    return Ok(Err(Refusal::RemoteUnknown));
                }
                if remote_state != "online" {
                    return Ok(Err(Refusal::RemoteOffline));
                }
                if lease_expires_at
                    .as_deref()
                    .is_some_and(|lease| lease <= now.as_str())
                {
                    return Ok(Err(Refusal::RemoteOffline));
                }
                let device_row: Option<(String, String, Option<String>)> = tx
                    .query_row(
                        "SELECT class, state_json, revoked_at FROM devices
                         WHERE device_id = ?1",
                        params![action.device_id],
                        |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
                    )
                    .optional()?;
                let Some((device_class, device_state_json, device_revoked)) = device_row else {
                    return Ok(Err(Refusal::DeviceUnknown));
                };
                if device_revoked.is_some() {
                    return Ok(Err(Refusal::DeviceUnknown));
                }
                let grant: i64 = tx.query_row(
                    "SELECT COUNT(*) FROM remote_grants
                     WHERE remote_id = ?1 AND device_id = ?2 AND capability_id = ?3
                       AND action = ?4 AND revoked_at IS NULL
                       AND (expires_at IS NULL OR expires_at > ?5)",
                    params![
                        action.remote_id,
                        action.device_id,
                        action.capability_id,
                        action.action,
                        now
                    ],
                    |row| row.get(0),
                )?;
                if grant == 0 {
                    return Ok(Err(Refusal::RemoteCapability));
                }
                let status = if approve { "confirmed" } else { "denied" };
                let state_after = if approve {
                    let Ok(after) =
                        fixture_apply(&device_class, &action.action, &device_state_json)
                    else {
                        return Ok(Err(Refusal::RemoteCapability));
                    };
                    after
                } else {
                    device_state_json.clone()
                };
                let changed = tx.execute(
                    "UPDATE device_actions SET status = ?2, approval_nonce = NULL,
                     state_after_hash = ?3, ended_at = ?4
                     WHERE action_id = ?1 AND status = 'awaiting_approval'
                       AND approval_nonce = ?5
                       AND (approval_expires_at IS NULL OR approval_expires_at > ?4)",
                    params![
                        action_id,
                        status,
                        hash(&state_after),
                        now,
                        nonce
                    ],
                )?;
                if changed != 1 {
                    return Ok(Err(Refusal::DeviceApprovalInvalid));
                }
                if approve {
                    tx.execute(
                        "UPDATE devices SET state_json = ?2, state_version = state_version + 1,
                         last_seen_at = ?3 WHERE device_id = ?1",
                        params![action.device_id, state_after, Timestamp::now().as_str()],
                    )?;
                }
                audit(
                    &tx,
                    "operator",
                    actor,
                    Some(&action.remote_id),
                    Some(&action.device_id),
                    "device_action_resolved",
                    Some(action_id),
                    Some(&action.idempotency_key),
                    None,
                    status,
                    None,
                    "{}",
                )?;
                tx.commit()?;
                Ok(Ok(()))
            })
            .map_err(map)?;
        refused?;
        self.get_action(action_id)?
            .ok_or(Refusal::DeviceApprovalInvalid)
    }

    /// Reads one action.
    ///
    /// # Errors
    /// Database refusal.
    pub fn get_action(&self, action_id: &str) -> Result<Option<DeviceAction>, Refusal> {
        self.db
            .with_connection(|connection| {
                connection
                    .query_row(
                        &format!("{ACTION_SELECT} WHERE action_id = ?1"),
                        params![action_id],
                        action_from_row,
                    )
                    .optional()
            })
            .map_err(map)
    }

    /// Records one HyperAgent control decision in the audit table even when the
    /// decision is not about a remote. This gives root gate actions an
    /// exportable audit row rather than only a memory-bank entry.
    ///
    /// # Errors
    /// Database refusal.
    pub fn record_control_decision(
        &self,
        actor_id: &str,
        _action: &str,
        decision: &str,
        detail_json: &str,
    ) -> Result<(), Refusal> {
        self.db
            .with_connection(|connection| {
                audit(
                    connection,
                    "system",
                    actor_id,
                    None,
                    None,
                    "root_control",
                    None,
                    None,
                    None,
                    decision,
                    None,
                    detail_json,
                )
            })
            .map_err(map)
    }

    /// Records one HyperAgent admission decision for a detached request. The
    /// remote row's own creation audit remains separate; this row names the
    /// approving gate and the caller's reason.
    ///
    /// # Errors
    /// Database refusal.
    pub fn record_admission(
        &self,
        remote_id: &str,
        actor_id: &str,
        detail_json: &str,
    ) -> Result<(), Refusal> {
        self.db
            .with_connection(|connection| {
                audit(
                    connection,
                    "system",
                    actor_id,
                    Some(remote_id),
                    None,
                    "hyper_admission",
                    None,
                    None,
                    None,
                    "ok",
                    None,
                    detail_json,
                )
            })
            .map_err(map)
    }

    /// Lists recent redacted audit rows.
    ///
    /// # Errors
    /// Database refusal.
    pub fn list_audit(&self, remote_id: Option<&str>, limit: usize) -> Result<Vec<RemoteAudit>, Refusal> {
        self.db
            .with_connection(|connection| {
                let mut statement = connection.prepare(
                    "SELECT at, actor_kind, actor_id, remote_id, device_id, action,
                            request_id, idempotency_key, outcome, refusal_code, detail_json
                     FROM remote_audit
                     WHERE (?1 IS NULL OR remote_id = ?1)
                     ORDER BY at DESC LIMIT ?2",
                )?;
                let rows = statement.query_map(
                    params![remote_id, i64::try_from(limit).unwrap_or(200)],
                    |row| {
                        Ok(RemoteAudit {
                            at: row.get(0)?,
                            actor_kind: row.get(1)?,
                            actor_id: row.get(2)?,
                            remote_id: row.get(3)?,
                            device_id: row.get(4)?,
                            action: row.get(5)?,
                            request_id: row.get(6)?,
                            idempotency_key: row.get(7)?,
                            outcome: row.get(8)?,
                            refusal_code: row.get(9)?,
                            detail_json: row.get(10)?,
                        })
                    },
                )?;
                let mut audit = Vec::new();
                for row in rows {
                    audit.push(row?);
                }
                Ok(audit)
            })
            .map_err(map)
    }
}

fn fixture_apply(class: &str, action: &str, current: &str) -> Result<String, Refusal> {
    let mut state: Value = serde_json::from_str(current).unwrap_or_else(|_| json!({}));
    match class {
        "light" | "switch" => {
            let next = match action {
                "on" => "on",
                "off" => "off",
                "toggle" => {
                    if state.get("power").and_then(Value::as_str) == Some("on") {
                        "off"
                    } else {
                        "on"
                    }
                }
                _ => return Err(Refusal::ToolNotAllowed),
            };
            state["power"] = json!(next);
        }
        "cover" => {
            let next = match action {
                "open" => "open",
                "close" => "closed",
                "stop" => "stopped",
                _ => return Err(Refusal::ToolNotAllowed),
            };
            state["position"] = json!(next);
        }
        "lock" => {
            let next = match action {
                "lock" => true,
                "unlock" => false,
                _ => return Err(Refusal::ToolNotAllowed),
            };
            state["locked"] = json!(next);
        }
        _ => return Err(Refusal::ToolNotAllowed),
    }
    serde_json::to_string(&state).map_err(|_| Refusal::InternalError)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn store() -> RemoteStore {
        store_with_db().0
    }

    fn store_with_db() -> (RemoteStore, Arc<Database>) {
        let db = Arc::new(Database::open_in_memory().expect("db"));
        db.migrate().expect("migrate");
        (RemoteStore::new(db.clone()), db)
    }

    fn remote_and_light(store: &RemoteStore) -> (RemoteAgent, Device) {
        let remote = store
            .create_remote(NewRemote {
                display_name: "Test remote".to_owned(),
                owner_kind: "operator".to_owned(),
                owner_id: "operator".to_owned(),
                transport_kind: "in_process".to_owned(),
                adapter_kind: "fixture".to_owned(),
                endpoint_ref: "fixture".to_owned(),
                credential_ref: None,
                capabilities_json: "[]".to_owned(),
                created_by: "operator".to_owned(),
                request_id: None,
                request_fingerprint: None,
            })
            .expect("remote");
        let device = store
            .create_device(NewDevice {                remote_id: remote.remote_id.clone(),
                class: "light".to_owned(),
                name: "Study light".to_owned(),
                room: Some("study".to_owned()),
                adapter_ref: "light.study".to_owned(),
                capabilities_json: r#"[{"capability_id":"power","actions":["on","off","toggle"]}]"#.to_owned(),
                safety_class: "reversible".to_owned(),
            })
            .expect("device");
        store
            .create_grant(NewGrant {
                remote_id: remote.remote_id.clone(),
                device_id: device.device_id.clone(),
                capability_id: "power".to_owned(),
                action: "toggle".to_owned(),
                constraints_json: "{}".to_owned(),
                expires_at: None,
            })
            .expect("grant");
        store
            .heartbeat(&remote.remote_id)
            .expect("fixture heartbeat");
        (remote, device)
    }

    #[test]
    fn remote_has_no_task_row_and_toggle_is_confirmed_once() {
        let (store, db) = store_with_db();
        let (_remote, device) = remote_and_light(&store);
        let action = store
            .request_action(
                &device.device_id,
                "power",
                "toggle",
                &json!({}),
                None,
                "idem-1",
                "operator",
                "operator",
            )
            .expect("action");
        assert_eq!(action.status, "confirmed");
        let after = store.get_device(&device.device_id).expect("device").expect("row");
        assert_eq!(after.state_json, r#"{"power":"on"}"#);
        let replay = store
            .request_action(
                &device.device_id,
                "power",
                "toggle",
                &json!({}),
                None,
                "idem-1",
                "operator",
                "operator",
            )
            .expect("replay");
        assert_eq!(replay.action_id, action.action_id);
        let task_count: i64 = db
            .with_connection(|connection| {
                connection.query_row("SELECT COUNT(*) FROM tasks", [], |row| row.get(0))
            })
            .expect("task count");
        assert_eq!(task_count, 0, "a remote must never become a task row");
    }

    #[test]
    fn boot_reconciliation_never_replays_an_executing_action() {
        let (store, db) = store_with_db();
        let (_remote, device) = remote_and_light(&store);
        let action = store
            .request_action(
                &device.device_id,
                "power",
                "toggle",
                &json!({}),
                None,
                "idem-boot",
                "operator",
                "operator",
            )
            .expect("action");
        db.with_connection(|connection| {
            connection.execute(
                "UPDATE device_actions SET status = 'executing' WHERE action_id = ?1",
                params![action.action_id],
            )?;
            Ok(())
        })
        .expect("mark executing");
        assert!(store.reconcile_on_boot().expect("reconcile") >= 1);
        let stored = store.get_action(&action.action_id).expect("action").expect("row");
        assert_eq!(stored.status, "interrupted");
        assert_eq!(
            store.reconcile_on_boot().expect("second reconcile"),
            0,
            "reconciliation is idempotent"
        );
    }

    #[test]
    fn critical_action_waits_for_nonce_and_never_replays() {
        let store = store();
        let (remote, _device) = remote_and_light(&store);
        let garage = store
            .create_device(NewDevice {
                remote_id: remote.remote_id.clone(),
                class: "cover".to_owned(),
                name: "Garage".to_owned(),
                room: Some("garage".to_owned()),
                adapter_ref: "cover.garage".to_owned(),
                capabilities_json: r#"[{"capability_id":"door","actions":["open","close"]}]"#.to_owned(),
                safety_class: "critical".to_owned(),
            })
            .expect("garage");
        store
            .create_grant(NewGrant {
                remote_id: remote.remote_id.clone(),
                device_id: garage.device_id.clone(),
                capability_id: "door".to_owned(),
                action: "open".to_owned(),
                constraints_json: "{}".to_owned(),
                expires_at: None,
            })
            .expect("grant");
        let action = store
            .request_action(
                &garage.device_id,
                "door",
                "open",
                &json!({}),
                None,
                "idem-garage",
                "operator",
                "operator",
            )
            .expect("action");
        assert_eq!(action.status, "awaiting_approval");
        let nonce = action.approval_nonce.clone().expect("nonce");
        let resolved = store
            .resolve_action(&action.action_id, &nonce, true, "operator")
            .expect("resolve");
        assert_eq!(resolved.status, "confirmed");
        assert_eq!(
            store
                .resolve_action(&action.action_id, &nonce, true, "operator"),
            Err(Refusal::DeviceApprovalInvalid)
        );
    }

    #[test]
    fn grant_is_required_and_revocation_blocks() {
        let store = store();
        let (remote, device) = remote_and_light(&store);
        store.revoke_remote(&remote.remote_id).expect("revoke");
        assert_eq!(
            store.request_action(
                &device.device_id,
                "power",
                "toggle",
                &json!({}),
                None,
                "idem-revoked",
                "operator",
                "operator",
            ),
            Err(Refusal::RemoteUnknown)
        );
    }
}
