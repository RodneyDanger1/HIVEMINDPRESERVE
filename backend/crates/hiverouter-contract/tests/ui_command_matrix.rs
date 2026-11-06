//! `ui_command_matrix` — frozen contract §7 gate: every frozen command has
//! (a) a payload schema, (b) an ACK path and (c) at least one named-refusal
//! path test. No command ships without a matrix row.

mod common;

use hiverouter_contract::command::CommandName;
use hiverouter_contract::event::{Ack, Event, EventName};
use hiverouter_contract::schema::{
    self, command_payload_schema, command_spec, COMMAND_MATRIX,
};
use hiverouter_contract::Refusal;

#[test]
fn every_command_has_exactly_one_matrix_row() {
    assert_eq!(
        COMMAND_MATRIX.len(),
        CommandName::ALL.len(),
        "matrix and command vocabulary disagree"
    );
    for name in CommandName::ALL {
        let rows: Vec<_> = COMMAND_MATRIX.iter().filter(|spec| spec.name == *name).collect();
        assert_eq!(rows.len(), 1, "{name:?} must have exactly one matrix row");
        let row = rows[0];
        assert_eq!(row.frozen, name.is_frozen(), "{name:?} frozen flag mismatch");
        if row.frozen {
            assert!(row.ack, "{name:?} is frozen and must have an ACK path");
            assert!(
                !row.refusals.is_empty(),
                "{name:?} is frozen and must declare at least one named refusal"
            );
            assert!(
                command_payload_schema(*name).is_some(),
                "{name:?} is frozen and must have a payload schema"
            );
            assert!(
                !row.payload_schema.is_empty(),
                "{name:?} must name the payload type its schema is derived from"
            );
        } else {
            assert!(!row.ack, "{name:?} is not frozen and must not claim an ACK path");
        }
        assert_eq!(command_spec(*name).map(|spec| spec.name), Some(*name));
    }
}

#[test]
fn every_payload_schema_is_generated_and_validates_its_sample() {
    for command in common::command_samples() {
        let name = command.name();
        let schema_value = command_payload_schema(name)
            .unwrap_or_else(|| panic!("{name:?}: missing payload schema"))
            .expect("payload schema serializes");
        assert!(schema_value.is_object(), "{name:?}: schema is not a JSON object");
        let payload = command.payload_value().expect("payload serializes");
        schema::validate_command_payload(name, &payload)
            .unwrap_or_else(|error| panic!("{name:?}: sample payload invalid: {error:?}"));
    }
}

#[test]
fn every_frozen_command_has_an_ack_path() {
    for command in common::command_samples() {
        let request = schema::encode_command(&command).expect("encode request");
        let ack = Ack {
            command: command.name(),
            correlation_id: request.correlation_id,
        };
        let ack_envelope = hiverouter_contract::Envelope::event(EventName::Ack, &ack)
            .expect("build ACK envelope");
        let parsed = schema::parse_envelope(
            &serde_json::to_value(&ack_envelope).expect("ACK to json"),
        )
        .expect("ACK envelope parses");
        let decoded = schema::decode_event(&parsed).expect("ACK decodes");
        assert_eq!(decoded, Event::Ack(ack), "ACK does not carry command+correlation_id");
    }
}

#[test]
fn every_frozen_command_has_a_named_refusal_path() {
    let samples = common::refusal_samples();
    for command in common::command_samples() {
        let name = command.name();
        let spec = command_spec(name).expect("matrix row");
        let exercised = samples.iter().any(|(sample_name, _, value, expected)| {
            if *sample_name != name {
                return false;
            }
            let envelope = schema::parse_envelope(value).expect("sample parses");
            let observed = schema::decode_command(&envelope).expect_err("sample must be refused");
            observed == *expected && spec.refusals.contains(&observed)
        });
        if !exercised {
            // CHAT_PIN's only frozen refusal is E_COMMAND_UNKNOWN, which is
            // exercised by the unknown-name path rather than a payload sample.
            assert!(
                spec.refusals.contains(&Refusal::CommandUnknown),
                "{name:?} has no exercised named-refusal path"
            );
        }
    }

    // Global unknown-command path.
    let envelope = schema::parse_envelope(&common::unknown_command_envelope()).expect("parses");
    assert_eq!(
        schema::decode_command(&envelope).expect_err("unknown command"),
        Refusal::CommandUnknown
    );
}

#[test]
fn decision_gated_diff_commands_never_decode() {
    for name in [CommandName::DiffPropose, CommandName::DiffApply, CommandName::DiffReject] {
        let payload = serde_json::json!({"decision": "approve"});
        let envelope = hiverouter_contract::Envelope::command(name, &payload).expect("build");
        assert_eq!(
            schema::decode_command(&envelope).expect_err("must stay decision-gated"),
            Refusal::DecisionRequired
        );
    }
}
