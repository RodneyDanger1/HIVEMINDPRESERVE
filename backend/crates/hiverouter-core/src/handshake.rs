//! Typed orchestration handshake (blueprint Phase 3, WP 3.5).
//!
//! `HandshakeFrame` is carried by the single Phase 2 transport (no second
//! socket). Depth / spawn sequence / lease are derived from the parent's stored
//! row by the caller (never by the model), and duplicate correlation ids are
//! idempotent: a replayed frame returns the previously processed frame instead
//! of running the handshake twice.

use std::collections::HashMap;
use std::sync::Mutex;

use hiverouter_contract::ids::{RunId, TaskId};
use hiverouter_contract::Refusal;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

/// Maximum spawn depth (Hyper -> Super -> Worker), blueprint §2a.
pub const MAX_HANDSHAKE_DEPTH: u8 = 3;
/// Lease duration handed to a child (seconds).
pub const HANDSHAKE_LEASE_SECONDS: u64 = 300;

/// Body of a handshake frame.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum HandshakeBody {
    /// Child announces itself with its capability set.
    Join {
        /// Tool ids the child may use.
        capability_set: Vec<String>,
    },
    /// Parent acknowledges a join.
    Ack {
        /// Whether the join was accepted.
        accepted: bool,
    },
    /// Child reports its result.
    Result {
        /// Child result body, validated by [`ChildResult`].
        payload: Value,
    },
    /// Child reports a named refusal.
    Refuse {
        /// Stable refusal code.
        code: String,
        /// Human-readable message.
        message: String,
    },
}

/// One typed handshake frame.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct HandshakeFrame {
    /// Run the handshake belongs to.
    pub run_id: RunId,
    /// Child task.
    pub task_id: TaskId,
    /// Parent task (`None` only for depth 1).
    pub parent_task_id: Option<TaskId>,
    /// Spawn depth, 1..=3.
    pub depth: u8,
    /// Monotonic spawn sequence within the run.
    pub spawn_seq: u64,
    /// Lease duration in seconds.
    pub lease_seconds: u64,
    /// Tool capability set of the child.
    pub capability_set: Vec<String>,
    /// Correlation id shared with the spawn command.
    pub correlation_id: Uuid,
    /// Handshake body.
    pub body: HandshakeBody,
}

impl HandshakeFrame {
    /// Validates the frame invariants.
    ///
    /// # Errors
    /// `E_SPAWN_DEPTH` for depth outside 1..=3, `E_SCHEMA_INVALID` for
    /// inconsistent parent/lease fields.
    pub fn validate(&self) -> Result<(), Refusal> {
        if self.depth == 0 || self.depth > MAX_HANDSHAKE_DEPTH {
            return Err(Refusal::SpawnDepth);
        }
        if self.lease_seconds == 0 || self.lease_seconds > 86_400 {
            return Err(Refusal::SchemaInvalid);
        }
        if self.depth > 1 && self.parent_task_id.is_none() {
            return Err(Refusal::SchemaInvalid);
        }
        if self.correlation_id.is_nil() {
            return Err(Refusal::SchemaInvalid);
        }
        Ok(())
    }

    /// Parses and validates a child result payload.
    ///
    /// # Errors
    /// `E_SCHEMA_INVALID` when the frame is not a `Result` or the payload does
    /// not match [`ChildResult`].
    pub fn child_result(&self) -> Result<ChildResult, Refusal> {
        match &self.body {
            HandshakeBody::Result { payload } => {
                serde_json::from_value(payload.clone()).map_err(|_| Refusal::SchemaInvalid)
            }
            _ => Err(Refusal::SchemaInvalid),
        }
    }
}

/// Terminal status reported by a child.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ChildStatus {
    /// Child finished successfully.
    Completed,
    /// Child failed (the failure is reported, never hidden).
    Failed,
}

/// Validated child result payload.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChildResult {
    /// Terminal status.
    pub status: ChildStatus,
    /// Result text / summary.
    pub text: String,
}

/// Outcome of accepting a handshake frame.
#[derive(Clone, Debug, PartialEq)]
pub enum HandshakeOutcome {
    /// Frame is new and must be processed.
    Processed(HandshakeFrame),
    /// Frame is a replay of an already processed correlation id; the previous
    /// frame is returned and the handshake is not executed again.
    Idempotent(HandshakeFrame),
}

