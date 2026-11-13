//! In-process approval waiters.
//!
//! One run can have at most one live approval wait per task turn. The engine
//! arms a waiter before publishing `APPROVAL_REQUEST`; the WebSocket resolve arm
//! wakes it after the decision is committed. This module only owns the
//! map/sender lifecycle; it never reads or mutates approval database state.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use hiverouter_contract::{ApprovalId, Refusal};
use hiverouter_tools::approval::ApprovalResolution;
use tokio::sync::oneshot;

/// The single wake value a suspended tool turn can receive.
#[derive(Debug)]
pub enum ApprovalWake {
    /// The decision was committed; the engine resumes from this resolution.
    Resolved(ApprovalResolution),
    /// The run/task was cancelled before a decision.
    Cancelled,
}

/// Thread-safe map of approval id to a one-shot wake sender.
#[derive(Clone, Debug, Default)]
pub struct ApprovalWaiters {
    inner: Arc<Mutex<HashMap<String, oneshot::Sender<ApprovalWake>>>>,
}

impl ApprovalWaiters {
    /// Creates an empty waiter map.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Arms a waiter for an approval before `APPROVAL_REQUEST` is published.
    ///
    /// # Errors
    /// `E_INTERNAL_ERROR` when the map mutex is poisoned; `E_APPROVAL_REPLAY`
    /// when the same approval id already has a live waiter (a second turn must
    /// never steal the first turn's wake).
    pub fn arm(&self, approval_id: ApprovalId) -> Result<oneshot::Receiver<ApprovalWake>, Refusal> {
        let mut map = self.inner.lock().map_err(|_| Refusal::InternalError)?;
        let key = approval_id.to_string();
        if map.contains_key(&key) {
            return Err(Refusal::ApprovalReplay);
        }
        let (sender, receiver) = oneshot::channel();
        map.insert(key, sender);
        Ok(receiver)
    }

    /// Wakes the waiter for a committed resolution. Returns `true` when a live
    /// waiter was present and accepted the wake; `false` when it was already
    /// disarmed or its receiver was dropped.
    pub fn wake_resolved(&self, resolution: &ApprovalResolution) -> bool {
        let key = resolution.request.approval_id.to_string();
        self.take_and_send(&key, ApprovalWake::Resolved(resolution.clone()))
    }

    /// Wakes the waiter for a cancelled run/task.
    pub fn wake_cancelled(&self, approval_id: ApprovalId) -> bool {
        self.take_and_send(&approval_id.to_string(), ApprovalWake::Cancelled)
    }

    /// Removes a waiter without waking it (owner exited its wait for another
    /// reason). Returns `true` when a waiter was present.
    pub fn disarm(&self, approval_id: ApprovalId) -> bool {
        self.inner
            .lock()
            .map(|mut map| map.remove(&approval_id.to_string()).is_some())
            .unwrap_or(false)
    }

    /// Number of currently armed waiters.
    #[must_use]
    pub fn len(&self) -> usize {
        self.inner.lock().map(|map| map.len()).unwrap_or(0)
    }

    /// Whether no waiter is currently armed.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    fn take_and_send(&self, key: &str, wake: ApprovalWake) -> bool {
        let sender = self
            .inner
            .lock()
            .ok()
            .and_then(|mut map| map.remove(key));
        sender.is_some_and(|sender| sender.send(wake).is_ok())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use hiverouter_contract::{RunId, TaskId, Timestamp};
    use hiverouter_tools::approval::{ApprovalGate, ApprovalRequest, ApprovalResolution};
    use hiverouter_tools::tool::ToolId;
    use hiverouter_tools::{ApprovalDecision, PermissionClass};

    fn resolution(id: ApprovalId) -> ApprovalResolution {
        ApprovalResolution {
            request: ApprovalRequest {
                approval_id: id,
                run_id: RunId::new(),
                task_id: TaskId::new(),
                tool_call_id: "call-1".to_owned(),
                tool: ToolId::FileWrite,
                permission_class: PermissionClass::Mutate,
                args_sha256: "a".repeat(64),
                args_preview: None,
                created_at: Timestamp::now(),
                expires_at: Timestamp::now(),
                nonce: "nonce-1".to_owned(),
            },
            decision: ApprovalDecision::AllowOnce,
            decided_at: Timestamp::now(),
        }
    }

    #[test]
    fn arm_then_resolution_wakes_exactly_once() {
        let waiters = ApprovalWaiters::new();
        let id = ApprovalId::new();
        let mut receiver = waiters.arm(id).expect("arm");
        assert_eq!(waiters.len(), 1);
        let resolution = resolution(id);
        assert!(waiters.wake_resolved(&resolution));
        assert!(!waiters.wake_resolved(&resolution), "second wake is false");
        assert!(waiters.is_empty());
        let wake = receiver.try_recv().expect("wake delivered");
        assert!(matches!(wake, ApprovalWake::Resolved(_)));
    }

    #[test]
    fn duplicate_arm_is_refused_and_cancel_disarms() {
        let waiters = ApprovalWaiters::new();
        let id = ApprovalId::new();
        let _receiver = waiters.arm(id).expect("first arm");
        assert_eq!(
            waiters.arm(id).err(),
            Some(Refusal::ApprovalReplay),
            "a second turn cannot steal the first wake"
        );
        assert!(waiters.wake_cancelled(id));
        assert!(!waiters.disarm(id), "already consumed");
        assert!(waiters.is_empty());
    }

    #[test]
    fn disarm_removes_without_waking() {
        let waiters = ApprovalWaiters::new();
        let id = ApprovalId::new();
        let receiver = waiters.arm(id).expect("arm");
        assert!(waiters.disarm(id));
        assert!(receiver.is_empty());
        assert!(waiters.is_empty());
    }

    #[test]
    fn approval_gate_resolution_shape_is_hashable_by_id() {
        // The waiter layer keys by the committed approval id; this test pins the
        // exact field the WS wake path will use.
        let id = ApprovalId::new();
        let resolution = resolution(id);
        assert_eq!(resolution.request.approval_id.to_string(), id.to_string());
        let _ = ApprovalGate::default();
    }
}

