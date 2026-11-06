//! Phase 3 acceptance: typed handshake round-trip, depth derivation and
//! idempotent duplicate correlation ids.

use hiverouter_contract::{Refusal, RunId, TaskId};
use hiverouter_core::handshake::{
    ChildResult, ChildStatus, HandshakeBody, HandshakeFrame, HandshakeLog, HandshakeOutcome,
    MAX_HANDSHAKE_DEPTH,
};
use uuid::Uuid;

fn frame(depth: u8, with_parent: bool) -> HandshakeFrame {
    HandshakeFrame {
        run_id: RunId::new(),
        task_id: TaskId::new(),
        parent_task_id: if with_parent { Some(TaskId::new()) } else { None },
        depth,
        spawn_seq: 7,
        lease_seconds: 300,
        capability_set: vec!["file_read".to_owned()],
        correlation_id: Uuid::now_v7(),
        body: HandshakeBody::Join {
            capability_set: vec!["file_read".to_owned()],
        },
    }
}

#[test]
fn handshake_schema_roundtrip() {
    let original = frame(2, true);
    let json = serde_json::to_string(&original).expect("serialize handshake");
    let back: HandshakeFrame = serde_json::from_str(&json).expect("deserialize handshake");
    assert_eq!(back, original);
    assert!(json.contains("join"), "body tag must be stable: {json}");

    // Depth out of range is refused by name.
    let mut too_deep = frame(MAX_HANDSHAKE_DEPTH + 1, true);
    too_deep.spawn_seq = 8;
    assert_eq!(too_deep.validate(), Err(Refusal::SpawnDepth));

    // Deep frame without a parent is inconsistent.
    assert_eq!(frame(2, false).validate(), Err(Refusal::SchemaInvalid));

    // A well-formed result payload validates.
    let mut result = frame(1, false);
    result.body = HandshakeBody::Result {
        payload: serde_json::json!({"status": "completed", "text": "done"}),
    };
    assert_eq!(
        result.child_result().expect("child result"),
        ChildResult {
            status: ChildStatus::Completed,
            text: "done".to_owned()
        }
    );

    // Malformed child result => E_SCHEMA_INVALID (parent never gets fake data).
    result.body = HandshakeBody::Result {
        payload: serde_json::json!({"status": "ok", "output": 1}),
    };
    assert_eq!(result.child_result(), Err(Refusal::SchemaInvalid));

    // A non-result body cannot be treated as a child result.
    assert_eq!(frame(1, false).child_result(), Err(Refusal::SchemaInvalid));
}

#[test]
fn duplicate_correlation_id_is_idempotent() {
    let log = HandshakeLog::new(16);
    let original = frame(1, false);
    let first = log.accept(original.clone()).expect("first accept");
    assert!(matches!(first, HandshakeOutcome::Processed(_)));
    let replay = log.accept(original.clone()).expect("replay accept");
    match replay {
        HandshakeOutcome::Idempotent(previous) => assert_eq!(previous, original),
        other => panic!("expected idempotent outcome, got {other:?}"),
    }
    assert_eq!(log.len(), 1, "idempotent replay must not add a second entry");
}