/// Bounded log of processed handshake correlation ids (idempotency guard).
#[derive(Debug)]
pub struct HandshakeLog {
    capacity: usize,
    seen: Mutex<HashMap<Uuid, HandshakeFrame>>,
}

impl HandshakeLog {
    /// Creates a log with the given capacity (oldest ids forgotten when full
    /// in insertion order via the replay-order vector).
    #[must_use]
    pub fn new(capacity: usize) -> Self {
        Self {
            capacity: capacity.max(1),
            seen: Mutex::new(HashMap::new()),
        }
    }

    /// Accepts a frame, enforcing validation + correlation-id idempotency.
    ///
    /// # Errors
    /// The frame's validation refusal, or `E_INTERNAL_ERROR` if the log lock is
    /// poisoned.
    pub fn accept(&self, frame: HandshakeFrame) -> Result<HandshakeOutcome, Refusal> {
        frame.validate()?;
        let mut seen = self.seen.lock().map_err(|_| Refusal::InternalError)?;
        if let Some(previous) = seen.get(&frame.correlation_id) {
            return Ok(HandshakeOutcome::Idempotent(previous.clone()));
        }
        if seen.len() >= self.capacity {
            // Deterministic eviction: drop the entry with the smallest uuid v7
            // (oldest by time order).
            if let Some(oldest) = seen.keys().min().copied() {
                seen.remove(&oldest);
            }
        }
        seen.insert(frame.correlation_id, frame.clone());
        Ok(HandshakeOutcome::Processed(frame))
    }

    /// Number of remembered correlation ids.
    #[must_use]
    pub fn len(&self) -> usize {
        self.seen.lock().map(|seen| seen.len()).unwrap_or(0)
    }

    /// Whether the log is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn frame(depth: u8, parent: bool) -> HandshakeFrame {
        HandshakeFrame {
            run_id: RunId::new(),
            task_id: TaskId::new(),
            parent_task_id: if parent { Some(TaskId::new()) } else { None },
            depth,
            spawn_seq: 1,
            lease_seconds: HANDSHAKE_LEASE_SECONDS,
            capability_set: vec!["file_read".to_owned()],
            correlation_id: Uuid::now_v7(),
            body: HandshakeBody::Join {
                capability_set: vec!["file_read".to_owned()],
            },
        }
    }

    #[test]
    fn depth_is_capped_at_three() {
        assert_eq!(frame(1, false).validate(), Ok(()));
        assert_eq!(frame(3, true).validate(), Ok(()));
        assert_eq!(frame(4, true).validate(), Err(Refusal::SpawnDepth));
        assert_eq!(frame(0, false).validate(), Err(Refusal::SpawnDepth));
    }

    #[test]
    fn deep_frame_requires_a_parent() {
        assert_eq!(frame(2, false).validate(), Err(Refusal::SchemaInvalid));
    }

    #[test]
    fn duplicate_correlation_is_idempotent() {
        let log = HandshakeLog::new(8);
        let original = frame(1, false);
        let first = log.accept(original.clone()).expect("first");
        assert!(matches!(first, HandshakeOutcome::Processed(_)));
        let replay = log.accept(original.clone()).expect("replay");
        match replay {
            HandshakeOutcome::Idempotent(previous) => assert_eq!(previous, original),
            other => panic!("expected idempotent, got {other:?}"),
        }
        assert_eq!(log.len(), 1);
    }

    #[test]
    fn malformed_child_result_is_refused() {
        let mut result_frame = frame(1, false);
        result_frame.body = HandshakeBody::Result {
            payload: serde_json::json!({"status": "done"}),
        };
        assert_eq!(result_frame.child_result(), Err(Refusal::SchemaInvalid));

        result_frame.body = HandshakeBody::Result {
            payload: serde_json::json!({"status": "completed", "text": "ok"}),
        };
        let parsed = result_frame.child_result().expect("valid result");
        assert_eq!(parsed.status, ChildStatus::Completed);
        assert_eq!(parsed.text, "ok");
    }
}
