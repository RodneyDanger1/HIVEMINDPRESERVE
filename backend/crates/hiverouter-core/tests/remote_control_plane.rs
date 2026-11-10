//! Integration tests for control-plane authority revalidation and supervisor
//! audits. These exercise the public `RemoteStore` facade only.

use std::sync::Arc;

use hiverouter_contract::Refusal;
use hiverouter_core::db::Database;
use hiverouter_core::remote::{Device, NewDevice, NewGrant, NewRemote, RemoteAgent, RemoteStore};
use rusqlite::params;

fn store() -> (RemoteStore, Arc<Database>) {
    let db = Arc::new(Database::open_in_memory().expect("db"));
    db.migrate().expect("migrate");
    (RemoteStore::new(db.clone()), db)
}

fn remote_light(store: &RemoteStore) -> (RemoteAgent, Device) {
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
        })
        .expect("remote");
    let device = store
        .create_device(NewDevice {
            remote_id: remote.remote_id.clone(),
            class: "light".to_owned(),
            name: "Study light".to_owned(),
            room: Some("study".to_owned()),
            adapter_ref: "light.study".to_owned(),
            capabilities_json:
                r#"[{"capability_id":"power","actions":["on","off","toggle"]}]"#.to_owned(),
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
    store.heartbeat(&remote.remote_id).expect("heartbeat");
    (remote, device)
}

fn critical_garage(store: &RemoteStore, remote: &RemoteAgent) -> Device {
    let device = store
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
            device_id: device.device_id.clone(),
            capability_id: "door".to_owned(),
            action: "open".to_owned(),
            constraints_json: "{}".to_owned(),
            expires_at: None,
        })
        .expect("grant");
    device
}

#[test]
fn resolve_rejects_a_remote_revoked_after_the_request() {
    let (store, _db) = store();
    let (remote, _light) = remote_light(&store);
    let garage = critical_garage(&store, &remote);
    let action = store
        .request_action(
            &garage.device_id,
            "door",
            "open",
            &serde_json::json!({}),
            None,
            "idem-revoke-race",
            "operator",
            "operator",
        )
        .expect("action");
    assert_eq!(action.status, "awaiting_approval");
    store.revoke_remote(&remote.remote_id).expect("revoke");
    let nonce = action.approval_nonce.clone().expect("nonce");
    assert_eq!(
        store.resolve_action(&action.action_id, &nonce, true, "operator"),
        Err(Refusal::RemoteUnknown),
        "resolution must revalidate remote revocation inside the transaction"
    );
    let stored = store
        .get_action(&action.action_id)
        .expect("action")
        .expect("row");
    assert_eq!(
        stored.status, "awaiting_approval",
        "a refused resolution must not mutate the action"
    );
}

#[test]
fn resolve_rejects_an_expired_lease_even_with_a_valid_nonce() {
    let (store, db) = store();
    let (remote, _light) = remote_light(&store);
    let garage = critical_garage(&store, &remote);
    let action = store
        .request_action(
            &garage.device_id,
            "door",
            "open",
            &serde_json::json!({}),
            None,
            "idem-lease-race",
            "operator",
            "operator",
        )
        .expect("action");
    db.with_connection(|connection| {
        connection.execute(
            "UPDATE remote_agents SET lease_expires_at = '1970-01-01T00:00:00Z'
             WHERE remote_id = ?1",
            params![remote.remote_id],
        )?;
        Ok(())
    })
    .expect("expire lease");
    let nonce = action.approval_nonce.clone().expect("nonce");
    assert_eq!(
        store.resolve_action(&action.action_id, &nonce, true, "operator"),
        Err(Refusal::RemoteOffline)
    );
    let device = store
        .get_device(&garage.device_id)
        .expect("device")
        .expect("row");
    assert!(
        !device.state_json.contains("\"open\""),
        "a refused resolution must not act on the device"
    );
}

#[test]
fn resolve_rejects_a_device_revoked_after_the_request() {
    let (store, db) = store();
    let (remote, _light) = remote_light(&store);
    let garage = critical_garage(&store, &remote);
    let action = store
        .request_action(
            &garage.device_id,
            "door",
            "open",
            &serde_json::json!({}),
            None,
            "idem-device-revoke",
            "operator",
            "operator",
        )
        .expect("action");
    db.with_connection(|connection| {
        connection.execute(
            "UPDATE devices SET revoked_at = '2026-01-01T00:00:00Z' WHERE device_id = ?1",
            params![garage.device_id],
        )?;
        Ok(())
    })
    .expect("revoke device");
    let nonce = action.approval_nonce.clone().expect("nonce");
    assert_eq!(
        store.resolve_action(&action.action_id, &nonce, true, "operator"),
        Err(Refusal::DeviceUnknown)
    );
}

