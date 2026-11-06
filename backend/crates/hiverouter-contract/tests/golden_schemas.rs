//! Golden JSON-schema snapshots (blueprint WP 1.7: "one schema source of truth
//! via schemars derives + jsonschema validator; golden schemas snapshotted").
//!
//! Regenerate intentionally with:
//!   `$env:UPDATE_GOLDEN="1"; cargo test -p hiverouter-contract golden_schemas`
//! and record the change in `docs/claims/CLAIM-REGISTER.csv` / a change ledger.

mod common;

use hiverouter_contract::command::CommandName;
use hiverouter_contract::event::EventName;
use hiverouter_contract::schema::{self, SCHEMA_VERSION};
use serde_json::{json, Map, Value};

fn snapshot_path() -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("schemas")
        .join("contract-v1.json")
}

fn build_snapshot() -> Value {
    let mut commands = Map::new();
    for name in CommandName::ALL {
        if let Some(schema) = schema::command_payload_schema(*name) {
            commands.insert(
                name.as_str().to_owned(),
                schema.expect("command payload schema serializes"),
            );
        }
    }
    let mut events = Map::new();
    for name in EventName::ALL {
        if let Some(schema) = schema::event_payload_schema(*name) {
            events.insert(
                name.as_str().to_owned(),
                schema.expect("event payload schema serializes"),
            );
        }
    }
    json!({
        "contract": SCHEMA_VERSION,
        "envelope": schema::envelope_schema().expect("envelope schema serializes"),
        "commands": Value::Object(commands),
        "events": Value::Object(events),
    })
}

#[test]
fn golden_schemas() {
    let snapshot = build_snapshot();
    let path = snapshot_path();
    if std::env::var("UPDATE_GOLDEN").as_deref() == Ok("1") {
        std::fs::create_dir_all(path.parent().expect("schema dir")).expect("create schema dir");
        let pretty = serde_json::to_string_pretty(&snapshot).expect("pretty json");
        std::fs::write(&path, format!("{pretty}\n")).expect("write golden snapshot");
        return;
    }
    let text = std::fs::read_to_string(&path).unwrap_or_else(|error| {
        panic!(
            "golden schema snapshot missing at {}: {error}; run UPDATE_GOLDEN=1 cargo test -p hiverouter-contract golden_schemas",
            path.display()
        )
    });
    let expected: Value = serde_json::from_str(&text).expect("golden snapshot is valid JSON");
    assert_eq!(
        snapshot, expected,
        "contract schema drifted from the golden snapshot; update it deliberately and record a change-ledger row"
    );
}

#[test]
fn invalid_payloads_fail_the_jsonschema_validator() {
    let cases: &[(CommandName, Value)] = &[
        (CommandName::SubmitPrompt, json!({"project_id": "p"})),
        (
            CommandName::ViewportSet,
            json!({"x": 0.0, "y": 0.0, "zoom": "big"}),
        ),
        (CommandName::ChatPin, json!({"pinned": "not-a-bool"})),
        (
            CommandName::RewireTask,
            json!({"task_id": "019a0d5c-3333-7abc-8def-0123456789ab"}),
        ),
    ];
    for (name, payload) in cases {
        assert_eq!(
            schema::validate_command_payload(*name, payload),
            Err(hiverouter_contract::Refusal::SchemaInvalid),
            "{name:?}: invalid payload must be refused by the schema validator"
        );
    }
}

#[test]
fn event_payloads_validate_against_their_generated_schemas() {
    for event in common::event_samples() {
        let name = event.name();
        let payload = event.payload_value().expect("event payload serializes");
        schema::validate_event_payload(name, &payload)
            .unwrap_or_else(|error| panic!("{name:?}: sample event invalid: {error:?}"));
    }
    assert_eq!(
        schema::validate_event_payload(EventName::Ack, &json!({"command": "NOPE"})),
        Err(hiverouter_contract::Refusal::SchemaInvalid)
    );
}
