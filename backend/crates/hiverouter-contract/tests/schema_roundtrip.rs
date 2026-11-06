//! `contract_schema_roundtrip` — every command and event fixture serializes,
//! deserializes and re-decodes losslessly (frozen contract §7).

mod common;

use hiverouter_contract::command::CommandName;
use hiverouter_contract::event::EventName;
use hiverouter_contract::ids::RunId;
use hiverouter_contract::schema::{self, SCHEMA_VERSION};
use hiverouter_contract::{FrameKind, Refusal};

#[test]
fn contract_schema_roundtrip() {
    for command in common::command_samples() {
        let envelope = schema::encode_command(&command).expect("encode command");
        assert_eq!(envelope.schema, SCHEMA_VERSION);
        assert_eq!(envelope.frame_type, FrameKind::Command);
        assert_eq!(envelope.name, command.name().as_str());
        assert!(envelope.seq.is_none(), "commands carry no server seq");
        assert!(envelope.ts.as_str().ends_with('Z'), "ts is canonical UTC");
        assert_eq!(envelope.id, envelope.correlation_id);

        let json = envelope.to_json().expect("serialize command");
        let parsed = schema::parse_envelope_json(&json).expect("parse command");
        let decoded = schema::decode_command(&parsed).expect("decode command");
        assert_eq!(decoded, command, "command roundtrip changed the value");
    }

    for event in common::event_samples() {
        let envelope = schema::encode_event(&event).expect("encode event");
        assert_eq!(envelope.frame_type, FrameKind::Event);
        assert_eq!(envelope.name, event.name().as_str());
        let json = envelope.to_json().expect("serialize event");
        let parsed = schema::parse_envelope_json(&json).expect("parse event");
        let decoded = schema::decode_event(&parsed).expect("decode event");
        assert_eq!(decoded, event, "event roundtrip changed the value");
    }
}

#[test]
fn frozen_envelope_fixture_decodes_and_keeps_wire_field_names() {
    let fixture = common::frozen_command_envelope();
    let envelope = schema::parse_envelope(&fixture).expect("frozen fixture parses");
    schema::validate_envelope(&envelope).expect("frozen fixture validates");

    let serialized = serde_json::to_value(&envelope).expect("serialize envelope");
    for field in [
        "schema",
        "type",
        "name",
        "id",
        "correlation_id",
        "run_id",
        "task_id",
        "seq",
        "ts",
        "payload",
    ] {
        assert!(serialized.get(field).is_some(), "envelope is missing `{field}`");
    }
    assert_eq!(serialized["type"], "COMMAND");
    assert_eq!(serialized["schema"], SCHEMA_VERSION);
    assert_eq!(serialized["ts"], "2026-11-06T09:15:41Z");

    let command = schema::decode_command(&envelope).expect("decodes");
    assert_eq!(command.name(), CommandName::SubmitPrompt);
}

#[test]
fn bad_frames_are_refused_by_name_never_dropped() {
    assert_eq!(
        schema::parse_envelope_json("{not json").expect_err("invalid json"),
        Refusal::BadFrame
    );
    assert_eq!(
        schema::parse_envelope_json("[]").expect_err("not an object"),
        Refusal::SchemaInvalid
    );

    // Wrong schema version.
    let mut value = common::frozen_command_envelope();
    value["schema"] = serde_json::json!("hiverouter.v2");
    let envelope = schema::parse_envelope(&value).expect("shape parses");
    assert_eq!(
        schema::decode_command(&envelope).expect_err("schema mismatch refused"),
        Refusal::SchemaInvalid
    );

    // Unknown command name.
    let envelope =
        schema::parse_envelope(&common::unknown_command_envelope()).expect("shape parses");
    assert_eq!(
        schema::decode_command(&envelope).expect_err("unknown refused"),
        Refusal::CommandUnknown
    );

    // Server seq must not appear on a command.
    let mut value = common::frozen_command_envelope();
    value["seq"] = serde_json::json!(7);
    let envelope = schema::parse_envelope(&value).expect("shape parses");
    assert_eq!(
        schema::decode_command(&envelope).expect_err("seq refused"),
        Refusal::SchemaInvalid
    );

    // Wrong direction: an EVENT envelope cannot decode as a command.
    let mut value = common::frozen_command_envelope();
    value["type"] = serde_json::json!("EVENT");
    let envelope = schema::parse_envelope(&value).expect("shape parses");
    assert_eq!(
        schema::decode_command(&envelope).expect_err("direction refused"),
        Refusal::SchemaInvalid
    );

    // Unknown event name.
    let mut event_value =
        serde_json::to_value(schema::encode_event(&common::event_samples()[0]).expect("encode"))
            .expect("value");
    event_value["name"] = serde_json::json!("NOT_AN_EVENT");
    let envelope = schema::parse_envelope(&event_value).expect("shape parses");
    assert_eq!(
        schema::decode_event(&envelope).expect_err("unknown event refused"),
        Refusal::SchemaInvalid
    );
}

#[test]
fn refusal_samples_return_their_named_code() {
    for (name, label, value, expected) in common::refusal_samples() {
        let envelope = schema::parse_envelope(&value).unwrap_or_else(|error| {
            panic!("{label}: fixture envelope must parse, got {error:?}")
        });
        let observed = schema::decode_command(&envelope).expect_err(&format!(
            "{label}: expected refusal {:?} for {name:?}", expected
        ));
        assert_eq!(observed, expected, "{label}");
        assert_eq!(observed.code(), expected.code());
        assert!(!observed.how_to_fix().is_empty(), "{label}: how_to_fix is empty");
    }
}

#[test]
fn ids_are_uuid_v7_and_roundtrip_on_the_wire() {
    let first = RunId::new();
    let second = RunId::new();
    assert_ne!(first, second);
    assert_eq!(first.as_uuid().get_version_num(), 7);
    assert_eq!(first.to_string().parse::<RunId>().expect("parse"), first);
    let wire = serde_json::to_string(&first).expect("serialize id");
    assert!(wire.starts_with('"') && wire.ends_with('"'));
    assert!(wire.contains('-'), "wire ids are hyphenated uuid strings");
    let back: RunId = serde_json::from_str(&wire).expect("deserialize id");
    assert_eq!(back, first);
}

#[test]
fn event_and_command_name_sets_are_stable() {
    assert_eq!(CommandName::ALL.len(), 17);
    assert_eq!(EventName::ALL.len(), 21);
    let mut names: Vec<&str> = CommandName::ALL.iter().map(CommandName::as_str).collect();
    names.sort_unstable();
    names.dedup();
    assert_eq!(names.len(), 17, "duplicate command wire names");
    let mut events: Vec<&str> = EventName::ALL.iter().map(EventName::as_str).collect();
    events.sort_unstable();
    events.dedup();
    assert_eq!(events.len(), 21, "duplicate event wire names");
}