#[test]
fn abandoned_approval_is_swept_and_unlocks_the_device() {
    let (store, db) = store();
    let (remote, _light) = remote_light(&store);
    let garage = critical_garage(&store, &remote);
    let first = store
        .request_action(
            &garage.device_id,
            "door",
            "open",
            &serde_json::json!({}),
            None,
            "idem-abandoned",
            "operator",
            "operator",
        )
        .expect("first action");
    assert_eq!(first.status, "awaiting_approval");
    // Simulate the approval window elapsing without a resolution.
    db.with_connection(|connection| {
        connection.execute(
            "UPDATE device_actions SET approval_expires_at = '1970-01-01T00:00:00Z'
             WHERE action_id = ?1",
            params![first.action_id],
        )?;
        Ok(())
    })
    .expect("age approval");
    assert_eq!(
        store
            .request_action(
                &garage.device_id,
                "door",
                "open",
                &serde_json::json!({}),
                None,
                "idem-blocked",
                "operator",
                "operator",
            ),
            Err(Refusal::DeviceActionInFlight),
        "the abandoned row still blocks the device before the sweep"
    );
    assert_eq!(store.expire_approvals().expect("sweep"), 1);
    let swept = store
        .get_action(&first.action_id)
        .expect("action")
        .expect("row");
    assert_eq!(swept.status, "timeout");
    let audit = store.list_audit(Some(&remote.remote_id), 50).expect("audit");
    assert!(
        audit.iter().any(|row| {
            row.actor_kind == "system"
                && row.actor_id == "supervisor"
                && row.action == "device_action_approval_expired"
                && row.outcome == "timeout"
        }),
        "the sweep must leave a supervisor audit row: {audit:?}"
    );
    let second = store
        .request_action(
            &garage.device_id,
            "door",
            "open",
            &serde_json::json!({}),
            None,
            "idem-after-sweep",
            "operator",
            "operator",
        )
        .expect("second action accepted after the sweep");
    assert_eq!(second.status, "awaiting_approval");
}

#[test]
fn lease_expiry_writes_a_supervisor_audit_row() {
    let (store, db) = store();
    let (remote, _light) = remote_light(&store);
    db.with_connection(|connection| {
        connection.execute(
            "UPDATE remote_agents SET lease_expires_at = '1970-01-01T00:00:00Z'
             WHERE remote_id = ?1",
            params![remote.remote_id],
        )?;
        Ok(())
    })
    .expect("expire lease");
    assert_eq!(store.expire_leases().expect("expire"), 1);
    let audit = store.list_audit(Some(&remote.remote_id), 50).expect("audit");
    assert!(
        audit.iter().any(|row| {
            row.actor_kind == "system"
                && row.action == "remote_lease_expired"
                && row.outcome == "offline"
        }),
        "lease expiry must be audited transactionally: {audit:?}"
    );
    assert_eq!(
        store.expire_leases().expect("idempotent"),
        0,
        "an already-offline remote is not re-audited"
    );
}

#[test]
fn boot_reconciliation_writes_a_supervisor_audit_row() {
    let (store, db) = store();
    let (_remote, device) = remote_light(&store);
    let action = store
        .request_action(
            &device.device_id,
            "power",
            "toggle",
            &serde_json::json!({}),
            None,
            "idem-boot-audit",
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
    let stored = store
        .get_action(&action.action_id)
        .expect("action")
        .expect("row");
    assert_eq!(stored.status, "interrupted");
    let audit = store.list_audit(None, 50).expect("audit");
    assert!(
        audit.iter().any(|row| {
            row.actor_kind == "system"
                && row.action == "device_action_boot_reconciled"
                && row.outcome == "interrupted"
                && row.request_id.as_deref() == Some(action.action_id.as_str())
        }),
        "boot reconciliation must be audited: {audit:?}"
    );
    assert_eq!(
        store.reconcile_on_boot().expect("idempotent"),
        0,
        "reconciliation never replays an action"
    );
}

#[test]
fn a_second_reversible_action_applies_on_the_committed_state() {
    let (store, _db) = store();
    let (_remote, device) = remote_light(&store);
    let first = store
        .request_action(
            &device.device_id,
            "power",
            "toggle",
            &serde_json::json!({}),
            None,
            "idem-toggle-1",
            "operator",
            "operator",
        )
        .expect("first toggle");
    assert_eq!(first.status, "confirmed");
    let second = store
        .request_action(
            &device.device_id,
            "power",
            "toggle",
            &serde_json::json!({}),
            None,
            "idem-toggle-2",
            "operator",
            "operator",
        )
        .expect("second toggle");
    assert_eq!(second.status, "confirmed");
    let after = store
        .get_device(&device.device_id)
        .expect("device")
        .expect("row");
    assert_eq!(
        after.state_json, r#"{"power":"off"}"#,
        "the second action must be computed from the state committed by the first"
    );
    assert!(after.state_version >= 2);
}
